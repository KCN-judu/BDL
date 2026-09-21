/// The Simulate page: feed the design's inputs, choose which timing domains
/// activate when, step the reference evaluator on bdld, read the values.
///
/// Task: "does this design behave as I mean, tick by tick?".  Ranked
/// facts: (1) the values per tick — the trace; (2) what stopped a tick —
/// the evaluator's failure, in product words; (3) what the inputs are now;
/// (4) which domains are active at a tick.  Input controls are generated
/// from each input's value form (quantity, on/off, count) as the
/// projection states it, never from a name.  Nothing here computes a
/// value: every number in the table came from bdld.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../../l10n/l10n.dart';
import '../../app/actions.dart';
import '../../app/simulation.dart';
import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../canvas/canvas_geometry.dart' show dimLabel, representationWords;
import '../canvas/concept_glyphs.dart';
import '../mac/controls.dart';
import '../mac/interactive.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';
import '../semantic_actions.dart';

class SimulatePage extends StatelessWidget {
  const SimulatePage({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.flat;
    if (p == null) {
      return Container(
        color: t.canvas,
        alignment: Alignment.center,
        child: Text(context.l10n.noProjectOpen, style: TextStyle(color: t.textTertiary)),
      );
    }
    final sim = state.editor.simulation;
    return Container(
      color: t.canvas,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          SizedBox(
            width: 300,
            child: Container(
              color: t.sidebar,
              child: ListView(
                padding: const EdgeInsets.symmetric(vertical: 4),
                children: [
                  _InputsSection(
                    project: p,
                    sim: sim,
                    selection: state.editor.selection,
                    dispatch: dispatch,
                  ),
                  _DomainsSection(project: p, sim: sim, dispatch: dispatch),
                ],
              ),
            ),
          ),
          VerticalDivider(width: 1, color: t.hairline),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Keyed for the documentation screenshots (docs/user-guide/screenshots).
                _Controls(
                  key: const ValueKey('simulation-controls'),
                  state: state,
                  dispatch: dispatch,
                ),
                _Blockers(
                  key: const ValueKey('simulation-readiness'),
                  state: state,
                  dispatch: dispatch,
                ),
                Expanded(
                  child: _Trace(state: state, dispatch: dispatch),
                ),
              ],
            ),
          ),
          VerticalDivider(width: 1, color: t.hairline),
          SizedBox(
            width: MacMetrics.inspectorWidth,
            child: _Probe(state: state, dispatch: dispatch),
          ),
        ],
      ),
    );
  }
}

/// What keeps the design from stepping, as sentences about named objects
/// with a link to each — the readiness state.  Step is disabled while any
/// is listed; nothing is sent.  A tick that *failed* is the controls' line.
///
/// Below the blockers, the notes: facts the compiler states about the
/// design that do not stop a step but explain what the trace will not
/// show — a rule nothing applies.  A filled dot stops Step; a hollow one
/// does not.  Each note carries its Fix and a link to the object.
class _Blockers extends StatelessWidget {
  const _Blockers({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final blockers = simulationBlockers(state);
    final notes = simulationNotes(state);
    if (blockers.isEmpty && notes.isEmpty) return const SizedBox.shrink();
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    String nameOf(int mappingId) => state.mapping(mappingId)?.name ?? '?';
    return Container(
      padding: const EdgeInsets.fromLTRB(MacMetrics.gapGroup, 10, MacMetrics.gapGroup, 10),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: t.hairline)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gapTight,
        children: [
          for (final b in blockers)
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: MacMetrics.gap,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 5),
                  child: Icon(Icons.circle, size: 7, color: b.checking ? t.textTertiary : t.open),
                ),
                Expanded(
                  child: Wrap(
                    crossAxisAlignment: WrapCrossAlignment.center,
                    spacing: MacMetrics.gapTight,
                    children: [
                      Text(
                        blockerSentence(context.l10n, b),
                        style: const TextStyle(fontSize: MacType.body),
                      ),
                      if (b.conceptId != null || b.mappingId != null)
                        MacLink(
                          label: context.l10n.show,
                          onTap: () => dispatch(
                            SelectionChanged(
                              b.conceptId != null
                                  ? ConceptSelected(b.conceptId!)
                                  : MappingSelected(b.mappingId!),
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          for (final n in notes)
            Row(
              key: ValueKey('readiness-note-${n.mappingId}'),
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: MacMetrics.gap,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 5),
                  child: Icon(Icons.circle_outlined, size: 7, color: t.open),
                ),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    spacing: MacMetrics.gapTight,
                    children: [
                      Text(n.message, style: const TextStyle(fontSize: MacType.body)),
                      if (n.explanation.isNotEmpty) Text(n.explanation, style: small),
                      Wrap(
                        crossAxisAlignment: WrapCrossAlignment.center,
                        spacing: MacMetrics.gap,
                        children: [
                          OfferedFix(
                            state: state,
                            selection: MappingSelected(n.mappingId),
                            actionKind: n.actionKind,
                            title: context.l10n.addAValueThatApplies(nameOf(n.mappingId)),
                            dispatch: dispatch,
                          ),
                          MacLink(
                            label: context.l10n.show,
                            onTap: () => dispatch(SelectionChanged(MappingSelected(n.mappingId))),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
        ],
      ),
    );
  }
}

/// One control per unresolved mapping without inputs, shaped by the value
/// form of what it produces.
class _InputsSection extends StatelessWidget {
  const _InputsSection({
    required this.project,
    required this.sim,
    required this.selection,
    required this.dispatch,
  });
  final pb.ProjectProjection project;
  final SimulationState sim;
  final Selection selection;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final inputs = simulationInputs(project);
    final functions = [
      for (final m in project.mappings)
        if (relationshipRole(m) == RelationshipRole.rule && !m.hasDefinition()) m,
    ];
    return InspectorSection(
      // The simulation's inputs are exactly the Sources (FV Phase 12
      // `SimulationInput = Source ∧ UnitDomain`): the values the
      // environment provides, one per activation.
      title: context.l10n.sources,
      children: [
        if (inputs.isEmpty) Text(context.l10n.noSourcesARelationshipWithNoReads, style: small),
        for (final m in inputs)
          _InputControl(
            mapping: m,
            concept: project.concepts.firstWhere((c) => c.id == m.signature.output),
            value: sim.current[m.id.toInt()],
            selection: selection,
            dispatch: dispatch,
          ),
        if (functions.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: Text(
              context.l10n.relationshipsWithInputsNeedADefinition(
                functions.map((m) => m.name).join(', '),
              ),
              style: small,
            ),
          ),
      ],
    );
  }
}

class _InputControl extends StatelessWidget {
  const _InputControl({
    required this.mapping,
    required this.concept,
    required this.value,
    required this.selection,
    required this.dispatch,
  });
  final pb.MappingView mapping;
  final pb.ConceptView concept;
  final pb.Value? value;
  final Selection selection;
  final void Function(AppAction) dispatch;

  pb.Value _semantic(pb.Value repr) => pb.Value(
    semantic: pb.SemanticValue(conceptId: concept.id, repr: repr),
  );

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final repr = value != null && value!.hasSemantic() ? value!.semantic.repr : null;
    final id = mapping.id.toInt();
    final Widget control;
    if (!concept.hasRepresentation()) {
      control = Text(context.l10n.hasNoValueFormYet(concept.name), style: small);
    } else {
      switch (concept.representation.whichKind()) {
        case pb.Representation_Kind.quantity:
          final dim = concept.representation.quantity;
          final unit = dimLabel(dim);
          control = Row(
            spacing: MacMetrics.gap,
            children: [
              Expanded(
                child: CommitTextField(
                  key: ValueKey('input-$id'),
                  value: repr != null && repr.hasQuantity() ? _fmt(repr.quantity.value) : '',
                  hint: context.l10n.noValueYet,
                  onCommit: (v) {
                    final n = double.tryParse(v.trim());
                    if (n == null) return;
                    dispatch(
                      SimulationInputChanged(
                        mappingId: id,
                        value: _semantic(
                          pb.Value(
                            quantity: pb.Quantity(dim: dim, value: n),
                          ),
                        ),
                      ),
                    );
                  },
                ),
              ),
              SizedBox(width: 40, child: Text(unit, style: small)),
            ],
          );
        case pb.Representation_Kind.boolean:
          // Three states, not two: off, on, and no value yet — the third
          // is a dashed, empty control with the words beside it, so a
          // Source nobody has decided never looks switched off.  One click
          // on a segment gives exactly that value; nothing is defaulted
          // (the runtime treats a missing input as an error by design).
          final bool? on = repr != null && repr.hasBoolean() ? repr.boolean : null;
          control = Row(
            spacing: MacMetrics.gap,
            children: [
              SizedBox(
                width: 96,
                child: MacSegmented<bool?>(
                  key: ValueKey('input-$id'),
                  value: on,
                  undecided: on == null,
                  options: {false: context.l10n.offWord, true: context.l10n.onWord},
                  onChanged: (v) => dispatch(
                    SimulationInputChanged(
                      mappingId: id,
                      value: _semantic(pb.Value(boolean: v ?? false)),
                    ),
                  ),
                ),
              ),
              if (on == null) Text(context.l10n.noValueYet, style: small),
            ],
          );
        case pb.Representation_Kind.count:
          control = CommitTextField(
            key: ValueKey('input-$id'),
            value: repr != null && repr.hasCount() ? repr.count.toString() : '',
            hint: context.l10n.noValueYet,
            onCommit: (v) {
              final n = int.tryParse(v.trim());
              if (n == null || n < 0) return;
              dispatch(
                SimulationInputChanged(
                  mappingId: id,
                  value: _semantic(pb.Value(count: Int64(n))),
                ),
              );
            },
          );
        case pb.Representation_Kind.list:
        case pb.Representation_Kind.pair:
        case pb.Representation_Kind.optional:
          // A structured value, written the way the design shows it:
          // `[1, 2, 3]`, `(20, 45)`, `none` / `some(1)`, nested as the
          // form nests.  Only the value's shape is read here; what it
          // means stays with the compiler.
          final r = concept.representation;
          control = CommitTextField(
            key: ValueKey('input-$id'),
            value: repr == null ? '' : renderValue(repr),
            hint: representationWords(r, unit: dimLabel),
            onCommit: (v) {
              final parsed = parseValue(r, v);
              if (parsed == null) return;
              dispatch(SimulationInputChanged(mappingId: id, value: _semantic(parsed)));
            },
          );
        case pb.Representation_Kind.notSet:
          control = Text(context.l10n.hasNoValueFormYet(concept.name), style: small);
      }
    }
    return Padding(
      padding: const EdgeInsets.only(bottom: MacMetrics.gap),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gapTight,
        children: [
          // The same glyph and hue the concept has on the canvas; the row
          // selects the input so the probe shows it.
          MacInteractive(
            onTap: () => dispatch(SelectionChanged(MappingSelected(id))),
            selected: switch (selection) {
              MappingSelected(id: final sel) => sel == id,
              _ => false,
            },
            padding: const EdgeInsets.symmetric(horizontal: 4),
            child: Row(
              spacing: MacMetrics.gap,
              children: [
                SocketGlyph.of(concept, t, size: 11),
                Flexible(
                  child: Text(
                    mapping.name,
                    style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Flexible(
                  child: Text(concept.name, style: small, overflow: TextOverflow.ellipsis),
                ),
              ],
            ),
          ),
          control,
        ],
      ),
    );
  }
}

/// Up to six significant digits, trailing zeros trimmed, integers plain.
String _fmt(double v) {
  if (v == v.roundToDouble() && v.abs() < 1e15) return v.toInt().toString();
  final s = v.toStringAsPrecision(6);
  if (s.contains('e') || !s.contains('.')) return s;
  return s.replaceFirst(RegExp(r'0+$'), '').replaceFirst(RegExp(r'\.$'), '');
}

/// Which domains activate when: a period per domain.  The schedule is
/// outside the design; a domain name never implies a rate.
class _DomainsSection extends StatelessWidget {
  const _DomainsSection({required this.project, required this.sim, required this.dispatch});
  final pb.ProjectProjection project;
  final SimulationState sim;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    return InspectorSection(
      title: context.l10n.timingDomains,
      children: [
        if (project.clocks.isEmpty)
          Text(context.l10n.noDomainsEveryRelationshipIsEvaluatedAt, style: small)
        else
          Text(context.l10n.activationPeriodInTicksChangingOneStarts, style: small),
        for (final c in project.clocks)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: Row(
              spacing: MacMetrics.gap,
              children: [
                Expanded(
                  child: Text('↻ ${c.name}', style: TextStyle(fontSize: MacType.body)),
                ),
                Text('every', style: small),
                SizedBox(
                  width: 48,
                  child: CommitTextField(
                    key: ValueKey('period-${c.id}'),
                    value: '${sim.periods[c.id.toInt()] ?? 1}',
                    onCommit: (v) {
                      final n = int.tryParse(v.trim());
                      if (n == null) return;
                      dispatch(SimulationPeriodChanged(clockId: c.id.toInt(), period: n));
                    },
                  ),
                ),
                Text('ticks', style: small),
              ],
            ),
          ),
      ],
    );
  }
}

/// Step, step ten, reset; the tick counter; and one line for what the
/// evaluator says — the failure in product words, or the design's own
/// reasons it cannot run.
class _Controls extends StatelessWidget {
  const _Controls({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final sim = state.editor.simulation;
    final blocked = simulationBlockers(state).isNotEmpty;
    final (String line, Color color) = sim.failure != null
        ? (sim.failure!, t.error)
        : sim.error != null
        ? (_errorSentence(context.l10n, state, sim.error!), t.error)
        : sim.pending
        ? (context.l10n.evaluating, t.textTertiary)
        : sim.hasRun
        ? (context.l10n.ticksEvaluated(sim.nextTick), t.textSecondary)
        : (blocked ? '' : context.l10n.setTheInputsThenStep, t.textSecondary);
    // Refused before any request while something blocks (listed below).
    final canStep = !sim.pending && !blocked && state.connection is Connected;
    return Container(
      padding: const EdgeInsets.fromLTRB(MacMetrics.gapGroup, 12, MacMetrics.gapGroup, 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: t.hairline)),
      ),
      child: Row(
        spacing: MacMetrics.gap,
        children: [
          MacButton.primary(
            label: context.l10n.step,
            onPressed: canStep ? () => dispatch(const SimulationStepRequested(1)) : null,
          ),
          MacButton(
            label: context.l10n.stepTen,
            onPressed: canStep ? () => dispatch(const SimulationStepRequested(10)) : null,
          ),
          MacButton(
            label: context.l10n.reset,
            onPressed: sim.hasRun ? () => dispatch(const SimulationResetRequested()) : null,
          ),
          const SizedBox(width: MacMetrics.gap),
          Text(
            context.l10n.tickN(sim.nextTick),
            style: TextStyle(
              fontSize: MacType.secondary,
              color: t.textSecondary,
              fontFeatures: kTabularFigures,
            ),
          ),
          const SizedBox(width: MacMetrics.gap),
          Expanded(
            child: Text(
              line,
              key: const ValueKey('simulation-line'),
              style: TextStyle(fontSize: MacType.body, color: color),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}

/// A readiness blocker worded for the designer: the kind says what is
/// missing, the names say of what.  Never a compiler word.
String blockerSentence(AppLocalizations l10n, SimulationBlocker b) {
  String name(int i) => b.names.elementAtOrNull(i) ?? '?';
  return switch (b.kind) {
    SimulationBlockerKind.checking => l10n.checkingTheDesign,
    SimulationBlockerKind.instantaneousCycle => l10n.dependOnEachOtherInTheSameInstant(
      b.names.join(', '),
    ),
    SimulationBlockerKind.instantaneousCycleUnnamed => l10n.theDesignContainsAnInstantaneousCycle,
    SimulationBlockerKind.noValidDefinition => l10n.hasNoValidDefinition(name(0)),
    SimulationBlockerKind.noDefinition => l10n.hasNoDefinitionReadsSomething(name(0)),
    SimulationBlockerKind.noValueForm => l10n.needsAValueFormBeforeInput(name(0), name(1)),
    SimulationBlockerKind.needsValue => l10n.needsAValueBeforeSimulationCanStep(name(0)),
  };
}

/// The evaluator's structured failure, worded for the designer.  The
/// code says what happened, the entity who; the tick is the row.
String _errorSentence(AppLocalizations l10n, AppState s, pb.Diagnostic d) {
  final who = d.hasMappingId()
      ? (s.mapping(d.mappingId.toInt())?.name ?? '?')
      : l10n.aRelationshipCapital;
  return switch (d.code) {
    'simulation.missing_input' => l10n.needsAValueForThisStep(who),
    'simulation.division_by_zero' => l10n.dividedByZero(who),
    'simulation.non_finite' => l10n.producedAValueThatIsNotANumber(who),
    'simulation.not_causal' => l10n.theDesignContainsAnInstantaneousCycle,
    _ => d.message,
  };
}

/// The trace: one row per tick, one column per relationship without
/// inputs (a relationship with inputs is a function, not a value per
/// tick) and per driven output.  Values are bdld's own rendering.
class _Trace extends StatelessWidget {
  const _Trace({required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.flat!;
    final sim = state.editor.simulation;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    // One column per Source and per value (the role the daemon states):
    // a rule's value at a tick is a function, never a sample.
    final columns = [
      for (final m in p.mappings)
        if (relationshipRole(m) != RelationshipRole.rule) m,
    ];
    final outputs = [
      for (final o in p.outputs)
        if (state.outputAnalysis(o.id.toInt()) case final a? when a.hasDriver())
          (o, a.driver.toInt()),
    ];
    String clockName(Int64 id) =>
        p.clocks.where((c) => c.id == id).map((c) => c.name).firstOrNull ?? '?';
    if (sim.samples.isEmpty) {
      return Center(
        child: Text(context.l10n.noTicksEvaluatedYet, style: TextStyle(color: t.textTertiary)),
      );
    }
    final mono = TextStyle(fontFamily: 'Menlo', fontSize: MacType.code, color: t.textPrimary);
    final head = TextStyle(
      fontSize: MacType.secondary,
      fontWeight: FontWeight.w600,
      color: t.textSecondary,
    );
    return SingleChildScrollView(
      padding: const EdgeInsets.all(MacMetrics.gapGroup),
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Table(
          defaultColumnWidth: const IntrinsicColumnWidth(),
          defaultVerticalAlignment: TableCellVerticalAlignment.middle,
          children: [
            TableRow(
              children: [
                _cell(Text(context.l10n.tickColumn, style: head)),
                _cell(Text(context.l10n.activeColumn, style: head)),
                for (final m in columns)
                  _cell(
                    MacInteractive(
                      onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
                      selected: switch (state.editor.selection) {
                        MappingSelected(:final id) => id == m.id.toInt(),
                        _ => false,
                      },
                      padding: const EdgeInsets.symmetric(horizontal: 4),
                      child: Text(m.name, style: head),
                    ),
                  ),
                for (final (o, _) in outputs) _cell(Text('→ ${o.name}', style: head)),
              ],
            ),
            for (final s in sim.samples)
              TableRow(
                key: ValueKey('tick-${s.tick}'),
                children: [
                  _cell(Text('${s.tick}', style: mono)),
                  _cell(Text(s.activeClockIds.map(clockName).join(' '), style: small)),
                  for (final m in columns) _cell(Text(_valueOf(s, m.id), style: mono)),
                  for (final (_, driver) in outputs)
                    _cell(Text(_valueOf(s, Int64(driver)), style: mono)),
                ],
              ),
          ],
        ),
      ),
    );
  }

  /// The evaluator's rendering, or `·` where the declaration was not due
  /// at the tick (an input is echoed only at ticks its domain activated).
  static String _valueOf(pb.TickSample s, Int64 mapping) =>
      sampleOf(s, mapping.toInt())?.rendered ?? '·';

  static Widget _cell(Widget child) =>
      Padding(padding: const EdgeInsets.fromLTRB(0, 3, MacMetrics.gutter, 3), child: child);
}

// ---------------------------------------------------------------------------
// Right: the probe of the selection
// ---------------------------------------------------------------------------

/// What the selected object is worth now and over the run, in the
/// evaluator's own rendering — the same text as the trace.  The selection
/// is the one shared with Design; the object keeps its name and glyph.
/// Explain holds the formal detail: DeclId, the run's revision, an error's
/// code and technical text.
class _Probe extends StatelessWidget {
  const _Probe({required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.flat!;
    final sim = state.editor.simulation;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    final value = TextStyle(
      fontSize: MacType.body,
      color: t.textPrimary,
      fontFeatures: kTabularFigures,
    );
    Widget body;
    switch (state.editor.selection) {
      case NoSelection() ||
          ComponentSelected() ||
          InstanceSelected() ||
          PortSelected() ||
          BindingSelected() ||
          LinkSelected() ||
          GroupSelected() ||
          MultiSelected():
        body = Padding(
          padding: const EdgeInsets.all(12),
          child: Text(
            context.l10n.selectAnInputAColumnOrA,
            style: TextStyle(color: t.textTertiary),
          ),
        );
      case ConceptSelected(:final id):
        final c = p.concepts.firstWhere((c) => c.id.toInt() == id);
        // *Produces* is the signature (any relationship whose output is
        // this concept — the canvas's input socket); a value of the concept
        // per tick exists only where a value or a Source produces it: the
        // concept is *carried by* those (ADR-0034).  A rule producing it
        // gives it no value until a value's formula applies the rule.
        final carriers = [
          for (final m in p.mappings)
            if (relationshipRole(m) != RelationshipRole.rule && m.signature.output.toInt() == id) m,
        ];
        final rules = [
          for (final m in p.mappings)
            if (relationshipRole(m) == RelationshipRole.rule && m.signature.output.toInt() == id) m,
        ];
        body = InspectorSection(
          title: c.name,
          trailing: SocketGlyph.of(c, t),
          children: [
            if (carriers.isEmpty) ...[
              Text(context.l10n.noValueCarries(c.name), style: small),
              for (final r in rules)
                Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(context.l10n.ruleProducesNoValue(r.name, c.name), style: small),
                ),
            ] else ...[
              Text(context.l10n.carriedBy, style: small),
              for (final m in carriers)
                FormRow(
                  label: m.name,
                  child: Text(_latest(sim, m.id.toInt()) ?? '—', style: value),
                ),
            ],
          ],
        );
      case MappingSelected(:final id):
        final m = p.mappings.firstWhere((m) => m.id.toInt() == id);
        final c = p.concepts.where((c) => c.id == m.signature.output).firstOrNull;
        // A Source or a value is sampled at every tick; a rule is not.
        final isValue = relationshipRole(m) != RelationshipRole.rule;
        // A rule is applied by the values whose formulas reference it
        // (the analysis's refs, the canvas's reference edges) — those are
        // what the simulator samples.
        final appliers = appliersOf(state, id);
        body = Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            InspectorSection(
              title: m.name,
              trailing: c == null ? null : SocketGlyph.of(c, t),
              children: [
                if (!isValue) ...[
                  Text(context.l10n.aRuleNoValueOfItsOwn, style: small),
                  // Who applies it, as links; when nobody does, the fix
                  // that would (the compiler's note on the rule).
                  if (appliers.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: FormRow(
                        label: context.l10n.appliedBy,
                        child: Wrap(
                          spacing: MacMetrics.gapTight,
                          children: [
                            for (final a in appliers)
                              MacLink(
                                label: a.name,
                                onTap: () =>
                                    dispatch(SelectionChanged(MappingSelected(a.id.toInt()))),
                              ),
                          ],
                        ),
                      ),
                    )
                  else ...[
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(context.l10n.noValueAppliesItYet, style: small),
                    ),
                    if (isUnappliedRule(state, id))
                      Padding(
                        padding: const EdgeInsets.only(top: MacMetrics.gapTight),
                        child: OfferedFix(
                          state: state,
                          selection: MappingSelected(id),
                          actionKind: kApplyRuleActionKind,
                          title: context.l10n.addAValueThatApplies(m.name),
                          dispatch: dispatch,
                        ),
                      ),
                  ],
                ] else
                  FormRow(
                    label: context.l10n.now,
                    child: Text(
                      _latest(sim, id) ?? (sim.hasRun ? '—' : context.l10n.notSteppedYet),
                      style: value,
                    ),
                  ),
              ],
            ),
            if (isValue && sim.hasRun) _Series(sim: sim, mappingId: id),
            MacDisclosure(
              title: context.l10n.explain,
              children: [
                ExplainLine('DeclId ${m.id}'),
                if (sim.revision case final r?) ExplainLine('run at revision $r'),
                if (sim.error case final e?) ...[
                  ExplainLine(e.code),
                  if (e.technical.isNotEmpty) ExplainLine(e.technical),
                ],
              ],
            ),
          ],
        );
      case OutputSelected(:final id):
        final o = p.outputs.firstWhere((o) => o.id.toInt() == id);
        final oa = state.analysis?.outputs.where((o) => o.id.toInt() == id).firstOrNull;
        final driver = oa?.hasDriver() == true ? oa!.driver.toInt() : null;
        final driverName = driver == null
            ? null
            : p.mappings.where((m) => m.id.toInt() == driver).firstOrNull?.name;
        body = Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            InspectorSection(
              title: o.name,
              children: [
                FormRow(
                  label: context.l10n.finalTarget,
                  child: driverName == null
                      ? Text(
                          context.l10n.noneYet,
                          style: TextStyle(fontSize: MacType.body, color: t.textTertiary),
                        )
                      : MacLink(
                          label: driverName,
                          onTap: () => dispatch(SelectionChanged(MappingSelected(driver!))),
                        ),
                ),
                FormRow(
                  label: context.l10n.now,
                  child: Text(driver == null ? '—' : (_latest(sim, driver) ?? '—'), style: value),
                ),
              ],
            ),
            if (driver != null && sim.hasRun) _Series(sim: sim, mappingId: driver),
          ],
        );
    }
    return Container(
      color: t.sidebar,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          PanelHeader(context.l10n.probe),
          Expanded(child: SingleChildScrollView(child: body)),
        ],
      ),
    );
  }
}

/// The latest value of a declaration in the run, in the evaluator's
/// rendering — the same text the trace shows.
String? _latest(SimulationState sim, int mappingId) {
  for (final s in sim.samples.reversed) {
    if (sampleOf(s, mappingId) case final v?) return v.rendered;
  }
  return null;
}

/// One declaration over the run: tick, value.
class _Series extends StatelessWidget {
  const _Series({required this.sim, required this.mappingId});
  final SimulationState sim;
  final int mappingId;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final cell = TextStyle(
      fontSize: MacType.body,
      fontFeatures: kTabularFigures,
      color: t.textPrimary,
    );
    final rows = [
      for (final s in sim.samples)
        if (sampleOf(s, mappingId) case final v?)
          [Text('${s.tick}', style: cell), Text(v.rendered, style: cell)],
    ];
    return InspectorSection(
      title: context.l10n.overTheRun,
      children: [
        if (rows.isEmpty)
          Text(
            context.l10n.notEvaluatedAtAnyTickYet,
            style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
          )
        else
          MacTable(
            columns: const [MacColumn(width: 44, numeric: true), MacColumn(numeric: true)],
            rows: rows,
          ),
      ],
    );
  }
}

/// A structured input value as text, the inverse of [parseValue].
String renderValue(pb.Value v) => switch (v.whichKind()) {
  pb.Value_Kind.boolean => v.boolean ? 'true' : 'false',
  pb.Value_Kind.count => v.count.toString(),
  pb.Value_Kind.quantity => _fmt(v.quantity.value),
  pb.Value_Kind.semantic => v.semantic.hasRepr() ? renderValue(v.semantic.repr) : '',
  pb.Value_Kind.none => 'none',
  pb.Value_Kind.some => 'some(${renderValue(v.some)})',
  pb.Value_Kind.list => '[${v.list.items.map(renderValue).join(', ')}]',
  pb.Value_Kind.pair => '(${renderValue(v.pair.fst)}, ${renderValue(v.pair.snd)})',
  pb.Value_Kind.opaque => v.opaque,
  pb.Value_Kind.notSet => '',
};

/// Read a value of form [r] from text: `true`, `3`, `0.5`, `[a, b]`,
/// `(a, b)`, `none`, `some(a)`.  `null` when the text does not have the
/// form's shape.
pb.Value? parseValue(pb.Representation r, String text) {
  final t = text.trim();
  switch (r.whichKind()) {
    case pb.Representation_Kind.quantity:
      final n = double.tryParse(t);
      return n == null
          ? null
          : pb.Value(
              quantity: pb.Quantity(dim: r.quantity, value: n),
            );
    case pb.Representation_Kind.boolean:
      return switch (t) {
        'true' => pb.Value(boolean: true),
        'false' => pb.Value(boolean: false),
        _ => null,
      };
    case pb.Representation_Kind.count:
      final n = int.tryParse(t);
      return n == null || n < 0 ? null : pb.Value(count: Int64(n));
    case pb.Representation_Kind.optional:
      if (t == 'none') return pb.Value(none: pb.Unit());
      if (t.startsWith('some(') && t.endsWith(')')) {
        final inner = parseValue(r.optional, t.substring(5, t.length - 1));
        return inner == null ? null : pb.Value(some: inner);
      }
      return null;
    case pb.Representation_Kind.list:
      if (!t.startsWith('[') || !t.endsWith(']')) return null;
      final items = <pb.Value>[];
      for (final part in _splitTopLevel(t.substring(1, t.length - 1))) {
        final v = parseValue(r.list, part);
        if (v == null) return null;
        items.add(v);
      }
      return pb.Value(list: pb.ValueList(items: items));
    case pb.Representation_Kind.pair:
      if (!t.startsWith('(') || !t.endsWith(')')) return null;
      final parts = _splitTopLevel(t.substring(1, t.length - 1));
      if (parts.length != 2) return null;
      final a = parseValue(r.pair.first, parts[0]);
      final b = parseValue(r.pair.second, parts[1]);
      return a == null || b == null
          ? null
          : pb.Value(
              pair: pb.ValuePair(fst: a, snd: b),
            );
    case pb.Representation_Kind.notSet:
      return null;
  }
}

/// Split on commas outside brackets and parentheses; nothing for an empty body.
List<String> _splitTopLevel(String body) {
  final out = <String>[];
  var depth = 0;
  var start = 0;
  for (var i = 0; i < body.length; i++) {
    final ch = body[i];
    if (ch == '[' || ch == '(') depth++;
    if (ch == ']' || ch == ')') depth = depth > 0 ? depth - 1 : 0;
    if (ch == ',' && depth == 0) {
      out.add(body.substring(start, i));
      start = i + 1;
    }
  }
  if (body.substring(start).trim().isNotEmpty) out.add(body.substring(start));
  return out;
}
