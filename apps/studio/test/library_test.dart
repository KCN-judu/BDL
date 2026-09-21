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

/// The value categories of the standard library (ADR-0041), a few of
/// them: two quantities, a level, a value form, and *decide later*.
final temperature = tpl(
  'std.quantity.temperature',
  'Temperature',
  category: 'quantity',
  representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
  unit: 'K',
  keywords: ['temp', 'heat'],
);
final light = tpl(
  'std.quantity.illuminance',
  'Illuminance',
  category: 'quantity',
  representation: pb.Representation(quantity: pb.Dim(luminous: 1, angle: 2, length: -2)),
  unit: 'lx',
  keywords: ['light', 'lux'],
);
final level = tpl(
  'std.value.level',
  'Level',
  category: 'form',
  representation: pb.Representation(quantity: pb.Dim()),
  keywords: ['ratio', 'brightness', 'motor speed'],
);
final angle = tpl(
  'std.quantity.angle',
  'Angle',
  category: 'quantity',
  representation: pb.Representation(quantity: pb.Dim(angle: 1)),
  unit: 'rad',
  keywords: ['rotation', 'tilt', 'servo', 'motor'],
);
final pressed = tpl(
  'std.value.boolean',
  'On / off',
  category: 'form',
  representation: pb.Representation(boolean: pb.Unit()),
  keywords: ['button', 'pressed'],
);
final open = tpl('std.value.open', 'Decide later', category: 'form');
final teamThing = tpl('team.x.open', 'Open Thing', category: 'x');

/// The quantity vocabulary as served: dimension and display symbol; and
/// the compiler's value categories with the units each is measured in.
final quantities = [
  pb.QuantityView(id: 'angle', typeName: 'Angle', unit: 'rad', dim: pb.Dim(angle: 1)),
  pb.QuantityView(
    id: 'temperature',
    typeName: 'Temperature',
    unit: 'K',
    dim: pb.Dim(temperature: 1),
  ),
  pb.QuantityView(
    id: 'illuminance',
    typeName: 'Illuminance',
    unit: 'lx',
    dim: pb.Dim(luminous: 1, angle: 2, length: -2),
  ),
  pb.QuantityView(
    id: 'angular_velocity',
    typeName: 'AngularVelocity',
    unit: 'rad/s',
    dim: pb.Dim(angle: 1, time: -1),
  ),
];

pb.UnitExprView unitOf(String source, String display, pb.Dim dim) =>
    pb.UnitExprView(source: source, display: display, dim: dim);

final categories = [
  pb.ValueCategoryView(
    id: 'angle',
    typeName: 'Angle',
    dim: pb.Dim(angle: 1),
    preferredUnit: unitOf('rad', 'rad', pb.Dim(angle: 1)),
    units: [
      unitOf('rad', 'rad', pb.Dim(angle: 1)),
      unitOf('deg', 'deg', pb.Dim(angle: 1)),
      unitOf('turn', 'turn', pb.Dim(angle: 1)),
    ],
  ),
  pb.ValueCategoryView(
    id: 'angular_velocity',
    typeName: 'AngularVelocity',
    dim: pb.Dim(angle: 1, time: -1),
    preferredUnit: unitOf('rad per s', 'rad/s', pb.Dim(angle: 1, time: -1)),
    units: [
      unitOf('rad per s', 'rad/s', pb.Dim(angle: 1, time: -1)),
      unitOf('deg per s', 'deg/s', pb.Dim(angle: 1, time: -1)),
    ],
  ),
];

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
  id: 'team.source.temperature',
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
  id: 'team.source.external',
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

/// The served libraries: the standard value categories, and a team
/// library that ships Source presets (the standard one ships none).
pb.LibraryItemsResponse library() => pb.LibraryItemsResponse(
  libraries: [
    pb.LibraryView(
      id: 'std',
      name: 'Standard',
      schemaVersion: 2,
      version: '0.3',
      items: [
        for (final t in [pressed, level, open, angle, temperature, light]) conceptItem(t),
      ],
    ),
    pb.LibraryView(
      id: 'team',
      name: 'Team',
      schemaVersion: 2,
      version: '0.1',
      items: [externalInput, conceptItem(teamThing), temperatureInput],
    ),
  ],
  quantities: quantities,
);

/// Every item of every served library, in order.
List<pb.LibraryItemView> allItems() => [for (final l in library().libraries) ...l.items];

pb.ProjectProjection project({int revision = 1, List<pb.ConceptView> concepts = const []}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll(concepts);

/// A connected store holding an empty project at revision 1, whose daemon
/// answers the library and instantiations.
Future<(TestStore, FakeDaemon)> connected({List<pb.ConceptView> concepts = const []}) async {
  concepts = [...concepts];
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
      final item = allItems().firstWhere((i) => i.id == r.itemId);
      // a Source item: the concept and the relationship, in one answer
      final mapping = item.creates.where((o) => o.kind == 'mapping').firstOrNull;
      final mappingId = mapping == null ? null : nextId++;
      final value = item.creates.first;
      concepts.add(pb.ConceptView(id: Int64(id), name: value.name));
      return pb.Response(
        editApplied: pb.EditApplied(
          project: project(revision: revision, concepts: concepts)
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
      final item = r.itemId.isEmpty ? null : allItems().firstWhere((i) => i.id == r.itemId);
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
    expect(store.state.templates.map((t) => t.id), contains('std.quantity.temperature'));
    expect(store.state.library!.libraries.first.version, '0.3');
    expect(store.state.libraryItems.where((i) => i.category == 'source').map((i) => i.id), [
      'team.source.external',
      'team.source.temperature',
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
      const InsertLibraryItemRequested('std.quantity.temperature', position: Offset(100, 40)),
    );
    final fromDrag = reduce(
      s,
      const InsertLibraryItemRequested('std.quantity.temperature', position: Offset(300, 80)),
    );
    final fromPanel = reduce(s, const InsertLibraryItemRequested('std.quantity.temperature'));
    for (final t in [fromMenu, fromDrag, fromPanel]) {
      expect(t.effects, hasLength(1));
      final e = t.effects.single as InstantiateLibraryItem;
      expect(e.itemId, 'std.quantity.temperature');
      expect(e.baseRevision, 1);
      expect(t.state.editor.pendingRequests, 1);
      expect(t.state.editor.recentTemplates, ['std.quantity.temperature']);
    }
    expect(fromMenu.state.editor.pendingInsert!.position, const Offset(100, 40));
    expect(fromDrag.state.editor.pendingInsert!.position, const Offset(300, 80));
    expect(fromPanel.state.editor.pendingInsert!.position, isNull);
    // A second insertion waits for the first to be answered.
    expect(
      reduce(fromMenu.state, const InsertLibraryItemRequested('std.quantity.angle')).effects,
      isEmpty,
    );
  });

  test('create-then-rename: the item makes the template, a block of it follows, placed, '
      'selected and open for naming', () async {
    final (store, daemon) = await connected();
    store.dispatch(
      const InsertLibraryItemRequested('std.quantity.temperature', position: Offset(120, 64)),
    );
    final s = await store.until(
      (x) => x.editor.pendingRequests == 0 && x.project!.mappings.length == 1,
    );
    final concept = s.project!.concepts.single.id.toInt();
    final id = s.project!.mappings.single.id.toInt();
    expect(s.project!.concepts.single.name, 'Temperature');
    expect(s.project!.mappings.single.name, 'temperature');
    expect((s.editor.selection as MappingSelected).id, id);
    expect(s.editor.renaming, NodeRef.mapping(id));
    expect(s.editor.layout[NodeRef.mapping(id)], const Offset(120, 64));
    expect(s.editor.layout[NodeRef.concept(concept)], isNull, reason: 'a template has no place');
    expect(s.editor.pendingInsert, isNull);
    // The position went to the daemon as layout (never a revision).
    await store.until((_) => daemon.requests.any((r) => r.hasSetLayout()));
    final layout = daemon.requests.firstWhere((r) => r.hasSetLayout()).setLayout.layout;
    expect(layout.mappings.single.id.toInt(), id);
    expect(layout.mappings.single.x, 120);

    // Typing the product's name commits one ordinary rename edit.
    daemon.requests.clear();
    store.dispatch(InlineRenameFinished(NodeRef.mapping(id), name: 'motorTemperature'));
    expect(store.state.editor.renaming, isNull);
    await store.until((x) => x.editor.pendingRequests == 0);
    final rename = daemon.requests.single.applyEdit.op.renameMapping;
    expect(rename.id.toInt(), id);
    expect(rename.name, 'motorTemperature');

    // Esc, an empty name, or the same name change nothing.
    for (final name in [null, '', '  ', 'temperature']) {
      store.dispatch(InlineRenameStarted(NodeRef.mapping(id)));
      expect(store.state.editor.renaming, NodeRef.mapping(id));
      daemon.requests.clear();
      store.dispatch(InlineRenameFinished(NodeRef.mapping(id), name: name));
      expect(store.state.editor.renaming, isNull);
      expect(daemon.requests, isEmpty, reason: 'name=$name');
    }
    await store.dispose();
  });

  test(
    'an insertion the panel started is the template alone: no block, no layout, selected',
    () async {
      final (store, daemon) = await connected();
      store.dispatch(const InsertLibraryItemRequested('std.value.boolean'));
      final s = await store.until((x) => x.editor.pendingRequests == 0);
      final id = s.project!.concepts.single.id.toInt();
      expect((s.editor.selection as ConceptSelected).id, id);
      expect(s.project!.mappings, isEmpty, reason: 'no point, no block');
      expect(s.editor.renaming, isNull, reason: 'a template is not on the canvas');
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
    store.dispatch(const InsertLibraryItemRequested('std.quantity.temperature'));
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

  test('search matches name, keywords, units, dimension words and category', () {
    final all = allItems();
    List<String> ids(String q) => searchItems(
      all,
      q,
      quantities: quantities,
      categories: categories,
    ).map((t) => t.id).toList();
    // a category by its name, its synonyms, the row's unit
    expect(ids('lux'), ['std.quantity.illuminance']);
    expect(ids('LX'), ['std.quantity.illuminance']);
    expect(ids('illuminance'), ['std.quantity.illuminance']);
    expect(ids('motor'), ['std.value.level', 'std.quantity.angle']);
    expect(ids('rotation'), ['std.quantity.angle']);
    expect(ids('Quantities'), [
      'std.quantity.angle',
      'std.quantity.temperature',
      'std.quantity.illuminance',
    ]);
    expect(ids('heat'), contains('std.quantity.temperature'));
    // every registered unit of the category's dimension, and its type
    // name — the compiler's, served with the vocabulary
    expect(ids('deg'), ['std.quantity.angle']);
    expect(ids('turn'), ['std.quantity.angle']);
    expect(ids('Angle'), ['std.quantity.angle']);
    // a served library's Source presets, by section and relationship name
    expect(ids('sensor'), contains('team.source.temperature'));
    expect(ids('temperatureInput'), ['team.source.temperature']);
    expect(ids(''), hasLength(all.length));
    expect(ids('zzz'), isEmpty);
    final concepts = allItems().where((i) => i.hasConcept()).map((i) => i.concept);
    expect(
      searchTemplates(concepts, 'heat').map((t) => t.id),
      contains('std.quantity.temperature'),
    );
  });

  test('search and names follow the locale; what is created does not', () {
    final all = allItems();
    final zh = lookupAppLocalizations(const Locale('zh'));
    final ja = lookupAppLocalizations(const Locale('ja'));
    List<String> ids(AppLocalizations l, String q) => searchItems(
      all,
      q,
      l10n: l,
      quantities: quantities,
      categories: categories,
    ).map((t) => t.id).toList();
    // the localized name and tags of a standard category
    expect(ids(zh, '温度'), ['std.quantity.temperature']);
    expect(ids(ja, '温度'), ['std.quantity.temperature']);
    expect(ids(zh, '角度'), ['std.quantity.angle']);
    expect(ids(ja, '回転'), ['std.quantity.angle']);
    expect(ids(zh, '照度'), ['std.quantity.illuminance']);
    // the canonical English, the units and the type names match in
    // every locale
    expect(ids(zh, 'Temperature'), ['std.quantity.temperature', 'team.source.temperature']);
    expect(ids(ja, 'lux'), ['std.quantity.illuminance']);
    expect(ids(ja, 'deg'), ['std.quantity.angle']);
    // a team library's items keep their canonical English (no catalog)
    expect(ids(kEnglish, 'sensor'), contains('team.source.temperature'));
    expect(ids(kEnglish, 'external'), contains('team.source.external'));
    expect(ids(zh, 'Temperature Input'), ['team.source.temperature']);
    // a Source item is a preset: its row says the value form it suggests
    // — never a signature over a concept nobody has chosen — and an open
    // one says so in the sheet's words
    expect(itemWord(kEnglish, temperatureInput), 'K');
    expect(itemWord(kEnglish, externalInput), 'decide later');
    expect(itemPreview(kEnglish, externalInput), isEmpty);
    expect(sourceItemHover(kEnglish, temperatureInput), contains('Temperature'));
    expect(sourceItemHover(kEnglish, temperatureInput), isNot(contains('() ->')));
    expect(sourceItemHover(kEnglish, externalInput), kEnglish.inputForAConcept);
    // names and descriptions of a standard category in each locale; a
    // foreign item's fall back to the daemon's English; the section titles
    expect(itemName(kEnglish, conceptItem(temperature)), 'Temperature');
    expect(itemName(zh, conceptItem(temperature)), '温度');
    expect(itemName(ja, conceptItem(temperature)), '温度');
    expect(itemName(zh, conceptItem(angle)), '角度');
    expect(itemDescription(zh, conceptItem(temperature)), isNot(temperature.description));
    expect(itemDescription(ja, conceptItem(temperature)), isNotEmpty);
    expect(itemName(ja, temperatureInput), 'Temperature Input');
    expect(categoryTitle(kEnglish, 'source'), 'Sources');
    expect(categoryTitle(zh, 'source'), '来源');
    expect(categoryTitle(ja, 'source'), '入力元');
    expect(categoryLabel(zh, 'quantity'), '物理量');
    expect(categoryLabel(ja, 'form'), '値');
    // what a preset suggests is the daemon's: identifiers and units alike
    for (final l in [kEnglish, zh, ja]) {
      expect(temperatureInput.preset.conceptName, 'Temperature');
      expect(temperatureInput.preset.sourceName, 'temperatureInput');
      expect(itemWord(l, temperatureInput), 'K');
      expect(conceptItem(temperature).creates.single.name, 'Temperature');
    }
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
      const NewSourceRequested(presetId: 'team.source.temperature', position: Offset(120, 64)),
    );
    var s = await store.until((x) => x.editor.sourceSheet?.ready ?? false);
    expect(s.revision, before, reason: 'nothing committed');
    expect(daemon.requests.where((r) => r.hasCreateSource()), isEmpty);
    expect(daemon.requests.where((r) => r.hasInstantiateLibraryItem()), isEmpty);
    final sheet = s.editor.sourceSheet!;
    expect(sheet.presetId, 'team.source.temperature');
    expect(sheet.candidates!.candidates.map((c) => c.conceptId.toInt()), [1, 2]);
    expect(sheet.candidates!.suggestedSourceName, 'temperatureInput');
    expect(s.editor.recentTemplates, ['team.source.temperature']);
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
      const NewSourceRequested(presetId: 'team.source.temperature', position: Offset(300, 100)),
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
    // the Sem block lands where the designer pointed and is selected; the
    // concept is its template and takes no place (ADR-0043)
    expect(s.editor.layout[NodeRef.mapping(mapping)], const Offset(300, 100));
    expect(s.editor.layout[NodeRef.concept(concept)], isNull);
    expect((s.editor.selection as MappingSelected).id, mapping);
    // named on the sheet: not opened for renaming
    expect(s.editor.renaming, isNull);
    await store.dispose();
  });

  testWidgets('the panel shows Values, Quantities and Sources as sections, localized', (t) async {
    final s = AppState(
      connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
      project: project(),
      library: library(),
    );
    for (final (locale, values, quantities, sources, temperature, source) in [
      (const Locale('en'), 'Values', 'Quantities', 'Sources', 'Temperature', 'Source'),
      (const Locale('zh'), '值', '物理量', '来源', '温度', '来源'),
      (const Locale('ja'), '値', '物理量', '入力元', '温度', 'Source'),
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
      // the standard library's two groups are the Values and Quantities
      // sections; the generic Source row is its own section; a team
      // library keeps its own sections and groups
      expect(find.byKey(const ValueKey('library-section-form')), findsOneWidget);
      expect(find.byKey(const ValueKey('library-section-quantity')), findsOneWidget);
      expect(find.byKey(const ValueKey('library-section-source')), findsOneWidget);
      expect(find.byKey(const ValueKey('library-section-other-source')), findsOneWidget);
      expect(find.byKey(const ValueKey('library-section-other-concept')), findsOneWidget);
      // section titles in sentence case, like every panel header
      expect(find.text(values), findsOneWidget);
      expect(find.text(quantities), findsOneWidget);
      expect(find.text(sources), findsWidgets);
      expect(find.text(temperature), findsWidgets);
      expect(find.byKey(const ValueKey('library-item-source')), findsOneWidget);
      expect(find.text(source), findsWidgets);
      // a category's row says its unit; a preset's row the value form it
      // suggests — never a signature over a concept nobody has chosen
      expect(find.text('K'), findsWidgets);
      expect(find.text('rad'), findsOneWidget);
      expect(find.textContaining('() ->'), findsNothing);
      expect(find.byKey(const ValueKey('library-item-std.quantity.angle')), findsOneWidget);
      expect(find.byKey(const ValueKey('library-item-team.source.temperature')), findsOneWidget);
      // no product-named concept anywhere: categories only
      expect(find.text('Motor Angle'), findsNothing);
      expect(find.text('Temperature Input'), findsOneWidget, reason: 'the team preset, English');
    }
  });

  test('rows say what a value is measured as in the contract\'s words', () {
    expect(representationWord(kEnglish, temperature), 'K');
    expect(representationWord(kEnglish, level), 'no unit');
    expect(representationWord(kEnglish, pressed), 'on–off');
    expect(representationWord(kEnglish, open), 'decide later');
    // a composite dimension the registry has no symbol for: the
    // vocabulary's display symbol, read off the served quantities
    final omega = tpl(
      'std.quantity.angular_velocity',
      'Angular velocity',
      category: 'quantity',
      representation: pb.Representation(quantity: pb.Dim(angle: 1, time: -1)),
    );
    expect(representationWord(kEnglish, omega), 'no unit');
    expect(representationWord(kEnglish, omega, quantities), 'rad/s');
    expect(categoryLabel(kEnglish, 'form'), 'Values');
    expect(categoryLabel(kEnglish, 'quantity'), 'Quantities');
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
