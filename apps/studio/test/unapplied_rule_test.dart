/// A rule nothing applies.  The compiler states it
/// (`reactive.rule_unapplied`, never an error) and offers the value that
/// would apply it (`rule.apply:<id>`); Studio repeats the fact where the
/// designer is looking — the Simulate page's readiness area, its probe,
/// the inspector's Relationship section, the canvas — and applies the fix
/// the service names, never one of its own.  Reducer, page, canvas, and
/// the acceptance scenario against the real bdld: the air-conditioner
/// design from a Source, a Source and a rule to a trace column.
@Tags(['daemon', 'filesystem'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/simulation.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/simulate_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/test_store.dart';

const roomTemp = 0;
const buttonHeld = 1;
const switchState = 2;
const tempSensor = 0;
const buttonInput = 1;
const ctrl = 2;
const acOn = 3;

const note = 'AirConditionerCtrl is a rule nothing applies yet.';
const explanation =
    'A rule has no value of its own; a value that applies it — '
    '`AirConditionerCtrl(TempSensor, ButtonInput)` — is what the simulator and an output can read.';
const fixTitle = 'Add a value that applies AirConditionerCtrl';

/// The design of the problem: two Sources, a rule over both, applied by
/// nothing — or, with [applied], by `acOn : () -> SwitchState`.
pb.ProjectProjection ac({bool applied = false}) =>
    pb.ProjectProjection(revision: Int64(1), name: 'ac', rootPath: '/p')
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
      ..mappings.addAll([
        pb.MappingView(
          id: Int64(tempSensor),
          name: 'TempSensor',
          signature: pb.Signature(inputs: [], output: Int64(roomTemp)),
        ),
        pb.MappingView(
          id: Int64(buttonInput),
          name: 'ButtonInput',
          signature: pb.Signature(inputs: [], output: Int64(buttonHeld)),
        ),
        pb.MappingView(
          id: Int64(ctrl),
          name: 'AirConditionerCtrl',
          signature: pb.Signature(
            inputs: [Int64(roomTemp), Int64(buttonHeld)],
            output: Int64(switchState),
          ),
          definition: pb.Definition(formula: 'RoomTemp > 299.15 K && ButtonHeld'),
        ),
        if (applied)
          pb.MappingView(
            id: Int64(acOn),
            name: 'acOn',
            signature: pb.Signature(inputs: [], output: Int64(switchState)),
            definition: pb.Definition(formula: 'AirConditionerCtrl(TempSensor, ButtonInput)'),
          ),
      ]);

/// The compiler's verdict: causal, and the note on the rule unless applied
/// (the applier's `references` then name the rule).
pb.ProjectAnalysis analysisOf({bool applied = false}) =>
    pb.ProjectAnalysis(revision: Int64(1), causal: true, clockConsistent: true)
      ..mappings.addAll([
        if (applied)
          pb.MappingAnalysis(
            id: Int64(acOn),
            status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
            references: [Int64(ctrl), Int64(tempSensor), Int64(buttonInput)],
          ),
        pb.MappingAnalysis(
          id: Int64(ctrl),
          status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
          diagnostics: applied
              ? []
              : [
                  pb.Diagnostic(
                    code: kRuleUnappliedCode,
                    severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_INFO,
                    mappingId: Int64(ctrl),
                    message: note,
                    explanation: explanation,
                  ),
                ],
        ),
      ]);

AppState connected(pb.ProjectProjection project, pb.ProjectAnalysis analysis) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  analysis: analysis,
  editor: const EditorState(page: StudioPage.simulate),
);

pb.Value semantic(int concept, pb.Value repr) => pb.Value(
  semantic: pb.SemanticValue(conceptId: Int64(concept), repr: repr),
);

/// Both Sources given a value.
AppState supplied(AppState s) {
  s = reduce(
    s,
    SimulationInputChanged(
      mappingId: tempSensor,
      value: semantic(
        roomTemp,
        pb.Value(quantity: pb.Quantity(dim: pb.Dim(temperature: 1), value: 300)),
      ),
    ),
  ).state;
  return reduce(
    s,
    SimulationInputChanged(
      mappingId: buttonInput,
      value: semantic(buttonHeld, pb.Value(boolean: true)),
    ),
  ).state;
}

/// The service's answer for the rule: the one ready fix.
pb.SemanticActionsResponse readyFix() => pb.SemanticActionsResponse(
  revision: Int64(1),
  entity: pb.EntityRef(mappingId: Int64(ctrl)),
  actions: [
    pb.SemanticActionView(
      id: 'rule.apply:decl#$ctrl',
      title: 'Add a value that applies `AirConditionerCtrl`',
      kind: 'quick_fix',
      applicability: pb.ActionApplicability.ACTION_APPLICABILITY_READY,
      explanation: '`AirConditionerCtrl` is a rule: it has no value of its own.',
      edits: [
        pb.EditOp(
          createMapping: pb.CreateMapping(
            name: 'airConditionerCtrl',
            signature: pb.Signature(inputs: [], output: Int64(switchState)),
            definition: pb.Definition(formula: 'AirConditionerCtrl(TempSensor, ButtonInput)'),
          ),
        ),
      ],
      addresses: [kRuleUnappliedCode],
    ),
  ],
);

void wide(WidgetTester t) {
  t.view.physicalSize = const Size(1280, 720);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
}

Widget page(AppState s, void Function(AppAction) d) => MaterialApp(
  theme: macTheme(Brightness.light),
  home: Scaffold(
    body: SizedBox(
      width: 1280,
      height: 700,
      child: SimulatePage(state: s, dispatch: d),
    ),
  ),
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

void main() {
  group('reducer', () {
    test('the note is read off the analysis and never blocks a step', () {
      final s = supplied(connected(ac(), analysisOf()));
      expect(simulationBlockers(s), isEmpty);
      final notes = simulationNotes(s);
      expect(notes.single.mappingId, ctrl);
      expect(notes.single.message, note);
      expect(notes.single.actionKind, kApplyRuleActionKind);
      expect(isUnappliedRule(s, ctrl), isTrue);
      expect(appliersOf(s, ctrl), isEmpty);
      expect(reduce(s, const SimulationStepRequested(1)).effects, isNotEmpty, reason: 'steps');
      // applied: no note, the applier named by the analysis
      final applied = connected(ac(applied: true), analysisOf(applied: true));
      expect(simulationNotes(applied), isEmpty);
      expect(isUnappliedRule(applied, ctrl), isFalse);
      expect(appliersOf(applied, ctrl).map((m) => m.name), ['acOn']);
      // a stale analysis says nothing
      final stale = s.copyWith(project: ac()..revision = Int64(2));
      expect(simulationNotes(stale), isEmpty);
      expect(appliersOf(stale, ctrl), isEmpty);
    });

    test('a fix chosen where the finding is shown selects, asks, and applies when ready', () {
      final s = supplied(connected(ac(), analysisOf()));
      // chosen: the rule is selected and its actions asked for, the choice kept
      final t = reduce(
        s,
        const SemanticActionChosen(
          selection: MappingSelected(ctrl),
          actionKind: kApplyRuleActionKind,
        ),
      );
      expect(t.state.editor.selection, const MappingSelected(ctrl));
      final ask = t.effects.whereType<ListSemanticActions>().single;
      expect(ask.entity.mappingId.toInt(), ctrl);
      expect(t.state.editor.actions!.applyOnArrival, kApplyRuleActionKind);
      expect(t.state.editor.actions!.pending, isTrue);
      // arrived ready: applied at once — one revisioned edit, the service's
      final u = reduce(
        t.state,
        SemanticActionsReceived(generation: ask.generation, result: readyFix()),
      );
      final edit = u.effects.whereType<ApplyEdit>().single;
      expect(edit.baseRevision, 1);
      expect(edit.op.createMapping.name, 'airConditionerCtrl');
      expect(
        edit.op.createMapping.definition.formula,
        'AirConditionerCtrl(TempSensor, ButtonInput)',
      );
      expect(u.state.editor.actions, isNull, reason: 'consumed');
      // arrived needing a choice: nothing applied, the choice is on show
      final choice = readyFix()
        ..actions.single.applicability = pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE
        ..actions.single.edits.clear()
        ..actions.single.options.addAll([
          pb.ActionChoiceView(label: 'AirConditionerCtrl(TempSensor, ButtonInput)'),
          pb.ActionChoiceView(label: 'AirConditionerCtrl(TempSetpoint, ButtonInput)'),
        ]);
      final v = reduce(
        t.state,
        SemanticActionsReceived(generation: ask.generation, result: choice),
      );
      expect(v.effects, isEmpty);
      expect(v.state.editor.actions!.pending, isFalse);
      expect(v.state.editor.actions!.applyOnArrival, isNull);
      expect(v.state.editor.actions!.actions.single.options, hasLength(2));
    });
  });

  group('page', () {
    testWidgets('the readiness area says who applies nothing, offers one Fix, keeps Step', (
      t,
    ) async {
      wide(t);
      final dispatched = <AppAction>[];
      final s = supplied(connected(ac(), analysisOf()));
      await t.pumpWidget(page(s, dispatched.add));
      expect(find.byKey(const ValueKey('readiness-note-$ctrl')), findsOneWidget);
      expect(find.text(note), findsOneWidget);
      expect(find.text(explanation), findsOneWidget);
      expect(find.text(fixTitle), findsOneWidget);
      // Step is not disabled by a note
      await t.tap(find.text('Step'));
      expect(dispatched.whereType<SimulationStepRequested>().single.ticks, 1);
      // Show selects the rule
      await t.tap(find.text('Show'));
      expect(
        dispatched.whereType<SelectionChanged>().single.selection,
        const MappingSelected(ctrl),
      );
      // the Fix is chosen by its title: selects, asks, applies when ready
      await t.tap(find.text(fixTitle));
      final chosen = dispatched.whereType<SemanticActionChosen>().single;
      expect(chosen.selection, const MappingSelected(ctrl));
      expect(chosen.actionKind, kApplyRuleActionKind);
      // once the service's list is on show for the rule, its fix as described
      final ask = reduce(s, chosen);
      final listed = reduce(
        ask.state,
        SemanticActionsReceived(
          generation: ask.effects.whereType<ListSemanticActions>().single.generation,
          result: readyFix()
            ..actions.single.applicability = pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED
            ..actions.single.reason = 'no value produces `RoomTemp` yet',
        ),
      ).state;
      await t.pumpWidget(page(listed, dispatched.add));
      // in the note and in the probe (the rule is now selected): the same fix
      expect(find.text('Not possible yet: no value produces `RoomTemp` yet'), findsNWidgets(2));
      expect(find.text('Add a value that applies `AirConditionerCtrl`'), findsNWidgets(2));
    });

    testWidgets('a rule applied by nothing carries no note once applied; the trace gains the '
        'column', (t) async {
      wide(t);
      final s = supplied(connected(ac(applied: true), analysisOf(applied: true)));
      await t.pumpWidget(page(s, (_) {}));
      expect(find.byKey(const ValueKey('readiness-note-$ctrl')), findsNothing);
      expect(find.byKey(const ValueKey('simulation-readiness')), findsOneWidget);
      // the trace's columns: the values, never the rule
      final stepped = reduce(s, const SimulationStepRequested(1)).state;
      final run = reduce(
        stepped,
        SimulationReceived(
          generation: stepped.editor.simulation.generation,
          response: pb.SimulationResponse(
            revision: Int64(1),
            nextTick: Int64(1),
            samples: [
              pb.TickSample(
                tick: Int64(0),
                values: [
                  pb.DeclarationSample(mappingId: Int64(acOn), rendered: 'SwitchState(true)'),
                  pb.DeclarationSample(mappingId: Int64(ctrl), rendered: '<function>'),
                ],
              ),
            ],
          ),
        ),
      ).state;
      await t.pumpWidget(page(run, (_) {}));
      expect(find.text('acOn'), findsOneWidget);
      expect(find.text('SwitchState(true)'), findsOneWidget);
      expect(find.text('AirConditionerCtrl'), findsNothing, reason: 'a rule has no column');
      expect(find.text('<function>'), findsNothing);
    });

    testWidgets('the probe for a rule says it has no value per tick and links its appliers or '
        'the Fix', (t) async {
      wide(t);
      final dispatched = <AppAction>[];
      // applied: the appliers, as links
      var s = connected(ac(applied: true), analysisOf(applied: true)).copyWith(
        editor: const EditorState(page: StudioPage.simulate, selection: MappingSelected(ctrl)),
      );
      await t.pumpWidget(page(s, dispatched.add));
      expect(
        find.text(
          'A rule: it has no value of its own. A value whose formula applies it is what the '
          'simulator samples.',
        ),
        findsOneWidget,
      );
      expect(find.text('Applied in'), findsOneWidget);
      await t.tap(find.text('acOn'));
      expect(
        dispatched.whereType<SelectionChanged>().single.selection,
        const MappingSelected(acOn),
      );
      expect(find.text('No value applies it yet.'), findsNothing);
      // applied by nothing: the sentence and the Fix
      s = connected(ac(), analysisOf()).copyWith(
        editor: const EditorState(page: StudioPage.simulate, selection: MappingSelected(ctrl)),
      );
      await t.pumpWidget(page(s, dispatched.add));
      expect(find.text('No value applies it yet.'), findsOneWidget);
      expect(find.text('Applied in'), findsNothing);
      // the Fix is in the note and in the probe: one meaning, the same words
      expect(find.text(fixTitle), findsNWidgets(2));
    });

    testWidgets('the inspector shows the finding with its Fix in Relationship, not in Fixes', (
      t,
    ) async {
      wide(t);
      final s = connected(ac(), analysisOf()).copyWith(
        editor: EditorState(
          selection: const MappingSelected(ctrl),
          actions: SemanticActionsState(
            entity: pb.EntityRef(mappingId: Int64(ctrl)),
            revision: 1,
            generation: 1,
            actions: readyFix().actions,
            pending: false,
          ),
        ),
      );
      await t.pumpWidget(
        MaterialApp(
          theme: macTheme(Brightness.light),
          home: Scaffold(
            body: SizedBox(
              width: 320,
              height: 1400,
              child: Inspector(state: s, dispatch: (_) {}),
            ),
          ),
        ),
      );
      expect(find.text(note), findsOneWidget);
      expect(find.text('Add a value that applies `AirConditionerCtrl`'), findsOneWidget);
      expect(find.text('Fixes'), findsNothing, reason: 'one home: beside its finding');
    });

    test('the canvas draws an unapplied rule with a hollow output socket and the word', () {
      final scene = buildScene(ac(), const {}, unapplied: const {ctrl});
      final node = scene.nodes.firstWhere((n) => n.title == 'AirConditionerCtrl');
      expect(node.unapplied, isTrue);
      expect(node.declared, isFalse);
      final out = node.sockets.singleWhere((s) => s.ref.side == SocketSide.output);
      expect(out.open, isTrue, reason: 'no value comes out of it');
      expect(out.kind, SocketKind.onOff, reason: 'the value form is known');
      for (final s in node.sockets.where((s) => s.ref.side == SocketSide.input)) {
        expect(s.open, isFalse);
      }
      final applied = buildScene(ac(applied: true), const {});
      final same = applied.nodes.firstWhere((n) => n.title == 'AirConditionerCtrl');
      expect(same.unapplied, isFalse);
      expect(same.sockets.singleWhere((s) => s.ref.side == SocketSide.output).open, isFalse);
    });
  });

  group('e2e', () {
    final bdld = _findBdld();
    late TestStore store;
    late Directory dir;

    Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
    Future<AppState> analysed() => store.until((s) => s.analysis?.revision.toInt() == s.revision);
    int conceptId(String n) =>
        store.state.project!.concepts.firstWhere((c) => c.name == n).id.toInt();
    int mappingId(String n) =>
        store.state.project!.mappings.firstWhere((m) => m.name == n).id.toInt();

    Future<int> concept(String name, pb.Representation r) async {
      store.dispatch(CreateConceptRequested(name: name, representation: r));
      await settled();
      return conceptId(name);
    }

    Future<int> mapping(String name, List<int> inputs, int output, {String? formula}) async {
      store.dispatch(CreateMappingRequested(name: name, inputs: inputs, output: output));
      await settled();
      final id = mappingId(name);
      if (formula != null) {
        store.dispatch(DefinitionDraftChanged(mappingId: id, source: formula));
        await store.until((s) => s.draft(id)?.check == DraftCheck.checked);
        store.dispatch(CommitDefinitionRequested(id));
        await store.until((s) => s.committedDefinition(id) == formula);
      }
      return id;
    }

    test('acceptance: the note, one Fix, the value, the column', () async {
      store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
      store.dispatch(const AppStarted());
      await store.until((s) => s.connection is Connected);
      dir = await Directory.systemTemp.createTemp('bdl-studio-ac');
      store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, 'ac'), name: 'ac'));
      await store.until((s) => s.project != null);
      try {
        final temp = await concept('RoomTemp', pb.Representation(quantity: pb.Dim(temperature: 1)));
        final held = await concept('ButtonHeld', pb.Representation(boolean: pb.Unit()));
        final sw = await concept('SwitchState', pb.Representation(boolean: pb.Unit()));
        final sensor = await mapping('TempSensor', [], temp);
        final button = await mapping('ButtonInput', [], held);
        final rule = await mapping(
          'AirConditionerCtrl',
          [temp, held],
          sw,
          formula: 'RoomTemp > 299.15 K && ButtonHeld',
        );
        store.dispatch(const PageSelected(StudioPage.simulate));
        await analysed();

        // the compiler's note, on the Simulate page as a note (not a blocker)
        final notes = simulationNotes(store.state);
        expect(notes.single.mappingId, rule);
        expect(notes.single.message, 'AirConditionerCtrl is a rule nothing applies yet.');
        expect(notes.single.explanation, contains('`AirConditionerCtrl(TempSensor, ButtonInput)`'));
        expect(
          simulationBlockers(store.state).map((b) => b.names.join(' ')),
          isNot(contains(contains('AirConditionerCtrl'))),
        );
        expect(store.state.analysis!.diagnostics.every((d) => !d.hasSpan()), isTrue);

        // one click on the Fix: selects the rule, asks the service, applies
        store.dispatch(
          SemanticActionChosen(selection: MappingSelected(rule), actionKind: kApplyRuleActionKind),
        );
        await store.until((s) => s.project!.mappings.any((m) => m.name == 'airConditionerCtrl'));
        // ignore: avoid_print
        print(
          'DEBUG actions=${store.actions.map((a) => a.runtimeType).toList()} sel=${store.state.editor.selection} entity=${store.state.selectedEntity} acts=${store.state.editor.actions?.actions.map((a) => '${a.id} ${a.applicability} ${a.reason}')} rev=${store.state.revision} arev=${store.state.editor.actions?.revision} err=${store.state.editor.lastError}',
        );
        final value = store.state.project!.mappings.firstWhere(
          (m) => m.name == 'airConditionerCtrl',
        );
        expect(value.signature.inputs, isEmpty);
        expect(value.signature.output.toInt(), sw);
        expect(value.definition.formula, 'AirConditionerCtrl(TempSensor, ButtonInput)');
        expect(value.hasClockId(), isFalse);
        await analysed();
        expect(simulationNotes(store.state), isEmpty);
        expect(appliersOf(store.state, rule).map((m) => m.name), ['airConditionerCtrl']);
        expect(
          store.state.analysis!.mappings.firstWhere((m) => m.id == value.id).status,
          pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
        );

        // the Sources given, a step: the value has a column and a value
        store.dispatch(
          SimulationInputChanged(
            mappingId: sensor,
            value: semantic(
              temp,
              pb.Value(quantity: pb.Quantity(dim: pb.Dim(temperature: 1), value: 300)),
            ),
          ),
        );
        store.dispatch(
          SimulationInputChanged(mappingId: button, value: semantic(held, pb.Value(boolean: true))),
        );
        expect(simulationBlockers(store.state), isEmpty);
        final gen = store.state.editor.simulation.generation;
        store.dispatch(const SimulationStepRequested(1));
        expect(store.state.editor.simulation.generation, gen + 1);
        final s = await store.until((x) => !x.editor.simulation.pending);
        expect(s.editor.simulation.error, isNull, reason: '${s.editor.simulation.error}');
        final sample = s.editor.simulation.samples.single;
        final rendered = sample.values.firstWhere((v) => v.mappingId == value.id).rendered;
        expect(rendered, 'SwitchState(on)');
      } finally {
        await store.dispose();
        await dir.delete(recursive: true);
      }
    }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);
  });
}
