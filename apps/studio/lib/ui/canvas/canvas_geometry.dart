/// Pure geometry of the node canvas: where nodes, sockets and links are,
/// and what is under a point.  No widgets, no state — a function of the
/// projection and the layout, so it is unit-testable and deterministic.
///
/// Node anatomy follows Blender (docs/architecture/studio-ui.md §2) over
/// the concept ladder (ADR-0043, ADR-0044).  A **Sem block** — a
/// unit-domain declaration, one value per tick — is a two-row object: its
/// name in the header (a Source when nothing defines it), its concept's
/// name at the output socket on the right, and on the left the socket its
/// definition arrives at.  A **mapping block** is that definition drawn as
/// a node of its own, beside the Sem block: a header naming the rules it
/// applies, one read socket per Sem block its definition names, one hollow
/// slot socket per open position, the output socket on the right, and the
/// definition region below.  A concept is a template — the hue, the socket
/// shape, the row's word — and never a node; a rule is a template and
/// never a node.  Data flows left → right.
///
/// Semantics are carried by the geometry, not by words (docs/architecture/studio-ui.md §7):
/// socket hue is identity, socket *shape* is the concept's value form, a
/// hollow ring means the form is not chosen yet, and a red mark at the
/// definition line means the definition does not check.
///
/// Three kinds of edge, all socket to socket in the concept's hue.  A
/// *read edge* runs from a Sem block to the mapping block whose definition
/// names it — the kernel's `dependsOn`, read off the analysis
/// (`MappingAnalysis.references`), never off the formula text; taking it
/// away is a text edit.  A *produce edge* runs from a mapping block to its
/// Sem block — the definition itself, one per Sem block, write-once; it is
/// never rerouted.  A *drive edge* runs from a Sem block to the sink it
/// drives.
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
  /// A Sem block: the header and its concept row; a sink has the same
  /// width.  The same numbers as `bdl_layout::metrics`, which places them.
  static const double semWidth = 168;
  static const double semHeight = headerHeight + rowHeight;
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

  /// The expanded formula region's bounds (the editor state's, so the
  /// reducer needs no geometry).
  static const double formulaInitialHeight = EditorState.formulaInitialHeight;
  static const double formulaMaxHeight = EditorState.formulaMaxHeight;
}

enum SocketSide { input, output }

/// What a socket stands for beyond its concept: a Sem block's output, a
/// mapping block's output, or a sink's input ([concept]); the socket a Sem
/// block takes its mapping block's produce edge at ([produce]); a mapping
/// block's read of a Sem block ([read], [index] = the read block's
/// declaration id, [concept] = its concept); an open position of a mapping
/// block's definition ([slot], [index] = the ordinal among its slots —
/// where a dropped Sem block goes); a port of an instance ([port], [index]
/// = port id); the realisation of an open base relationship ([realise]:
/// where a provided port may bind); an aggregate socket of a collapsed
/// group ([aggregate], [index] = the declaration it stands for — a view).
enum SocketRole { concept, produce, read, slot, port, realise, aggregate }

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
    this.wrong = false,
    this.socketLabels = const {},
    this.timing = '',
    this.required = false,
    this.sink,
    this.subtitle = '',
    this.headerWord = '',
    this.unrealized = false,
    this.source = false,
    this.dependsOn = const [],
    this.applies = const [],
    this.formulaHeight = 0,
  });
  final NodeRef ref;
  final Rect rect;
  final String title;
  final List<SocketShape> sockets;

  /// The height of the expanded formula region under the definition line
  /// (docs/architecture/studio-ui.md §2, *Expanded formula*): 0 when the
  /// mapping's saved formula is not shown on the node.  A reading state of
  /// the editor, never layout data.
  final double formulaHeight;

  bool get expanded => formulaHeight > 0;

  /// The names of the Sem blocks this block's definition reads, in the
  /// analysis's order — what the read edges into its sockets stand for,
  /// said to assistive technology.
  final List<String> dependsOn;

  /// The names of the rules this mapping block's definition applies
  /// (ADR-0044: a rule is a template, not a node; the block's header names
  /// it).
  final List<String> applies;

  /// An instance node: the component's name, in the body row.  A collapsed
  /// group: how many relationships it holds.  A mapping block: the name of
  /// the Sem block it defines.
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

  /// The definition does not check: a red mark at the definition line.
  final bool wrong;

  /// Label drawn next to each socket (mapping inputs and output).
  final Map<SocketRef, String> socketLabels;

  Rect get header => Rect.fromLTWH(rect.left, rect.top, rect.width, NodeMetrics.headerHeight);

  /// The definition region of a mapping (below the socket rows): the
  /// summary line; the expanded formula, when shown, sits under it.
  Rect get definitionRegion => Rect.fromLTWH(
    rect.left,
    rect.bottom - formulaHeight - NodeMetrics.bodyHeight,
    rect.width,
    NodeMetrics.bodyHeight,
  );

  /// The expanded formula region (empty when collapsed).
  Rect get formulaRegion =>
      Rect.fromLTWH(rect.left, rect.bottom - formulaHeight, rect.width, formulaHeight);

  /// The disclosure at the right end of the definition line — the one
  /// control on a node: a click shows or hides the saved formula.  Only a
  /// mapping with a definition has one.
  Rect get disclosure => Rect.fromLTWH(
    definitionRegion.right - 18,
    definitionRegion.top + 4,
    14,
    NodeMetrics.bodyHeight - 8,
  );

  /// Whether a Sem block dropped on this node has somewhere to go: a
  /// Source (no definition yet — the drop makes one) or a mapping block
  /// with an open position (the drop fills the first).
  bool get acceptsBlock => switch (ref.kind) {
    NodeKind.mapping => source,
    NodeKind.definition => sockets.any((s) => s.ref.role == SocketRole.slot),
    _ => false,
  };
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

  /// The socket the link ends at: a mapping block's read socket (a read
  /// edge, ADR-0044), a sink's input (a drive edge), a port or a
  /// realisation socket (a binding), or a collapsed group's aggregate
  /// socket.
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

  /// The edge as the selection names it: by its ends.  A binding's edge
  /// keeps its own selection ([BindingSelected]).
  LinkId get id => LinkId(from: from.node, to: to.node, concept: concept, index: to.index);
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

/// The disclosure of a mapping's saved formula.
class HitDisclosure extends CanvasHit {
  const HitDisclosure(this.node);
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

/// The canvas is the value graph (ADR-0044; BDL_FV Phase 21): a **Sem
/// block** per unit-domain relationship — a declaration of a concept
/// holding one value per tick, a Source when it has no definition — and,
/// for each Sem block with a definition, its **mapping block**: the
/// definition drawn as a node of its own ([NodeKind.definition], keyed by
/// the Sem block's id, placed by the layout service beside it) with one
/// read socket per Sem block the definition names, one slot socket per
/// open position, the formula on the definition line, and a produce edge
/// into the Sem block.  A concept is the template a Sem block is created
/// from and a rule the template a mapping block applies: neither is a
/// node.  [refs]: per mapping id, the relationships
/// its definition references (`MappingAnalysis.references`, when an
/// analysis of this revision exists) — the read edges.  [slots]: per
/// mapping id, the open positions of its definition
/// (`MappingAnalysis.slots`).  [statuses] and [outputStates] are the same
/// analysis's verdicts.
CanvasScene buildScene(
  pb.ProjectProjection p,
  Map<NodeRef, Offset> layout, {
  Map<int, pb.MappingStatus> statuses = const {},
  Map<int, pb.OutputState> outputStates = const {},
  Map<int, List<int>> refs = const {},
  Map<int, List<String>> slots = const {},
  SystemSceneInput system = const SystemSceneInput(),
  Map<int, double> expanded = const {},
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

  // The value graph's nodes (ADR-0044).  A rule (a relationship with
  // inputs) is a template applied inside a mapping block and is not a node
  // of the value graph; the Project list and the inspector hold it.
  pb.MappingView? mappingById(int d) => mappings.where((x) => x.id.toInt() == d).firstOrNull;
  bool isSem(int d) => mappingById(d)?.signature.inputs.isEmpty ?? false;
  // Per Sem block: the Sem blocks its definition reads, in the analysis's
  // order (the rules it applies are words, not sockets).
  List<int> readsOf(int id) => [
    for (final d in refs[id] ?? const <int>[])
      if (d != id && isSem(d)) d,
  ];
  List<String> rulesOf(int id) => [
    for (final d in refs[id] ?? const <int>[])
      if (d != id && !isSem(d))
        if (mappingById(d) case final x?) x.name,
  ];

  // The Sem blocks: a two-row node — the name, then the concept at the
  // output socket.  On the left, the socket its definition arrives at: the
  // produce edge of its mapping block, or — on a system canvas — the
  // realisation a binding makes.
  for (final m in mappings) {
    final id = m.id.toInt();
    final ref = NodeRef.mapping(id);
    final pos = layout[ref] ?? NodeMetrics.origin;
    if (hiddenMembers.containsKey(id)) continue;
    if (m.signature.inputs.isNotEmpty) continue;
    final rect = Rect.fromLTWH(pos.dx, pos.dy, NodeMetrics.semWidth, NodeMetrics.semHeight);
    final rowY = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight / 2;
    final outId = m.signature.output.toInt();
    final sockets = <SocketShape>[];
    final labels = <SocketRef, String>{};
    final realised = realisedBy[id];
    // An open base relationship of a system can be realised by a provided
    // port or by another base relationship of the same concept: a hollow
    // socket until bound.  Only while something could bind to it — an
    // instance, or another Sem block of the concept; otherwise a Source
    // keeps its left edge clear (ADR-0032: no input sockets).
    final realisable =
        sys != null &&
        (sys.instances.isNotEmpty ||
            mappings.any(
              (x) =>
                  x.id != m.id &&
                  x.signature.inputs.isEmpty &&
                  x.signature.output == m.signature.output,
            ));
    if (m.hasDefinition() && realised == null) {
      final r = SocketRef(
        node: ref,
        side: SocketSide.input,
        concept: outId,
        role: SocketRole.produce,
      );
      final sock = SocketShape._(Offset(rect.left, rowY), r, kindOf(outId));
      sockets.add(sock);
      socketByRef[r] = sock;
    } else if (realised != null || (realisable && !m.hasDefinition())) {
      final r = SocketRef(
        node: ref,
        side: SocketSide.input,
        concept: outId,
        index: realiseIndex,
        role: SocketRole.realise,
      );
      // a base relationship a binding realises: the socket is filled and
      // the binding's edge says by what (the inspector names it)
      final sock = SocketShape._(Offset(rect.left, rowY), r, kindOf(outId), open: realised == null);
      sockets.add(sock);
      socketByRef[r] = sock;
    }
    final outRef = SocketRef(node: ref, side: SocketSide.output, concept: outId);
    final out = SocketShape._(Offset(rect.right, rowY), outRef, kindOf(outId));
    sockets.add(out);
    socketByRef[outRef] = out;
    // the row names the concept: the block's type
    labels[outRef] = _conceptName(p, outId);
    // The role is the daemon's (ADR-0032, `MappingView.role`): a Source is
    // drawn as one where the designer is — inside an open component a
    // port-backed Source wears its port's word instead; the system view
    // already states a bound base relationship as a Value.
    final role = relationshipRole(m);
    final portWord = system.portWords[id];
    final source = role == RelationshipRole.source && portWord == null;
    nodes.add(
      NodeShape(
        ref: ref,
        rect: rect,
        title: m.name,
        source: source,
        dependsOn: [for (final d in readsOf(id)) mappingById(d)!.name],
        applies: rulesOf(id),
        wrong: statuses[id] == pb.MappingStatus.MAPPING_STATUS_INVALID,
        sockets: sockets,
        socketLabels: labels,
        timing: m.hasClockId() ? clockName(m.clockId) : '',
        headerWord: portWord ?? '',
      ),
    );
  }

  // The mapping blocks: one per Sem block with a definition of its own (a
  // realisation the system made is drawn on the Sem block).  Sockets: one
  // per Sem block the definition reads, typed by that block's concept and
  // named after it; then one hollow socket per open position of the
  // definition; the output socket on the right joins the Sem block.
  for (final m in mappings) {
    final id = m.id.toInt();
    if (m.signature.inputs.isNotEmpty || !m.hasDefinition()) continue;
    if (hiddenMembers.containsKey(id) || realisedBy.containsKey(id)) continue;
    final ref = NodeRef.definition(id);
    final pos = layout[ref] ?? attachedTo(layout[NodeRef.mapping(id)]);
    final reads = readsOf(id);
    final open = slots[id] ?? const <String>[];
    final rows = reads.length + open.length == 0 ? 1 : reads.length + open.length;
    final formulaHeight = expanded[id] ?? 0.0;
    final rect = Rect.fromLTWH(
      pos.dx,
      pos.dy,
      NodeMetrics.mappingWidth,
      NodeMetrics.headerHeight +
          rows * NodeMetrics.rowHeight +
          NodeMetrics.bodyHeight +
          formulaHeight,
    );
    final sockets = <SocketShape>[];
    final labels = <SocketRef, String>{};
    for (var i = 0; i < reads.length; i++) {
      final y = rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (i + 0.5);
      final read = mappingById(reads[i])!;
      final c = read.signature.output.toInt();
      final r = SocketRef(
        node: ref,
        side: SocketSide.input,
        concept: c,
        index: reads[i],
        role: SocketRole.read,
      );
      final sock = SocketShape._(Offset(rect.left, y), r, kindOf(c));
      sockets.add(sock);
      socketByRef[r] = sock;
      labels[r] = read.name;
    }
    for (var i = 0; i < open.length; i++) {
      final y =
          rect.top + NodeMetrics.headerHeight + NodeMetrics.rowHeight * (reads.length + i + 0.5);
      final r = SocketRef(
        node: ref,
        side: SocketSide.input,
        concept: -1,
        index: i,
        role: SocketRole.slot,
      );
      final sock = SocketShape._(Offset(rect.left, y), r, SocketKind.open, open: true);
      sockets.add(sock);
      socketByRef[r] = sock;
      labels[r] = '?';
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
    final rules = rulesOf(id);
    nodes.add(
      NodeShape(
        ref: ref,
        rect: rect,
        // the header names the rules the block applies; a block applying
        // none is a formula of its own (the painter says so)
        title: rules.join(', '),
        subtitle: m.name,
        definition: definitionSummary(m.definition),
        dependsOn: [for (final d in reads) mappingById(d)!.name],
        applies: rules,
        wrong: statuses[id] == pb.MappingStatus.MAPPING_STATUS_INVALID,
        sockets: sockets,
        socketLabels: labels,
        // the domain is the declaration's: on the Sem block, not here
        formulaHeight: formulaHeight,
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
      NodeMetrics.semWidth,
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
        // A rule is a template, not a socket of the boundary.
        final ins = <int>[
          for (final d in boundary?.externalInputs ?? const <Int64>[])
            if (isSem(d.toInt())) d.toInt(),
        ];
        final outs = <int>[
          for (final d in boundary?.externalOutputs ?? const <Int64>[])
            if (isSem(d.toInt())) d.toInt(),
          for (final d in boundary?.drivenMembers ?? const <Int64>[])
            if (!(boundary?.externalOutputs.contains(d) ?? false) && isSem(d.toInt())) d.toInt(),
        ];
        final rows = ins.length > outs.length ? ins.length : outs.length;
        final ref = NodeRef.group(id);
        // A box without a stored place stands where its members are (a
        // transient summary at low zoom, or a group collapsed before any
        // layout was stored) — never at the origin.
        Rect? blockRect(int m) {
          // its mapping block, where it is drawn (attached when unplaced)
          if (!(mappingById(m)?.hasDefinition() ?? false)) return null;
          final pos = layout[NodeRef.definition(m)] ?? attachedTo(layout[NodeRef.mapping(m)]);
          return Rect.fromLTWH(pos.dx, pos.dy, NodeMetrics.mappingWidth, NodeMetrics.headerHeight);
        }

        final memberRects = [
          for (final m in members) ...[
            if (layout[NodeRef.mapping(m)] case final pos?)
              Rect.fromLTWH(pos.dx, pos.dy, NodeMetrics.semWidth, NodeMetrics.headerHeight),
            ?blockRect(m),
          ],
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
          // crossing-in Sem block: the members' mapping blocks that read
          // it — each as its read socket for it.  Never the group.
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
                ProxyTarget(
                  label: nameOfDecl(m),
                  node: NodeRef.definition(m),
                  socket: readsOf(m).contains(ins[i])
                      ? SocketRef(
                          node: NodeRef.definition(m),
                          side: SocketSide.input,
                          concept: c,
                          index: ins[i],
                          role: SocketRole.read,
                        )
                      : null,
                ),
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
            for (final ref in [NodeRef.mapping(m), NodeRef.definition(m)])
              if (byId[ref] case final n?) n.rect,
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

  // Links (ADR-0044).  Read edges: from each Sem block a mapping block's
  // definition names — its output socket, or its group's aggregate output
  // socket when it is hidden — into the mapping block's read socket that
  // stands for it.  Produce edges: from a mapping block's output socket
  // into its Sem block's produce socket — the definition itself.  Drive
  // edges: from a Sem block's output socket into the sink it drives, as
  // authored; whether the drive is well formed is the output pass's
  // verdict, shown on the sink.  A hidden member's drive edge leaves from
  // its group's aggregate socket; its reads are the crossing-in edges
  // drawn below.
  final links = <LinkShape>[];
  SocketShape? groupOut(int member) {
    final g = hiddenMembers[member];
    final r = g == null ? null : groupSockets[g]?.outs[member];
    return r == null ? null : socketByRef[r];
  }

  SocketShape? outSocketOf(int decl) {
    if (hiddenMembers.containsKey(decl)) return groupOut(decl);
    final m = mappingById(decl);
    if (m == null) return null;
    return socketByRef[SocketRef(
      node: NodeRef.mapping(decl),
      side: SocketSide.output,
      concept: m.signature.output.toInt(),
    )];
  }

  for (final m in mappings) {
    final id = m.id.toInt();
    if (!isSem(id)) continue;
    final hidden = hiddenMembers.containsKey(id);
    final outId = m.signature.output.toInt();
    if (!hidden) {
      for (final d in readsOf(id)) {
        final from = outSocketOf(d);
        final read = mappingById(d);
        final to =
            socketByRef[SocketRef(
              node: NodeRef.definition(id),
              side: SocketSide.input,
              concept: read?.signature.output.toInt() ?? -1,
              index: d,
              role: SocketRole.read,
            )];
        if (from == null || to == null || read == null) continue;
        links.add(
          LinkShape(
            from: from.ref,
            to: to.ref,
            concept: read.signature.output.toInt(),
            path: linkPath(from.center, to.center),
          ),
        );
      }
      final produce =
          socketByRef[SocketRef(
            node: NodeRef.definition(id),
            side: SocketSide.output,
            concept: outId,
          )];
      final at =
          socketByRef[SocketRef(
            node: NodeRef.mapping(id),
            side: SocketSide.input,
            concept: outId,
            role: SocketRole.produce,
          )];
      if (produce != null && at != null) {
        links.add(
          LinkShape(
            from: produce.ref,
            to: at.ref,
            concept: outId,
            path: linkPath(produce.center, at.center),
          ),
        );
      }
    }
    if (m.hasDrivesOutputId()) {
      final from = outSocketOf(id);
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
      final m = mappingById(decl);
      if (m == null || hiddenMembers.containsKey(decl) || !isSem(decl)) continue;
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

  return CanvasScene(nodes: nodes, links: links, groups: groups, proxies: proxies);
}

extension<T> on T {
  R let<R>(R Function(T) f) => f(this);
}

/// Where a mapping block without a stored position is drawn: beside its
/// Sem block, as the layout service attaches it (`bdl_layout`: directly to
/// the left, its rows centred on the block's row) — reached only between a
/// commit and the daemon's placement.
Offset attachedTo(Offset? sem) => sem == null ? NodeMetrics.origin : attachedBlockPosition(sem);

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

/// How far from an edge's stroke, in scene units, a pointer still takes
/// it: eight screen pixels whatever the zoom (never narrower than the
/// stroke's own six units, never so wide at a far zoom that a link is hit
/// from across the canvas).  The stroke itself never widens.
double linkTolerance(double zoom) => (8 / zoom).clamp(6.0, 32.0);

/// Topmost thing under [point], sockets first (their hit area extends
/// outside the node), then nodes in reverse draw order, then the nearest
/// edge within [linkHitTolerance] (scene units; see [linkTolerance]) —
/// nearest, so two edges crossing or running side by side resolve to the
/// one the pointer is closest to, the earlier one drawn on a tie.  Hover,
/// click and right-click all ask this one question.
CanvasHit hitTest(CanvasScene scene, Offset point, {double linkHitTolerance = 6}) {
  for (final n in scene.nodes.reversed) {
    for (final s in n.sockets) {
      if ((s.center - point).distance <= NodeMetrics.socketHitRadius) return HitSocket(s, n);
    }
  }
  for (final n in scene.nodes.reversed) {
    if (n.rect.contains(point)) {
      if (n.ref.kind == NodeKind.definition && n.disclosure.contains(point)) {
        return HitDisclosure(n);
      }
      return HitNode(n, header: n.header.contains(point));
    }
  }
  // Every edge has a hit area: a read, a drive, a binding, an aggregate.
  if (nearestLink(scene, point, linkHitTolerance) case final l?) return HitLink(l);
  for (final g in scene.groups.reversed) {
    if (g.titleBand.contains(point)) return HitGroup(g);
  }
  return const HitNothing();
}

/// The selectable edge nearest [point] within [tolerance], or none.
LinkShape? nearestLink(CanvasScene scene, Offset point, double tolerance) {
  LinkShape? best;
  var bestDistance = double.infinity;
  for (final l in scene.links) {
    final d = _distanceToPath(l.path, point, tolerance);
    if (d != null && d < bestDistance) {
      best = l;
      bestDistance = d;
    }
  }
  return best;
}

/// The distance from [p] to [path] when within [tolerance], sampled every
/// four scene units along the path (finer than any tolerance, so a hit
/// never falls between samples).
double? _distanceToPath(Path path, Offset p, double tolerance) {
  final bounds = path.getBounds().inflate(tolerance);
  if (!bounds.contains(p)) return null;
  double? best;
  for (final m in path.computeMetrics()) {
    final steps = (m.length / 4).ceil().clamp(1, 600);
    for (var i = 0; i <= steps; i++) {
      final t = m.getTangentForOffset(m.length * i / steps);
      if (t == null) continue;
      final d = (t.position - p).distance;
      if (d <= tolerance && (best == null || d < best)) best = d;
    }
  }
  return best;
}

/// The expanded group region a point falls in, if any (for dropping a
/// relationship into a group).  The group's own moving member is left out
/// of the region so it can be dragged out.
GroupShape? groupAt(CanvasScene scene, Offset point) =>
    scene.groups.where((g) => g.rect.contains(point)).lastOrNull;

/// Whether a link may run between two sockets (ADR-0044).  A Sem block's
/// output into a mapping block's open position (a [SocketRole.slot]): the
/// drop is a text edit of the definition and the compiler types it — the
/// gesture refuses nothing by concept there.  A Sem block's output into a
/// sink accepting its concept: the drive edge.  A port or a realisation
/// socket: a binding.  A read socket is a name in a formula and takes no
/// link; a produce socket is the joint of a Sem block and its definition
/// and takes none; a mapping block's output socket starts none (the
/// definition produces its own Sem block and nothing else); an aggregate
/// socket is a picture of a group's boundary; nothing reads from a sink.
bool canLink(SocketRef from, SocketRef to) {
  if (to.node == from.node || to.side == from.side) return false;
  if (from.role == SocketRole.aggregate || to.role == SocketRole.aggregate) return false;
  final (out, inp) = from.side == SocketSide.output ? (from, to) : (to, from);
  if (inp.role == SocketRole.read ||
      inp.role == SocketRole.produce ||
      out.node.kind == NodeKind.definition ||
      out.role != SocketRole.concept && out.role != SocketRole.port) {
    return false;
  }
  if (inp.role == SocketRole.slot) {
    // any Sem block of the design; a port's value is bound, not named
    return out.node.kind == NodeKind.mapping && out.role == SocketRole.concept;
  }
  if (to.concept != from.concept) return false;
  final isBinding =
      out.role == SocketRole.port || inp.role == SocketRole.port || inp.role == SocketRole.realise;
  if (isBinding) {
    if (out.node.kind == NodeKind.output || inp.node.kind == NodeKind.output) return false;
    return inp.role == SocketRole.port || inp.role == SocketRole.realise;
  }
  return out.node.kind == NodeKind.mapping && inp.node.kind == NodeKind.output;
}

/// The socket a dragged link may legally be dropped on.  Typing by
/// identity, made visible.
SocketShape? dropTarget(CanvasScene scene, SocketRef from, Offset point) {
  final hit = hitTest(scene, point);
  if (hit is! HitSocket) return null;
  return canLink(from, hit.socket.ref) ? hit.socket : null;
}

/// The node a dragged link from a Sem block's output may be dropped on as
/// a whole (its body, not a socket): a Source, which the drop gives a
/// definition; a mapping block with an open position, which the drop
/// fills; or a Sem block whose mapping block has one — the drop goes
/// there.  A text edit either way (ADR-0028).
NodeShape? blockDropTarget(CanvasScene scene, SocketRef from, Offset point) {
  if (from.side != SocketSide.output ||
      from.role != SocketRole.concept ||
      from.node.kind != NodeKind.mapping) {
    return null;
  }
  final hit = hitTest(scene, point);
  var node = switch (hit) {
    HitNode(:final node) || HitDisclosure(:final node) => node,
    _ => null,
  };
  if (node == null || node.ref.id == from.node.id) return null;
  if (node.ref.kind == NodeKind.mapping && !node.source) {
    final block = NodeRef.definition(node.ref.id);
    node = scene.nodes.where((n) => n.ref == block).firstOrNull;
  }
  if (node == null || !node.acceptsBlock) return null;
  return node;
}

/// Every socket a link from [from] could land on — shown with a halo while
/// dragging so the rule is seen before the drop.
Set<SocketRef> compatibleSockets(CanvasScene scene, SocketRef from) => {
  for (final n in scene.nodes)
    for (final s in n.sockets)
      if (canLink(from, s.ref)) s.ref,
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
/// path over the drawn edges (read, produce, drive, bindings) from [from]
/// to [to], inclusive — when exactly one shortest path exists.  Two or
/// more shortest paths (a branched graph) make the range ambiguous:
/// `null`, and the caller selects nothing it did not click.
Set<NodeRef>? uniqueSignatureChain(CanvasScene scene, NodeRef from, NodeRef to) {
  if (from == to) return {from};
  final adjacent = <NodeRef, Set<NodeRef>>{};
  for (final l in scene.links) {
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
  // a mapping block on the way is its Sem block: the chain is declarations
  return {for (final n in chain) asDeclaration(n)};
}
