/// The Deploy page: target list from bdld, target-relative analysis kept
/// only for the revision on screen, the verdict worded per board, placement
/// and dead end rendered from the compiler's structures.
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/mac/tokens.dart';
import 'package:bdl_studio/ui/pages/deploy_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/test_store.dart';

pb.ProjectProjection rover({int revision = 1}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'rover', rootPath: '/r')
      ..concepts.add(pb.ConceptView(id: Int64(0), name: 'Speed'))
      ..outputs.add(pb.OutputView(id: Int64(0), name: 'motor', accepts: Int64(0)))
      ..devices.add(
        pb.DeviceView(
          id: Int64(0),
          name: 'drive',
          kind: pb.DeviceKind.DEVICE_KIND_H_BRIDGE_CHANNEL,
          outputId: Int64(0),
          requirements: [
            pb.RequirementLabel(index: 0, capability: 'pwm', label: 'drive PWM'),
            pb.RequirementLabel(index: 1, capability: 'digital_out', label: 'drive direction'),
          ],
        ),
      );

AppState connected(pb.ProjectProjection project, {StudioPage page = StudioPage.deploy}) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  editor: EditorState(page: page),
);

final targets = [
  pb.TargetView(id: 'arduino_nano', name: 'Arduino Nano', resourceCount: 20),
  pb.TargetView(id: 'big_board', name: 'Big Board', resourceCount: 40),
];

pb.DeploymentAnalysis feasible({int revision = 1, String target = 'Arduino Nano'}) =>
    pb.DeploymentAnalysis(
      revision: Int64(revision),
      target: target,
      status: pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE,
      requirements: [
        pb.RequirementView(deviceId: Int64(0), index: 0, capability: 'pwm', label: 'drive PWM'),
        pb.RequirementView(
          deviceId: Int64(0),
          index: 1,
          capability: 'digital_out',
          label: 'drive direction',
        ),
      ],
      assignment: [
        pb.Placement(deviceId: Int64(0), index: 0, resource: 'D3'),
        pb.Placement(deviceId: Int64(0), index: 1, resource: 'D2'),
      ],
    );

pb.DeploymentAnalysis infeasible({int revision = 1}) => pb.DeploymentAnalysis(
  revision: Int64(revision),
  target: 'Arduino Nano',
  status: pb.DeploymentStatus.DEPLOYMENT_STATUS_INFEASIBLE,
  requirements: [
    pb.RequirementView(deviceId: Int64(0), index: 0, capability: 'pwm', label: 'drive PWM'),
  ],
  deadEnd: pb.DeadEnd(deviceId: Int64(0), index: 0, fixedUnavailable: 'D4'),
  diagnostics: [
    pb.Diagnostic(
      code: 'deploy.infeasible',
      severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
      project: pb.Unit(),
      message: 'D4 cannot carry drive PWM on Arduino Nano.',
      explanation:
          'The pin chosen by hand for drive is not on this board or lacks the needed function.',
    ),
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
      width: 900,
      height: 700,
      child: DeployPage(state: s, dispatch: d),
    ),
  ),
);

void main() {
  group('reducer', () {
    test('entering Deploy asks for the boards once; choosing one asks for its analysis', () {
      var t = reduce(
        connected(rover(), page: StudioPage.design),
        const PageSelected(StudioPage.deploy),
      );
      expect(t.effects.whereType<ListTargets>(), hasLength(1));
      t = reduce(t.state, TargetsReceived(targets));
      expect(t.state.editor.deploy.targetsLoaded, isTrue);
      expect(reduce(t.state, const PageSelected(StudioPage.deploy)).effects, isEmpty);
      t = reduce(t.state, const TargetSelected('big_board'));
      final e = t.effects.whereType<AnalyzeDeployment>().single;
      expect(e.targetId, 'big_board');
      expect(t.state.editor.deploy.pending, isTrue);
      expect(t.state.editor.deploy.target!.name, 'Big Board');
    });

    test('an answer is kept only for its generation and the revision on screen', () {
      var s = reduce(connected(rover()), TargetsReceived(targets)).state;
      s = reduce(s, const TargetSelected('arduino_nano')).state;
      final g = s.editor.deploy.generation;
      // a late answer for an earlier request: ignored
      s = reduce(s, DeploymentReceived(generation: g - 1, analysis: feasible())).state;
      expect(s.editor.deploy.analysis, isNull);
      // an answer about another revision: ignored
      s = reduce(s, DeploymentReceived(generation: g, analysis: feasible(revision: 7))).state;
      expect(s.editor.deploy.analysis, isNull);
      s = reduce(s, DeploymentReceived(generation: g, analysis: feasible())).state;
      expect(s.editor.deploy.analysis!.status, pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE);
      expect(s.editor.deploy.pending, isFalse);
      // the design changes: the placement is about an old design — dropped, asked again
      final t = reduce(s, ProjectReceived(rover(revision: 2), fromRequest: false));
      expect(t.state.editor.deploy.analysis, isNull);
      expect(t.effects.whereType<AnalyzeDeployment>().single.generation, g + 1);
      // a same-revision projection (a save) changes nothing
      final same = reduce(s, ProjectReceived(rover(revision: 1)));
      expect(same.state.editor.deploy.analysis, isNotNull);
      expect(same.effects.whereType<AnalyzeDeployment>(), isEmpty);
    });

    test('a failure is shown in place; closing keeps the board, drops the answer', () {
      var s = reduce(connected(rover()), TargetsReceived(targets)).state;
      s = reduce(s, const TargetSelected('arduino_nano')).state;
      final g = s.editor.deploy.generation;
      s = reduce(
        s,
        DeploymentFailed(
          generation: g,
          code: 'deploy.unknown_target',
          message: 'No target named x.',
        ),
      ).state;
      expect(s.editor.deploy.error, 'No target named x.');
      expect(s.editor.deploy.pending, isFalse);
      s = reduce(s, DeploymentReceived(generation: g, analysis: feasible())).state;
      s = reduce(s, const ProjectClosed()).state;
      expect(s.editor.deploy.targetId, 'arduino_nano');
      expect(s.editor.deploy.analysis, isNull);
    });
  });

  group('page', () {
    testWidgets('the verdict is per board; placement is device → requirement → pin', (t) async {
      var s = reduce(connected(rover()), TargetsReceived(targets)).state;
      s = reduce(s, const TargetSelected('arduino_nano')).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(find.text('Checking Arduino Nano…'), findsOneWidget);
      s = reduce(
        s,
        DeploymentReceived(generation: s.editor.deploy.generation, analysis: feasible()),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      final verdict = t.widget<Text>(find.byKey(const ValueKey('deploy-verdict')));
      expect(verdict.data, 'Feasible on Arduino Nano.');
      final tokens = MacTokens.of(t.element(find.byKey(const ValueKey('deploy-verdict'))));
      expect(verdict.style!.color, tokens.settled);
      expect(find.text('→ D3'), findsOneWidget);
      expect(find.text('→ D2'), findsOneWidget);
      expect(find.text('drive PWM'), findsWidgets);
      expect(find.textContaining('invalid'), findsNothing, reason: 'feasibility is not validity');
    });

    testWidgets('an infeasible board shows the dead end, in its own terms', (t) async {
      var s = reduce(connected(rover()), TargetsReceived(targets)).state;
      s = reduce(s, const TargetSelected('arduino_nano')).state;
      s = reduce(
        s,
        DeploymentReceived(generation: s.editor.deploy.generation, analysis: infeasible()),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(
        t.widget<Text>(find.byKey(const ValueKey('deploy-verdict'))).data,
        'Not feasible on Arduino Nano.',
      );
      expect(find.text('D4 cannot carry drive PWM on Arduino Nano.'), findsOneWidget);
      expect(find.textContaining('chosen by hand, D4'), findsOneWidget);
      expect(find.textContaining('minimal'), findsNothing);
    });

    testWidgets('devices are edited in place; the board list is the service\'s', (t) async {
      final dispatched = <AppAction>[];
      var s = reduce(connected(rover()), TargetsReceived(targets)).state;
      await t.pumpWidget(page(s, dispatched.add));
      expect(find.text('drive'), findsOneWidget);
      expect(find.text('H-bridge channel'), findsOneWidget);
      expect(find.text('drive direction'), findsOneWidget);
      await t.tap(find.text('Add device'));
      await t.pump();
      expect(dispatched.whereType<CreateDeviceRequested>().single.outputId, 0);
      await t.tap(find.byKey(const ValueKey('target-picker')));
      await t.pumpAndSettle();
      expect(find.text('Big Board'), findsOneWidget);
      expect(find.text('Arduino Nano'), findsOneWidget);
      await t.tap(find.text('Big Board'));
      await t.pumpAndSettle();
      expect(dispatched.whereType<TargetSelected>().single.targetId, 'big_board');
    });
  });

  group('e2e', () {
    final bdld = _findBdld();
    test('targets and placement come from bdld; a hand-fixed pin makes it infeasible', () async {
      final store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
      final dir = await Directory.systemTemp.createTemp('bdl-studio-deploy-e2e');
      try {
        store.dispatch(const AppStarted());
        await store.until((s) => s.connection is Connected);
        store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, 'rover'), name: 'rover'));
        await store.until((s) => s.project != null);
        Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
        store.dispatch(
          CreateConceptRequested(
            name: 'Speed',
            representation: pb.Representation(quantity: pb.Dim()),
          ),
        );
        await settled();
        final speed = store.state.project!.concepts.single.id.toInt();
        store.dispatch(const CreateClockDomainRequested('main'));
        var s = await settled();
        final main_ = s.project!.clocks.single.id.toInt();
        store.dispatch(CreateOutputRequested(name: 'motor', accepts: speed, clockId: main_));
        s = await settled();
        final motor = s.project!.outputs.single.id.toInt();
        store.dispatch(
          CreateDeviceRequested(
            name: 'driver',
            kind: pb.DeviceKind.DEVICE_KIND_H_BRIDGE_CHANNEL,
            outputId: motor,
          ),
        );
        s = await settled();
        final device = s.project!.devices.single.id.toInt();

        store.dispatch(const PageSelected(StudioPage.deploy));
        s = await store.until((s) => s.editor.deploy.targetsLoaded);
        expect(
          s.editor.deploy.targets.map((t) => t.id),
          containsAll(['arduino_nano', 'big_board']),
        );
        final t0 = DateTime.now();
        store.dispatch(const TargetSelected('arduino_nano'));
        s = await store.until((s) => s.editor.deploy.analysis != null);
        // ignore: avoid_print
        print('deployment analysis round trip: ${DateTime.now().difference(t0).inMilliseconds} ms');
        var a = s.editor.deploy.analysis!;
        expect(a.status, pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE);
        expect(a.revision.toInt(), s.revision);
        expect(a.assignment, hasLength(2));
        expect(a.assignment.first.resource, 'D3');

        // a hand-fixed pin that cannot do PWM: the design changes, the old
        // placement goes, the new answer says why
        final before = s.revision;
        store.dispatch(SetDevicePinRequested(id: device, index: 0, resource: 'D4'));
        s = await store.until(
          (s) =>
              s.revision > before &&
              s.editor.deploy.analysis != null &&
              s.editor.deploy.analysis!.revision.toInt() == s.revision,
        );
        a = s.editor.deploy.analysis!;
        expect(a.status, pb.DeploymentStatus.DEPLOYMENT_STATUS_INFEASIBLE);
        expect(a.deadEnd.whichReason(), pb.DeadEnd_Reason.fixedUnavailable);
        expect(a.diagnostics.first.message, contains('D4 cannot carry'));
        // the semantic analysis is untouched by any board — it is its own
        // answer for the new revision and may land after the placement
        s = await store.until(
          (s) => s.analysis != null && s.analysis!.revision == s.flat!.revision,
        );
        expect(s.analysis, isNotNull);

        // another board: its own verdict
        store.dispatch(const TargetSelected('big_board'));
        s = await store.until((s) => s.editor.deploy.analysis?.target == 'big_board');
        expect(s.editor.deploy.analysis!.revision.toInt(), s.revision);
      } finally {
        await store.dispose();
        await dir.delete(recursive: true);
      }
    }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);
  });
}
