/// Pure geometry of the node canvas: where nodes, sockets and links are,
/// and what is under a point.  No widgets, no state — a function of the
/// projection and the layout, so it is unit-testable and deterministic.
///
/// Node anatomy follows Blender (docs/STUDIO_UI.md §2): header with title
/// and state word, input sockets down the left edge, output socket on the
/// right, body below.  Data flows left → right.
library;

import 'dart:ui';

import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

abstract final class NodeMetrics {
  static const double conceptWidth = 168;
  static const double mappingWidth = 200;
  static const double headerHeight = 26;
  static const double rowHeight = 22;
  static const double bodyHeight = 22;
  static const double socketRadius = 5.5;
  static const double socketHitRadius = 10;
  static const double cornerRadius = 7;

  /// Auto-placement grid for nodes without a stored position.
  static const double columnGap = 320;
  static const double rowGap = 96;
  static const Offset origin = Offset(48, 48);
}

enum SocketSide { input, output }

/// One socket: which node, which side, which concept it is typed by, and —
/// for mapping inputs — the input index.
class SocketRef {
  const SocketRef({required this.node, required this.side, required this.concept, this.index = 0});
  final NodeRef node;
  final SocketSide side;
  final int concept;
  final int index;

  @override
  bool operator ==(Object other) =>
      other is SocketRef &&
      other.node == node &&
      other.side == side &&
      other.concept == concept &&
      other.index == index;
  @override
  int get hashCode => Object.hash(node, side, concept, index);
}

class SocketShape {
  const SocketShape._(this.center, this.ref, this.bound);
  final Offset center;
  final SocketRef ref;

  /// Whether the concept's representation is bound (hollow ring otherwise).
  final bool bound;
}

class NodeShape {
  const NodeShape({
    required this.ref,
    required this.rect,
    required this.title,
    required this.subtitle,
    required this.stateWord,
    required this.sockets,
    required this.unresolved,
    this.socketLabels = const {},
  });
  final NodeRef ref;
  final Rect rect;
  final String title;
  final String subtitle;
  final String stateWord;
  final List<SocketShape> sockets;
  final bool unresolved;

  /// Label drawn next to each socket (mapping inputs).
  final Map<SocketRef, String> socketLabels;

  Rect get header => Rect.fromLTWH(rect.left, rect.top, rect.width, NodeMetrics.headerHeight);
}

class LinkShape {
  const LinkShape({
    required this.from,
    required this.to,
    required this.concept,
    required this.path,
  });
  final SocketRef from;
  final SocketRef to;
  final int concept;
  final Path path;
}

class CanvasScene {
  const CanvasScene({required this.nodes, required this.links});
  final List<NodeShape> nodes;
  final List<LinkShape> links;

  Rect get bounds {
    if (nodes.isEmpty) return const Rect.fromLTWH(0, 0, 400, 300);
    var r = nodes.first.rect;
    for (final n in nodes.skip(1)) {
      r = r.expandToInclude(n.rect);
    }
    return r;
  }
}

sealed class CanvasHit {
  const CanvasHit();
}

class HitNothing extends CanvasHit {
  const HitNothing();
}

class HitNode extends CanvasHit {
  const HitNode(this.node, {required this.header});
  final NodeShape node;
  final bool header;
}

class HitSocket extends CanvasHit {
  const HitSocket(this.socket, this.node);
  final SocketShape socket;
  final NodeShape node;
}

/// Build the scene.  Positions come from [layout]; anything missing is
/// auto-placed deterministically by id (concepts in column 0, mappings in
/// column 1) so an untouched project still reads left → right.
CanvasScene buildScene(pb.ProjectProjection p, Map<NodeRef, Offset> layout) {
  final concepts = [...p.concepts]..sort((a, b) => a.id.compareTo(b.id));
  final mappings = [...p.mappings]..sort((a, b) => a.id.compareTo(b.id));
  final bound = {for (final c in concepts) c.id.toInt(): c.hasRepresentation()};

  final nodes = <NodeShape>[];
  final socketByRef = <SocketRef, SocketShape>{};

  var row = 0;
  for (final c in concepts) {
    final ref = NodeRef.concept(c.id.toInt());
    final pos = layout[ref] ?? NodeMetrics.origin + Offset(0, row * NodeMetrics.rowGap);
    row++;
    final rect = Rect.fromLTWH(
      pos.dx,
      pos.dy,
      NodeMetrics.conceptWidth,
      NodeMetrics.headerHeight + NodeMetrics.rowHeight,
    );
    final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight / 2;
    final id = c.id.toInt();
    final inRef = SocketRef(node: ref, side: SocketSide.input, concept: id);
    final outRef = SocketRef(node: ref, side: SocketSide.output, concept: id);
    final sockets = [
      SocketShape._(Offset(rect.left, y), inRef, bound[id] ?? false),
      SocketShape._(Offset(rect.right, y), outRef, bound[id] ?? false),
    ];
    for (final s in sockets) {
      socketByRef[s.ref] = s;
    }
    nodes.add(
      NodeShape(
        ref: ref,
        rect: rect,
        title: c.name,
        subtitle: _representationLabel(c),
        stateWord: c.hasRepresentation() ? '' : 'open',
        sockets: sockets,
        unresolved: false,
      ),
    );
  }

  row = 0;
  for (final m in mappings) {
    final ref = NodeRef.mapping(m.id.toInt());
    final pos =
        layout[ref] ??
        NodeMetrics.origin + Offset(NodeMetrics.columnGap, row * NodeMetrics.rowGap * 1.3);
    row++;
    final inputs = m.signature.inputs.map((i) => i.toInt()).toList();
    final rows = inputs.isEmpty ? 1 : inputs.length;
    final rect = Rect.fromLTWH(
      pos.dx,
      pos.dy,
      NodeMetrics.mappingWidth,
      NodeMetrics.headerHeight + rows * NodeMetrics.rowHeight + NodeMetrics.bodyHeight,
    );
    final sockets = <SocketShape>[];
    final labels = <SocketRef, String>{};
    for (var i = 0; i < inputs.length; i++) {
      final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (i + 0.5);
      final r = SocketRef(node: ref, side: SocketSide.input, concept: inputs[i], index: i);
      final s = SocketShape._(Offset(rect.left, y), r, bound[inputs[i]] ?? false);
      sockets.add(s);
      socketByRef[r] = s;
      labels[r] = _conceptName(p, inputs[i]);
    }
    final outId = m.signature.output.toInt();
    final outRef = SocketRef(node: ref, side: SocketSide.output, concept: outId);
    final out = SocketShape._(
      Offset(rect.right, rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight / 2),
      outRef,
      bound[outId] ?? false,
    );
    sockets.add(out);
    socketByRef[outRef] = out;
    labels[outRef] = _conceptName(p, outId);
    nodes.add(
      NodeShape(
        ref: ref,
        rect: rect,
        title: m.name,
        subtitle: m.hasDefinition() ? m.definition.formula : 'no definition yet',
        stateWord: stateWord(m.state),
        sockets: sockets,
        unresolved: !m.hasDefinition(),
        socketLabels: labels,
      ),
    );
  }

  // Links: concept.out → mapping.in (per signature input); mapping.out → concept.in.
  final links = <LinkShape>[];
  for (final m in mappings) {
    final ref = NodeRef.mapping(m.id.toInt());
    final inputs = m.signature.inputs.map((i) => i.toInt()).toList();
    for (var i = 0; i < inputs.length; i++) {
      final from =
          socketByRef[SocketRef(
            node: NodeRef.concept(inputs[i]),
            side: SocketSide.output,
            concept: inputs[i],
          )];
      final to =
          socketByRef[SocketRef(node: ref, side: SocketSide.input, concept: inputs[i], index: i)];
      if (from != null && to != null) {
        links.add(
          LinkShape(
            from: from.ref,
            to: to.ref,
            concept: inputs[i],
            path: linkPath(from.center, to.center),
          ),
        );
      }
    }
    final outId = m.signature.output.toInt();
    final from = socketByRef[SocketRef(node: ref, side: SocketSide.output, concept: outId)];
    final to =
        socketByRef[SocketRef(
          node: NodeRef.concept(outId),
          side: SocketSide.input,
          concept: outId,
        )];
    if (from != null && to != null) {
      links.add(
        LinkShape(
          from: from.ref,
          to: to.ref,
          concept: outId,
          path: linkPath(from.center, to.center),
        ),
      );
    }
  }

  return CanvasScene(nodes: nodes, links: links);
}

/// Cubic Bézier with horizontal tangents, as Blender draws noodles.
Path linkPath(Offset a, Offset b) {
  final dx = (b.dx - a.dx).abs();
  final bend = (dx * 0.5).clamp(40.0, 160.0);
  return Path()
    ..moveTo(a.dx, a.dy)
    ..cubicTo(a.dx + bend, a.dy, b.dx - bend, b.dy, b.dx, b.dy);
}

/// Topmost thing under [point], sockets first (their hit area extends
/// outside the node), then nodes in reverse draw order.
CanvasHit hitTest(CanvasScene scene, Offset point) {
  for (final n in scene.nodes.reversed) {
    for (final s in n.sockets) {
      if ((s.center - point).distance <= NodeMetrics.socketHitRadius) return HitSocket(s, n);
    }
  }
  for (final n in scene.nodes.reversed) {
    if (n.rect.contains(point)) return HitNode(n, header: n.header.contains(point));
  }
  return const HitNothing();
}

/// The socket a dragged link may legally be dropped on: same concept,
/// opposite side, different node.  Typing by identity, made visible.
SocketShape? dropTarget(CanvasScene scene, SocketRef from, Offset point) {
  final hit = hitTest(scene, point);
  if (hit is! HitSocket) return null;
  final s = hit.socket.ref;
  if (s.node == from.node || s.side == from.side || s.concept != from.concept) return null;
  return hit.socket;
}

String stateWord(pb.AcceptanceState s) => switch (s) {
  pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED => 'declared',
  pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED => 'defined',
  pb.AcceptanceState.ACCEPTANCE_STATE_TYPE_VALID => 'type-valid',
  pb.AcceptanceState.ACCEPTANCE_STATE_TEMPORALLY_VALID => 'temporally valid',
  pb.AcceptanceState.ACCEPTANCE_STATE_CLOCK_CONSISTENT => 'clock-consistent',
  pb.AcceptanceState.ACCEPTANCE_STATE_OUTPUT_COMPLETE => 'output-complete',
  pb.AcceptanceState.ACCEPTANCE_STATE_HARDWARE_FEASIBLE => 'hardware-feasible',
  _ => '',
};

String _conceptName(pb.ProjectProjection p, int id) =>
    p.concepts.where((c) => c.id.toInt() == id).map((c) => c.name).firstOrNull ?? '?';

String _representationLabel(pb.ConceptView c) {
  if (!c.hasRepresentation()) return 'representation not chosen';
  final r = c.representation;
  return switch (r.whichKind()) {
    pb.Representation_Kind.quantity => 'quantity ${dimLabel(r.quantity)}',
    pb.Representation_Kind.boolean => 'boolean',
    pb.Representation_Kind.count => 'count',
    pb.Representation_Kind.notSet => 'representation not chosen',
  };
}

/// `m·s⁻²`-style label for a dimension; `1` when dimensionless.
String dimLabel(pb.Dim d) {
  final parts = <String>[];
  void add(String sym, int e) {
    if (e == 0) return;
    parts.add(e == 1 ? sym : '$sym^$e');
  }

  add('m', d.length);
  add('kg', d.mass);
  add('s', d.time);
  add('A', d.current);
  add('K', d.temperature);
  add('mol', d.amount);
  add('cd', d.luminous);
  add('rad', d.angle);
  return parts.isEmpty ? '1' : parts.join('·');
}
