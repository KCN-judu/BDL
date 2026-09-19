/// The Standard Concept Library in Studio: one creation path for the
/// right-click menu and the Library drag, the create-then-rename flow,
/// recent templates as a preference, inline rename, and search — all
/// through the real reducer and effect executor against a scripted daemon.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/concept_library_panel.dart';
import 'package:fixnum/fixnum.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/test_store.dart';

pb.ConceptTemplateView tpl(
  String id,
  String name, {
  String category = 'environment',
  pb.RoleHint role = pb.RoleHint.ROLE_HINT_INPUT,
  pb.Representation? representation,
  String unit = '',
  List<String> keywords = const [],
}) => pb.ConceptTemplateView(
  id: id,
  displayName: name,
  defaultName: name.replaceAll(' ', ''),
  description: 'about $name',
  category: category,
  roleHint: role,
  representation: representation,
  unit: unit,
  keywords: keywords,
);

final temperature = tpl(
  'std.environment.temperature',
  'Temperature',
  representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
  unit: 'K',
  keywords: ['temp', 'heat'],
);
final light = tpl(
  'std.environment.ambient_light',
  'Ambient Light',
  representation: pb.Representation(quantity: pb.Dim(luminous: 1, angle: 2, length: -2)),
  unit: 'lx',
  keywords: ['light', 'lux'],
);
final motorSpeed = tpl(
  'std.actuator.motor_speed',
  'Motor Speed',
  category: 'actuation',
  role: pb.RoleHint.ROLE_HINT_OUTPUT,
  representation: pb.Representation(quantity: pb.Dim()),
  keywords: ['motor', 'pwm'],
);
final motorAngle = tpl(
  'std.actuator.motor_angle',
  'Motor Angle',
  category: 'actuation',
  role: pb.RoleHint.ROLE_HINT_OUTPUT,
  representation: pb.Representation(quantity: pb.Dim(angle: 1)),
  unit: 'deg',
  keywords: ['motor'],
);
final pressed = tpl(
  'std.human.button_pressed',
  'Button Pressed',
  category: 'human',
  representation: pb.Representation(boolean: pb.Unit()),
);
final open = tpl('std.x.open', 'Open Thing', category: 'x');

pb.ConceptTemplatesResponse library() => pb.ConceptTemplatesResponse(
  libraries: [
    pb.ConceptLibraryView(
      id: 'std',
      name: 'Standard',
      schemaVersion: 1,
      version: '0.1',
      templates: [temperature, light, motorSpeed, motorAngle, pressed, open],
    ),
  ],
);

pb.ProjectProjection project({int revision = 1, List<pb.ConceptView> concepts = const []}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll(concepts);

/// A connected store holding an empty project at revision 1, whose daemon
/// answers the library and instantiations.
Future<(TestStore, FakeDaemon)> connected() async {
  var nextId = 10;
  var revision = 1;
  final daemon = FakeDaemon((m) async {
    if (m.hasHandshake()) return okHandshake();
    if (m.hasListConceptTemplates()) return pb.Response(conceptTemplates: library());
    if (m.hasInstantiateConceptTemplate()) {
      final r = m.instantiateConceptTemplate;
      final id = nextId++;
      revision += 1;
      final t = library().libraries.single.templates.firstWhere((t) => t.id == r.templateId);
      return pb.Response(
        editApplied: pb.EditApplied(
          project: project(
            revision: revision,
            concepts: [pb.ConceptView(id: Int64(id), name: t.defaultName)],
          ),
          outcome: pb.EditOutcome(createdConcept: Int64(id)),
        ),
      );
    }
    if (m.hasSetLayout() || m.hasSubscribeProject()) return pb.Response(ack: pb.Ack());
    if (m.hasRunAnalysis()) {
      return pb.Response(
        analysis: pb.AnalysisResponse(analysis: pb.ProjectAnalysis(revision: Int64(revision))),
      );
    }
    if (m.hasApplyEdit()) {
      revision += 1;
      return pb.Response(
        editApplied: pb.EditApplied(project: project(revision: revision)),
      );
    }
    return errorResponse('test.unexpected', 'unexpected request ${m.whichPayload()}');
  });
  final store = TestStore(spawn: (_) async => daemon);
  store.dispatch(const AppStarted());
  await store.until((s) => s.connection is Connected && s.library != null);
  store.dispatch(ProjectReceived(project()));
  await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
  daemon.requests.clear();
  return (store, daemon);
}

void main() {
  test('the library is asked for once per connection and kept apart from the project', () async {
    final (store, daemon) = await connected();
    expect(store.state.templates.map((t) => t.id), contains('std.environment.temperature'));
    expect(store.state.library!.libraries.single.version, '0.1');
    // Closing the project does not lose the vocabulary.
    store.dispatch(const ProjectClosed());
    expect(store.state.project, isNull);
    expect(store.state.templates, isNotEmpty);
    expect(daemon.requests.where((r) => r.hasListConceptTemplates()), isEmpty);
    await store.dispose();
  });

  test('right-click and drag are one creation path; position is the only difference', () {
    final s = AppState(
      connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
      project: project(),
      library: library(),
    );
    final fromMenu = reduce(
      s,
      const InsertConceptTemplateRequested(
        'std.environment.temperature',
        position: Offset(100, 40),
      ),
    );
    final fromDrag = reduce(
      s,
      const InsertConceptTemplateRequested(
        'std.environment.temperature',
        position: Offset(300, 80),
      ),
    );
    final fromPanel = reduce(
      s,
      const InsertConceptTemplateRequested('std.environment.temperature'),
    );
    for (final t in [fromMenu, fromDrag, fromPanel]) {
      expect(t.effects, hasLength(1));
      final e = t.effects.single as InstantiateConceptTemplate;
      expect(e.templateId, 'std.environment.temperature');
      expect(e.baseRevision, 1);
      expect(t.state.editor.pendingRequests, 1);
      expect(t.state.editor.recentTemplates, ['std.environment.temperature']);
    }
    expect(fromMenu.state.editor.pendingInsert!.position, const Offset(100, 40));
    expect(fromDrag.state.editor.pendingInsert!.position, const Offset(300, 80));
    expect(fromPanel.state.editor.pendingInsert!.position, isNull);
    // A second insertion waits for the first to be answered.
    expect(
      reduce(fromMenu.state, const InsertConceptTemplateRequested('std.motion.tilt')).effects,
      isEmpty,
    );
  });

  test('create-then-rename: the answer places, selects and opens the name', () async {
    final (store, daemon) = await connected();
    store.dispatch(
      const InsertConceptTemplateRequested(
        'std.environment.temperature',
        position: Offset(120, 64),
      ),
    );
    final s = await store.until((x) => x.editor.pendingRequests == 0);
    final id = s.project!.concepts.single.id.toInt();
    expect(s.project!.concepts.single.name, 'Temperature');
    expect((s.editor.selection as ConceptSelected).id, id);
    expect(s.editor.renaming, NodeRef.concept(id));
    expect(s.editor.layout[NodeRef.concept(id)], const Offset(120, 64));
    expect(s.editor.pendingInsert, isNull);
    // The position went to the daemon as layout (never a revision).
    await store.until((_) => daemon.requests.any((r) => r.hasSetLayout()));
    final layout = daemon.requests.firstWhere((r) => r.hasSetLayout()).setLayout.layout;
    expect(layout.concepts.single.id.toInt(), id);
    expect(layout.concepts.single.x, 120);

    // Typing the product's name commits one ordinary rename edit.
    daemon.requests.clear();
    store.dispatch(InlineRenameFinished(NodeRef.concept(id), name: 'MotorTemperature'));
    expect(store.state.editor.renaming, isNull);
    await store.until((x) => x.editor.pendingRequests == 0);
    final rename = daemon.requests.single.applyEdit.op.renameConcept;
    expect(rename.id.toInt(), id);
    expect(rename.name, 'MotorTemperature');

    // Esc, an empty name, or the same name change nothing.
    for (final name in [null, '', '  ', 'Temperature']) {
      store.dispatch(InlineRenameStarted(NodeRef.concept(id)));
      expect(store.state.editor.renaming, NodeRef.concept(id));
      daemon.requests.clear();
      store.dispatch(InlineRenameFinished(NodeRef.concept(id), name: name));
      expect(store.state.editor.renaming, isNull);
      expect(daemon.requests, isEmpty, reason: 'name=$name');
    }
    await store.dispose();
  });

  test(
    'an insertion the panel started auto-places: no layout, still selected and renaming',
    () async {
      final (store, daemon) = await connected();
      store.dispatch(const InsertConceptTemplateRequested('std.human.button_pressed'));
      final s = await store.until((x) => x.editor.pendingRequests == 0);
      final id = s.project!.concepts.single.id.toInt();
      expect((s.editor.selection as ConceptSelected).id, id);
      expect(s.editor.renaming, NodeRef.concept(id));
      expect(s.editor.layout, isEmpty);
      expect(daemon.requests.where((r) => r.hasSetLayout()), isEmpty);
      await store.dispose();
    },
  );

  test('a refused insertion clears the pending state and reports', () async {
    final daemon = FakeDaemon((m) async {
      if (m.hasHandshake()) return okHandshake();
      if (m.hasListConceptTemplates()) return pb.Response(conceptTemplates: library());
      return errorResponse('edit.stale_revision', 'moved on');
    });
    final store = TestStore(spawn: (_) async => daemon);
    store.dispatch(const AppStarted());
    await store.until((s) => s.library != null);
    store.dispatch(ProjectReceived(project()));
    store.dispatch(const InsertConceptTemplateRequested('std.environment.temperature'));
    final s = await store.until((x) => x.editor.pendingRequests == 0);
    expect(s.editor.pendingInsert, isNull);
    expect(s.editor.lastError?.code, 'edit.stale_revision');
    expect(s.editor.renaming, isNull);
    await store.dispose();
  });

  test('recent templates: most recent first, no duplicates, capped', () {
    var recent = <String>[];
    for (final id in ['a', 'b', 'c', 'a', 'd', 'e', 'f', 'g', 'h']) {
      recent = rememberTemplate(recent, id);
    }
    expect(recent, ['h', 'g', 'f', 'e', 'd', 'a']);
    expect(recent.length, EditorState.maxRecentTemplates);
    expect(rememberTemplate(recent, 'e').first, 'e');
    expect(rememberTemplate(recent, 'e').length, EditorState.maxRecentTemplates);
  });

  test('search matches display name, default name, keywords, unit and category', () {
    final all = library().libraries.single.templates;
    List<String> ids(String q) => searchTemplates(all, q).map((t) => t.id).toList();
    expect(ids('lux'), ['std.environment.ambient_light']);
    expect(ids('LX'), ['std.environment.ambient_light']);
    expect(ids('motor'), ['std.actuator.motor_speed', 'std.actuator.motor_angle']);
    expect(ids('ambientlight'), ['std.environment.ambient_light']);
    expect(ids('Actuation'), ['std.actuator.motor_speed', 'std.actuator.motor_angle']);
    expect(ids('heat'), ['std.environment.temperature']);
    expect(ids(''), hasLength(all.length));
    expect(ids('zzz'), isEmpty);
  });

  test('rows say what a value is measured as in the contract\'s words', () {
    expect(representationWord(kEnglish, temperature), 'K');
    expect(representationWord(kEnglish, motorSpeed), 'no unit');
    expect(representationWord(kEnglish, pressed), 'on–off');
    expect(representationWord(kEnglish, open), 'decide later');
    expect(categoryLabel(kEnglish, 'human'), 'Human interaction');
    expect(categoryLabel(kEnglish, 'custom'), 'Custom');
  });

  test('selecting anything else closes an inline rename', () {
    final s = AppState(
      project: project(
        concepts: [pb.ConceptView(id: Int64(3), name: 'Tilt')],
      ),
      editor: const EditorState(renaming: NodeRef.concept(3)),
    );
    final t = reduce(s, const SelectionChanged(NoSelection()));
    expect(t.state.editor.renaming, isNull);
  });
}
