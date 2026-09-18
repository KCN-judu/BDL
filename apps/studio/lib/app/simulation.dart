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

/// Why the design cannot step yet: about a named object, saying what
/// would let it.  Never a compiler word.
@immutable
class SimulationBlocker {
  const SimulationBlocker(this.message, {this.mappingId, this.conceptId, this.checking = false});
  final String message;
  final int? mappingId;
  final int? conceptId;

  /// The compiler has not answered for this revision yet; nothing is wrong.
  final bool checking;
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
    return const [SimulationBlocker('Checking the design…', checking: true)];
  }
  String name(Int64 id) =>
      p.mappings.where((m) => m.id == id).map((m) => m.name).firstOrNull ?? '?';
  final out = <SimulationBlocker>[];
  if (!a.causal) {
    for (final c in a.cycles) {
      out.add(
        SimulationBlocker(
          'These relationships depend on each other in the same instant: '
          '${c.mappingIds.map(name).join(', ')}. One of them must read the previous value '
          'instead.',
          mappingId: c.mappingIds.firstOrNull?.toInt(),
        ),
      );
    }
    if (a.cycles.isEmpty) {
      out.add(const SimulationBlocker('This design contains an instantaneous cycle.'));
    }
  }
  for (final m in p.mappings) {
    final status = a.mappings.where((x) => x.id == m.id).firstOrNull?.status;
    if (status == pb.MappingStatus.MAPPING_STATUS_INVALID) {
      out.add(SimulationBlocker('${m.name} has no valid definition.', mappingId: m.id.toInt()));
    } else if (!m.hasDefinition() && !m.signature.isUnitDomain) {
      out.add(
        SimulationBlocker(
          '${m.name} has no definition. A relationship that reads something needs one '
          'before the design can run.',
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
          '${c.name} needs a value form (Quantity, On / off or Count) before ${m.name} can '
          'be given a value.',
          conceptId: c.id.toInt(),
          mappingId: m.id.toInt(),
        ),
      );
    } else if (!s.editor.simulation.current.containsKey(m.id.toInt())) {
      out.add(
        SimulationBlocker(
          '${m.name} needs a value before simulation can step.',
          mappingId: m.id.toInt(),
        ),
      );
    }
  }
  return out;
}

/// The value of a value declaration at a tick.  Computed declarations are
/// the daemon's sample.  An input is not echoed by the evaluator, so its
/// value is the one Studio fed for that tick — shown only at ticks where
/// the input's domain activated, as the sample's `active_clock_ids` say.
pb.Value? sampleOf(SimulationState sim, pb.ProjectProjection p, pb.TickSample t, int mappingId) {
  final computed = t.values.where((v) => v.mappingId.toInt() == mappingId).firstOrNull;
  if (computed != null) return computed.value;
  final m = p.mappings.where((m) => m.id.toInt() == mappingId).firstOrNull;
  if (m == null || m.hasDefinition() || !m.signature.isUnitDomain) return null;
  final active = m.hasClockId()
      ? t.activeClockIds.contains(m.clockId)
      : t.activeClockIds.isNotEmpty || p.clocks.isEmpty;
  if (!active) return null;
  return sim.inputs[mappingId]?[t.tick.toInt()];
}

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
