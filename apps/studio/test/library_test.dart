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

/// The Temperature Sensor Source item: a value and its `() -> Value`
/// relationship.
final temperatureSensor = pb.LibraryItemView(
  id: 'std.source.temperature',
  category: 'source',
  displayName: 'Temperature Sensor',
  description: 'A temperature the product measures.',
  group: 'environment',
  keywords: ['temp', 'thermal', 'sensor'],
  creates: [
    pb.LibraryObjectView(
      kind: 'concept',
      key: 'value',
      name: 'RoomTemp',
      typeName: 'Temperature',
      representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
      unit: 'K',
    ),
    pb.LibraryObjectView(
      kind: 'mapping',
      key: 'source',
      name: 'TempSensor',
      signature: '() -> RoomTemp',
    ),
  ],
);

/// The External Value Source item: a value whose form is left open.
final externalValue = pb.LibraryItemView(
  id: 'std.source.external',
  category: 'source',
  displayName: 'External Value',
  description: 'A value provided from outside the product.',
  group: 'external',
  keywords: ['host', 'network', 'external', 'source'],
  creates: [
    pb.LibraryObjectView(kind: 'concept', key: 'value', name: 'ExternalValue'),
    pb.LibraryObjectView(
      kind: 'mapping',
      key: 'source',
      name: 'ExternalSource',
      signature: '() -> ExternalValue',
    ),
  ],
);

pb.LibraryItemsResponse library() => pb.LibraryItemsResponse(
  libraries: [
    pb.LibraryView(
      id: 'std',
      name: 'Standard',
      schemaVersion: 2,
      version: '0.2',
      items: [
        externalValue,

        for (final t in [temperature, light, motorSpeed, motorAngle, pressed, open]) conceptItem(t),
        temperatureSensor,
      ],
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
                pb.MappingView(
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
    expect(ids('TempSensor'), ['std.source.temperature']);
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
    expect(ids(zh, 'Temperature Sensor'), ['std.source.temperature']);
    expect(ids(ja, 'lux'), ['std.environment.ambient_light']);
    // the required cases, per locale: a name, a keyword, a tag
    expect(ids(kEnglish, 'temperature'), contains('std.source.temperature'));
    expect(ids(kEnglish, 'sensor'), contains('std.source.temperature'));
    expect(ids(kEnglish, 'external'), ['std.source.external']);
    expect(ids(zh, '外部'), ['std.source.external']);
    expect(ids(ja, '外部'), ['std.source.external']);
    expect(ids(ja, 'センサー'), isNot(contains('std.source.external')));
    // an open value form is said in the sheet's words, never a presumed scalar
    expect(itemPreview(kEnglish, externalValue).first, 'value: ExternalValue (decide later)');
    expect(
      itemPreview(zh, externalValue).first,
      contains('ExternalValue (${zh.decideLaterLower})'),
    );
    expect(itemWord(kEnglish, externalValue), '() -> ExternalValue');
    // names and descriptions in each locale; the section titles
    expect(itemName(kEnglish, temperatureSensor), 'Temperature Sensor');
    expect(itemName(zh, temperatureSensor), '温度传感器');
    expect(itemName(ja, temperatureSensor), '温度センサー');
    expect(itemDescription(zh, temperatureSensor), isNot(temperatureSensor.description));
    expect(itemDescription(ja, temperatureSensor), isNotEmpty);
    expect(categoryTitle(kEnglish, 'source'), 'Sources');
    expect(categoryTitle(zh, 'source'), '来源');
    expect(categoryTitle(ja, 'source'), '入力元');
    expect(categoryTitle(kEnglish, 'concept'), 'Concepts');
    // what an item creates is the daemon's: identifiers and types alike
    for (final l in [kEnglish, zh, ja]) {
      expect(itemPreview(l, temperatureSensor).join(' '), contains('RoomTemp (Temperature)'));
      expect(itemPreview(l, temperatureSensor).join(' '), contains('TempSensor'));
      expect(itemPreview(l, temperatureSensor).join(' '), contains('() -> RoomTemp'));
      expect(itemWord(l, temperatureSensor), '() -> RoomTemp');
    }
    // an item the catalog does not know keeps its canonical English
    final foreign = pb.LibraryItemView(id: 'team.x', category: 'concept', displayName: 'X Thing');
    expect(itemName(ja, foreign), 'X Thing');
  });

  test('a Source item places its concept where dropped and its relationship to its left', () async {
    final (store, daemon) = await connected();
    store.dispatch(
      const InsertLibraryItemRequested('std.source.temperature', position: Offset(120, 64)),
    );
    final s = await store.until((x) => x.editor.pendingRequests == 0);
    final concept = s.project!.concepts.single.id.toInt();
    final mapping = s.project!.mappings.single.id.toInt();
    expect(s.project!.concepts.single.name, 'RoomTemp');
    expect(s.project!.mappings.single.name, 'TempSensor');
    expect(s.editor.layout[NodeRef.concept(concept)], const Offset(120, 64));
    // the Source lands to the concept's left: environment → Source → behavior
    expect(s.editor.layout[NodeRef.mapping(mapping)], const Offset(120, 64) - const Offset(240, 0));
    expect((s.editor.selection as ConceptSelected).id, concept);
    expect(s.editor.renaming, NodeRef.concept(concept));
    expect(s.editor.recentTemplates, ['std.source.temperature']);
    final e = daemon.requests.firstWhere((r) => r.hasInstantiateLibraryItem());
    expect(e.instantiateLibraryItem.itemId, 'std.source.temperature');
    await store.dispose();
  });

  testWidgets('the panel shows Concepts and Sources as sections, localized', (t) async {
    final s = AppState(
      connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
      project: project(),
      library: library(),
    );
    for (final (locale, concepts, sources, sensor) in [
      (const Locale('en'), 'CONCEPTS', 'SOURCES', 'Temperature Sensor'),
      (const Locale('zh'), '概念', '来源', '温度传感器'),
      (const Locale('ja'), 'コンセプト', '入力元', '温度センサー'),
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
      expect(find.text(concepts.toUpperCase()), findsOneWidget);
      expect(find.text(sources.toUpperCase()), findsOneWidget);
      expect(find.text(sensor), findsOneWidget);
      expect(find.text('() -> RoomTemp'), findsOneWidget);
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
