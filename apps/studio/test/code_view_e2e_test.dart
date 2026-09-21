/// The Code and Split views through the real Design page against the real
/// `bdld` (ADR-0023 §3–§5): the sources show what the canvas made; text
/// typed in the editor becomes a node with its identity kept; text that
/// does not build keeps the graph at the last version that did and says
/// why; selecting a node reveals its declaration; a save writes the text.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/pages/code_pane.dart';
import 'package:flutter/material.dart';
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
  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);
  Future<AppState> sourcesAt(int revision) =>
      canvas.settle(store, (s) => s.editor.sources.revision == revision);

  /// Type the whole file and let the editor's pause elapse: one edit.
  Future<void> type(WidgetTester tester, String text) async {
    await tester.enterText(
      find.descendant(of: find.byType(CodePane), matching: find.byType(TextField)),
      text,
    );
    await tester.pump(kSourceEditPause + const Duration(milliseconds: 20));
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
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-code-view')) ??
        Directory.systemTemp;
    root = p.join(dir.path, 'lamp');
    await act(
      NewProjectRequested(rootPath: root, name: 'lamp'),
      (s) => s.project != null && s.system != null && s.editor.pendingRequests == 0,
    );
    await act(
      CreateConceptRequested(
        name: 'Tilt',
        representation: pb.Representation(quantity: pb.Dim(angle: 1)),
      ),
    );
    await act(
      CreateConceptRequested(
        name: 'Brightness',
        representation: pb.Representation(quantity: pb.Dim()),
      ),
    );
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  testWidgets(
    'one New Project; Design, Code and Split are views of it',
    (tester) async {
      await project(tester);
      try {
        // Design: the segmented control, no editor
        expect(find.text('Design'), findsOneWidget);
        expect(find.byType(CodePane), findsNothing);

        // Code: the sources arrive and show what the canvas made
        await tester.tap(find.text('Code'));
        var s = await sourcesAt(store.state.revision);
        expect(s.editor.view, DesignView.code);
        expect(find.byType(CodePane), findsOneWidget);
        final text = s.editor.sources.text;
        expect(text, contains('concept Tilt : Angle'));
        expect(text, contains('concept Brightness : Scalar'));
        await tester.pump();
        expect(find.textContaining('does not build yet'), findsNothing);
        final tilt = conceptId('Tilt');
        final bright = conceptId('Brightness');

        // typing a relationship: the graph follows, identities kept
        final typed =
            '${text.trimRight()}\n\n/// dims with the tilt\n'
            'mapping dimByTilt : Tilt -> Brightness\ndimByTilt(Tilt) =\n  Tilt / 90 deg\n';
        final before = s.revision;
        await type(tester, typed);
        s = await canvas.settle(store, (s) => s.revision > before && s.editor.pendingRequests == 0);
        s = await sourcesAt(s.revision);
        final dim = s.project!.mappings.firstWhere((m) => m.name == 'dimByTilt');
        expect(dim.signature.inputs.map((i) => i.toInt()), [tilt]);
        expect(dim.signature.output.toInt(), bright);
        expect(conceptId('Tilt'), tilt, reason: 'identity survives a text edit');
        expect(s.editor.sources.buffer, isNull, reason: 'the typed text is the daemon\'s now');
        expect(s.editor.sources.text, typed);
        // a rule is a template, not a node of the value graph (ADR-0044):
        // the layout service places nothing for it
        expect(s.editor.layout[NodeRef.mapping(dim.id.toInt())], isNull, reason: 'not a node');
        expect(s.flat!.dirty, isTrue);

        // text that does not build: the graph stays, the draft is kept, why is shown
        final broken = typed.replaceFirst('Tilt -> Brightness', 'Tilt ->');
        await type(tester, broken);
        s = await canvas.settle(
          store,
          (s) => s.editor.sources.outOfSync && s.editor.pendingRequests == 0,
        );
        expect(s.project!.mappings.any((m) => m.id == dim.id), isTrue, reason: 'nothing discarded');
        expect(s.editor.sources.text, broken, reason: 'exactly as typed');
        expect(s.editor.lastError, isNull, reason: 'a draft is a state, not a failure');
        await tester.pump();
        expect(find.textContaining('does not build yet'), findsWidgets);
        final faults = s.editor.sources.diagnosticsOf('src/main.bdl');
        expect(faults.where((d) => !d.open), isNotEmpty);
        expect(find.text(faults.first.message), findsOneWidget);

        // fixed again: in step
        await type(tester, typed);
        s = await canvas.settle(
          store,
          (s) => !s.editor.sources.outOfSync && s.editor.pendingRequests == 0,
        );
        await tester.pump();
        expect(find.textContaining('does not build yet'), findsNothing);

        // Split: both, and selecting a node on the canvas reveals its item
        await tester.tap(find.text('Split'));
        await tester.pump();
        expect(find.byType(CodePane), findsOneWidget);
        await act(SelectionChanged(MappingSelected(dim.id.toInt())));
        await tester.pump(const Duration(milliseconds: 200));
        final field = tester.widget<TextField>(
          find.descendant(of: find.byType(CodePane), matching: find.byType(TextField)),
        );
        final caret = field.controller!.selection.baseOffset;
        expect(s.editor.sources.text.substring(caret), startsWith('/// dims with the tilt'));

        // a graph edit reaches the text without touching the comment
        await act(RenameConceptRequested(id: bright, name: 'Level'));
        s = await sourcesAt(store.state.revision);
        expect(s.editor.sources.text, contains('mapping dimByTilt : Tilt -> Level'));
        expect(s.editor.sources.text, contains('/// dims with the tilt'));

        // save: the text on disk is the Code view's
        await act(const SaveRequested(), (s) => !s.flat!.dirty && s.editor.pendingRequests == 0);
        expect(File(p.join(root, 'src', 'main.bdl')).readAsStringSync(), s.editor.sources.text);
        expect(store.failure, isNull);
      } finally {
        await tester.runAsync(teardown);
      }
    },
    skip: bdld == null,
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
