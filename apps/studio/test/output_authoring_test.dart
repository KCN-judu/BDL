/// Concept → Output as an authoring gesture (docs/architecture/studio-ui.md
/// §2, "Concept → Output"): the designer drags the visible concept onto the
/// sink that accepts it; the edit is the driver's.  One eligible driver
/// connects; several are offered by name, never chosen; none is explained
/// at the pointer; a driven sink is offered a replacement, never a second
/// driver.  Candidates come from the projection the daemon sent — role,
/// exact concept, domain, drives — and the direct Mapping → Output drag is
/// what it was.  Nothing here touches the kernel: every dispatch is the
/// existing `SetMappingDrive`.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/canvas_harness.dart' show SceneLookup;

const servoPosition = 10, otherPosition = 11, brightness = 12;
const servoValue = 20, altServo = 21, posRule = 22, otherValue = 23, offDomain = 24;
const servo = 30, lamp = 31;
const main_ = 1, aux = 2;

pb.ConceptView angle(int id, String name) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(quantity: pb.Dim(angle: 1)),
);

pb.MappingView valueOf(
  int id,
  String name,
  int concept, {
  int? clock,
  int? drives,
  List<int> inputs = const [],
}) => pb.MappingView(
  id: Int64(id),
  name: name,
  signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(concept)),
  definition: pb.Definition(formula: '1'),
  clockId: clock == null ? null : Int64(clock),
  drivesOutputId: drives == null ? null : Int64(drives),
  role: inputs.isEmpty
      ? pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE
      : pb.RelationshipRole.RELATIONSHIP_ROLE_RULE,
);

/// ServoPosition and OtherPosition (both angles: one representation, two
/// identities), Brightness; servoValue and altServo produce ServoPosition
/// in `main`, posRule is a rule producing it, offDomain a value producing it
/// in `aux`, otherValue produces OtherPosition; Servo accepts ServoPosition
/// in `main`, lamp accepts Brightness.
pb.ProjectProjection design({List<pb.MappingView>? mappings, int? servoDriver}) =>
    pb.ProjectProjection(
      revision: Int64(1),
      name: 'arm',
      concepts: [
        angle(servoPosition, 'ServoPosition'),
        angle(otherPosition, 'OtherPosition'),
        pb.ConceptView(
          id: Int64(brightness),
          name: 'Brightness',
          representation: pb.Representation(quantity: pb.Dim()),
        ),
      ],
      mappings:
          mappings ??
          [
            valueOf(servoValue, 'servoValue', servoPosition, clock: main_, drives: servoDriver),
            valueOf(altServo, 'altServo', servoPosition, clock: main_),
            valueOf(posRule, 'posRule', servoPosition, clock: main_, inputs: [otherPosition]),
            valueOf(otherValue, 'otherValue', otherPosition, clock: main_),
            valueOf(offDomain, 'offDomain', servoPosition, clock: aux),
          ],
      outputs: [
        pb.OutputView(
          id: Int64(servo),
          name: 'Servo',
          accepts: Int64(servoPosition),
          clockId: Int64(main_),
        ),
        pb.OutputView(
          id: Int64(lamp),
          name: 'lamp',
          accepts: Int64(brightness),
          clockId: Int64(main_),
        ),
      ],
      clocks: [
        pb.ClockView(id: Int64(main_), name: 'main'),
        pb.ClockView(id: Int64(aux), name: 'aux'),
      ],
    );

const cServo = NodeRef.concept(servoPosition);
const cOther = NodeRef.concept(otherPosition);
const oServo = NodeRef.output(servo);
const oLamp = NodeRef.output(lamp);

final layout = <NodeRef, Offset>{
  cServo: const Offset(40, 40),
  cOther: const Offset(40, 100),
  const NodeRef.concept(brightness): const Offset(40, 160),
  const NodeRef.mapping(servoValue): const Offset(320, 40),
  const NodeRef.mapping(altServo): const Offset(320, 160),
  const NodeRef.mapping(posRule): const Offset(320, 280),
  const NodeRef.mapping(otherValue): const Offset(320, 420),
  const NodeRef.mapping(offDomain): const Offset(600, 420),
  oServo: const Offset(900, 40),
  oLamp: const Offset(900, 200),
};

class Harness extends StatefulWidget {
  const Harness({super.key, required this.actions, required this.project});
  final List<AppAction> actions;
  final pb.ProjectProjection project;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  Selection selection = const NoSelection();
  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: NodeCanvas(
        project: widget.project,
        layout: layout,
        selection: selection,
        dispatch: (a) {
          widget.actions.add(a);
          if (a is SelectionChanged) setState(() => selection = a.selection);
        },
      ),
    ),
  );
}

Offset origin(WidgetTester t) => t.getTopLeft(find.byType(NodeCanvas));

Future<void> drag(WidgetTester tester, Offset from, Offset to) async {
  final o = origin(tester);
  final g = await tester.startGesture(o + from, kind: PointerDeviceKind.mouse);
  await tester.pump(const Duration(milliseconds: 20));
  for (var i = 1; i <= 8; i++) {
    await g.moveTo(o + Offset.lerp(from, to, i / 8)!);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await g.up();
  await tester.pump(const Duration(milliseconds: 20));
  await tester.pump(const Duration(milliseconds: 100));
}

Future<List<AppAction>> pumpCanvas(WidgetTester tester, pb.ProjectProjection project) async {
  tester.view.physicalSize = const Size(1400, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final actions = <AppAction>[];
  await tester.pumpWidget(Harness(actions: actions, project: project));
  return actions;
}

Iterable<String> menuLabels(WidgetTester t) => t
    .widgetList<Text>(find.descendant(of: find.byType(MenuItemButton), matching: find.byType(Text)))
    .map((w) => w.data ?? '');

Finder menuItem(String label) =>
    find.descendant(of: find.byType(MenuItemButton), matching: find.text(label));

void main() {
  group('candidates (compiler/model truth off the projection)', () {
    test('a value or Source producing exactly the concept, in the sink\'s domain, not '
        'driving another sink; never a rule, never another identity', () {
      final p = design();
      final servoOut = p.outputs.firstWhere((o) => o.id.toInt() == servo);
      final names = driveCandidates(p, servoOut).map((m) => m.name).toList();
      expect(names, ['servoValue', 'altServo']);
      // G: offDomain updates in aux, the sink in main — refused
      expect(names, isNot(contains('offDomain')));
      // a rule has an arrow type — refused
      expect(names, isNot(contains('posRule')));
      // F: OtherPosition is another identity with the same representation
      expect(names, isNot(contains('otherValue')));
      // the current driver is the sink's state, not a candidate
      final driven = design(servoDriver: servo);
      final o = driven.outputs.firstWhere((o) => o.id.toInt() == servo);
      expect(driveCandidates(driven, o).map((m) => m.name), ['altServo']);
      expect(currentDriver(driven, o)?.name, 'servoValue');
      // a value already driving another sink is not offered
      final busy = design(
        mappings: [
          valueOf(servoValue, 'servoValue', servoPosition, clock: main_, drives: lamp),
          valueOf(altServo, 'altServo', servoPosition, clock: main_),
        ],
      );
      expect(driveCandidates(busy, o).map((m) => m.name), ['altServo']);
    });

    test('a sink without a domain takes any value of its concept; the pass reports later', () {
      final p = design()..outputs.first.clearClockId();
      final o = p.outputs.first;
      expect(driveCandidates(p, o).map((m) => m.name), ['servoValue', 'altServo', 'offDomain']);
    });
  });

  group('geometry', () {
    test('A/E/F: the concept\'s value socket targets the sink accepting it, and no other', () {
      final scene = buildScene(design(), layout);
      final servoOut = scene.socket(cServo, side: SocketSide.output).ref;
      final otherOut = scene.socket(cOther, side: SocketSide.output).ref;
      final servoIn = scene.socket(oServo, side: SocketSide.input).ref;
      final lampIn = scene.socket(oLamp, side: SocketSide.input).ref;
      expect(isAuthoringTarget(servoOut, servoIn), isTrue);
      expect(isAuthoringTarget(otherOut, servoIn), isFalse, reason: 'another identity');
      expect(isAuthoringTarget(servoOut, lampIn), isFalse, reason: 'another concept');
      // K: the model's own link rule is untouched — a concept never links a sink
      expect(canLink(servoOut, servoIn), isFalse);
      // the eligible sink lights up while the concept is dragged
      expect(compatibleSockets(scene, servoOut), contains(servoIn));
      expect(compatibleSockets(scene, servoOut), isNot(contains(lampIn)));
      expect(
        dropTarget(scene, servoOut, scene.socket(oServo, side: SocketSide.input).center)?.ref,
        servoIn,
      );
      expect(
        dropTarget(scene, servoOut, scene.socket(oLamp, side: SocketSide.input).center),
        isNull,
      );
    });
  });

  group('pointer', () {
    Offset socketAt(pb.ProjectProjection p, NodeRef n, SocketSide side) =>
        buildScene(p, layout).socket(n, side: side).center;

    testWidgets('B: one eligible producer resolves to that driver, at once', (t) async {
      final p = design(mappings: [valueOf(servoValue, 'servoValue', servoPosition, clock: main_)]);
      final actions = await pumpCanvas(t, p);
      await drag(t, socketAt(p, cServo, SocketSide.output), socketAt(p, oServo, SocketSide.input));
      final drive = actions.whereType<SetMappingDriveRequested>().single;
      expect(drive.mappingId, servoValue);
      expect(drive.outputId, servo);
      expect(menuLabels(t), isEmpty, reason: 'nothing to choose');
    });

    testWidgets('C: several producers open the chooser by name; nothing is chosen for the '
        'designer; the choice is the drive', (t) async {
      final p = design();
      final actions = await pumpCanvas(t, p);
      await drag(t, socketAt(p, cServo, SocketSide.output), socketAt(p, oServo, SocketSide.input));
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty);
      final labels = menuLabels(t).toList();
      expect(labels, ['Drive with servoValue', 'Drive with altServo']);
      expect(labels.join(), isNot(contains('offDomain')), reason: 'G: another domain');
      expect(labels.join(), isNot(contains('posRule')), reason: 'a rule is not a value');
      await t.tap(menuItem('Drive with altServo'));
      await t.pump();
      final drive = actions.whereType<SetMappingDriveRequested>().single;
      expect(drive.mappingId, altServo);
      expect(drive.outputId, servo);
    });

    testWidgets('D: no producer: the notice names the sink and the concept, nothing is made', (
      t,
    ) async {
      final p = design(mappings: [valueOf(otherValue, 'otherValue', otherPosition, clock: main_)]);
      final actions = await pumpCanvas(t, p);
      await drag(t, socketAt(p, cServo, SocketSide.output), socketAt(p, oServo, SocketSide.input));
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty);
      expect(
        menuLabels(t),
        contains('Servo accepts ServoPosition, but no current relationship can drive it.'),
      );
      // the way on: the sink in the inspector
      await t.tap(menuItem('Show Servo in the Inspector'));
      await t.pump();
      expect(actions.whereType<SelectionChanged>().last.selection, const OutputSelected(servo));
    });

    testWidgets('E/F: a concept the sink does not accept is refused at the pointer', (t) async {
      final p = design();
      final actions = await pumpCanvas(t, p);
      await drag(t, socketAt(p, cOther, SocketSide.output), socketAt(p, oServo, SocketSide.input));
      await drag(t, socketAt(p, cServo, SocketSide.output), socketAt(p, oLamp, SocketSide.input));
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty);
      expect(menuLabels(t), isEmpty);
    });

    testWidgets('H/I: a driven sink is offered a replacement, never a second driver', (t) async {
      final p = design(servoDriver: servo);
      final actions = await pumpCanvas(t, p);
      await drag(t, socketAt(p, cServo, SocketSide.output), socketAt(p, oServo, SocketSide.input));
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty);
      expect(menuLabels(t).toList(), ['Replace servoValue with altServo']);
      await t.tap(menuItem('Replace servoValue with altServo'));
      await t.pump();
      expect(
        actions.whereType<SetMappingDriveRequested>(),
        isEmpty,
        reason: 'never a second driver',
      );
      final replace = actions.whereType<ReplaceDriverRequested>().single;
      expect(replace.from, servoValue, reason: 'the old driver lets go first');
      expect(replace.to, altServo);
      expect(replace.outputId, servo);
    });

    testWidgets('J: the direct Mapping → Output drag is what it was', (t) async {
      final p = design();
      final actions = await pumpCanvas(t, p);
      await drag(
        t,
        socketAt(p, const NodeRef.mapping(altServo), SocketSide.output),
        socketAt(p, oServo, SocketSide.input),
      );
      final drive = actions.whereType<SetMappingDriveRequested>().single;
      expect(drive.mappingId, altServo);
      expect(drive.outputId, servo);
      expect(menuLabels(t), isEmpty);
    });
  });
}
