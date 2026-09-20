/// The canvas under real pointer events, with a recording dispatch and no
/// daemon (brief §28–§30): hit precedence between regions, nodes, sockets
/// and links; drags across a group boundary; socket drags that make links;
/// aggregate sockets resolving to concrete declarations; zoom levels.
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
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/canvas_harness.dart' show SceneLookup;

import 'support/roles.dart';

const tilt = 0, level = 1;
const tiltValue = 0, a = 1, b = 2, follower = 3, openTilt = 4;
const groupId = 7;

pb.ConceptView concept(int id, String name) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(quantity: pb.Dim()),
);

pb.MappingView mapping(
  int id,
  String name,
  int output, {
  List<int> inputs = const [],
  String? formula,
}) => mappingView(
  id: Int64(id),
  name: name,
  signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
  definition: formula == null ? null : pb.Definition(formula: formula),
);

/// tiltValue → A (reads it), B (does not); follower reads A; openTilt is open.
pb.ProjectProjection design() => pb.ProjectProjection(
  revision: Int64(1),
  name: 'lamp',
  concepts: [concept(tilt, 'Tilt'), concept(level, 'Brightness')],
  mappings: [
    mapping(tiltValue, 'tiltValue', tilt, formula: '1'),
    mapping(a, 'A', tilt, formula: 'tiltValue'),
    mapping(b, 'B', tilt, formula: '2'),
    mapping(follower, 'follower', tilt, formula: 'A'),
    mapping(openTilt, 'openTilt', tilt),
  ],
);

pb.BehaviorGroupView readers() =>
    pb.BehaviorGroupView(id: Int64(groupId), name: 'Readers', members: [Int64(a), Int64(b)]);

pb.BehaviorGroupBoundaryView boundary() => pb.BehaviorGroupBoundaryView(
  id: Int64(groupId),
  members: [Int64(a), Int64(b)],
  crossingIn: [Int64(tiltValue)],
  crossingOut: [Int64(a)],
  externalInputs: [Int64(tiltValue)],
  externalOutputs: [Int64(a)],
  privateCandidates: [Int64(b)],
  crossingEdges: [pb.DeclEdge(from: Int64(a), to: Int64(tiltValue))],
);

final layout = <NodeRef, Offset>{
  const NodeRef.concept(tilt): const Offset(20, 20),
  const NodeRef.concept(level): const Offset(20, 100),
  const NodeRef.mapping(tiltValue): const Offset(300, 20),
  const NodeRef.mapping(a): const Offset(600, 20),
  const NodeRef.mapping(b): const Offset(600, 160),
  const NodeRef.mapping(follower): const Offset(950, 20),
  const NodeRef.mapping(openTilt): const Offset(950, 200),
};

pb.SystemView systemView() => pb.SystemView(
  revision: Int64(1),
  base: design(),
  groups: [readers()],
  boundaries: [boundary()],
);

class Harness extends StatefulWidget {
  const Harness({
    super.key,
    required this.actions,
    this.collapsed = false,
    this.selection = const NoSelection(),
  });
  final List<AppAction> actions;
  final bool collapsed;
  final Selection selection;
  @override
  State<Harness> createState() => _HarnessState();
}

class _HarnessState extends State<Harness> {
  late Map<NodeRef, Offset> nodes = {...layout};
  late Selection selection = widget.selection;

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: NodeCanvas(
        project: design(),
        layout: nodes,
        selection: selection,
        context: const SystemContext(),
        groupsEnabled: true,
        groups: [readers()],
        system: SystemSceneInput(
          system: systemView(),
          groups: [readers()],
          boundaries: [boundary()],
          groupBoxes: {
            if (widget.collapsed)
              groupId: const GroupBox(rect: Rect.fromLTWH(560, 0, 208, 92), collapsed: true),
          },
        ),
        dispatch: (a) {
          widget.actions.add(a);
          setState(() {
            if (a is SelectionChanged) selection = a.selection;
            if (a is NodeMoved) nodes = {...nodes, a.node: a.position};
          });
        },
      ),
    ),
  );
}

CanvasScene sceneOf(Map<NodeRef, Offset> nodes, {bool collapsed = false}) => buildScene(
  design(),
  nodes,
  system: SystemSceneInput(
    system: systemView(),
    groups: [readers()],
    boundaries: [boundary()],
    groupBoxes: {
      if (collapsed) groupId: const GroupBox(rect: Rect.fromLTWH(560, 0, 208, 92), collapsed: true),
    },
  ),
);

Future<void> drag(WidgetTester tester, Offset from, Offset to) async {
  final origin = tester.getTopLeft(find.byType(NodeCanvas));
  final g = await tester.startGesture(origin + from, kind: PointerDeviceKind.mouse);
  await tester.pump(const Duration(milliseconds: 20));
  for (var i = 1; i <= 8; i++) {
    await g.moveTo(origin + Offset.lerp(from, to, i / 8)!);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await g.up();
  await tester.pump(const Duration(milliseconds: 20));
}

void main() {
  group('hit precedence', () {
    test('a socket beats its node, a node beats the region under it, the band beats nothing', () {
      final scene = sceneOf(layout);
      final aNode = scene.node(const NodeRef.mapping(a));
      final out = aNode.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
      expect(hitTest(scene, out.center), isA<HitSocket>());
      expect(hitTest(scene, aNode.rect.center), isA<HitNode>());
      final region = scene.groups.single;
      // inside the region but on no node: the region's band only counts at
      // its title band; the body is empty canvas (so box select and pan
      // still work over a group)
      expect(hitTest(scene, region.titleBand.center), isA<HitGroup>());
      final gap = Offset(region.rect.left + 8, aNode.rect.bottom + 20);
      expect(region.rect.contains(gap), isTrue);
      expect(hitTest(scene, gap), isA<HitNothing>());
      expect(groupAt(scene, gap)?.id, groupId);
    });

    test('a collapsed box is a node; its aggregate sockets are hit like any socket', () {
      final scene = sceneOf(layout, collapsed: true);
      expect(scene.nodes.any((n) => n.ref == const NodeRef.mapping(a)), isFalse);
      final box = scene.node(const NodeRef.group(groupId));
      expect(hitTest(scene, box.rect.center), isA<HitNode>());
      final agg = box.sockets.first;
      final hit = hitTest(scene, agg.center);
      expect(hit, isA<HitSocket>());
      expect((hit as HitSocket).socket.ref.role, SocketRole.aggregate);
      // an instance socket and an aggregate socket never confuse: roles
      expect(agg.ref.role, isNot(SocketRole.port));
    });

    test('the same scene reads at 0.25× and at 3×: geometry is zoom-free, summary is opt-in', () {
      final normal = sceneOf(layout);
      final summary = buildScene(
        design(),
        layout,
        system: SystemSceneInput(
          system: systemView(),
          groups: [readers()],
          boundaries: [boundary()],
          summarize: true,
        ),
      );
      expect(normal.groups, hasLength(1));
      expect(summary.groups, isEmpty, reason: 'at a low zoom every group is its box');
      final box = summary.node(const NodeRef.group(groupId));
      // the transient box stands where the region was, not at the origin
      expect(box.rect.topLeft, normal.groups.single.rect.topLeft);
    });
  });

  group('pointer', () {
    testWidgets(
      'a socket drag makes a concrete link; dragging a member out of its region removes it',
      (tester) async {
        tester.view.physicalSize = const Size(1600, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.reset);
        final actions = <AppAction>[];
        await tester.pumpWidget(Harness(actions: actions));
        final scene = sceneOf(layout);
        // A.out → openTilt's realisation socket: a binding between base ends
        final aOut = scene
            .node(const NodeRef.mapping(a))
            .sockets
            .firstWhere((s) => s.ref.side == SocketSide.output);
        final realise = scene
            .node(const NodeRef.mapping(openTilt))
            .sockets
            .firstWhere((s) => s.ref.role == SocketRole.realise);
        await drag(tester, aOut.center, realise.center);
        final link = actions.whereType<LinkEndsRequested>().single;
        expect(link.source.baseDecl.toInt(), a);
        expect(link.destination.baseDecl.toInt(), openTilt);

        // B dragged far below the region: it leaves the group (membership only)
        actions.clear();
        final bHeader = scene.node(const NodeRef.mapping(b)).header.center;
        await drag(tester, bHeader, bHeader + const Offset(0, 500));
        expect(actions.whereType<NodeMoved>(), hasLength(1));
        final removed = actions.whereType<RemoveGroupMemberRequested>().single;
        expect(removed.group, groupId);
        expect(removed.decl, b);
        expect(actions.whereType<UnlinkMappingInput>(), isEmpty, reason: 'no semantic edge moved');

        // follower dragged into the region: it joins
        actions.clear();
        final fHeader = sceneOf(layout).node(const NodeRef.mapping(follower)).header.center;
        final aRect = sceneOf(layout).node(const NodeRef.mapping(a)).rect;
        await drag(tester, fHeader, Offset(aRect.left + 100, aRect.bottom + 60));
        final added = actions.whereType<AddGroupMemberRequested>().single;
        expect(added.group, groupId);
        expect(added.decl, follower);
      },
    );

    testWidgets('a window marquee selects; ⌘-click toggles; Group as Behavior is offered for '
        'the free ones', (tester) async {
      tester.view.physicalSize = const Size(1600, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      final actions = <AppAction>[];
      await tester.pumpWidget(Harness(actions: actions));
      // left → right: a window over tiltValue, A and B (whole), and the
      // Readers region (a region is never taken by a rectangle)
      await drag(tester, const Offset(280, 5), const Offset(820, 260));
      final sel = actions.whereType<SelectionChanged>().last.selection;
      expect(sel, isA<MultiSelected>());
      expect((sel as MultiSelected).mappings.toSet(), {tiltValue, a, b});
      // ⌘-click follower: added and active; ⌘-click A: removed
      final origin = tester.getTopLeft(find.byType(NodeCanvas));
      final scene = sceneOf(layout);
      await tester.pump(const Duration(milliseconds: 400));
      await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
      await tester.tapAt(origin + scene.node(const NodeRef.mapping(follower)).header.center);
      await tester.pump(const Duration(milliseconds: 400));
      await tester.tapAt(origin + scene.node(const NodeRef.mapping(a)).header.center);
      await tester.pump(const Duration(milliseconds: 400));
      await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
      final sel2 = actions.whereType<SelectionChanged>().last.selection as MultiSelected;
      expect(sel2.mappings.toSet(), {tiltValue, b, follower});
      expect(sel2.active, const NodeRef.mapping(follower));
      // the contextual menu on one of them is about the set: grouping the
      // free ones (tiltValue and follower; B is already in Readers)
      await tester.tapAt(
        origin + scene.node(const NodeRef.mapping(follower)).header.center,
        buttons: kSecondaryButton,
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));
      expect(find.text('Group as Behavior (2 relationships)'), findsOneWidget);
      expect(find.text('Delete 3 objects'), findsOneWidget);
      await tester.tap(find.text('Group as Behavior (2 relationships)'));
      await tester.pump();
      expect(actions.whereType<GroupSelectionRequested>(), hasLength(1));
    });

    testWidgets(
      'aggregate sockets resolve to concrete declarations, never the group (socket_no_fanout)',
      (tester) async {
        tester.view.physicalSize = const Size(1600, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.reset);
        final actions = <AppAction>[];
        await tester.pumpWidget(Harness(actions: actions, collapsed: true));
        final scene = sceneOf(layout, collapsed: true);
        final box = scene.node(const NodeRef.group(groupId));
        final aggOut = box.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
        final aggIn = box.sockets.firstWhere((s) => s.ref.side == SocketSide.input);
        expect(box.socketLabels[aggOut.ref], 'A');
        expect(box.socketLabels[aggIn.ref], 'tiltValue');
        // out: the drag starts from A's own output socket
        final realise = scene
            .node(const NodeRef.mapping(openTilt))
            .sockets
            .firstWhere((s) => s.ref.role == SocketRole.realise);
        await drag(tester, aggOut.center, realise.center);
        final link = actions.whereType<LinkEndsRequested>().single;
        expect(link.source.hasBaseDecl(), isTrue);
        expect(link.source.baseDecl.toInt(), a, reason: 'the concrete member, not the group');
        // in: the Tilt concept dropped on the box reaches A (the one reader), never B
        actions.clear();
        final tiltOut = scene
            .node(const NodeRef.concept(tilt))
            .sockets
            .firstWhere((s) => s.ref.side == SocketSide.output);
        await drag(tester, tiltOut.center, aggIn.center);
        final read = actions.whereType<LinkConceptToMappingInput>().single;
        expect(read.mappingId, a);
        expect(read.conceptId, tilt);
        expect(actions.where((x) => x is LinkConceptToMappingInput && x.mappingId == b), isEmpty);
        // and the reverse: dragging a link *from* the aggregate input toward
        // the concept makes nothing (an input socket is not a source)
        actions.clear();
        await drag(tester, aggIn.center, tiltOut.center);
        expect(actions.whereType<LinkConceptToMappingInput>(), isEmpty);
        expect(actions.whereType<LinkEndsRequested>(), isEmpty);
      },
    );
  });
}
