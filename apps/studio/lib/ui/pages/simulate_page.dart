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

import '../../app/actions.dart';
import '../../app/simulation.dart';
import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../canvas/canvas_geometry.dart' show dimLabel;
import '../mac/controls.dart';
import '../mac/tokens.dart';
import '../mac/widgets.dart';

class SimulatePage extends StatelessWidget {
  const SimulatePage({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project;
    if (p == null) {
      return Container(
        color: t.canvas,
        alignment: Alignment.center,
        child: Text('No project open.', style: TextStyle(color: t.textTertiary)),
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
                  _InputsSection(project: p, sim: sim, dispatch: dispatch),
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
                _Controls(state: state, dispatch: dispatch),
                Expanded(child: _Trace(state: state)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// One control per unresolved mapping without inputs, shaped by the value
/// form of what it produces.
class _InputsSection extends StatelessWidget {
  const _InputsSection({required this.project, required this.sim, required this.dispatch});
  final pb.ProjectProjection project;
  final SimulationState sim;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final inputs = simulationInputs(project);
    final functions = [
      for (final m in project.mappings)
        if (!m.hasDefinition() && m.signature.inputs.isNotEmpty) m,
    ];
    return InspectorSection(
      title: 'Inputs',
      children: [
        if (inputs.isEmpty)
          Text(
            'No inputs: a relationship without inputs and without a definition is one. '
            'Values then come from outside the design, one per tick.',
            style: small,
          ),
        for (final m in inputs)
          _InputControl(
            mapping: m,
            concept: project.concepts.firstWhere((c) => c.id == m.signature.output),
            value: sim.current[m.id.toInt()],
            dispatch: dispatch,
          ),
        if (functions.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: Text(
              '${functions.map((m) => m.name).join(', ')}: a relationship with inputs needs a '
              'definition before it can run.',
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
    required this.dispatch,
  });
  final pb.MappingView mapping;
  final pb.ConceptView concept;
  final pb.Value? value;
  final void Function(AppAction) dispatch;

  pb.Value _semantic(pb.Value repr) => pb.Value(
    semantic: pb.SemanticValue(conceptId: concept.id, repr: repr),
  );

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final repr = value != null && value!.hasSemantic() ? value!.semantic.repr : null;
    final id = mapping.id.toInt();
    final Widget control;
    if (!concept.hasRepresentation()) {
      control = Text('${concept.name} has no value form yet.', style: small);
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
                  hint: 'value',
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
          final on = repr != null && repr.hasBoolean() && repr.boolean;
          control = Row(
            spacing: MacMetrics.gap,
            children: [
              Switch(
                key: ValueKey('input-$id'),
                value: on,
                onChanged: (v) => dispatch(
                  SimulationInputChanged(
                    mappingId: id,
                    value: _semantic(pb.Value(boolean: v)),
                  ),
                ),
              ),
              Text(on ? 'on' : 'off', style: small),
            ],
          );
        case pb.Representation_Kind.count:
          control = CommitTextField(
            key: ValueKey('input-$id'),
            value: repr != null && repr.hasCount() ? repr.count.toString() : '',
            hint: 'whole number',
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
        case pb.Representation_Kind.notSet:
          control = Text('${concept.name} has no value form yet.', style: small);
      }
    }
    return Padding(
      padding: const EdgeInsets.only(bottom: MacMetrics.gap),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gapTight,
        children: [
          Row(
            spacing: MacMetrics.gap,
            children: [
              Text(mapping.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
              Text(concept.name, style: small),
            ],
          ),
          control,
        ],
      ),
    );
  }
}

String _fmt(double v) => v == v.roundToDouble() ? v.toInt().toString() : v.toString();

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
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    return InspectorSection(
      title: 'Timing domains',
      children: [
        if (project.clocks.isEmpty)
          Text('No domains: every relationship is evaluated at every tick.', style: small)
        else
          Text('Activation period, in ticks. Changing one starts the run over.', style: small),
        for (final c in project.clocks)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: Row(
              spacing: MacMetrics.gap,
              children: [
                Expanded(child: Text('↻ ${c.name}', style: TextStyle(fontSize: 12))),
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
  const _Controls({required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final sim = state.editor.simulation;
    final a = state.analysis;
    final blockers = <String>[
      if (a != null && !a.causal) 'The design contains an instantaneous cycle.',
      if (a != null)
        for (final m in a.mappings)
          if (m.status == pb.MappingStatus.MAPPING_STATUS_INVALID)
            '${state.mapping(m.id.toInt())?.name ?? '?'} cannot be simulated because its '
                'definition is invalid.',
    ];
    final (String line, Color color) = sim.failure != null
        ? (sim.failure!, t.error)
        : sim.error != null
        ? (_errorSentence(state, sim.error!), t.error)
        : blockers.isNotEmpty
        ? (blockers.first, t.error)
        : sim.pending
        ? ('Evaluating…', t.textTertiary)
        : sim.hasRun
        ? ('${sim.nextTick} tick${sim.nextTick == 1 ? '' : 's'} evaluated.', t.textSecondary)
        : ('Set the inputs, then step.', t.textSecondary);
    final canStep = !sim.pending && state.connection is Connected;
    return Container(
      padding: const EdgeInsets.fromLTRB(MacMetrics.gapGroup, 12, MacMetrics.gapGroup, 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: t.hairline)),
      ),
      child: Row(
        spacing: MacMetrics.gap,
        children: [
          MacButton.primary(
            label: 'Step',
            onPressed: canStep ? () => dispatch(const SimulationStepRequested(1)) : null,
          ),
          MacButton(
            label: 'Step ×10',
            onPressed: canStep ? () => dispatch(const SimulationStepRequested(10)) : null,
          ),
          MacButton(
            label: 'Reset',
            onPressed: sim.hasRun ? () => dispatch(const SimulationResetRequested()) : null,
          ),
          const SizedBox(width: MacMetrics.gap),
          Text(
            'tick ${sim.nextTick}',
            style: TextStyle(fontSize: 12, color: t.textSecondary, fontFeatures: kTabularFigures),
          ),
          const SizedBox(width: MacMetrics.gap),
          Expanded(
            child: Text(
              line,
              key: const ValueKey('simulation-line'),
              style: TextStyle(fontSize: 12, color: color),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}

/// The evaluator's structured failure, worded for the designer.  The
/// code says what happened, the entity who; the tick is the row.
String _errorSentence(AppState s, pb.Diagnostic d) {
  final who = d.hasMappingId() ? (s.mapping(d.mappingId.toInt())?.name ?? '?') : 'A relationship';
  return switch (d.code) {
    'simulation.missing_input' => '$who needs a value for this step.',
    'simulation.division_by_zero' => '$who divided by zero.',
    'simulation.non_finite' => '$who produced a value that is not a number.',
    'simulation.not_causal' => 'The design contains an instantaneous cycle.',
    _ => d.message,
  };
}

/// The trace: one row per tick, one column per relationship without
/// inputs (a relationship with inputs is a function, not a value per
/// tick) and per driven output.  Values are bdld's own rendering.
class _Trace extends StatelessWidget {
  const _Trace({required this.state});
  final AppState state;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project!;
    final sim = state.editor.simulation;
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final columns = [
      for (final m in p.mappings)
        if (m.signature.inputs.isEmpty) m,
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
        child: Text('No ticks evaluated yet.', style: TextStyle(color: t.textTertiary)),
      );
    }
    final mono = TextStyle(fontFamily: 'Menlo', fontSize: 12, color: t.textPrimary);
    final head = TextStyle(fontSize: 11, fontWeight: FontWeight.w600, color: t.textSecondary);
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
                _cell(Text('tick', style: head)),
                _cell(Text('active', style: head)),
                for (final m in columns) _cell(Text(m.name, style: head)),
                for (final (o, _) in outputs) _cell(Text('→ ${o.name}', style: head)),
              ],
            ),
            for (final s in sim.samples)
              TableRow(
                key: ValueKey('tick-${s.tick}'),
                children: [
                  _cell(Text('${s.tick}', style: mono)),
                  _cell(Text(s.activeClockIds.map(clockName).join(' '), style: small)),
                  for (final m in columns)
                    _cell(
                      Text(
                        m.hasDefinition()
                            ? _valueOf(s, m.id)
                            : _fedValue(sim, m.id.toInt(), s.tick.toInt()),
                        style: mono,
                      ),
                    ),
                  for (final (_, driver) in outputs)
                    _cell(Text(_valueOf(s, Int64(driver)), style: mono)),
                ],
              ),
          ],
        ),
      ),
    );
  }

  /// An input's value at a tick is Studio's own trace (the evaluator
  /// records computed declarations only): rendered from what was fed.
  static String _fedValue(SimulationState sim, int mapping, int tick) {
    final v = sim.inputs[mapping]?[tick];
    if (v == null) return '·';
    final repr = v.hasSemantic() ? v.semantic.repr : v;
    return switch (repr.whichKind()) {
      pb.Value_Kind.quantity => _fmt(repr.quantity.value),
      pb.Value_Kind.boolean => repr.boolean ? 'on' : 'off',
      pb.Value_Kind.count => repr.count.toString(),
      _ => '·',
    };
  }

  static String _valueOf(pb.TickSample s, Int64 mapping) =>
      s.values.where((v) => v.mappingId == mapping).map((v) => v.rendered).firstOrNull ?? '·';

  static Widget _cell(Widget child) =>
      Padding(padding: const EdgeInsets.fromLTRB(0, 3, MacMetrics.gutter, 3), child: child);
}
