/// The Standard Concept Library end to end: Studio's reducer and executor
/// against the real `bdld` (skipped when it is not built).  The library
/// Studio shows is the daemon's; inserting a template twice yields two
/// concepts; renaming one leaves the other's defaults alone; the project
/// saves and reopens without the library.
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
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

  test('templates come from bdld; two insertions are two concepts', () async {
    final store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    final dir = await Directory.systemTemp.createTemp('bdl-studio-library');
    try {
      store.dispatch(const AppStarted());
      await store.until((s) => s.library != null || s.connection is ConnectionFailed);
      expect(store.state.connection, isA<Connected>(), reason: '${store.state.connection}');
      final temperature = store.state.template('std.environment.temperature');
      expect(temperature, isNotNull);
      expect(temperature!.unit, 'K');
      expect(temperature.typeName, 'Temperature');
      expect(store.state.templates.length, greaterThanOrEqualTo(30));
      expect(store.state.library!.quantities.any((q) => q.typeName == 'Illuminance'), isTrue);

      final root = p.join(dir.path, 'lamp');
      store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp'));
      await store.until((s) => s.project != null && s.editor.pendingRequests == 0);

      // Right-click insertion, then a drag insertion of the same template.
      store.dispatch(
        const InsertConceptTemplateRequested(
          'std.environment.temperature',
          position: Offset(40, 40),
        ),
      );
      await store.until((s) => s.editor.pendingRequests == 0 && s.project!.concepts.length == 1);
      final first = store.state.project!.concepts.single;
      expect(store.state.editor.renaming, NodeRef.concept(first.id.toInt()));
      store.dispatch(
        InlineRenameFinished(NodeRef.concept(first.id.toInt()), name: 'RoomTemperature'),
      );
      await store.until((s) => s.editor.pendingRequests == 0);
      store.dispatch(
        const InsertConceptTemplateRequested(
          'std.environment.temperature',
          position: Offset(40, 200),
        ),
      );
      await store.until((s) => s.editor.pendingRequests == 0 && s.project!.concepts.length == 2);
      final second = store.state.project!.concepts.firstWhere((c) => c.id != first.id);
      expect(second.name, 'Temperature', reason: 'the default name is free again');
      store.dispatch(
        InlineRenameFinished(NodeRef.concept(second.id.toInt()), name: 'MotorTemperature'),
      );
      await store.until((s) => s.editor.pendingRequests == 0);

      final concepts = store.state.project!.concepts;
      expect(concepts.map((c) => c.name), containsAll(['RoomTemperature', 'MotorTemperature']));
      expect(concepts[0].id, isNot(concepts[1].id));
      for (final c in concepts) {
        expect(c.representation.quantity, pb.Dim(temperature: 1));
        expect(c.description, temperature.description);
      }
      expect(store.state.editor.layout[NodeRef.concept(second.id.toInt())], const Offset(40, 200));

      // Saved and reopened without any library lookup: names, identities,
      // representations all come from the project.
      store.dispatch(const SaveRequested());
      await store.until((s) => s.editor.pendingRequests == 0 && !s.project!.dirty);
      store.dispatch(const CloseProjectRequested());
      await store.until((s) => s.project == null);
      store.dispatch(OpenProjectRequested(root));
      await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
      final reopened = store.state.project!.concepts;
      expect(reopened.map((c) => c.id), containsAll(concepts.map((c) => c.id)));
      expect(reopened.map((c) => c.name), containsAll(['RoomTemperature', 'MotorTemperature']));
      for (final c in reopened) {
        expect(c.representation.quantity, pb.Dim(temperature: 1));
      }
    } finally {
      await store.dispose();
      await dir.delete(recursive: true);
    }
  }, skip: bdld == null ? 'bdld is not built' : false);
}
