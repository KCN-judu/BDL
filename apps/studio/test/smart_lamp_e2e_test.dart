/// The demonstration project, examples/smart_lamp, through the whole Studio
/// stack against the real bdld: Design (the ladder and the output pass),
/// Simulate (tilt and ambient light → brightness), Deploy (the PWM light
/// placed on the Nano) — one project, one set of identities, three views.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
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

void main() {
  final bdld = _findBdld();
  final example = p.normalize(p.join(Directory.current.path, '../../examples/smart_lamp'));

  test('Smart Lamp: design → simulate → deploy with one set of identities', () async {
    final store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    try {
      store.dispatch(const AppStarted());
      await store.until((s) => s.connection is Connected);
      store.dispatch(OpenProjectRequested(example));
      var s = await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
      s = await store.until((s) => s.analysis?.revision.toInt() == s.revision);
      int mappingId(String n) => s.project!.mappings.firstWhere((m) => m.name == n).id.toInt();
      int conceptId(String n) => s.project!.concepts.firstWhere((c) => c.name == n).id.toInt();

      // Design: every defined relationship clock-consistent, the sink driven
      for (final m in s.project!.mappings) {
        final a = s.mappingAnalysis(m.id.toInt())!;
        expect(
          a.status,
          m.hasDefinition()
              ? pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT
              : pb.MappingStatus.MAPPING_STATUS_DECLARED,
          reason: '${m.name}: ${a.diagnostics}',
        );
      }
      expect(s.analysis!.outputComplete, isTrue);
      final light = s.project!.outputs.single;
      expect(s.outputAnalysis(light.id.toInt())!.driver.toInt(), mappingId('brightness'));
      expect(s.editor.drafts, isEmpty, reason: 'no phantom draft on open');

      // Simulate: the brightness identity is the same mapping the Design page names
      store.dispatch(const PageSelected(StudioPage.simulate));
      pb.Value q(String concept, pb.Dim dim, double v) => pb.Value(
        semantic: pb.SemanticValue(
          conceptId: Int64(conceptId(concept)),
          repr: pb.Value(
            quantity: pb.Quantity(dim: dim, value: v),
          ),
        ),
      );
      final tiltDim = s.project!.concepts
          .firstWhere((c) => c.name == 'Tilt')
          .representation
          .quantity;
      final luxDim = s.project!.concepts
          .firstWhere((c) => c.name == 'AmbientLight')
          .representation
          .quantity;
      store.dispatch(
        SimulationInputChanged(
          mappingId: mappingId('tilt'),
          value: q('Tilt', tiltDim, 0.7853981633974483),
        ),
      );
      store.dispatch(
        SimulationInputChanged(
          mappingId: mappingId('ambient'),
          value: q('AmbientLight', luxDim, 50),
        ),
      );
      store.dispatch(const SimulationStepRequested(1));
      s = await store.until((s) => !s.editor.simulation.pending);
      store.dispatch(
        SimulationInputChanged(
          mappingId: mappingId('ambient'),
          value: q('AmbientLight', luxDim, 800),
        ),
      );
      store.dispatch(const SimulationStepRequested(1));
      s = await store.until(
        (s) => !s.editor.simulation.pending && s.editor.simulation.nextTick == 2,
      );
      expect(s.editor.simulation.error, isNull, reason: '${s.editor.simulation.error}');
      String brightnessAt(int t) => s.editor.simulation.samples[t].values
          .firstWhere((v) => v.mappingId.toInt() == mappingId('brightness'))
          .rendered;
      expect(brightnessAt(0), 'Brightness(0.5)', reason: '45° in a dim room');
      expect(brightnessAt(1), 'Brightness(0.25)', reason: 'the same tilt in a bright room');

      // Deploy: the PWM light placed on the Nano; the two Sources (tilt,
      // ambient) have no device providing them, so the deployment is
      // incomplete and names them
      store.dispatch(const PageSelected(StudioPage.deploy));
      await store.until((s) => s.editor.deploy.targetsLoaded);
      store.dispatch(const TargetSelected('arduino_nano'));
      s = await store.until((s) => s.editor.deploy.analysis != null);
      final d = s.editor.deploy.analysis!;
      expect(d.status, pb.DeploymentStatus.DEPLOYMENT_STATUS_INCOMPLETE);
      expect(d.provisions.map((p) => p.status).toSet(), {
        pb.ProvisionStatus.PROVISION_STATUS_NO_DEVICE,
      });
      expect(d.provisions.map((p) => p.sourceName).toSet(), {'tilt', 'ambient'});
      expect(d.assignment.single.deviceId.toInt(), s.project!.devices.single.id.toInt());
      expect(d.assignment.single.resource, isNotEmpty);
      expect(s.project!.dirty, isFalse, reason: 'nothing here edits the example');
    } finally {
      await store.dispose();
    }
  }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);
}
