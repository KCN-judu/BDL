/// Completion and hover through the reducer (state, generations, stale
/// answers) and the editor widget (pop-up, keys, accept, hover card).
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/mac/tokens.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

const tilt = 0;
const brightness = 1;
const dim = 0;

pb.ProjectProjection lamp({int revision = 1}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(tilt), name: 'Tilt'),
        pb.ConceptView(id: Int64(brightness), name: 'Brightness'),
      ])
      ..mappings.add(
        pb.MappingView(
          id: Int64(dim),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(tilt)], output: Int64(brightness)),
        ),
      );

AppState connected(pb.ProjectProjection project) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  editor: const EditorState(
    selection: MappingSelected(dim),
    // these tests exercise the Text projection of the draft
    composer: ComposerState(formulaMode: false),
  ),
);

pb.DraftCompletionItem item(String label, {String kind = 'input', int start = 0, int end = 0}) =>
    pb.DraftCompletionItem(
      label: label,
      kind: kind,
      insert: label,
      replaceStart: start,
      replaceEnd: end,
      resultingType: 'an angle',
    );

pb.DraftCompletionResponse completions(
  int revision,
  List<pb.DraftCompletionItem> items, {
  int mappingId = dim,
}) => pb.DraftCompletionResponse(
  revision: Int64(revision),
  mappingId: Int64(mappingId),
  items: items,
);

pb.DraftHoverResponse hoverCard(int revision, {bool found = true}) => pb.DraftHoverResponse(
  revision: Int64(revision),
  mappingId: Int64(dim),
  found: found,
  title: 'Tilt',
  representation: 'an angle',
  status: 'representation bound',
  details: [pb.HoverDetail(label: 'canonical unit', value: 'rad')],
);

class Harness extends StatefulWidget {
  const Harness({super.key, required this.initial});
  final AppState initial;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late AppState state = widget.initial;
  final List<Effect> effects = [];

  void dispatch(AppAction a) {
    setState(() {
      final t = reduce(state, a);
      state = t.state;
      effects.addAll(t.effects);
    });
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: Align(
        alignment: Alignment.topLeft,
        child: SizedBox(
          width: MacMetrics.inspectorWidth,
          height: 900,
          child: Inspector(state: state, dispatch: dispatch),
        ),
      ),
    ),
  );
}

Future<HarnessState> pump(WidgetTester tester, AppState initial) async {
  final key = GlobalKey<HarnessState>();
  await tester.pumpWidget(Harness(key: key, initial: initial));
  return key.currentState!;
}

Finder get field => find.descendant(
  of: find.byKey(const ValueKey('definition-field')),
  matching: find.byType(TextField),
);

Future<void> ctrlSpace(WidgetTester t) async {
  await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
  await t.sendKeyEvent(LogicalKeyboardKey.space);
  await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
  await t.pump();
}

void main() {
  group('completion state', () {
    test('a request tags a generation and asks the service at the byte offset', () {
      final t = reduce(
        connected(lamp()),
        const CompletionRequested(mappingId: dim, source: 'Ti', offset: 2),
      );
      final c = t.state.editor.completion!;
      expect((c.mappingId, c.generation, c.offset, c.pending), (dim, 1, 2, true));
      final e = t.effects.single as CompleteDraft;
      expect((e.revision, e.mappingId, e.source, e.offset, e.generation), (1, dim, 'Ti', 2, 1));
    });

    test('only the latest generation at the held revision is shown, in service order', () {
      var s = reduce(
        connected(lamp()),
        const CompletionRequested(mappingId: dim, source: 'T', offset: 1),
      ).state;
      s = reduce(s, const CompletionRequested(mappingId: dim, source: 'Ti', offset: 2)).state;
      s = reduce(s, const CompletionRequested(mappingId: dim, source: 'Til', offset: 3)).state;
      expect(s.editor.completion!.generation, 3);
      // answers arrive 2, 1, 3: only 3 lands
      s = reduce(s, CompletionReceived(generation: 2, result: completions(1, [item('two')]))).state;
      expect(s.editor.completion!.pending, isTrue);
      s = reduce(s, CompletionReceived(generation: 1, result: completions(1, [item('one')]))).state;
      expect(s.editor.completion!.items, isEmpty);
      s = reduce(
        s,
        CompletionReceived(
          generation: 3,
          result: completions(1, [
            item('rad', kind: 'unit'),
            item('Tilt'),
            item('deg', kind: 'unit'),
          ]),
        ),
      ).state;
      final c = s.editor.completion!;
      expect(c.pending, isFalse);
      expect(c.items.map((i) => i.label), ['rad', 'Tilt', 'deg'], reason: 'never re-sorted');
      // another revision's answer is ignored
      s = reduce(
        s,
        CompletionReceived(generation: 3, result: completions(2, [item('stale')])),
      ).state;
      expect(s.editor.completion!.items.first.label, 'rad');
    });

    test('moving wraps; dismiss, selection change and a new revision close the pop-up', () {
      var s = reduce(
        connected(lamp()),
        const CompletionRequested(mappingId: dim, source: '', offset: 0),
      ).state;
      s = reduce(
        s,
        CompletionReceived(generation: 1, result: completions(1, [item('a'), item('b')])),
      ).state;
      s = reduce(s, const CompletionMoved(-1)).state;
      expect(s.editor.completion!.selected, 1);
      s = reduce(s, const CompletionMoved(1)).state;
      expect(s.editor.completion!.selected, 0);
      expect(reduce(s, const CompletionDismissed()).state.editor.completion, isNull);
      expect(reduce(s, const SelectionChanged(NoSelection())).state.editor.completion, isNull);
      expect(
        reduce(s, ProjectReceived(lamp(revision: 2), fromRequest: false)).state.editor.completion,
        isNull,
      );
      // a failure just closes it
      expect(
        reduce(
          s,
          const ToolingFailed(generation: 1, code: 'draft.stale_revision', message: ''),
        ).state.editor.completion,
        isNull,
      );
    });
  });

  group('hover state', () {
    test('a formula hover is asked once per offset and its card kept by generation', () {
      var t = reduce(
        connected(lamp()),
        const FormulaHoverRequested(mappingId: dim, source: 'Tilt', offset: 1),
      );
      expect(t.effects.single, isA<HoverDraft>());
      final s1 = t.state;
      // same spot again: nothing new is asked
      expect(
        reduce(s1, const FormulaHoverRequested(mappingId: dim, source: 'Tilt', offset: 1)).effects,
        isEmpty,
      );
      t = reduce(s1, HoverReceived(generation: 1, result: hoverCard(1)));
      expect(t.state.editor.hover!.card!.title, 'Tilt');
      // an old generation's card cannot replace a newer request
      var s = reduce(
        t.state,
        const FormulaHoverRequested(mappingId: dim, source: 'Tilt', offset: 3),
      ).state;
      s = reduce(s, HoverReceived(generation: 1, result: hoverCard(1))).state;
      expect(s.editor.hover!.card, isNull);
      // leaving ends it
      s = reduce(
        s,
        const FormulaHoverRequested(mappingId: dim, source: 'Tilt', offset: null),
      ).state;
      expect(s.editor.hover, isNull);
    });

    test('an entity hover asks the service by identity', () {
      final entity = pb.EntityRef(mappingId: Int64(dim));
      final t = reduce(connected(lamp()), EntityHoverRequested(entity));
      final e = t.effects.single as HoverEntity;
      expect(e.entity, entity);
      expect(t.state.editor.hover!.entity, entity);
      expect(reduce(t.state, const EntityHoverRequested(null)).state.editor.hover, isNull);
    });
  });

  group('editor widget', () {
    testWidgets('⌃Space opens the pop-up; arrows move; Enter inserts the service range', (t) async {
      final h = await pump(t, connected(lamp()));
      await t.enterText(field, 'Ti');
      await t.pump();
      await ctrlSpace(t);
      final c = h.state.editor.completion!;
      expect((c.source, c.offset), ('Ti', 2));
      expect(find.byKey(const ValueKey('completion-popup')), findsOneWidget);
      h.dispatch(
        CompletionReceived(
          generation: c.generation,
          result: completions(1, [item('Tilt', end: 2), item('time', kind: 'unit', end: 2)]),
        ),
      );
      await t.pump();
      final popup = find.byKey(const ValueKey('completion-popup'));
      expect(find.descendant(of: popup, matching: find.text('Tilt')), findsOneWidget);
      expect(find.descendant(of: popup, matching: find.text('time')), findsOneWidget);
      await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await t.pump();
      expect(h.state.editor.completion!.selected, 1);
      await t.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      await t.pump();
      expect(h.state.editor.completion!.selected, 0);
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pump();
      expect(t.widget<TextField>(field).controller!.text, 'Tilt');
      expect(h.state.draft(dim)!.source, 'Tilt');
      expect(h.state.editor.completion, isNull);
      expect(find.byKey(const ValueKey('completion-popup')), findsNothing);
    });

    testWidgets('Esc closes the pop-up and leaves the draft; typing re-asks', (t) async {
      final h = await pump(t, connected(lamp()));
      await t.enterText(field, 'T');
      await t.pump();
      await ctrlSpace(t);
      h.dispatch(CompletionReceived(generation: 1, result: completions(1, [item('Tilt', end: 1)])));
      await t.pump();
      await t.enterText(field, 'Ti');
      await t.pump();
      expect(h.effects.whereType<CompleteDraft>().last.source, 'Ti');
      expect(h.effects.whereType<CompleteDraft>().last.offset, 2);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(h.state.editor.completion, isNull);
      expect(h.state.draft(dim)!.source, 'Ti', reason: 'Esc closed the pop-up, not the draft');
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(h.state.draft(dim), isNull, reason: 'the second Esc reverts');
    });

    testWidgets('accepting a candidate with a non-ASCII prefix replaces the right code units', (
      t,
    ) async {
      final h = await pump(t, connected(lamp()));
      // `×` is two bytes; the service's range is in bytes.
      await t.enterText(field, '2 × Ti');
      await t.pump();
      await ctrlSpace(t);
      expect(h.state.editor.completion!.offset, 7);
      h.dispatch(
        CompletionReceived(generation: 1, result: completions(1, [item('Tilt', start: 5, end: 7)])),
      );
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.tab);
      await t.pump();
      expect(t.widget<TextField>(field).controller!.text, '2 × Tilt');
    });

    testWidgets('a hover card shows the service\'s meaning, and only when found', (t) async {
      final h = await pump(t, connected(lamp()));
      await t.enterText(field, 'Tilt / 90 deg');
      await t.pump();
      h.dispatch(const FormulaHoverRequested(mappingId: dim, source: 'Tilt / 90 deg', offset: 1));
      h.dispatch(HoverReceived(generation: 1, result: hoverCard(1)));
      await t.pump();
      expect(find.byKey(const ValueKey('hover-card')), findsOneWidget);
      expect(find.text('an angle'), findsOneWidget);
      expect(find.text('rad'), findsOneWidget);
      h.dispatch(const FormulaHoverRequested(mappingId: dim, source: 'Tilt / 90 deg', offset: 8));
      h.dispatch(HoverReceived(generation: 2, result: hoverCard(1, found: false)));
      await t.pump();
      expect(find.byKey(const ValueKey('hover-card')), findsNothing);
    });
  });
}
