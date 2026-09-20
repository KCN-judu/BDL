/// The Formula view's typed structure end to end — the real Studio stack
/// (widgets, reducer, executor, Dart client) against the real `bdld`: the
/// UX tasks of the authoring brief, driven by the keys and the pointer as
/// a designer would, with the counts a UX comparison reads.
///
///   C. `Brightness = clamp(Tilt / 90 deg, 0, 1)` by keyboard only;
///   E. the same by keyboard and pointer mixed;
///   F. a conditional, read back as a branch;
///   G. an existing mapping reopened and its formula inspected on the
///      canvas; undo and redo of the saved definition.
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

/// What a UX comparison counts for one task.
class Tally {
  int keystrokes = 0;
  int pointer = 0;
  int corrections = 0;
  int modeSwitches = 0;
  int completions = 0;
  int wrongUnit = 0;
  int navigationFailures = 0;

  @override
  String toString() =>
      'keys=$keystrokes pointer=$pointer corrections=$corrections '
      'modeSwitches=$modeSwitches completions=$completions wrongUnit=$wrongUnit '
      'navigationFailures=$navigationFailures';
}

void main() {
  final bdld = _findBdld();
  late LiveStore store;
  late CanvasPointer canvas;
  late Directory dir;
  late String root;

  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();

  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);

  /// Let the real daemon answer and the fake clock advance (a typed
  /// draft's check is debounced on a timer of the test zone), then
  /// rebuild — until [test] holds.
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

  /// The compiler has read the text on screen: the picture is current.
  Future<AppState> read(WidgetTester tester, int id) =>
      settle(tester, (s) => composerInSync(s, id));

  Future<void> project(WidgetTester tester) async {
    tester.view.physicalSize = const Size(1600, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    store = LiveStore(
      spawn: DaemonClient.spawn,
      executable: bdld!,
      draftDebounce: const Duration(milliseconds: 10),
    );
    canvas = CanvasPointer(tester);
    await tester.pumpWidget(DesignHarness(store: store));
    await act(
      const AppStarted(),
      (s) => s.connection is Connected || s.connection is ConnectionFailed,
    );
    expect(store.state.connection, isA<Connected>());
    dir =
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-keys')) ??
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
    await act(
      CreateMappingRequested(
        name: 'dimByTilt',
        inputs: [conceptId('Tilt')],
        output: conceptId('Brightness'),
      ),
    );
    await act(SelectionChanged(MappingSelected(mappingId('dimByTilt'))), (s) => s.analysis != null);
    await tester.pump(const Duration(milliseconds: 100));
  }

  /// In the real zone: after a failure nothing pumps the fake one.
  Future<void> teardown(WidgetTester tester) => tester.runAsync(() async {
    await store.dispose();
    await dir.delete(recursive: true);
  });

  /// One key, then the compiler's reading of what it made (a text edit
  /// or a structured action): the picture is current again before the
  /// next key, as a designer's keystrokes are read one by one.
  Future<void> key(
    WidgetTester tester,
    int id,
    Tally t,
    LogicalKeyboardKey k, {
    String? char,
  }) async {
    t.keystrokes++;
    await tester.sendKeyEvent(k, character: char);
    await tester.pump();
    // a structured key is answered by the compiler first, then read
    await settle(tester, (s) => !s.editor.composer.pendingCompose);
    await read(tester, id);
  }

  Future<void> type(WidgetTester tester, int id, Tally t, String text) async {
    for (final c in text.split('')) {
      // the logical key is the physical key's on a US layout; what the
      // composer reads is the character
      final k = switch (c) {
        ' ' => LogicalKeyboardKey.space,
        '(' => LogicalKeyboardKey.digit9,
        ')' => LogicalKeyboardKey.digit0,
        ',' => LogicalKeyboardKey.comma,
        '/' => LogicalKeyboardKey.slash,
        '+' || '=' => LogicalKeyboardKey.equal,
        '*' => LogicalKeyboardKey.digit8,
        '&' => LogicalKeyboardKey.digit7,
        '-' => LogicalKeyboardKey.minus,
        '<' => LogicalKeyboardKey.comma,
        '>' => LogicalKeyboardKey.period,
        _ =>
          LogicalKeyboardKey.findKeyByKeyId(c.toLowerCase().codeUnitAt(0)) ??
              LogicalKeyboardKey.keyA,
      };
      await key(tester, id, t, k, char: c);
    }
  }

  Future<void> focusComposer(WidgetTester tester, Tally t) async {
    t.pointer++;
    await tester.tap(find.byKey(const ValueKey('node-r')));
    await tester.pump();
  }

  testWidgets('C. Brightness = clamp(Tilt / 90 deg, 0, 1) by keyboard only', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      final t = Tally();
      expect(store.state.editor.composer.formulaMode, isTrue, reason: 'Formula view, no switch');
      // the empty slot takes the keyboard on one click (the one pointer act
      // of this task; Tab from the Reads section would do the same)
      await focusComposer(tester, t);
      // `clamp` letter by letter: text, read by the compiler as a name;
      // completion opens on the first letter and offers the equation
      await type(tester, id, t, 'clamp');
      expect(store.state.draft(id)!.source, 'clamp');
      await settle(tester, (s) => s.editor.completion?.pending == false);
      expect(
        store.state.editor.completion?.items.any((i) => i.label.startsWith('clamp(')),
        isTrue,
        reason: 'the equation, by its shape',
      );
      t.completions++;
      // `(` applies the name: the compiler's arity, the caret in the first slot
      await type(tester, id, t, '(');
      expect(store.state.draft(id)!.source, 'clamp(?, ?, ?)');
      expect(store.state.editor.composer.selectedNode, 'r.0');
      await type(tester, id, t, 'Tilt');
      expect(store.state.draft(id)!.source, 'clamp(Tilt, ?, ?)');
      // `/` is structure: the compiler puts the quotient with its slot
      await type(tester, id, t, '/');
      expect(store.state.draft(id)!.source, 'clamp(Tilt / ?, ?, ?)');
      await type(tester, id, t, '90');
      expect(store.state.draft(id)!.source, 'clamp(Tilt / 90, ?, ?)');
      // a space after a number starts its unit; the unit is typed, the
      // compiler reads `90 deg` as a quantity
      await type(tester, id, t, ' deg');
      expect(store.state.draft(id)!.source, 'clamp(Tilt / 90 deg, ?, ?)');
      final quotient = store.state.draft(id)!.projection!.root.children[0];
      expect(quotient.children[1].kind, 'quantity');
      expect(quotient.children[1].unitId, 'angle.deg');
      // `,` moves to the next argument, out of the fraction
      await type(tester, id, t, ',0,1');
      expect(store.state.draft(id)!.source, 'clamp(Tilt / 90 deg, 0, 1)');
      final d = store.state.draft(id)!;
      expect(d.analysis!.status, isNot(pb.MappingStatus.MAPPING_STATUS_INVALID));
      expect(d.projection!.complete, isTrue);
      // ⌘↩ saves
      t.keystrokes++;
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      await settle(tester, (s) => s.mapping(id)!.hasDefinition());
      expect(store.state.mapping(id)!.definition.formula, 'clamp(Tilt / 90 deg, 0, 1)');
      expect(t.modeSwitches, 0);
      expect(t.corrections, 0);
      expect(t.wrongUnit, 0);
      // ignore: avoid_print
      print('task C (keyboard only): $t');
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('E. the same by keyboard and pointer mixed; undo and redo', (tester) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      final t = Tally();
      await focusComposer(tester, t);
      // the quotient by keys
      await type(tester, id, t, 'Tilt/90 deg');
      expect(store.state.draft(id)!.source, 'Tilt / 90 deg');
      // the palette wraps it in clamp: select the fraction by its rule,
      // then Function ▸ clamp
      t.pointer++;
      await tester.tap(find.byKey(const ValueKey('fraction-r')));
      await settle(tester, (s) => s.editor.composer.slot?.nodeId == 'r');
      t.pointer++;
      await tester.tap(find.byKey(const ValueKey('op-function')));
      await tester.pumpAndSettle();
      t.pointer++;
      await tester.tap(find.textContaining('clamp(').last);
      await tester.pump();
      await settle(tester, (s) => s.draft(id)!.source != 'Tilt / 90 deg');
      await settle(tester, (s) => !s.editor.composer.pendingCompose);
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'clamp(Tilt / 90 deg, ?, ?)');
      // the compiler selected the first new slot and put the caret in it:
      // the keys continue from there
      expect(store.state.editor.composer.selectedNode, 'r.1');
      await type(tester, id, t, '0,1');
      expect(store.state.draft(id)!.source, 'clamp(Tilt / 90 deg, 0, 1)');
      // Text view shows the same draft, byte for byte
      t.modeSwitches++;
      await tester.tap(find.text('Text'));
      await tester.pump();
      expect(find.text('clamp(Tilt / 90 deg, 0, 1)'), findsOneWidget);
      t.modeSwitches++;
      await tester.tap(find.text('Formula'));
      await tester.pump();
      await read(tester, id);
      // save, undo, redo: the committed definition, the Formula view
      // following it
      await act(CommitDefinitionRequested(id), (s) => s.mapping(id)!.hasDefinition());
      final saved = store.state.revision;
      await act(
        const UndoRequested(),
        (s) => s.revision > saved && !s.mapping(id)!.hasDefinition(),
      );
      expect(store.state.draft(id), isNull);
      await act(const RedoRequested(), (s) => s.mapping(id)!.hasDefinition());
      expect(store.state.mapping(id)!.definition.formula, 'clamp(Tilt / 90 deg, 0, 1)');
      // ignore: avoid_print
      print('task E (mixed): $t');
    } finally {
      await teardown(tester);
    }
  });

  testWidgets('F. a conditional is a branch; G. reopened and inspected on the canvas', (
    tester,
  ) async {
    if (bdld == null) return;
    await project(tester);
    try {
      final id = mappingId('dimByTilt');
      final t = Tally();
      await focusComposer(tester, t);
      await settle(tester, (s) => s.editor.composer.slot != null);
      // the palette's Choose on the empty slot: the compiler's form
      t.pointer++;
      await tester.tap(find.byKey(const ValueKey('op-choose')));
      await tester.pump();
      await settle(tester, (s) => (s.draft(id)?.source ?? '').isNotEmpty);
      await settle(tester, (s) => !s.editor.composer.pendingCompose);
      await read(tester, id);
      expect(store.state.draft(id)!.source, 'if ? then ? else ?');
      expect(find.byKey(const ValueKey('if-r')), findsOneWidget, reason: 'a branch, not a row');
      expect(find.byKey(const ValueKey('result-r')), findsOneWidget);
      // the condition: Tilt > 45 deg, by keys, from the caret the compiler
      // put in the condition slot
      expect(store.state.editor.composer.selectedNode, 'r.0');
      await type(tester, id, t, 'Tilt>45 deg');
      expect(store.state.draft(id)!.source, 'if Tilt > 45 deg then ? else ?');
      // Tab to the next slot: the `then` outcome; then the `else`
      await key(tester, id, t, LogicalKeyboardKey.tab);
      await type(tester, id, t, '1');
      await key(tester, id, t, LogicalKeyboardKey.tab);
      await type(tester, id, t, '0');
      expect(store.state.draft(id)!.source, 'if Tilt > 45 deg then 1 else 0');
      await act(CommitDefinitionRequested(id), (s) => s.mapping(id)!.hasDefinition());
      // ignore: avoid_print
      print('task F (conditional): $t');

      // G. reopen the project; the mapping's formula is folded on its
      // node, expanded on demand, read as a branch, with its edit action
      await act(const SaveRequested(), (s) => !s.flat!.dirty && s.editor.pendingRequests == 0);
      await act(const CloseProjectRequested(), (s) => s.project == null);
      await act(
        OpenProjectRequested(root),
        (s) => s.project != null && s.system != null && s.editor.pendingRequests == 0,
      );
      final reopened = mappingId('dimByTilt');
      expect(store.state.editor.expandedFormulas, isEmpty);
      final scene = canvas.scene(store.state);
      final node = scene.node(NodeRef.mapping(reopened));
      expect(node.expanded, isFalse);
      expect(node.definition, 'if Tilt > 45 deg then 1 else 0', reason: 'the collapsed summary');
      // the disclosure at the definition line's right end: one click
      await tester.tapAt(canvas.toGlobal(node.disclosure.center));
      final expanded = await settle(
        tester,
        (s) => s.editor.formulaPreviews[reopened]?.projection != null,
      );
      expect(expanded.editor.expandedFormulas.containsKey(reopened), isTrue);
      expect(expanded.editor.selection, isNot(MappingSelected(reopened)), reason: 'reading only');
      await tester.pump(const Duration(milliseconds: 100));
      expect(find.byKey(ValueKey('expanded-$reopened')), findsOneWidget);
      expect(find.byKey(const ValueKey('if-r')), findsOneWidget, reason: 'the branch, on the node');
      // the node grew by the picture's measured height, within the bound
      await settle(
        tester,
        (s) => s.editor.expandedFormulas[reopened] != EditorState.formulaInitialHeight,
      );
      final grown = buildScene(
        store.state.project!,
        store.state.editor.layout,
        expanded: store.state.editor.expandedFormulas,
      ).node(NodeRef.mapping(reopened));
      expect(grown.formulaHeight, greaterThan(NodeMetrics.bodyHeight));
      expect(grown.formulaHeight, lessThanOrEqualTo(NodeMetrics.formulaMaxHeight));
      // the picture stays while the designer navigates: select a concept
      await act(SelectionChanged(ConceptSelected(conceptId('Tilt'))));
      expect(store.state.editor.expandedFormulas.containsKey(reopened), isTrue);
      // the deliberate way into editing: the inspector's editor takes focus
      await tester.tap(find.byKey(ValueKey('expanded-edit-$reopened')));
      await settle(tester, (s) => s.editor.selection == MappingSelected(reopened));
      expect(store.state.editor.definitionFocus, greaterThan(0));
      // hide it again from the menu
      await canvas.rightClick(grown.header.center);
      await canvas.menu('Hide Formula');
      await tester.pump();
      expect(store.state.editor.expandedFormulas, isEmpty);
    } finally {
      await teardown(tester);
    }
  });
}
