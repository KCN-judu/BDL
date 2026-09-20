/// Saving keeps the whole authoring state, through the real Design page
/// against the real `bdld`: a formula typed in the editor — valid or not —
/// text typed in the Code view that does not build, and a moved node all
/// come back exactly as they were after Save, close and reopen; and the
/// close guard asks only when the project differs from what is saved.
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
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);
  Future<AppState> opened() => canvas.settle(
    store,
    (s) => s.project != null && s.system != null && s.editor.pendingRequests == 0,
  );

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
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-persist')) ??
        Directory.systemTemp;
    root = p.join(dir.path, 'lamp');
    await act(NewProjectRequested(rootPath: root, name: 'lamp'), (s) => s.project != null);
    await opened();
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
    await act(
      CreateMappingRequested(
        name: 'dimByTilt',
        inputs: [conceptId('Tilt')],
        output: conceptId('Brightness'),
      ),
    );
    await act(
      CreateMappingRequested(name: 'level', inputs: const [], output: conceptId('Brightness')),
    );
    await act(const SaveRequested(), (s) => !s.flat!.dirty && s.editor.pendingRequests == 0);
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  /// Close through the guard (the project is expected clean) and reopen.
  Future<AppState> reopen(WidgetTester tester) async {
    await act(const CloseProjectRequested(), (s) => s.project == null);
    expect(store.state.editor.closeGuard, isNull, reason: 'a saved project asks nothing');
    await act(OpenProjectRequested(root), (s) => s.project != null);
    return opened();
  }

  testWidgets(
    'Save keeps every unfinished edit; reopening returns to it',
    (tester) async {
      await project(tester);
      try {
        final dim = mappingId('dimByTilt');
        final level = mappingId('level');

        // a valid formula draft and an invalid one, neither committed
        await act(DefinitionDraftChanged(mappingId: dim, source: 'Tilt / 90 deg'));
        await act(DefinitionDraftChanged(mappingId: level, source: 'Tilt +'));
        await canvas.settle(
          store,
          (s) =>
              s.draft(dim)?.check == DraftCheck.checked &&
              s.draft(level)?.check == DraftCheck.checked,
        );
        expect(store.state.draft(level)!.parseOk, isFalse, reason: 'invalid, and still kept');

        // text that does not build, typed in the Code view
        await act(const DesignViewChanged(DesignView.code));
        var s = await canvas.settle(store, (s) => s.editor.sources.revision == s.revision);
        final typed =
            '${s.editor.sources.text.trimRight()}\n\n// half a sink\noutput light : Brightness\ndrive light by\n';
        await tester.enterText(
          find.descendant(of: find.byType(CodePane), matching: find.byType(TextField)),
          typed,
        );
        await tester.pump(kSourceEditPause + const Duration(milliseconds: 20));
        s = await canvas.settle(
          store,
          (s) => s.editor.sources.outOfSync && s.editor.pendingRequests == 0,
        );

        // a moved node
        await act(NodeMoved(NodeRef.mapping(dim), const Offset(777, 333)));

        // the project says it differs from what is saved; Save answers it
        await act(const CloseProjectRequested(), (s) => s.editor.closeGuard != null);
        expect(store.state.project, isNotNull, reason: 'the question, not a close');
        await act(const CloseGuardAnswered(CloseGuardAnswer.cancel));
        await act(const SaveRequested(), (s) => !s.flat!.dirty && s.editor.pendingRequests == 0);
        expect(File(p.join(root, 'src', 'main.bdl')).readAsStringSync(), typed, reason: 'as typed');

        // close, reopen: everything exactly as it was
        s = await reopen(tester);
        expect(s.draft(dim)?.source, 'Tilt / 90 deg');
        expect(s.draft(level)?.source, 'Tilt +');
        expect(
          s.project!.mappings.every((m) => !m.hasDefinition()),
          isTrue,
          reason: 'not committed',
        );
        expect(s.editor.layout[NodeRef.mapping(dim)], const Offset(777, 333));
        s = await canvas.settle(store, (s) => s.editor.sources.revision == s.revision);
        expect(s.editor.sources.text, typed, reason: 'the text that does not build, as typed');
        expect(s.editor.sources.outOfSync, isTrue);
        expect(s.editor.view, DesignView.code, reason: 'the view the designer left');
        expect(s.flat!.dirty, isFalse, reason: 'reopened clean');

        // Don't Save reverts to what was saved: type more, close without saving
        await act(DefinitionDraftChanged(mappingId: dim, source: 'Tilt / 45 deg'));
        await canvas.settle(store, (s) => s.draft(dim)?.check == DraftCheck.checked);
        await act(const CloseProjectRequested(), (s) => s.editor.closeGuard != null);
        await act(const CloseGuardAnswered(CloseGuardAnswer.dontSave), (s) => s.project == null);
        await act(OpenProjectRequested(root), (s) => s.project != null);
        s = await opened();
        expect(s.draft(mappingId('dimByTilt'))?.source, 'Tilt / 90 deg', reason: 'the saved draft');
        expect(store.failure, isNull);
      } finally {
        await tester.runAsync(teardown);
      }
    },
    skip: bdld == null,
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
