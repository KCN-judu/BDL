/// Spawns the real `bdld` (built by `cargo build`) and drives the handshake and
/// the first edits through the Dart client.  Skipped when the binary is absent.
@Tags(['daemon', 'filesystem'])
library;

import 'dart:io';

import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/protocol/versions.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

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

  test('handshake, create concepts, unresolved mapping, save, reopen', () async {
    final client = await DaemonClient.spawn(bdld!);
    final dir = await Directory.systemTemp.createTemp('bdl-studio-test');
    final root = p.join(dir.path, 'lamp');
    try {
      final hs = await client.request(
        pb.ClientMessage(
          handshake: pb.HandshakeRequest(
            clientProtocolVersion: kClientProtocolVersion,
            clientName: 'test',
            clientVersion: '0',
          ),
        ),
      );
      expect(hs.handshake.compatible, isTrue);
      expect(hs.handshake.protocolVersion.major, kClientProtocolVersion.major);

      var r = await client.request(
        pb.ClientMessage(
          initProject: pb.InitProjectRequest(rootPath: root, name: 'lamp'),
        ),
      );
      expect(r.project.project.revision.toInt(), 0);

      Future<pb.ProjectProjection> edit(int base, pb.EditOp op) async {
        final r = await client.request(
          pb.ClientMessage(
            applyEdit: pb.ApplyEditRequest(baseRevision: Int64(base), op: op),
          ),
        );
        return r.editApplied.project;
      }

      var proj = await edit(0, pb.EditOp(createConcept: pb.CreateConcept(name: 'Tilt')));
      final tilt = proj.concepts.single.id;
      proj = await edit(1, pb.EditOp(createConcept: pb.CreateConcept(name: 'Brightness')));
      final bright = proj.concepts.firstWhere((c) => c.name == 'Brightness').id;
      proj = await edit(
        2,
        pb.EditOp(
          createMapping: pb.CreateMapping(
            name: 'dimByTilt',
            signature: pb.Signature(inputs: [tilt], output: bright),
          ),
        ),
      );
      expect(proj.mappings.single.state, pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED);
      expect(proj.mappings.single.hasDefinition(), isFalse);

      // A typed error, not an exception from the transport.
      await expectLater(
        edit(3, pb.EditOp(createConcept: pb.CreateConcept(name: 'Tilt'))),
        throwsA(isA<DaemonError>().having((e) => e.code, 'code', 'edit.duplicate_concept_name')),
      );

      await client.request(pb.ClientMessage(saveProject: pb.SaveProjectRequest()));
      await client.request(pb.ClientMessage(closeProject: pb.CloseProjectRequest()));
      r = await client.request(
        pb.ClientMessage(openProject: pb.OpenProjectRequest(rootPath: root)),
      );
      expect(r.project.project.mappings.single.name, 'dimByTilt');
      expect(r.project.project.mappings.single.state, pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED);
    } finally {
      await client.shutdown();
      expect(await client.exitCode, 0);
      await dir.delete(recursive: true);
    }
  }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);
}
