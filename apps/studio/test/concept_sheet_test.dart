/// The concept sheet (ADR-0041): a concept is created from a value
/// category and a name given before anything is created.  The name is
/// required and checked as a hint (an identifier, not taken); the category
/// is the library item's, changeable; the units are the compiler's for the
/// category's dimension — a fact, never a choice; Create is one ordinary
/// edit that lands where asked, selected and named; Cancel commits nothing.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/concept_sheet.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

pb.LibraryItemView category(
  String id,
  String name, {
  pb.Representation? representation,
  String unit = '',
  String typeName = '',
  String group = 'quantity',
}) => pb.LibraryItemView(
  id: id,
  category: 'concept',
  displayName: name,
  description: 'about $name',
  group: group,
  creates: [
    pb.LibraryObjectView(
      kind: 'concept',
      key: 'concept',
      name: name.replaceAll(' ', ''),
      typeName: typeName,
      representation: representation,
      unit: unit,
    ),
  ],
  concept: pb.ConceptTemplateView(
    id: id,
    displayName: name,
    defaultName: name.replaceAll(' ', ''),
    category: group,
    representation: representation,
    typeName: typeName,
    unit: unit,
  ),
);

final angle = category(
  'std.quantity.angle',
  'Angle',
  representation: pb.Representation(quantity: pb.Dim(angle: 1)),
  unit: 'rad',
  typeName: 'Angle',
);
final temperature = category(
  'std.quantity.temperature',
  'Temperature',
  representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
  unit: 'K',
  typeName: 'Temperature',
);
final omega = category(
  'std.quantity.angular_velocity',
  'Angular velocity',
  representation: pb.Representation(quantity: pb.Dim(angle: 1, time: -1)),
  typeName: 'AngularVelocity',
);
final onOff = category(
  'std.value.boolean',
  'On / off',
  representation: pb.Representation(boolean: pb.Unit()),
  typeName: 'Bool',
  group: 'form',
);
final open = category('std.value.open', 'Decide later', group: 'form');

final quantities = [
  pb.QuantityView(id: 'angle', typeName: 'Angle', unit: 'rad', dim: pb.Dim(angle: 1)),
  pb.QuantityView(
    id: 'temperature',
    typeName: 'Temperature',
    unit: 'K',
    dim: pb.Dim(temperature: 1),
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

/// The compiler's value categories with the units each is measured in:
/// the atoms of the dimension, then the curated composites
/// (`ListValueCategories`, protocol 0.27).
final categories = [
  pb.ValueCategoryView(
    id: 'angle',
    displayName: 'angle',
    typeName: 'Angle',
    representation: pb.Representation(quantity: pb.Dim(angle: 1)),
    dim: pb.Dim(angle: 1),
    preferredUnit: unitOf('rad', 'rad', pb.Dim(angle: 1)),
    units: [
      unitOf('rad', 'rad', pb.Dim(angle: 1)),
      unitOf('deg', 'deg', pb.Dim(angle: 1)),
      unitOf('turn', 'turn', pb.Dim(angle: 1)),
    ],
  ),
  pb.ValueCategoryView(
    id: 'temperature',
    displayName: 'temperature',
    typeName: 'Temperature',
    representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
    dim: pb.Dim(temperature: 1),
    preferredUnit: unitOf('K', 'K', pb.Dim(temperature: 1)),
    units: [unitOf('K', 'K', pb.Dim(temperature: 1))],
  ),
  pb.ValueCategoryView(
    id: 'angular_velocity',
    displayName: 'angular velocity',
    typeName: 'AngularVelocity',
    representation: pb.Representation(quantity: pb.Dim(angle: 1, time: -1)),
    dim: pb.Dim(angle: 1, time: -1),
    preferredUnit: unitOf('rad per s', 'rad/s', pb.Dim(angle: 1, time: -1)),
    units: [
      unitOf('rad per s', 'rad/s', pb.Dim(angle: 1, time: -1)),
      unitOf('deg per s', 'deg/s', pb.Dim(angle: 1, time: -1)),
      unitOf('turn per s', 'turn/s', pb.Dim(angle: 1, time: -1)),
    ],
  ),
];

pb.LibraryItemsResponse library() => pb.LibraryItemsResponse(
  libraries: [
    pb.LibraryView(id: 'std', name: 'Standard', items: [onOff, open, angle, temperature, omega]),
  ],
  quantities: quantities,
);

pb.ProjectProjection project() =>
    pb.ProjectProjection(revision: Int64(1), name: 'lamp', rootPath: '/p')
      ..concepts.add(pb.ConceptView(id: Int64(3), name: 'Tilt'));

AppState connected() => AppState(
  connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
  project: project(),
  library: library(),
);

void main() {
  group('reducer', () {
    test('every concept entry point opens the sheet; nothing is created before Create', () {
      final s = connected();
      final fromRow = reduce(s, const NewConceptRequested(presetId: 'std.quantity.angle'));
      final fromCanvas = reduce(
        s,
        const NewConceptRequested(presetId: 'std.quantity.angle', position: Offset(120, 40)),
      );
      final fromProjectTab = reduce(s, const NewConceptRequested());
      for (final t in [fromRow, fromCanvas, fromProjectTab]) {
        expect(t.effects, isEmpty, reason: 'the sheet asks; the daemon is not touched');
        expect(t.state.editor.conceptSheet, isNotNull);
        expect(t.state.editor.pendingInsert, isNull);
      }
      expect(fromRow.state.editor.conceptSheet!.presetId, 'std.quantity.angle');
      expect(fromCanvas.state.editor.conceptSheet!.position, const Offset(120, 40));
      expect(fromProjectTab.state.editor.conceptSheet!.presetId, '');
      // cancelling closes it and changes nothing
      final cancelled = reduce(fromCanvas.state, const ConceptSheetDismissed());
      expect(cancelled.state.editor.conceptSheet, isNull);
      expect(cancelled.effects, isEmpty);
      // a busy insertion refuses a second sheet
      final busy = fromCanvas.state.copyWith(
        editor: fromCanvas.state.editor.copyWith(
          pendingInsert: const PendingInsert(templateId: 'x'),
        ),
      );
      expect(reduce(busy, const NewConceptRequested()).state.editor.conceptSheet, isNotNull);
    });

    test(
      'Create from the canvas is one transaction: the concept, and a block of it at the point',
      () {
        final s = reduce(
          connected(),
          const NewConceptRequested(presetId: 'std.quantity.angle', position: Offset(120, 40)),
        ).state;
        final t = reduce(
          s,
          CreateConceptRequested(
            name: 'LidAngle',
            description: 'How far the lid is open.',
            representation: pb.Representation(quantity: pb.Dim(angle: 1)),
            position: const Offset(120, 40),
            presetId: 'std.quantity.angle',
          ),
        );
        // the concept is the template (ADR-0044); the Sem block of it is what
        // lands where the sheet was asked for — the Source path's one
        // transaction, the block named after the concept
        expect(t.effects, hasLength(1));
        final create = t.effects.single as CreateSource;
        expect(create.newConcept!.name, 'LidAngle');
        expect(create.newConcept!.description, 'How far the lid is open.');
        expect(create.newConcept!.representation.quantity, pb.Dim(angle: 1));
        expect(create.sourceName, 'lidAngle');
        expect(t.state.editor.conceptSheet, isNull);
        expect(t.state.editor.pendingInsert!.position, const Offset(120, 40));
        expect(t.state.editor.pendingInsert!.named, isTrue);
        expect(t.state.editor.recentTemplates, ['std.quantity.angle']);
        // the answer: the block placed and selected, not opened for renaming
        final answered = reduce(
          t.state,
          ProjectReceived(
            project()
              ..revision = Int64(2)
              ..concepts.add(
                pb.ConceptView(
                  id: Int64(9),
                  name: 'LidAngle',
                  representation: pb.Representation(quantity: pb.Dim(angle: 1)),
                ),
              )
              ..mappings.add(
                pb.MappingView(
                  id: Int64(4),
                  name: 'lidAngle',
                  signature: pb.Signature(output: Int64(9)),
                  role: pb.RelationshipRole.RELATIONSHIP_ROLE_SOURCE,
                ),
              ),
            fromRequest: true,
            outcome: pb.EditOutcome(createdConcept: Int64(9), createdMapping: Int64(4)),
          ),
        ).state;
        expect(answered.editor.selection, const MappingSelected(4));
        expect(answered.editor.layout[const NodeRef.mapping(4)], const Offset(120, 40));
        expect(answered.editor.layout[const NodeRef.concept(9)], isNull);
        expect(answered.editor.renaming, isNull);
        expect(answered.editor.pendingInsert, isNull);
      },
    );

    test('Create from the Library (no point) is the template alone: one ordinary edit', () {
      final t = reduce(
        connected(),
        CreateConceptRequested(
          name: 'LidAngle',
          description: '',
          representation: pb.Representation(quantity: pb.Dim(angle: 1)),
        ),
      );
      expect(t.effects, hasLength(1));
      final edit = (t.effects.single as ApplyEdit).op.createConcept;
      expect(edit.name, 'LidAngle');
      expect(t.state.editor.pendingInsert!.position, isNull);
      final answered = reduce(
        t.state,
        ProjectReceived(
          project()
            ..revision = Int64(2)
            ..concepts.add(pb.ConceptView(id: Int64(9), name: 'LidAngle')),
          fromRequest: true,
          outcome: pb.EditOutcome(createdConcept: Int64(9)),
        ),
      );
      expect(answered.effects.whereType<CreateSource>(), isEmpty, reason: 'no point, no block');
      expect(answered.state.editor.pendingInsert, isNull);
      expect(answered.state.editor.layout, isEmpty);
    });
  });

  group('sheet', () {
    Future<List<CreateConceptRequested>> pump(
      WidgetTester t,
      ConceptSheetState sheet, {
      List<String> taken = const ['Tilt'],
    }) async {
      final created = <CreateConceptRequested>[];
      await t.pumpWidget(
        MaterialApp(
          theme: macTheme(Brightness.light),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: kSupportedLocales,
          home: Scaffold(
            body: SingleChildScrollView(
              child: ConceptSheetForm(
                sheet: sheet,
                items: [onOff, open, angle, temperature, omega],
                quantities: quantities,
                categories: categories,
                taken: taken,
                onCreate: created.add,
                onCancel: () {},
              ),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      return created;
    }

    testWidgets('the name is required: Create waits for one, and says why a name will not do', (
      t,
    ) async {
      final created = await pump(t, const ConceptSheetState(presetId: 'std.quantity.angle'));
      final create = find.byKey(const ValueKey('concept-create'));
      // nothing typed: the primary button is disabled, no hint yet
      expect(find.byKey(const ValueKey('concept-name-problem')), findsNothing);
      await t.tap(create);
      await t.pumpAndSettle();
      expect(created, isEmpty);
      // a taken name, and one that is not an identifier
      await t.enterText(find.byKey(const ValueKey('concept-name')), 'Tilt');
      await t.pumpAndSettle();
      expect(find.text('Tilt is already in use.'), findsOneWidget);
      await t.tap(create);
      await t.pumpAndSettle();
      expect(created, isEmpty);
      await t.enterText(find.byKey(const ValueKey('concept-name')), '9lives');
      await t.pumpAndSettle();
      expect(find.byKey(const ValueKey('concept-name-problem')), findsOneWidget);
      // a good name: the preview says the declaration; Create sends it
      await t.enterText(find.byKey(const ValueKey('concept-name')), 'LidAngle');
      await t.pumpAndSettle();
      expect(find.byKey(const ValueKey('concept-name-problem')), findsNothing);
      expect(find.text('concept LidAngle : Angle'), findsOneWidget);
      await t.tap(create);
      await t.pumpAndSettle();
      final r = created.single;
      expect(r.name, 'LidAngle');
      expect(r.representation, pb.Representation(quantity: pb.Dim(angle: 1)));
      expect(r.presetId, 'std.quantity.angle');
    });

    testWidgets('the units are the compiler\'s for the category, a fact beside the choice', (
      t,
    ) async {
      await pump(t, const ConceptSheetState(presetId: 'std.quantity.angle'));
      // Angle: rad deg turn, the registry's order, canonical first
      expect(find.text('rad'), findsWidgets);
      expect(find.text('deg'), findsOneWidget);
      expect(find.text('turn'), findsOneWidget);
      expect(find.text('K'), findsNothing);
      // change the category: the units follow — the compiler's composites
      // for an angular velocity, rendered as it renders them
      await t.tap(find.byKey(const ValueKey('concept-category')));
      await t.pumpAndSettle();
      await t.tap(find.text('Angular velocity').last);
      await t.pumpAndSettle();
      expect(find.text('deg'), findsNothing);
      expect(find.text('rad/s'), findsWidgets);
      expect(find.text('deg/s'), findsOneWidget);
      expect(find.text('turn/s'), findsOneWidget);
      // a value form has no unit row
      await t.tap(find.byKey(const ValueKey('concept-category')));
      await t.pumpAndSettle();
      await t.tap(find.text('On / off').last);
      await t.pumpAndSettle();
      expect(find.byKey(const ValueKey('concept-units')), findsNothing);
      await t.enterText(find.byKey(const ValueKey('concept-name')), 'LidClosed');
      await t.pumpAndSettle();
      expect(find.text('concept LidClosed : Bool'), findsOneWidget);
    });

    testWidgets('from the Project tab the category is chosen on the sheet', (t) async {
      final created = await pump(t, const ConceptSheetState());
      await t.enterText(find.byKey(const ValueKey('concept-name')), 'Mood');
      await t.pumpAndSettle();
      // no category yet: Create waits
      await t.tap(find.byKey(const ValueKey('concept-create')));
      await t.pumpAndSettle();
      expect(created, isEmpty);
      await t.tap(find.byKey(const ValueKey('concept-category')));
      await t.pumpAndSettle();
      await t.tap(find.text('Decide later').last);
      await t.pumpAndSettle();
      expect(find.text('concept Mood'), findsOneWidget);
      await t.tap(find.byKey(const ValueKey('concept-create')));
      await t.pumpAndSettle();
      expect(created.single.representation, isNull);
      expect(created.single.presetId, 'std.value.open');
    });
  });
}
