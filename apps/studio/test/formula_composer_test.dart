/// The Formula Composer: the reducer's transitions (mode, selection, slot
/// queries, structured actions, stale generations) and the widget over a
/// scripted projection (toggle, selecting, filling a slot, the unit picker
/// filtered by the compiler, a value-preserving unit switch, the two round
/// trips, the invalid-text fallback, save/revert/conflict unchanged).
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/composer.dart' show composerInSync;
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/definition_editor.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

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
      mappingView(
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

pb.TypeView boolean() => pb.TypeView(description: 'true or false', kind: 'boolean');

/// `all angle in [Tilt, Tilt]: angle in -45 deg .. ?` as the compiler
/// projects it: a binder over a collection literal (opaque here), a local
/// read by the body, a range with one end still empty.
pb.FormulaProjection binderOverRange() => pb.FormulaProjection(
  source: 'all angle in [Tilt, Tilt]: angle in -45 deg .. ?',
  parseOk: true,
  complete: false,
  slots: ['r.1.1.1'],
  root: pb.FormulaNode(
    id: 'r',
    kind: 'binder',
    name: 'all',
    param: 'angle',
    paramType: angle(),
    text: 'all angle in [Tilt, Tilt]: angle in -45 deg .. ?',
    range: pb.SourceSpan(start: 0, end: 48),
    actual: boolean(),
    children: [
      pb.FormulaNode(
        id: 'r.0',
        kind: 'opaque',
        name: 'a collection',
        text: '[Tilt, Tilt]',
        range: pb.SourceSpan(start: 13, end: 25),
        actual: pb.TypeView(
          description: 'a collection of angles',
          kind: 'structured',
          element: angle(),
        ),
        because: 'all reads every element of a collection',
      ),
      pb.FormulaNode(
        id: 'r.1',
        kind: 'compare',
        name: 'in',
        text: 'angle in -45 deg .. ?',
        range: pb.SourceSpan(start: 27, end: 48),
        expected: boolean(),
        because: 'all asks a question of every angle: the body is true or false',
        children: [
          pb.FormulaNode(
            id: 'r.1.0',
            kind: 'reference',
            name: 'angle',
            local: true,
            text: 'angle',
            range: pb.SourceSpan(start: 27, end: 32),
            actual: angle(),
          ),
          pb.FormulaNode(
            id: 'r.1.1',
            kind: 'range',
            text: '-45 deg .. ?',
            range: pb.SourceSpan(start: 36, end: 48),
            expected: angle(),
            because: 'both ends of the range must be comparable with angle (an angle)',
            children: [
              pb.FormulaNode(
                id: 'r.1.1.0',
                kind: 'unary',
                name: '-',
                text: '-45 deg',
                range: pb.SourceSpan(start: 36, end: 43),
                actual: angle(),
                expected: angle(),
                children: [
                  pb.FormulaNode(
                    id: 'r.1.1.0.0',
                    kind: 'quantity',
                    text: '45 deg',
                    coordinate: '45',
                    unit: 'deg',
                    unitId: 'angle.deg',
                    range: pb.SourceSpan(start: 37, end: 43),
                    actual: angle(),
                    expected: angle(),
                  ),
                ],
              ),
              pb.FormulaNode(
                id: 'r.1.1.1',
                kind: 'slot',
                text: '?',
                range: pb.SourceSpan(start: 47, end: 48),
                expected: angle(),
                because: 'both ends of the range must be comparable with angle (an angle)',
              ),
            ],
          ),
        ],
      ),
    ],
  ),
);

// ---- the air conditioner: boolean logic and a choice ---------------------------

const roomTemp = 10;
const buttonHeld = 11;
const switchState = 12;

/// `AirConditionerCtrl : RoomTemp -> ButtonHeld -> SwitchState`.
pb.ProjectProjection airConditioner({String? definition}) {
  final p = pb.ProjectProjection(revision: Int64(1), name: 'ac', rootPath: '/p')
    ..concepts.addAll([
      pb.ConceptView(
        id: Int64(roomTemp),
        name: 'RoomTemp',
        representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
      ),
      pb.ConceptView(
        id: Int64(buttonHeld),
        name: 'ButtonHeld',
        representation: pb.Representation(boolean: pb.Unit()),
      ),
      pb.ConceptView(
        id: Int64(switchState),
        name: 'SwitchState',
        representation: pb.Representation(boolean: pb.Unit()),
      ),
    ])
    ..mappings.add(
      mappingView(
        id: Int64(dim),
        name: 'AirConditionerCtrl',
        signature: pb.Signature(
          inputs: [Int64(roomTemp), Int64(buttonHeld)],
          output: Int64(switchState),
        ),
        definition: definition == null ? null : pb.Definition(formula: definition),
      ),
    );
  return p;
}

pb.TypeView truth() => pb.TypeView(description: 'true or false', kind: 'boolean');
pb.TypeView temperature() =>
    pb.TypeView(description: 'a temperature', kind: 'quantity', dim: pb.Dim(temperature: 1));

/// `RoomTemp > 299.15 K && ButtonHeld` — or, with `held` false, the same
/// with a slot in ButtonHeld's place — as the compiler projects it, rooted
/// at `id` from byte `at`.
pb.FormulaNode warmAndHeld({String id = 'r', int at = 0, bool held = true}) {
  final right = held ? 'ButtonHeld' : '?';
  final text = 'RoomTemp > 299.15 K && $right';
  return pb.FormulaNode(
    id: id,
    kind: 'compare',
    name: '&&',
    text: text,
    range: pb.SourceSpan(start: at, end: at + text.length),
    actual: truth(),
    children: [
      pb.FormulaNode(
        id: '$id.0',
        kind: 'compare',
        name: '>',
        text: 'RoomTemp > 299.15 K',
        range: pb.SourceSpan(start: at, end: at + 19),
        actual: truth(),
        expected: truth(),
        children: [
          pb.FormulaNode(
            id: '$id.0.0',
            kind: 'reference',
            name: 'RoomTemp',
            text: 'RoomTemp',
            range: pb.SourceSpan(start: at, end: at + 8),
            entity: pb.EntityRef(conceptId: Int64(roomTemp)),
            actual: pb.TypeView(
              description: 'a RoomTemp',
              kind: 'concept',
              dim: pb.Dim(temperature: 1),
              conceptId: Int64(roomTemp),
            ),
            expected: temperature(),
          ),
          pb.FormulaNode(
            id: '$id.0.1',
            kind: 'quantity',
            coordinate: '299.15',
            unit: 'K',
            unitId: 'temperature.K',
            text: '299.15 K',
            range: pb.SourceSpan(start: at + 11, end: at + 19),
            actual: temperature(),
            expected: temperature(),
          ),
        ],
      ),
      held
          ? pb.FormulaNode(
              id: '$id.1',
              kind: 'reference',
              name: 'ButtonHeld',
              text: 'ButtonHeld',
              range: pb.SourceSpan(start: at + 23, end: at + 33),
              entity: pb.EntityRef(conceptId: Int64(buttonHeld)),
              actual: pb.TypeView(
                description: 'a ButtonHeld',
                kind: 'concept',
                conceptId: Int64(buttonHeld),
              ),
              expected: truth(),
            )
          : pb.FormulaNode(
              id: '$id.1',
              kind: 'slot',
              text: '?',
              range: pb.SourceSpan(start: at + 23, end: at + 24),
              expected: truth(),
              because: 'both sides of a logical operator are true or false',
            ),
    ],
  );
}

/// `if RoomTemp > 299.15 K && ButtonHeld then true else false`.
pb.FormulaProjection warmAndHeldChoice() {
  const text = 'if RoomTemp > 299.15 K && ButtonHeld then true else false';
  final result = pb.TypeView(
    description: 'a SwitchState (true or false)',
    kind: 'concept',
    conceptId: Int64(switchState),
  );
  return pb.FormulaProjection(
    source: text,
    parseOk: true,
    complete: true,
    result: result,
    root: pb.FormulaNode(
      id: 'r',
      kind: 'if',
      text: text,
      range: pb.SourceSpan(start: 0, end: text.length),
      actual: truth(),
      expected: result,
      children: [
        warmAndHeld(id: 'r.0', at: 3)..expected = truth(),
        pb.FormulaNode(
          id: 'r.1',
          kind: 'bool',
          name: 'true',
          text: 'true',
          range: pb.SourceSpan(start: 42, end: 46),
          actual: truth(),
          expected: result,
        ),
        pb.FormulaNode(
          id: 'r.2',
          kind: 'bool',
          name: 'false',
          text: 'false',
          range: pb.SourceSpan(start: 52, end: 57),
          actual: truth(),
          expected: result,
        ),
      ],
    ),
  );
}

/// What a truth-valued slot offers: the two literals and ButtonHeld first.
pb.FormulaSlotResponse truthSlot(String node) => pb.FormulaSlotResponse(
  revision: Int64(1),
  mappingId: Int64(dim),
  nodeId: node,
  expected: truth(),
  explanation:
      'Expected: true or false, because both sides of a logical operator are true or false.',
  technical: 'expected true or false',
  booleans: ['true', 'false'],
  references: [
    pb.ReferenceCandidate(
      label: 'ButtonHeld',
      insert: 'ButtonHeld',
      entity: pb.EntityRef(conceptId: Int64(buttonHeld)),
      produces: 'ButtonHeld (true or false)',
      relevance: 90,
    ),
  ],
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

/// A draft that the compiler has answered: the text, and a projection of
/// exactly that text (the tree's shape does not matter to the reducer).
AppState drafted(AppState s, String source, {pb.FormulaProjection? projection}) {
  final t = reduce(s, DefinitionDraftChanged(mappingId: dim, source: source)).state;
  final p = projection ?? pb.FormulaProjection(source: source, parseOk: true, complete: true);
  return reduce(
    t,
    DraftAnalysisReceived(verdict(generation: t.draft(dim)!.generation, projection: p)),
  ).state;
}

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
  // the verdict carries every finding as text; the projection places the
  // same findings on nodes (one diagnostic, two projections)
  analysis: pb.MappingAnalysis(
    id: Int64(dim),
    status: status,
    diagnostics: projection.hasRoot() ? _findings(projection.root) : const [],
  ),
  projection: projection,
);

/// A node re-rooted under `id` (its children renumbered below it).
pb.FormulaNode _shift(pb.FormulaNode n, String id) => pb.FormulaNode()
  ..mergeFromMessage(n)
  ..id = id
  ..children.clear()
  ..children.addAll([for (var i = 0; i < n.children.length; i++) _shift(n.children[i], '$id.$i')]);

List<pb.Diagnostic> _findings(pb.FormulaNode n) => [
  ...n.diagnostics,
  for (final c in n.children) ..._findings(c),
];

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
      final s = drafted(connected(lamp()), 'Tilt');
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
      final s = drafted(connected(lamp()), 'Tilt');
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

  group('stale structure never edits current text', () {
    test('a compose answer for older text is discarded when the draft moved on', () {
      // generation N: `Tilt / ?`, an action on r.1 sent
      var s = drafted(connected(lamp()), 'Tilt / ?', projection: tiltOverSlot());
      s = reduce(s, const FormulaNodeSelected(mappingId: dim, nodeId: 'r.1')).state;
      final t = reduce(
        s,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(nodeId: 'r.1', fill: '90 deg'),
        ),
      );
      final e = t.effects.single as ComposeFormula;
      expect(e.source, 'Tilt / ?');
      // the designer types before the answer arrives (generation N+1):
      // the selection of the old tree is dropped with the old text
      var typed = reduce(
        t.state,
        const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / 45 deg'),
      ).state;
      expect(typed.editor.composer.selectedNode, isNull);
      // the old answer arrives: right generation, wrong text — discarded,
      // and the pending action is released
      typed = reduce(
        typed,
        ComposeReceived(
          generation: e.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            source: 'Tilt / 90 deg',
            select: 'r.1',
          ),
        ),
      ).state;
      expect(typed.draft(dim)!.source, 'Tilt / 45 deg', reason: 'the typed text stands');
      expect(typed.editor.composer.pendingCompose, isFalse);
      expect(typed.editor.composer.selectedNode, isNull, reason: 'no node of old text is selected');
    });

    test('a structured action on a stale or unreadable projection is refused, not sent', () {
      // projection of `Tilt / ?`, but the text has moved on: nothing is sent
      var s = drafted(connected(lamp()), 'Tilt / ?', projection: tiltOverSlot());
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / 4')).state;
      expect(s.draft(dim)!.projection, isNull, reason: 'a new generation drops the projection');
      var t = reduce(
        s,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(nodeId: 'r.1', fill: '1'),
        ),
      );
      expect(t.effects, isEmpty);
      expect(t.state.editor.composer.pendingCompose, isFalse);
      // unreadable text: a projection without a tree is not current either
      s = drafted(
        connected(lamp()),
        'Tilt / (',
        projection: pb.FormulaProjection(source: 'Tilt / (', parseOk: false),
      );
      t = reduce(
        s,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(nodeId: 'r', remove: pb.Unit()),
        ),
      );
      expect(t.effects, isEmpty);
      // two actions never fly at once: the second waits for the first
      s = drafted(connected(lamp()), 'Tilt');
      t = reduce(
        s,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(
            nodeId: 'r',
            operator: pb.ComposeOperator(op: '/'),
          ),
        ),
      );
      expect(t.effects, hasLength(1));
      final again = reduce(
        t.state,
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(
            nodeId: 'r',
            operator: pb.ComposeOperator(op: '*'),
          ),
        ),
      );
      expect(again.effects, isEmpty);
    });

    test('valid → invalid → valid: the exact text stands, the projection follows the verdict', () {
      var s = drafted(connected(lamp()), 'Tilt / (45 deg)', projection: tiltOver45());
      expect(composerInSync(s, dim), isTrue);
      // invalid: the typed text is the draft; the projection is gone until
      // the verdict says the text cannot be read
      s = reduce(s, const DefinitionDraftChanged(mappingId: dim, source: 'Tilt / (')).state;
      expect(s.draft(dim)!.source, 'Tilt / (');
      expect(composerInSync(s, dim), isFalse);
      s = reduce(
        s,
        DraftAnalysisReceived(
          verdict(
            generation: s.draft(dim)!.generation,
            parseOk: false,
            projection: pb.FormulaProjection(source: 'Tilt / (', parseOk: false),
          ),
        ),
      ).state;
      expect(composerInSync(s, dim), isFalse);
      // a late answer for the *valid* generation arrives during the
      // invalid one: dropped (its generation is old)
      final late = reduce(
        s,
        DraftAnalysisReceived(verdict(generation: 1, projection: tiltOver45())),
      ).state;
      expect(late.draft(dim)!.projection!.source, 'Tilt / (');
      // valid again: the verdict's projection is of the new text
      s = drafted(s, 'Tilt / (30 deg)');
      expect(composerInSync(s, dim), isTrue);
      expect(s.draft(dim)!.projection!.source, 'Tilt / (30 deg)');
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
      // the reference candidates are rows; the equations fold open
      expect(find.byKey(const ValueKey('ref-Tilt')), findsOneWidget);
      expect(find.byKey(const ValueKey('eq-clamp')), findsNothing);
      await t.tap(find.text('Equations (1)'));
      await t.pump();
      expect(find.byKey(const ValueKey('eq-clamp')), findsOneWidget);
      // the objection on the component: the diagnostic row under the field
      expect(find.text('This slot is empty; it expects an angle.'), findsWidgets);
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
        // the answer lands, the verdict projects the new text; only then is
        // the next structured action possible
        h.answer(
          ComposeReceived(
            generation: c.generation,
            result: pb.ComposeFormulaResponse(
              revision: Int64(1),
              mappingId: Int64(dim),
              source: 'Tilt / 0.7853981633974483 rad',
              select: 'r.1',
            ),
          ),
        );
        final p = tiltOver45()
          ..source = 'Tilt / 0.7853981633974483 rad'
          ..root.text = 'Tilt / 0.7853981633974483 rad'
          ..root.children[1].coordinate = '0.7853981633974483'
          ..root.children[1].unit = 'rad'
          ..root.children[1].unitId = 'angle.rad'
          ..root.children[1].text = '0.7853981633974483 rad'
          ..root.children[1].range = pb.SourceSpan(start: 7, end: 29);
        h.answer(
          DraftAnalysisReceived(
            verdict(
              generation: h.state.draft(dim)!.generation,
              projection: p,
              status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
            ),
          ),
        );
        await t.pump();
        expect(find.text('0.7853981633974483'), findsOneWidget);
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
      // Text → Formula while still invalid: no structured action is offered
      // (the chips are inert and no slot panel opens), the notice stays
      await t.tap(find.text('Formula'));
      await t.pump();
      expect(find.text('The text cannot be read as a formula.'), findsOneWidget);
      expect(find.byKey(const ValueKey('node-r')), findsNothing, reason: 'no tree is invented');
      expect(find.byKey(const ValueKey('node-r.1')), findsNothing);
      expect(h.effects.whereType<GetFormulaSlot>().where((e) => e.source == 'Tilt / ('), isEmpty);
      // recovery: the text is fixed; until the verdict of *that* text arrives
      // the Composer waits rather than showing the old tree as current
      await t.tap(find.text('Text'));
      await t.pump();
      await t.enterText(field, 'Tilt / (60 deg)');
      await t.pump();
      await t.tap(find.text('Formula'));
      await t.pump();
      expect(find.text('Waiting for the compiler to read the formula…'), findsOneWidget);
      // a stale answer (the invalid generation's) during recovery changes nothing
      h.answer(
        DraftAnalysisReceived(
          verdict(
            generation: h.state.draft(dim)!.generation - 1,
            parseOk: false,
            projection: pb.FormulaProjection(source: 'Tilt / (', parseOk: false),
          ),
        ),
      );
      await t.pump();
      expect(find.text('Waiting for the compiler to read the formula…'), findsOneWidget);
      final fixed = tiltOver45()
        ..source = 'Tilt / (60 deg)'
        ..root.text = 'Tilt / (60 deg)'
        ..root.children[1].coordinate = '60'
        ..root.children[1].text = '(60 deg)';
      h.answer(
        DraftAnalysisReceived(
          verdict(generation: h.state.draft(dim)!.generation, projection: fixed),
        ),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('composer-out-of-sync')), findsNothing);
      expect(find.text('60'), findsOneWidget);
    });

    testWidgets('a binder is a head over an indented body; its local is visibly the formula\'s', (
      t,
    ) async {
      var s = connected(lamp());
      s = drafted(s, binderOverRange().source, projection: binderOverRange());
      final h = await pump(t, s);
      // the head: the word, the local, `in`, the collection, the colon
      expect(find.byKey(const ValueKey('binder-r')), findsOneWidget);
      expect(find.text('all'), findsOneWidget);
      expect(find.byKey(const ValueKey('binder-local-r')), findsOneWidget);
      expect(find.text('in'), findsWidgets);
      expect(find.text('[Tilt, Tilt]'), findsOneWidget);
      // the local's declaration and its use are both italic; the design's
      // `Tilt` never appears as a chip here (it is inside the opaque region)
      final declared = t.widget<Text>(
        find.descendant(
          of: find.byKey(const ValueKey('binder-local-r')),
          matching: find.byType(Text),
        ),
      );
      expect(declared.style!.fontStyle, FontStyle.italic);
      final used = t.widget<Text>(
        find.descendant(of: find.byKey(const ValueKey('node-r.1.0')), matching: find.byType(Text)),
      );
      expect(used.style!.fontStyle, FontStyle.italic);
      expect(used.data, 'angle');
      // the range: lo, `..`, the slot
      expect(find.byKey(const ValueKey('node-r.1.1')), findsOneWidget);
      expect(find.text('..'), findsOneWidget);
      expect(find.byKey(const ValueKey('node-r.1.1.1')), findsOneWidget);
      // selecting the local selects the binder: the panel says what it is
      await t.tap(find.byKey(const ValueKey('binder-local-r')));
      await t.pump();
      expect(h.state.editor.composer.selectedNode, 'r');
      expect(find.byKey(const ValueKey('binder-local-note')), findsOneWidget);
      expect(find.text('angle is each element: an angle.'), findsOneWidget);
      // selecting the use selects the reference node, never a design entity
      await t.tap(find.byKey(const ValueKey('node-r.1.0')));
      await t.pump();
      expect(h.state.editor.composer.selectedNode, 'r.1.0');
      final e = h.effects.whereType<GetFormulaSlot>().last;
      expect(e.nodeId, 'r.1.0');
    });

    testWidgets('a range endpoint slot offers the subject\'s units; the literal end has a picker', (
      t,
    ) async {
      var s = connected(lamp());
      s = drafted(s, binderOverRange().source, projection: binderOverRange());
      final h = await pump(t, s);
      await t.tap(find.byKey(const ValueKey('node-r.1.1.1')));
      await t.pump();
      final e = h.effects.whereType<GetFormulaSlot>().single;
      expect(e.nodeId, 'r.1.1.1');
      h.answer(
        FormulaSlotReceived(
          generation: e.generation,
          result: angleSlot('r.1.1.1')..explanation = 'Expected: an angle, because both ends of the range must be comparable with angle (an angle).',
        ),
      );
      await t.pump();
      expect(find.textContaining('both ends of the range'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('slot-unit')));
      await t.pumpAndSettle();
      expect(find.text('turn'), findsOneWidget);
      expect(find.text('mm'), findsNothing);
      await t.tap(find.text('deg').last);
      await t.pumpAndSettle();
      await t.enterText(
        find.descendant(
          of: find.byKey(const ValueKey('slot-number')),
          matching: find.byType(TextField),
        ),
        '45',
      );
      await t.tap(find.text('Insert'));
      await t.pump();
      final c = h.effects.whereType<ComposeFormula>().single;
      expect((c.action.nodeId, c.action.fill), ('r.1.1.1', '45 deg'));
      // the literal end: selecting it shows its own unit picker
      h.answer(
        ComposeReceived(
          generation: c.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            source: 'all angle in [Tilt, Tilt]: angle in -45 deg .. 45 deg',
          ),
        ),
      );
      await t.pump();
      expect(h.state.draft(dim)!.source, 'all angle in [Tilt, Tilt]: angle in -45 deg .. 45 deg');
    });

    testWidgets('wrapping in a binder and inserting a range are structured actions', (t) async {
      var s = connected(lamp());
      s = drafted(s, 'Tilt / (45 deg)', projection: tiltOver45());
      final h = await pump(t, s);
      // a quantity: the range action; no binder action (it is no collection)
      await t.tap(find.byKey(const ValueKey('coordinate-r.1')));
      await t.pump();
      final e = h.effects.whereType<GetFormulaSlot>().single;
      h.answer(FormulaSlotReceived(generation: e.generation, result: angleSlot('r.1')));
      await t.pump();
      expect(find.byKey(const ValueKey('op-range')), findsOneWidget);
      expect(find.byKey(const ValueKey('op-binder')), findsNothing);
      await t.tap(find.byKey(const ValueKey('op-range')));
      await t.pump();
      final c = h.effects.whereType<ComposeFormula>().single;
      expect(c.action.nodeId, 'r.1');
      expect(c.action.hasRange(), isTrue);
      // a collection: the binder action, sent as the form chosen
      h.answer(
        ComposeReceived(
          generation: c.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            source: 'Tilt / (45 deg in ? .. ?)',
          ),
        ),
      );
      await t.pump();
      final coll = binderOverRange();
      final again = await pump(
        t,
        drafted(
          connected(lamp()),
          coll.root.children[0].text,
          projection: pb.FormulaProjection(
            source: coll.root.children[0].text,
            parseOk: true,
            complete: true,
            root: coll.root.children[0]..id = 'r',
          ),
        ),
      );
      await t.tap(find.byKey(const ValueKey('node-r')));
      await t.pump();
      final e2 = again.effects.whereType<GetFormulaSlot>().single;
      again.answer(
        FormulaSlotReceived(
          generation: e2.generation,
          result: pb.FormulaSlotResponse(revision: Int64(1), mappingId: Int64(dim), nodeId: 'r'),
        ),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('op-binder')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('op-binder')));
      await t.pumpAndSettle();
      await t.tap(find.text('any … satisfies').last);
      await t.pumpAndSettle();
      final c2 = again.effects.whereType<ComposeFormula>().single;
      expect((c2.action.nodeId, c2.action.binder.form), ('r', 'any'));
    });

    testWidgets('nested binders render nested; an unreadable binder falls back to text', (t) async {
      final inner = binderOverRange();
      final nested = pb.FormulaProjection(
        source: 'any row in rows: ${inner.source}',
        parseOk: true,
        complete: false,
        root: pb.FormulaNode(
          id: 'r',
          kind: 'binder',
          name: 'any',
          param: 'row',
          text: 'any row in rows: ${inner.source}',
          range: pb.SourceSpan(start: 0, end: 17 + 48),
          children: [
            pb.FormulaNode(
              id: 'r.0',
              kind: 'reference',
              name: 'rows',
              text: 'rows',
              range: pb.SourceSpan(start: 11, end: 15),
            ),
            _shift(inner.root, 'r.1'),
          ],
        ),
      );
      var s = connected(lamp());
      s = drafted(s, nested.source, projection: nested);
      final h = await pump(t, s);
      expect(find.byKey(const ValueKey('binder-r')), findsOneWidget);
      expect(find.byKey(const ValueKey('binder-r.1')), findsOneWidget);
      expect(find.byKey(const ValueKey('binder-local-r')), findsOneWidget);
      expect(find.byKey(const ValueKey('binder-local-r.1')), findsOneWidget);
      // the inner binder's body is inside the outer's body
      expect(
        find.descendant(
          of: find.byKey(const ValueKey('binder-r')),
          matching: find.byKey(const ValueKey('node-r.1.1.1.1')),
        ),
        findsOneWidget,
      );
      // typing an unfinished binder: the text cannot be read, so the
      // structure is dimmed and nothing is editable structurally
      h.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'all angle in'));
      await t.pump();
      h.answer(
        DraftAnalysisReceived(
          verdict(
            generation: h.state.draft(dim)!.generation,
            projection: pb.FormulaProjection(source: 'all angle in', parseOk: false),
            parseOk: false,
          ),
        ),
      );
      await t.pump();
      expect(composerInSync(h.state, dim), isFalse);
      expect(find.text('The text cannot be read as a formula.'), findsOneWidget);
      expect(find.byKey(const ValueKey('binder-r')), findsNothing);
      h.dispatch(
        ComposeRequested(
          mappingId: dim,
          action: pb.ComposeAction(nodeId: 'r', range: pb.Unit()),
        ),
      );
      expect(h.effects.whereType<ComposeFormula>(), isEmpty);
    });

    testWidgets('a truth value offers and, or, not and Choose; a quantity offers none of them', (
      t,
    ) async {
      const src = 'RoomTemp > 299.15 K && ButtonHeld';
      var s = connected(airConditioner());
      s = drafted(
        s,
        src,
        projection: pb.FormulaProjection(
          source: src,
          parseOk: true,
          complete: true,
          root: warmAndHeld(),
        ),
      );
      final h = await pump(t, s);
      // the operator is its word, in the language's weight; the
      // comparison its symbol
      expect(find.text('and'), findsOneWidget);
      expect(find.text('&&'), findsNothing);
      expect(find.text('>'), findsOneWidget);
      // ButtonHeld: a truth value
      await t.tap(find.byKey(const ValueKey('node-r.1')));
      await t.pump();
      var e = h.effects.whereType<GetFormulaSlot>().single;
      h.answer(FormulaSlotReceived(generation: e.generation, result: truthSlot('r.1')));
      await t.pump();
      for (final k in ['op-&&', 'op-||', 'op-not', 'op-choose', 'op-+', 'op-remove']) {
        expect(find.byKey(ValueKey(k)), findsOneWidget, reason: k);
      }
      expect(find.byKey(const ValueKey('op-range')), findsNothing);
      await t.tap(find.byKey(const ValueKey('op-||')));
      await t.pump();
      var c = h.effects.whereType<ComposeFormula>().single;
      expect(
        (c.action.nodeId, c.action.operator.op, c.action.operator.before),
        ('r.1', '||', false),
      );
      h.answer(
        ComposeReceived(
          generation: c.generation,
          result: pb.ComposeFormulaResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            source: '$src || ?',
            select: 'r.1',
          ),
        ),
      );
      // `not` wraps in place; `Choose` makes the component an outcome
      final again = await pump(
        t,
        drafted(
          connected(airConditioner()),
          src,
          projection: pb.FormulaProjection(
            source: src,
            parseOk: true,
            complete: true,
            root: warmAndHeld(),
          ),
        ),
      );
      await t.tap(find.byKey(const ValueKey('node-r')));
      await t.pump();
      e = again.effects.whereType<GetFormulaSlot>().single;
      again.answer(FormulaSlotReceived(generation: e.generation, result: truthSlot('r')));
      await t.pump();
      await t.tap(find.byKey(const ValueKey('op-not')));
      await t.pump();
      c = again.effects.whereType<ComposeFormula>().single;
      expect((c.action.nodeId, c.action.operator.op), ('r', '!'));
      // RoomTemp: a temperature — no logical operator, a range
      final third = await pump(
        t,
        drafted(
          connected(airConditioner()),
          src,
          projection: pb.FormulaProjection(
            source: src,
            parseOk: true,
            complete: true,
            root: warmAndHeld(),
          ),
        ),
      );
      await t.tap(find.byKey(const ValueKey('node-r.0.0')));
      await t.pump();
      e = third.effects.whereType<GetFormulaSlot>().single;
      third.answer(
        FormulaSlotReceived(
          generation: e.generation,
          result: pb.FormulaSlotResponse(
            revision: Int64(1),
            mappingId: Int64(dim),
            nodeId: 'r.0.0',
            expected: temperature(),
          ),
        ),
      );
      await t.pump();
      for (final k in ['op-&&', 'op-||', 'op-not']) {
        expect(find.byKey(ValueKey(k)), findsNothing, reason: k);
      }
      expect(find.byKey(const ValueKey('op-range')), findsOneWidget);
      expect(find.byKey(const ValueKey('op-choose')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('op-choose')));
      await t.pump();
      c = third.effects.whereType<ComposeFormula>().single;
      expect(c.action.nodeId, 'r.0.0');
      expect(c.action.hasChoose(), isTrue);
    });

    testWidgets('the keys on a selected component: comparisons, ==, and, or, not', (t) async {
      const src = 'RoomTemp > 299.15 K && ButtonHeld';
      // the character typed, the (shifted) key it comes from, the
      // operator it inserts
      const keys = [
        ('<', LogicalKeyboardKey.comma, PhysicalKeyboardKey.comma, '<'),
        ('>', LogicalKeyboardKey.period, PhysicalKeyboardKey.period, '>'),
        ('=', LogicalKeyboardKey.equal, PhysicalKeyboardKey.equal, '=='),
        ('&', LogicalKeyboardKey.digit7, PhysicalKeyboardKey.digit7, '&&'),
        ('|', LogicalKeyboardKey.backslash, PhysicalKeyboardKey.backslash, '||'),
        ('!', LogicalKeyboardKey.digit1, PhysicalKeyboardKey.digit1, '!'),
        ('+', LogicalKeyboardKey.equal, PhysicalKeyboardKey.equal, '+'),
      ];
      for (final (ch, logical, key, op) in keys) {
        final h = await pump(
          t,
          drafted(
            connected(airConditioner()),
            src,
            projection: pb.FormulaProjection(
              source: src,
              parseOk: true,
              complete: true,
              root: warmAndHeld(),
            ),
          ),
        );
        // a click selects and takes the keys
        await t.tap(find.byKey(const ValueKey('node-r.1')));
        await t.pump();
        await simulateKeyDownEvent(
          LogicalKeyboardKey.shift,
          physicalKey: PhysicalKeyboardKey.shiftLeft,
        );
        await simulateKeyDownEvent(logical, physicalKey: key, character: ch);
        await simulateKeyUpEvent(logical, physicalKey: key);
        await simulateKeyUpEvent(
          LogicalKeyboardKey.shift,
          physicalKey: PhysicalKeyboardKey.shiftLeft,
        );
        await t.pump();
        final c = h.effects.whereType<ComposeFormula>().single;
        expect((c.action.nodeId, c.action.operator.op), ('r.1', op), reason: ch);
      }
    });

    testWidgets('a truth-valued slot offers true and false, the references, Choose and not', (
      t,
    ) async {
      const src = 'RoomTemp > 299.15 K && ?';
      final h = await pump(
        t,
        drafted(
          connected(airConditioner()),
          src,
          projection: pb.FormulaProjection(
            source: src,
            parseOk: true,
            complete: false,
            slots: ['r.1'],
            root: warmAndHeld(held: false),
          ),
        ),
      );
      await t.tap(find.byKey(const ValueKey('node-r.1')));
      await t.pump();
      final e = h.effects.whereType<GetFormulaSlot>().single;
      h.answer(FormulaSlotReceived(generation: e.generation, result: truthSlot('r.1')));
      await t.pump();
      expect(
        find.text(
          'Expected: true or false, because both sides of a logical operator are true or false.',
        ),
        findsOneWidget,
      );
      // no number to type for a truth value; the two literals instead
      expect(find.byKey(const ValueKey('slot-number')), findsNothing);
      expect(find.byKey(const ValueKey('bool-true')), findsOneWidget);
      expect(find.byKey(const ValueKey('bool-false')), findsOneWidget);
      expect(find.byKey(const ValueKey('ref-ButtonHeld')), findsOneWidget);
      expect(find.byKey(const ValueKey('op-choose')), findsOneWidget);
      expect(find.byKey(const ValueKey('op-not')), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('bool-true')));
      await t.pump();
      final c = h.effects.whereType<ComposeFormula>().single;
      expect((c.action.nodeId, c.action.fill), ('r.1', 'true'));
    });

    testWidgets('a choice is if and its condition over then and else, each part selectable', (
      t,
    ) async {
      final p = warmAndHeldChoice();
      final h = await pump(t, drafted(connected(airConditioner()), p.source, projection: p));
      // no opaque text: the three words and every part of the condition
      expect(find.byKey(const ValueKey('if-r')), findsOneWidget);
      expect(find.text('if'), findsOneWidget);
      expect(find.text('then'), findsOneWidget);
      expect(find.text('else'), findsOneWidget);
      expect(find.text(p.source), findsNothing);
      expect(find.byKey(const ValueKey('node-r.0.0.0')), findsOneWidget);
      expect(find.byKey(const ValueKey('node-r.1')), findsOneWidget);
      expect(find.byKey(const ValueKey('node-r.2')), findsOneWidget);
      expect(find.text('true'), findsOneWidget);
      expect(find.text('false'), findsOneWidget);
      // the word selects the choice; an outcome selects itself
      await t.tap(find.byKey(const ValueKey('node-r')));
      await t.pump();
      expect(h.state.editor.composer.selectedNode, 'r');
      await t.tap(find.byKey(const ValueKey('node-r.2')));
      await t.pump();
      expect(h.state.editor.composer.selectedNode, 'r.2');
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
