/// Typing at a designer's pace: several characters land before the
/// compiler has read the first (the draft check is debounced, the
/// reading is a round trip), and the text must still come out in the
/// order they were typed, with the caret at the end of what was typed.
/// The keyboard brief (`formula_keyboard_e2e_test.dart`) waits for a
/// reading after every key; this file never does between the keys of a
/// word, which is how the composer is used.
///
/// Skipped when the binary is absent (run `cargo build`).
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/composer.dart' show composerInSync;
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
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
  late CanvasPointer canvas;
  late Directory dir;

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
    tester.view.physicalSize = const Size(1600, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    store = LiveStore(
      spawn: DaemonClient.spawn,
      executable: bdld!,
      // the real debounce: the keys of a word land inside it
      draftDebounce: const Duration(milliseconds: 150),
    );
    canvas = CanvasPointer(tester);
    await tester.pumpWidget(DesignHarness(store: store));
    await act(
      const AppStarted(),
      (s) => s.connection is Connected || s.connection is ConnectionFailed,
    );
    expect(store.state.connection, isA<Connected>());
    dir =
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-typing')) ??
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

  /// Type at [gap] between keys — a frame apart by default, well inside
  /// the debounce — without waiting for any reading.
  Future<void> type(
    WidgetTester tester,
    String text, {
    Duration gap = const Duration(milliseconds: 16),
  }) async {
    for (final c in text.split('')) {
      await tester.sendKeyEvent(keyOf(c), character: c);
      await tester.runAsync(() => Future<void>.delayed(gap));
      await tester.pump(gap);
    }
  }

  Rect caretRect(WidgetTester tester) =>
      tester.getRect(find.byKey(const ValueKey('composer-caret')));
  Rect fieldRect(WidgetTester tester) =>
      tester.getRect(find.byKey(const ValueKey('composer-field')));

  testWidgets('a word typed at a designer\'s pace keeps its order', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      await type(tester, 'tilt');
      expect(store.state.draft(id)!.source, 'tilt', reason: 'before any reading');
      expect(store.state.editor.composer.caret?.offset, 4);
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt');
      // the caret is drawn at the end of the word, not before it
      final word = tester.getRect(find.text('tilt').first);
      expect(caretRect(tester).left, greaterThan(word.right - 2));
      // keep typing without a pause: the operator, the number, its unit
      await type(tester, '/90 deg');
      expect(store.state.draft(id)!.source, 'tilt / 90 deg', reason: 'before any reading');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt / 90 deg');
      expect(store.state.draft(id)!.projection!.root.children[1].kind, 'quantity');
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('two keys in one frame both land, in order', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      await type(tester, 'ab', gap: Duration.zero);
      expect(store.state.draft(id)!.source, 'ab');
      await read(tester, id);
      await type(tester, 'cd', gap: Duration.zero);
      expect(store.state.draft(id)!.source, 'abcd');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'abcd');
      expect(store.state.editor.composer.caret?.offset, 4);
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('End reaches the end of the field; typing there appends', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      await type(tester, 'tilt/90');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt / 90');
      await tester.sendKeyEvent(LogicalKeyboardKey.home);
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.end);
      // the caret is measured after layout: one more frame
      await tester.pump();
      await tester.pump();
      final field = fieldRect(tester);
      final caret = caretRect(tester);
      final picture = tester.getRect(find.byKey(const ValueKey('struct-r')));
      expect(
        caret.left,
        greaterThanOrEqualTo(picture.right - 2),
        reason: 'after the whole formula',
      );
      expect(caret.left, lessThan(field.right));
      await type(tester, '+1');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt / 90 + 1');
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('a whole formula typed in one go: the structural keys wait their turn, in order', (
    tester,
  ) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      // `(` after a name is the compiler's (its arity); `/` is the
      // compiler's on the part before it; `,` moves on; the letters
      // between them never wait
      await type(tester, 'clamp(tilt/90 deg,0,1)');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'clamp(tilt / 90 deg, 0, 1)');
      expect(store.state.draft(id)!.projection!.complete, isTrue);
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('a sign in a slot, a two-character comparison, a negation', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      await type(tester, 'tilt>=45 deg');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt >= 45 deg');
      await tester.sendKeyEvent(LogicalKeyboardKey.home);
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.home);
      await tester.pump();
      // `!` before the whole comparison negates it; then a subtraction
      // whose slot takes a negative number
      await type(tester, '!');
      await read(tester, id);
      expect(store.state.draft(id)!.source, '!(tilt >= 45 deg)');
      await tester.sendKeyEvent(LogicalKeyboardKey.end);
      await tester.pump();
      await type(tester, '-');
      await read(tester, id);
      await type(tester, '-5');
      await read(tester, id);
      expect(store.state.draft(id)!.source, '!(tilt >= 45 deg) - -5');
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('text the compiler cannot read is mended as text', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      // the Text view lets anything in
      await tester.tap(find.text('Text'));
      await tester.pump();
      await tester.enterText(find.byKey(const ValueKey('definition-field')), 'tilt / 90 deg +');
      await settle(tester, (s) => s.draft(id)?.projection?.parseOk == false);
      await tester.tap(find.text('Formula'));
      await tester.pump();
      expect(find.byKey(const ValueKey('composer-out-of-sync')), findsOneWidget);
      // a click at the end of the text puts the caret there; ⌫ ⌫ mend it
      await tester.tapAt(
        tester.getRect(find.byKey(const ValueKey('composer-field'))).centerRight -
            const Offset(4, 0),
      );
      await tester.pump();
      for (var i = 0; i < 2; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
        await tester.pump(const Duration(milliseconds: 16));
      }
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt / 90 deg');
      expect(find.byKey(const ValueKey('composer-out-of-sync')), findsNothing);
      expect(find.byKey(const ValueKey('fraction-r')), findsOneWidget);
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('backspace at a designer\'s pace removes what was just typed', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      await tester.tap(find.byKey(const ValueKey('node-r')));
      await tester.pump();
      await type(tester, 'tilt');
      await read(tester, id);
      await type(tester, 'xy');
      for (var i = 0; i < 2; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.backspace);
        await tester.pump(const Duration(milliseconds: 16));
      }
      expect(store.state.draft(id)!.source, 'tilt');
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'tilt');
      expect(store.state.editor.composer.caret?.offset, 4);
    } finally {
      await teardown(tester);
    }
  });
}
