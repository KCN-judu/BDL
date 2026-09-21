/// The selected set through the real Studio stack against the real `bdld`
/// (docs/architecture/studio-ui.md §2, "Interaction"): a window marquee
/// selects three nodes; the contextual menu on one of them is about all of
/// them; dragging one moves all three, spacing kept, as one layout write;
/// a save, a close and a reopen bring the positions back.  Then the drive:
/// a Sem block dragged onto the sink is the drive edge, and the output
/// pass agrees; a second one is a conflict the pass reports (ADR-0044).
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/canvas_harness.dart';

String? _findBdld() {
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && File(env).existsSync()) return env;
  for (final rel in ['../../target/debug/bdld', '../../target/release/bdld']) {
    final f = p.normalize(p.join(Directory.current.path, rel));
    if (File(f).existsSync()) return f;
  }
  return null;
}

void main() {
  final bdld = _findBdld();

  late LiveStore store;
  late Directory dir;
  late String root;
  late CanvasPointer canvas;

  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  int clockId(String name) =>
      store.state.project!.clocks.firstWhere((c) => c.name == name).id.toInt();
  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);
  Future<AppState> analysed() => canvas.settle(
    store,
    (s) =>
        s.editor.pendingRequests == 0 &&
        s.analysis != null &&
        s.analysis!.revision == s.project!.revision &&
        s.system != null &&
        s.system!.revision == s.flat!.revision,
  );
  Offset header(AppState s, String name) =>
      canvas.scene(s).node(NodeRef.mapping(mappingId(name))).header.center;
  Rect rect(AppState s, NodeRef ref) => canvas.scene(s).node(ref).rect;

  Future<void> value(String name, String formula, int output, {required Offset at}) async {
    await act(CreateMappingRequested(name: name, inputs: const [], output: output));
    final id = mappingId(name);
    await act(SetMappingClockRequested(mappingId: id, clockId: clockId('main')));
    await act(
      DefinitionDraftChanged(mappingId: id, source: formula),
      (s) => s.draft(id)?.check == DraftCheck.checked,
    );
    await act(CommitDefinitionRequested(id), (s) => s.committedDefinition(id) == formula);
    await act(NodeMoved(NodeRef.mapping(id), at));
  }

  Future<void> project(WidgetTester tester) async {
    tester.view.physicalSize = const Size(2400, 1400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    store = LiveStore(spawn: DaemonClient.spawn, executable: bdld!);
    canvas = CanvasPointer(tester);
    await tester.pumpWidget(DesignHarness(store: store));
    await act(
      const AppStarted(),
      (s) => s.connection is Connected || s.connection is ConnectionFailed,
    );
    expect(store.state.connection, isA<Connected>());
    dir =
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-multi')) ??
        Directory.systemTemp;
    root = p.join(dir.path, 'arm');
    await act(
      NewProjectRequested(rootPath: root, name: 'arm'),
      (s) => s.project != null && s.system != null,
    );
    await act(
      CreateConceptRequested(
        name: 'ServoPosition',
        representation: pb.Representation(quantity: pb.Dim(angle: 1)),
      ),
    );
    await act(const CreateClockDomainRequested('main'));
    final pos = conceptId('ServoPosition');
    await value('rest', '0 deg', pos, at: const Offset(400, 40));
    await value('lifted', '45 deg', pos, at: const Offset(400, 200));
    await value('parked', '90 deg', pos, at: const Offset(400, 360));
    await act(
      CreateOutputRequested(name: 'Servo', accepts: pos, clockId: clockId('main'), required: true),
      (s) => s.project!.outputs.length == 1 && s.editor.pendingRequests == 0,
    );
    await act(
      NodeMoved(
        NodeRef.output(store.state.project!.outputs.single.id.toInt()),
        const Offset(900, 40),
      ),
    );
    await analysed();
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  testWidgets(
    'select three by window marquee, menu about the set, drag one moves all, positions persist',
    (tester) async {
      await project(tester);
      try {
        var s = store.state;
        // a window over the three values (whole), the concept left out
        final top = rect(s, NodeRef.mapping(mappingId('rest')));
        final bottom = rect(s, NodeRef.mapping(mappingId('parked')));
        await canvas.drag(
          top.topLeft - const Offset(20, 20),
          bottom.bottomRight + const Offset(20, 20),
        );
        await tester.pump();
        s = store.state;
        expect(s.editor.selection, isA<MultiSelected>());
        final set = (s.editor.selection as MultiSelected).mappings.toSet();
        expect(set, {mappingId('rest'), mappingId('lifted'), mappingId('parked')});

        // the contextual menu on the middle one is about the three
        await canvas.rightClick(header(s, 'lifted'));
        expect(
          find.descendant(of: find.byType(MenuItemButton), matching: find.text('Delete 3 objects')),
          findsOneWidget,
        );
        expect(find.text('Rename'), findsNothing);
        expect(store.state.editor.selection, s.editor.selection, reason: 'the set is kept');
        await tester.sendKeyEvent(LogicalKeyboardKey.escape);
        await tester.pump(const Duration(milliseconds: 100));

        // drag the middle one: all three move, spacing kept
        final before = {
          for (final name in ['rest', 'lifted', 'parked'])
            name: rect(s, NodeRef.mapping(mappingId(name))).topLeft,
        };
        await canvas.drag(header(s, 'lifted'), header(s, 'lifted') + const Offset(160, 90));
        s = await canvas.settle(store, (s) => s.editor.pendingRequests == 0);
        for (final name in ['rest', 'lifted', 'parked']) {
          expect(
            s.editor.layout[NodeRef.mapping(mappingId(name))],
            before[name]! + const Offset(160, 90),
            reason: '$name moved with the set',
          );
        }
        expect(s.editor.selection, isA<MultiSelected>(), reason: 'the set survives the move');

        // saved, closed, reopened: the positions are the project's
        await act(const SaveRequested(), (s) => !s.project!.dirty && s.editor.pendingRequests == 0);
        await act(const CloseProjectRequested(), (s) => s.project == null);
        await act(
          OpenProjectRequested(root),
          (s) => s.project != null && s.system != null && s.editor.pendingRequests == 0,
        );
        s = store.state;
        for (final name in ['rest', 'lifted', 'parked']) {
          expect(
            s.editor.layout[NodeRef.mapping(mappingId(name))],
            before[name]! + const Offset(160, 90),
          );
        }
        expect(s.editor.selection, const NoSelection(), reason: 'selection is editor state');

        // a Sem block → the sink is the drive edge (ADR-0044): at once,
        // and the pass agrees; a second block dropped on the driven sink is
        // a second drive, which the output pass reports as a conflict
        s = await analysed();
        final servo = s.project!.outputs.single.id.toInt();
        final scene = canvas.scene(s);
        final liftedOut = scene
            .socket(NodeRef.mapping(mappingId('lifted')), side: SocketSide.output)
            .center;
        final sinkIn = scene.socket(NodeRef.output(servo), side: SocketSide.input).center;
        await canvas.drag(liftedOut, sinkIn);
        s = await canvas.settle(
          store,
          (s) =>
              s.editor.pendingRequests == 0 &&
              s.project!.mappings.any((m) => m.hasDrivesOutputId()),
        );
        final driver = s.project!.mappings.singleWhere((m) => m.hasDrivesOutputId());
        expect(driver.name, 'lifted');
        expect(driver.drivesOutputId.toInt(), servo);
        s = await analysed();
        expect(s.outputAnalysis(servo)!.state, pb.OutputState.OUTPUT_STATE_DRIVEN);
        expect(s.outputAnalysis(servo)!.driver.toInt(), mappingId('lifted'));
        final restOut = canvas
            .scene(s)
            .socket(NodeRef.mapping(mappingId('rest')), side: SocketSide.output)
            .center;
        await canvas.drag(restOut, sinkIn);
        s = await canvas.settle(
          store,
          (s) =>
              s.editor.pendingRequests == 0 &&
              s.project!.mappings.where((m) => m.hasDrivesOutputId()).length == 2,
        );
        s = await analysed();
        expect(s.outputAnalysis(servo)!.state, pb.OutputState.OUTPUT_STATE_CONFLICT);
      } catch (e, st) {
        // ignore: avoid_print
        print('DBG FAIL $e');
        // ignore: avoid_print
        print(st);
        rethrow;
      } finally {
        await tester.runAsync(teardown);
      }
    },
    skip: bdld == null,
    timeout: const Timeout(Duration(minutes: 3)),
  );
}
