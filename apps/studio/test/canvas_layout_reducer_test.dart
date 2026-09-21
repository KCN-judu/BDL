/// The reducer's side of edges as objects and of the whole-graph
/// arrangement: one disconnect for every gesture, refused where the model
/// has none; _Arrange Automatically_ as an asked-for layout that is
/// applied when it arrives (layout only, no edit, no revision), kept for
/// one _Undo Arrange_, dropped by the designer's next move; the canvas
/// told to frame once; and the connection inspector's words and button.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/inspector.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'canvas_selection_test.dart' show design, c0, m0, m1, m2, o0, rule, value, tilt, level;
import 'outputs_test.dart' show Harness, HarnessState;

AppState connected({Selection selection = const NoSelection(), CanvasLayout? layouts}) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: design(),
  editor: EditorState(
    selection: selection,
    layouts: layouts ?? const CanvasLayout(),
    layout: layouts?.system.nodes ?? const {},
  ),
);

pb.EditOp opOf(Transition t) => t.effects.whereType<ApplyEdit>().single.op;

const readEdge = LinkId(from: m0, to: m1, concept: tilt);
const driveEdge = LinkId(from: m2, to: o0, concept: level);
const aggregateEdge = LinkId(from: NodeRef.group(7), to: o0, concept: level);

final authored = CanvasLayout(
  system: ContextLayout(
    nodes: {c0: const Offset(1, 1), m1: const Offset(2, 2)},
    viewport: const CanvasViewport(pan: Offset(9, 9), zoom: 0.75),
  ),
);

pb.Layout arranged() => pb.Layout(
  concepts: [pb.NodePosition(id: Int64(tilt), x: 100, y: 100)],
  mappings: [pb.NodePosition(id: Int64(rule), x: 400, y: 100)],
  outputs: [pb.NodePosition(id: Int64(0), x: 700, y: 100)],
);

void main() {
  group('disconnect', () {
    test('a drive edge is the drive set to none; a read edge is a formula and is refused', () {
      final drive = reduce(connected(), const DisconnectLinkRequested(driveEdge));
      final d = opOf(drive).setMappingDrive;
      expect(d.id.toInt(), value);
      expect(d.hasOutputId(), isFalse);
      // a read edge is a name in the reading block's formula (ADR-0043):
      // no signature edit, no edit at all
      expect(readEdge.disconnectable, isFalse);
      expect(reduce(connected(), const DisconnectLinkRequested(readEdge)).effects, isEmpty);
    });

    test('a collapsed group\'s edge is refused: no edit, nothing implied', () {
      expect(aggregateEdge.disconnectable, isFalse);
      expect(reduce(connected(), const DisconnectLinkRequested(aggregateEdge)).effects, isEmpty);
    });

    test('Delete on a selected edge is that disconnect, and the selection goes with the edge', () {
      final s = connected(selection: const LinkSelected(driveEdge));
      final t = reduce(s, const DeleteSelectionRequested());
      expect(opOf(t).setMappingDrive.id.toInt(), value);
      expect(t.state.editor.selection, const NoSelection());
      // a refused one leaves the selection alone
      final kept = reduce(
        connected(selection: const LinkSelected(readEdge)),
        const DeleteSelectionRequested(),
      );
      expect(kept.effects, isEmpty);
      expect(kept.state.editor.selection, const LinkSelected(readEdge));
    });
  });

  group('arrange', () {
    test('asked for: one request, no edit, nothing applied yet', () {
      final t = reduce(connected(layouts: authored), const AutoLayoutRequested());
      expect(t.effects, [isA<ArrangeLayout>()]);
      expect(t.state.editor.layouts, authored);
      expect(t.state.editor.frameRequest, 0);
    });

    test('arrived: the layout, kept viewport, one SetLayout, one frame; the old layout kept', () {
      final s = connected(layouts: authored);
      final t = reduce(s, ArrangedLayoutReceived(arranged()));
      final l = t.state.editor.layouts.system;
      expect(l.nodes[c0], const Offset(100, 100));
      expect(l.nodes[m1], const Offset(400, 100));
      expect(l.nodes[o0], const Offset(700, 100));
      expect(l.viewport, authored.system.viewport, reason: 'the designer\'s view is theirs');
      expect(t.state.editor.layout, l.nodes, reason: 'the canvas on screen follows');
      expect(t.state.editor.layoutBefore, authored);
      expect(t.state.editor.frameRequest, 1);
      expect(t.effects.whereType<SetLayout>().length, 1);
      expect(t.effects.whereType<ApplyEdit>(), isEmpty, reason: 'layout is never a revision');
      expect(t.state.project!.revision, s.project!.revision);
    });

    test('Undo Arrange puts the layout back, once', () {
      final s = reduce(connected(layouts: authored), ArrangedLayoutReceived(arranged())).state;
      final t = reduce(s, const RestoreLayoutRequested());
      expect(t.state.editor.layouts, authored);
      expect(t.state.editor.layout, authored.system.nodes);
      expect(t.state.editor.layoutBefore, isNull);
      expect(t.state.editor.frameRequest, 2);
      expect(t.effects.whereType<SetLayout>().length, 1);
      // nothing to put back a second time
      expect(reduce(t.state, const RestoreLayoutRequested()).effects, isEmpty);
    });

    test('the designer\'s next move drops what was kept', () {
      final s = reduce(connected(layouts: authored), ArrangedLayoutReceived(arranged())).state;
      final t = reduce(s, const NodeMoved(m1, Offset(50, 50)));
      expect(t.state.editor.layoutBefore, isNull);
      expect(t.state.editor.frameRequest, 1, reason: 'a move never frames');
    });

    test('the same arrangement twice is the same state: nothing depends on order or time', () {
      final a = reduce(connected(layouts: authored), ArrangedLayoutReceived(arranged())).state;
      final b = reduce(connected(layouts: authored), ArrangedLayoutReceived(arranged())).state;
      expect(a.editor.layouts.system, b.editor.layouts.system);
      expect(a.editor.layout, b.editor.layout);
    });
  });

  group('inspector', () {
    testWidgets('a selected edge: its ends, what it means, and Disconnect where it can', (t) async {
      final key = GlobalKey<HarnessState>();
      await t.pumpWidget(
        Harness(
          key: key,
          initial: connected(selection: const LinkSelected(driveEdge)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Connection'), findsOneWidget);
      expect(find.text('brightness'), findsOneWidget);
      expect(find.text('light'), findsOneWidget);
      expect(find.text('brightness drives light'), findsOneWidget);
      await t.tap(find.text('Disconnect'));
      await t.pump();
      final op = key.currentState!.effects.whereType<ApplyEdit>().single.op;
      expect(op.setMappingDrive.id.toInt(), value);
    });

    testWidgets('a read edge: the sentence that says why there is no Disconnect', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(selection: const LinkSelected(readEdge)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('dimByTilt reads tiltSensor'), findsOneWidget);
      expect(find.textContaining('edit the formula'), findsOneWidget);
      expect(find.text('Disconnect'), findsNothing);
    });
  });
}
