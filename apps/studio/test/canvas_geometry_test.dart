/// The scene as the value graph (ADR-0044): a Sem block per unit-domain
/// relationship and, for each one with a definition, its mapping block as
/// a node of its own — read sockets for the Sem blocks the definition
/// names, slot sockets for its open positions, a produce edge into the
/// Sem block; no concept node, no rule node; read, produce and drive
/// edges; the drop rules.
library;

import 'dart:ui' show Size;

import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

const tiltC = 0, brightC = 1;
const tilt = 0, dimByTilt = 1, brightness = 2, light = 0;

/// `tilt : Tilt` (a Source), the rule `dimByTilt : Tilt -> Brightness`,
/// the Sem block `brightness : Brightness = dimByTilt(tilt)` driving the
/// sink `light`.
pb.ProjectProjection lamp({String? formula = 'dimByTilt(tilt)'}) =>
    pb.ProjectProjection(revision: Int64(1), name: 'lamp')
      ..concepts.addAll([
        pb.ConceptView(
          id: Int64(tiltC),
          name: 'Tilt',
          representation: pb.Representation(quantity: pb.Dim(angle: 1)),
        ),
        pb.ConceptView(id: Int64(brightC), name: 'Brightness'),
      ])
      ..mappings.addAll([
        mappingView(
          id: Int64(tilt),
          name: 'tilt',
          signature: pb.Signature(output: Int64(tiltC)),
        ),
        mappingView(
          id: Int64(dimByTilt),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(tiltC)], output: Int64(brightC)),
          definition: pb.Definition(formula: 'Tilt / 90 deg'),
        ),
        mappingView(
          id: Int64(brightness),
          name: 'brightness',
          signature: pb.Signature(output: Int64(brightC)),
          definition: formula == null ? null : pb.Definition(formula: formula),
          drivesOutputId: Int64(light),
        ),
      ])
      ..outputs.add(pb.OutputView(id: Int64(light), name: 'light', accepts: Int64(brightC)));

/// The analysis's read edges: brightness names tilt and the rule.
const refs = {
  brightness: [tilt, dimByTilt],
};

const semBlock = NodeRef.mapping(brightness);
const block = NodeRef.definition(brightness);

final placedLamp = <NodeRef, Offset>{
  const NodeRef.mapping(tilt): const Offset(48, 48),
  block: const Offset(368, 48),
  semBlock: const Offset(600, 48),
  const NodeRef.output(light): const Offset(900, 48),
};

NodeShape node(CanvasScene s, NodeRef ref) => s.nodes.firstWhere((n) => n.ref == ref);

void main() {
  test('the scene is Sem blocks, their mapping blocks and sinks: no concept, no rule', () {
    final scene = buildScene(lamp(), placedLamp, refs: refs);
    expect(scene.nodes.map((n) => n.ref).toSet(), {
      const NodeRef.mapping(tilt),
      semBlock,
      block,
      const NodeRef.output(light),
    });
    // the Source is a Sem block with no definition and no mapping block
    final src = node(scene, const NodeRef.mapping(tilt));
    expect(src.source, isTrue);
    expect(src.definition, isNull);
    expect(src.sockets.map((s) => s.ref.side), [SocketSide.output]);
    expect(src.rect.size, const Size(NodeMetrics.semWidth, NodeMetrics.semHeight));
    // the value: a Sem block with a produce socket on the left, its
    // concept at the output socket; the definition on its mapping block
    final b = node(scene, semBlock);
    expect(b.source, isFalse);
    expect(b.definition, isNull);
    expect(b.sockets.map((s) => s.ref.role), [SocketRole.produce, SocketRole.concept]);
    expect(b.socketLabels[b.sockets.last.ref], 'Brightness');
    final d = node(scene, block);
    expect(d.definition, 'dimByTilt(tilt)');
    expect(d.subtitle, 'brightness', reason: 'the block names the Sem block it defines');
    final reads = d.sockets.where((s) => s.ref.role == SocketRole.read).toList();
    expect(reads.length, 1);
    expect(reads.single.ref.concept, tiltC, reason: 'typed by the read block\'s concept');
    expect(reads.single.ref.index, tilt, reason: 'named by the read block');
    expect(reads.single.kind, SocketKind.quantity);
    expect(d.socketLabels[reads.single.ref], 'tilt');
    expect(d.dependsOn, ['tilt']);
    expect(d.applies, ['dimByTilt'], reason: 'the rule is a word on the block, not a node');
    expect(d.title, 'dimByTilt', reason: 'the header names the rule applied');
  });

  test('a read edge per Sem block named, a produce edge per definition, a drive per sink', () {
    final scene = buildScene(lamp(), placedLamp, refs: refs);
    expect(scene.links.length, 3);
    final read = scene.links.firstWhere((l) => l.to.node == block);
    expect(read.from.node, const NodeRef.mapping(tilt));
    expect(read.to.role, SocketRole.read);
    expect(read.concept, tiltC, reason: 'the hue is the read block\'s concept');
    expect(read.id.kind, LinkKind.read);
    final produce = scene.links.firstWhere((l) => l.to.node == semBlock);
    expect(produce.from.node, block);
    expect(produce.to.role, SocketRole.produce);
    expect(produce.concept, brightC);
    expect(produce.id.kind, LinkKind.produce);
    final drive = scene.links.firstWhere((l) => l.to.node == const NodeRef.output(light));
    expect(drive.from.node, semBlock);
    expect(drive.concept, brightC);
    expect(drive.id.kind, LinkKind.drive);
    // without an analysis there are no read edges yet; the rest is authored
    expect(buildScene(lamp(), placedLamp).links.map((l) => l.id.kind), [
      LinkKind.produce,
      LinkKind.drive,
    ]);
  });

  test('an open position of a definition is a hollow slot socket', () {
    final scene = buildScene(
      lamp(formula: 'dimByTilt(?)'),
      placedLamp,
      slots: {
        brightness: ['r.0'],
      },
    );
    final d = node(scene, block);
    final slot = d.sockets.where((s) => s.ref.role == SocketRole.slot).toList();
    expect(slot.length, 1);
    expect(slot.single.open, isTrue);
    expect(slot.single.kind, SocketKind.open);
    expect(d.socketLabels[slot.single.ref], '?');
    expect(d.acceptsBlock, isTrue);
    // a defined block with no open position accepts nothing as a whole
    expect(node(buildScene(lamp(), placedLamp), block).acceptsBlock, isFalse);
    // a Source does: the drop would give it a definition
    expect(node(scene, const NodeRef.mapping(tilt)).acceptsBlock, isTrue);
    // a Sem block with a definition never does itself: its mapping block may
    expect(node(scene, semBlock).acceptsBlock, isFalse);
  });

  test('positions come from the layout; a mapping block without one sits by its Sem block', () {
    final a = buildScene(lamp(), const {});
    expect(node(a, const NodeRef.mapping(tilt)).rect.topLeft, NodeMetrics.origin);
    expect(node(a, semBlock).rect.topLeft, NodeMetrics.origin);
    expect(node(a, block).rect.topLeft, NodeMetrics.origin, reason: 'nothing to attach to');
    final moved = buildScene(lamp(), {...placedLamp, semBlock: const Offset(900, 10)});
    expect(node(moved, semBlock).rect.topLeft, const Offset(900, 10));
    expect(node(moved, block).rect.topLeft, const Offset(368, 48), reason: 'its own position');
    final unplaced = buildScene(lamp(), {...placedLamp}..remove(block));
    expect(
      node(unplaced, block).rect.topLeft,
      attachedTo(const Offset(600, 48)),
      reason: 'attached left of the Sem block until the daemon places it',
    );
  });

  test('sockets are hit before nodes; drops: a slot takes any Sem block, a sink its concept', () {
    final scene = buildScene(
      lamp(formula: 'dimByTilt(?)'),
      placedLamp,
      slots: {
        brightness: ['r.0'],
      },
    );
    final src = node(scene, const NodeRef.mapping(tilt));
    final out = src.sockets.single;
    expect(hitTest(scene, out.center), isA<HitSocket>());
    expect(hitTest(scene, src.rect.center), isA<HitNode>());
    expect(hitTest(scene, const Offset(-500, -500)), isA<HitNothing>());
    final d = node(scene, block);
    final slot = d.sockets.firstWhere((s) => s.ref.role == SocketRole.slot);
    final b = node(scene, semBlock);
    final bOut = b.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
    final produceAt = b.sockets.firstWhere((s) => s.ref.role == SocketRole.produce);
    final sink = node(scene, const NodeRef.output(light)).sockets.single;
    // tilt (an angle) may go into the open position — the compiler types it
    expect(dropTarget(scene, out.ref, slot.center)?.ref, slot.ref);
    // tilt may not drive light (Brightness); brightness may
    expect(dropTarget(scene, out.ref, sink.center), isNull);
    expect(dropTarget(scene, bOut.ref, sink.center)?.ref, sink.ref);
    // into its own definition's open position too: a self-read is a text
    // the compiler types (memory, `delay(…)`), never refused here
    expect(dropTarget(scene, bOut.ref, slot.center)?.ref, slot.ref);
    // a produce socket is the joint of a block and its definition: no drop
    expect(canLink(out.ref, produceAt.ref), isFalse);
    // a mapping block's output starts nothing: it produces its own Sem block
    final dOut = d.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
    expect(canLink(dOut.ref, sink.ref), isFalse);
    // a read socket is a name in the formula: no drop
    final defined = buildScene(lamp(), placedLamp, refs: refs);
    final read = node(defined, block).sockets.firstWhere((s) => s.ref.role == SocketRole.read);
    expect(canLink(out.ref, read.ref), isFalse);
    // the whole node as a target: a Source, a mapping block with a slot,
    // or a Sem block whose mapping block has one
    expect(blockDropTarget(scene, out.ref, d.rect.center)?.ref, block);
    expect(blockDropTarget(scene, out.ref, b.rect.center)?.ref, block);
    expect(blockDropTarget(defined, out.ref, d.rect.center), isNull);
    expect(blockDropTarget(defined, out.ref, b.rect.center), isNull);
    expect(blockDropTarget(scene, bOut.ref, src.rect.center)?.ref, src.ref);
    // never its own block
    expect(blockDropTarget(scene, bOut.ref, d.rect.center), isNull);
  });

  test('the disclosure is on the mapping block', () {
    final scene = buildScene(lamp(), placedLamp, refs: refs);
    final d = node(scene, block);
    expect(hitTest(scene, d.disclosure.center), isA<HitDisclosure>());
    final b = node(scene, semBlock);
    expect(hitTest(scene, b.rect.center), isA<HitNode>());
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
      ..mappings.add(
        mappingView(
          id: Int64(9),
          name: 'held',
          signature: pb.Signature(output: Int64(2)),
        ),
      );
    final scene = buildScene(p, const {});
    expect(node(scene, const NodeRef.mapping(tilt)).sockets.single.kind, SocketKind.quantity);
    expect(node(scene, const NodeRef.mapping(tilt)).sockets.single.bound, isTrue);
    final b = node(scene, semBlock);
    final bOut = b.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
    expect(bOut.kind, SocketKind.open, reason: 'Brightness has no representation yet');
    expect(bOut.bound, isFalse);
    expect(node(scene, const NodeRef.mapping(9)).sockets.single.kind, SocketKind.onOff);
  });

  test('a definition that does not check is marked on the mapping block', () {
    final checked = buildScene(
      lamp(formula: 'tilt + 1 s'),
      const {},
      statuses: {brightness: pb.MappingStatus.MAPPING_STATUS_INVALID},
    );
    final d = node(checked, block);
    expect(d.definition, 'tilt + 1 s');
    expect(d.wrong, isTrue);
    expect(d.title, '', reason: 'no rule applied: the painter says "Formula"');
    expect(node(buildScene(lamp(), const {}), block).wrong, isFalse);
  });

  test('compatible sockets: the sink of its concept, and every open position', () {
    final scene = buildScene(
      lamp(formula: 'dimByTilt(?)'),
      placedLamp,
      slots: {
        brightness: ['r.0'],
      },
    );
    final b = node(scene, semBlock);
    final bOut = b.sockets.firstWhere((s) => s.ref.side == SocketSide.output);
    expect(compatibleSockets(scene, bOut.ref).map((s) => s.node).toSet(), {
      const NodeRef.output(light),
      block,
    });
    final src = node(scene, const NodeRef.mapping(tilt));
    final targets = compatibleSockets(scene, src.sockets.single.ref);
    expect(targets.length, 1);
    expect(targets.single.role, SocketRole.slot);
  });

  test('a selection of a mapping block is its Sem block; a read and a drive edge disconnect', () {
    expect(singleSelection(block), const MappingSelected(brightness));
    expect(selectionOfNodes({block, semBlock}), const MappingSelected(brightness));
    expect(asDeclaration(block), semBlock);
    const read = LinkId(from: NodeRef.mapping(tilt), to: block, concept: tiltC, index: tilt);
    const produce = LinkId(from: block, to: semBlock, concept: brightC);
    const drive = LinkId(from: semBlock, to: NodeRef.output(light), concept: brightC);
    expect(read.disconnectable, isTrue);
    expect(drive.disconnectable, isTrue);
    expect(produce.disconnectable, isFalse, reason: 'the definition goes with its editor');
  });

  test('dimension labels', () {
    expect(dimLabel(pb.Dim()), '1');
    expect(dimLabel(pb.Dim(angle: 1, time: -1)), 's^-1·rad');
  });
}
