/// A text project through the real Studio stack against the real `bdld`
/// (ADR-0020): created from the welcome page's "New Text Project…", edited
/// on the canvas, saved as `src/*.bdl` text with identities kept, refused
/// when the sources changed underneath, reloaded; and a hand-written
/// project opened and simulated with nothing but its text.
library;

import 'dart:convert';
import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/shell.dart';
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

const _concepts = '''
// Shared vocabulary.
concept Tilt : Angle
concept Brightness : Scalar

clock interaction
''';

const _main = '''
mapping tilt : Tilt @interaction
mapping tiltValue : Tilt @interaction
tiltValue() = tilt

/// How bright the lamp is for a tilt.
mapping dimByTilt : Tilt -> Brightness
dimByTilt(t) = t / (90 deg)

mapping brightness : Brightness @interaction
brightness() = dimByTilt(tiltValue)

output light : Brightness @interaction
drive light = brightness
''';

void main() {
  final bdld = _findBdld();

  late TestStore store;
  late Directory dir;

  Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
  Future<AppState> analysed() => store.until(
    (s) =>
        s.analysis != null &&
        s.analysis!.revision == s.flat!.revision &&
        s.system != null &&
        s.system!.revision == s.flat!.revision,
  );
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  Map<String, dynamic> identities(String root) =>
      jsonDecode(File(p.join(root, '.bdl', 'identities.json')).readAsStringSync())
          as Map<String, dynamic>;
  int idOf(String root, String key) => (identities(root)['keys'][key]['id'] as num).toInt();

  Future<void> connect() async {
    store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    store.dispatch(const AppStarted());
    await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
    expect(store.state.connection, isA<Connected>());
    dir = await Directory.systemTemp.createTemp('bdl-studio-text-e2e');
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  test(
    'new text project: canvas edits become source text, identities survive a save and a reload',
    () async {
      await connect();
      final root = p.join(dir.path, 'lamp');
      try {
        store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp'));
        await store.until((s) => s.project != null && s.system != null);
        expect(store.state.flat!.kind, pb.ProjectKind.PROJECT_KIND_TEXT);
        expect(store.state.isSystem, isTrue, reason: 'a text project is a system project');
        expect(File(p.join(root, 'src', 'main.bdl')).existsSync(), isTrue);
        expect(File(p.join(root, 'bdl.toml')).readAsStringSync(), isNot(contains('kind =')));

        // Author on the canvas as in any project.
        store.dispatch(
          CreateConceptRequested(
            name: 'Tilt',
            representation: pb.Representation(quantity: pb.Dim(angle: 1)),
          ),
        );
        await settled();
        store.dispatch(
          CreateConceptRequested(
            name: 'Brightness',
            representation: pb.Representation(quantity: pb.Dim()),
          ),
        );
        await settled();
        store.dispatch(
          CreateMappingRequested(
            name: 'dimByTilt',
            inputs: [conceptId('Tilt')],
            output: conceptId('Brightness'),
          ),
        );
        await settled();
        final dim = mappingId('dimByTilt');
        store.dispatch(DefinitionDraftChanged(mappingId: dim, source: 'Tilt / 90 deg'));
        await store.until((s) => s.draft(dim)?.check == DraftCheck.checked);
        store.dispatch(CommitDefinitionRequested(dim));
        await store.until((s) => s.committedDefinition(dim) == 'Tilt / 90 deg');
        expect(store.state.flat!.dirty, isTrue);

        // Save: the edits are text under src/, the ids are in the sidecar.
        store.dispatch(const SaveRequested());
        await store.until((s) => s.editor.pendingRequests == 0 && !s.flat!.dirty);
        expect(store.state.editor.lastError, isNull);
        final main = File(p.join(root, 'src', 'main.bdl')).readAsStringSync();
        expect(main, contains('concept Tilt : Angle'));
        expect(main, contains('mapping dimByTilt : Tilt -> Brightness'));
        expect(main, contains('dimByTilt(Tilt) =\n  Tilt / 90 deg'));
        final tiltId = conceptId('Tilt');
        expect(idOf(root, 'concept:Tilt'), tiltId);
        expect(idOf(root, 'mapping:dimByTilt'), dim);

        // Rename on the canvas, save: the text follows and the id stays.
        store.dispatch(RenameConceptRequested(id: tiltId, name: 'Lean'));
        await settled();
        store.dispatch(const SaveRequested());
        await store.until((s) => s.editor.pendingRequests == 0 && !s.flat!.dirty);
        final renamed = File(p.join(root, 'src', 'main.bdl')).readAsStringSync();
        expect(renamed, contains('concept Lean : Angle'));
        expect(renamed, contains('mapping dimByTilt : Lean -> Brightness'));
        expect(renamed, contains('dimByTilt(Lean) =\n  Lean / 90 deg'), reason: 'the body follows');
        expect(renamed, isNot(contains(' Tilt')));
        expect(idOf(root, 'concept:Lean'), tiltId);
        expect(identities(root)['keys'], isNot(contains('concept:Tilt')));

        // Another tool edits the source: a save is refused with the
        // conflict code the banner offers to reload from, and reloading
        // brings the disk's version in under the same identities.
        store.dispatch(
          CreateConceptRequested(
            name: 'Warm',
            representation: pb.Representation(boolean: pb.Unit()),
          ),
        );
        await settled();
        final external = renamed.replaceFirst(
          'concept Lean : Angle',
          'concept Lean : Angle\nconcept Glow : Scalar',
        );
        // File stamps are second-granular on some file systems.
        await Future<void>.delayed(const Duration(milliseconds: 1100));
        File(p.join(root, 'src', 'main.bdl')).writeAsStringSync(external);
        store.dispatch(const SaveRequested());
        await settled();
        expect(store.state.editor.lastError?.code, kChangedOnDisk);
        expect(store.state.flat!.dirty, isTrue);
        store.dispatch(const ReloadProjectRequested());
        await store.until(
          (s) => s.editor.pendingRequests == 0 && s.project != null && s.system != null,
        );
        await analysed();
        expect(store.state.editor.lastError, isNull);
        final names = store.state.project!.concepts.map((c) => c.name).toList();
        expect(names, containsAll(['Lean', 'Brightness', 'Glow']));
        expect(names, isNot(contains('Warm')), reason: 'unsaved edits are dropped by a reload');
        expect(conceptId('Lean'), tiltId);
        expect(mappingId('dimByTilt'), dim);
        expect(store.state.flat!.dirty, isFalse);
      } finally {
        await teardown();
      }
    },
    skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false,
    timeout: const Timeout(Duration(minutes: 2)),
  );

  test(
    'a hand-written text project opens, analyses, simulates and keeps its doc comments',
    () async {
      await connect();
      final root = p.join(dir.path, 'written');
      try {
        Directory(p.join(root, 'src')).createSync(recursive: true);
        File(p.join(root, 'bdl.toml')).writeAsStringSync(
          'schema_version = 1\nname = "written"\ncompiler_version = "test"\nkind = "text"\n',
        );
        File(p.join(root, 'src', 'concepts.bdl')).writeAsStringSync(_concepts);
        File(p.join(root, 'src', 'main.bdl')).writeAsStringSync(_main);
        store.dispatch(OpenProjectRequested(root));
        await store.until((s) => s.project != null && s.system != null);
        await analysed();
        final s = store.state;
        expect(s.flat!.kind, pb.ProjectKind.PROJECT_KIND_TEXT);
        expect(s.editor.lastError, isNull);
        expect(s.project!.concepts.map((c) => c.name), containsAll(['Tilt', 'Brightness']));
        expect(
          s.project!.mappings.firstWhere((m) => m.name == 'dimByTilt').description,
          'How bright the lamp is for a tilt.',
        );
        expect(s.analysis!.causal, isTrue);
        expect(s.systemAnalysis, isNotNull);

        // Simulate: tilt 45° → brightness 0.5 on the light.
        final tilt = s.flat!.concepts.firstWhere((c) => c.name == 'Tilt').id;
        final raw = s.flat!.mappings.firstWhere((m) => m.name == 'tilt').id.toInt();
        store.dispatch(const SimulationResetRequested());
        store.dispatch(
          SimulationInputChanged(
            mappingId: raw,
            value: pb.Value(
              semantic: pb.SemanticValue(
                conceptId: Int64(tilt.toInt()),
                repr: pb.Value(
                  quantity: pb.Quantity(dim: pb.Dim(angle: 1), value: 45 * 3.141592653589793 / 180),
                ),
              ),
            ),
          ),
        );
        store.dispatch(const SimulationStepRequested(2));
        final ran = await store.until((s) => s.editor.simulation.samples.length == 2);
        final bright = ran.flat!.mappings.firstWhere((m) => m.name == 'brightness').id.toInt();
        final last = ran.editor.simulation.samples.last;
        expect(
          last.values.firstWhere((v) => v.mappingId.toInt() == bright).rendered,
          contains('0.5'),
        );

        // The sidecar was written on open with every item's id.
        expect(
          idOf(root, 'concept:Tilt'),
          s.flat!.concepts.firstWhere((c) => c.name == 'Tilt').id.toInt(),
        );
        expect(File(p.join(root, 'src', 'main.bdl')).readAsStringSync(), _main);
      } finally {
        await teardown();
      }
    },
    skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false,
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
