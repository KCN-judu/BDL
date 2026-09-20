/// The demo's whole path through Studio and the real `bdld`: a project
/// from the wired template, the Deploy page ready to build, the build
/// followed through its events to a UF2, the one pretend board flashed,
/// the trial named, then a pin moved and the image reported stale.  The
/// compiler behind the daemon is a stand-in `cargo` (the real one is
/// proved in the daemon's own tests); the "board" is a directory with an
/// `INFO_UF2.TXT`.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/pages/firmware_section.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/test_store.dart';

String? _findBdld() {
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && File(env).existsSync()) return env;
  for (final rel in ['../../target/debug/bdld', '../../target/release/bdld']) {
    final f = p.normalize(p.join(Directory.current.path, rel));
    if (File(f).existsSync()) return f;
  }
  return null;
}

String _repoRoot() => p.normalize(p.join(Directory.current.path, '../..'));

/// A minimal ELF32 whose one loadable segment lies in the Pico's flash.
List<int> _tinyElf() {
  final segment = List<int>.filled(300, 0xAA);
  final header = List<int>.filled(52, 0);
  header.setRange(0, 4, [0x7f, 0x45, 0x4c, 0x46]);
  header[4] = 1;
  header[5] = 1;
  header[6] = 1;
  header[16] = 2;
  header[18] = 40;
  header[28] = 52;
  header[42] = 32;
  header[44] = 1;
  final ph = List<int>.filled(32, 0);
  void u32(List<int> b, int at, int v) {
    b[at] = v & 0xff;
    b[at + 1] = (v >> 8) & 0xff;
    b[at + 2] = (v >> 16) & 0xff;
    b[at + 3] = (v >> 24) & 0xff;
  }

  u32(ph, 0, 1);
  u32(ph, 4, 52 + 32);
  u32(ph, 8, 0x10000000);
  u32(ph, 12, 0x10000000);
  u32(ph, 16, segment.length);
  u32(ph, 20, segment.length);
  return [...header, ...ph, ...segment];
}

/// `bdld` wrapped so the daemon sees a stand-in cargo, the runtime crates,
/// a pretend volume root and a short restart wait.
Future<String> _wrappedBdld(String bdld, Directory dir) async {
  final elf = File(p.join(dir.path, 'firmware.elf'));
  await elf.writeAsBytes(_tinyElf());
  final cargo = File(p.join(dir.path, 'cargo'));
  await cargo.writeAsString(
    '#!/bin/sh\n'
    'if [ "\$1" = "--version" ]; then echo cargo 1.89.0; exit 0; fi\n'
    'bin=""; prev=""; for a in "\$@"; do if [ "\$prev" = "--bin" ]; then bin="\$a"; fi; prev="\$a"; done\n'
    'printf \'%s\\n\' \'{"reason":"compiler-artifact","target":{"name":"embassy_rp"},"executable":null}\'\n'
    'printf \'%s\\n\' "{\\"reason\\":\\"compiler-artifact\\",\\"target\\":{\\"name\\":\\"\$bin\\"},\\"executable\\":\\"${elf.path}\\"}"\n'
    'printf \'%s\\n\' \'{"reason":"build-finished","success":true}\'\n'
    'exit 0\n',
  );
  await Process.run('chmod', ['755', cargo.path]);
  final vols = Directory(p.join(dir.path, 'vols'));
  await vols.create();
  final wrapper = File(p.join(dir.path, 'bdld-wrapped'));
  await wrapper.writeAsString(
    '#!/bin/sh\n'
    'export CARGO="${cargo.path}"\n'
    'export BDL_RUNTIME_DIR="${p.join(_repoRoot(), 'runtime')}"\n'
    'export BDL_UF2_ROOTS="${vols.path}"\n'
    'export BDL_FLASH_RESTART_WAIT_MS=200\n'
    'export BDL_PROBE_RS=/nonexistent/probe-rs\n'
    'exec "$bdld" "\$@"\n',
  );
  await Process.run('chmod', ['755', wrapper.path]);
  return wrapper.path;
}

void main() {
  final bdld = _findBdld();
  test(
    'from the wired demo to a flashed board, and stale after a pin moves',
    () async {
      final dir = await Directory.systemTemp.createTemp('bdl-studio-firmware-e2e');
      final store = TestStore(
        spawn: DaemonClient.spawn,
        executable: await _wrappedBdld(bdld!, dir),
      );
      try {
        store.dispatch(const AppStarted());
        var s = await store.until((s) => s.connection is Connected);
        s = await store.until((s) => s.editor.deploy.templatesLoaded);
        expect(s.editor.deploy.templates.map((t) => t.id), [
          'button-lamp',
          'button-lamp-configured',
        ]);

        // The demo is a new project from a template.
        store.dispatch(
          NewProjectRequested(
            rootPath: p.join(dir.path, 'demo'),
            name: 'demo',
            template: 'button-lamp-configured',
          ),
        );
        s = await store.until((s) => s.project != null);
        expect(s.flat!.devices.map((d) => d.name), containsAll(['button', 'led']));
        expect(File(p.join(dir.path, 'demo/src/main.bdl')).existsSync(), isTrue);

        // Deploy: the Pico chosen, ready to build, at the Build step.
        store.dispatch(const PageSelected(StudioPage.deploy));
        await store.until((s) => s.editor.deploy.targetsLoaded);
        store.dispatch(const TargetSelected('rp2040_pico'));
        s = await store.until(
          (s) =>
              s.editor.deploy.analysis != null &&
              s.editor.deploy.firmware.status != null &&
              s.editor.deploy.firmware.status!.targetId == 'rp2040_pico',
        );
        final a = s.editor.deploy.analysis!;
        expect(a.status, pb.DeploymentStatus.DEPLOYMENT_STATUS_FEASIBLE);
        expect(a.buildReady, isTrue, reason: '${a.buildBlockers}');
        expect(s.editor.deploy.firmware.artifact, isNull);
        expect(firmwareStep(a, s.editor.deploy.firmware), FirmwareStep.build);

        // Build: the stages arrive as events; the image is a UF2 under the
        // project; fresh.
        store.dispatch(const BuildRequested());
        s = await store.until((s) => s.editor.deploy.firmware.isBuilding);
        s = await store.until(
          (s) => !s.editor.deploy.firmware.isBuilding && s.editor.deploy.firmware.artifact != null,
          timeout: const Duration(seconds: 60),
        );
        var fw = s.editor.deploy.firmware;
        expect(fw.status!.stage, pb.BuildStage.BUILD_STAGE_COMPLETED);
        expect(fw.artifact!.kind, 'uf2');
        expect(fw.artifact!.path, startsWith(p.join(dir.path, 'demo', 'build', 'rp2040_pico')));
        expect(File(fw.artifact!.path).existsSync(), isTrue);
        expect(fw.artifactFresh, isTrue);
        expect(fw.status!.command, startsWith('cargo build --release'));
        expect(fw.status!.triple, 'thumbv6m-none-eabi');
        final progress = store.actions.whereType<BuildProgressReceived>().map(
          (a) => a.progress.stage,
        );
        expect(
          progress,
          containsAllInOrder([
            pb.BuildStage.BUILD_STAGE_CHECKING,
            pb.BuildStage.BUILD_STAGE_COMPILING,
            pb.BuildStage.BUILD_STAGE_COMPLETED,
          ]),
        );
        // The board is looked for: none yet.
        s = await store.until((s) => s.editor.deploy.firmware.devicesLoaded);
        expect(s.editor.deploy.firmware.devices, isEmpty);
        expect(
          s.editor.deploy.firmware.methods
              .where((m) => m.method == pb.FlashMethod.FLASH_METHOD_UF2_VOLUME)
              .single
              .available,
          isTrue,
        );
        expect(
          firmwareStep(s.editor.deploy.analysis, s.editor.deploy.firmware),
          FirmwareStep.flash,
        );

        // A flash with nothing plugged in: refused by the daemon, shown in place.
        store.dispatch(const FlashRequested());
        s = await store.until((s) => s.editor.deploy.firmware.error != null);
        expect(s.editor.deploy.firmware.error, contains('BOOTSEL'));

        // The board appears; Look again finds it; the flash writes the image.
        final volume = Directory(p.join(dir.path, 'vols', 'RPI-RP2'));
        await volume.create();
        await File(p.join(volume.path, 'INFO_UF2.TXT'))
            .writeAsString('UF2 Bootloader v3.0\nModel: Raspberry Pi RP2\nBoard-ID: RPI-RP2\n');
        store.dispatch(const FlashDevicesRequested());
        s = await store.until((s) => s.editor.deploy.firmware.devices.length == 1);
        store.dispatch(const FlashRequested());
        s = await store.until(
          (s) => s.editor.deploy.firmware.flashed != null,
          timeout: const Duration(seconds: 30),
        );
        fw = s.editor.deploy.firmware;
        expect(fw.flashed!.stage, pb.FlashStage.FLASH_STAGE_COMPLETED);
        expect(fw.flashed!.device.label, contains('BOOTSEL'));
        final written = volume
            .listSync()
            .map((e) => p.basename(e.path))
            .where((n) => n.endsWith('.uf2'));
        expect(written, hasLength(1));
        expect(firmwareStep(s.editor.deploy.analysis, fw), FirmwareStep.observe);

        // The button moves to another pad: the analysis is asked again, the
        // status too, and the image is stale — Build again, never Flash.
        final button = s.flat!.devices.firstWhere((d) => d.name == 'button').id.toInt();
        final before = s.revision;
        store.dispatch(SetDevicePinRequested(id: button, index: 0, resource: 'GP3'));
        s = await store.until(
          (s) =>
              s.revision > before &&
              s.editor.deploy.firmware.status != null &&
              !s.editor.deploy.firmware.artifactFresh,
        );
        expect(
          s.editor.deploy.firmware.artifact,
          isNotNull,
          reason: 'the old image is still named',
        );
        expect(
          firmwareStep(s.editor.deploy.analysis, s.editor.deploy.firmware),
          FirmwareStep.build,
        );
        expect(s.editor.deploy.analysis!.buildReady, isTrue);
      } finally {
        await store.dispose();
        await dir.delete(recursive: true);
      }
    },
    skip: bdld == null
        ? 'bdld not built'
        : Platform.isWindows
        ? 'the stand-in tools are sh scripts'
        : false,
  );
}
