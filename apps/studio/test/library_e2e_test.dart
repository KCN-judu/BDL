/// The Standard Library end to end: Studio's reducer and executor
/// against the real `bdld` (skipped when it is not built).  The library
/// Studio shows is the daemon's value categories with the compiler's units
/// (ADR-0041); a concept is created from a category with the name the
/// designer gives, twice for two concepts; the legacy item path still
/// works; the project saves and reopens without the library.  A Source is
/// created over a chosen concept, new or existing, written as
/// `mapping S : () -> C`.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/simulation.dart';
import 'package:bdl_studio/app/state.dart';
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
      // the categories: the value forms, then one per named quantity,
      // each with the compiler's units for its dimension
      final temperature = store.state.template('std.quantity.temperature');
      expect(temperature, isNotNull);
      expect(temperature!.unit, 'K');
      expect(temperature.typeName, 'Temperature');
      expect(store.state.templates.length, inInclusiveRange(15, 30));
      expect(store.state.templates.any((t) => t.displayName == 'Motor Angle'), isFalse);
      expect(store.state.libraryItems.where((i) => i.category == 'source'), isEmpty);
      expect(store.state.library!.quantities.any((q) => q.typeName == 'Illuminance'), isTrue);
      // the value categories arrive with the library: the units each is
      // measured in are the compiler's, composites included
      await store.until((s) => s.valueCategories.isNotEmpty);
      final angle = store.state.valueCategories.firstWhere((c) => c.typeName == 'Angle');
      expect(angle.units.map((u) => u.display), containsAll(['rad', 'deg', 'turn']));
      final omega = store.state.valueCategories.firstWhere((c) => c.typeName == 'AngularVelocity');
      expect(omega.preferredUnit.display, 'rad/s');
      expect(omega.units.map((u) => u.display), contains('deg/s'));

      final root = p.join(dir.path, 'lamp');
      store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp'));
      await store.until((s) => s.project != null && s.editor.pendingRequests == 0);

      // The concept sheet: a category and a name, then one ordinary edit;
      // the concept lands where the sheet was asked for, selected, named.
      store.dispatch(
        const NewConceptRequested(presetId: 'std.quantity.temperature', position: Offset(40, 40)),
      );
      expect(store.state.editor.conceptSheet?.presetId, 'std.quantity.temperature');
      final fresh = store.state.revision;
      expect(store.state.project!.concepts, isEmpty, reason: 'nothing is created by the sheet');
      store.dispatch(
        CreateConceptRequested(
          name: 'RoomTemperature',
          representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
          position: const Offset(40, 40),
          presetId: 'std.quantity.temperature',
        ),
      );
      await store.until((s) => s.editor.pendingRequests == 0 && s.project!.concepts.length == 1);
      final first = store.state.project!.concepts.single;
      expect(first.name, 'RoomTemperature');
      expect(store.state.revision, fresh + 1, reason: 'one ordinary edit');
      expect(store.state.editor.conceptSheet, isNull);
      expect(store.state.editor.renaming, isNull, reason: 'named on the sheet');
      expect(store.state.editor.selection, ConceptSelected(first.id.toInt()));
      expect(store.state.editor.layout[NodeRef.concept(first.id.toInt())], const Offset(40, 40));
      expect(store.state.editor.recentTemplates, ['std.quantity.temperature']);
      // the legacy item path (an older client, a third-party library):
      // the category's default name, then create-then-rename
      store.dispatch(
        const InsertLibraryItemRequested('std.quantity.temperature', position: Offset(40, 200)),
      );
      await store.until((s) => s.editor.pendingRequests == 0 && s.project!.concepts.length == 2);
      final second = store.state.project!.concepts.firstWhere((c) => c.id != first.id);
      expect(second.name, 'Temperature', reason: 'the category\'s default name');
      expect(store.state.editor.renaming, NodeRef.concept(second.id.toInt()));
      store.dispatch(
        InlineRenameFinished(NodeRef.concept(second.id.toInt()), name: 'MotorTemperature'),
      );
      await store.until((s) => s.editor.pendingRequests == 0);

      final concepts = store.state.project!.concepts;
      expect(concepts.map((c) => c.name), containsAll(['RoomTemperature', 'MotorTemperature']));
      expect(concepts[0].id, isNot(concepts[1].id));
      for (final c in concepts) {
        expect(c.representation.quantity, pb.Dim(temperature: 1));
      }
      expect(second.description, temperature.description);
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

  test('a Source is created over a chosen concept: existing, or new in one transaction', () async {
    final store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    final dir = await Directory.systemTemp.createTemp('bdl-studio-sources');
    try {
      store.dispatch(const AppStarted());
      await store.until((s) => s.library != null || s.connection is ConnectionFailed);
      expect(store.state.connection, isA<Connected>(), reason: '${store.state.connection}');
      // the standard library ships no Source preset: a Source is created
      // on the sheet over a concept the designer chooses
      expect(store.state.libraryItems.where((i) => i.category == 'source'), isEmpty);
      expect(store.state.templates.any((t) => t.id.startsWith('std.source.')), isFalse);

      final root = p.join(dir.path, 'lamp');
      store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp'));
      await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
      // two temperatures of one value form: two identities, two choices
      Future<int> concept(String name) async {
        store.dispatch(
          CreateConceptRequested(
            name: name,
            representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
          ),
        );
        await store.until((s) => s.editor.pendingRequests == 0);
        return store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
      }

      final room = await concept('RoomTemperature');
      final motor = await concept('MotorTemperature');
      final base = store.state.revision;

      // the sheet opens with both concepts as candidates, nothing hidden;
      // cancelling changes nothing
      store.dispatch(const NewSourceRequested(position: Offset(400, 40)));
      var s = await store.until((x) => x.editor.sourceSheet?.ready ?? false);
      expect(
        s.editor.sourceSheet!.candidates!.candidates.map((c) => (c.conceptId.toInt(), c.preferred)),
        [(room, false), (motor, false)],
      );
      expect(s.editor.sourceSheet!.candidates!.hasPreset(), isFalse);
      expect(s.revision, base);
      store.dispatch(const SourceSheetDismissed());
      expect(store.state.revision, base);
      expect(store.state.project!.mappings, isEmpty);

      // an existing concept: only the Source, at the drop point, selected
      store.dispatch(const NewSourceRequested(position: Offset(400, 40)));
      await store.until((x) => x.editor.sourceSheet?.ready ?? false);
      store.dispatch(
        CreateSourceRequested(sourceName: 'roomTemperatureInput', existingConcept: room),
      );
      s = await store.until((x) => x.editor.pendingRequests == 0 && x.project!.mappings.isNotEmpty);
      expect(s.revision, base + 1);
      expect(s.project!.concepts, hasLength(2), reason: 'no new concept');
      final source = s.project!.mappings.single;
      expect(source.name, 'roomTemperatureInput');
      expect(source.signature.inputs, isEmpty);
      expect(source.signature.output.toInt(), room, reason: 'the chosen identity');
      expect(source.hasDefinition(), isFalse);
      expect(relationshipRole(source), RelationshipRole.source);
      expect(s.editor.layout[NodeRef.mapping(source.id.toInt())], const Offset(400, 40));
      expect(s.editor.selection, MappingSelected(source.id.toInt()));
      final scene = buildScene(s.project!, s.editor.layout);
      final node = scene.nodes.firstWhere((n) => n.ref == NodeRef.mapping(source.id.toInt()));
      expect(node.source, isTrue);
      expect(node.sockets.where((x) => x.ref.side == SocketSide.input), isEmpty);
      expect(simulationInputs(s.project!).map((m) => m.id), [source.id]);
      // one undo removes the Source and nothing else
      store.dispatch(const UndoRequested());
      s = await store.until((x) => x.editor.pendingRequests == 0 && x.project!.mappings.isEmpty);
      expect(s.project!.concepts, hasLength(2));
      store.dispatch(const RedoRequested());
      s = await store.until((x) => x.editor.pendingRequests == 0 && x.project!.mappings.isNotEmpty);
      expect(s.project!.mappings.single.id, source.id);

      // a new concept and its Source: one revision, both placed, one undo
      final before = store.state.revision;
      store.dispatch(const NewSourceRequested(position: Offset(400, 200)));
      await store.until((x) => x.editor.sourceSheet?.ready ?? false);
      expect(store.state.editor.sourceSheet!.candidates!.preset.hasConceptName(), isFalse);
      store.dispatch(
        CreateSourceRequested(
          sourceName: 'humidityInput',
          newConceptName: 'Humidity',
          newConceptRepresentation: pb.Representation(quantity: pb.Dim()),
        ),
      );
      s = await store.until(
        (x) => x.editor.pendingRequests == 0 && x.project!.concepts.length == 3,
      );
      expect(s.revision, before + 1, reason: 'one transaction, one revision');
      final humidity = s.project!.concepts.firstWhere((c) => c.name == 'Humidity');
      final input = s.project!.mappings.firstWhere((m) => m.name == 'humidityInput');
      expect(input.signature.output, humidity.id);
      expect(relationshipRole(input), RelationshipRole.source);
      expect(s.editor.layout[NodeRef.concept(humidity.id.toInt())], const Offset(400, 200));
      expect(s.editor.layout[NodeRef.mapping(input.id.toInt())], const Offset(160, 200));
      expect((s.editor.selection as ConceptSelected).id, humidity.id.toInt());
      expect(s.editor.renaming, isNull, reason: 'named on the sheet');
      store.dispatch(const UndoRequested());
      s = await store.until(
        (x) => x.editor.pendingRequests == 0 && x.project!.concepts.length == 2,
      );
      expect(s.project!.mappings.any((m) => m.name == 'humidityInput'), isFalse);
      store.dispatch(const RedoRequested());
      s = await store.until(
        (x) => x.editor.pendingRequests == 0 && x.project!.concepts.length == 3,
      );

      // a taken name is the ordinary refusal: nothing created, no identity
      // consumed, the revision unmoved
      final rev = store.state.revision;
      store.dispatch(const NewSourceRequested());
      await store.until((x) => x.editor.sourceSheet?.ready ?? false);
      store.dispatch(
        const CreateSourceRequested(sourceName: 'roomTemperatureInput', newConceptName: 'Pressure'),
      );
      s = await store.until((x) => x.editor.pendingRequests == 0);
      expect(s.revision, rev);
      expect(s.project!.concepts.any((c) => c.name == 'Pressure'), isFalse);
      expect(s.editor.lastError?.code, startsWith('edit.duplicate_'));

      // the text: ordinary declarations, the preferred spelling, no `?`
      store.dispatch(const DesignViewChanged(DesignView.code));
      s = await store.until((x) => x.editor.sources.revision == x.revision);
      expect(
        s.editor.sources.text,
        contains('mapping roomTemperatureInput : () -> RoomTemperature'),
      );
      expect(s.editor.sources.text, contains('mapping humidityInput : () -> Humidity'));
      expect(s.editor.sources.text, isNot(contains('?')));

      // saved and reopened: the same shape, the role re-derived from it
      store.dispatch(const SaveRequested());
      await store.until((x) => x.editor.pendingRequests == 0 && !x.project!.dirty);
      final sidecar = File(p.join(root, '.bdl', 'authoring.json')).readAsStringSync();
      expect(sidecar.toLowerCase(), isNot(contains('source')));
      expect(sidecar, isNot(contains('preset')));
      store.dispatch(const CloseProjectRequested());
      await store.until((x) => x.project == null);
      store.dispatch(OpenProjectRequested(root));
      s = await store.until((x) => x.project != null && x.editor.pendingRequests == 0);
      final reopened = s.project!.mappings.firstWhere((m) => m.name == 'roomTemperatureInput');
      expect(reopened.id, source.id);
      expect(relationshipRole(reopened), RelationshipRole.source);
    } finally {
      await store.dispose();
      await dir.delete(recursive: true);
    }
  }, skip: bdld == null ? 'bdld is not built' : false);
}
