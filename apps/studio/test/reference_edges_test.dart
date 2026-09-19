/// Reference edges and the three shapes (ADR-0034): the canvas draws what a
/// definition references — from the analysis's `references`, never from the
/// formula text — into the formula line of the relationship naming it; a
/// rule is told from a value by its input sockets and the word *rule*; the
/// inspector, the Simulate probe and the creation sheet say which shape a
/// relationship is with one vocabulary — *produces* is the signature,
/// *carried by* is a value per tick.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/dialogs.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/simulate_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';

const roomTemp = 0, buttonHeld = 1, switchState = 2;
const tempSensor = 10, buttonInput = 11, ctrl = 12, acOn = 13;

pb.ConceptView concept(int id, String name) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(boolean: pb.Unit()),
);

pb.MappingView mapping(int id, String name, List<int> inputs, int output, {String? formula}) =>
    pb.MappingView(
      id: Int64(id),
      name: name,
      signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
      definition: formula == null ? null : pb.Definition(formula: formula),
    );

/// Sources `TempSensor : () -> RoomTemp`, `ButtonInput : () -> ButtonHeld`;
/// the rule `AirConditionerCtrl : RoomTemp -> ButtonHeld -> SwitchState`;
/// the value `acOn : () -> SwitchState = AirConditionerCtrl(TempSensor, ButtonInput)`.
pb.ProjectProjection design({bool withValue = true}) => pb.ProjectProjection(
  revision: Int64(1),
  name: 'ac',
  rootPath: '/p',
  concepts: [
    concept(roomTemp, 'RoomTemp'),
    concept(buttonHeld, 'ButtonHeld'),
    concept(switchState, 'SwitchState'),
  ],
  mappings: [
    mapping(tempSensor, 'TempSensor', const [], roomTemp),
    mapping(buttonInput, 'ButtonInput', const [], buttonHeld),
    mapping(
      ctrl,
      'AirConditionerCtrl',
      const [roomTemp, buttonHeld],
      switchState,
      formula: 'RoomTemp and ButtonHeld',
    ),
    if (withValue)
      mapping(
        acOn,
        'acOn',
        const [],
        switchState,
        formula: 'AirConditionerCtrl(TempSensor, ButtonInput)',
      ),
  ],
);

/// What the compiler reports for [design]: the value references the rule
/// and both Sources; nothing else references anything.
pb.ProjectAnalysis analysis() => pb.ProjectAnalysis(
  revision: Int64(1),
  mappings: [
    pb.MappingAnalysis(id: Int64(tempSensor), status: pb.MappingStatus.MAPPING_STATUS_DECLARED),
    pb.MappingAnalysis(id: Int64(buttonInput), status: pb.MappingStatus.MAPPING_STATUS_DECLARED),
    pb.MappingAnalysis(id: Int64(ctrl), status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT),
    pb.MappingAnalysis(
      id: Int64(acOn),
      status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
      references: [Int64(tempSensor), Int64(buttonInput), Int64(ctrl)],
    ),
  ],
);

Map<int, List<int>> refsOf(pb.ProjectAnalysis a) => {
  for (final m in a.mappings) m.id.toInt(): [for (final d in m.references) d.toInt()],
};

final layout = <NodeRef, Offset>{
  const NodeRef.concept(roomTemp): const Offset(48, 48),
  const NodeRef.concept(buttonHeld): const Offset(48, 96),
  const NodeRef.concept(switchState): const Offset(48, 144),
  const NodeRef.mapping(tempSensor): const Offset(300, 48),
  const NodeRef.mapping(buttonInput): const Offset(300, 148),
  const NodeRef.mapping(ctrl): const Offset(600, 48),
  const NodeRef.mapping(acOn): const Offset(900, 48),
};

AppState connected(
  pb.ProjectProjection project, {
  Selection selection = const NoSelection(),
  pb.ProjectAnalysis? analysis,
}) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  analysis: analysis,
  editor: EditorState(selection: selection),
);

class Harness extends StatefulWidget {
  const Harness({super.key, required this.initial, required this.child, this.width = 420});
  final AppState initial;
  final double width;
  final Widget Function(AppState, void Function(AppAction)) child;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late AppState state = widget.initial;
  void dispatch(AppAction a) => setState(() => state = reduce(state, a).state);

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    supportedLocales: kSupportedLocales,
    localizationsDelegates: const [
      ...AppLocalizations.localizationsDelegates,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    home: Scaffold(
      body: Align(
        alignment: Alignment.topLeft,
        child: SizedBox(width: widget.width, height: 1200, child: widget.child(state, dispatch)),
      ),
    ),
  );
}

void main() {
  group('canvas geometry', () {
    test('the value is joined to the rule and both Sources by reference edges', () {
      final scene = buildScene(design(), layout, refs: refsOf(analysis()));
      final refs = scene.links.where((l) => l.reference).toList();
      expect(refs.map((l) => l.from.node.id).toSet(), {tempSensor, buttonInput, ctrl});
      expect(refs.every((l) => l.to.node == const NodeRef.mapping(acOn)), isTrue);
      // From the referenced relationship's output socket …
      for (final l in refs) {
        expect(l.from.side, SocketSide.output);
        final from = scene.nodes.firstWhere((n) => n.ref == l.from.node);
        final socket = from.sockets.firstWhere((s) => s.ref == l.from);
        expect(l.path.getBounds().left, closeTo(socket.center.dx, 0.01));
      }
      // … into the formula line of the one naming it, never a socket.
      final value = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(acOn));
      for (final l in refs) {
        final end = l.path.computeMetrics().first.let((m) => m.getTangentForOffset(m.length));
        expect(end!.position.dx, closeTo(value.formulaEntry.dx, 0.01));
        expect(end.position.dy, closeTo(value.formulaEntry.dy, 0.01));
      }
      expect(value.sockets.where((s) => s.ref.side == SocketSide.input), isEmpty);
      expect(value.dependsOn, ['TempSensor', 'ButtonInput', 'AirConditionerCtrl']);
    });

    test('signature edges are unchanged: reference edges are added, not substituted', () {
      final without = buildScene(design(), layout);
      final with_ = buildScene(design(), layout, refs: refsOf(analysis()));
      expect(without.links.where((l) => l.reference), isEmpty);
      expect(
        with_.links.where((l) => !l.reference).length,
        without.links.length,
        reason: 'concept → rule input ×2, relationship → concept ×4',
      );
      expect(without.links.length, 6);
    });

    test('nothing is drawn without an analysis, for a self-reference, or for an unknown id', () {
      expect(buildScene(design(), layout).links.where((l) => l.reference), isEmpty);
      final scene = buildScene(
        design(),
        layout,
        refs: {
          acOn: [acOn, 999],
        },
      );
      expect(scene.links.where((l) => l.reference), isEmpty);
      final value = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(acOn));
      expect(value.dependsOn, isEmpty);
    });

    test('a rule is told from a value by its input sockets and the rule flag', () {
      final scene = buildScene(design(), layout, refs: refsOf(analysis()));
      final rule = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(ctrl));
      final value = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(acOn));
      final source = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tempSensor));
      expect(rule.rule, isTrue);
      expect(rule.sockets.where((s) => s.ref.side == SocketSide.input).length, 2);
      expect(value.rule, isFalse);
      expect(source.rule, isFalse);
      expect(source.source, isTrue);
      // A declared rule keeps the flag: the state word wins the header slot,
      // the sockets still say rule.
      final declared = buildScene(
        pb.ProjectProjection(name: 'd')
          ..concepts.addAll(design().concepts)
          ..mappings.add(mapping(ctrl, 'AirConditionerCtrl', const [roomTemp], switchState)),
        const {},
      );
      final d = declared.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(ctrl));
      expect(d.rule && d.declared, isTrue);
    });

    test('a reference edge is not a drop target and not selectable', () {
      final scene = buildScene(design(), layout, refs: refsOf(analysis()));
      final ref = scene.links.firstWhere((l) => l.reference);
      expect(hitTest(scene, ref.midpoint), isNot(isA<HitLink>()));
      final value = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(acOn));
      // The formula entry is on the node: a drop there is a drop on the node,
      // never on a socket.
      expect(hitTest(scene, value.formulaEntry + const Offset(1, 0)), isA<HitNode>());
      expect(compatibleSockets(scene, ref.from).any((s) => s.node == value.ref), isFalse);
    });
  });

  group('inspector', () {
    testWidgets('a rule: Role Rule, its explanation, Named in the value', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(
            design(),
            selection: const MappingSelected(ctrl),
            analysis: analysis(),
          ),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Rule'), findsOneWidget);
      expect(find.text(kEnglish.ruleExplanation), findsOneWidget);
      expect(find.text('Named in'), findsOneWidget);
      expect(find.text('acOn'), findsOneWidget);
      expect(find.text('Depends on'), findsOneWidget);
      expect(find.text('Source'), findsNothing);
    });

    testWidgets('a value: Role Value, Depends on the rule and both Sources', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(
            design(),
            selection: const MappingSelected(acOn),
            analysis: analysis(),
          ),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Value'), findsOneWidget);
      expect(find.text(kEnglish.valueExplanation), findsOneWidget);
      expect(find.text('Depends on'), findsOneWidget);
      for (final name in ['TempSensor', 'ButtonInput', 'AirConditionerCtrl']) {
        expect(find.text(name), findsOneWidget);
      }
      expect(find.text('Rule'), findsNothing);
    });

    testWidgets('a Source keeps its own role and no Depends on row', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(
            design(),
            selection: const MappingSelected(tempSensor),
            analysis: analysis(),
          ),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Source'), findsWidgets);
      expect(find.text('Depends on'), findsNothing);
      expect(find.text('Rule'), findsNothing);
      expect(find.text('Value'), findsNothing);
    });
  });

  group('Simulate probe', () {
    void wide(WidgetTester t) {
      t.view.physicalSize = const Size(1280, 720);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.reset);
    }

    testWidgets('a concept is carried by its values and Sources, never by a rule', (t) async {
      wide(t);
      await t.pumpWidget(
        Harness(
          width: 1280,
          initial: connected(
            design(),
            selection: const ConceptSelected(switchState),
            analysis: analysis(),
          ),
          child: (s, d) => SimulatePage(state: s, dispatch: d),
        ),
      );
      expect(find.text(kEnglish.carriedBy), findsOneWidget);
      expect(find.text('acOn'), findsWidgets);
      expect(find.textContaining('No value declaration'), findsNothing);
    });

    testWidgets('without the value: nothing carries the concept; the rule is named', (t) async {
      wide(t);
      await t.pumpWidget(
        Harness(
          width: 1280,
          initial: connected(
            design(withValue: false),
            selection: const ConceptSelected(switchState),
          ),
          child: (s, d) => SimulatePage(state: s, dispatch: d),
        ),
      );
      expect(find.text(kEnglish.noValueCarries('SwitchState')), findsOneWidget);
      expect(
        find.text(kEnglish.ruleProducesNoValue('AirConditionerCtrl', 'SwitchState')),
        findsOneWidget,
      );
    });

    testWidgets('a rule: no value of its own; applied in the value, or in nothing yet', (t) async {
      wide(t);
      await t.pumpWidget(
        Harness(
          width: 1280,
          initial: connected(
            design(),
            selection: const MappingSelected(ctrl),
            analysis: analysis(),
          ),
          child: (s, d) => SimulatePage(state: s, dispatch: d),
        ),
      );
      expect(find.text(kEnglish.aRuleNoValueOfItsOwn), findsOneWidget);
      expect(find.text(kEnglish.appliedIn('acOn')), findsOneWidget);

      await t.pumpWidget(
        Harness(
          key: UniqueKey(),
          width: 1280,
          initial: connected(design(withValue: false), selection: const MappingSelected(ctrl)),
          child: (s, d) => SimulatePage(state: s, dispatch: d),
        ),
      );
      expect(find.text(kEnglish.noValueAppliesItYet), findsOneWidget);
    });
  });

  group('creation sheet', () {
    testWidgets('says which shape the reads make: a Source, then a rule', (t) async {
      late BuildContext ctx;
      await t.pumpWidget(
        MaterialApp(
          supportedLocales: kSupportedLocales,
          localizationsDelegates: const [
            ...AppLocalizations.localizationsDelegates,
            GlobalMaterialLocalizations.delegate,
            GlobalWidgetsLocalizations.delegate,
            GlobalCupertinoLocalizations.delegate,
          ],
          theme: macTheme(Brightness.light),
          home: Builder(
            builder: (c) {
              ctx = c;
              return const Scaffold();
            },
          ),
        ),
      );
      final result = showNewMappingSheet(ctx, design().concepts);
      await t.pumpAndSettle();
      await t.enterText(find.byType(EditableText).first, 'isHot');
      await t.tap(find.text('choose'));
      await t.pumpAndSettle();
      await t.tap(find.text('SwitchState').last);
      await t.pumpAndSettle();
      expect(find.text(kEnglish.sheetSourceShape('SwitchState')), findsOneWidget);
      await t.tap(find.text('RoomTemp'));
      await t.pumpAndSettle();
      expect(find.text(kEnglish.sheetRuleShape('RoomTemp', 'SwitchState')), findsOneWidget);
      expect(find.text(kEnglish.sheetSourceShape('SwitchState')), findsNothing);
      await t.tap(find.text('Cancel'));
      await t.pumpAndSettle();
      expect(await result, isNull);
    });
  });
}

extension<T> on T {
  R let<R>(R Function(T) f) => f(this);
}
