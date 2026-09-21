/// The canvas selection algebra under real pointer and keyboard events
/// (docs/architecture/studio-ui.md §2, "Interaction"): the click rules, the
/// platform primary modifier, the CAD marquee (window left → right, crossing
/// right → left, ⌘ adds, ⇧ subtracts), the drag threshold, dragging the
/// selected set as one, the selection surviving a projection, ⌘A, Esc, and
/// the shortcuts that must not fire while a field owns the keyboard.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/system.dart' show surviving;
import 'package:bdl_studio/platform/desktop.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/canvas_harness.dart' show SceneLookup;
import 'support/roles.dart';

const tilt = 0, level = 1;
const source = 0, rule = 1, value = 2;
const light = 0;

pb.ConceptView concept(int id, String name) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(quantity: pb.Dim()),
);

/// Tilt, Brightness; tiltSensor : () -> Tilt (a Source), dimByTilt : Tilt ->
/// Brightness (a rule), brightness : () -> Brightness (a value); light, a
/// sink accepting Brightness.
/// The lamp as Sem blocks (ADR-0043): `tiltSensor : Tilt` a Source,
/// `dimByTilt : Brightness = tiltSensor / 90 deg` and
/// `brightness : Brightness = dimByTilt` two Sem blocks with mapping
/// blocks, `light` the sink `brightness` drives.  (`rule` keeps its name
/// from the days it was one; every relationship here is a value.)
pb.ProjectProjection design() => pb.ProjectProjection(
  revision: Int64(1),
  name: 'lamp',
  concepts: [concept(tilt, 'Tilt'), concept(level, 'Brightness')],
  mappings: [
    mappingView(
      id: Int64(source),
      name: 'tiltSensor',
      signature: pb.Signature(output: Int64(tilt)),
    ),
    mappingView(
      id: Int64(rule),
      name: 'dimByTilt',
      signature: pb.Signature(output: Int64(level)),
      definition: pb.Definition(formula: 'tiltSensor / 90 deg'),
    ),
    mappingView(
      id: Int64(value),
      name: 'brightness',
      signature: pb.Signature(output: Int64(level)),
      definition: pb.Definition(formula: 'dimByTilt'),
      drivesOutputId: Int64(light),
    ),
  ],
  outputs: [pb.OutputView(id: Int64(light), name: 'light', accepts: Int64(level))],
);

/// The analysis's read edges of [design]: dimByTilt reads tiltSensor,
/// brightness reads dimByTilt.
const refs = <int, List<int>>{
  rule: [source],
  value: [rule],
};

const c0 = NodeRef.concept(tilt);
const c1 = NodeRef.concept(level);
const m0 = NodeRef.mapping(source);
const m1 = NodeRef.mapping(rule);
const m2 = NodeRef.mapping(value);
const d1 = NodeRef.definition(rule);
const d2 = NodeRef.definition(value);
const o0 = NodeRef.output(light);

/// The Sem blocks in a row, the sink right, each mapping block away from
/// the marquees the tests draw (a concept is a template and takes no
/// place).
final layout = <NodeRef, Offset>{
  m0: const Offset(300, 40),
  m1: const Offset(300, 200),
  d1: const Offset(40, 200),
  m2: const Offset(600, 40),
  d2: const Offset(600, 300),
  o0: const Offset(900, 40),
};

class Harness extends StatefulWidget {
  const Harness({super.key, required this.actions, this.selection = const NoSelection()});
  final List<AppAction> actions;
  final Selection selection;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late Map<NodeRef, Offset> nodes = {...layout};
  late Selection selection = widget.selection;
  NodeRef? renaming;
  pb.ProjectProjection project = design();

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: Column(
        children: [
          // a field outside the canvas: the shortcuts must not reach the
          // canvas while it has the keyboard
          const SizedBox(height: 30, child: TextField(key: ValueKey('field'))),
          Expanded(
            child: NodeCanvas(
              project: project,
              layout: nodes,
              selection: selection,
              refs: refs,
              renaming: renaming,
              dispatch: (a) {
                widget.actions.add(a);
                setState(() {
                  if (a is SelectionChanged) selection = a.selection;
                  if (a is NodeMoved) nodes = {...nodes, a.node: a.position};
                  if (a is NodesMoved) nodes = {...nodes, ...a.positions};
                  if (a is InlineRenameStarted) renaming = a.node;
                  if (a is InlineRenameFinished) renaming = null;
                });
              },
            ),
          ),
        ],
      ),
    ),
  );
}

CanvasScene sceneOf(Map<NodeRef, Offset> nodes) => buildScene(design(), nodes, refs: refs);

Offset origin(WidgetTester t) => t.getTopLeft(find.byType(NodeCanvas));

Future<void> drag(WidgetTester tester, Offset from, Offset to, {int steps = 8}) async {
  final o = origin(tester);
  final g = await tester.startGesture(o + from, kind: PointerDeviceKind.mouse);
  await tester.pump(const Duration(milliseconds: 20));
  for (var i = 1; i <= steps; i++) {
    await g.moveTo(o + Offset.lerp(from, to, i / steps)!);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await g.up();
  await tester.pump(const Duration(milliseconds: 20));
}

Future<void> click(WidgetTester tester, Offset scene) async {
  await tester.tapAt(origin(tester) + scene);
  await tester.pump(const Duration(milliseconds: 20));
  // past the double-click interval, so the next click is a single one
  await tester.pump(kDoubleClickInterval + const Duration(milliseconds: 10));
}

Future<void> withKey(
  WidgetTester tester,
  LogicalKeyboardKey key,
  Future<void> Function() body,
) async {
  await tester.sendKeyDownEvent(key);
  try {
    await body();
  } finally {
    await tester.sendKeyUpEvent(key);
  }
}

Selection lastSelection(List<AppAction> actions) =>
    actions.whereType<SelectionChanged>().last.selection;

Set<NodeRef> lastSet(List<AppAction> actions) => selectedNodes(lastSelection(actions));

Future<HarnessState> pumpCanvas(
  WidgetTester tester,
  List<AppAction> actions, {
  Selection selection = const NoSelection(),
}) async {
  tester.view.physicalSize = const Size(1400, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final key = GlobalKey<HarnessState>();
  await tester.pumpWidget(Harness(key: key, actions: actions, selection: selection));
  return key.currentState!;
}

void main() {
  final scene = sceneOf(layout);
  Offset header(NodeRef ref) => scene.node(ref).header.center;
  final primaryKey = primaryModifierIsControl
      ? LogicalKeyboardKey.controlLeft
      : LogicalKeyboardKey.metaLeft;

  group('click', () {
    testWidgets('A/B: a plain click selects one; another replaces; empty clears', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await click(t, header(m0));
      expect(lastSelection(actions), const MappingSelected(source));
      await click(t, header(m1));
      expect(lastSelection(actions), const MappingSelected(rule));
      await click(t, const Offset(700, 400));
      expect(lastSelection(actions), const NoSelection());
      // the click is a click: nothing moved
      expect(actions.whereType<NodeMoved>(), isEmpty);
      expect(actions.whereType<NodesMoved>(), isEmpty);
    });

    testWidgets('a click on an already-selected member keeps the set and makes it active', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1, m2}, active: m0));
      await click(t, header(m2));
      expect(lastSelection(actions), MultiSelected({m0, m1, m2}, active: m2));
    });

    testWidgets('C/D: the primary modifier adds an unselected node and removes a selected one', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await click(t, header(m0));
      await withKey(t, primaryKey, () async {
        await click(t, header(m1));
        expect(lastSelection(actions), MultiSelected({m0, m1}, active: m1));
        await click(t, header(m2));
        expect(lastSet(actions), {m0, m1, m2});
        await click(t, header(m1));
        expect(lastSelection(actions), MultiSelected({m0, m2}, active: m2));
        // the modifier keeps the set on empty canvas
        await click(t, const Offset(700, 400));
        expect(lastSet(actions), {m0, m2});
      });
    });

    testWidgets('E: on Windows and Linux the primary modifier is Ctrl, and ⌘ is not', (t) async {
      final actions = <AppAction>[];
      final was = primaryModifierIsControl;
      primaryModifierIsControl = true;
      addTearDown(() => primaryModifierIsControl = was);
      await pumpCanvas(t, actions);
      await click(t, header(m0));
      await withKey(t, LogicalKeyboardKey.controlLeft, () async {
        await click(t, header(m1));
      });
      expect(lastSet(actions), {m0, m1});
      await withKey(t, LogicalKeyboardKey.metaLeft, () async {
        await click(t, header(m2));
      });
      expect(lastSelection(actions), const MappingSelected(value), reason: '⌘ is a plain click');
    });
  });

  group('marquee', () {
    testWidgets('F/G: left → right is a window: only nodes wholly inside are taken', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      final r0 = scene.node(m0).rect;
      final r1 = scene.node(m1).rect;
      // encloses tiltSensor whole, touches dimByTilt (its top edge only)
      await drag(t, r0.topLeft - const Offset(10, 10), Offset(r0.right + 10, r1.top + 10));
      expect(lastSelection(actions), const MappingSelected(source));
      expect(actions.whereType<NodeMoved>(), isEmpty);
      expect(actions.whereType<NodesMoved>(), isEmpty, reason: 'a marquee moves nothing');
    });

    testWidgets('H/I: right → left is a crossing: touched and enclosed nodes are taken', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      final r0 = scene.node(m0).rect;
      final r1 = scene.node(m1).rect;
      // the same rectangle dragged from its right edge to its left
      await drag(t, Offset(r0.right + 10, r1.top + 10), r0.topLeft - const Offset(10, 10));
      expect(lastSet(actions), {m0, m1});
    });

    testWidgets('J: the vertical direction does not change the mode', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      final r0 = scene.node(m0).rect;
      final r1 = scene.node(m1).rect;
      // bottom-left → top-right: still a window
      await drag(t, Offset(r0.left - 10, r1.top + 10), Offset(r0.right + 10, r0.top - 10));
      expect(lastSelection(actions), const MappingSelected(source));
      // bottom-right → top-left: still a crossing
      await drag(t, Offset(r0.right + 10, r1.top + 10), Offset(r0.left - 10, r0.top - 10));
      expect(lastSet(actions), {m0, m1});
    });

    testWidgets('K: the primary modifier extends the selection with the marquee', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const OutputSelected(light));
      final r0 = scene.node(m0).rect;
      await withKey(t, primaryKey, () async {
        await drag(t, r0.topLeft - const Offset(10, 10), r0.bottomRight + const Offset(10, 10));
      });
      expect(lastSet(actions), {o0, m0});
    });

    testWidgets('L: ⇧ subtracts the marquee from the selection, leaving the rest', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1, m2, o0}, active: o0));
      final r0 = scene.node(m0).rect;
      final r1 = scene.node(m1).rect;
      await withKey(t, LogicalKeyboardKey.shiftLeft, () async {
        // a crossing over tiltSensor and dimByTilt
        await drag(t, Offset(r0.right + 10, r1.bottom + 10), r0.topLeft - const Offset(10, 10));
      });
      expect(lastSelection(actions), MultiSelected({m2, o0}, active: o0));
    });

    testWidgets('a press that moves less than the threshold is a click, not a marquee', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const MappingSelected(source));
      final o = origin(t);
      final g = await t.startGesture(o + const Offset(700, 400), kind: PointerDeviceKind.mouse);
      await t.pump(const Duration(milliseconds: 20));
      await g.moveTo(o + const Offset(702, 401));
      await t.pump(const Duration(milliseconds: 20));
      await g.up();
      await t.pump(const Duration(milliseconds: 20));
      expect(lastSelection(actions), const NoSelection(), reason: 'a click on empty canvas');
    });
  });

  group('drag', () {
    testWidgets('M/N/O: dragging a selected node moves the set, offsets kept; one operation', (
      t,
    ) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions, selection: MultiSelected({m0, m1}, active: m0));
      final before0 = scene.node(m0).rect.topLeft;
      final before1 = scene.node(m1).rect.topLeft;
      await drag(t, header(m0), header(m0) + const Offset(80, 60));
      final moved = actions.whereType<NodesMoved>().single;
      expect(moved.positions.keys.toSet(), {m0, m1, d1}, reason: 'a block takes its mapping block');
      expect(moved.positions[d1]! - layout[d1]!, moved.positions[m1]! - before1);
      expect(moved.positions[m0]! - before0, moved.positions[m1]! - before1);
      expect(actions.whereType<NodeMoved>(), isEmpty, reason: 'one layout operation');
      expect(h.selection, MultiSelected({m0, m1}, active: m0), reason: 'the set stays');
    });

    testWidgets('M: dragging an unselected node selects it alone and moves only it', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1}, active: m0));
      await drag(t, header(m2), header(m2) + const Offset(0, 80));
      expect(lastSelection(actions), const MappingSelected(value));
      final moved = actions.whereType<NodesMoved>().single;
      expect(moved.positions.keys.toSet(), {m2, d2}, reason: 'one declaration, two nodes');
      expect(actions.whereType<NodeMoved>(), isEmpty);
    });

    testWidgets('a press on a node that does not move keeps the selection as it was', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1, m2}, active: m1));
      final o = origin(t);
      final g = await t.startGesture(o + header(m0), kind: PointerDeviceKind.mouse);
      await t.pump(const Duration(milliseconds: 20));
      // nothing is dispatched on the press itself
      expect(actions.whereType<SelectionChanged>(), isEmpty);
      await g.up();
      await t.pump(const Duration(milliseconds: 20));
      expect(lastSelection(actions), MultiSelected({m0, m1, m2}, active: m0));
    });

    testWidgets('Esc cancels a node drag: nothing moves', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      final o = origin(t);
      final g = await t.startGesture(o + header(m0), kind: PointerDeviceKind.mouse);
      await t.pump(const Duration(milliseconds: 20));
      await g.moveTo(o + header(m0) + const Offset(60, 60));
      await t.pump(const Duration(milliseconds: 20));
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      await g.up();
      await t.pump(const Duration(milliseconds: 20));
      expect(actions.whereType<NodeMoved>(), isEmpty);
      expect(actions.whereType<NodesMoved>(), isEmpty);
    });
  });

  group('keyboard', () {
    testWidgets('P: ⌘A selects every visible node; Esc clears the idle selection', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await click(t, const Offset(700, 400));
      await withKey(t, primaryKey, () async {
        await t.sendKeyEvent(LogicalKeyboardKey.keyA);
      });
      await t.pump();
      expect(lastSet(actions), {m0, m1, m2, o0});
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(lastSelection(actions), const NoSelection());
    });

    testWidgets('S: Esc cancels a marquee in progress without selecting', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const MappingSelected(value));
      final o = origin(t);
      final g = await t.startGesture(o + const Offset(250, 10), kind: PointerDeviceKind.mouse);
      await t.pump(const Duration(milliseconds: 20));
      await g.moveTo(o + const Offset(560, 380));
      await t.pump(const Duration(milliseconds: 20));
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      await g.up();
      await t.pump(const Duration(milliseconds: 20));
      expect(actions.whereType<SelectionChanged>(), isEmpty, reason: 'cancelled, not committed');
    });

    testWidgets('Q/V: Delete and ⌘A do not fire while a text field has the keyboard', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const MappingSelected(source));
      await t.tap(find.byKey(const ValueKey('field')));
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.backspace);
      await withKey(t, primaryKey, () async {
        await t.sendKeyEvent(LogicalKeyboardKey.keyA);
      });
      await t.pump();
      expect(actions.whereType<DeleteSelectionRequested>(), isEmpty);
      expect(actions.whereType<SelectionChanged>(), isEmpty);
    });

    testWidgets('Backspace inside the inline rename edits the name, never deletes the node', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await click(t, header(m0));
      await t.tapAt(origin(t) + header(m0));
      await t.pump(const Duration(milliseconds: 40));
      await t.tapAt(origin(t) + header(m0));
      await t.pump(const Duration(milliseconds: 100));
      expect(actions.whereType<InlineRenameStarted>(), hasLength(1));
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.backspace);
      await t.pump();
      expect(actions.whereType<DeleteSelectionRequested>(), isEmpty);
    });

    testWidgets('arrow keys nudge the selected set by the grid, ⇧ by one point', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1}, active: m0));
      await click(t, header(m0));
      await t.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await t.pump();
      final moved = actions.whereType<NodesMoved>().single;
      expect(moved.positions[m0], layout[m0]! + const Offset(8, 0));
      expect(moved.positions[m1], layout[m1]! + const Offset(8, 0));
      await withKey(t, LogicalKeyboardKey.shiftLeft, () async {
        await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      });
      await t.pump();
      expect(actions.whereType<NodesMoved>().last.positions[m0], layout[m0]! + const Offset(8, 1));
    });
  });

  group('selection state', () {
    test('a set keeps its surviving members when one is deleted; a single selection clears', () {
      final s = AppState(project: design(), editor: const EditorState());
      final gone = design()..mappings.removeWhere((m) => m.id.toInt() == rule);
      final after = s.copyWith(project: gone);
      expect(surviving(after, MultiSelected({m0, m1, m2}, active: m1)), MultiSelected({m0, m2}));
      expect(surviving(after, const MappingSelected(rule)), const NoSelection());
      expect(surviving(after, const MappingSelected(source)), const MappingSelected(source));
    });

    test('the active object is named, never the first of the set', () {
      final sel = MultiSelected({m2, m0}, active: m0);
      expect(activeNode(sel), m0);
      expect(selectionOfNodes({m0}, active: m0), const MappingSelected(source));
      expect(selectionOfNodes({}), const NoSelection());
      expect(
        selectionOfNodes({m0, m1}, active: m2),
        MultiSelected({m0, m1}),
        reason: 'an active object outside the set is dropped',
      );
    });
  });

  group('chain', () {
    test('⇧-click takes the one shortest chain over read and drive edges, never a branch', () {
      // tiltSensor → dimByTilt → brightness → light: one chain
      expect(uniqueSignatureChain(scene, m0, o0), {m0, m1, m2, o0});
      expect(uniqueSignatureChain(scene, m1, o0), {m1, m2, o0});
      expect(uniqueSignatureChain(scene, m1, m1), {m1});
      // a second block also reading tiltSensor and read by brightness: two
      // shortest paths from tiltSensor to brightness — ambiguous, no chain
      final branched = design()
        ..mappings.add(
          mappingView(
            id: Int64(9),
            name: 'dimByTilt2',
            signature: pb.Signature(output: Int64(level)),
            definition: pb.Definition(formula: 'tiltSensor / 45 deg'),
          ),
        );
      final s2 = buildScene(
        branched,
        {...layout, const NodeRef.mapping(9): const Offset(300, 320)},
        refs: {
          rule: [source],
          9: [source],
          value: [rule, 9],
        },
      );
      expect(uniqueSignatureChain(s2, m0, m2), isNull);
      expect(uniqueSignatureChain(s2, m0, m1), {m0, m1}, reason: 'the direct edge is unique');
      // without an analysis there are no read edges: no chain
      expect(uniqueSignatureChain(buildScene(design(), layout), m0, m1), isNull);
    });
  });
}
