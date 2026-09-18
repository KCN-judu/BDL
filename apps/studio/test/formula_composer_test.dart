/// The Formula Composer: the reducer's transitions (mode, selection, slot
/// queries, structured actions, stale generations) and the widget over a
/// scripted projection (toggle, selecting, filling a slot, the unit picker
/// filtered by the compiler, a value-preserving unit switch, the two round
/// trips, the invalid-text fallback, save/revert/conflict unchanged).
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/definition_editor.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

const tilt = 0;
const brightness = 1;
const dim = 0;

pb.ProjectProjection lamp({int revision = 1, String? definition}) {
  final p = pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
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
      pb.MappingView(
        id: Int64(dim),
        name: 'dimByTilt',
        signature: pb.Signature(inputs: [Int64(tilt)], output: Int64(brightness)),
        definition: definition == null ? null : pb.Definition(formula: definition),
      ),
    );
  return p;
}

AppState connected(pb.ProjectProjection project) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  editor: const EditorState(selection: MappingSelected(dim)),
);

pb.TypeView angle() =>
    pb.TypeView(description: 'an angle', kind: 'quantity', dim: pb.Dim(angle: 1));
pb.TypeView scalar() =>
    pb.TypeView(description: 'a dimensionless quantity', kind: 'quantity', dim: pb.Dim());

/// `Tilt / ?` as the compiler projects it.
pb.FormulaProjection tiltOverSlot() => pb.FormulaProjection(
  source: 'Tilt / ?',
  parseOk: true,
  complete: false,
  slots: ['r.1'],
  result: pb.TypeView(
    description: 'a Brightness (a dimensionless quantity)',
    kind: 'concept',
    dim: pb.Dim(),
    conceptId: Int64(brightness),
  ),
  root: pb.FormulaNode(
    id: 'r',
    kind: 'binary',
    name: '/',
    text: 'Tilt / ?',
    range: pb.SourceSpan(start: 0, end: 8),
    expected: scalar(),
    children: [
      pb.FormulaNode(
        id: 'r.0',
        kind: 'reference',
        name: 'Tilt',
        text: 'Tilt',
        range: pb.SourceSpan(start: 0, end: 4),
        entity: pb.EntityRef(conceptId: Int64(tilt)),
        actual: pb.TypeView(
          description: 'a Tilt',
          kind: 'concept',
          dim: pb.Dim(angle: 1),
          conceptId: Int64(tilt),
        ),
        expected: angle(),
      ),
      pb.FormulaNode(
        id: 'r.1',
        kind: 'slot',
        text: '?',
        range: pb.SourceSpan(start: 7, end: 8),
        expected: angle(),
        because: 'an angle ÷ an angle = a dimensionless quantity',
        diagnostics: [
          pb.Diagnostic(
            code: 'formula.slot.empty',
            severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
            mappingId: Int64(dim),
            message: 'This slot is empty; it expects an angle.',
            span: pb.SourceSpan(start: 7, end: 8),
          ),
        ],
      ),
    ],
  ),
);

/// `Tilt / (45 deg)` complete.
pb.FormulaProjection tiltOver45() => pb.FormulaProjection(
  source: 'Tilt / (45 deg)',
  parseOk: true,
  complete: true,
  root: pb.FormulaNode(
    id: 'r',
    kind: 'binary',
    name: '/',
    text: 'Tilt / (45 deg)',
    range: pb.SourceSpan(start: 0, end: 15),
    actual: scalar(),
    children: [
      pb.FormulaNode(
        id: 'r.0',
        kind: 'reference',
        name: 'Tilt',
        text: 'Tilt',
        range: pb.SourceSpan(start: 0, end: 4),
        entity: pb.EntityRef(conceptId: Int64(tilt)),
      ),
      pb.FormulaNode(
        id: 'r.1',
        kind: 'quantity',
        text: '(45 deg)',
        coordinate: '45',
        unit: 'deg',
        unitId: 'angle.deg',
        range: pb.SourceSpan(start: 7, end: 15),
        actual: angle(),
        expected: angle(),
      ),
    ],
  ),
);

pb.FormulaSlotResponse angleSlot(String node) => pb.FormulaSlotResponse(
  revision: Int64(1),
  mappingId: Int64(dim),
  nodeId: node,
  expected: angle(),
  explanation: 'Expected: an angle, because an angle ÷ an angle = a dimensionless quantity.',
  technical: 'expected q[angle]',
  units: [
    pb.UnitCandidate(id: 'angle.rad', symbol: 'rad', measures: 'an angle'),
    pb.UnitCandidate(id: 'angle.deg', symbol: 'deg', measures: 'an angle'),
    pb.UnitCandidate(id: 'angle.turn', symbol: 'turn', measures: 'an angle'),
  ],
  references: [
    pb.ReferenceCandidate(
      label: 'Tilt',
      insert: 'Tilt',
      produces: 'Tilt (an angle)',
      relevance: 90,
    ),
  ],
  equations: [
    pb.EquationCandidate(
      name: 'clamp',
      shape: 'clamp(x, low, high)',
      insert: 'clamp(?, ?, ?)',
      summary: 'x held within low and high',
    ),
  ],
);

pb.DefinitionDraftAnalysis verdict({
  required int generation,
  required pb.FormulaProjection projection,
  bool parseOk = true,
  pb.MappingStatus status = pb.MappingStatus.MAPPING_STATUS_INVALID,
}) => pb.DefinitionDraftAnalysis(
  revision: Int64(1),
  mappingId: Int64(dim),
  generation: Int64(generation),
  parseOk: parseOk,
  analysis: pb.MappingAnalysis(id: Int64(dim), status: status),
  projection: projection,
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
  final List<AppAction> actions = [];

  void dispatch(AppAction a) {
    setState(() {
      actions.add(a);
      final t = reduce(state, a);
      state = t.state;
      effects.addAll(t.effects);
    });
  }

  /// The daemon answered: apply a response as the executor would.
  void answer(AppAction a) => dispatch(a);

  @override
  Widget build(BuildContext context) {
    final m = state.project!.mappings.single;
    final concepts = {for (final c in state.project!.concepts) c.id.toInt(): c};
    final draft = state.draft(dim);
    final projection =
        draft?.projection ??
        (state.editor.composer.mappingId == dim ? state.editor.composer.projection : null);
    return MaterialApp(
      theme: macTheme(Brightness.light),
      home: Scaffold(
        body: Align(
          alignment: Alignment.topLeft,
          child: SizedBox(
            width: 360,
            child: SingleChildScrollView(
              child: DefinitionEditor(
                mappingId: dim,
                committed: m.hasDefinition() ? m.definition.formula : null,
                draft: draft,
                committedAnalysis: null,
                inputNames: const ['Tilt'],
                composer: state.editor.composer,
                projection: projection,
                concepts: concepts,
                dispatch: dispatch,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

Future<HarnessState> pump(WidgetTester tester, AppState initial) async {
  final key = GlobalKey<HarnessState>();
  await tester.pumpWidget(Harness(key: key, initial: initial));
  await tester.pump();
  return key.currentState!;
}

void main() {
  group('reducer', () {
    test('the mode is a preference of the editor and clears the selection', () {
      var s = connected(lamp());
      expect(s.editor.composer.formulaMode, isTrue);
      s = reduce(s, const FormulaModeChanged(false)).state;
      expect(s.editor.composer.formulaMode, isFalse);
      s = reduce(s, const FormulaModeChanged(true)).state;
      expect(s.editor.composer.formulaMode, isTrue);
    });

    test('selecting a node asks the service what fits, with the draft source', () {
      var s = connected(lamp());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / ?')).state;
      final t = reduce(s, const FormulaNodeSelected(mappingId: dim, nodeId: 'r.1'));
      expect(t.state.editor.composer.selectedNode, 'r.1');
      final e = t.effects.single as GetFormulaSlot;
      expect((e.mappingId, e.source, e.nodeId), (dim, 'Tilt / ?', 'r.1'));
      // the answer for that generation and node lands; an older one is dropped
      final ok = reduce(
        t.state,
        FormulaSlotReceived(generation: e.generation, result: angleSlot('r.1')),
      ).state;
      expect(ok.editor.composer.slot!.units.map((u) => u.symbol), ['rad', 'deg', 'turn']);
      final stale = reduce(
        t.state,
        FormulaSlotReceived(generation: e.generation - 1, result: angleSlot('r.1')),
      ).state;
      expect(stale.editor.composer.slot, isNull);
      // a new selection drops the slot; `null` clears
      final cleared = reduce(ok, const FormulaNodeSelected(mappingId: dim, nodeId: null)).state;
      expect(cleared.editor.composer.selectedNode, isNull);
      expect(cleared.editor.composer.slot, isNull);
    });

    test('a composed answer is a draft change, then the next slot is selected', () {
      var s = connected(lamp());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt')).state;
      final t = reduce(
        s,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(
            nodeId: 'r',
            operator: pb.ComposeOperator(op: '/', before: false),
          ),
        ),
      );
      expect(t.state.editor.composer.pendingCompose, isTrue);
      final e = t.effects.single as ComposeFormula;
      expect(e.source, 'Tilt');
      expect(e.action.operator.op, '/');
      final r = reduce(
        t.state,
        ComposeReceived(
          generation: e.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            source: 'Tilt / ?',
            select: 'r.1',
          ),
        ),
      );
      // the draft moved through the ordinary path: a new generation, a check
      final d = r.state.draft(dim)!;
      expect(d.source, 'Tilt / ?');
      expect(d.generation, 2);
      expect(d.check, DraftCheck.checking);
      expect(r.effects.whereType<AnalyzeDraft>().single.source, 'Tilt / ?');
      expect(r.state.editor.composer.pendingCompose, isFalse);
      expect(r.state.editor.composer.selectedNode, 'r.1');
      expect(r.effects.whereType<GetFormulaSlot>().single.nodeId, 'r.1');
      // the verdict carries the projection for that generation
      final v = reduce(
        r.state,
        DraftAnalysisReceived(verdict(generation: 2, projection: tiltOverSlot())),
      ).state;
      expect(v.draft(dim)!.projection!.slots, ['r.1']);
    });

    test('a stale compose answer and a failure release the pending action', () {
      var s = connected(lamp());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt')).state;
      final t = reduce(
        s,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(nodeId: 'r', remove: pb.Unit()),
        ),
      );
      final e = t.effects.single as ComposeFormula;
      final other = reduce(
        t.state,
        ComposeReceived(
          generation: e.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(7),
            mappingId: Int64(dim),
            source: 'never',
          ),
        ),
      ).state;
      expect(other.draft(dim)!.source, 'Tilt', reason: 'another revision: dropped');
      expect(other.editor.composer.pendingCompose, isFalse);
      final failed = reduce(
        t.state,
        ToolingFailed(generation: e.generation, code: 'draft.unavailable', message: ''),
      ).state;
      expect(failed.editor.composer.pendingCompose, isFalse);
    });

    test('a committed definition is projected on request, once per mapping', () {
      final s = connected(lamp(definition: 'Tilt / (45 deg)'));
      final t = reduce(s, const FormulaProjectionRequested(dim));
      final e = t.effects.single as GetFormulaProjection;
      expect(e.mappingId, dim);
      final r = reduce(
        t.state,
        FormulaProjectionReceived(
          generation: e.generation,
          result: pb.FormulaProjectionResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            projection: tiltOver45(),
          ),
        ),
      ).state;
      expect(r.editor.composer.projection!.root.children[1].coordinate, '45');
      // with a draft, the draft's verdict carries it: no request
      final withDraft = reduce(
        r,
        const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / (50 deg)'),
      ).state;
      expect(reduce(withDraft, const FormulaProjectionRequested(dim)).effects, isEmpty);
    });
  });

  group('widget', () {
    testWidgets('Formula and Text are two projections of the one draft', (t) async {
      final h = await pump(t, connected(lamp()));
      // Formula mode by default: the empty slot names what it must produce
      expect(find.byKey(const ValueKey('composer-field')), findsOneWidget);
      expect(find.byKey(const ValueKey('definition-field')), findsNothing);
      expect(find.byKey(const ValueKey('node-r')), findsOneWidget);
      await t.tap(find.text('Text'));
      await t.pump();
      expect(h.state.editor.composer.formulaMode, isFalse);
      expect(find.byKey(const ValueKey('definition-field')), findsOneWidget);
      expect(find.text('expression over Tilt'), findsOneWidget);
      await t.tap(find.text('Formula'));
      await t.pump();
      expect(find.byKey(const ValueKey('composer-field')), findsOneWidget);
    });

    testWidgets('selecting a slot shows what it expects; the unit picker is the compiler\'s list', (
      t,
    ) async {
      var s = connected(lamp());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / ?')).state;
      s = reduce(
        s,
        DraftAnalysisReceived(verdict(generation: 1, projection: tiltOverSlot())),
      ).state;
      final h = await pump(t, s);
      // the tree: a reference, the operator, a dashed slot
      expect(find.byKey(const ValueKey('node-r.0')), findsOneWidget);
      expect(find.text('÷'), findsOneWidget);
      expect(find.byKey(const ValueKey('node-r.1')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('node-r.1')));
      await t.pump();
      expect(h.state.editor.composer.selectedNode, 'r.1');
      expect(find.text('Asking what fits here…'), findsOneWidget);
      final e = h.effects.whereType<GetFormulaSlot>().single;
      h.answer(FormulaSlotReceived(generation: e.generation, result: angleSlot('r.1')));
      await t.pump();
      expect(
        find.text('Expected: an angle, because an angle ÷ an angle = a dimensionless quantity.'),
        findsOneWidget,
      );
      // the number entry with the unit pop-up: deg, rad, turn — never mm
      expect(find.byKey(const ValueKey('slot-number')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('slot-unit')));
      await t.pumpAndSettle();
      expect(find.text('deg'), findsWidgets);
      expect(find.text('turn'), findsOneWidget);
      expect(find.text('mm'), findsNothing);
      await t.tap(find.text('deg').last);
      await t.pumpAndSettle();
      // the compiler's words only; exponent vectors behind Explain
      expect(find.text('expected q[angle]'), findsNothing);
      await t.tap(find.text('Explain'));
      await t.pump();
      expect(find.text('expected q[angle]'), findsOneWidget);
      // the reference and equation candidates are rows
      expect(find.byKey(const ValueKey('ref-Tilt')), findsOneWidget);
      expect(find.byKey(const ValueKey('eq-clamp')), findsOneWidget);
      // the objection on the component
      expect(find.text('This slot is empty; it expects an angle.'), findsOneWidget);
    });

    testWidgets('filling a slot with a number and a unit is one structured action', (t) async {
      var s = connected(lamp());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / ?')).state;
      s = reduce(
        s,
        DraftAnalysisReceived(verdict(generation: 1, projection: tiltOverSlot())),
      ).state;
      final h = await pump(t, s);
      await t.tap(find.byKey(const ValueKey('node-r.1')));
      await t.pump();
      final e = h.effects.whereType<GetFormulaSlot>().single;
      h.answer(FormulaSlotReceived(generation: e.generation, result: angleSlot('r.1')));
      await t.pump();
      await t.tap(find.byKey(const ValueKey('slot-unit')));
      await t.pumpAndSettle();
      await t.tap(find.text('deg').last);
      await t.pumpAndSettle();
      await t.enterText(
        find.descendant(
          of: find.byKey(const ValueKey('slot-number')),
          matching: find.byType(TextField),
        ),
        '90',
      );
      await t.tap(find.text('Insert'));
      await t.pump();
      final c = h.effects.whereType<ComposeFormula>().single;
      expect(c.source, 'Tilt / ?');
      expect((c.action.nodeId, c.action.fill), ('r.1', '90 deg'));
      // a reference fills without any unit chooser (18C)
      h.answer(
        ComposeReceived(
          generation: c.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            source: 'Tilt / 90 deg',
          ),
        ),
      );
      await t.pump();
      expect(h.state.draft(dim)!.source, 'Tilt / 90 deg');
      final again = await pump(
        t,
        reduce(
          reduce(
            connected(lamp()),
            const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / ?'),
          ).state,
          DraftAnalysisReceived(verdict(generation: 1, projection: tiltOverSlot())),
        ).state,
      );
      await t.tap(find.byKey(const ValueKey('node-r.1')));
      await t.pump();
      final e3 = again.effects.whereType<GetFormulaSlot>().single;
      again.answer(FormulaSlotReceived(generation: e3.generation, result: angleSlot('r.1')));
      await t.pump();
      await t.tap(find.byKey(const ValueKey('ref-Tilt')));
      await t.pump();
      final ref = again.effects.whereType<ComposeFormula>().single;
      expect(ref.action.fill, 'Tilt');
      expect(ref.action.hasSetUnit(), isFalse);
    });

    testWidgets(
      'a literal owns the unit picker and switching keeps the value; a reference has none',
      (t) async {
        var s = connected(lamp());
        s = reduce(
          s,
          const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / (45 deg)'),
        ).state;
        s = reduce(
          s,
          DraftAnalysisReceived(
            verdict(
              generation: 1,
              projection: tiltOver45(),
              status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
            ),
          ),
        ).state;
        final h = await pump(t, s);
        // the literal is a coordinate field and a unit pop-up; the reference is a chip
        expect(find.byKey(const ValueKey('coordinate-r.1')), findsOneWidget);
        expect(find.byKey(const ValueKey('unit-r.1')), findsOneWidget);
        expect(find.byKey(const ValueKey('unit-r.0')), findsNothing);
        // selecting the literal fetches its own units
        await t.tap(find.byKey(const ValueKey('coordinate-r.1')));
        await t.pump();
        final e = h.effects.whereType<GetFormulaSlot>().single;
        expect(e.nodeId, 'r.1');
        h.answer(FormulaSlotReceived(generation: e.generation, result: angleSlot('r.1')));
        await t.pump();
        await t.tap(find.byKey(const ValueKey('unit-r.1')));
        await t.pumpAndSettle();
        await t.tap(find.text('rad').last);
        await t.pumpAndSettle();
        final c = h.effects.whereType<ComposeFormula>().single;
        expect(c.action.setUnit.unitId, 'angle.rad');
        expect(c.action.setUnit.preserveValue, isTrue, reason: 'the quantity is kept');
        // typing a coordinate and Return is the other action
        await t.enterText(
          find.descendant(
            of: find.byKey(const ValueKey('coordinate-r.1')),
            matching: find.byType(TextField),
          ),
          '90',
        );
        await t.testTextInput.receiveAction(TextInputAction.done);
        await t.pump();
        expect(h.effects.whereType<ComposeFormula>().last.action.setCoordinate, '90');
      },
    );

    testWidgets('Composer → Text → Composer keeps one draft; invalid text falls back', (t) async {
      var s = connected(lamp());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / (45 deg)')).state;
      s = reduce(s, DraftAnalysisReceived(verdict(generation: 1, projection: tiltOver45()))).state;
      final h = await pump(t, s);
      expect(find.text('45'), findsOneWidget);
      // to Text: the same source, as text
      await t.tap(find.text('Text'));
      await t.pump();
      final field = find.descendant(
        of: find.byKey(const ValueKey('definition-field')),
        matching: find.byType(TextField),
      );
      expect(t.widget<TextField>(field).controller!.text, 'Tilt / (45 deg)');
      // edit the text, the compiler answers with the new projection
      await t.enterText(field, 'Tilt / (30 deg)');
      await t.pump();
      expect(h.state.draft(dim)!.source, 'Tilt / (30 deg)');
      final p = tiltOver45()
        ..source = 'Tilt / (30 deg)'
        ..root.text = 'Tilt / (30 deg)'
        ..root.children[1].coordinate = '30'
        ..root.children[1].text = '(30 deg)';
      h.answer(
        DraftAnalysisReceived(verdict(generation: h.state.draft(dim)!.generation, projection: p)),
      );
      await t.tap(find.text('Formula'));
      await t.pump();
      expect(find.text('30'), findsOneWidget);
      expect(find.byKey(const ValueKey('composer-out-of-sync')), findsNothing);
      // invalid text: exact text kept, the last readable form shown, out of sync
      await t.tap(find.text('Text'));
      await t.pump();
      await t.enterText(field, 'Tilt / (');
      await t.pump();
      h.answer(
        DraftAnalysisReceived(
          verdict(
            generation: h.state.draft(dim)!.generation,
            parseOk: false,
            projection: pb.FormulaProjection(source: 'Tilt / (', parseOk: false),
          ),
        ),
      );
      await t.tap(find.text('Formula'));
      await t.pump();
      expect(find.byKey(const ValueKey('composer-out-of-sync')), findsOneWidget);
      expect(h.state.draft(dim)!.source, 'Tilt / (', reason: 'the typed text is kept exactly');
      await t.tap(find.text('Edit as text'));
      await t.pump();
      expect(t.widget<TextField>(field).controller!.text, 'Tilt / (');
    });

    testWidgets('save, revert and a conflict work the same in Formula mode', (t) async {
      var s = connected(lamp(definition: 'Tilt / (90 deg)'));
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / (45 deg)')).state;
      s = reduce(
        s,
        DraftAnalysisReceived(
          verdict(
            generation: 1,
            projection: tiltOver45(),
            status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
          ),
        ),
      ).state;
      final h = await pump(t, s);
      expect(find.text('Save definition'), findsOneWidget);
      expect(find.text('Revert'), findsOneWidget);
      await t.tap(find.text('Save definition'));
      await t.pump();
      expect(
        h.effects.whereType<ApplyEdit>().single.op.replaceDefinition.definition.formula,
        'Tilt / (45 deg)',
      );
      // a conflict: the notice, never a silent overwrite
      h.state = h.state.copyWith(
        editor: h.state.editor.copyWith(
          drafts: {dim: h.state.draft(dim)!.copyWith(conflict: true, clearPendingCommit: true)},
        ),
      );
      h.dispatch(const FormulaModeChanged(true));
      await t.pump();
      expect(find.text('Reload'), findsOneWidget);
      expect(find.text('Keep mine'), findsOneWidget);
      await t.tap(find.text('Keep mine'));
      await t.pump();
      expect(h.state.draft(dim)!.conflict, isFalse);
      await t.tap(find.text('Revert'));
      await t.pump();
      expect(h.state.draft(dim), isNull);
      expect(h.effects.whereType<DiscardDraft>(), isNotEmpty);
    });
  });
}
