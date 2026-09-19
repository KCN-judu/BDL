/// The Standard Library end to end: Studio's reducer and executor
/// against the real `bdld` (skipped when it is not built).  The library
/// Studio shows is the daemon's; inserting a template twice yields two
/// concepts; renaming one leaves the other's defaults alone; the project
/// saves and reopens without the library.  A Source item is one transaction
/// of two ordinary edits, written as `mapping S : () -> C`.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/simulation.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/ui/library_panel.dart' show itemName;
import 'package:flutter/widgets.dart' show Locale;
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
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
        const InsertLibraryItemRequested('std.environment.temperature', position: Offset(40, 40)),
      );
      await store.until((s) => s.editor.pendingRequests == 0 && s.project!.concepts.length == 1);
      final first = store.state.project!.concepts.single;
      expect(store.state.editor.renaming, NodeRef.concept(first.id.toInt()));
      store.dispatch(
        InlineRenameFinished(NodeRef.concept(first.id.toInt()), name: 'RoomTemperature'),
      );
      await store.until((s) => s.editor.pendingRequests == 0);
      store.dispatch(
        const InsertLibraryItemRequested('std.environment.temperature', position: Offset(40, 200)),
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

  test('a Source item is one transaction: a concept and its () -> relationship, as text', () async {
    final store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    final dir = await Directory.systemTemp.createTemp('bdl-studio-sources');
    try {
      store.dispatch(const AppStarted());
      await store.until((s) => s.library != null || s.connection is ConnectionFailed);
      expect(store.state.connection, isA<Connected>(), reason: '${store.state.connection}');
      // the library serves eight Sources, each creating a concept and its
      // `() -> Value` relationship; the concept templates hold none of it
      final sources = store.state.libraryItems.where((i) => i.category == 'source').toList();
      expect(sources.map((i) => i.id), [
        'std.source.temperature',
        'std.source.tilt',
        'std.source.distance',
        'std.source.ambient_light',
        'std.source.button',
        'std.source.encoder',
        'std.source.analog',
        'std.source.external',
      ]);
      for (final i in sources) {
        expect(i.creates.length, 2, reason: i.id);
        expect(i.creates.last.signature, startsWith('() -> '), reason: i.id);
      }
      expect(store.state.templates.any((t) => t.id.startsWith('std.source.')), isFalse);
      final temp = store.state.libraryItem('std.source.temperature')!;
      expect(temp.creates.last.name, 'TempSensor');
      expect(itemName(kEnglish, temp), 'Temperature Sensor');
      expect(itemName(lookupAppLocalizations(const Locale('zh')), temp), '温度传感器');
      expect(itemName(lookupAppLocalizations(const Locale('ja')), temp), '温度センサー');
      // Analog Input and External Value leave the value form to the designer
      for (final id in ['std.source.analog', 'std.source.external']) {
        expect(store.state.libraryItem(id)!.creates.first.hasRepresentation(), isFalse, reason: id);
      }

      final root = p.join(dir.path, 'lamp');
      store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp'));
      await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
      final before = store.state.revision;
      store.dispatch(
        const InsertLibraryItemRequested('std.source.temperature', position: Offset(400, 40)),
      );
      var s = await store.until(
        (x) => x.editor.pendingRequests == 0 && x.project!.mappings.isNotEmpty,
      );
      final concept = s.project!.concepts.single;
      final source = s.project!.mappings.single;
      expect(concept.name, 'RoomTemp');
      expect(source.name, 'TempSensor');
      expect(source.signature.inputs, isEmpty);
      expect(source.signature.output, concept.id);
      expect(source.hasDefinition(), isFalse);
      expect(relationshipRole(source), RelationshipRole.source);
      expect(s.editor.renaming, NodeRef.concept(concept.id.toInt()));
      expect(s.editor.layout[NodeRef.mapping(source.id.toInt())], const Offset(160, 40));
      final scene = buildScene(s.project!, s.editor.layout);
      final node = scene.nodes.firstWhere((n) => n.ref == NodeRef.mapping(source.id.toInt()));
      expect(node.source, isTrue);
      expect(node.sockets.where((x) => x.ref.side == SocketSide.input), isEmpty);
      expect(simulationInputs(s.project!).map((m) => m.id), [source.id]);

      // one history entry: undo removes both, redo restores both
      store.dispatch(const UndoRequested());
      s = await store.until((x) => x.editor.pendingRequests == 0 && x.revision != before + 1);
      expect(s.project!.concepts, isEmpty);
      expect(s.project!.mappings, isEmpty);
      store.dispatch(const RedoRequested());
      s = await store.until((x) => x.editor.pendingRequests == 0 && x.project!.mappings.isNotEmpty);
      expect(s.project!.mappings.single.name, 'TempSensor');

      // the text is the preferred spelling — never the shorthand
      store.dispatch(const DesignViewChanged(DesignView.code));
      s = await store.until((x) => x.editor.sources.revision == x.revision);
      expect(s.editor.sources.text, contains('mapping TempSensor : () -> RoomTemp'));
      expect(s.editor.sources.text, isNot(contains('mapping TempSensor : RoomTemp\n')));

      // saved and reopened: the same shape, the role re-derived from it
      store.dispatch(const SaveRequested());
      await store.until((x) => x.editor.pendingRequests == 0 && !x.project!.dirty);
      final sidecar = File(p.join(root, '.bdl', 'authoring.json')).readAsStringSync();
      expect(sidecar.toLowerCase(), isNot(contains('source')));
      store.dispatch(const CloseProjectRequested());
      await store.until((x) => x.project == null);
      store.dispatch(OpenProjectRequested(root));
      s = await store.until((x) => x.project != null && x.editor.pendingRequests == 0);
      final reopened = s.project!.mappings.single;
      expect(reopened.id, source.id);
      expect(relationshipRole(reopened), RelationshipRole.source);
    } finally {
      await store.dispose();
      await dir.delete(recursive: true);
    }
  }, skip: bdld == null ? 'bdld is not built' : false);
}
