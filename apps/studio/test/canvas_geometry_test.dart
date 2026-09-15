import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

pb.ProjectProjection lamp() => pb.ProjectProjection(revision: Int64(1), name: 'lamp')
  ..concepts.addAll([
    pb.ConceptView(
      id: Int64(0),
      name: 'Tilt',
      representation: pb.Representation(quantity: pb.Dim(angle: 1)),
    ),
    pb.ConceptView(id: Int64(1), name: 'Brightness'),
  ])
  ..mappings.add(
    pb.MappingView(
      id: Int64(0),
      name: 'dimByTilt',
      signature: pb.Signature(inputs: [Int64(0)], output: Int64(1)),
      state: pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED,
    ),
  );

void main() {
  test('scene has one link per signature input plus one for the output', () {
    final scene = buildScene(lamp(), const {});
    expect(scene.nodes.length, 3);
    expect(scene.links.length, 2);
    final byConcept = scene.links.map((l) => l.concept).toList()..sort();
    expect(byConcept, [0, 1]);
  });

  test('auto placement is deterministic and left-to-right; stored layout wins', () {
    final a = buildScene(lamp(), const {});
    final b = buildScene(lamp(), const {});
    expect(a.nodes.map((n) => n.rect).toList(), b.nodes.map((n) => n.rect).toList());
    final concept = a.nodes.firstWhere((n) => n.ref == const NodeRef.concept(0));
    final mapping = a.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(0));
    expect(concept.rect.left < mapping.rect.left, isTrue);

    final moved = buildScene(lamp(), {const NodeRef.mapping(0): const Offset(900, 10)});
    expect(
      moved.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(0)).rect.topLeft,
      const Offset(900, 10),
    );
  });

  test('sockets are hit before nodes, and only same-concept opposite sockets accept drops', () {
    final scene = buildScene(lamp(), const {});
    final mapping = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(0));
    final out = mapping.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
    expect(hitTest(scene, out.center), isA<HitSocket>());
    expect(hitTest(scene, mapping.rect.center), isA<HitNode>());
    expect(hitTest(scene, const Offset(-500, -500)), isA<HitNothing>());

    // mapping output (Brightness) may drop on Brightness.in, not on Tilt.in
    final brightness = scene.nodes.firstWhere((n) => n.ref == const NodeRef.concept(1));
    final tilt = scene.nodes.firstWhere((n) => n.ref == const NodeRef.concept(0));
    final bIn = brightness.sockets.firstWhere((s) => s.ref.side == SocketSide.input);
    final tIn = tilt.sockets.firstWhere((s) => s.ref.side == SocketSide.input);
    expect(dropTarget(scene, out.ref, bIn.center)?.ref, bIn.ref);
    expect(dropTarget(scene, out.ref, tIn.center), isNull);
    // never onto the same node
    expect(dropTarget(scene, out.ref, mapping.sockets.first.center), isNull);
  });

  test('bound representation fills the socket; open concepts are hollow', () {
    final scene = buildScene(lamp(), const {});
    final tilt = scene.nodes.firstWhere((n) => n.ref == const NodeRef.concept(0));
    final bright = scene.nodes.firstWhere((n) => n.ref == const NodeRef.concept(1));
    expect(tilt.sockets.every((s) => s.bound), isTrue);
    expect(bright.sockets.every((s) => !s.bound), isTrue);
    expect(bright.stateWord, 'open');
  });

  test('dimension labels', () {
    expect(dimLabel(pb.Dim()), '1');
    expect(dimLabel(pb.Dim(angle: 1, time: -1)), 's^-1·rad');
  });
}
