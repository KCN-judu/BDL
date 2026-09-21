/// The Sem-block canvas (ADR-0044; BDL_FV Phase 21): the design's value
/// graph is Sem blocks — `TempSensor`, `ButtonInput` (Sources) and `acOn`
/// (a Sem block whose mapping block applies the rule
/// `AirConditionerCtrl` to both) — joined by read edges the analysis
/// reports (`references`, never the formula text) into the reading block's
/// sockets; the rule is a template named on the block and in the
/// inspector, not a node; the concept is a template with instances; the
/// Simulate probe is per Sem block.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/l10n/l10n.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/dialogs.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/interactive.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/simulate_page.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

const roomTemp = 0, buttonHeld = 1, switchState = 2;
const tempSensor = 10, buttonInput = 11, ctrl = 12, acOn = 13;

pb.ConceptView concept(int id, String name) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(boolean: pb.Unit()),
);

pb.MappingView mapping(int id, String name, List<int> inputs, int output, {String? formula}) =>
    mappingView(
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
pb.ProjectAnalysis analysis() => withAppliedBy(
  pb.ProjectAnalysis(
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
  ),
);

Map<int, List<int>> refsOf(pb.ProjectAnalysis a) => {
  for (final m in a.mappings) m.id.toInt(): [for (final d in m.references) d.toInt()],
};

final layout = <NodeRef, Offset>{
  const NodeRef.mapping(tempSensor): const Offset(48, 48),
  const NodeRef.mapping(buttonInput): const Offset(48, 148),
  const NodeRef.mapping(acOn): const Offset(640, 48),
  const NodeRef.definition(acOn): const Offset(360, 48),
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
    test('the value graph: three Sem blocks and one mapping block; no concept, no rule', () {
      final scene = buildScene(design(), layout, refs: refsOf(analysis()));
      expect(scene.nodes.map((n) => n.ref).toSet(), {
        const NodeRef.mapping(tempSensor),
        const NodeRef.mapping(buttonInput),
        const NodeRef.mapping(acOn),
        const NodeRef.definition(acOn),
      });
      final value = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(acOn));
      expect(value.applies, ['AirConditionerCtrl']);
      expect(value.sockets.map((s) => s.ref.role), [SocketRole.produce, SocketRole.concept]);
      final block = scene.nodes.firstWhere((n) => n.ref == const NodeRef.definition(acOn));
      expect(block.dependsOn, ['TempSensor', 'ButtonInput']);
      expect(block.applies, ['AirConditionerCtrl']);
      expect(block.title, 'AirConditionerCtrl', reason: 'the header names the rule applied');
      expect(block.definition, 'AirConditionerCtrl(TempSensor, ButtonInput)');
      final reads = block.sockets.where((s) => s.ref.role == SocketRole.read).toList();
      expect(reads.map((s) => block.socketLabels[s.ref]), ['TempSensor', 'ButtonInput']);
      expect(reads.map((s) => s.ref.concept), [roomTemp, buttonHeld]);
      for (final src in [tempSensor, buttonInput]) {
        final n = scene.nodes.firstWhere((n) => n.ref == NodeRef.mapping(src));
        expect(n.source, isTrue);
        expect(n.sockets.length, 1, reason: 'a Source has its output socket only');
        expect(
          scene.nodes.any((n) => n.ref == NodeRef.definition(src)),
          isFalse,
          reason: 'a Source has no mapping block',
        );
      }
    });

    test('one read edge per Sem block the definition names, into its own socket', () {
      final scene = buildScene(design(), layout, refs: refsOf(analysis()));
      final reads = scene.links.where((l) => l.id.kind == LinkKind.read).toList();
      expect(reads.length, 2, reason: 'the rule makes no edge');
      final byFrom = {for (final l in reads) l.from.node: l};
      expect(byFrom.keys.toSet(), {
        const NodeRef.mapping(tempSensor),
        const NodeRef.mapping(buttonInput),
      });
      for (final l in reads) {
        expect(l.to.node, const NodeRef.definition(acOn));
        expect(l.to.role, SocketRole.read);
        expect(l.binding, isNull);
      }
      expect(byFrom[const NodeRef.mapping(tempSensor)]!.to.index, tempSensor);
      expect(
        byFrom[const NodeRef.mapping(buttonInput)]!.to.index,
        buttonHeld == 1 ? buttonInput : 0,
      );
      expect(byFrom[const NodeRef.mapping(tempSensor)]!.concept, roomTemp);
      // the definition itself: one produce edge into the Sem block
      final produce = scene.links.where((l) => l.id.kind == LinkKind.produce).single;
      expect(produce.from.node, const NodeRef.definition(acOn));
      expect(produce.to.node, const NodeRef.mapping(acOn));
      expect(produce.concept, switchState);
    });

    test('no read edge without an analysis, for a self-reference, or for an unknown id', () {
      expect(buildScene(design(), layout).links.map((l) => l.id.kind), [LinkKind.produce]);
      final scene = buildScene(
        design(),
        layout,
        refs: {
          acOn: [acOn, 999],
        },
      );
      expect(scene.links.map((l) => l.id.kind), [LinkKind.produce]);
      final block = scene.nodes.firstWhere((n) => n.ref == const NodeRef.definition(acOn));
      expect(block.sockets.where((s) => s.ref.role == SocketRole.read), isEmpty);
    });

    test(
      'a read edge is an object: hit, selectable, disconnectable by the compiler; no drop on it',
      () {
        final scene = buildScene(design(), layout, refs: refsOf(analysis()));
        final l = scene.links.firstWhere((l) => l.id.kind == LinkKind.read);
        expect(hitTest(scene, l.midpoint), isA<HitLink>());
        expect(l.id.disconnectable, isTrue, reason: 'ComposeAction.unreference');
        final produce = scene.links.firstWhere((l) => l.id.kind == LinkKind.produce);
        expect(produce.id.disconnectable, isFalse, reason: 'the definition is not cut alone');
        final out = scene.nodes
            .firstWhere((n) => n.ref == const NodeRef.mapping(buttonInput))
            .sockets
            .single;
        expect(
          dropTarget(
            scene,
            out.ref,
            l.to.let(
              (r) => scene.nodes
                  .firstWhere((n) => n.ref == r.node)
                  .sockets
                  .firstWhere((s) => s.ref == r)
                  .center,
            ),
          ),
          isNull,
        );
      },
    );
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

    testWidgets('a Sem block: Role Value, Reads both Sources, Applies the rule', (t) async {
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
      expect(find.text('Reads'), findsOneWidget);
      expect(find.text('Applies'), findsOneWidget);
      expect(find.text('Depends on'), findsNothing);
      for (final name in ['TempSensor', 'ButtonInput', 'AirConditionerCtrl']) {
        expect(find.text(name), findsOneWidget);
      }
      expect(find.text('Rule'), findsNothing);
    });

    testWidgets('a Source keeps its own role, no Reads row, and is Read by the value', (t) async {
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
      expect(find.text('Reads'), findsNothing);
      expect(find.text('Read by'), findsOneWidget);
      expect(find.text('acOn'), findsOneWidget);
      expect(find.text('Rule'), findsNothing);
      expect(find.text('Value'), findsNothing);
    });

    testWidgets('a concept: its blocks (the instances) and the rules over it', (t) async {
      await t.pumpWidget(
        Harness(
          initial: connected(
            design(),
            selection: const ConceptSelected(switchState),
            analysis: analysis(),
          ),
          child: (s, d) => Inspector(state: s, dispatch: d),
        ),
      );
      expect(find.text('Blocks'), findsOneWidget);
      expect(find.text('acOn'), findsOneWidget);
      expect(find.text('Rules'), findsOneWidget);
      expect(find.text('AirConditionerCtrl'), findsOneWidget);
      expect(find.text('Produced by'), findsNothing);
    });
  });

  group('Simulate probe', () {
    void wide(WidgetTester t) {
      t.view.physicalSize = const Size(1280, 720);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.reset);
    }

    testWidgets('a concept lists its Sem blocks, one value per tick each; never a rule', (t) async {
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
      expect(find.text(kEnglish.blocksOfConcept), findsOneWidget);
      expect(find.text('acOn'), findsWidgets);
      expect(find.text('AirConditionerCtrl'), findsNothing);
    });

    testWidgets('without a block of the concept: the sentence that says how to add one', (t) async {
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
      expect(find.text(kEnglish.noBlockOfConcept('SwitchState')), findsOneWidget);
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
      expect(find.text(kEnglish.appliedBy), findsOneWidget);
      expect(find.widgetWithText(MacLink, 'acOn'), findsOneWidget, reason: 'a link to the value');

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
