/// The Standard Library in Studio: one creation path for the right-click
/// menu and the Library drag, the create-then-rename flow, a Source item's
/// two objects placed together, recent items as a preference, inline
/// rename, and search in three locales — all through the real reducer and
/// effect executor against a scripted daemon.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/library_panel.dart';
import 'package:flutter/material.dart';
import 'package:fixnum/fixnum.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/test_store.dart';

import 'support/roles.dart';

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

/// A Concept item carrying its template view.
pb.LibraryItemView conceptItem(pb.ConceptTemplateView t) => pb.LibraryItemView(
  id: t.id,
  category: 'concept',
  displayName: t.displayName,
  description: t.description,
  group: t.category,
  keywords: t.keywords,
  creates: [
    pb.LibraryObjectView(
      kind: 'concept',
      key: 'concept',
      name: t.defaultName,
      typeName: t.typeName,
      representation: t.hasRepresentation() ? t.representation : null,
      unit: t.unit,
    ),
  ],
  concept: t,
);

/// The Temperature Input Source item as the daemon serves it: a preset —
/// the names, value form and unit it suggests — beside what the legacy
/// request would create.
final temperatureInput = pb.LibraryItemView(
  id: 'std.source.temperature',
  category: 'source',
  displayName: 'Temperature Input',
  description: 'A temperature the environment provides.',
  group: 'environment',
  keywords: ['temp', 'thermal', 'sensor', 'input'],
  creates: [
    pb.LibraryObjectView(
      kind: 'concept',
      key: 'value',
      name: 'Temperature',
      typeName: 'Temperature',
      representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
      unit: 'K',
    ),
    pb.LibraryObjectView(
      kind: 'mapping',
      key: 'source',
      name: 'temperatureInput',
      signature: '() -> Temperature',
    ),
  ],
  preset: pb.SourcePresetView(
    conceptName: 'Temperature',
    representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
    typeName: 'Temperature',
    unit: 'K',
    sourceName: 'temperatureInput',
  ),
);

/// The External Input Source item: a preset whose value form is left open.
final externalInput = pb.LibraryItemView(
  id: 'std.source.external',
  category: 'source',
  displayName: 'External Input',
  description: 'A value provided from outside the product.',
  group: 'external',
  keywords: ['host', 'network', 'external', 'input', 'source'],
  creates: [
    pb.LibraryObjectView(kind: 'concept', key: 'value', name: 'ExternalValue'),
    pb.LibraryObjectView(
      kind: 'mapping',
      key: 'source',
      name: 'externalInput',
      signature: '() -> ExternalValue',
    ),
  ],
  preset: pb.SourcePresetView(conceptName: 'ExternalValue', sourceName: 'externalInput'),
);

pb.LibraryItemsResponse library() => pb.LibraryItemsResponse(
  libraries: [
    pb.LibraryView(
      id: 'std',
      name: 'Standard',
      schemaVersion: 2,
      version: '0.2',
      items: [
        externalInput,

        for (final t in [temperature, light, motorSpeed, motorAngle, pressed, open]) conceptItem(t),
        temperatureInput,
      ],
    ),
  ],
);

pb.ProjectProjection project({int revision = 1, List<pb.ConceptView> concepts = const []}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll(concepts);

/// A connected store holding an empty project at revision 1, whose daemon
/// answers the library and instantiations.
Future<(TestStore, FakeDaemon)> connected({List<pb.ConceptView> concepts = const []}) async {
  var nextId = 10;
  var revision = 1;
  concepts = List.of(concepts);
  final daemon = FakeDaemon((m) async {
    if (m.hasHandshake()) return okHandshake();
    if (m.hasListLibraryItems()) return pb.Response(libraryItems: library());
    if (m.hasInstantiateLibraryItem()) {
      final r = m.instantiateLibraryItem;
      final id = nextId++;
      revision += 1;
      final item = library().libraries.single.items.firstWhere((i) => i.id == r.itemId);
      // a Source item: the concept and the relationship, in one answer
      final mapping = item.creates.where((o) => o.kind == 'mapping').firstOrNull;
      final mappingId = mapping == null ? null : nextId++;
      final value = item.creates.first;
      return pb.Response(
        editApplied: pb.EditApplied(
          project: project(revision: revision)
            ..concepts.add(pb.ConceptView(id: Int64(id), name: value.name))
            ..mappings.addAll([
              if (mappingId != null)
                mappingView(
                  id: Int64(mappingId),
                  name: mapping!.name,
                  signature: pb.Signature(inputs: const [], output: Int64(id)),
                ),
            ]),
          outcome: pb.EditOutcome(
            createdConcept: Int64(id),
            createdMapping: mappingId == null ? null : Int64(mappingId),
          ),
        ),
      );
    }
    if (m.hasListSourceCandidates()) {
      // ranked by the daemon: the design's concepts, the preset's value
      // form first (here: every concept, in id order)
      final r = m.listSourceCandidates;
      final item = r.itemId.isEmpty
          ? null
          : library().libraries.single.items.firstWhere((i) => i.id == r.itemId);
      return pb.Response(
        sourceCandidates: pb.SourceCandidatesResponse(
          revision: r.revision,
          candidates: [
            for (final c in concepts)
              pb.SourceCandidateView(conceptId: c.id, preferred: item != null),
          ],
          preset: item?.preset,
          suggestedConceptName: item?.preset.conceptName ?? '',
          suggestedSourceName: item?.preset.sourceName ?? '',
        ),
      );
    }
    if (m.hasCreateSource()) {
      final r = m.createSource;
      revision += 1;
      final conceptId = r.hasNewConcept() ? nextId++ : r.existingConcept.toInt();
      if (r.hasNewConcept()) {
        concepts.add(pb.ConceptView(id: Int64(conceptId), name: r.newConcept.name));
      }
      final mappingId = nextId++;
      return pb.Response(
        editApplied: pb.EditApplied(
          project: project(revision: revision, concepts: concepts)
            ..mappings.add(
              mappingView(
                id: Int64(mappingId),
                name: r.sourceName,
                signature: pb.Signature(inputs: const [], output: Int64(conceptId)),
              ),
            ),
          outcome: pb.EditOutcome(
            createdConcept: r.hasNewConcept() ? Int64(conceptId) : null,
            createdMapping: Int64(mappingId),
          ),
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
  store.dispatch(ProjectReceived(project(concepts: concepts)));
  await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
  daemon.requests.clear();
  return (store, daemon);
}

void main() {
  test('the library is asked for once per connection and kept apart from the project', () async {
    final (store, daemon) = await connected();
    expect(store.state.templates.map((t) => t.id), contains('std.environment.temperature'));
    expect(store.state.library!.libraries.single.version, '0.2');
    expect(store.state.libraryItems.where((i) => i.category == 'source').map((i) => i.id), [
      'std.source.external',
      'std.source.temperature',
    ]);
    // Closing the project does not lose the vocabulary.
    store.dispatch(const ProjectClosed());
    expect(store.state.project, isNull);
    expect(store.state.templates, isNotEmpty);
    expect(daemon.requests.where((r) => r.hasListLibraryItems()), isEmpty);
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
      const InsertLibraryItemRequested('std.environment.temperature', position: Offset(100, 40)),
    );
    final fromDrag = reduce(
      s,
      const InsertLibraryItemRequested('std.environment.temperature', position: Offset(300, 80)),
    );
    final fromPanel = reduce(s, const InsertLibraryItemRequested('std.environment.temperature'));
    for (final t in [fromMenu, fromDrag, fromPanel]) {
      expect(t.effects, hasLength(1));
      final e = t.effects.single as InstantiateLibraryItem;
      expect(e.itemId, 'std.environment.temperature');
      expect(e.baseRevision, 1);
      expect(t.state.editor.pendingRequests, 1);
      expect(t.state.editor.recentTemplates, ['std.environment.temperature']);
    }
    expect(fromMenu.state.editor.pendingInsert!.position, const Offset(100, 40));
    expect(fromDrag.state.editor.pendingInsert!.position, const Offset(300, 80));
    expect(fromPanel.state.editor.pendingInsert!.position, isNull);
    // A second insertion waits for the first to be answered.
    expect(
      reduce(fromMenu.state, const InsertLibraryItemRequested('std.motion.tilt')).effects,
      isEmpty,
    );
  });

  test('create-then-rename: the answer places, selects and opens the name', () async {
    final (store, daemon) = await connected();
    store.dispatch(
      const InsertLibraryItemRequested('std.environment.temperature', position: Offset(120, 64)),
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
      store.dispatch(const InsertLibraryItemRequested('std.human.button_pressed'));
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
      if (m.hasListLibraryItems()) return pb.Response(libraryItems: library());
      return errorResponse('edit.stale_revision', 'moved on');
    });
    final store = TestStore(spawn: (_) async => daemon);
    store.dispatch(const AppStarted());
    await store.until((s) => s.library != null);
    store.dispatch(ProjectReceived(project()));
    store.dispatch(const InsertLibraryItemRequested('std.environment.temperature'));
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
    final all = library().libraries.single.items;
    List<String> ids(String q) => searchItems(all, q).map((t) => t.id).toList();
    expect(ids('lux'), ['std.environment.ambient_light']);
    expect(ids('LX'), ['std.environment.ambient_light']);
    expect(ids('motor'), ['std.actuator.motor_speed', 'std.actuator.motor_angle']);
    expect(ids('ambientlight'), ['std.environment.ambient_light']);
    expect(ids('Actuation'), ['std.actuator.motor_speed', 'std.actuator.motor_angle']);
    expect(ids('heat'), ['std.environment.temperature']);
    expect(ids('sensor'), ['std.source.temperature']);
    expect(ids('temperatureInput'), ['std.source.temperature']);
    expect(ids(''), hasLength(all.length));
    expect(ids('zzz'), isEmpty);
    final concepts = library().libraries.single.items
        .where((i) => i.hasConcept())
        .map((i) => i.concept);
    expect(searchTemplates(concepts, 'heat').map((t) => t.id), ['std.environment.temperature']);
  });

  test('search and names follow the locale; what is created does not', () {
    final all = library().libraries.single.items;
    final zh = lookupAppLocalizations(const Locale('zh'));
    final ja = lookupAppLocalizations(const Locale('ja'));
    List<String> ids(AppLocalizations l, String q) =>
        searchItems(all, q, l10n: l).map((t) => t.id).toList();
    // the localized name and tags
    expect(ids(zh, '温度'), ['std.environment.temperature', 'std.source.temperature']);
    expect(ids(ja, '温度'), ['std.environment.temperature', 'std.source.temperature']);
    expect(ids(zh, '传感器'), ['std.source.temperature']);
    expect(ids(ja, 'センサー'), ['std.source.temperature']);
    // the canonical English still matches in every locale
    expect(ids(zh, 'Temperature Input'), ['std.source.temperature']);
    expect(ids(ja, 'lux'), ['std.environment.ambient_light']);
    // the required cases, per locale: a name, a keyword, a tag
    expect(ids(kEnglish, 'temperature'), contains('std.source.temperature'));
    expect(ids(kEnglish, 'sensor'), contains('std.source.temperature'));
    expect(ids(kEnglish, 'external'), ['std.source.external']);
    expect(ids(zh, '外部'), ['std.source.external']);
    expect(ids(ja, '外部'), ['std.source.external']);
    expect(ids(ja, 'センサー'), isNot(contains('std.source.external')));
    expect(ids(zh, '输入'), containsAll(['std.source.temperature', 'std.source.external']));
    expect(ids(ja, '入力'), containsAll(['std.source.temperature', 'std.source.external']));
    // a Source item is a preset: its row says the value form it suggests
    // — never a signature over a concept nobody has chosen — and an open
    // one says so in the sheet's words
    expect(itemWord(kEnglish, temperatureInput), 'K');
    expect(itemWord(kEnglish, externalInput), 'decide later');
    expect(itemPreview(kEnglish, externalInput), isEmpty);
    expect(sourceItemHover(kEnglish, temperatureInput), contains('Temperature'));
    expect(sourceItemHover(kEnglish, temperatureInput), isNot(contains('() ->')));
    expect(sourceItemHover(kEnglish, externalInput), kEnglish.inputForAConcept);
    // names and descriptions in each locale; the section titles
    expect(itemName(kEnglish, temperatureInput), 'Temperature Input');
    expect(itemName(zh, temperatureInput), '温度输入');
    expect(itemName(ja, temperatureInput), '温度入力');
    expect(itemDescription(zh, temperatureInput), isNot(temperatureInput.description));
    expect(itemDescription(ja, temperatureInput), isNotEmpty);
    expect(categoryTitle(kEnglish, 'source'), 'Sources');
    expect(categoryTitle(zh, 'source'), '来源');
    expect(categoryTitle(ja, 'source'), '入力元');
    expect(categoryTitle(kEnglish, 'concept'), 'Concepts');
    // what a preset suggests is the daemon's: identifiers and units alike
    for (final l in [kEnglish, zh, ja]) {
      expect(temperatureInput.preset.conceptName, 'Temperature');
      expect(temperatureInput.preset.sourceName, 'temperatureInput');
      expect(itemWord(l, temperatureInput), 'K');
    }
    // an item the catalog does not know keeps its canonical English
    final foreign = pb.LibraryItemView(id: 'team.x', category: 'concept', displayName: 'X Thing');
    expect(itemName(ja, foreign), 'X Thing');
  });

  test('a Source item opens the sheet, prefilled; nothing is created before the choice', () async {
    final (store, daemon) = await connected(
      concepts: [
        pb.ConceptView(id: Int64(1), name: 'RoomTemperature'),
        pb.ConceptView(id: Int64(2), name: 'MotorTemperature'),
      ],
    );
    final before = store.state.revision;
    store.dispatch(
      const NewSourceRequested(presetId: 'std.source.temperature', position: Offset(120, 64)),
    );
    var s = await store.until((x) => x.editor.sourceSheet?.ready ?? false);
    expect(s.revision, before, reason: 'nothing committed');
    expect(daemon.requests.where((r) => r.hasCreateSource()), isEmpty);
    expect(daemon.requests.where((r) => r.hasInstantiateLibraryItem()), isEmpty);
    final sheet = s.editor.sourceSheet!;
    expect(sheet.presetId, 'std.source.temperature');
    expect(sheet.candidates!.candidates.map((c) => c.conceptId.toInt()), [1, 2]);
    expect(sheet.candidates!.suggestedSourceName, 'temperatureInput');
    expect(s.editor.recentTemplates, ['std.source.temperature']);
    // cancelling leaves the project as it was
    store.dispatch(const SourceSheetDismissed());
    expect(store.state.editor.sourceSheet, isNull);
    expect(store.state.revision, before);
    expect(store.state.project!.mappings, isEmpty);

    // an existing concept: only the Source is created, placed where the
    // designer pointed, selected — not opened for renaming
    store.dispatch(const NewSourceRequested(position: Offset(120, 64)));
    await store.until((x) => x.editor.sourceSheet?.ready ?? false);
    store.dispatch(
      const CreateSourceRequested(sourceName: 'roomTemperatureInput', existingConcept: 1),
    );
    s = await store.until((x) => x.editor.pendingRequests == 0 && x.project!.mappings.isNotEmpty);
    expect(s.editor.sourceSheet, isNull);
    expect(s.project!.concepts, hasLength(2), reason: 'no new concept');
    final source = s.project!.mappings.single;
    expect(source.name, 'roomTemperatureInput');
    expect(source.signature.output.toInt(), 1);
    expect(s.editor.layout[NodeRef.mapping(source.id.toInt())], const Offset(120, 64));
    expect(s.editor.selection, MappingSelected(source.id.toInt()));
    expect(s.editor.renaming, isNull);
    final create = daemon.requests.firstWhere((r) => r.hasCreateSource()).createSource;
    expect(create.existingConcept.toInt(), 1);
    expect(create.hasNewConcept(), isFalse);

    // a new concept: both created in one answer; the concept lands where
    // pointed and opens for naming, the Source to its left
    store.dispatch(
      const NewSourceRequested(presetId: 'std.source.temperature', position: Offset(300, 100)),
    );
    await store.until((x) => x.editor.sourceSheet?.ready ?? false);
    store.dispatch(
      CreateSourceRequested(
        sourceName: 'temperatureInput',
        newConceptName: 'Temperature',
        newConceptRepresentation: pb.Representation(quantity: pb.Dim(temperature: 1)),
      ),
    );
    s = await store.until((x) => x.editor.pendingRequests == 0 && x.project!.concepts.length == 3);
    final concept = s.project!.concepts.last.id.toInt();
    final mapping = s.project!.mappings.last.id.toInt();
    expect(s.project!.concepts.last.name, 'Temperature');
    expect(s.editor.layout[NodeRef.concept(concept)], const Offset(300, 100));
    expect(
      s.editor.layout[NodeRef.mapping(mapping)],
      const Offset(300, 100) - const Offset(240, 0),
    );
    expect((s.editor.selection as ConceptSelected).id, concept);
    expect(s.editor.renaming, NodeRef.concept(concept));
    await store.dispose();
  });

  testWidgets('the panel shows Concepts and Sources as sections, localized', (t) async {
    final s = AppState(
      connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
      project: project(),
      library: library(),
    );
    for (final (locale, concepts, sources, sensor) in [
      (const Locale('en'), 'Concepts', 'Sources', 'Temperature Input'),
      (const Locale('zh'), '概念', '来源', '温度输入'),
      (const Locale('ja'), 'コンセプト', '入力元', '温度入力'),
    ]) {
      await t.pumpWidget(
        MaterialApp(
          locale: locale,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: kSupportedLocales,
          home: Scaffold(
            body: SizedBox(
              width: 320,
              child: LibraryPanel(state: s, dispatch: (_) {}),
            ),
          ),
        ),
      );
      await t.pump();
      expect(find.byKey(const ValueKey('library-section-concept')), findsOneWidget);
      expect(find.byKey(const ValueKey('library-section-source')), findsOneWidget);
      // section titles in sentence case, like every panel header
      expect(find.text(concepts), findsOneWidget);
      expect(find.text(sources), findsOneWidget);
      expect(find.text(sensor), findsOneWidget);
      // the row names the value form the preset suggests, never a
      // signature over a concept nobody has chosen
      expect(find.text('K'), findsWidgets);
      expect(find.textContaining('() ->'), findsNothing);
      expect(find.byKey(const ValueKey('library-item-std.source.temperature')), findsOneWidget);
    }
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
