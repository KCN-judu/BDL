/// Pure geometry of the node canvas: where nodes, sockets and links are,
/// and what is under a point.  No widgets, no state — a function of the
/// projection and the layout, so it is unit-testable and deterministic.
///
/// Node anatomy follows Blender (docs/STUDIO_UI.md §2).  A concept is a
/// single-row object: its name, an input socket (what produces it) on the
/// left and an output socket (what reads it) on the right.  A mapping has a
/// header with title and state word, one input socket per read concept, the
/// output socket on the right, and a definition region below.  Data flows
/// left → right.
///
/// Semantics are carried by the geometry, not by words (STUDIO_UI.md §7):
/// socket hue is identity, socket *shape* is the concept's value form, a
/// hollow ring means the form is not chosen yet, a dashed outline means
/// declared-not-defined, and a red mark at the definition line means the
/// definition does not check.
///
/// A system canvas (docs/STUDIO_UI.md §11) adds component-instance nodes
/// drawn from their ports' *contracts* (never their bodies), binding links
/// between ports (a transport gate where a value is carried across timing
/// domains), a realisation socket on an open base relationship (where a
/// provided port may realise it), and behaviour-group regions: a tinted
/// area around the members while expanded, a box with aggregate sockets
/// while collapsed.  Aggregate sockets are a picture of the group's
/// boundary (crossing in / out), never a declaration.
library;

import 'dart:ui';

import 'package:fixnum/fixnum.dart';

import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

abstract final class NodeMetrics {
  static const double conceptWidth = 168;
  static const double conceptHeight = 26;
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

  /// Component-instance and collapsed-group nodes.
  static const double instanceWidth = 208;

  /// Padding of an expanded group region around its members, and the
  /// height of its title band.
  static const double regionPadding = 16;
  static const double regionTitle = 22;
}

enum SocketSide { input, output }

/// What a socket stands for beyond its concept: a mapping's read or
/// produce ([concept]); a port of an instance ([port], [index] = port id);
/// the realisation of an open base relationship ([realise]: where a
/// provided port may bind); an aggregate socket of a collapsed group
/// ([aggregate], [index] = the declaration it stands for — a view).
enum SocketRole { concept, port, realise, aggregate }

/// The realisation socket's index (a mapping has no input at −1).
const int realiseIndex = -1;

/// One socket: which node, which side, which concept it is typed by, and —
/// for mapping inputs — the input index (for ports and aggregates, the
/// port or declaration id).
class SocketRef {
  const SocketRef({
    required this.node,
    required this.side,
    required this.concept,
    this.index = 0,
    this.role = SocketRole.concept,
  });
  final NodeRef node;
  final SocketSide side;
  final int concept;
  final int index;
  final SocketRole role;

  @override
  bool operator ==(Object other) =>
      other is SocketRef &&
      other.node == node &&
      other.side == side &&
      other.concept == concept &&
      other.index == index &&
      other.role == role;
  @override
  int get hashCode => Object.hash(node, side, concept, index, role);

  /// The binding end this socket is, on a system canvas: an instance's
  /// port, or the base relationship it realises / provides.
  pb.PortRefView? get bindingEnd => switch (role) {
    SocketRole.port => pb.PortRefView(instance: Int64(node.id), port: Int64(index)),
    SocketRole.realise => pb.PortRefView(baseDecl: Int64(node.id)),
    SocketRole.concept when node.kind == NodeKind.mapping && side == SocketSide.output =>
      pb.PortRefView(baseDecl: Int64(node.id)),
    _ => null,
  };
}

/// The value form of the concept a socket carries — drawn as the socket's
/// shape: ○ quantity, ◇ on–off, □ count; a hollow ring while undecided.
enum SocketKind { open, quantity, onOff, count }

SocketKind socketKind(pb.ConceptView c) {
  if (!c.hasRepresentation()) return SocketKind.open;
  return switch (c.representation.whichKind()) {
    pb.Representation_Kind.quantity => SocketKind.quantity,
    pb.Representation_Kind.boolean => SocketKind.onOff,
    pb.Representation_Kind.count => SocketKind.count,
    pb.Representation_Kind.notSet => SocketKind.open,
  };
}

class SocketShape {
  const SocketShape._(this.center, this.ref, this.kind, {this.open = false});
  final Offset center;
  final SocketRef ref;
  final SocketKind kind;

  /// A required port or parameter nobody has bound, or an open base
  /// relationship's realisation socket: drawn hollow — the value form is
  /// known, the value is not supplied.  A legal state, never an error.
  final bool open;

  /// Whether the concept's value form is chosen (hollow ring otherwise).
  bool get bound => kind != SocketKind.open;
}

/// Where a physical output stands, as the output pass says (an open sink
/// — no domain yet — is not a kernel output at all).
enum SinkState { open, undriven, driven, illFormed, contested }

class NodeShape {
  const NodeShape({
    required this.ref,
    required this.rect,
    required this.title,
    required this.sockets,
    this.definition,
    this.declared = false,
    this.wrong = false,
    this.socketLabels = const {},
    this.timing = '',
    this.required = false,
    this.sink,
    this.subtitle = '',
    this.headerWord = '',
    this.unrealized = false,
  });
  final NodeRef ref;
  final Rect rect;
  final String title;
  final List<SocketShape> sockets;

  /// An instance node: the component's name, in the body row.  A collapsed
  /// group: how many relationships it holds.
  final String subtitle;

  /// A word the geometry cannot carry, at the header's right: a port-backed
  /// relationship in a component's source (*requires*, *provides*,
  /// *parameter*).
  final String headerWord;

  /// An instance whose component's body no longer keeps its promise
  /// (`Realizes` fails): a red mark in the body row.
  final bool unrealized;

  /// The timing domain the node updates in ('' when agnostic or open): a
  /// quiet word at the right of the body, never a badge.
  final String timing;

  /// A physical output the design must drive.
  final bool required;

  /// For an output node: its state in the output pass.
  final SinkState? sink;

  /// One-line summary of a mapping's definition; `null` when there is none
  /// (the definition region stays empty) and for concepts.
  final String? definition;

  /// Declared, not yet defined: dashed outline, the word *declared*.  A
  /// legal state, never an error.
  final bool declared;

  /// The definition does not check: a red mark at the definition line.
  final bool wrong;

  /// Label drawn next to each socket (mapping inputs and output).
  final Map<SocketRef, String> socketLabels;

  Rect get header => Rect.fromLTWH(rect.left, rect.top, rect.width, NodeMetrics.headerHeight);

  /// The definition region of a mapping (below the socket rows).
  Rect get definitionRegion => Rect.fromLTWH(
    rect.left,
    rect.bottom - NodeMetrics.bodyHeight,
    rect.width,
    NodeMetrics.bodyHeight,
  );
}

class LinkShape {
  const LinkShape({
    required this.from,
    required this.to,
    required this.concept,
    required this.path,
    this.binding,
    this.transport,
  });
  final SocketRef from;
  final SocketRef to;
  final int concept;
  final Path path;

  /// The binding this link is, on a system canvas.
  final int? binding;

  /// The initial value of a transported binding: a gate is drawn on the
  /// link where the value crosses timing domains.
  final String? transport;

  Offset get midpoint {
    final metrics = path.computeMetrics().first;
    return metrics.getTangentForOffset(metrics.length / 2)?.position ?? Offset.zero;
  }
}

/// A behaviour group on the canvas: an expanded region around its members
/// (a background with a title band), or — collapsed — a node-like box
/// whose sockets are the group's boundary.
class GroupShape {
  const GroupShape({
    required this.id,
    required this.rect,
    required this.title,
    required this.collapsed,
    required this.members,
  });
  final int id;
  final Rect rect;
  final String title;
  final bool collapsed;
  final List<int> members;

  Rect get titleBand => Rect.fromLTWH(rect.left, rect.top, rect.width, NodeMetrics.regionTitle);
}

class CanvasScene {
  const CanvasScene({required this.nodes, required this.links, this.groups = const []});
  final List<NodeShape> nodes;
  final List<LinkShape> links;

  /// Expanded group regions (their collapsed counterparts are nodes of
  /// kind [NodeKind.group]).
  final List<GroupShape> groups;

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

/// The title band of an expanded group region.
class HitGroup extends CanvasHit {
  const HitGroup(this.group);
  final GroupShape group;
}

class HitLink extends CanvasHit {
  const HitLink(this.link);
  final LinkShape link;
}

/// Build the scene.  Positions come from [layout]; anything missing is
/// auto-placed deterministically by id (concepts in column 0, mappings in
/// column 1) so an untouched project still reads left → right.
/// What a system canvas adds to a flat one: the system (instances, their
/// contracts, bindings, groups), its analysis (port statuses, boundaries,
/// realizes), and the groups' boxes.  Absent for a flat project and for a
/// component's source (where the body is an ordinary design and
/// [portWords] name the port-backed relationships).
class SystemSceneInput {
  const SystemSceneInput({
    this.system,
    this.analysis,
    this.groupBoxes = const {},
    this.portWords = const {},
  });
  final pb.SystemView? system;
  final pb.SystemAnalysisView? analysis;
  final Map<int, GroupBox> groupBoxes;
  final Map<int, String> portWords;
}

CanvasScene buildScene(
  pb.ProjectProjection p,
  Map<NodeRef, Offset> layout, {
  Map<int, pb.MappingStatus> statuses = const {},
  Map<int, pb.OutputState> outputStates = const {},
  SystemSceneInput system = const SystemSceneInput(),
}) {
  final concepts = [...p.concepts]..sort((a, b) => a.id.compareTo(b.id));
  final mappings = [...p.mappings]..sort((a, b) => a.id.compareTo(b.id));
  final outputs = [...p.outputs]..sort((a, b) => a.id.compareTo(b.id));
  final kinds = {for (final c in concepts) c.id.toInt(): socketKind(c)};
  SocketKind kindOf(int id) => kinds[id] ?? SocketKind.open;
  String clockName(Int64 id) =>
      p.clocks.where((c) => c.id == id).map((c) => c.name).firstOrNull ?? '';
  final sys = system.system;
  // A base relationship realised by a binding (its definition is a
  // reference the system made): shown as "= source" on the node.
  final realisedBy = <int, String>{};
  final boundInto = <int>{};
  if (sys != null) {
    for (final b in sys.bindings) {
      if (b.destination.hasBaseDecl()) {
        realisedBy[b.destination.baseDecl.toInt()] = _endLabel(sys, b.source);
      } else {
        boundInto.add(_portKey(b.destination.instance.toInt(), b.destination.port.toInt()));
      }
    }
  }
  // Members of collapsed groups are not drawn; the group box stands in.
  final hiddenMembers = <int, int>{};
  if (sys != null) {
    for (final g in sys.groups) {
      final box = system.groupBoxes[g.id.toInt()];
      if (box != null && box.collapsed) {
        for (final m in g.members) {
          hiddenMembers[m.toInt()] = g.id.toInt();
        }
      }
    }
  }

  final nodes = <NodeShape>[];
  final socketByRef = <SocketRef, SocketShape>{};

  var row = 0;
  for (final c in concepts) {
    final ref = NodeRef.concept(c.id.toInt());
    final pos = layout[ref] ?? NodeMetrics.origin + Offset(0, row * NodeMetrics.rowGap);
    row++;
    final rect = Rect.fromLTWH(pos.dx, pos.dy, NodeMetrics.conceptWidth, NodeMetrics.conceptHeight);
    final y = rect.center.dy;
    final id = c.id.toInt();
    final inRef = SocketRef(node: ref, side: SocketSide.input, concept: id);
    final outRef = SocketRef(node: ref, side: SocketSide.output, concept: id);
    final sockets = [
      SocketShape._(Offset(rect.left, y), inRef, kindOf(id)),
      SocketShape._(Offset(rect.right, y), outRef, kindOf(id)),
    ];
    for (final s in sockets) {
      socketByRef[s.ref] = s;
    }
    nodes.add(NodeShape(ref: ref, rect: rect, title: c.name, sockets: sockets));
  }

  row = 0;
  for (final m in mappings) {
    final ref = NodeRef.mapping(m.id.toInt());
    final pos =
        layout[ref] ??
        NodeMetrics.origin + Offset(NodeMetrics.columnGap, row * NodeMetrics.rowGap * 1.3);
    row++;
    if (hiddenMembers.containsKey(m.id.toInt())) continue;
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
    // An open base relationship of a system can be realised by a provided
    // port: a socket at its definition row, hollow until bound.
    if (sys != null && !m.hasDefinition()) {
      final outId = m.signature.output.toInt();
      final r = SocketRef(
        node: ref,
        side: SocketSide.input,
        concept: outId,
        index: realiseIndex,
        role: SocketRole.realise,
      );
      final sock = SocketShape._(
        Offset(rect.left, rect.bottom - NodeMetrics.bodyHeight / 2),
        r,
        kindOf(outId),
        open: !realisedBy.containsKey(m.id.toInt()),
      );
      sockets.add(sock);
      socketByRef[r] = sock;
    }
    for (var i = 0; i < inputs.length; i++) {
      final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (i + 0.5);
      final r = SocketRef(node: ref, side: SocketSide.input, concept: inputs[i], index: i);
      final s = SocketShape._(Offset(rect.left, y), r, kindOf(inputs[i]));
      sockets.add(s);
      socketByRef[r] = s;
      labels[r] = _conceptName(p, inputs[i]);
    }
    final outId = m.signature.output.toInt();
    final outRef = SocketRef(node: ref, side: SocketSide.output, concept: outId);
    final out = SocketShape._(
      Offset(rect.right, rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight / 2),
      outRef,
      kindOf(outId),
    );
    sockets.add(out);
    socketByRef[outRef] = out;
    labels[outRef] = _conceptName(p, outId);
    final realised = realisedBy[m.id.toInt()];
    nodes.add(
      NodeShape(
        ref: ref,
        rect: rect,
        title: m.name,
        definition: m.hasDefinition()
            ? definitionSummary(m.definition)
            : realised == null
            ? null
            : '= $realised',
        declared: !m.hasDefinition() && realised == null,
        wrong: statuses[m.id.toInt()] == pb.MappingStatus.MAPPING_STATUS_INVALID,
        sockets: sockets,
        socketLabels: labels,
        timing: m.hasClockId() ? clockName(m.clockId) : '',
        headerWord: system.portWords[m.id.toInt()] ?? '',
      ),
    );
  }

  // Component instances: one node per instance, rows from the component's
  // ports' contracts — required and parameters on the left, provided on the
  // right — typed by the concept each port carries in the system (a shared
  // concept's own identity, a private one's flat identity).
  final instanceNodes = <NodeShape>[];
  if (sys != null) {
    final instances = [...sys.instances]..sort((a, b) => a.id.compareTo(b.id));
    final realizes = {
      for (final c in system.analysis?.components ?? const <pb.ComponentStatusView>[])
        c.id.toInt(): c.realizes,
    };
    row = 0;
    for (final inst in instances) {
      final comp = sys.components.where((c) => c.id == inst.component).firstOrNull;
      if (comp == null) continue;
      final ref = NodeRef.instance(inst.id.toInt());
      final pos =
          layout[ref] ??
          NodeMetrics.origin + Offset(NodeMetrics.columnGap, row * NodeMetrics.rowGap * 1.5 + 400);
      row++;
      final ports = [...comp.ports]..sort((a, b) => a.id.compareTo(b.id));
      final rows = ports.isEmpty ? 1 : ports.length;
      final rect = Rect.fromLTWH(
        pos.dx,
        pos.dy,
        NodeMetrics.instanceWidth,
        NodeMetrics.headerHeight + rows * NodeMetrics.rowHeight + NodeMetrics.bodyHeight,
      );
      final sockets = <SocketShape>[];
      final labels = <SocketRef, String>{};
      for (var i = 0; i < ports.length; i++) {
        final port = ports[i];
        final provided = port.kind == pb.PortKind.PORT_KIND_PROVIDED;
        final concept = _systemConcept(sys, comp, inst, port.contract.signature.output);
        final r = SocketRef(
          node: ref,
          side: provided ? SocketSide.output : SocketSide.input,
          concept: concept,
          index: port.id.toInt(),
          role: SocketRole.port,
        );
        final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (i + 0.5);
        final valued = inst.parameterBindings.any((b) => b.port == port.id);
        final s = SocketShape._(
          Offset(provided ? rect.right : rect.left, y),
          r,
          _contractKind(sys, comp, port.contract),
          open:
              !provided &&
              !valued &&
              !boundInto.contains(_portKey(inst.id.toInt(), port.id.toInt())),
        );
        sockets.add(s);
        socketByRef[r] = s;
        final value = inst.parameterBindings.where((b) => b.port == port.id).firstOrNull;
        labels[r] = value == null ? port.name : '${port.name} = ${value.source}';
      }
      final domains = [
        for (final b in inst.clockBindings)
          sys.base.clocks.where((c) => c.id == b.system).map((c) => c.name).firstOrNull ?? '',
      ].where((n) => n.isNotEmpty).toList();
      instanceNodes.add(
        NodeShape(
          ref: ref,
          rect: rect,
          title: inst.name,
          subtitle: comp.name,
          sockets: sockets,
          socketLabels: labels,
          timing: domains.join(', '),
          unrealized: realizes[comp.id.toInt()] == false,
        ),
      );
    }
  }

  // Physical outputs: sinks in a third column.  One socket on the left,
  // typed by the concept the sink accepts, no output socket — to the right
  // is the world.
  row = 0;
  for (final o in outputs) {
    final ref = NodeRef.output(o.id.toInt());
    final pos =
        layout[ref] ??
        NodeMetrics.origin + Offset(NodeMetrics.columnGap * 2, row * NodeMetrics.rowGap);
    row++;
    final rect = Rect.fromLTWH(
      pos.dx,
      pos.dy,
      NodeMetrics.conceptWidth,
      NodeMetrics.headerHeight + NodeMetrics.rowHeight,
    );
    final accepts = o.accepts.toInt();
    final inRef = SocketRef(node: ref, side: SocketSide.input, concept: accepts);
    final socket = SocketShape._(
      Offset(rect.left, rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight / 2),
      inRef,
      kindOf(accepts),
    );
    socketByRef[inRef] = socket;
    final sink = !o.hasClockId()
        ? SinkState.open
        : switch (outputStates[o.id.toInt()]) {
            pb.OutputState.OUTPUT_STATE_DRIVEN => SinkState.driven,
            pb.OutputState.OUTPUT_STATE_ILL_FORMED => SinkState.illFormed,
            pb.OutputState.OUTPUT_STATE_CONFLICT => SinkState.contested,
            _ => SinkState.undriven,
          };
    nodes.add(
      NodeShape(
        ref: ref,
        rect: rect,
        title: o.name,
        sockets: [socket],
        socketLabels: {inRef: _conceptName(p, accepts)},
        timing: o.hasClockId() ? clockName(o.clockId) : '',
        required: o.required,
        sink: sink,
      ),
    );
  }

  nodes.addAll(instanceNodes);

  // Groups: expanded ones are regions around their members; collapsed ones
  // are nodes whose sockets are the boundary (crossing in → left, crossing
  // out and driven members → right).  A picture of the dependency graph's
  // cut, never a declaration.
  final groups = <GroupShape>[];
  final groupSockets = <int, ({Map<int, SocketRef> ins, Map<int, SocketRef> outs})>{};
  if (sys != null) {
    final byId = {for (final n in nodes) n.ref: n};
    for (final g in sys.groups) {
      final id = g.id.toInt();
      final members = g.members.map((m) => m.toInt()).toList();
      final box = system.groupBoxes[id];
      final boundary =
          sys.boundaries.where((b) => b.id == g.id).firstOrNull ??
          system.analysis?.groups.where((b) => b.id == g.id).firstOrNull;
      if (box != null && box.collapsed) {
        final ins = <int>[...?boundary?.externalInputs.map((d) => d.toInt())];
        final outs = <int>[
          ...?boundary?.externalOutputs.map((d) => d.toInt()),
          for (final d in boundary?.drivenMembers ?? const <Int64>[])
            if (!(boundary?.externalOutputs.contains(d) ?? false)) d.toInt(),
        ];
        final rows = ins.length > outs.length ? ins.length : outs.length;
        final ref = NodeRef.group(id);
        final origin = box.rect == Rect.zero ? NodeMetrics.origin : box.rect.topLeft;
        final rect = Rect.fromLTWH(
          origin.dx,
          origin.dy,
          NodeMetrics.instanceWidth,
          NodeMetrics.headerHeight +
              (rows == 0 ? 1 : rows) * NodeMetrics.rowHeight +
              NodeMetrics.bodyHeight,
        );
        final sockets = <SocketShape>[];
        final labels = <SocketRef, String>{};
        final inRefs = <int, SocketRef>{};
        final outRefs = <int, SocketRef>{};
        int conceptOfDecl(int d) =>
            p.mappings.where((m) => m.id.toInt() == d).firstOrNull?.signature.output.toInt() ?? -1;
        String nameOfDecl(int d) =>
            p.mappings.where((m) => m.id.toInt() == d).firstOrNull?.name ?? '?';
        for (var i = 0; i < ins.length; i++) {
          final c = conceptOfDecl(ins[i]);
          final r = SocketRef(
            node: ref,
            side: SocketSide.input,
            concept: c,
            index: ins[i],
            role: SocketRole.aggregate,
          );
          final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (i + 0.5);
          final open = boundary?.openMembers.any((d) => d.toInt() == ins[i]) ?? false;
          final sock = SocketShape._(Offset(rect.left, y), r, kindOf(c), open: open);
          sockets.add(sock);
          socketByRef[r] = sock;
          inRefs[ins[i]] = r;
          labels[r] = nameOfDecl(ins[i]);
        }
        for (var i = 0; i < outs.length; i++) {
          final c = conceptOfDecl(outs[i]);
          final r = SocketRef(
            node: ref,
            side: SocketSide.output,
            concept: c,
            index: outs[i],
            role: SocketRole.aggregate,
          );
          final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (i + 0.5);
          final sock = SocketShape._(Offset(rect.right, y), r, kindOf(c));
          sockets.add(sock);
          socketByRef[r] = sock;
          outRefs[outs[i]] = r;
          labels[r] = nameOfDecl(outs[i]);
        }
        groupSockets[id] = (ins: inRefs, outs: outRefs);
        nodes.add(
          NodeShape(
            ref: ref,
            rect: rect,
            title: g.name,
            subtitle: '${members.length} relationship${members.length == 1 ? '' : 's'}',
            sockets: sockets,
            socketLabels: labels,
          ),
        );
      } else {
        final rects = [
          for (final m in members)
            if (byId[NodeRef.mapping(m)] case final n?) n.rect,
        ];
        final rect = rects.isEmpty
            ? Rect.fromLTWH(
                (box?.rect ?? Rect.zero).left,
                (box?.rect ?? Rect.zero).top,
                NodeMetrics.mappingWidth + 2 * NodeMetrics.regionPadding,
                NodeMetrics.regionTitle + NodeMetrics.headerHeight + 2 * NodeMetrics.regionPadding,
              )
            : rects
                  .reduce((a, b) => a.expandToInclude(b))
                  .inflate(NodeMetrics.regionPadding)
                  .let(
                    (r) =>
                        Rect.fromLTRB(r.left, r.top - NodeMetrics.regionTitle, r.right, r.bottom),
                  );
        groups.add(
          GroupShape(id: id, rect: rect, title: g.name, collapsed: false, members: members),
        );
      }
    }
  }

  // Links: concept.out → mapping.in (per signature input); mapping.out → concept.in.
  final links = <LinkShape>[];
  // A hidden member's produce and drive edges leave from its group's
  // aggregate output socket; what crosses in is drawn below, from the
  // crossing-in relationship to the aggregate input socket.
  SocketShape? groupOut(int member) {
    final g = hiddenMembers[member];
    final r = g == null ? null : groupSockets[g]?.outs[member];
    return r == null ? null : socketByRef[r];
  }

  for (final m in mappings) {
    final ref = NodeRef.mapping(m.id.toInt());
    final hidden = hiddenMembers.containsKey(m.id.toInt());
    if (hidden) {
      // The member's produce and drive edges leave from the group's socket.
      final outId = m.signature.output.toInt();
      final from = groupOut(m.id.toInt());
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
      if (m.hasDrivesOutputId()) {
        final sink = outputs.where((o) => o.id == m.drivesOutputId).firstOrNull;
        final toSink = sink == null
            ? null
            : socketByRef[SocketRef(
                node: NodeRef.output(sink.id.toInt()),
                side: SocketSide.input,
                concept: sink.accepts.toInt(),
              )];
        if (from != null && toSink != null) {
          links.add(
            LinkShape(
              from: from.ref,
              to: toSink.ref,
              concept: outId,
              path: linkPath(from.center, toSink.center),
            ),
          );
        }
      }
      continue;
    }
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
    // The drive edge, as authored: mapping.out → sink.  Whether it is well
    // formed is the output pass's verdict, shown on the sink.
    if (m.hasDrivesOutputId()) {
      final sink = outputs.where((o) => o.id == m.drivesOutputId).firstOrNull;
      final to = sink == null
          ? null
          : socketByRef[SocketRef(
              node: NodeRef.output(sink.id.toInt()),
              side: SocketSide.input,
              concept: sink.accepts.toInt(),
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
  }

  // Dependency edges into a collapsed group: from each crossing-in
  // relationship's output socket to the aggregate input socket that stands
  // for it (Theorem H: the socket adds no edge; it *is* the edge).
  for (final entry in groupSockets.entries) {
    for (final e in entry.value.ins.entries) {
      final decl = e.key;
      final m = mappings.where((m) => m.id.toInt() == decl).firstOrNull;
      if (m == null || hiddenMembers.containsKey(decl)) continue;
      final outId = m.signature.output.toInt();
      final from =
          socketByRef[SocketRef(
            node: NodeRef.mapping(decl),
            side: SocketSide.output,
            concept: outId,
          )];
      final to = socketByRef[e.value];
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
  }

  // Bindings: provided port → required port / parameter, base relationship
  // → required port, provided port → open base relationship.  A transport
  // is a gate on the link.
  if (sys != null) {
    for (final b in sys.bindings) {
      final from = _endSocket(sys, socketByRef, b.source, SocketSide.output);
      final to = _endSocket(sys, socketByRef, b.destination, SocketSide.input);
      if (from == null || to == null) continue;
      links.add(
        LinkShape(
          from: from.ref,
          to: to.ref,
          concept: from.ref.concept,
          path: linkPath(from.center, to.center),
          binding: b.id.toInt(),
          transport: b.hasTransportInit() ? b.transportInit : null,
        ),
      );
    }
  }

  return CanvasScene(nodes: nodes, links: links, groups: groups);
}

extension<T> on T {
  R let<R>(R Function(T) f) => f(this);
}

int _portKey(int instance, int port) => instance * 1000003 + port;

String _endLabel(pb.SystemView sys, pb.PortRefView e) {
  if (e.hasBaseDecl()) {
    return sys.base.mappings.where((m) => m.id == e.baseDecl).map((m) => m.name).firstOrNull ?? '?';
  }
  final inst = sys.instances.where((i) => i.id == e.instance).firstOrNull;
  final comp = inst == null
      ? null
      : sys.components.where((c) => c.id == inst.component).firstOrNull;
  final port = comp?.ports.where((p) => p.id == e.port).firstOrNull;
  return '${inst?.name ?? '?'}.${port?.name ?? '?'}';
}

/// The socket a binding end is on the canvas: a port's, a base
/// relationship's realisation socket (destination) or output socket
/// (source).
SocketShape? _endSocket(
  pb.SystemView sys,
  Map<SocketRef, SocketShape> sockets,
  pb.PortRefView e,
  SocketSide side,
) {
  if (e.hasBaseDecl()) {
    final m = sys.base.mappings.where((m) => m.id == e.baseDecl).firstOrNull;
    if (m == null) return null;
    final concept = m.signature.output.toInt();
    final node = NodeRef.mapping(e.baseDecl.toInt());
    return side == SocketSide.output
        ? sockets[SocketRef(node: node, side: SocketSide.output, concept: concept)]
        : sockets[SocketRef(
            node: node,
            side: SocketSide.input,
            concept: concept,
            index: realiseIndex,
            role: SocketRole.realise,
          )];
  }
  return sockets.entries
      .where(
        (s) =>
            s.key.role == SocketRole.port &&
            s.key.node == NodeRef.instance(e.instance.toInt()) &&
            s.key.index == e.port.toInt(),
      )
      .map((s) => s.value)
      .firstOrNull;
}

/// The system-level identity of a component-local concept at one instance:
/// the shared system concept, or the private concept's flat identity (the
/// origin map has it).  Identity is the hue; a private concept of another
/// instance is another hue.
int _systemConcept(
  pb.SystemView sys,
  pb.ComponentView comp,
  pb.ComponentInstanceView inst,
  Int64 local,
) {
  final shared = comp.sharedConcepts.where((p) => p.local == local).firstOrNull;
  if (shared != null) return shared.system.toInt();
  final origin = sys.origins
      .where(
        (o) => o.sort == pb.LocalSort.LOCAL_SORT_SEM && o.instance == inst.id && o.local == local,
      )
      .firstOrNull;
  // A flat identity never collides with a system concept id on the wire
  // (they are the same allocator); before the table is complete, the local
  // id offset far away keeps hues apart.
  return origin?.flat.toInt() ?? (1 << 40) + inst.id.toInt() * 4096 + local.toInt();
}

/// The socket shape of a port: the value form of the concept its contract
/// produces, read off the component's body.
SocketKind _contractKind(pb.SystemView sys, pb.ComponentView comp, pb.PortContractView k) {
  final c = comp.body.concepts.where((c) => c.id == k.signature.output).firstOrNull;
  return c == null ? SocketKind.open : socketKind(c);
}

/// One line for a definition on the node: the formula, a component's
/// formula (pinned scope — generated, read-only), or a reference the
/// system made.
String definitionSummary(pb.Definition d) => switch (d.whichKind()) {
  pb.Definition_Kind.formula => d.formula,
  pb.Definition_Kind.reference => 'bound',
  pb.Definition_Kind.notSet => '',
};

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
  for (final l in scene.links) {
    if (l.binding != null && _nearPath(l.path, point, 6)) return HitLink(l);
  }
  for (final g in scene.groups.reversed) {
    if (g.titleBand.contains(point)) return HitGroup(g);
  }
  return const HitNothing();
}

bool _nearPath(Path path, Offset p, double tolerance) {
  for (final m in path.computeMetrics()) {
    final steps = (m.length / 6).ceil().clamp(1, 400);
    for (var i = 0; i <= steps; i++) {
      final t = m.getTangentForOffset(m.length * i / steps);
      if (t != null && (t.position - p).distance <= tolerance) return true;
    }
  }
  return false;
}

/// The expanded group region a point falls in, if any (for dropping a
/// relationship into a group).  The group's own moving member is left out
/// of the region so it can be dragged out.
GroupShape? groupAt(CanvasScene scene, Offset point) =>
    scene.groups.where((g) => g.rect.contains(point)).lastOrNull;

/// Whether a link may run between two sockets: same concept, opposite
/// side, different node.  The nominal typing rule, as a gesture constraint.
/// A sink takes only a mapping's output: a concept cannot drive the world
/// by itself, and nothing reads from a sink.
bool canLink(SocketRef from, SocketRef to) {
  if (to.node == from.node || to.side == from.side || to.concept != from.concept) return false;
  // Aggregate sockets are a picture of a group's boundary: nothing binds
  // to them (an edge attaches to a declaration, never to the group).
  if (from.role == SocketRole.aggregate || to.role == SocketRole.aggregate) return false;
  final (out, inp) = from.side == SocketSide.output ? (from, to) : (to, from);
  // A binding end is a port or a base relationship (its output, or its
  // realisation socket): a concept node is neither.
  final isBinding =
      out.role == SocketRole.port || inp.role == SocketRole.port || inp.role == SocketRole.realise;
  if (isBinding) {
    if (out.node.kind == NodeKind.concept || inp.node.kind == NodeKind.concept) return false;
    if (out.node.kind == NodeKind.output || inp.node.kind == NodeKind.output) return false;
    return inp.role == SocketRole.port || inp.role == SocketRole.realise;
  }
  final kinds = {from.node.kind, to.node.kind};
  if (kinds.contains(NodeKind.output)) return kinds.contains(NodeKind.mapping);
  return true;
}

/// The socket a dragged link may legally be dropped on.  Typing by identity,
/// made visible.
SocketShape? dropTarget(CanvasScene scene, SocketRef from, Offset point) {
  final hit = hitTest(scene, point);
  if (hit is! HitSocket) return null;
  return canLink(from, hit.socket.ref) ? hit.socket : null;
}

/// Every socket a link from [from] could land on — shown with a halo while
/// dragging so the rule is seen before the drop.
Set<SocketRef> compatibleSockets(CanvasScene scene, SocketRef from) => {
  for (final n in scene.nodes)
    for (final s in n.sockets)
      if (canLink(from, s.ref)) s.ref,
};

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

/// The compiler's verdict on a mapping, as a word — Explain layer only; the
/// canvas shows it as object state (dashed, hollow socket, red mark).
String statusWord(pb.MappingStatus s) => switch (s) {
  pb.MappingStatus.MAPPING_STATUS_DECLARED => 'declared',
  pb.MappingStatus.MAPPING_STATUS_OPEN => 'open',
  pb.MappingStatus.MAPPING_STATUS_INVALID => 'invalid',
  pb.MappingStatus.MAPPING_STATUS_TYPE_VALID => 'type-valid',
  pb.MappingStatus.MAPPING_STATUS_TEMPORALLY_VALID => 'temporally valid',
  pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT => 'clock-consistent',
  _ => '',
};

/// Whether a status word means "settled" (green), "open" (orange) or
/// "wrong" (red).
enum StatusTone { settled, open, error }

StatusTone statusTone(pb.MappingStatus s) => switch (s) {
  pb.MappingStatus.MAPPING_STATUS_TYPE_VALID ||
  pb.MappingStatus.MAPPING_STATUS_TEMPORALLY_VALID ||
  pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT => StatusTone.settled,
  pb.MappingStatus.MAPPING_STATUS_INVALID => StatusTone.error,
  _ => StatusTone.open,
};

String _conceptName(pb.ProjectProjection p, int id) =>
    p.concepts.where((c) => c.id.toInt() == id).map((c) => c.name).firstOrNull ?? '?';

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
