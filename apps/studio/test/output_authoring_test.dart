/// Driving a sink (ADR-0044): the driver is a Sem block, and the gesture is
/// the Sem block's output socket dropped on the sink accepting its concept
/// — the drive edge, `SetMappingDrive`.  The sink's inspector still offers
/// the eligible drivers by name (`driveCandidates`: a value or Source
/// producing exactly the concept, in the sink's domain, not driving another
/// sink), read off the projection the daemon sent.  With the concept node
/// gone (a concept is a template), the former Concept → Output authoring
/// gesture has no socket to start from; nothing here touches the kernel.
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

const oServo = NodeRef.output(servo);
const oLamp = NodeRef.output(lamp);

final layout = <NodeRef, Offset>{
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
    test('a Sem block\'s output targets the sink accepting its concept, and no other', () {
      final scene = buildScene(design(), layout);
      final servoOut = scene.socket(const NodeRef.mapping(servoValue), side: SocketSide.output).ref;
      final otherOut = scene.socket(const NodeRef.mapping(otherValue), side: SocketSide.output).ref;
      final servoIn = scene.socket(oServo, side: SocketSide.input).ref;
      final lampIn = scene.socket(oLamp, side: SocketSide.input).ref;
      expect(canLink(servoOut, servoIn), isTrue);
      expect(canLink(otherOut, servoIn), isFalse, reason: 'another identity');
      expect(canLink(servoOut, lampIn), isFalse, reason: 'another concept');
      expect(compatibleSockets(scene, servoOut), contains(servoIn));
      expect(compatibleSockets(scene, servoOut), isNot(contains(lampIn)));
      // no concept node, no rule node: a template is not on the canvas
      expect(scene.nodes.where((n) => n.ref.kind == NodeKind.concept), isEmpty);
      expect(scene.nodes.where((n) => n.ref == const NodeRef.mapping(posRule)), isEmpty);
    });
  });

  group('pointer', () {
    Offset socketAt(pb.ProjectProjection p, NodeRef n, SocketSide side) =>
        buildScene(p, layout).socket(n, side: side).center;

    testWidgets('the Sem block → Output drag is the drive edge, at once', (t) async {
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

    testWidgets('a Sem block of another concept is refused at the sink', (t) async {
      final p = design();
      final actions = await pumpCanvas(t, p);
      await drag(
        t,
        socketAt(p, const NodeRef.mapping(otherValue), SocketSide.output),
        socketAt(p, oServo, SocketSide.input),
      );
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty);
      expect(menuLabels(t), isEmpty);
    });
  });
}
