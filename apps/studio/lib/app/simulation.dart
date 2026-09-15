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

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition;
import 'state.dart';

/// Unresolved mappings without inputs: the declarations the evaluator
/// needs a value for at every tick.  (Read off the projection.)
List<pb.MappingView> simulationInputs(pb.ProjectProjection p) => [
  for (final m in p.mappings)
    if (!m.hasDefinition() && m.signature.inputs.isEmpty) m,
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

Transition simulationStepRequested(AppState s, int ticks) {
  final p = s.project;
  if (p == null || ticks < 1) return Transition(s);
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
