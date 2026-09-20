/// The Firmware section of the Deploy page and its transitions: one
/// primary action per state, the daemon's blocker named when there is
/// none, a build and a flash followed through their events, a stale
/// image never offered, several boards never guessed between.  Nothing
/// here judges readiness: the analysis' `build_ready` and `build_blockers`
/// are given, as bdld gives them.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/mac/tokens.dart';
import 'package:bdl_studio/ui/pages/deploy_page.dart';
import 'package:bdl_studio/ui/pages/firmware_section.dart';
import 'package:bdl_studio/ui/welcome/welcome_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

const pico = 'rp2040_pico';

/// Button → Lamp: a Source `pressed`, a value `lit`, an output `lamp`.
pb.ProjectProjection lamp({int revision = 1}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'button-lamp', rootPath: '/demo')
      ..concepts.add(pb.ConceptView(id: Int64(0), name: 'Pressed'))
      ..concepts.add(pb.ConceptView(id: Int64(1), name: 'Lit'))
      ..mappings.add(
        pb.MappingView(
          id: Int64(0),
          name: 'pressed',
          role: pb.RelationshipRole.RELATIONSHIP_ROLE_SOURCE,
          signature: pb.Signature(output: Int64(0)),
        ),
      )
      ..mappings.add(
        pb.MappingView(
          id: Int64(1),
          name: 'lit',
          role: pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE,
          signature: pb.Signature(output: Int64(1)),
        ),
      )
      ..outputs.add(pb.OutputView(id: Int64(0), name: 'lamp', accepts: Int64(1)));

AppState connected(pb.ProjectProjection? project) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  editor: const EditorState(page: StudioPage.deploy),
);

final targets = [pb.TargetView(id: pico, name: 'Raspberry Pi Pico', resourceCount: 27)];

pb.DeploymentAnalysis analysis({
  int revision = 1,
  bool ready = true,
  List<pb.BuildBlocker> blockers = const [],
}) => pb.DeploymentAnalysis(
  revision: Int64(revision),
  target: pico,
  status: pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE,
  designReady: true,
  deployable: ready,
  buildReady: ready,
  buildBlockers: blockers,
);

pb.BuildArtifact artifact({String identity = 'abc'}) => pb.BuildArtifact(
  path: '/demo/build/rp2040_pico/demo-rp2040.uf2',
  kind: 'uf2',
  elfPath: '/demo/build/rp2040_pico/target/thumbv6m-none-eabi/release/demo-rp2040',
  identity: identity,
  builtAt: Int64(1_700_000_000),
  sizeBytes: Int64(6144),
);

pb.BuildStatus built({bool fresh = true}) => pb.BuildStatus(
  targetId: pico,
  stage: pb.BuildStage.BUILD_STAGE_COMPLETED,
  artifact: artifact(),
  artifactFresh: fresh,
  generatedDir: '/demo/build/rp2040_pico',
  command: 'cargo build --release --target thumbv6m-none-eabi --features rp2040',
  triple: 'thumbv6m-none-eabi',
);

pb.FlashDevice volume([String path = '/Volumes/RPI-RP2']) => pb.FlashDevice(
  id: 'uf2:$path',
  method: pb.FlashMethod.FLASH_METHOD_UF2_VOLUME,
  label: 'Raspberry Pi Pico in BOOTSEL mode (RPI-RP2)',
  detail: path,
);

final usb = pb.FlashMethodView(
  method: pb.FlashMethod.FLASH_METHOD_UF2_VOLUME,
  available: true,
  label: 'USB, the board\'s own bootloader',
  hint: 'Hold BOOTSEL while plugging the board in over USB; it appears as a drive named RPI-RP2.',
);

/// A state on the Deploy page with the Pico chosen and its analysis in.
AppState onPico(pb.DeploymentAnalysis a, {pb.ProjectProjection? project}) {
  var s = reduce(connected(project ?? lamp()), TargetsReceived(targets)).state;
  s = reduce(s, const TargetSelected(pico)).state;
  return reduce(s, DeploymentReceived(generation: s.editor.deploy.generation, analysis: a)).state;
}

Widget page(AppState s, void Function(AppAction) d) => MaterialApp(
  theme: macTheme(Brightness.light),
  home: Scaffold(
    body: SizedBox(
      width: 900,
      height: 900,
      child: DeployPage(state: s, dispatch: d),
    ),
  ),
);

Widget welcome(AppState s, void Function(AppAction) d) => MaterialApp(
  theme: macTheme(Brightness.light),
  home: Scaffold(
    body: SizedBox(
      width: 1200,
      height: 800,
      child: WelcomePage(state: s, dispatch: d),
    ),
  ),
);

void main() {
  group('reducer', () {
    test(
      'the analysis arriving asks for the firmware status; a fresh image looks for the board',
      () {
        var s = reduce(connected(lamp()), TargetsReceived(targets)).state;
        s = reduce(s, const TargetSelected(pico)).state;
        final t = reduce(
          s,
          DeploymentReceived(generation: s.editor.deploy.generation, analysis: analysis()),
        );
        final ask = t.effects.whereType<GetBuildStatus>().single;
        expect(ask.targetId, pico);
        final g = t.state.editor.deploy.firmware.statusGeneration;
        expect(ask.generation, g);
        // a late status: ignored
        var u = reduce(t.state, BuildStatusReceived(generation: g - 1, status: built()));
        expect(u.state.editor.deploy.firmware.status, isNull);
        u = reduce(t.state, BuildStatusReceived(generation: g, status: built()));
        expect(u.state.editor.deploy.firmware.artifactFresh, isTrue);
        expect(u.effects.whereType<ListFlashDevices>().single.targetId, pico);
        // a stale one does not
        u = reduce(t.state, BuildStatusReceived(generation: g, status: built(fresh: false)));
        expect(u.effects.whereType<ListFlashDevices>(), isEmpty);
      },
    );

    test('a build is asked at the revision on screen and followed through its events', () {
      final s = onPico(analysis());
      final t = reduce(s, const BuildRequested());
      final e = t.effects.whereType<BuildFirmware>().single;
      expect((e.targetId, e.revision), (pico, 1));
      var fw = t.state.editor.deploy.firmware;
      expect(fw.isBuilding, isTrue);
      // a second click while running does nothing
      expect(reduce(t.state, const BuildRequested()).effects, isEmpty);
      // stages: the message is kept when an event carries only a detail
      var u = reduce(
        t.state,
        BuildProgressReceived(
          pb.BuildProgress(
            targetId: pico,
            stage: pb.BuildStage.BUILD_STAGE_COMPILING,
            message: 'Compiling the firmware',
          ),
        ),
      ).state;
      u = reduce(
        u,
        BuildProgressReceived(
          pb.BuildProgress(
            targetId: pico,
            stage: pb.BuildStage.BUILD_STAGE_COMPILING,
            done: 3,
            detail: 'warning: unused',
          ),
        ),
      ).state;
      fw = u.editor.deploy.firmware;
      expect(fw.building!.message, 'Compiling the firmware');
      expect(fw.building!.done, 3);
      expect(fw.output, ['warning: unused']);
      // an event about another board: ignored
      u = reduce(
        u,
        BuildProgressReceived(
          pb.BuildProgress(targetId: 'arduino_nano', stage: pb.BuildStage.BUILD_STAGE_FAILED),
        ),
      ).state;
      expect(u.editor.deploy.firmware.isBuilding, isTrue);
      // the final event carries the status: the build is over, the board is looked for
      final done = reduce(
        u,
        BuildProgressReceived(
          pb.BuildProgress(
            targetId: pico,
            stage: pb.BuildStage.BUILD_STAGE_COMPLETED,
            message: 'Firmware built',
            status: built(),
          ),
        ),
      );
      fw = done.state.editor.deploy.firmware;
      expect(fw.isBuilding, isFalse);
      expect(fw.artifact!.kind, 'uf2');
      expect(fw.artifactFresh, isTrue);
      expect(done.effects.whereType<ListFlashDevices>(), hasLength(1));
    });

    test('a refused request is shown in place and clears the running state', () {
      var s = reduce(onPico(analysis()), const BuildRequested()).state;
      s = reduce(
        s,
        const FirmwareRequestFailed(code: 'build.busy', message: 'A build is already running.'),
      ).state;
      expect(s.editor.deploy.firmware.isBuilding, isFalse);
      expect(s.editor.deploy.firmware.error, 'A build is already running.');
      // the next build clears it
      s = reduce(s, const BuildRequested()).state;
      expect(s.editor.deploy.firmware.error, isNull);
    });

    test('a flash takes the one board, waits for a choice between two, never guesses', () {
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(generation: s.editor.deploy.firmware.statusGeneration, status: built()),
      ).state;
      // none reachable: the request is still made (the daemon says no_device)
      s = reduce(s, const FlashDevicesReceived(targetId: pico, devices: [], methods: [])).state;
      var t = reduce(s, const FlashRequested());
      expect(t.effects.whereType<FlashFirmware>().single.deviceId, isNull);
      // one: flashed to it without a choice
      s = reduce(
        s,
        FlashDevicesReceived(targetId: pico, devices: [volume()], methods: [usb]),
      ).state;
      t = reduce(s, const FlashRequested());
      expect(t.effects.whereType<FlashFirmware>().single.deviceId, 'uf2:/Volumes/RPI-RP2');
      expect(t.state.editor.deploy.firmware.isFlashing, isTrue);
      // two: nothing is asked until one is chosen
      s = reduce(
        s,
        FlashDevicesReceived(
          targetId: pico,
          devices: [volume(), volume('/Volumes/RPI-RP2 1')],
          methods: [usb],
        ),
      ).state;
      t = reduce(s, const FlashRequested());
      expect(t.effects, isEmpty);
      expect(t.state.editor.deploy.firmware.isFlashing, isFalse);
      s = reduce(s, const FlashDeviceChosen('uf2:/Volumes/RPI-RP2 1')).state;
      t = reduce(s, const FlashRequested());
      expect(t.effects.whereType<FlashFirmware>().single.deviceId, 'uf2:/Volumes/RPI-RP2 1');
      // the chosen board unplugged: the choice is forgotten
      s = reduce(
        s,
        FlashDevicesReceived(targetId: pico, devices: [volume()], methods: [usb]),
      ).state;
      expect(s.editor.deploy.firmware.chosenDevice, isNull);
    });

    test('a flash is followed through its events and the board is looked for again', () {
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(generation: s.editor.deploy.firmware.statusGeneration, status: built()),
      ).state;
      s = reduce(
        s,
        FlashDevicesReceived(targetId: pico, devices: [volume()], methods: [usb]),
      ).state;
      s = reduce(s, const FlashRequested()).state;
      s = reduce(
        s,
        FlashProgressReceived(
          pb.FlashProgress(
            targetId: pico,
            stage: pb.FlashStage.FLASH_STAGE_WRITING,
            message: 'Writing',
            device: volume(),
          ),
        ),
      ).state;
      expect(s.editor.deploy.firmware.flashing!.stage, pb.FlashStage.FLASH_STAGE_WRITING);
      final t = reduce(
        s,
        FlashProgressReceived(
          pb.FlashProgress(
            targetId: pico,
            stage: pb.FlashStage.FLASH_STAGE_COMPLETED,
            message: 'The board restarted into the new firmware.',
            device: volume(),
            artifact: artifact(),
            at: Int64(1_700_000_100),
          ),
        ),
      );
      final fw = t.state.editor.deploy.firmware;
      expect(fw.isFlashing, isFalse);
      expect(fw.flashed!.stage, pb.FlashStage.FLASH_STAGE_COMPLETED);
      expect(t.effects.whereType<ListFlashDevices>(), hasLength(1));
      expect(firmwareStep(analysis(), fw), FirmwareStep.observe);
    });

    test('another board, or a closed project, forgets the firmware', () {
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(generation: s.editor.deploy.firmware.statusGeneration, status: built()),
      ).state;
      expect(s.editor.deploy.firmware.artifact, isNotNull);
      final other = reduce(s, const TargetSelected('arduino_nano')).state;
      expect(other.editor.deploy.firmware.artifact, isNull);
      final closed = reduce(s, const ProjectClosed()).state;
      expect(closed.editor.deploy.firmware.artifact, isNull);
      expect(closed.editor.deploy.targetId, pico);
    });

    test('the templates are asked once and kept', () {
      var s = const AppState();
      final t = reduce(s, const TemplatesRequested());
      expect(t.effects.whereType<ListTemplates>(), hasLength(1));
      s = reduce(
        t.state,
        TemplatesReceived([pb.TemplateView(id: 'button-lamp', displayName: 'Button → Lamp')]),
      ).state;
      expect(s.editor.deploy.templates.single.id, 'button-lamp');
      expect(reduce(s, const TemplatesRequested()).effects, isEmpty);
      // a demo is a new project with a template
      final pick = reduce(connected(null), const NewProjectPickRequested(template: 'button-lamp'));
      expect(pick.effects.whereType<PickNewProjectLocation>().single.template, 'button-lamp');
      final create = reduce(
        connected(null),
        const NewProjectRequested(rootPath: '/x/demo', name: 'demo', template: 'button-lamp'),
      );
      expect(create.effects.whereType<InitProject>().single.template, 'button-lamp');
    });
  });

  group('page', () {
    testWidgets('not ready: the first blocker is named and Build is not offered', (t) async {
      final s = onPico(
        analysis(
          ready: false,
          blockers: [
            pb.BuildBlocker(
              code: 'source_no_device',
              message: 'pressed has no device on Raspberry Pi Pico.',
              explanation: 'Add a device for it and choose a provider.',
              mappingId: Int64(0),
            ),
            pb.BuildBlocker(code: 'output_no_device', message: 'lamp has no device.'),
          ],
        ),
      );
      await t.pumpWidget(page(s, (_) {}));
      expect(find.byKey(const ValueKey('firmware-blocked')), findsOneWidget);
      expect(find.text('pressed has no device on Raspberry Pi Pico.'), findsOneWidget);
      expect(find.text('Add a device for it and choose a provider.'), findsOneWidget);
      expect(find.text('lamp has no device.'), findsNothing, reason: 'one blocker at a time');
      expect(find.byKey(const ValueKey('firmware-build')), findsNothing);
      expect(find.byKey(const ValueKey('firmware-step-deployment-current')), findsOneWidget);
      expect(find.text('Fix it on the Design page'), findsNothing);
    });

    testWidgets('a blocker about the design itself leads to the Design page', (t) async {
      final s = onPico(
        analysis(
          ready: false,
          blockers: [
            pb.BuildBlocker(
              code: 'relationship_not_checking',
              message: 'lit is not fully defined.',
            ),
          ],
        ),
      );
      final actions = <AppAction>[];
      await t.pumpWidget(page(s, actions.add));
      await t.tap(find.text('Fix it on the Design page'));
      expect(actions.whereType<PageSelected>().single.page, StudioPage.design);
    });

    testWidgets('ready: one primary Build; building: the stage, the count, Stop', (t) async {
      final actions = <AppAction>[];
      var s = onPico(analysis());
      await t.pumpWidget(page(s, actions.add));
      expect(find.byKey(const ValueKey('firmware-step-build-current')), findsOneWidget);
      expect(find.text('✓ Deployment'), findsOneWidget);
      expect(find.text('Ready to build for Raspberry Pi Pico.'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('firmware-build')));
      expect(actions.whereType<BuildRequested>(), hasLength(1));
      s = reduce(s, const BuildRequested()).state;
      s = reduce(
        s,
        BuildProgressReceived(
          pb.BuildProgress(
            targetId: pico,
            stage: pb.BuildStage.BUILD_STAGE_COMPILING,
            message: 'Compiling',
            done: 42,
          ),
        ),
      ).state;
      await t.pumpWidget(page(s, actions.add));
      expect(find.byKey(const ValueKey('firmware-running')), findsOneWidget);
      expect(find.text('Compiling'), findsOneWidget);
      expect(find.text('42 crates compiled'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-build')), findsNothing);
      await t.tap(find.text('Stop'));
      expect(actions.whereType<CancelBuildRequested>(), hasLength(1));
      await t.pump(const Duration(seconds: 1));
    });

    testWidgets('failed: the stage\'s words, Build again, the details open', (t) async {
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(
          generation: s.editor.deploy.firmware.statusGeneration,
          status: pb.BuildStatus(
            targetId: pico,
            stage: pb.BuildStage.BUILD_STAGE_FAILED,
            failure: pb.BuildFailure(
              stage: pb.BuildStage.BUILD_STAGE_PREPARING,
              code: 'build.target_missing',
              message: 'The Rust target for Raspberry Pi Pico is not installed.',
              explanation: 'Run `rustup target add thumbv6m-none-eabi` once, then build again.',
            ),
            generatedDir: '/demo/build/rp2040_pico',
            triple: 'thumbv6m-none-eabi',
          ),
        ),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(find.byKey(const ValueKey('firmware-failed')), findsOneWidget);
      expect(find.text('The Rust target for Raspberry Pi Pico is not installed.'), findsOneWidget);
      expect(find.textContaining('rustup target add'), findsOneWidget);
      expect(find.text('Build again'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-details-open')), findsOneWidget);
      expect(find.text('/demo/build/rp2040_pico'), findsOneWidget);
      expect(find.text('thumbv6m-none-eabi'), findsOneWidget);
    });

    testWidgets('built and fresh: Flash with one board, a choice with two, a hint with none', (
      t,
    ) async {
      final actions = <AppAction>[];
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(generation: s.editor.deploy.firmware.statusGeneration, status: built()),
      ).state;
      // none
      s = reduce(s, FlashDevicesReceived(targetId: pico, devices: [], methods: [usb])).state;
      await t.pumpWidget(page(s, actions.add));
      expect(find.byKey(const ValueKey('firmware-built')), findsOneWidget);
      expect(find.text('6144 bytes'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-no-device')), findsOneWidget);
      expect(find.textContaining('Hold BOOTSEL'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-flash')), findsNothing);
      expect(find.byKey(const ValueKey('firmware-step-flash-current')), findsOneWidget);
      await t.tap(find.text('Look again'));
      expect(actions.whereType<FlashDevicesRequested>(), hasLength(1));
      // one
      s = reduce(
        s,
        FlashDevicesReceived(targetId: pico, devices: [volume()], methods: [usb]),
      ).state;
      await t.pumpWidget(page(s, actions.add));
      expect(find.textContaining('Raspberry Pi Pico in BOOTSEL mode'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('firmware-flash')));
      expect(actions.whereType<FlashRequested>(), hasLength(1));
      // two
      s = reduce(
        s,
        FlashDevicesReceived(
          targetId: pico,
          devices: [volume(), volume('/Volumes/RPI-RP2 1')],
          methods: [usb],
        ),
      ).state;
      await t.pumpWidget(page(s, actions.add));
      expect(find.text('Several boards are reachable — choose one.'), findsOneWidget);
      expect(find.byKey(const ValueKey('flash-device-picker')), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-flash')), findsNothing, reason: 'no guess');
      s = reduce(s, const FlashDeviceChosen('uf2:/Volumes/RPI-RP2 1')).state;
      await t.pumpWidget(page(s, actions.add));
      expect(find.byKey(const ValueKey('firmware-flash')), findsOneWidget);
    });

    testWidgets('stale: named, Build again, never Flash; flashed: the trial to make', (t) async {
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(
          generation: s.editor.deploy.firmware.statusGeneration,
          status: built(fresh: false),
        ),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(find.byKey(const ValueKey('firmware-stale')), findsOneWidget);
      expect(find.text('Build again'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-flash')), findsNothing);
      expect(find.byKey(const ValueKey('firmware-step-build-current')), findsOneWidget);

      // flashed, then the design moved on: the board runs an earlier design
      s = reduce(
        s,
        BuildStatusReceived(generation: s.editor.deploy.firmware.statusGeneration, status: built()),
      ).state;
      s = reduce(
        s,
        FlashDevicesReceived(targetId: pico, devices: [volume()], methods: [usb]),
      ).state;
      s = reduce(s, const FlashRequested()).state;
      s = reduce(
        s,
        FlashProgressReceived(
          pb.FlashProgress(
            targetId: pico,
            stage: pb.FlashStage.FLASH_STAGE_COMPLETED,
            message: 'The board restarted into the new firmware.',
            device: volume(),
            artifact: artifact(),
            at: Int64(1_700_000_100),
          ),
        ),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(find.byKey(const ValueKey('firmware-flashed')), findsOneWidget);
      expect(find.text('Try it: act on pressed; lamp should follow the design.'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-step-observe-current')), findsOneWidget);
      expect(find.text('Flash again'), findsOneWidget);
      final moved = reduce(
        s,
        BuildStatusReceived(
          generation: s.editor.deploy.firmware.statusGeneration,
          status: built(fresh: false),
        ),
      ).state;
      await t.pumpWidget(page(moved, (_) {}));
      expect(find.byKey(const ValueKey('firmware-stale')), findsOneWidget);
      expect(find.textContaining('runs an earlier design'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-flash')), findsNothing);
    });

    testWidgets('a flash that failed says why, beside the fresh image', (t) async {
      var s = onPico(analysis());
      s = reduce(
        s,
        BuildStatusReceived(generation: s.editor.deploy.firmware.statusGeneration, status: built()),
      ).state;
      s = reduce(
        s,
        FlashDevicesReceived(targetId: pico, devices: [volume()], methods: [usb]),
      ).state;
      s = reduce(
        s,
        FlashProgressReceived(
          pb.FlashProgress(
            targetId: pico,
            stage: pb.FlashStage.FLASH_STAGE_FAILED,
            message: 'The board is no longer in bootloader mode.',
            device: volume(),
            failure: pb.FlashFailure(
              code: 'flash.device_gone',
              message: 'The board is no longer in bootloader mode.',
              explanation: 'Hold BOOTSEL while plugging it in again, then flash.',
            ),
          ),
        ),
      ).state;
      await t.pumpWidget(page(s, (_) {}));
      expect(find.byKey(const ValueKey('firmware-flash-failed')), findsOneWidget);
      expect(find.textContaining('plugging it in again'), findsOneWidget);
      expect(find.byKey(const ValueKey('firmware-flash')), findsOneWidget);
    });

    testWidgets('the Welcome page lists the demos as new projects from a template', (t) async {
      final actions = <AppAction>[];
      var s = connected(null);
      s = reduce(
        s,
        TemplatesReceived([
          pb.TemplateView(
            id: 'button-lamp',
            displayName: 'Button → Lamp',
            description: 'A push button and a lamp.',
            targetId: pico,
          ),
          pb.TemplateView(
            id: 'button-lamp-configured',
            displayName: 'Button → Lamp, wired',
            targetId: pico,
            configured: true,
          ),
        ]),
      ).state;
      await t.pumpWidget(welcome(s, actions.add));
      expect(find.text('Demos'), findsOneWidget);
      expect(find.text('Button → Lamp'), findsOneWidget);
      expect(find.text('Button → Lamp, wired'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('template-button-lamp')));
      expect(actions.whereType<NewProjectPickRequested>().single.template, 'button-lamp');
    });
  });

  test('the step follows the daemon\'s facts', () {
    expect(firmwareStep(null, const FirmwareState()), FirmwareStep.deployment);
    expect(firmwareStep(analysis(ready: false), const FirmwareState()), FirmwareStep.deployment);
    expect(firmwareStep(analysis(), const FirmwareState()), FirmwareStep.build);
    expect(firmwareStep(analysis(), FirmwareState(status: built())), FirmwareStep.flash);
    expect(
      firmwareStep(analysis(), FirmwareState(status: built(fresh: false))),
      FirmwareStep.build,
    );
    final tokens = MacTokens.light;
    expect(tokens.settled, isNot(tokens.open));
  });
}
