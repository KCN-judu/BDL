/// The Code view as an IDE surface, against the real `bdld` (protocol
/// 0.22): completion at the caret from the service's engine, a hover
/// card from the service's hover, definition across two files, the
/// references list, and formatting as one authored edit after which the
/// tokens are over the new text — and the same service path the formula
/// field takes, so both surfaces agree on what a name is.
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/ui/code/completion_popup.dart';
import 'package:bdl_studio/ui/code/hover_card.dart';
import 'package:bdl_studio/ui/pages/code_pane.dart';
import 'package:flutter/gestures.dart';
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

const _a = '''
concept Tilt : Angle
concept Brightness : Scalar
clock main
mapping tilt : () -> Tilt @main
mapping dimByTilt : Tilt -> Brightness
dimByTilt(t) = clamp(t / (90 deg), 0, 1)
''';

const _b = '''
mapping brightness : () -> Brightness @main
brightness() = dimByTilt(tilt)
''';

void main() {
  final bdld = _findBdld();

  late LiveStore store;
  late Directory dir;
  late CanvasPointer canvas;

  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);

  final field = find.descendant(of: find.byType(CodePane), matching: find.byType(TextField));
  TextEditingController controller(WidgetTester t) => t.widget<TextField>(field).controller!;

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
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-code-ide')) ??
        Directory.systemTemp;
    final root = p.join(dir.path, 'ws');
    Directory(p.join(root, 'src')).createSync(recursive: true);
    File(p.join(root, 'bdl.toml')).writeAsStringSync(
      'schema_version = 1\nname = "ws"\ncompiler_version = "test"\nkind = "text"\n',
    );
    File(p.join(root, 'src', 'a.bdl')).writeAsStringSync(_a);
    File(p.join(root, 'src', 'b.bdl')).writeAsStringSync(_b);
    await act(
      OpenProjectRequested(root),
      (s) => s.project != null && s.system != null && s.editor.pendingRequests == 0,
    );
    await tester.tap(find.text('Code'));
    await canvas.settle(store, (s) => s.editor.sources.revision == s.revision);
    await act(const SourceFileOpened('src/b.bdl'));
    await canvas.settle(
      store,
      (s) => s.editor.highlights[HighlightState.fileKey('src/b.bdl')]?.spans.isNotEmpty ?? false,
    );
    await tester.pump();
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  Future<void> ctrlSpace(WidgetTester t) async {
    await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.space);
    await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await t.pump();
  }

  testWidgets(
    'completion, hover, definition, references and format come from the service',
    (tester) async {
      await project(tester);
      try {
        expect(controller(tester).text, _b);

        // -- completion at the end of a new value's formula
        final typed = '${_b}mapping level : () -> Brightness @main\nlevel() = ';
        await tester.enterText(field, typed);
        await tester.pump();
        controller(tester).selection = TextSelection.collapsed(offset: typed.length);
        await ctrlSpace(tester);
        var s = await canvas.settle(
          store,
          (s) => s.editor.completion != null && !s.editor.completion!.pending,
        );
        final labels = s.editor.completion!.items.map((i) => i.label).toList();
        expect(labels, containsAll(['tilt', 'brightness', 'dimByTilt(Tilt)']));
        expect(labels, isNot(contains('level')));
        expect(labels.where((l) => l.startsWith('clamp(')), isNotEmpty);
        expect(labels, isNot(contains('dimByTilt')), reason: 'a rule is a call');
        await tester.pump();
        expect(find.byType(CompletionPopup), findsOneWidget);
        // pick the rule: the service's insert text lands, the caret after it
        final rule = s.editor.completion!.items.indexWhere((i) => i.label == 'dimByTilt(Tilt)');
        for (var i = 0; i < rule; i++) {
          await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
          await tester.pump();
        }
        await tester.sendKeyEvent(LogicalKeyboardKey.enter);
        await tester.pump();
        expect(controller(tester).text, '${typed}dimByTilt(');
        expect(find.byType(CompletionPopup), findsNothing);
        // the same text through the formula path names the same things
        final brightness = s.project!.mappings.firstWhere((m) => m.name == 'brightness');
        await act(
          CompletionRequested(mappingId: brightness.id.toInt(), source: '', offset: 0),
          (s) => s.editor.completion != null && !s.editor.completion!.pending,
        );
        final formulaLabels = store.state.editor.completion!.items.map((i) => i.label).toList();
        expect(formulaLabels, containsAll(['tilt', 'dimByTilt(Tilt)']));
        expect(formulaLabels, isNot(contains('brightness')), reason: 'its own name is a cycle');
        await act(const CompletionDismissed());

        // put the file back
        await tester.enterText(field, _b);
        await tester.pump(kSourceEditPause + const Duration(milliseconds: 50));
        s = await canvas.settle(store, (s) => s.editor.pendingRequests == 0);

        // -- hover over `dimByTilt` in the formula: the rule's card
        final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
        await gesture.addPointer(location: Offset.zero);
        addTearDown(gesture.removePointer);
        RenderEditable? found;
        void visit(RenderObject r) {
          if (found != null) return;
          if (r is RenderEditable) {
            found = r;
          } else {
            r.visitChildren(visit);
          }
        }

        visit(tester.renderObject(field));
        final editable = found!;
        final over = editable.getLocalRectForCaret(
          TextPosition(offset: _b.indexOf('dimByTilt') + 3),
        );
        await gesture.moveTo(editable.localToGlobal(over.center));
        await tester.pump(const Duration(milliseconds: 300));
        s = await canvas.settle(store, (s) => s.editor.hover?.card != null);
        final card = s.editor.hover!.card!;
        expect(card.found, isTrue);
        expect(card.title, 'dimByTilt');
        expect(card.details.any((d) => d.label == 'role' && d.value == 'Rule'), isTrue);
        await tester.pump();
        expect(find.byType(HoverCard), findsOneWidget);
        await gesture.moveTo(const Offset(1, 1));
        await tester.pump();

        // -- definition of `tilt` (line 2) is in a.bdl: the file opens there
        await tester.tap(field);
        await tester.pump();
        final tiltAt = _b.indexOf('tilt)');
        controller(tester).selection = TextSelection.collapsed(offset: tiltAt + 1);
        await tester.sendKeyEvent(LogicalKeyboardKey.f12);
        s = await canvas.settle(store, (s) => s.editor.sources.openPath == 'src/a.bdl');
        await tester.pump();
        final c = controller(tester);
        expect(c.text, _a);
        expect(c.text.substring(c.selection.start, c.selection.end), 'tilt');
        expect(c.selection.start, _a.indexOf('mapping tilt') + 'mapping '.length);

        // -- references to `Brightness` from a: both files
        c.selection = TextSelection.collapsed(offset: _a.indexOf('Brightness') + 2);
        await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
        await tester.sendKeyEvent(LogicalKeyboardKey.f12);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
        s = await canvas.settle(
          store,
          (s) => s.editor.references != null && !s.editor.references!.pending,
        );
        final refs = s.editor.references!;
        expect(refs.title, 'Brightness');
        expect(refs.locations.map((l) => l.path).toSet(), {'src/a.bdl', 'src/b.bdl'});
        await tester.pump();
        expect(find.textContaining('name Brightness'), findsOneWidget);
        await tester.tap(find.text('src/b.bdl:1'));
        s = await canvas.settle(store, (s) => s.editor.sources.openPath == 'src/b.bdl');
        await tester.pump();
        expect(
          controller(tester).text
              .substring(controller(tester).selection.start, controller(tester).selection.end),
          'Brightness',
        );
        await act(const ReferencesDismissed());

        // -- format: the canonical layout is one authored edit, then tokens
        final messy = _b.replaceFirst('mapping brightness :', 'mapping   brightness:');
        await tester.enterText(field, messy);
        await tester.pump(kSourceEditPause + const Duration(milliseconds: 50));
        s = await canvas.settle(
          store,
          (s) => s.editor.pendingRequests == 0 && s.editor.sources.text == messy,
        );
        final before = s.revision;
        await tester.tap(find.byKey(const ValueKey('format-source')));
        s = await canvas.settle(
          store,
          (s) =>
              s.revision > before &&
              s.editor.pendingRequests == 0 &&
              s.editor.sources.buffer == null,
        );
        expect(s.editor.sources.text, _b);
        await tester.pump();
        expect(controller(tester).text, _b);
        s = await canvas.settle(store, (s) {
          final h = s.editor.highlights[HighlightState.fileKey('src/b.bdl')];
          return h != null && h.text == _b && !h.pending;
        });
        expect(s.editor.highlights[HighlightState.fileKey('src/b.bdl')]!.spans, isNotEmpty);
        expect(store.failure, isNull);
      } finally {
        await tester.runAsync(teardown);
      }
    },
    skip: bdld == null,
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
