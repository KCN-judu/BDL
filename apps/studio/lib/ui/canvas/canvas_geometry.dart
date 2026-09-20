/// Pure geometry of the node canvas: where nodes, sockets and links are,
/// and what is under a point.  No widgets, no state — a function of the
/// projection and the layout, so it is unit-testable and deterministic.
///
/// Node anatomy follows Blender (docs/architecture/studio-ui.md §2).  A concept is a
/// single-row object: its name, an input socket (what produces it) on the
/// left and an output socket (what reads it) on the right.  A mapping has a
/// header with title and state word, one input socket per read concept, the
/// output socket on the right, and a definition region below.  Data flows
/// left → right.
///
/// Semantics are carried by the geometry, not by words (docs/architecture/studio-ui.md §7):
/// socket hue is identity, socket *shape* is the concept's value form, a
/// hollow ring means the form is not chosen yet, a dashed outline means
/// declared-not-defined, and a red mark at the definition line means the
/// definition does not check.
///
/// Two kinds of edge (ADR-0034).  A *signature edge* runs socket to socket
/// in the concept's hue: concept → a relationship that reads it, a
/// relationship → the concept it produces, a value → the sink it drives;
/// it is what the designer edits by dragging.  A *reference edge* runs from
/// a relationship's output socket into the formula line of a relationship
/// whose definition names it — the kernel's `dependsOn`, read off the
/// analysis (`MappingAnalysis.references`), never off the formula text — and is
/// drawn neutral and thin: it is not a typed port and cannot be dragged.
///
/// A system canvas (docs/architecture/studio-ui.md §11) adds component-instance nodes
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

  /// Where a node without a stored position is drawn.  Placement is the
  /// daemon's (the layout service places every unpositioned entity on
  /// open and on commit, ADR-0023 §7), so this is reached only between a
  /// commit and its projection; Studio never arranges nodes itself.
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
/// shape: ○ quantity, ◇ on–off, □ count, ⧉ collection (a stack), ▯ grouped
/// value (a split square), ◎ optional value (a ring with a hole); a hollow
/// ring while undecided.  What a collection holds is the inspector's
/// word, not a second shape.
enum SocketKind { open, quantity, onOff, count, collection, grouped, optional }

SocketKind socketKind(pb.ConceptView c) {
  if (!c.hasRepresentation()) return SocketKind.open;
  return socketKindOf(c.representation);
}

SocketKind socketKindOf(pb.Representation r) => switch (r.whichKind()) {
  pb.Representation_Kind.quantity => SocketKind.quantity,
  pb.Representation_Kind.boolean => SocketKind.onOff,
  pb.Representation_Kind.count => SocketKind.count,
  pb.Representation_Kind.list => SocketKind.collection,
  pb.Representation_Kind.pair => SocketKind.grouped,
  pb.Representation_Kind.optional => SocketKind.optional,
  pb.Representation_Kind.notSet => SocketKind.open,
};

/// The value form in the designer's words: "a quantity", "a collection of
/// temperatures", "a grouped value (a temperature and a quantity)".
String representationWords(pb.Representation r, {String Function(pb.Dim)? unit}) {
  String go(pb.Representation r) => switch (r.whichKind()) {
    pb.Representation_Kind.quantity =>
      unit == null ? 'a quantity' : 'a quantity in ${unit(r.quantity)}',
    pb.Representation_Kind.boolean => 'on or off',
    pb.Representation_Kind.count => 'a count',
    pb.Representation_Kind.list => 'a collection of values (${go(r.list)})',
    pb.Representation_Kind.pair => 'a grouped value (${go(r.pair.first)} and ${go(r.pair.second)})',
    pb.Representation_Kind.optional => 'an optional value (${go(r.optional)})',
    pb.Representation_Kind.notSet => 'not decided',
  };
  return go(r);
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
    this.source = false,
    this.rule = false,
    this.dependsOn = const [],
    this.unapplied = false,
  });
  final NodeRef ref;
  final Rect rect;
  final String title;
  final List<SocketShape> sockets;

  /// A rule (ADR-0034): a relationship that reads something — a function
  /// from its inputs to its output, applied by other formulas; it has no
  /// value of its own.  The word *rule* in the header when no state word
  /// takes the slot; the input sockets are the shape.
  final bool rule;

  /// The names of the relationships this one's definition references, in
  /// the analysis's order — what the reference edges into its formula line
  /// stand for, said to assistive technology.
  final List<String> dependsOn;

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

  /// A Source (ADR-0032): a value entering the behavior model from the
  /// environment — no input sockets, one output socket, the environment
  /// boundary drawn on its left edge.  Not *declared*: nothing is missing.
  final bool source;

  /// A rule nothing applies (the compiler's `reactive.rule_unapplied`):
  /// its output socket is hollow — the value form is known, no value comes
  /// out of it until a value applies the rule — and, once defined, the
  /// header word *not applied*.  A legal state, never an error.
  final bool unapplied;

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

  /// Where a reference edge enters a mapping: the left end of its formula
  /// line.  Not a socket — nothing can be dropped there.
  Offset get formulaEntry => Offset(rect.left, definitionRegion.center.dy);
}

class LinkShape {
  const LinkShape({
    required this.from,
    required this.to,
    required this.concept,
    required this.path,
    this.binding,
    this.transport,
    this.reference = false,
  });
  final SocketRef from;

  /// The socket the link ends at.  A reference edge has none: [to] is the
  /// referencing relationship's output socket ref standing for the node,
  /// and the path ends at its [NodeShape.formulaEntry].
  final SocketRef to;
  final int concept;
  final Path path;

  /// A reference edge (ADR-0034): [to]'s node names [from]'s node in its
  /// definition.  Drawn neutral and thin into the formula line; never a
  /// drop target, never selectable.
  final bool reference;

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

/// A concrete endpoint an aggregate socket stands for: a member's socket
/// (when its signature has one for the concept) or just the member.  A
/// link started or dropped on an aggregate socket resolves to one of these
/// — the committed edit names the declaration, never the group.
class ProxyTarget {
  const ProxyTarget({required this.label, required this.node, this.socket});
  final String label;
  final NodeRef node;
  final SocketRef? socket;
}

class CanvasScene {
  const CanvasScene({
    required this.nodes,
    required this.links,
    this.groups = const [],
    this.proxies = const {},
  });
  final List<NodeShape> nodes;
  final List<LinkShape> links;

  /// Expanded group regions (their collapsed counterparts are nodes of
  /// kind [NodeKind.group]).
  final List<GroupShape> groups;

  /// What each aggregate socket of a collapsed group stands for.
  final Map<SocketRef, List<ProxyTarget>> proxies;

  /// The concrete endpoints behind a socket: itself, or — for an aggregate
  /// socket — the declarations it stands for.
  List<ProxyTarget> resolve(SocketRef s) => s.role == SocketRole.aggregate
      ? proxies[s] ?? const []
      : [ProxyTarget(label: '', node: s.node, socket: s)];

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

/// Build the scene.  Positions come from [layout], which the daemon has
/// completed for every entity (ADR-0023 §7); Studio does not arrange.
/// What a system canvas adds to a flat one: the system (instances, their
/// contracts, bindings, groups), its analysis (port statuses, boundaries,
/// realizes), and the groups' boxes.  Absent for a flat project and for a
/// component's source (where the body is an ordinary design and
/// [portWords] name the port-backed relationships).
class SystemSceneInput {
  const SystemSceneInput({
    this.system,
    this.analysis,
    this.groups = const [],
    this.boundaries = const [],
    this.groupBoxes = const {},
    this.portWords = const {},
    this.summarize = false,
  });

  /// Instances and bindings: the system canvas only.
  final pb.SystemView? system;
  final pb.SystemAnalysisView? analysis;

  /// The groups of the design on screen (the system's own, or the open
  /// component's) and their boundaries, as the compiler computed them.
  final List<pb.BehaviorGroupView> groups;
  final List<pb.BehaviorGroupBoundaryView> boundaries;
  final Map<int, GroupBox> groupBoxes;
  final Map<int, String> portWords;

  /// Semantic zoom: at a low zoom every group reads as its summary box,
  /// whatever its authored collapse state (which is not touched).
  final bool summarize;

  bool isCollapsed(int group) => summarize || (groupBoxes[group]?.collapsed ?? false);
}

/// [refs]: per mapping id, the mappings its definition references
/// (`MappingAnalysis.references`, when an analysis of this revision exists) —
/// the reference edges.  [statuses] and [outputStates] are the same
/// analysis's verdicts.
CanvasScene buildScene(
  pb.ProjectProjection p,
  Map<NodeRef, Offset> layout, {
  Map<int, pb.MappingStatus> statuses = const {},
  Map<int, pb.OutputState> outputStates = const {},
  Map<int, List<int>> refs = const {},
  Set<int> unapplied = const {},
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
  for (final g in system.groups) {
    if (system.isCollapsed(g.id.toInt())) {
      for (final m in g.members) {
        hiddenMembers[m.toInt()] = g.id.toInt();
      }
    }
  }

  final nodes = <NodeShape>[];
  final socketByRef = <SocketRef, SocketShape>{};

  for (final c in concepts) {
    final ref = NodeRef.concept(c.id.toInt());
    final pos = layout[ref] ?? NodeMetrics.origin;
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

  for (final m in mappings) {
    final ref = NodeRef.mapping(m.id.toInt());
    final pos = layout[ref] ?? NodeMetrics.origin;
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
    // port or by another base relationship of the same concept: a socket at
    // its definition row, hollow until bound.  Only while something could
    // bind to it — an instance, or another relationship producing the
    // concept; otherwise a Source keeps its left edge clear (ADR-0032: no
    // input sockets).
    final realisable =
        sys != null &&
        (sys.instances.isNotEmpty ||
            mappings.any((x) => x.id != m.id && x.signature.output == m.signature.output));
    if (realisable && !m.hasDefinition()) {
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
    // A rule nothing applies produces nothing yet: the socket is hollow.
    final isUnapplied = unapplied.contains(m.id.toInt());
    final out = SocketShape._(
      Offset(rect.right, rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight / 2),
      outRef,
      kindOf(outId),
      open: isUnapplied,
    );
    sockets.add(out);
    socketByRef[outRef] = out;
    labels[outRef] = _conceptName(p, outId);
    final realised = realisedBy[m.id.toInt()];
    // The role is the daemon's (ADR-0032, `MappingView.role`): a Source is
    // drawn as one where the designer is — inside an open component a
    // port-backed Source wears its port's word instead; the system view
    // already states a bound base relationship as a Value.
    final role = relationshipRole(m);
    final portWord = system.portWords[m.id.toInt()];
    final source = role == RelationshipRole.source && portWord == null;
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
        // Declared — dashed — is the one hole a designer fills: a rule
        // with no formula.  A Source is complete; a value has its
        // realization.
        declared: role == RelationshipRole.rule && !m.hasDefinition(),
        source: source,
        // A rule reads something: the input sockets are the shape, the
        // word says the consequence (it is applied; it has no value).
        rule: role == RelationshipRole.rule,
        dependsOn: [
          for (final d in refs[m.id.toInt()] ?? const <int>[])
            if (d != m.id.toInt())
              if (mappings.where((x) => x.id.toInt() == d).firstOrNull case final x?) x.name,
        ],
        unapplied: isUnapplied,
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
    for (final inst in instances) {
      final comp = sys.components.where((c) => c.id == inst.component).firstOrNull;
      if (comp == null) continue;
      final ref = NodeRef.instance(inst.id.toInt());
      final pos = layout[ref] ?? NodeMetrics.origin;
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
  for (final o in outputs) {
    final ref = NodeRef.output(o.id.toInt());
    final pos = layout[ref] ?? NodeMetrics.origin;
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
  final proxies = <SocketRef, List<ProxyTarget>>{};
  {
    final byId = {for (final n in nodes) n.ref: n};
    for (final g in system.groups) {
      final id = g.id.toInt();
      final members = g.members.map((m) => m.toInt()).toList();
      final box = system.groupBoxes[id];
      final boundary = system.boundaries.where((b) => b.id == g.id).firstOrNull;
      if (system.isCollapsed(id)) {
        final ins = <int>[...?boundary?.externalInputs.map((d) => d.toInt())];
        final outs = <int>[
          ...?boundary?.externalOutputs.map((d) => d.toInt()),
          for (final d in boundary?.drivenMembers ?? const <Int64>[])
            if (!(boundary?.externalOutputs.contains(d) ?? false)) d.toInt(),
        ];
        final rows = ins.length > outs.length ? ins.length : outs.length;
        final ref = NodeRef.group(id);
        // A box without a stored place stands where its members are (a
        // transient summary at low zoom, or a group collapsed before any
        // layout was stored) — never at the origin.
        final memberRects = [
          for (final m in members)
            if (layout[NodeRef.mapping(m)] case final pos?)
              Rect.fromLTWH(pos.dx, pos.dy, NodeMetrics.mappingWidth, NodeMetrics.headerHeight),
        ];
        final origin = box != null && box.rect != Rect.zero && (box.collapsed || !system.summarize)
            ? box.rect.topLeft
            : memberRects.isEmpty
            ? NodeMetrics.origin
            : memberRects.reduce((a, b) => a.expandToInclude(b)).topLeft -
                  const Offset(
                    NodeMetrics.regionPadding,
                    NodeMetrics.regionPadding + NodeMetrics.regionTitle,
                  );
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
        pb.MappingView? mappingOf(int d) => p.mappings.where((m) => m.id.toInt() == d).firstOrNull;
        int conceptOfDecl(int d) => mappingOf(d)?.signature.output.toInt() ?? -1;
        String nameOfDecl(int d) => mappingOf(d)?.name ?? '?';
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
          // What the socket stands for, concretely.  An open member: its
          // own realisation socket (a provided port may realise it).  A
          // crossing-in declaration: the members that read it — each as
          // its input socket of that concept when its signature has one,
          // else the member itself.  Never the group.
          if (open) {
            proxies[r] = [
              ProxyTarget(
                label: nameOfDecl(ins[i]),
                node: NodeRef.mapping(ins[i]),
                socket: SocketRef(
                  node: NodeRef.mapping(ins[i]),
                  side: SocketSide.input,
                  concept: c,
                  index: realiseIndex,
                  role: SocketRole.realise,
                ),
              ),
            ];
          } else {
            final readers = [
              for (final e in boundary?.crossingEdges ?? const <pb.DeclEdge>[])
                if (e.to.toInt() == ins[i]) e.from.toInt(),
            ];
            proxies[r] = [
              for (final m in readers)
                () {
                  final mv = mappingOf(m);
                  final inputs = mv?.signature.inputs.map((x) => x.toInt()).toList() ?? const [];
                  final at = inputs.indexOf(c);
                  return ProxyTarget(
                    label: nameOfDecl(m),
                    node: NodeRef.mapping(m),
                    socket: at < 0
                        ? null
                        : SocketRef(
                            node: NodeRef.mapping(m),
                            side: SocketSide.input,
                            concept: c,
                            index: at,
                          ),
                  );
                }(),
            ];
          }
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
          // An output socket stands for exactly one member's output.
          proxies[r] = [
            ProxyTarget(
              label: nameOfDecl(outs[i]),
              node: NodeRef.mapping(outs[i]),
              socket: SocketRef(
                node: NodeRef.mapping(outs[i]),
                side: SocketSide.output,
                concept: c,
              ),
            ),
          ];
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

  // Reference edges (ADR-0034): from each relationship a definition names
  // to the formula line of the relationship naming it — the kernel's
  // `dependsOn`, as the analysis reports it.  A hidden referencing member
  // adds nothing (its group's crossing-in edges stand for it above); a
  // hidden referenced member's edge leaves from its group's aggregate
  // socket; a reference to itself (memory through `delay`) and one to a
  // declaration the projection does not show (an instance's private
  // relationship, or one already deleted) draw nothing.
  {
    final byId = {for (final n in nodes) n.ref: n};
    for (final m in mappings) {
      final id = m.id.toInt();
      if (hiddenMembers.containsKey(id)) continue;
      final node = byId[NodeRef.mapping(id)];
      if (node == null) continue;
      final outId = m.signature.output.toInt();
      final self = SocketRef(node: node.ref, side: SocketSide.output, concept: outId);
      for (final d in refs[id] ?? const <int>[]) {
        if (d == id) continue;
        final target = mappings.where((x) => x.id.toInt() == d).firstOrNull;
        if (target == null) continue;
        final targetOut = target.signature.output.toInt();
        final from = hiddenMembers.containsKey(d)
            ? groupOut(d)
            : socketByRef[SocketRef(
                node: NodeRef.mapping(d),
                side: SocketSide.output,
                concept: targetOut,
              )];
        if (from == null) continue;
        links.add(
          LinkShape(
            from: from.ref,
            to: self,
            concept: targetOut,
            path: linkPath(from.center, node.formulaEntry),
            reference: true,
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

  return CanvasScene(nodes: nodes, links: links, groups: groups, proxies: proxies);
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
  // Signature and binding edges have a hit area (a contextual menu, a
  // binding's selection); reference edges have none (ADR-0034).
  for (final l in scene.links) {
    if (!l.reference && _nearPath(l.path, point, 6)) return HitLink(l);
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
/// made visible.  An *authoring* target (a concept's value socket over a
/// sink that accepts the concept) counts: the drop then resolves to the
/// relationship that drives the sink, never to the concept
/// (docs/architecture/studio-ui.md §2, "Concept → Output").
SocketShape? dropTarget(CanvasScene scene, SocketRef from, Offset point) {
  final hit = hitTest(scene, point);
  if (hit is! HitSocket) return null;
  return canLink(from, hit.socket.ref) || isAuthoringTarget(from, hit.socket.ref)
      ? hit.socket
      : null;
}

/// The Concept → Output authoring gesture: dragging a concept's value socket
/// onto a sink accepting that concept.  Not a link the model has — a sink
/// is driven by a relationship — but the gesture the designer reaches for;
/// the canvas resolves it against the relationships that produce the
/// concept and could drive the sink (`driveCandidates`).
bool isAuthoringTarget(SocketRef from, SocketRef to) {
  final (out, inp) = from.side == SocketSide.output ? (from, to) : (to, from);
  return out.node.kind == NodeKind.concept &&
      out.side == SocketSide.output &&
      out.role == SocketRole.concept &&
      inp.node.kind == NodeKind.output &&
      inp.side == SocketSide.input &&
      inp.concept == out.concept;
}

/// Every socket a link from [from] could land on — shown with a halo while
/// dragging so the rule is seen before the drop.  Authoring targets are
/// included: an eligible sink lights up under a dragged concept.
Set<SocketRef> compatibleSockets(CanvasScene scene, SocketRef from) => {
  for (final n in scene.nodes)
    for (final s in n.sockets)
      if (canLink(from, s.ref) || isAuthoringTarget(from, s.ref)) s.ref,
};

/// Rectangle selection, the CAD convention (docs/architecture/studio-ui.md
/// §2): dragged left → right it is a **window** — only nodes wholly inside
/// are taken; dragged right → left it is **crossing** — nodes inside or
/// touched are taken.  The vertical direction means nothing.
enum MarqueeMode { window, crossing }

/// The mode of a marquee from where it started and where the pointer is.
MarqueeMode marqueeMode(Offset anchor, Offset current) =>
    current.dx >= anchor.dx ? MarqueeMode.window : MarqueeMode.crossing;

/// The nodes a marquee takes: the scene's visible nodes only — a collapsed
/// group is its box, hidden members are not in the scene; links and expanded
/// regions are never taken by a rectangle.
Set<NodeRef> marqueeNodes(CanvasScene scene, Rect box, MarqueeMode mode) => {
  for (final n in scene.nodes)
    if (switch (mode) {
      MarqueeMode.window => box.contains(n.rect.topLeft) && box.contains(n.rect.bottomRight),
      MarqueeMode.crossing => box.overlaps(n.rect),
    })
      n.ref,
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

/// ⇧-click range selection on the graph: the nodes of the one shortest
/// path over *signature* edges (concept ↔ relationship ↔ sink, bindings)
/// from [from] to [to], inclusive — when exactly one shortest path exists.
/// Reference edges are dependency, not the displayed signature, and never
/// take part.  Two or more shortest paths (a branched graph) make the range
/// ambiguous: `null`, and the caller selects nothing it did not click.
Set<NodeRef>? uniqueSignatureChain(CanvasScene scene, NodeRef from, NodeRef to) {
  if (from == to) return {from};
  final adjacent = <NodeRef, Set<NodeRef>>{};
  for (final l in scene.links) {
    if (l.reference) continue;
    adjacent.putIfAbsent(l.from.node, () => {}).add(l.to.node);
    adjacent.putIfAbsent(l.to.node, () => {}).add(l.from.node);
  }
  // Breadth-first: the distance of every node and how many shortest paths
  // reach it; a single predecessor along a single count is the chain.
  final distance = <NodeRef, int>{from: 0};
  final ways = <NodeRef, int>{from: 1};
  final parent = <NodeRef, NodeRef>{};
  final queue = <NodeRef>[from];
  for (var i = 0; i < queue.length; i++) {
    final n = queue[i];
    for (final m in adjacent[n] ?? const <NodeRef>{}) {
      if (!distance.containsKey(m)) {
        distance[m] = distance[n]! + 1;
        ways[m] = ways[n]!;
        parent[m] = n;
        queue.add(m);
      } else if (distance[m] == distance[n]! + 1) {
        ways[m] = ways[m]! + ways[n]!;
      }
    }
  }
  if (!distance.containsKey(to) || ways[to] != 1) return null;
  final chain = <NodeRef>{to};
  var cur = to;
  while (cur != from) {
    cur = parent[cur]!;
    chain.add(cur);
  }
  return chain;
}
