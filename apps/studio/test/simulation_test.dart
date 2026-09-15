/// The Simulate page: Studio's input trace and schedule, bdld's values.
/// Reducer (trace extension, generations, revision gating), the page (input
/// controls from value forms, product-language failures, the trace table),
/// and — against the real bdld — the lamp (Tilt → Brightness through
/// dimByTilt), a delay accumulator and a two-domain sync.
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/simulate_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/test_store.dart';

const tilt = 0;
const brightness = 1;
const held = 2;
const tiltIn = 0;
const dim = 1;
const bright = 2;
const heldIn = 3;
const interaction = 0;

pb.ProjectProjection lamp({int revision = 1}) =>
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
        pb.ConceptView(
          id: Int64(held),
          name: 'Held',
          representation: pb.Representation(boolean: pb.Unit()),
        ),
      ])
      ..clocks.add(pb.ClockView(id: Int64(interaction), name: 'interaction'))
      ..mappings.addAll([
        pb.MappingView(
          id: Int64(tiltIn),
          name: 'tilt',
          signature: pb.Signature(inputs: [], output: Int64(tilt)),
          clockId: Int64(interaction),
        ),
        pb.MappingView(
          id: Int64(dim),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(tilt)], output: Int64(brightness)),
          definition: pb.Definition(formula: 'Tilt / 90 deg'),
        ),
        pb.MappingView(
          id: Int64(bright),
          name: 'brightness',
          signature: pb.Signature(inputs: [], output: Int64(brightness)),
          definition: pb.Definition(formula: 'dimByTilt(tilt)'),
          clockId: Int64(interaction),
        ),
        pb.MappingView(
          id: Int64(heldIn),
          name: 'held',
          signature: pb.Signature(inputs: [], output: Int64(held)),
        ),
      ]);

AppState connected(pb.ProjectProjection project) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  analysis: pb.ProjectAnalysis(revision: project.revision, causal: true),
  editor: const EditorState(page: StudioPage.simulate),
);

pb.Value angle(double deg) => pb.Value(
  semantic: pb.SemanticValue(
    conceptId: Int64(tilt),
    repr: pb.Value(
      quantity: pb.Quantity(dim: pb.Dim(angle: 1), value: deg * 3.141592653589793 / 180),
    ),
  ),
);

pb.SimulationResponse response({
  required int revision,
  required int nextTick,
  List<pb.TickSample> samples = const [],
  pb.Diagnostic? error,
}) => pb.SimulationResponse(
  revision: Int64(revision),
  nextTick: Int64(nextTick),
  samples: samples,
  error: error,
);

pb.TickSample sample(int tick, Map<int, String> rendered) => pb.TickSample(
  tick: Int64(tick),
  activeClockIds: [Int64(interaction)],
  values: [
    for (final e in rendered.entries)
      pb.DeclarationSample(mappingId: Int64(e.key), rendered: e.value),
  ],
);

String? _findBdld() {
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && File(env).existsSync()) return env;
  for (final rel in ['../../target/debug/bdld', '../../target/release/bdld']) {
    final f = p.normalize(p.join(Directory.current.path, rel));
    if (File(f).existsSync()) return f;
  }
  return null;
}

Widget page(AppState s, void Function(AppAction) d) => MaterialApp(
  theme: macTheme(Brightness.light),
  home: Scaffold(
    body: SizedBox(
      width: 1000,
      height: 700,
      child: SimulatePage(state: s, dispatch: d),
    ),
  ),
);

void main() {
  group('reducer', () {
    test('a step extends the authored trace with the current inputs and re-runs from tick 0', () {
      var s = connected(lamp());
      s = reduce(s, SimulationInputChanged(mappingId: tiltIn, value: angle(30))).state;
      var t = reduce(s, const SimulationStepRequested(2));
      final run = t.effects.whereType<RunSimulation>().single;
      expect(run.ticks, 2);
      expect(run.inputs.map((i) => (i.mappingId.toInt(), i.tick.toInt())), [
        (tiltIn, 0),
        (tiltIn, 1),
      ]);
      expect(run.schedule.single.clockId.toInt(), interaction);
      expect(run.schedule.single.period.toInt(), 1);
      expect(t.state.editor.simulation.pending, isTrue);
      final g = t.state.editor.simulation.generation;
      s = reduce(
        t.state,
        SimulationReceived(
          generation: g,
          response: response(
            revision: 1,
            nextTick: 2,
            samples: [
              sample(0, {bright: 'Brightness(0.33)'}),
              sample(1, {bright: 'Brightness(0.33)'}),
            ],
          ),
        ),
      ).state;
      expect(s.editor.simulation.nextTick, 2);
      expect(s.editor.simulation.samples, hasLength(2));
      // the input changes: earlier ticks keep their value, the next tick takes the new one
      s = reduce(s, SimulationInputChanged(mappingId: tiltIn, value: angle(90))).state;
      t = reduce(s, const SimulationStepRequested(1));
      final again = t.effects.whereType<RunSimulation>().single;
      expect(again.ticks, 3, reason: 're-created from tick 0, stepped to 3');
      final fed = {
        for (final i in again.inputs) i.tick.toInt(): i.value.semantic.repr.quantity.value,
      };
      expect(fed[0], closeTo(0.5236, 1e-3));
      expect(fed[2], closeTo(1.5708, 1e-3));
    });

    test('only the latest generation at the held revision lands; a new revision drops the run', () {
      var s = connected(lamp());
      s = reduce(s, SimulationInputChanged(mappingId: tiltIn, value: angle(0))).state;
      s = reduce(s, const SimulationStepRequested(1)).state;
      s = reduce(s, const SimulationStepRequested(1)).state;
      final g = s.editor.simulation.generation;
      s = reduce(
        s,
        SimulationReceived(generation: g - 1, response: response(revision: 1, nextTick: 1)),
      ).state;
      expect(s.editor.simulation.samples, isEmpty);
      s = reduce(
        s,
        SimulationReceived(generation: g, response: response(revision: 5, nextTick: 2)),
      ).state;
      expect(s.editor.simulation.samples, isEmpty, reason: 'another revision');
      s = reduce(
        s,
        SimulationReceived(
          generation: g,
          response: response(revision: 1, nextTick: 2, samples: [sample(0, {}), sample(1, {})]),
        ),
      ).state;
      expect(s.editor.simulation.hasRun, isTrue);
      // the design changes: samples go, the authored inputs stay
      s = reduce(s, ProjectReceived(lamp(revision: 2), fromRequest: false)).state;
      expect(s.editor.simulation.hasRun, isFalse);
      expect(s.editor.simulation.nextTick, 0);
      expect(s.editor.simulation.current[tiltIn], isNotNull);
      expect(s.editor.simulation.inputs[tiltIn], isNotEmpty);
    });

    test('a period change and reset start over; failures are kept in place', () {
      var s = connected(lamp());
      s = reduce(s, const SimulationStepRequested(1)).state;
      final g = s.editor.simulation.generation;
      s = reduce(
        s,
        SimulationReceived(
          generation: g,
          response: response(revision: 1, nextTick: 1, samples: [sample(0, {})]),
        ),
      ).state;
      s = reduce(s, const SimulationPeriodChanged(clockId: interaction, period: 2)).state;
      expect(s.editor.simulation.periods[interaction], 2);
      expect(s.editor.simulation.hasRun, isFalse);
      s = reduce(s, const SimulationStepRequested(1)).state;
      s = reduce(
        s,
        SimulationFailed(
          generation: s.editor.simulation.generation,
          code: 'simulation.not_causal',
          message: 'cycle',
        ),
      ).state;
      expect(s.editor.simulation.failure, 'cycle');
      expect(s.editor.simulation.pending, isFalse);
      s = reduce(s, const SimulationResetRequested()).state;
      expect(s.editor.simulation.failure, isNull);
      expect(s.editor.simulation.nextTick, 0);
    });
  });

  group('page', () {
    testWidgets('inputs are controls by value form; the trace shows bdld\'s rendering', (t) async {
      final dispatched = <AppAction>[];
      var s = connected(lamp());
      s = reduce(s, const SimulationStepRequested(1)).state;
      s = reduce(
        s,
        SimulationReceived(
          generation: s.editor.simulation.generation,
          response: response(
            revision: 1,
            nextTick: 1,
            samples: [
              sample(0, {
                tiltIn: 'Tilt(0.5236 rad)',
                bright: 'Brightness(0.3333)',
                dim: '<function>',
              }),
            ],
          ),
        ),
      ).state;
      await t.pumpWidget(page(s, dispatched.add));
      // a quantity input is a number field with its unit; an on/off input a switch
      expect(find.byKey(const ValueKey('input-$tiltIn')), findsOneWidget);
      expect(find.text('rad'), findsOneWidget);
      expect(find.byType(Switch), findsOneWidget);
      expect(find.text('1 tick evaluated.'), findsOneWidget);
      expect(find.text('Brightness(0.3333)'), findsOneWidget);
      expect(find.text('<function>'), findsNothing, reason: 'a function is not a column');
      await t.tap(find.byType(Switch));
      await t.pump();
      final changed = dispatched.whereType<SimulationInputChanged>().single;
      expect(changed.mappingId, heldIn);
      expect(changed.value.semantic.repr.boolean, isTrue);
      await t.tap(find.text('Step'));
      expect(dispatched.whereType<SimulationStepRequested>().single.ticks, 1);
    });

    testWidgets('failures are worded for the designer', (t) async {
      var s = connected(lamp());
      s = reduce(s, const SimulationStepRequested(1)).state;
      s = reduce(
        s,
        SimulationReceived(
          generation: s.editor.simulation.generation,
          response: response(
            revision: 1,
            nextTick: 0,
            error: pb.Diagnostic(
              code: 'simulation.missing_input',
              severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
              mappingId: Int64(tiltIn),
              message: 'runtime: missing input decl 0 at tick 0',
            ),
          ),
        ),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(find.text('tilt needs a value for this step.'), findsOneWidget);
      expect(find.textContaining('decl 0'), findsNothing);
      // an uncausal design says so before any step
      final cyclic = s.copyWith(
        analysis: pb.ProjectAnalysis(revision: Int64(1), causal: false),
        editor: s.editor.copyWith(simulation: const SimulationState()),
      );
      await t.pumpWidget(page(cyclic, (_) {}));
      expect(find.text('The design contains an instantaneous cycle.'), findsOneWidget);
    });
  });

  group('e2e', () {
    final bdld = _findBdld();
    late TestStore store;
    late Directory dir;

    Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
    int conceptId(String n) =>
        store.state.project!.concepts.firstWhere((c) => c.name == n).id.toInt();
    int mappingId(String n) =>
        store.state.project!.mappings.firstWhere((m) => m.name == n).id.toInt();
    int clockId(String n) => store.state.project!.clocks.firstWhere((c) => c.name == n).id.toInt();

    Future<void> open(String name) async {
      store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
      store.dispatch(const AppStarted());
      await store.until((s) => s.connection is Connected);
      dir = await Directory.systemTemp.createTemp('bdl-studio-sim');
      store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, name), name: name));
      await store.until((s) => s.project != null);
    }

    Future<int> concept(String name, pb.Representation r) async {
      store.dispatch(CreateConceptRequested(name: name, representation: r));
      await settled();
      return conceptId(name);
    }

    Future<int> mapping(
      String name,
      List<int> inputs,
      int output, {
      String? formula,
      int? clock,
    }) async {
      store.dispatch(CreateMappingRequested(name: name, inputs: inputs, output: output));
      await settled();
      final id = mappingId(name);
      if (formula != null) {
        store.dispatch(DefinitionDraftChanged(mappingId: id, source: formula));
        await store.until((s) => s.draft(id)?.check == DraftCheck.checked);
        store.dispatch(CommitDefinitionRequested(id));
        await store.until((s) => s.committedDefinition(id) == formula);
      }
      if (clock != null) {
        store.dispatch(SetMappingClockRequested(mappingId: id, clockId: clock));
        await settled();
      }
      return id;
    }

    Future<int> clock(String name) async {
      store.dispatch(CreateClockDomainRequested(name));
      await settled();
      return clockId(name);
    }

    Future<AppState> step([int n = 1]) async {
      store.dispatch(SimulationStepRequested(n));
      return store.until((s) => !s.editor.simulation.pending);
    }

    String rendered(AppState s, int tick, int mapping) => s.editor.simulation.samples
        .firstWhere((x) => x.tick.toInt() == tick)
        .values
        .firstWhere((v) => v.mappingId.toInt() == mapping)
        .rendered;

    Future<void> close() async {
      await store.dispose();
      await dir.delete(recursive: true);
    }

    test('lamp: Tilt 0, 30, 60, 90 deg → Brightness 0, ⅓, ⅔, 1', () async {
      await open('lamp');
      try {
        final tiltC = await concept('Tilt', pb.Representation(quantity: pb.Dim(angle: 1)));
        final brightC = await concept('Brightness', pb.Representation(quantity: pb.Dim()));
        final inter = await clock('interaction');
        final tiltM = await mapping('tilt', [], tiltC, clock: inter);
        await mapping('dimByTilt', [tiltC], brightC, formula: 'Tilt / 90 deg');
        final brightM = await mapping(
          'brightness',
          [],
          brightC,
          formula: 'dimByTilt(tilt)',
          clock: inter,
        );
        store.dispatch(const PageSelected(StudioPage.simulate));
        await store.until((s) => s.analysis?.revision.toInt() == s.revision);
        expect(store.state.analysis!.causal, isTrue);

        var s = store.state;
        final t0 = DateTime.now();
        for (final deg in [0.0, 30.0, 60.0, 90.0]) {
          store.dispatch(SimulationInputChanged(mappingId: tiltM, value: angle(deg)));
          s = await step();
          expect(s.editor.simulation.error, isNull, reason: '${s.editor.simulation.error}');
        }
        // ignore: avoid_print
        print(
          'four simulation steps (each a restart + step): ${DateTime.now().difference(t0).inMilliseconds} ms',
        );
        expect(s.editor.simulation.nextTick, 4);
        final b = [for (var t = 0; t < 4; t++) rendered(s, t, brightM)];
        expect(b[0], 'Brightness(0)');
        expect(b[1], startsWith('Brightness(0.333'));
        expect(b[2], startsWith('Brightness(0.666'));
        expect(b[3], 'Brightness(1)');
        // inputs are Studio's own trace (the evaluator records computed values only)
        expect(
          s.editor.simulation.inputs[tiltM]![1]!.semantic.repr.quantity.value,
          closeTo(0.5236, 1e-3),
        );
      } finally {
        await close();
      }
    }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);

    test('delay: tick 0 is the initial value, tick n+1 the previous accumulation', () async {
      await open('delay');
      try {
        final level = await concept('Level', pb.Representation(quantity: pb.Dim()));
        final main_ = await clock('main');
        final x = await mapping('x', [], level, clock: main_);
        final acc = await mapping('acc', [], level, formula: 'delay(0, acc + x)', clock: main_);
        store.dispatch(const PageSelected(StudioPage.simulate));
        await store.until((s) => s.analysis?.revision.toInt() == s.revision);
        var s = store.state;
        for (final v in [1.0, 2.0, 3.0, 4.0]) {
          store.dispatch(
            SimulationInputChanged(
              mappingId: x,
              value: pb.Value(
                semantic: pb.SemanticValue(
                  conceptId: Int64(level),
                  repr: pb.Value(
                    quantity: pb.Quantity(dim: pb.Dim(), value: v),
                  ),
                ),
              ),
            ),
          );
          s = await step();
          expect(s.editor.simulation.error, isNull);
        }
        expect(
          [for (var t = 0; t < 4; t++) rendered(s, t, acc)],
          ['Level(0)', 'Level(1)', 'Level(3)', 'Level(6)'],
        );
      } finally {
        await close();
      }
    }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);

    test('sync: a source activation at the same global tick is not yet visible', () async {
      await open('sync');
      try {
        final level = await concept('Level', pb.Representation(quantity: pb.Dim()));
        final fast = await clock('fast');
        final slow = await clock('slow');
        final x = await mapping('x', [], level, clock: fast);
        final y = await mapping('y', [], level, formula: 'sync(fast, -1, x)', clock: slow);
        store.dispatch(const PageSelected(StudioPage.simulate));
        await store.until((s) => s.analysis?.revision.toInt() == s.revision);
        expect(store.state.analysis!.clockConsistent, isTrue);
        store.dispatch(SimulationPeriodChanged(clockId: slow, period: 2));
        var s = store.state;
        for (var n = 0; n < 5; n++) {
          store.dispatch(
            SimulationInputChanged(
              mappingId: x,
              value: pb.Value(
                semantic: pb.SemanticValue(
                  conceptId: Int64(level),
                  repr: pb.Value(
                    quantity: pb.Quantity(dim: pb.Dim(), value: n * 10.0),
                  ),
                ),
              ),
            ),
          );
          s = await step();
          expect(s.editor.simulation.error, isNull, reason: '${s.editor.simulation.error}');
        }
        // slow activates at 0, 2, 4: y sees the fast activation strictly before
        String? at(int t) => s.editor.simulation.samples
            .firstWhere((x) => x.tick.toInt() == t)
            .values
            .where((v) => v.mappingId.toInt() == y)
            .map((v) => v.rendered)
            .firstOrNull;
        expect(at(0), 'Level(-1)', reason: 'nothing before tick 0');
        expect(at(1), isNull, reason: 'slow is not active');
        expect(at(2), 'Level(10)', reason: 'x at tick 1, not x at tick 2');
        expect(at(4), 'Level(30)');
        final active = s.editor.simulation.samples[1].activeClockIds.map((c) => c.toInt());
        expect(active, [fast]);
      } finally {
        await close();
      }
    }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);
  });
}
