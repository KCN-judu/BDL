/// The Code view's IDE transitions (`app/code_tooling.dart`): completion,
/// hover, definition, references and format over a file's text as
/// typed, with the generation guards that keep an answer to an older
/// text from acting on a newer one; and the pane rendering the answers
/// — the pop-up at the caret, the card at the name, the references
/// list, the formatted text as one edit.  No test here classifies text.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/code/completion_popup.dart';
import 'package:bdl_studio/ui/code/hover_card.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/code_pane.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

const a = 'concept Tilt : Angle\nmapping tilt : () -> Tilt\n';
const b = 'mapping lean : () -> Tilt\nlean() = tilt\n';

pb.ProjectProjection project({int revision = 3}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/tmp/lamp')
      ..kind = pb.ProjectKind.PROJECT_KIND_TEXT
      ..concepts.add(pb.ConceptView(id: Int64(0), name: 'Tilt'));

pb.SourcesView sources({int revision = 3, String aText = a}) => pb.SourcesView(
  revision: Int64(revision),
  files: [
    pb.SourceFileView(path: 'src/a.bdl', text: aText),
    pb.SourceFileView(path: 'src/b.bdl', text: b),
  ],
);

AppState opened() {
  final connected = AppState(
    connection: Connected(executable: 'x', handshake: pb.HandshakeResponse(compatible: true)),
  );
  var s = reduce(connected, ProjectReceived(project())).state;
  s = reduce(s, SystemReceived(pb.SystemView(revision: Int64(3)))).state;
  s = reduce(s, const DesignViewChanged(DesignView.code)).state;
  s = reduce(s, SourcesReceived(sources())).state;
  return s;
}

pb.DraftCompletionItem item(String label, {String insert = '', int start = 0, int end = 0}) =>
    pb.DraftCompletionItem(
      label: label,
      kind: 'mapping',
      insert: insert.isEmpty ? label : insert,
      replaceStart: start,
      replaceEnd: end,
    );

pb.SourceLocationsResponse locations(String title, List<(String, int, int)> at, {int gen = 0}) =>
    pb.SourceLocationsResponse(
      revision: Int64(3),
      generation: Int64(gen),
      title: title,
      locations: [for (final (p, s, e) in at) pb.SourceLocation(path: p, start: s, end: e)],
    );

void main() {
  group('completion', () {
    test('a request is tagged per document and only the latest answer lands', () {
      var t = reduce(
        opened(),
        const SourceCompletionRequested(path: 'src/b.bdl', text: b, offset: 12),
      );
      final e = t.effects.single as CompleteSource;
      expect((e.path, e.text, e.offset), ('src/b.bdl', b, 12));
      final g1 = e.generation;
      expect(t.state.editor.completion?.path, 'src/b.bdl');
      expect(t.state.editor.completion?.mappingId, isNull);
      // typed further: a newer request keeps the rows and re-tags
      t = reduce(
        t.state,
        const SourceCompletionRequested(path: 'src/b.bdl', text: '${b}x', offset: 13),
      );
      final g2 = (t.effects.single as CompleteSource).generation;
      expect(g2, greaterThan(g1));
      // the old answer is dropped, the new one shown
      var s = reduce(
        t.state,
        SourceCompletionReceived(
          generation: g1,
          result: pb.SourceCompletionResponse(items: [item('stale')]),
        ),
      ).state;
      expect(s.editor.completion?.items, isEmpty);
      expect(s.editor.completion?.pending, isTrue);
      s = reduce(
        s,
        SourceCompletionReceived(
          generation: g2,
          result: pb.SourceCompletionResponse(items: [item('tilt'), item('lean')]),
        ),
      ).state;
      expect(s.editor.completion?.items.map((i) => i.label), ['tilt', 'lean']);
      expect(s.editor.completion?.pending, isFalse);
      // a formula answer never lands on a source pop-up
      s = reduce(
        s,
        CompletionReceived(
          generation: g2,
          result: pb.DraftCompletionResponse(revision: Int64(3), items: [item('nope')]),
        ),
      ).state;
      expect(s.editor.completion?.items.map((i) => i.label), ['tilt', 'lean']);
      // a failure under the generation closes it
      s = reduce(s, ToolingFailed(generation: g2, code: 'x', message: '')).state;
      expect(s.editor.completion, isNull);
    });
  });

  group('hover', () {
    test('asks once per spot, not again inside the card on show, and ends on null', () {
      var t = reduce(opened(), const SourceHoverRequested(path: 'src/b.bdl', text: b, offset: 9));
      final g = (t.effects.single as HoverSource).generation;
      expect(t.state.editor.hover?.path, 'src/b.bdl');
      // the same spot: nothing
      expect(
        reduce(t.state, const SourceHoverRequested(path: 'src/b.bdl', text: b, offset: 9)).effects,
        isEmpty,
      );
      final card = pb.DraftHoverResponse(
        revision: Int64(3),
        found: true,
        title: 'tilt',
        span: pb.SourceSpan(start: 9, end: 13),
      );
      var s = reduce(t.state, HoverReceived(generation: g, result: card)).state;
      expect(s.editor.hover?.card?.title, 'tilt');
      // a spot inside the card's span: nothing asked
      expect(
        reduce(s, const SourceHoverRequested(path: 'src/b.bdl', text: b, offset: 11)).effects,
        isEmpty,
      );
      // outside: asked again; a stale answer to the first is dropped
      t = reduce(s, const SourceHoverRequested(path: 'src/b.bdl', text: b, offset: 2));
      expect(t.effects, hasLength(1));
      s = reduce(t.state, HoverReceived(generation: g, result: card)).state;
      expect(s.editor.hover?.card, isNull);
      // null ends it
      s = reduce(s, const SourceHoverRequested(path: 'src/b.bdl', text: b, offset: null)).state;
      expect(s.editor.hover, isNull);
    });
  });

  group('navigation', () {
    test('a definition in another file opens it and sets the reveal once', () {
      var t = reduce(
        reduce(opened(), const SourceFileOpened('src/b.bdl')).state,
        const SourceDefinitionRequested(path: 'src/b.bdl', text: b, offset: 9),
      );
      final g = (t.effects.single as DefineSource).generation;
      // a stale answer (an older generation) moves nothing
      var s = reduce(
        t.state,
        SourceDefinitionReceived(
          generation: g - 1,
          result: locations('tilt', [('src/a.bdl', 29, 33)]),
        ),
      ).state;
      expect(s.editor.reveal, isNull);
      s = reduce(
        t.state,
        SourceDefinitionReceived(generation: g, result: locations('tilt', [('src/a.bdl', 29, 33)])),
      ).state;
      expect(
        s.editor.reveal?.location,
        const SourceLocation(path: 'src/a.bdl', start: 29, end: 33),
      );
      expect(s.editor.sources.openPath, 'src/a.bdl');
      // nothing found: nothing moves
      s = reduce(t.state, SourceDefinitionReceived(generation: g, result: locations('', []))).state;
      expect(s.editor.reveal, isNull);
      expect(s.editor.sources.openPath, 'src/b.bdl');
    });

    test('references list only the latest answer, and dismiss', () {
      var t = reduce(
        opened(),
        const SourceReferencesRequested(path: 'src/a.bdl', text: a, offset: 8),
      );
      final g = (t.effects.single as ReferencesSource).generation;
      expect(t.state.editor.references?.pending, isTrue);
      var s = reduce(
        t.state,
        SourceReferencesReceived(
          generation: g,
          result: locations('Tilt', [('src/a.bdl', 8, 12), ('src/b.bdl', 21, 25)]),
        ),
      ).state;
      expect(s.editor.references?.title, 'Tilt');
      expect(s.editor.references?.locations, hasLength(2));
      expect(s.editor.references?.pending, isFalse);
      s = reduce(s, const ReferencesDismissed()).state;
      expect(s.editor.references, isNull);
    });
  });

  group('format', () {
    test('the canonical text replaces the editor and goes as one edit — for the text asked', () {
      const messy = 'concept  Tilt:Angle\nmapping tilt : () -> Tilt\n';
      var s = reduce(opened(), SourcesReceived(sources(aText: messy))).state;
      s = reduce(s, const SourceFileOpened('src/a.bdl')).state;
      var t = reduce(s, const FormatSourceRequested(path: 'src/a.bdl', text: messy));
      final e = t.effects.single as FormatSource;
      expect(e.text, messy);
      final g = e.generation;
      expect(t.state.editor.sources.formatting, messy);
      // the designer typed meanwhile: the answer changes nothing
      final typed = reduce(t.state, const SourceTextChanged('src/a.bdl', '$messy// x\n')).state;
      final ignored = reduce(
        typed,
        FormatSourceReceived(
          generation: g,
          path: 'src/a.bdl',
          result: pb.FormatSourceResponse(formatted: true, text: a),
        ),
      );
      expect(ignored.state.editor.sources.text, '$messy// x\n');
      expect(ignored.effects, isEmpty);
      expect(ignored.state.editor.sources.formatting, isNull);
      // untouched: replaced, counted, and sent as one edit
      final applied = reduce(
        t.state,
        FormatSourceReceived(
          generation: g,
          path: 'src/a.bdl',
          result: pb.FormatSourceResponse(formatted: true, text: a),
        ),
      );
      expect(applied.state.editor.sources.text, a);
      expect(applied.state.editor.sources.replaced, 1);
      final edit = applied.effects.whereType<ApplySourceEdit>().single;
      expect(edit.text, a);
      expect(edit.path, 'src/a.bdl');
      // an older generation: nothing
      t = reduce(applied.state, const FormatSourceRequested(path: 'src/a.bdl', text: a));
      final again = reduce(
        t.state,
        FormatSourceReceived(
          generation: g,
          path: 'src/a.bdl',
          result: pb.FormatSourceResponse(formatted: true, text: 'zzz'),
        ),
      );
      expect(again.state.editor.sources.text, a);
      // not formatted (does not parse, or canonical): nothing
      final same = reduce(
        t.state,
        FormatSourceReceived(
          generation: (t.effects.single as FormatSource).generation,
          path: 'src/a.bdl',
          result: pb.FormatSourceResponse(formatted: false, text: a),
        ),
      );
      expect(same.effects, isEmpty);
      expect(same.state.editor.sources.replaced, 1);
    });
  });

  group('the pane', () {
    late AppState state;
    final effects = <Effect>[];
    late StateSetter refresh;

    Future<void> pump(WidgetTester t, AppState initial) async {
      state = initial;
      effects.clear();
      t.view.physicalSize = const Size(1200, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.reset);
      await t.pumpWidget(
        MaterialApp(
          theme: macTheme(Brightness.light),
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                refresh = setState;
                return CodePane(
                  state: state,
                  dispatch: (a) {
                    final tr = reduce(state, a);
                    state = tr.state;
                    effects.addAll(tr.effects);
                    setState(() {});
                  },
                );
              },
            ),
          ),
        ),
      );
      await t.pump();
    }

    void answer(WidgetTester t, AppAction a) {
      final tr = reduce(state, a);
      state = tr.state;
      effects.addAll(tr.effects);
      refresh(() {});
    }

    final field = find.descendant(of: find.byType(CodePane), matching: find.byType(TextField));

    testWidgets('⌃Space asks at the caret, shows the pop-up, Return inserts the service range', (
      t,
    ) async {
      var s = reduce(opened(), const SourceFileOpened('src/b.bdl')).state;
      await pump(t, s);
      await t.tap(field);
      await t.pump();
      final c = t.widget<TextField>(field).controller!;
      c.selection = const TextSelection.collapsed(offset: 39); // after `lean() = tilt`
      await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.space);
      await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await t.pump();
      final asked = effects.whereType<CompleteSource>().single;
      expect(asked.path, 'src/b.bdl');
      expect(asked.offset, 39);
      expect(asked.text, b);
      answer(
        t,
        SourceCompletionReceived(
          generation: asked.generation,
          result: pb.SourceCompletionResponse(
            items: [item('tilt', start: 35, end: 39), item('tiltValue', start: 35, end: 39)],
          ),
        ),
      );
      await t.pump();
      expect(find.byType(CompletionPopup), findsOneWidget);
      expect(find.text('tiltValue'), findsOneWidget);
      await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pump();
      expect(find.byType(CompletionPopup), findsNothing);
      expect(c.text, 'mapping lean : () -> Tilt\nlean() = tiltValue\n');
      expect(c.selection.baseOffset, 35 + 'tiltValue'.length);
      expect(state.editor.sources.buffer, c.text);
      // the diagnostics' marks and the highlight controller are untouched:
      // the editor still asks for its tokens after the pause
      await t.pump(const Duration(milliseconds: 200));
      expect(effects.whereType<FetchSemanticTokens>().last.text, c.text);
      // Esc with the pop-up open closes it only
      await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.space);
      await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await t.pump();
      answer(
        t,
        SourceCompletionReceived(
          generation: effects.whereType<CompleteSource>().last.generation,
          result: pb.SourceCompletionResponse(items: [item('x')]),
        ),
      );
      await t.pump();
      expect(find.byType(CompletionPopup), findsOneWidget);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(find.byType(CompletionPopup), findsNothing);
      expect(c.text, 'mapping lean : () -> Tilt\nlean() = tiltValue\n');
    });

    testWidgets('a dwell shows the card at the name; typing ends it', (t) async {
      final s = reduce(opened(), const SourceFileOpened('src/b.bdl')).state;
      await pump(t, s);
      final gesture = await t.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      // over `tilt` on the second line: x past the 10th column, y on line 2
      final origin = t.getTopLeft(field);
      await gesture.moveTo(origin + const Offset(16 + 7.3 * 10, 12 + 18 * 1.5));
      await t.pump(const Duration(milliseconds: 300));
      final asked = effects.whereType<HoverSource>().toList();
      expect(asked, hasLength(1));
      expect(asked.single.path, 'src/b.bdl');
      answer(
        t,
        HoverReceived(
          generation: asked.single.generation,
          result: pb.DraftHoverResponse(
            revision: Int64(3),
            found: true,
            title: 'tilt',
            signature: 'mapping tilt : () -> Tilt',
            span: pb.SourceSpan(start: 35, end: 39),
            details: [pb.HoverDetail(label: 'role', value: 'Source')],
          ),
        ),
      );
      await t.pump();
      expect(find.byType(HoverCard), findsOneWidget);
      expect(find.text('mapping tilt : () -> Tilt'), findsOneWidget);
      // typing ends the hover
      await t.enterText(field, '${b}x');
      await t.pump();
      expect(state.editor.hover, isNull);
      expect(find.byType(HoverCard), findsNothing);
    });

    testWidgets('F12 asks for the definition; ⇧F12 lists references; a row goes there', (t) async {
      final s = reduce(opened(), const SourceFileOpened('src/b.bdl')).state;
      await pump(t, s);
      await t.tap(field);
      await t.pump();
      final c = t.widget<TextField>(field).controller!;
      c.selection = const TextSelection.collapsed(offset: 36);
      await t.sendKeyEvent(LogicalKeyboardKey.f12);
      await t.pump();
      final def = effects.whereType<DefineSource>().single;
      expect((def.path, def.offset), ('src/b.bdl', 36));
      answer(
        t,
        SourceDefinitionReceived(
          generation: def.generation,
          result: locations('tilt', [('src/a.bdl', 29, 33)]),
        ),
      );
      await t.pump();
      // the other file is open and its range selected
      expect(state.editor.sources.openPath, 'src/a.bdl');
      final c2 = t.widget<TextField>(field).controller!;
      expect(c2.text, a);
      expect(c2.selection, const TextSelection(baseOffset: 29, extentOffset: 33));
      // references from here
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.f12);
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.pump();
      final refs = effects.whereType<ReferencesSource>().single;
      expect((refs.path, refs.offset), ('src/a.bdl', 29));
      answer(
        t,
        SourceReferencesReceived(
          generation: refs.generation,
          result: locations('tilt', [('src/a.bdl', 29, 33), ('src/b.bdl', 35, 39)]),
        ),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('references')), findsOneWidget);
      expect(find.textContaining('2 places name tilt'), findsOneWidget);
      expect(find.text('src/b.bdl:2'), findsOneWidget);
      await t.tap(find.text('src/b.bdl:2'));
      await t.pump();
      expect(state.editor.sources.openPath, 'src/b.bdl');
      final c3 = t.widget<TextField>(field).controller!;
      expect(c3.selection, const TextSelection(baseOffset: 35, extentOffset: 39));
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(find.byKey(const ValueKey('references')), findsNothing);
    });

    testWidgets('Format asks with the text on screen and shows the answer with the caret kept', (
      t,
    ) async {
      const messy = 'concept  Tilt:Angle\nmapping tilt : () -> Tilt\n';
      var s = reduce(opened(), SourcesReceived(sources(aText: messy))).state;
      s = reduce(s, const SourceFileOpened('src/a.bdl')).state;
      await pump(t, s);
      await t.tap(field);
      await t.pump();
      final c = t.widget<TextField>(field).controller!;
      // caret on line 2, column 8 (`tilt`)
      c.selection = const TextSelection.collapsed(offset: 20 + 8);
      await t.tap(find.byKey(const ValueKey('format-source')));
      await t.pump();
      final asked = effects.whereType<FormatSource>().single;
      expect(asked.text, messy);
      answer(
        t,
        FormatSourceReceived(
          generation: asked.generation,
          path: 'src/a.bdl',
          result: pb.FormatSourceResponse(formatted: true, text: a),
        ),
      );
      await t.pump();
      expect(c.text, a);
      expect(c.selection.baseOffset, 21 + 8);
      expect(effects.whereType<ApplySourceEdit>().single.text, a);
      // ⌥⇧F is the same request
      await t.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
      await t.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyEvent(LogicalKeyboardKey.keyF);
      await t.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await t.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
      await t.pump();
      expect(effects.whereType<FormatSource>(), hasLength(2));
    });
  });
}
