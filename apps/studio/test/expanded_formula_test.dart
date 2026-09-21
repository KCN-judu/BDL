/// Expanded formulas on canvas nodes (docs/architecture/studio-ui.md §2,
/// *Expanded formula*): a reading state of the editor — toggled from the
/// node's disclosure or its menu, the committed definition's projection
/// fetched per revision and never shown for another, the node grown by
/// the measured picture within a bound, the picture read only with the
/// first finding and one way into editing — and the structured forms the
/// same renderer draws: a choice as a branch, a match as its cases, a
/// block as its bindings, a temporal boundary as a marked region, each
/// with a reading for assistive technology.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/formula_render.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/canvas_harness.dart' show SceneLookup;
import 'support/roles.dart';

const tilt = 0;
const brightness = 1;
const dim = 5;

pb.ProjectProjection lamp({int revision = 1, String? definition}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(
          id: Int64(tilt),
          name: 'Tilt',
          representation: pb.Representation(quantity: pb.Dim(angle: 1)),
        ),
        pb.ConceptView(
          id: Int64(brightness),
          name: 'Brightness',
          representation: pb.Representation(quantity: pb.Dim()),
        ),
      ])
      ..mappings.add(
        mappingView(
          id: Int64(dim),
          name: 'dimByTilt',
          // a Sem block (ADR-0044): the formula is on its mapping block
          signature: pb.Signature(output: Int64(brightness)),
          definition: definition == null ? null : pb.Definition(formula: definition),
        ),
      );

AppState connected(pb.ProjectProjection p) => AppState(
  connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
  project: p,
  editor: EditorState(
    layout: {
      const NodeRef.mapping(dim): const Offset(300, 60),
      const NodeRef.concept(tilt): Offset.zero,
    },
  ),
);

pb.FormulaNode n(
  String id,
  String kind,
  String text,
  int start,
  int end, {
  String name = '',
  List<pb.FormulaNode> children = const [],
  String coordinate = '',
  String unit = '',
  List<String> binds = const [],
  pb.TypeView? expected,
}) => pb.FormulaNode(
  id: id,
  kind: kind,
  text: text,
  name: name,
  range: pb.SourceSpan(start: start, end: end),
  children: children,
  coordinate: coordinate,
  unit: unit,
  binds: binds,
  expected: expected,
);

pb.TypeView scalar() =>
    pb.TypeView(description: 'a dimensionless quantity', kind: 'quantity', dim: pb.Dim());

/// `if Tilt > 45 deg then 1 else 0`.
pb.FormulaProjection choice() => pb.FormulaProjection(
  source: 'if Tilt > 45 deg then 1 else 0',
  parseOk: true,
  complete: true,
  result: scalar(),
  root: n(
    'r',
    'if',
    'if Tilt > 45 deg then 1 else 0',
    0,
    30,
    expected: scalar(),
    children: [
      n(
        'r.0',
        'compare',
        'Tilt > 45 deg',
        3,
        16,
        name: '>',
        children: [
          n('r.0.0', 'reference', 'Tilt', 3, 7, name: 'Tilt'),
          n('r.0.1', 'quantity', '45 deg', 10, 16, coordinate: '45', unit: 'deg'),
        ],
      ),
      n('r.1', 'number', '1', 22, 23, coordinate: '1', expected: scalar()),
      n('r.2', 'number', '0', 29, 30, coordinate: '0', expected: scalar()),
    ],
  ),
);

/// `match Tilt { x => x / (90 deg), _ => 0 }`.
pb.FormulaProjection matchOf() => pb.FormulaProjection(
  source: 'match Tilt { x => x / (90 deg), _ => 0 }',
  parseOk: true,
  complete: true,
  root: n(
    'r',
    'match',
    'match Tilt { x => x / (90 deg), _ => 0 }',
    0,
    40,
    expected: scalar(),
    children: [
      n('r.0', 'reference', 'Tilt', 6, 10, name: 'Tilt'),
      n(
        'r.1',
        'arm',
        'x => x / (90 deg),',
        13,
        31,
        name: 'x',
        binds: ['x'],
        children: [
          n(
            'r.1.0',
            'binary',
            'x / (90 deg)',
            18,
            30,
            name: '/',
            children: [
              n('r.1.0.0', 'reference', 'x', 18, 19, name: 'x'),
              n('r.1.0.1', 'quantity', '(90 deg)', 22, 30, coordinate: '90', unit: 'deg'),
            ],
          ),
        ],
      ),
      n(
        'r.2',
        'arm',
        '_ => 0',
        32,
        38,
        name: '_',
        children: [n('r.2.0', 'number', '0', 37, 38, coordinate: '0')],
      ),
    ],
  ),
);

/// `{ let half = Tilt / 2; delay(0, half / (45 deg)) }`.
pb.FormulaProjection blockOf() => pb.FormulaProjection(
  source: '{ let half = Tilt / 2; delay(0, half / (45 deg)) }',
  parseOk: true,
  complete: true,
  root: n(
    'r',
    'block',
    '{ let half = Tilt / 2; delay(0, half / (45 deg)) }',
    0,
    52,
    children: [
      n(
        'r.0',
        'let',
        'let half = Tilt / 2;',
        2,
        22,
        name: 'half',
        binds: ['half'],
        children: [
          n(
            'r.0.0',
            'binary',
            'Tilt / 2',
            13,
            21,
            name: '/',
            children: [
              n('r.0.0.0', 'reference', 'Tilt', 13, 17, name: 'Tilt'),
              n('r.0.0.1', 'number', '2', 20, 21, coordinate: '2'),
            ],
          ),
        ],
      ),
      n(
        'r.1',
        'delay',
        'delay(0, half / (45 deg))',
        23,
        50,
        children: [
          n('r.1.0', 'number', '0', 29, 30, coordinate: '0'),
          n(
            'r.1.1',
            'binary',
            'half / (45 deg)',
            32,
            49,
            name: '/',
            children: [
              n('r.1.1.0', 'reference', 'half', 32, 36, name: 'half'),
              n('r.1.1.1', 'quantity', '(45 deg)', 39, 49, coordinate: '45', unit: 'deg'),
            ],
          ),
        ],
      ),
    ],
  ),
);

Widget app(Widget child) => MaterialApp(
  theme: macTheme(Brightness.light),
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: kSupportedLocales,
  home: Scaffold(body: child),
);

void main() {
  group('reducer', () {
    test('toggling asks for the committed projection; the answer is kept per revision', () {
      final s = connected(lamp(definition: 'Tilt / 90 deg'));
      final opened = reduce(s, const FormulaExpansionToggled(dim));
      expect(opened.state.editor.expandedFormulas[dim], EditorState.formulaInitialHeight);
      final e = opened.effects.whereType<GetFormulaPreview>().single;
      expect((e.mappingId, e.revision), (dim, 1));
      expect(opened.state.editor.formulaPreviews[dim]!.projection, isNull);
      // the answer, for the generation and revision asked
      final got = reduce(
        opened.state,
        FormulaPreviewReceived(
          generation: e.generation,
          response: pb.FormulaProjectionResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            projection: choice(),
          ),
        ),
      ).state;
      expect(got.editor.formulaPreviews[dim]!.projection!.source, choice().source);
      // an answer for another generation or revision is dropped
      final stale = reduce(
        got,
        FormulaPreviewReceived(
          generation: e.generation + 7,
          response: pb.FormulaProjectionResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            projection: pb.FormulaProjection(source: 'other', parseOk: true),
          ),
        ),
      ).state;
      expect(stale.editor.formulaPreviews[dim]!.projection!.source, choice().source);
      // the measured picture sets the node's height, within the bound
      final measured = reduce(
        got,
        const FormulaExpansionMeasured(mappingId: dim, height: 90),
      ).state;
      expect(measured.editor.expandedFormulas[dim], 90);
      final capped = reduce(
        got,
        const FormulaExpansionMeasured(mappingId: dim, height: 10000),
      ).state;
      expect(capped.editor.expandedFormulas[dim], EditorState.formulaMaxHeight);
      // toggling again folds it and drops the preview
      final closed = reduce(measured, const FormulaExpansionToggled(dim));
      expect(closed.state.editor.expandedFormulas, isEmpty);
      expect(closed.effects, isEmpty);
    });

    test('a new revision re-asks; a mapping that lost its definition folds', () {
      var s = reduce(
        connected(lamp(definition: 'Tilt / 90 deg')),
        const FormulaExpansionToggled(dim),
      ).state;
      final next = reduce(
        s,
        ProjectReceived(lamp(revision: 2, definition: 'Tilt / 45 deg'), fromRequest: false),
      );
      expect(next.state.editor.expandedFormulas.containsKey(dim), isTrue, reason: 'stays expanded');
      final again = next.effects.whereType<GetFormulaPreview>().single;
      expect(again.revision, 2);
      expect(next.state.editor.formulaPreviews[dim]!.revision, 2);
      expect(next.state.editor.formulaPreviews[dim]!.projection, isNull, reason: 'never the old');
      // the definition detached: nothing to show, nothing asked
      s = reduce(next.state, ProjectReceived(lamp(revision: 3), fromRequest: false)).state;
      expect(s.editor.expandedFormulas, isEmpty);
      expect(s.editor.formulaPreviews, isEmpty);
      // another project: nothing stays expanded
      final other = reduce(
        reduce(connected(lamp(definition: 'Tilt')), const FormulaExpansionToggled(dim)).state,
        ProjectReceived(lamp()..rootPath = '/q', fromRequest: false),
      ).state;
      expect(other.editor.expandedFormulas, isEmpty);
    });

    test(
      'the geometry: an expanded mapping node grows by the picture; the disclosure is a hit',
      () {
        final p = lamp(definition: 'Tilt / 90 deg');
        // the definition is the mapping block's (ADR-0044): it grows, the
        // Sem block does not
        const block = NodeRef.definition(dim);
        final at = {const NodeRef.mapping(dim): const Offset(400, 0), block: Offset.zero};
        final folded = buildScene(p, at);
        final open = buildScene(p, at, expanded: {dim: 80});
        final a = folded.node(block);
        final b = open.node(block);
        expect(a.expanded, isFalse);
        expect(b.expanded, isTrue);
        expect(b.rect.height, a.rect.height + 80);
        expect(b.formulaRegion.height, 80);
        expect(b.definitionRegion.top, a.definitionRegion.top, reason: 'the summary line stays');
        expect(hitTest(open, b.disclosure.center), isA<HitDisclosure>());
        expect(hitTest(open, b.formulaRegion.center), isA<HitNode>());
        expect(open.node(const NodeRef.mapping(dim)).expanded, isFalse);
        // a Sem block without a definition has no mapping block and never grows
        final declared = buildScene(lamp(), at, expanded: {dim: 80});
        expect(declared.nodes.any((n) => n.ref == block), isFalse);
        final d = declared.node(const NodeRef.mapping(dim));
        expect(d.expanded, isFalse);
        expect(hitTest(declared, d.disclosure.center), isA<HitNode>());
      },
    );
  });

  group('the picture', () {
    testWidgets('on the canvas: read only, the finding, the way into editing', (t) async {
      final actions = <AppAction>[];
      final p = lamp(definition: choice().source);
      await t.pumpWidget(
        app(
          SizedBox(
            width: 900,
            height: 500,
            child: NodeCanvas(
              project: p,
              layout: {const NodeRef.mapping(dim): const Offset(300, 60)},
              selection: const NoSelection(),
              dispatch: actions.add,
              expanded: {dim: 200},
              previews: {dim: FormulaPreview(revision: 1, generation: 1, projection: choice())},
              analyses: {
                dim: pb.MappingAnalysis(
                  id: Int64(dim),
                  status: pb.MappingStatus.MAPPING_STATUS_OPEN,
                  diagnostics: [
                    pb.Diagnostic(
                      code: 'x',
                      severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_INFO,
                      message: 'Checked once Tilt is decided.',
                    ),
                  ],
                ),
              },
            ),
          ),
        ),
      );
      await t.pump();
      await t.pump(const Duration(milliseconds: 50));
      expect(find.byKey(const ValueKey('expanded-$dim')), findsOneWidget);
      // the branch, not a row of text
      expect(find.byKey(const ValueKey('if-r')), findsOneWidget);
      expect(find.text('then'), findsOneWidget);
      expect(find.text('else'), findsOneWidget);
      expect(find.text(choice().source), findsNothing);
      expect(find.byKey(const ValueKey('expanded-finding-$dim')), findsOneWidget);
      expect(find.text('Checked once Tilt is decided.'), findsOneWidget);
      // measured once drawn
      expect(actions.whereType<FormulaExpansionMeasured>(), isNotEmpty);
      // a click on the picture selects the node and rewrites nothing
      await t.tap(find.byKey(const ValueKey('if-r')));
      await t.pump();
      expect(actions.whereType<SelectionChanged>().last.selection, const MappingSelected(dim));
      expect(actions.whereType<ComposeRequested>(), isEmpty);
      expect(actions.whereType<DefinitionDraftChanged>(), isEmpty);
      // the deliberate way into editing
      await t.tap(find.byKey(const ValueKey('expanded-edit-$dim')));
      await t.pump();
      expect(actions.whereType<EditDefinitionRequested>().single.mappingId, dim);
      // a preview still on its way says so; none at all says why
      await t.pumpWidget(
        app(
          SizedBox(
            width: 900,
            height: 500,
            child: NodeCanvas(
              project: p,
              layout: {const NodeRef.mapping(dim): const Offset(300, 60)},
              selection: const NoSelection(),
              dispatch: actions.add,
              expanded: {dim: 64},
              previews: {dim: const FormulaPreview(revision: 1, generation: 2)},
            ),
          ),
        ),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('expanded-note-$dim')), findsOneWidget);
      expect(find.text(kEnglish.formulaPreviewLoading), findsOneWidget);
    });

    testWidgets('a match, a block and a temporal boundary are structure, read aloud', (t) async {
      final concepts = {for (final c in lamp().concepts) c.id.toInt(): c};
      await t.pumpWidget(
        app(FormulaRender(projection: matchOf(), concepts: concepts, geometry: FormulaGeometry())),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('match-r')), findsOneWidget);
      expect(find.text('match'), findsOneWidget);
      expect(find.text('⇒'), findsNWidgets(2));
      // the arm's pattern is the arm's own (its name), not a part
      expect(find.byKey(const ValueKey('pattern-r.1')), findsOneWidget, reason: 'a pattern');
      expect(find.text('x'), findsNWidgets(2));
      expect(find.byKey(const ValueKey('fraction-r.1.0')), findsOneWidget);
      expect(find.text(matchOf().source), findsNothing);
      expect(find.byKey(const ValueKey('result-r')), findsOneWidget);
      // (a leaf's reading merges into its container's: the start is the part's)
      expect(find.bySemanticsLabel(RegExp('^a match on Tilt with 2 cases')), findsOneWidget);

      await t.pumpWidget(
        app(FormulaRender(projection: blockOf(), concepts: concepts, geometry: FormulaGeometry())),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('block-r')), findsOneWidget);
      expect(find.text('let'), findsOneWidget);
      expect(find.byKey(const ValueKey('pattern-r.0')), findsOneWidget);
      expect(find.byKey(const ValueKey('temporal-r.1')), findsOneWidget);
      expect(find.text('delay'), findsOneWidget);
      expect(find.bySemanticsLabel(RegExp('^a block with 1 local bindings')), findsOneWidget);
      expect(find.bySemanticsLabel(RegExp('^a delay boundary')), findsOneWidget);
      // the readings of the fraction and the choice
      expect(find.bySemanticsLabel(RegExp('^half over 45 deg')), findsOneWidget);
      await t.pumpWidget(
        app(FormulaRender(projection: choice(), concepts: concepts, geometry: FormulaGeometry())),
      );
      await t.pump();
      expect(
        find.bySemanticsLabel(RegExp('^a choice: if Tilt greater than 45 deg, then 1, else 0')),
        findsOneWidget,
      );
    });
  });
}
