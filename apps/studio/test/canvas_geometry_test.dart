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

  test('socket shape is the value form; an undecided one is a hollow ring', () {
    final p = lamp()
      ..concepts.add(
        pb.ConceptView(
          id: Int64(2),
          name: 'Held',
          representation: pb.Representation(boolean: pb.Unit()),
        ),
      )
      ..concepts.add(
        pb.ConceptView(
          id: Int64(3),
          name: 'Presses',
          representation: pb.Representation(count: pb.Unit()),
        ),
      );
    final scene = buildScene(p, const {});
    NodeShape node(int id) => scene.nodes.firstWhere((n) => n.ref == NodeRef.concept(id));
    expect(node(0).sockets.every((s) => s.kind == SocketKind.quantity && s.bound), isTrue);
    expect(node(1).sockets.every((s) => s.kind == SocketKind.open && !s.bound), isTrue);
    expect(node(2).sockets.every((s) => s.kind == SocketKind.onOff), isTrue);
    expect(node(3).sockets.every((s) => s.kind == SocketKind.count), isTrue);
    // the mapping's input socket carries the concept's form too
    final mapping = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(0));
    expect(mapping.sockets.first.kind, SocketKind.quantity);
    // a concept is one row: no type words, nothing but name and sockets
    expect(node(0).rect.height, NodeMetrics.conceptHeight);
    expect(node(0).definition, isNull);
  });

  test('declared is dashed and empty; a definition that does not check is marked', () {
    final scene = buildScene(lamp(), const {});
    final declared = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(0));
    expect(declared.declared, isTrue);
    expect(declared.definition, isNull);
    expect(declared.wrong, isFalse);

    final defined = lamp()..mappings.first.definition = pb.Definition(formula: 'Tilt + 1 s');
    final checked = buildScene(
      defined,
      const {},
      statuses: {0: pb.MappingStatus.MAPPING_STATUS_INVALID},
    );
    final m = checked.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(0));
    expect(m.declared, isFalse);
    expect(m.definition, 'Tilt + 1 s');
    expect(m.wrong, isTrue);
  });

  test('compatible sockets: same identity, other side, other node', () {
    final scene = buildScene(lamp(), const {});
    final tilt = scene.nodes.firstWhere((n) => n.ref == const NodeRef.concept(0));
    final out = tilt.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
    final targets = compatibleSockets(scene, out.ref);
    expect(targets.length, 1);
    final t = targets.single;
    expect(t.node, const NodeRef.mapping(0));
    expect(t.side, SocketSide.input);
    expect(t.concept, 0);
    expect(canLink(out.ref, tilt.sockets.first.ref), isFalse);
  });

  test('dimension labels', () {
    expect(dimLabel(pb.Dim()), '1');
    expect(dimLabel(pb.Dim(angle: 1, time: -1)), 's^-1·rad');
  });
}
