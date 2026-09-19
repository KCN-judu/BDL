/// Simulation transitions: Studio authors the input trace and the schedule;
/// bdld's reference evaluator produces every value.
///
/// The protocol fixes a run's inputs at `StartSimulation`, so stepping
/// interactively re-creates the run from tick 0 with the trace so far and
/// steps to the wanted tick — cheap, and exactly as deterministic as the
/// evaluator itself.  A run belongs to one revision: a commit drops the
/// samples, never the authored inputs.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition;
import 'state.dart';

/// Unresolved relationships with the unit domain `()` — values the design
/// reads and does not define: the declarations the evaluator needs a value
/// for at every tick.  (Read off the projection.)
List<pb.MappingView> simulationInputs(pb.ProjectProjection p) => [
  for (final m in p.mappings)
    if (!m.hasDefinition() && m.signature.isUnitDomain) m,
];

Transition simulationInputChanged(AppState s, int mappingId, pb.Value value) {
  final sim = s.editor.simulation;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        simulation: sim.copyWith(current: {...sim.current, mappingId: value}),
      ),
    ),
  );
}

Transition simulationPeriodChanged(AppState s, int clockId, int period) {
  final sim = s.editor.simulation;
  final periods = {...sim.periods, clockId: period < 1 ? 1 : period};
  // The schedule shapes every tick: the run starts over.
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(simulation: sim.copyWith(periods: periods, clearRun: true)),
    ),
  );
}

// ---------------------------------------------------------------------------
// Readiness — read off the analysis and the projection, never judged here
// ---------------------------------------------------------------------------

/// What keeps the design from stepping — the fact, not its wording.  The
/// Simulate page words each kind in the designer's language and locale
/// (`blockerSentence`); the names are the objects' own.
enum SimulationBlockerKind {
  /// The compiler has not answered for this revision yet; nothing is wrong.
  checking,

  /// Relationships that depend on each other in the same instant ([names]).
  instantaneousCycle,

  /// The design has an instantaneous cycle the analysis did not name.
  instantaneousCycleUnnamed,

  /// [names]: the relationship, whose definition does not check.
  noValidDefinition,

  /// [names]: a relationship that reads something and has no definition.
  noDefinition,

  /// [names]: the concept, then the input whose value form it lacks.
  noValueForm,

  /// [names]: the input still without a value for the run.
  needsValue,
}

/// Why the design cannot step yet: about named objects, saying what would
/// let it.  Never a compiler word.
@immutable
class SimulationBlocker {
  const SimulationBlocker(this.kind, {this.names = const [], this.mappingId, this.conceptId});
  final SimulationBlockerKind kind;

  /// The objects the sentence names, in the order the kind says.
  final List<String> names;
  final int? mappingId;
  final int? conceptId;

  bool get checking => kind == SimulationBlockerKind.checking;
}

/// Facts that keep the design from running at all (an instantaneous
/// cycle, a definition that does not check, a relationship with inputs
/// and no definition, an input whose concept has no value form), then the
/// inputs still without a value.  While any is listed a Step sends nothing.
List<SimulationBlocker> simulationBlockers(AppState s) {
  final p = s.flat;
  if (p == null) return const [];
  final a = s.analysis;
  if (a == null || a.revision != p.revision) {
    return const [SimulationBlocker(SimulationBlockerKind.checking)];
  }
  String name(Int64 id) =>
      p.mappings.where((m) => m.id == id).map((m) => m.name).firstOrNull ?? '?';
  final out = <SimulationBlocker>[];
  if (!a.causal) {
    for (final c in a.cycles) {
      out.add(
        SimulationBlocker(
          SimulationBlockerKind.instantaneousCycle,
          names: c.mappingIds.map(name).toList(),
          mappingId: c.mappingIds.firstOrNull?.toInt(),
        ),
      );
    }
    if (a.cycles.isEmpty) {
      out.add(const SimulationBlocker(SimulationBlockerKind.instantaneousCycleUnnamed));
    }
  }
  for (final m in p.mappings) {
    final status = a.mappings.where((x) => x.id == m.id).firstOrNull?.status;
    if (status == pb.MappingStatus.MAPPING_STATUS_INVALID) {
      out.add(
        SimulationBlocker(
          SimulationBlockerKind.noValidDefinition,
          names: [m.name],
          mappingId: m.id.toInt(),
        ),
      );
    } else if (!m.hasDefinition() && !m.signature.isUnitDomain) {
      out.add(
        SimulationBlocker(
          SimulationBlockerKind.noDefinition,
          names: [m.name],
          mappingId: m.id.toInt(),
        ),
      );
    }
  }
  for (final m in simulationInputs(p)) {
    final c = p.concepts.where((c) => c.id == m.signature.output).firstOrNull;
    if (c == null) continue;
    if (!c.hasRepresentation()) {
      out.add(
        SimulationBlocker(
          SimulationBlockerKind.noValueForm,
          names: [c.name, m.name],
          conceptId: c.id.toInt(),
          mappingId: m.id.toInt(),
        ),
      );
    } else if (!s.editor.simulation.current.containsKey(m.id.toInt())) {
      out.add(
        SimulationBlocker(
          SimulationBlockerKind.needsValue,
          names: [m.name],
          mappingId: m.id.toInt(),
        ),
      );
    }
  }
  return out;
}

/// The evaluator's sample of a declaration at a tick, or `null` where the
/// declaration was not due (its domain did not activate).  Every value on
/// screen is the sample's own rendering — a fed input is echoed by the
/// evaluator like any other declaration, so Studio never renders a value
/// (ADR-0033).
pb.DeclarationSample? sampleOf(pb.TickSample t, int mappingId) =>
    t.values.where((v) => v.mappingId.toInt() == mappingId).firstOrNull;

/// Refused — with no request — while anything blocks; the blockers are on
/// screen, each naming its object.
Transition simulationStepRequested(AppState s, int ticks) {
  final p = s.flat;
  if (p == null || ticks < 1 || simulationBlockers(s).isNotEmpty) return Transition(s);
  final sim = s.editor.simulation;
  final upto = sim.nextTick + ticks;
  // Extend the authored trace: every input takes its current value at the
  // ticks about to be evaluated; earlier ticks keep what they were fed.
  final inputs = {
    for (final e in sim.inputs.entries) e.key: {...e.value},
  };
  for (final m in simulationInputs(p)) {
    final id = m.id.toInt();
    final row = inputs.putIfAbsent(id, () => {});
    final value = sim.current[id];
    if (value == null) continue;
    for (var t = sim.nextTick; t < upto; t++) {
      row[t] = value;
    }
  }
  final generation = sim.generation + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        simulation: sim.copyWith(
          inputs: inputs,
          pending: true,
          generation: generation,
          clearFailure: true,
        ),
      ),
    ),
    [
      RunSimulation(
        inputs: [
          for (final e in inputs.entries)
            for (final t in e.value.entries)
              if (t.key < upto)
                pb.SimulationInput(mappingId: Int64(e.key), tick: Int64(t.key), value: t.value),
        ],
        schedule: [
          for (final e in sim.periods.entries)
            pb.SchedulePeriod(clockId: Int64(e.key), period: Int64(e.value)),
          // domains not mentioned activate every tick
          for (final c in p.clocks)
            if (!sim.periods.containsKey(c.id.toInt()))
              pb.SchedulePeriod(clockId: c.id, period: Int64(1)),
        ],
        ticks: upto,
        generation: generation,
      ),
    ],
  );
}

Transition simulationResetRequested(AppState s) {
  final sim = s.editor.simulation;
  return Transition(
    s.copyWith(editor: s.editor.copyWith(simulation: sim.copyWith(clearRun: true))),
  );
}

Transition simulationReceived(AppState s, int generation, pb.SimulationResponse r) {
  final sim = s.editor.simulation;
  if (generation != sim.generation) return Transition(s);
  // A run is about a revision; only the one on screen counts.
  if (r.revision.toInt() != s.revision) {
    return Transition(
      s.copyWith(editor: s.editor.copyWith(simulation: sim.copyWith(pending: false))),
    );
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        simulation: sim.copyWith(
          revision: r.revision.toInt(),
          nextTick: r.nextTick.toInt(),
          samples: r.samples,
          pending: false,
          error: r.hasError() ? r.error : null,
          clearError: !r.hasError(),
        ),
      ),
    ),
  );
}

Transition simulationFailed(AppState s, int generation, String message) {
  final sim = s.editor.simulation;
  if (generation != sim.generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(simulation: sim.copyWith(pending: false, failure: message)),
    ),
  );
}

/// The design changed: the samples are about an old design and go; the
/// authored inputs stay for the mappings that still exist.
SimulationState simulationAfterRevision(SimulationState sim, pb.ProjectProjection p) {
  final ids = {for (final m in simulationInputs(p)) m.id.toInt()};
  return sim.copyWith(
    clearRun: true,
    inputs: {
      for (final e in sim.inputs.entries)
        if (ids.contains(e.key)) e.key: e.value,
    },
    current: {
      for (final e in sim.current.entries)
        if (ids.contains(e.key)) e.key: e.value,
    },
    periods: {
      for (final e in sim.periods.entries)
        if (p.clocks.any((c) => c.id.toInt() == e.key)) e.key: e.value,
    },
  );
}
