/// The formula sheet end to end — the real Studio stack against the real
/// `bdld`: opened from the inspector's *Edit…*, the same draft as the
/// inspector's editor, the keys typing at display size, the palette in
/// its column, Esc closing with the draft kept, ⌘↩ saving and closing.
/// With `SNAP_DIR` set, a PNG of the sheet is written there (a look, not
/// a golden).
///
/// Skipped when the binary is absent (run `cargo build`).
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';
import 'dart:ui' as ui;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/composer.dart' show composerInSync;
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
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

Future<void> _loadFonts() async {
  Future<void> load(String family, List<String> paths) async {
    final loader = FontLoader(family);
    var any = false;
    for (final path in paths) {
      final f = File(path);
      if (!f.existsSync()) continue;
      loader.addFont(Future.value(ByteData.sublistView(f.readAsBytesSync())));
      any = true;
    }
    if (any) await loader.load();
  }

  final flutterRoot = Platform.environment['FLUTTER_ROOT'];
  await load('.AppleSystemUIFont', ['/System/Library/Fonts/SFNS.ttf']);
  await load('Menlo', ['/System/Library/Fonts/Menlo.ttc', '/System/Library/Fonts/SFNSMono.ttf']);
  await load('MaterialIcons', [
    if (flutterRoot != null)
      p.join(
        flutterRoot,
        'bin',
        'cache',
        'artifacts',
        'material_fonts',
        'MaterialIcons-Regular.otf',
      ),
  ]);
}

void main() {
  final bdld = _findBdld();
  final snapDir = Platform.environment['SNAP_DIR'];
  late LiveStore store;
  late CanvasPointer canvas;
  late Directory dir;
  final shotKey = GlobalKey();

  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();

  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);

  Future<AppState> settle(WidgetTester tester, bool Function(AppState) test) async {
    for (var i = 0; i < 600 && !test(store.state); i++) {
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await tester.pump(const Duration(milliseconds: 20));
    }
    if (!test(store.state)) {
      fail('timed out; last error: ${store.state.editor.lastError?.message}');
    }
    await tester.pump(const Duration(milliseconds: 20));
    return store.state;
  }

  Future<AppState> read(WidgetTester tester, int id) =>
      settle(tester, (s) => composerInSync(s, id) && !s.editor.composer.pendingCompose);

  Future<void> project(WidgetTester tester) async {
    if (snapDir != null) await _loadFonts();
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    store = LiveStore(
      spawn: DaemonClient.spawn,
      executable: bdld!,
      draftDebounce: const Duration(milliseconds: 150),
    );
    canvas = CanvasPointer(tester);
    await tester.pumpWidget(
      RepaintBoundary(
        key: shotKey,
        child: DesignHarness(store: store),
      ),
    );
    await act(
      const AppStarted(),
      (s) => s.connection is Connected || s.connection is ConnectionFailed,
    );
    expect(store.state.connection, isA<Connected>());
    dir =
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-sheet')) ??
        Directory.systemTemp;
    await act(
      NewProjectRequested(rootPath: p.join(dir.path, 'lamp'), name: 'lamp'),
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
    await act(CreateMappingRequested(name: 'tilt', inputs: const [], output: conceptId('Tilt')));
    await act(
      CreateMappingRequested(name: 'dimByTilt', inputs: const [], output: conceptId('Brightness')),
    );
    await act(SelectionChanged(MappingSelected(mappingId('dimByTilt'))), (s) => s.analysis != null);
    await tester.pump(const Duration(milliseconds: 100));
  }

  Future<void> teardown(WidgetTester tester) => tester.runAsync(() async {
    await store.dispose();
    await dir.delete(recursive: true);
  });

  LogicalKeyboardKey keyOf(String c) => switch (c) {
    ' ' => LogicalKeyboardKey.space,
    '(' => LogicalKeyboardKey.digit9,
    ')' => LogicalKeyboardKey.digit0,
    ',' => LogicalKeyboardKey.comma,
    '/' => LogicalKeyboardKey.slash,
    '+' || '=' => LogicalKeyboardKey.equal,
    '*' => LogicalKeyboardKey.digit8,
    '-' => LogicalKeyboardKey.minus,
    '!' => LogicalKeyboardKey.digit1,
    '>' => LogicalKeyboardKey.period,
    _ =>
      LogicalKeyboardKey.findKeyByKeyId(c.toLowerCase().codeUnitAt(0)) ?? LogicalKeyboardKey.keyA,
  };

  Future<void> type(WidgetTester tester, String text) async {
    for (final c in text.split('')) {
      await tester.sendKeyEvent(keyOf(c), character: c);
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 16)));
      await tester.pump(const Duration(milliseconds: 16));
    }
  }

  Future<void> snap(WidgetTester tester, String name) async {
    if (snapDir == null) return;
    await tester.pumpAndSettle(const Duration(milliseconds: 100));
    final boundary = shotKey.currentContext!.findRenderObject()! as RenderRepaintBoundary;
    final image = await tester.runAsync(() => boundary.toImage(pixelRatio: 2));
    final bytes = await tester.runAsync(() => image!.toByteData(format: ui.ImageByteFormat.png));
    File(p.join(snapDir, '$name.png'))
      ..parent.createSync(recursive: true)
      ..writeAsBytesSync(bytes!.buffer.asUint8List());
  }

  testWidgets('opened from the inspector, typed into, closed with the draft kept, saved', (
    tester,
  ) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      // the inspector's Edit… opens the sheet on the selected relationship
      await snap(tester, 'inspector-definition');
      await tester.tap(find.byKey(const ValueKey('edit-in-sheet')));
      await tester.pump();
      expect(store.state.editor.formulaSheet, id);
      expect(find.byKey(const ValueKey('formula-sheet')), findsOneWidget);
      expect(
        find.byKey(const ValueKey('editing-in-sheet')),
        findsOneWidget,
        reason: 'the inspector defers',
      );
      expect(find.byKey(const ValueKey('formula-sheet-title')), findsOneWidget);
      // the equation: the name it defines beside the field
      expect(find.byKey(const ValueKey('sheet-equation-lhs')), findsOneWidget);
      // the empty formula takes the keys after one click on its slot
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      await type(tester, 'clamp(tilt/90 deg,0,');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'clamp(tilt / 90 deg, 0, ?)');
      // the reading arrived: the slot under the caret is selected and the
      // palette column says what it expects
      await settle(tester, (s) => s.editor.composer.slot != null);
      expect(find.byKey(const ValueKey('sheet-palette')), findsOneWidget);
      expect(find.byKey(const ValueKey('slot-explanation')), findsOneWidget);
      await snap(tester, 'formula-sheet');
      // Esc: the composer clears its caret; a second Esc closes the sheet,
      // the draft kept
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();
      expect(store.state.editor.formulaSheet, isNull);
      expect(store.state.draft(id)!.source, 'clamp(tilt / 90 deg, 0, ?)');
      expect(find.byKey(const ValueKey('formula-sheet')), findsNothing);
      // ⌘E from the inspector's field opens it again; ⌘↩ saves and closes
      await tester.tap(find.byKey(const ValueKey('node-r.2')));
      await tester.pump();
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyE);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await tester.pump();
      expect(store.state.editor.formulaSheet, id);
      await tester.tap(find.byKey(const ValueKey('node-r.2')));
      await tester.pump();
      await type(tester, '1');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'clamp(tilt / 90 deg, 0, 1)');
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await settle(tester, (s) => s.mapping(id)!.hasDefinition());
      expect(store.state.editor.formulaSheet, isNull);
      expect(store.state.mapping(id)!.definition.formula, 'clamp(tilt / 90 deg, 0, 1)');
    } finally {
      await teardown(tester);
    }
  });
}
