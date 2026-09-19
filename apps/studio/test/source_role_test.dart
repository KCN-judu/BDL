/// The Source role in Studio (ADR-0032): derived from the projection —
/// unresolved, unit domain, at the environment boundary — never stored,
/// never name-based.  The canvas draws it with a non-colour cue and no
/// input sockets, the inspector says *Role Source · Provides · Realization*,
/// the library serves the Sources with localized names, the simulation's
/// inputs are exactly the Sources, and a Source item's relationship
/// lands beside its concept.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/simulation.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/concept_glyphs.dart';
import 'package:bdl_studio/ui/library_panel.dart';
import 'package:bdl_studio/ui/dialogs.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

const tilt = 0, level = 1;
const tiltSensor = 0, pulse = 1, dimByTilt = 2;

pb.ConceptView concept(int id, String name, {int angle = 0}) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(quantity: pb.Dim(angle: angle)),
);

pb.MappingView mapping(int id, String name, List<int> inputs, int output, {String? formula}) =>
    mappingView(
      id: Int64(id),
      name: name,
      signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
      definition: formula == null ? null : pb.Definition(formula: formula),
    );

/// `tiltSensor : () -> Tilt` open (a Source), `pulse : () -> Brightness`
/// resolved by memory (a relationship), `dimByTilt : Tilt -> Brightness`
/// open (a declared relationship).
pb.ProjectProjection design({int revision = 1}) => pb.ProjectProjection(
  revision: Int64(revision),
  name: 'lamp',
  rootPath: '/p',
  concepts: [concept(tilt, 'Tilt', angle: 1), concept(level, 'Brightness')],
  mappings: [
    mapping(tiltSensor, 'tiltSensor', const [], tilt),
    mapping(pulse, 'pulse', const [], level, formula: 'delay(0, 1)'),
    mapping(dimByTilt, 'dimByTilt', const [tilt], level),
  ],
);

pb.MappingView of(pb.ProjectProjection p, int id) =>
    p.mappings.firstWhere((m) => m.id.toInt() == id);

AppState connected(pb.ProjectProjection project, {Selection selection = const NoSelection()}) =>
    AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
      ),
      project: project,
      editor: EditorState(selection: selection),
    );

class Harness extends StatefulWidget {
  const Harness({super.key, required this.initial, required this.child, this.locale});
  final AppState initial;
  final Locale? locale;
  final Widget Function(AppState, void Function(AppAction)) child;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late AppState state = widget.initial;
  final List<Effect> effects = [];
  void dispatch(AppAction a) => setState(() {
    final t = reduce(state, a);
    state = t.state;
    effects.addAll(t.effects);
  });

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    locale: widget.locale,
    supportedLocales: kSupportedLocales,
    localizationsDelegates: const [
      ...AppLocalizations.localizationsDelegates,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    localeResolutionCallback: resolveLocale,
    home: Scaffold(
      body: Align(
        alignment: Alignment.topLeft,
        child: SizedBox(width: 320, height: 1200, child: widget.child(state, dispatch)),
      ),
    ),
  );
}

/// The Temperature Sensor Source item as the daemon serves it: what it
/// creates, in the canonical English; Studio localizes by id.
pb.LibraryItemView sourceItem() => pb.LibraryItemView(
  id: 'std.source.temperature',
  category: 'source',
  displayName: 'Temperature Sensor',
  description: 'The temperature of a room, as the environment provides it.',
  group: 'environment',
  keywords: ['temp', 'source'],
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

pb.LibraryItemView conceptItem() => pb.LibraryItemView(
  id: 'std.environment.humidity',
  category: 'concept',
  displayName: 'Humidity',
  description: 'How damp the air is.',
  group: 'environment',
  creates: [
    pb.LibraryObjectView(
      kind: 'concept',
      key: 'concept',
      name: 'Humidity',
      typeName: 'Scalar',
      representation: pb.Representation(quantity: pb.Dim()),
    ),
  ],
  concept: pb.ConceptTemplateView(
    id: 'std.environment.humidity',
    displayName: 'Humidity',
    defaultName: 'Humidity',
    description: 'How damp the air is.',
    category: 'environment',
    roleHint: pb.RoleHint.ROLE_HINT_INPUT,
    representation: pb.Representation(quantity: pb.Dim()),
  ),
);

final zh = lookupAppLocalizations(const Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans'));
final ja = lookupAppLocalizations(const Locale('ja'));

void main() {
  group('the role is the daemon\'s', () {
    test('stated on every relationship: Source, Rule, Value — never re-derived here', () {
      final p = design();
      expect(relationshipRole(of(p, tiltSensor)), RelationshipRole.source);
      expect(relationshipRole(of(p, pulse)), RelationshipRole.value);
      expect(relationshipRole(of(p, dimByTilt)), RelationshipRole.rule);
      // a view without a role is not this daemon's projection
      final bare = of(p, tiltSensor).deepCopy()..clearRole();
      expect(() => relationshipRole(bare), throwsStateError);
    });

    test('a port-backed Source in an open component is not a Source where the designer is', () {
      final p = design();
      final s = connected(p).copyWith(
        editor: connected(p).editor.copyWith(context: const ComponentContext(3)),
        system: pb.SystemView(
          revision: Int64(1),
          name: 'lamp',
          base: pb.ProjectProjection(revision: Int64(1), name: 'lamp'),
          components: [
            pb.ComponentView(
              id: Int64(3),
              name: 'Probe',
              body: p,
              ports: [
                pb.PortView(
                  id: Int64(1),
                  name: 'tiltSensor',
                  kind: pb.PortKind.PORT_KIND_REQUIRED,
                  decl: Int64(tiltSensor),
                ),
              ],
            ),
          ],
        ),
      );
      // the role stays Source (the port's binding provides it); the
      // surface wears the port instead
      expect(relationshipRole(of(p, tiltSensor)), RelationshipRole.source);
      expect(s.backsPort(of(p, tiltSensor)), isTrue);
      expect(s.isSource(of(p, tiltSensor)), isFalse);
      expect(s.isDeclared(of(p, tiltSensor)), isFalse, reason: 'nothing is missing');
    });

    test('a state never changes it: renaming, a definition, an invalid one', () {
      final p = design();
      final renamed = of(p, tiltSensor).deepCopy()..name = 'Whatever';
      expect(relationshipRole(renamed), RelationshipRole.source);
      final defined = mappingView(
        id: Int64(tiltSensor),
        name: 'tiltSensor',
        signature: pb.Signature(output: Int64(tilt)),
        definition: pb.Definition(formula: 'true + 1'),
      );
      expect(relationshipRole(defined), RelationshipRole.value, reason: 'invalid is a state');
      final sensorByName = of(p, dimByTilt).deepCopy()..name = 'TempSensor';
      expect(relationshipRole(sensorByName), RelationshipRole.rule);
    });

    test('declared is a rule with no formula; a Source and a value are complete', () {
      final s = connected(design());
      expect(s.isDeclared(of(design(), dimByTilt)), isTrue);
      expect(s.isDeclared(of(design(), tiltSensor)), isFalse);
      expect(s.isDeclared(of(design(), pulse)), isFalse);
    });

    test('where the designer is: the system states a bound base relationship as a Value', () {
      // the daemon's system view already carries the Value role for a base
      // relationship a binding realises (`base_projection`); Studio reads
      // it and says *bound to*, it does not decide it
      final bound = design()
        ..mappings.removeWhere((m) => m.id.toInt() == tiltSensor)
        ..mappings.add(
          mappingView(
            id: Int64(tiltSensor),
            name: 'tiltSensor',
            signature: pb.Signature(output: Int64(tilt)),
            role: pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE,
          ),
        );
      final s = connected(bound).copyWith(
        system: pb.SystemView(
          revision: Int64(1),
          name: 'lamp',
          base: bound,
          bindings: [
            pb.BindingView(
              id: Int64(7),
              source: pb.PortRefView(baseDecl: Int64(pulse)),
              destination: pb.PortRefView(baseDecl: Int64(tiltSensor)),
            ),
          ],
        ),
      );
      expect(s.isSource(of(bound, tiltSensor)), isFalse);
      expect(s.realisedByBinding(of(bound, tiltSensor)), isTrue);
      expect(connected(design()).isSource(of(design(), tiltSensor)), isTrue);
      expect(connected(design()).isSource(of(design(), pulse)), isFalse);
    });
  });

  group('canvas', () {
    test('a Source node: no input sockets, one output socket, the source cue, not declared', () {
      final scene = buildScene(design(), const {});
      final src = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tiltSensor));
      expect(src.source, isTrue);
      expect(src.declared, isFalse, reason: 'nothing is missing');
      expect(src.sockets.where((s) => s.ref.side == SocketSide.input), isEmpty);
      expect(src.sockets.where((s) => s.ref.side == SocketSide.output), hasLength(1));
      // the unit is never a port: nothing is labelled "()"
      expect(src.socketLabels.values, isNot(contains('()')));
      expect(src.definition, isNull);

      final resolved = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(pulse));
      expect(resolved.source, isFalse);
      expect(resolved.declared, isFalse);
      final declared = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(dimByTilt));
      expect(declared.source, isFalse);
      expect(declared.declared, isTrue);
    });

    test('inside a component a port-backed () -> A keeps its port role on the canvas', () {
      final scene = buildScene(
        design(),
        const {},
        system: const SystemSceneInput(portWords: {tiltSensor: 'requires'}),
      );
      final n = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tiltSensor));
      expect(n.source, isFalse);
      expect(n.headerWord, 'requires');
    });

    test('on the system canvas a Source has no realise socket until something could bind it', () {
      // tiltSensor is the only producer of Tilt and there is no instance:
      // nothing could realise it, so its left edge stays clear.
      final alone = pb.SystemView(revision: Int64(1), name: 'lamp', base: design());
      var scene = buildScene(design(), const {}, system: SystemSceneInput(system: alone));
      var n = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tiltSensor));
      expect(n.source, isTrue);
      expect(n.sockets.where((s) => s.ref.side == SocketSide.input), isEmpty);
      // another producer of Tilt: the binding affordance appears, hollow
      final p = design()..mappings.add(mapping(9, 'tiltFromA', const [], tilt, formula: '1 deg'));
      final withProducer = pb.SystemView(revision: Int64(1), name: 'lamp', base: p);
      scene = buildScene(p, const {}, system: SystemSceneInput(system: withProducer));
      n = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tiltSensor));
      expect(n.source, isTrue, reason: 'still a Source until bound');
      final realise = n.sockets.where((s) => s.ref.role == SocketRole.realise).toList();
      expect(realise, hasLength(1));
      expect(realise.single.open, isTrue);
    });

    test('a base relationship a binding realises is shown as realised, not as a Source', () {
      // the system view states the bound base relationship as a Value
      // (`base_projection`); the canvas draws the binding as its definition
      final base = design()
        ..mappings.removeWhere((m) => m.id.toInt() == tiltSensor)
        ..mappings.add(
          mappingView(
            id: Int64(tiltSensor),
            name: 'tiltSensor',
            signature: pb.Signature(output: Int64(tilt)),
            role: pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE,
          ),
        );
      final sys = pb.SystemView(
        revision: Int64(1),
        name: 'lamp',
        base: base,
        bindings: [
          pb.BindingView(
            id: Int64(7),
            source: pb.PortRefView(baseDecl: Int64(pulse)),
            destination: pb.PortRefView(baseDecl: Int64(tiltSensor)),
          ),
        ],
      );
      final scene = buildScene(base, const {}, system: SystemSceneInput(system: sys));
      final n = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tiltSensor));
      expect(n.source, isFalse);
      expect(n.definition, '= pulse');
    });
  });

  group('inspector', () {
    testWidgets('a Source: Role Source, Provides, Realization by the environment', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(design(), selection: const MappingSelected(tiltSensor)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Role'), findsOneWidget);
      expect(find.text('Source'), findsWidgets);
      expect(find.text(kEnglish.sourceExplanation), findsOneWidget);
      expect(find.text('Provides'), findsOneWidget);
      expect(find.text('Produces'), findsNothing);
      expect(find.text('Realization'), findsOneWidget);
      expect(find.text(kEnglish.realizationEnvironment), findsOneWidget);
      expect(find.text('declared'), findsNothing);
      expect(find.textContaining('hardware'), findsNothing);
      expect(find.textContaining('calls'), findsNothing);
    });

    testWidgets('a resolved () -> A is a relationship: no Source, no realization row', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(design(), selection: const MappingSelected(pulse)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Source'), findsNothing);
      expect(find.text('Relationship'), findsWidgets);
      expect(find.text('Produces'), findsOneWidget);
      expect(find.text('Realization'), findsNothing);
    });

    testWidgets('the locale changes the words, never the role or the identifier', (t) async {
      await t.pumpWidget(
        Harness(
          locale: const Locale.fromSubtags(languageCode: 'zh', scriptCode: 'Hans'),
          initial: connected(design(), selection: const MappingSelected(tiltSensor)),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text(zh.roleSource), findsWidgets);
      expect(find.text(zh.realizationEnvironment), findsOneWidget);
      expect(find.text('Source'), findsNothing);
      expect(find.text('tiltSensor'), findsOneWidget);
    });
  });

  group('library', () {
    test('the section is Sources, in each locale', () {
      expect(categoryTitle(kEnglish, 'source'), 'Sources');
      expect(categoryTitle(zh, 'source'), '来源');
      expect(categoryTitle(ja, 'source'), '入力元');
      expect(categoryTitle(kEnglish, 'concept'), 'Concepts');
    });

    test('an item\'s text follows the locale; what it creates does not', () {
      final t = sourceItem();
      expect(t.category, 'source');
      expect(conceptItem().category, 'concept');
      expect(itemName(kEnglish, t), 'Temperature Sensor');
      expect(itemName(zh, t), '温度传感器');
      expect(itemName(ja, t), '温度センサー');
      expect(itemDescription(ja, t), isNot(t.description));
      for (final l in [kEnglish, zh, ja]) {
        expect(t.creates.first.name, 'RoomTemp');
        expect(t.creates.last.name, 'TempSensor');
        expect(itemWord(l, t), '() -> RoomTemp');
        expect(itemPreview(l, t).join(' '), contains('RoomTemp (Temperature)'));
      }
      // an item the catalog does not know keeps the daemon's English
      final foreign = pb.LibraryItemView(id: 'team.x', category: 'source', displayName: 'X Sensor');
      expect(itemName(ja, foreign), 'X Sensor');
    });

    test('search finds a Source by its localized name, its tags and its relationship name', () {
      final all = [sourceItem(), conceptItem()];
      List<String> ids(String q, AppLocalizations l) =>
          searchItems(all, q, l10n: l).map((t) => t.id).toList();
      expect(ids('温度', zh), ['std.source.temperature']);
      expect(ids('传感器', zh), ['std.source.temperature']);
      expect(ids('温度', ja), ['std.source.temperature']);
      expect(ids('センサー', ja), ['std.source.temperature']);
      expect(ids('TempSensor', kEnglish), ['std.source.temperature']);
      expect(ids('Sources', kEnglish), ['std.source.temperature']);
      expect(ids('来源', zh), ['std.source.temperature']);
      expect(ids('humid', kEnglish), ['std.environment.humidity']);
    });

    testWidgets('the row glyph names a Source for assistive technology', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(design()),
          child: (s, d) => const MappingGlyph(declared: false, wrong: false, source: true),
        ),
      );
      expect(find.bySemanticsLabel('Source'), findsOneWidget);
    });
  });

  group('simulation', () {
    test('the inputs are exactly the Sources: resolved and non-unit relationships are not', () {
      final inputs = simulationInputs(design()).map((m) => m.id.toInt()).toList();
      expect(inputs, [tiltSensor]);
    });
  });

  group('reducer', () {
    test('a Source item\'s relationship lands to the left of its concept', () {
      final s = connected(design()).copyWith(
        editor: const EditorState(
          pendingRequests: 1,
          pendingInsert: PendingInsert(
            templateId: 'std.source.temperature',
            position: Offset(300, 100),
          ),
        ),
      );
      final next = design(revision: 2)
        ..concepts.add(concept(10, 'RoomTemp'))
        ..mappings.add(mapping(11, 'TempSensor', const [], 10));
      final t = reduce(
        s,
        ProjectReceived(
          next,
          outcome: pb.EditOutcome(createdConcept: Int64(10), createdMapping: Int64(11)),
        ),
      );
      expect(t.state.editor.layout[const NodeRef.concept(10)], const Offset(300, 100));
      expect(t.state.editor.layout[const NodeRef.mapping(11)], const Offset(60, 100));
      expect(t.state.editor.renaming, const NodeRef.concept(10));
      expect(relationshipRole(of(next, 11)), RelationshipRole.source);
    });
  });

  group('new source sheet', () {
    testWidgets('no Reads row; Provides is chosen; the result has no inputs', (t) async {
      late BuildContext ctx;
      await t.pumpWidget(
        MaterialApp(
          theme: macTheme(Brightness.light),
          home: Builder(
            builder: (c) {
              ctx = c;
              return const Scaffold();
            },
          ),
        ),
      );
      final result = showNewSourceSheet(ctx, design().concepts);
      await t.pumpAndSettle();
      expect(find.text('New source'), findsOneWidget);
      expect(find.text('Reads'), findsNothing);
      expect(find.text('Provides'), findsOneWidget);
      expect(find.text(kEnglish.chooseWhatItProvidesTheOutputSocket), findsOneWidget);
      await t.enterText(find.byType(EditableText).first, 'AmbientLight');
      await t.tap(find.text('choose'));
      await t.pumpAndSettle();
      await t.tap(find.text('Brightness').last);
      await t.pumpAndSettle();
      expect(find.text(kEnglish.sheetSourceShape('Brightness')), findsOneWidget);
      await t.tap(find.text('Create'));
      await t.pumpAndSettle();
      final r = await result;
      expect(r, isNotNull);
      expect(r!.name, 'AmbientLight');
      expect(r.inputs, isEmpty);
      expect(r.output, level);
    });
  });
}
