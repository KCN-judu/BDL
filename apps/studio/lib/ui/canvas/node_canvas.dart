/// The node canvas: Blender-style interaction on top of [buildScene].
///
/// High-frequency state (pan, zoom, an in-progress drag) lives here as
/// widget state and never reaches the reducer.  Only *results* are
/// dispatched: a node's final position, a link made, a selection.
library;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart' show SemanticsProperties;
import 'package:flutter/services.dart';

import '../../app/actions.dart';
import '../../app/state.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../concept_library_panel.dart' show ConceptTemplateDrag, categoryLabel;
import '../mac/tokens.dart';
import 'canvas_geometry.dart';

class NodeCanvas extends StatefulWidget {
  const NodeCanvas({
    super.key,
    required this.project,
    required this.layout,
    required this.selection,
    required this.dispatch,
    this.statuses = const {},
    this.outputStates = const {},
    this.templates = const [],
    this.recentTemplates = const [],
    this.renaming,
    this.canInsert = true,
    this.system = const SystemSceneInput(),
    this.context = const SystemContext(),
    this.components = const [],
    this.groups = const [],
  });

  final pb.ProjectProjection project;
  final Map<NodeRef, Offset> layout;
  final Selection selection;
  final void Function(AppAction) dispatch;

  /// A system project's instances, bindings, groups and their verdicts;
  /// empty for a flat project and inside a component's source.
  final SystemSceneInput system;
  final DesignContext context;

  /// The components an instance can be placed of (right-click menu).
  final List<pb.ComponentView> components;

  /// The groups a relationship can be added to.
  final List<pb.BehaviorGroupView> groups;

  /// Compiler verdicts per mapping id, when an analysis of this revision
  /// exists.
  final Map<int, pb.MappingStatus> statuses;

  /// The output pass per sink id, when an analysis of this revision exists.
  final Map<int, pb.OutputState> outputStates;

  /// The concept templates the right-click menu offers, and the recently
  /// used ones (most recent first).
  final List<pb.ConceptTemplateView> templates;
  final List<String> recentTemplates;

  /// The node whose name is open for inline editing, if any.
  final NodeRef? renaming;

  /// False while an insertion is awaiting the daemon.
  final bool canInsert;

  @override
  State<NodeCanvas> createState() => _NodeCanvasState();
}

class _LinkDrag {
  _LinkDrag(this.from, this.start, {required this.fromConnectedInput}) : current = start;
  final SocketRef from;
  final Offset start;
  Offset current;

  /// Dragging away from a connected mapping input: releasing on empty space
  /// disconnects (Blender: "drag the link away from its input socket").
  final bool fromConnectedInput;
}

class _NodeCanvasState extends State<NodeCanvas> {
  Offset _pan = Offset.zero;
  double _zoom = 1;

  NodeRef? _draggingNode;
  Offset _dragDelta = Offset.zero;
  _LinkDrag? _linkDrag;
  bool _panning = false;
  NodeRef? _hoverNode;
  SocketRef? _hoverSocket;
  final FocusNode _focus = FocusNode(debugLabel: 'canvas');

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  CanvasScene _scene(Map<NodeRef, Offset> layout) => buildScene(
    widget.project,
    layout,
    statuses: widget.statuses,
    outputStates: widget.outputStates,
    system: widget.system,
  );

  bool get _isSystemCanvas => widget.system.system != null && widget.context is SystemContext;

  /// Dragging a group's title band moves every member together.
  int? _draggingGroup;

  final MenuController _menu = MenuController();

  /// Where the context menu was opened, in scene coordinates, so an
  /// insertion from it lands there.
  Offset _menuScene = Offset.zero;
  NodeRef? _menuNode;

  /// Top-left of a new concept node centred on a scene point.
  static Offset nodeOriginFor(Offset scenePoint) =>
      scenePoint - const Offset(NodeMetrics.conceptWidth / 2, NodeMetrics.headerHeight / 2);

  void _insert(String templateId, Offset scenePoint) {
    widget.dispatch(
      InsertConceptTemplateRequested(templateId, position: nodeOriginFor(scenePoint)),
    );
  }

  void _onSecondaryTapDown(TapDownDetails d) {
    _focus.requestFocus();
    final p = _toScene(d.localPosition);
    final node = switch (hitTest(_scene(widget.layout), p)) {
      HitNode(:final node) => node.ref,
      HitSocket(:final node) => node.ref,
      HitGroup(:final group) => NodeRef.group(group.id),
      HitLink(:final link) => () {
        widget.dispatch(SelectionChanged(BindingSelected(link.binding!)));
        return null;
      }(),
      HitNothing() => null,
    };
    if (node != null) widget.dispatch(SelectionChanged(_select(node)));
    setState(() {
      _menuScene = p;
      _menuNode = node;
    });
    _menu.open(position: d.localPosition);
  }

  void _onDoubleTapDown(TapDownDetails d) {
    switch (hitTest(_scene(widget.layout), _toScene(d.localPosition))) {
      case HitNode(:final node):
        if (node.ref.kind == NodeKind.instance) {
          // Into the component's source.
          final inst = widget.system.system?.instances
              .where((i) => i.id.toInt() == node.ref.id)
              .firstOrNull;
          if (inst != null) {
            widget.dispatch(ContextChanged(ComponentContext(inst.component.toInt())));
          }
        } else if (node.ref.kind != NodeKind.output) {
          widget.dispatch(InlineRenameStarted(node.ref));
        }
      case HitGroup(:final group):
        widget.dispatch(InlineRenameStarted(NodeRef.group(group.id)));
      default:
        break;
    }
  }

  /// The contextual menu (docs/STUDIO_UI.md §2): what can be done here,
  /// and the compact quick-insert tree — Recent, by role, the three most
  /// common categories, then the Library tab for the rest.
  List<Widget> _menuItems(BuildContext context) {
    final node = _menuNode;
    final all = widget.templates;
    final byId = {for (final t in all) t.id: t};
    MenuItemButton item(pb.ConceptTemplateView t) => MenuItemButton(
      onPressed: widget.canInsert ? () => _insert(t.id, _menuScene) : null,
      child: Text(t.displayName),
    );
    List<Widget> group(Iterable<pb.ConceptTemplateView> ts) => [for (final t in ts) item(t)];
    final recent = [for (final id in widget.recentTemplates) ?byId[id]];
    final environment = all.where((t) => t.category == 'environment');
    final motion = all.where((t) => t.category == 'motion');
    final human = all.where((t) => t.category == 'human');
    final inputs = all.where((t) => t.roleHint != pb.RoleHint.ROLE_HINT_OUTPUT);
    final outputs = all.where((t) => t.roleHint != pb.RoleHint.ROLE_HINT_INPUT);
    final groupOf = node == null || node.kind != NodeKind.mapping
        ? null
        : widget.groups.where((g) => g.members.any((m) => m.toInt() == node.id)).firstOrNull;
    return [
      if (node != null) ...[
        if (node.kind != NodeKind.output)
          MenuItemButton(
            onPressed: () => widget.dispatch(InlineRenameStarted(node)),
            child: const Text('Rename'),
          ),
        if (node.kind == NodeKind.instance)
          MenuItemButton(
            onPressed: () {
              final inst = widget.system.system?.instances
                  .where((i) => i.id.toInt() == node.id)
                  .firstOrNull;
              if (inst != null) {
                widget.dispatch(ContextChanged(ComponentContext(inst.component.toInt())));
              }
            },
            child: const Text('Edit Source'),
          ),
        if (node.kind == NodeKind.group) ...[
          MenuItemButton(
            onPressed: () => widget.dispatch(
              GroupCollapsedChanged(
                id: node.id,
                collapsed: !(widget.system.groupBoxes[node.id]?.collapsed ?? false),
              ),
            ),
            child: Text(
              (widget.system.groupBoxes[node.id]?.collapsed ?? false) ? 'Expand' : 'Collapse',
            ),
          ),
          MenuItemButton(
            onPressed: () => widget.dispatch(ExtractionSheetOpened(node.id)),
            child: const Text('Package as Reusable Component…'),
          ),
          MenuItemButton(
            onPressed: () => widget.dispatch(UngroupRequested(node.id)),
            child: const Text('Ungroup'),
          ),
        ],
        if (_isSystemCanvas && node.kind == NodeKind.mapping) ...[
          if (groupOf == null) ...[
            MenuItemButton(
              onPressed: () => widget.dispatch(
                CreateGroupRequested(name: _freshGroupName(), members: [node.id]),
              ),
              child: const Text('Group'),
            ),
            if (widget.groups.isNotEmpty)
              SubmenuButton(
                menuChildren: [
                  for (final g in widget.groups)
                    MenuItemButton(
                      onPressed: () => widget.dispatch(
                        AddGroupMemberRequested(group: g.id.toInt(), decl: node.id),
                      ),
                      child: Text(g.name),
                    ),
                ],
                child: const Text('Add to Group'),
              ),
          ] else
            MenuItemButton(
              onPressed: () => widget.dispatch(
                RemoveGroupMemberRequested(group: groupOf.id.toInt(), decl: node.id),
              ),
              child: Text('Remove from ${groupOf.name}'),
            ),
        ],
        if (node.kind != NodeKind.group)
          MenuItemButton(
            onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
            child: const Text('Delete'),
          ),
        const Divider(height: 8),
      ],
      if (_isSystemCanvas && node == null) ...[
        if (widget.components.isNotEmpty)
          SubmenuButton(
            menuChildren: [
              for (final c in widget.components)
                MenuItemButton(
                  onPressed: () => widget.dispatch(
                    CreateInstanceRequested(
                      component: c.id.toInt(),
                      name: _freshInstanceName(c),
                      position: _menuScene,
                    ),
                  ),
                  child: Text(c.name),
                ),
            ],
            child: const Text('Add Instance'),
          ),
        MenuItemButton(
          onPressed: () => widget.dispatch(CreateGroupRequested(name: _freshGroupName())),
          child: const Text('New Group'),
        ),
        const Divider(height: 8),
      ],
      SubmenuButton(
        menuChildren: [
          if (recent.isNotEmpty) ...[
            SubmenuButton(menuChildren: group(recent), child: const Text('Recent')),
            const Divider(height: 8),
          ],
          SubmenuButton(menuChildren: group(inputs), child: const Text('Input')),
          SubmenuButton(menuChildren: group(outputs), child: const Text('Output')),
          const Divider(height: 8),
          SubmenuButton(
            menuChildren: group(environment),
            child: Text(categoryLabel('environment')),
          ),
          SubmenuButton(menuChildren: group(motion), child: Text(categoryLabel('motion'))),
          SubmenuButton(menuChildren: group(human), child: Text(categoryLabel('human'))),
          const Divider(height: 8),
          MenuItemButton(
            onPressed: () => widget.dispatch(const SidebarTabSelected(SidebarTab.library)),
            child: const Text('More…'),
          ),
        ],
        child: const Text('Add Concept'),
      ),
    ];
  }

  /// `Group 2`, `lampA2`: a default name the designer renames inline.
  String _freshGroupName() {
    final taken = widget.groups.map((g) => g.name).toSet();
    var i = widget.groups.length + 1;
    while (taken.contains('Group $i')) {
      i++;
    }
    return 'Group $i';
  }

  String _freshInstanceName(pb.ComponentView c) {
    final base = c.name.isEmpty ? 'instance' : c.name[0].toLowerCase() + c.name.substring(1);
    final taken = (widget.system.system?.instances ?? const []).map((i) => i.name).toSet();
    if (!taken.contains(base)) return base;
    var i = 2;
    while (taken.contains('$base$i')) {
      i++;
    }
    return '$base$i';
  }

  Map<NodeRef, Offset> get _effectiveLayout {
    if (_draggingGroup != null) {
      final scene = _scene(widget.layout);
      final g = scene.groups.where((g) => g.id == _draggingGroup).firstOrNull;
      if (g == null) return widget.layout;
      final moved = {...widget.layout};
      for (final m in g.members) {
        final ref = NodeRef.mapping(m);
        final base =
            widget.layout[ref] ?? scene.nodes.where((n) => n.ref == ref).firstOrNull?.rect.topLeft;
        if (base != null) moved[ref] = base + _dragDelta;
      }
      return moved;
    }
    if (_draggingNode == null) return widget.layout;
    final scene = _scene(widget.layout);
    final base =
        widget.layout[_draggingNode!] ??
        scene.nodes.firstWhere((n) => n.ref == _draggingNode).rect.topLeft;
    return {...widget.layout, _draggingNode!: base + _dragDelta};
  }

  Offset _toScene(Offset local) => (local - _pan) / _zoom;

  void _onPointerSignal(PointerSignalEvent e) {
    if (e is PointerScrollEvent) {
      final factor = e.scrollDelta.dy > 0 ? 0.9 : 1.1;
      final before = _toScene(e.localPosition);
      setState(() {
        _zoom = (_zoom * factor).clamp(0.25, 3.0);
        _pan = e.localPosition - before * _zoom;
      });
    }
  }

  void _onHover(PointerHoverEvent e) {
    final scene = _scene(_effectiveLayout);
    final hit = hitTest(scene, _toScene(e.localPosition));
    final (NodeRef? node, SocketRef? socket) = switch (hit) {
      HitSocket(:final socket, :final node) => (node.ref, socket.ref),
      HitNode(:final node) => (node.ref, null),
      HitGroup(:final group) => (NodeRef.group(group.id), null),
      HitLink() || HitNothing() => (null, null),
    };
    if (node != _hoverNode || socket != _hoverSocket) {
      setState(() {
        _hoverNode = node;
        _hoverSocket = socket;
      });
    }
  }

  void _onPanStart(DragStartDetails d) {
    _focus.requestFocus();
    final scene = _scene(widget.layout);
    final p = _toScene(d.localPosition);
    switch (hitTest(scene, p)) {
      case HitSocket(:final socket):
        final connected =
            socket.ref.side == SocketSide.input && scene.links.any((l) => l.to == socket.ref);
        setState(() {
          _linkDrag = _LinkDrag(socket.ref, socket.center, fromConnectedInput: connected)
            ..current = p;
        });
      case HitNode(:final node):
        widget.dispatch(SelectionChanged(_select(node.ref)));
        setState(() {
          _draggingNode = node.ref;
          _dragDelta = Offset.zero;
        });
      case HitGroup(:final group):
        widget.dispatch(SelectionChanged(GroupSelected(group.id)));
        setState(() {
          _draggingGroup = group.id;
          _dragDelta = Offset.zero;
        });
      case HitLink(:final link):
        widget.dispatch(SelectionChanged(BindingSelected(link.binding!)));
        setState(() => _panning = true);
      case HitNothing():
        widget.dispatch(const SelectionChanged(NoSelection()));
        setState(() => _panning = true);
    }
  }

  void _onPanUpdate(DragUpdateDetails d) {
    setState(() {
      if (_linkDrag != null) {
        final p = _toScene(d.localPosition);
        _linkDrag!.current = p;
        // Hover is not reported while a button is down; track the socket
        // under the dragged link end so the cursor can refuse an illegal one.
        final hit = hitTest(_scene(widget.layout), p);
        _hoverSocket = hit is HitSocket ? hit.socket.ref : null;
      } else if (_draggingNode != null || _draggingGroup != null) {
        _dragDelta += d.delta / _zoom;
      } else if (_panning) {
        _pan += d.delta;
      }
    });
  }

  void _onPanEnd(DragEndDetails d) {
    final link = _linkDrag;
    if (link != null) {
      final scene = _scene(widget.layout);
      final target = dropTarget(scene, link.from, link.current);
      if (target != null) {
        _makeLink(link.from, target.ref);
      } else if (link.fromConnectedInput && hitTest(scene, link.current) is HitNothing) {
        _unlink(scene, link.from);
      }
    }
    final node = _draggingNode;
    if (node != null && _dragDelta != Offset.zero) {
      final layout = _effectiveLayout;
      widget.dispatch(NodeMoved(node, layout[node]!));
      // Into or out of a group region: membership follows the drop.  Only
      // the membership changes — a group is authoring metadata.
      if (_isSystemCanvas && node.kind == NodeKind.mapping) _membershipAfterDrop(node, layout);
    }
    final group = _draggingGroup;
    if (group != null && _dragDelta != Offset.zero) {
      final layout = _effectiveLayout;
      final g = _scene(widget.layout).groups.where((g) => g.id == group).firstOrNull;
      for (final m in g?.members ?? const <int>[]) {
        final ref = NodeRef.mapping(m);
        if (layout[ref] case final p?) widget.dispatch(NodeMoved(ref, p));
      }
    }
    setState(() {
      _linkDrag = null;
      _draggingNode = null;
      _draggingGroup = null;
      _dragDelta = Offset.zero;
      _panning = false;
    });
  }

  void _membershipAfterDrop(NodeRef node, Map<NodeRef, Offset> layout) {
    final scene = _scene(layout);
    final shape = scene.nodes.where((n) => n.ref == node).firstOrNull;
    if (shape == null) return;
    final current = widget.groups
        .where((g) => g.members.any((m) => m.toInt() == node.id))
        .firstOrNull;
    // The region of the node's own group is measured without the node, so
    // dragging out is possible.
    final sceneWithout = buildScene(
      widget.project,
      {...layout}..remove(node),
      system: widget.system,
    );
    final target = sceneWithout.groups
        .where(
          (g) =>
              g.rect.contains(shape.rect.center) &&
              !(g.members.length == 1 && g.members.first == node.id),
        )
        .lastOrNull;
    if (target == null) {
      if (current != null) {
        final own = sceneWithout.groups.where((g) => g.id == current.id.toInt()).firstOrNull;
        final stillInside = own != null && own.rect.contains(shape.rect.center);
        if (!stillInside && current.members.length > 1) {
          widget.dispatch(RemoveGroupMemberRequested(group: current.id.toInt(), decl: node.id));
        }
      }
      return;
    }
    if (current == null) {
      widget.dispatch(AddGroupMemberRequested(group: target.id, decl: node.id));
    } else if (current.id.toInt() != target.id) {
      widget.dispatch(MoveGroupMemberRequested(decl: node.id, to: target.id));
    }
  }

  /// A link is always output → input in data-flow terms, whichever end was
  /// dragged first.  Into a sink it is a drive: the mapping commits to the
  /// world there.
  void _makeLink(SocketRef a, SocketRef b) {
    final (out, inp) = a.side == SocketSide.output ? (a, b) : (b, a);
    // A binding: between ports, or between a port and a base relationship.
    final isBinding =
        out.role == SocketRole.port ||
        inp.role == SocketRole.port ||
        inp.role == SocketRole.realise;
    if (isBinding) {
      final source = out.bindingEnd;
      final destination = inp.bindingEnd;
      if (source != null && destination != null) {
        widget.dispatch(LinkEndsRequested(source: source, destination: destination));
      }
      return;
    }
    if (out.node.kind == NodeKind.concept && inp.node.kind == NodeKind.mapping) {
      widget.dispatch(LinkConceptToMappingInput(conceptId: out.concept, mappingId: inp.node.id));
    } else if (out.node.kind == NodeKind.mapping && inp.node.kind == NodeKind.concept) {
      widget.dispatch(LinkMappingOutputToConcept(mappingId: out.node.id, conceptId: inp.concept));
    } else if (out.node.kind == NodeKind.mapping && inp.node.kind == NodeKind.output) {
      widget.dispatch(SetMappingDriveRequested(mappingId: out.node.id, outputId: inp.node.id));
    }
  }

  /// Dragging a connected input away into empty space disconnects it: a
  /// mapping's read, or a sink's driver (the mapping stops driving).
  void _unlink(CanvasScene scene, SocketRef input) {
    // A bound port or realised relationship: the binding goes.
    if (input.role == SocketRole.port || input.role == SocketRole.realise) {
      for (final l in scene.links.where((l) => l.to == input && l.binding != null)) {
        widget.dispatch(UnbindRequested(l.binding!));
      }
      return;
    }
    switch (input.node.kind) {
      case NodeKind.mapping:
        widget.dispatch(UnlinkMappingInput(mappingId: input.node.id, conceptId: input.concept));
      case NodeKind.output:
        for (final l in scene.links.where((l) => l.to == input)) {
          widget.dispatch(SetMappingDriveRequested(mappingId: l.from.node.id, outputId: null));
        }
      case NodeKind.concept:
      case NodeKind.instance:
      case NodeKind.group:
        break;
    }
  }

  Selection _select(NodeRef ref) => switch (ref.kind) {
    NodeKind.concept => ConceptSelected(ref.id),
    NodeKind.mapping => MappingSelected(ref.id),
    NodeKind.output => OutputSelected(ref.id),
    NodeKind.instance => InstanceSelected(ref.id),
    NodeKind.group => GroupSelected(ref.id),
  };

  void _frameAll(Size viewport) {
    final scene = _scene(widget.layout);
    final b = scene.bounds.inflate(40);
    final zoom = (viewport.width / b.width).clamp(0.25, 1.0).clamp(0.0, viewport.height / b.height);
    setState(() {
      _zoom = zoom.clamp(0.25, 1.0);
      _pan = Offset(
        (viewport.width - b.width * _zoom) / 2 - b.left * _zoom,
        (viewport.height - b.height * _zoom) / 2 - b.top * _zoom,
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final scene = _scene(_effectiveLayout);
    final selected = switch (widget.selection) {
      ConceptSelected(:final id) => NodeRef.concept(id),
      MappingSelected(:final id) => NodeRef.mapping(id),
      OutputSelected(:final id) => NodeRef.output(id),
      InstanceSelected(:final id) => NodeRef.instance(id),
      GroupSelected(:final id) => NodeRef.group(id),
      _ => null,
    };
    final selectedBinding = switch (widget.selection) {
      BindingSelected(:final id) => id,
      _ => null,
    };

    return LayoutBuilder(
      builder: (context, constraints) {
        final size = constraints.biggest;
        return Focus(
          focusNode: _focus,
          onKeyEvent: (node, event) {
            if (event is! KeyDownEvent) return KeyEventResult.ignored;
            final key = event.logicalKey;
            if (key == LogicalKeyboardKey.backspace || key == LogicalKeyboardKey.delete) {
              widget.dispatch(const DeleteSelectionRequested());
              return KeyEventResult.handled;
            }
            if (key == LogicalKeyboardKey.home ||
                (key == LogicalKeyboardKey.digit0 && HardwareKeyboard.instance.isMetaPressed)) {
              _frameAll(size);
              return KeyEventResult.handled;
            }
            if (key == LogicalKeyboardKey.escape) {
              widget.dispatch(const SelectionChanged(NoSelection()));
              return KeyEventResult.handled;
            }
            return KeyEventResult.ignored;
          },
          child: DragTarget<ConceptTemplateDrag>(
            onWillAcceptWithDetails: (_) => widget.canInsert,
            onAcceptWithDetails: (d) {
              final box = context.findRenderObject() as RenderBox?;
              if (box == null) return;
              _insert(d.data.templateId, _toScene(box.globalToLocal(d.offset)));
            },
            builder: (context, candidates, _) => MenuAnchor(
              controller: _menu,
              consumeOutsideTap: true,
              menuChildren: _menuItems(context),
              child: Listener(
                onPointerSignal: _onPointerSignal,
                onPointerHover: _onHover,
                child: MouseRegion(
                  // Over a socket the link cannot reach, the pointer says so
                  // before the drop: the typing rule is refused, not diagnosed.
                  cursor: _linkDrag != null && _hoverSocket != null
                      ? (canLink(_linkDrag!.from, _hoverSocket!)
                            ? SystemMouseCursors.precise
                            : SystemMouseCursors.forbidden)
                      : _hoverSocket != null
                      ? SystemMouseCursors.precise
                      : _hoverNode != null
                      ? SystemMouseCursors.grab
                      : SystemMouseCursors.basic,
                  onExit: (_) => setState(() {
                    _hoverNode = null;
                    _hoverSocket = null;
                  }),
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onPanStart: _onPanStart,
                    onPanUpdate: _onPanUpdate,
                    onPanEnd: _onPanEnd,
                    onSecondaryTapDown: _onSecondaryTapDown,
                    onDoubleTapDown: _onDoubleTapDown,
                    child: ClipRect(
                      child: Stack(
                        children: [
                          CustomPaint(
                            painter: _CanvasPainter(
                              scene: scene,
                              tokens: t,
                              pan: _pan,
                              zoom: _zoom,
                              selected: selected,
                              selectedBinding: selectedBinding,
                              hovered: _hoverNode,
                              hoveredSocket: _hoverSocket,
                              linkDrag: _linkDrag,
                              dropOk: _linkDrag == null
                                  ? null
                                  : dropTarget(scene, _linkDrag!.from, _linkDrag!.current)?.ref,
                            ),
                            size: Size.infinite,
                          ),
                          if (candidates.isNotEmpty)
                            Positioned.fill(
                              child: IgnorePointer(
                                child: DecoratedBox(
                                  decoration: BoxDecoration(
                                    border: Border.all(color: t.accent, width: 2),
                                  ),
                                ),
                              ),
                            ),
                          if (widget.renaming case final node?) ...[
                            for (final shape in scene.nodes.where((n) => n.ref == node))
                              _InlineRename(
                                key: ValueKey(node),
                                rect: Rect.fromLTWH(
                                  shape.rect.left * _zoom + _pan.dx,
                                  shape.rect.top * _zoom + _pan.dy,
                                  shape.rect.width * _zoom,
                                  NodeMetrics.headerHeight * _zoom,
                                ),
                                zoom: _zoom,
                                initial: shape.title,
                                onDone: (name) =>
                                    widget.dispatch(InlineRenameFinished(node, name: name)),
                              ),
                            for (final g in scene.groups.where((g) => NodeRef.group(g.id) == node))
                              _InlineRename(
                                key: ValueKey(node),
                                rect: Rect.fromLTWH(
                                  g.rect.left * _zoom + _pan.dx,
                                  g.rect.top * _zoom + _pan.dy,
                                  (g.rect.width / 2).clamp(120, 320) * _zoom,
                                  NodeMetrics.regionTitle * _zoom,
                                ),
                                zoom: _zoom,
                                initial: g.title,
                                onDone: (name) =>
                                    widget.dispatch(InlineRenameFinished(node, name: name)),
                              ),
                          ],
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

/// The name field over a node's header (Finder's new-folder behaviour):
/// the whole name selected, Return commits, Esc keeps the current name,
/// leaving the field commits what was typed.
class _InlineRename extends StatefulWidget {
  const _InlineRename({
    super.key,
    required this.rect,
    required this.zoom,
    required this.initial,
    required this.onDone,
  });
  final Rect rect;
  final double zoom;
  final String initial;

  /// `null` means "keep the current name".
  final void Function(String? name) onDone;

  @override
  State<_InlineRename> createState() => _InlineRenameState();
}

class _InlineRenameState extends State<_InlineRename> {
  late final TextEditingController _controller = TextEditingController(text: widget.initial)
    ..selection = TextSelection(baseOffset: 0, extentOffset: widget.initial.length);
  final FocusNode _focus = FocusNode(debugLabel: 'inline rename');
  bool _done = false;

  @override
  void initState() {
    super.initState();
    _focus.addListener(() {
      if (!_focus.hasFocus) _finish(_controller.text);
    });
  }

  @override
  void dispose() {
    _focus.dispose();
    _controller.dispose();
    super.dispose();
  }

  void _finish(String? name) {
    if (_done) return;
    _done = true;
    widget.onDone(name);
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Positioned.fromRect(
      rect: widget.rect.deflate(3),
      child: Focus(
        onKeyEvent: (_, e) {
          if (e is KeyDownEvent && e.logicalKey == LogicalKeyboardKey.escape) {
            _finish(null);
            return KeyEventResult.handled;
          }
          return KeyEventResult.ignored;
        },
        child: TextField(
          controller: _controller,
          focusNode: _focus,
          autofocus: true,
          style: TextStyle(
            fontSize: 12.5 * widget.zoom,
            fontWeight: FontWeight.w600,
            color: t.textPrimary,
          ),
          decoration: InputDecoration(
            isDense: true,
            filled: true,
            fillColor: t.content,
            contentPadding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(4),
              borderSide: BorderSide(color: t.accent),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(4),
              borderSide: BorderSide(color: t.accent),
            ),
          ),
          onSubmitted: _finish,
        ),
      ),
    );
  }
}

class _CanvasPainter extends CustomPainter {
  _CanvasPainter({
    required this.scene,
    required this.tokens,
    required this.pan,
    required this.zoom,
    required this.selected,
    required this.selectedBinding,
    required this.hovered,
    required this.hoveredSocket,
    required this.linkDrag,
    required this.dropOk,
  });

  final CanvasScene scene;
  final MacTokens tokens;
  final Offset pan;
  final double zoom;
  final NodeRef? selected;
  final int? selectedBinding;
  final NodeRef? hovered;
  final SocketRef? hoveredSocket;
  final _LinkDrag? linkDrag;
  final SocketRef? dropOk;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..color = tokens.canvas);
    _grid(canvas, size);

    canvas.save();
    canvas.translate(pan.dx, pan.dy);
    canvas.scale(zoom);

    // Group regions first: backgrounds, not boxes (node-editor rule 8).
    final painter = NodePainter(
      tokens,
      hoveredSocket: hoveredSocket,
      dropOk: dropOk,
      compatible: linkDrag == null ? const {} : compatibleSockets(scene, linkDrag!.from),
    );
    for (final g in scene.groups) {
      painter.region(
        canvas,
        g,
        selected: selected == NodeRef.group(g.id),
        hovered: hovered == NodeRef.group(g.id),
      );
    }

    for (final l in scene.links) {
      final isSelected = l.binding != null && l.binding == selectedBinding;
      canvas.drawPath(
        l.path,
        Paint()
          ..color = isSelected ? tokens.accent : tokens.conceptColor(l.concept)
          ..style = PaintingStyle.stroke
          ..strokeWidth = isSelected ? 3 : 2
          ..strokeCap = StrokeCap.round,
      );
      // A transported binding: a gate where the value crosses domains,
      // with its initial value.
      if (l.transport case final init?) {
        painter.gate(canvas, l.midpoint, init, tokens.conceptColor(l.concept));
      }
    }
    final drag = linkDrag;
    if (drag != null) {
      canvas.drawPath(
        linkPath(drag.start, drag.current),
        Paint()
          ..color = tokens.conceptColor(drag.from.concept).withValues(alpha: 0.7)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2,
      );
    }
    for (final n in scene.nodes) {
      painter.node(canvas, n, selected: n.ref == selected, hovered: n.ref == hovered);
    }
    canvas.restore();

    // An empty design names its first step (the canvas is the entry point).
    if (scene.nodes.isEmpty) {
      final tp = TextPainter(
        text: TextSpan(
          text: 'Add a concept from the Library to start',
          style: TextStyle(
            fontFamily: '.AppleSystemUIFont',
            fontSize: 13,
            color: tokens.textTertiary,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      tp.paint(canvas, Offset((size.width - tp.width) / 2, (size.height - tp.height) / 2));
    }
  }

  /// One semantics node per canvas node, in reading order, labelled in
  /// product language — the painted graph is otherwise invisible to
  /// assistive technology.
  @override
  SemanticsBuilderCallback get semanticsBuilder => (size) {
    final nodes = [...scene.nodes]
      ..sort((a, b) {
        final dx = a.rect.left.compareTo(b.rect.left);
        return dx != 0 ? dx : a.rect.top.compareTo(b.rect.top);
      });
    return [
      for (final n in nodes)
        CustomPainterSemantics(
          rect: Rect.fromLTWH(
            n.rect.left * zoom + pan.dx,
            n.rect.top * zoom + pan.dy,
            n.rect.width * zoom,
            n.rect.height * zoom,
          ),
          properties: SemanticsProperties(
            label: _describe(n),
            textDirection: TextDirection.ltr,
            selected: n.ref == selected,
            button: true,
          ),
        ),
    ];
  };

  String _describe(NodeShape n) {
    switch (n.ref.kind) {
      case NodeKind.concept:
        final kind = n.sockets.first.kind;
        final form = switch (kind) {
          SocketKind.open => 'value not decided',
          SocketKind.quantity => 'a quantity',
          SocketKind.onOff => 'on or off',
          SocketKind.count => 'a count',
        };
        return '${n.title}, concept, $form';
      case NodeKind.mapping:
        final reads = n.sockets
            .where((s) => s.ref.side == SocketSide.input)
            .map((s) => n.socketLabels[s.ref] ?? '')
            .join(', ');
        final produces = n.sockets
            .where((s) => s.ref.side == SocketSide.output)
            .map((s) => n.socketLabels[s.ref] ?? '')
            .join(', ');
        final state = n.declared
            ? 'declared, not yet defined'
            : n.wrong
            ? 'definition does not check'
            : 'defined';
        return '${n.title}, relationship, reads $reads, produces $produces, $state';
      case NodeKind.output:
        final accepts = n.socketLabels.values.join(', ');
        final state = switch (n.sink) {
          SinkState.open => 'no timing domain yet',
          SinkState.undriven => 'undriven',
          SinkState.driven => 'driven',
          SinkState.illFormed => 'driven by an ill-formed connection',
          SinkState.contested => 'contested by several drivers',
          null => '',
        };
        return '${n.title}, physical output, accepts $accepts, '
            '${n.required ? 'required' : 'optional'}, $state';
      case NodeKind.instance:
        final requires = n.sockets
            .where((s) => s.ref.side == SocketSide.input)
            .map((s) => '${n.socketLabels[s.ref] ?? ''}${s.open ? ' (open)' : ''}')
            .join(', ');
        final provides = n.sockets
            .where((s) => s.ref.side == SocketSide.output)
            .map((s) => n.socketLabels[s.ref] ?? '')
            .join(', ');
        return '${n.title}, instance of ${n.subtitle}, requires $requires, provides $provides'
            '${n.unrealized ? ', body does not keep its promise' : ''}';
      case NodeKind.group:
        final ins = n.sockets
            .where((s) => s.ref.side == SocketSide.input)
            .map((s) => n.socketLabels[s.ref] ?? '')
            .join(', ');
        final outs = n.sockets
            .where((s) => s.ref.side == SocketSide.output)
            .map((s) => n.socketLabels[s.ref] ?? '')
            .join(', ');
        return '${n.title}, collapsed group of ${n.subtitle}, reads $ins, produces $outs';
    }
  }

  void _grid(Canvas canvas, Size size) {
    const step = 24.0;
    final paint = Paint()
      ..color = tokens.canvasGrid
      ..strokeWidth = 1;
    final s = step * zoom;
    if (s < 8) return;
    final ox = pan.dx % s;
    final oy = pan.dy % s;
    for (var x = ox; x < size.width; x += s) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (var y = oy; y < size.height; y += s) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(_CanvasPainter old) => true;
}

/// Paints one node the way the canvas does — shared with previews, chips
/// and library rows so a concept looks the same wherever it appears.
///
/// What the geometry means (STUDIO_UI.md §7): socket hue = identity, socket
/// shape = value form (○ quantity, ◇ on–off, □ count, hollow ring while
/// undecided), dashed outline = declared-not-defined, a red mark at the
/// definition line = the definition does not check.  No other state is
/// written on the node.
class NodePainter {
  NodePainter(
    this.tokens, {
    this.hoveredSocket,
    this.dropOk,
    this.compatible = const {},
    Color Function(int)? conceptColor,
  }) : conceptColor = conceptColor ?? tokens.conceptColor;
  final MacTokens tokens;
  final SocketRef? hoveredSocket;
  final SocketRef? dropOk;

  /// While a link is being dragged: every socket it could legally land on
  /// (same identity, opposite side, other node).  Drawn with a faint halo
  /// so the typing rule is visible before the drop, not after.
  final Set<SocketRef> compatible;

  /// Socket/link colour per concept id.  Previews of a concept that does
  /// not exist yet pass a neutral colour: its real hue is decided by the
  /// identity the compiler allocates.
  final Color Function(int) conceptColor;

  void node(Canvas canvas, NodeShape n, {bool selected = false, bool hovered = false}) {
    final rrect = RRect.fromRectAndRadius(n.rect, const Radius.circular(NodeMetrics.cornerRadius));
    canvas.drawRRect(
      rrect.shift(const Offset(0, 1)),
      Paint()
        ..color = const Color(0x22000000)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3),
    );
    canvas.drawRRect(rrect, Paint()..color = tokens.content);

    // Category tint: the whole concept object, the mapping's header strip,
    // a warmer strip for the physical boundary.
    final headerColor = switch (n.ref.kind) {
      NodeKind.concept => tokens.isDark ? const Color(0xFF3A4556) : const Color(0xFFDCE3EE),
      NodeKind.mapping => tokens.isDark ? const Color(0xFF2E4A6B) : const Color(0xFFCFE0F5),
      NodeKind.output => tokens.isDark ? const Color(0xFF4A4030) : const Color(0xFFEFE3CF),
      // A component instance: a teal-grey strip — a reusable behaviour,
      // seen from outside.
      NodeKind.instance => tokens.isDark ? const Color(0xFF2F4F4A) : const Color(0xFFD0E6E1),
      // A collapsed group: the region tint, as a box.
      NodeKind.group => tokens.isDark ? const Color(0xFF4A3F5C) : const Color(0xFFE4DCF0),
    };
    canvas.save();
    canvas.clipRRect(rrect);
    canvas.drawRect(n.header, Paint()..color = headerColor);
    canvas.restore();

    // Outline: hairline at rest, secondary on hover, accent when selected.
    // Declared stays dashed in every state — selection changes the colour,
    // never the meaning.
    final outline = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = selected ? 2 : 1
      ..color = selected
          ? tokens.accent
          : hovered
          ? tokens.textSecondary
          : n.declared
          ? tokens.textTertiary
          : tokens.hairline;
    // An undriven or open sink is incomplete, not wrong: dashed like a
    // declared mapping.
    final dashed = n.declared || n.sink == SinkState.open || n.sink == SinkState.undriven;
    if (dashed) {
      _dashedRRect(canvas, rrect, outline..color = selected ? tokens.accent : tokens.textTertiary);
    } else {
      canvas.drawRRect(rrect, outline);
    }
    // The physical boundary: a solid bar on the sink's right edge — to the
    // right of it is the world, and nothing reads from there.
    if (n.ref.kind == NodeKind.output) {
      canvas.drawLine(
        n.rect.topRight + const Offset(-1, NodeMetrics.cornerRadius),
        n.rect.bottomRight + const Offset(-1, -NodeMetrics.cornerRadius),
        Paint()
          ..color = selected ? tokens.accent : tokens.textSecondary
          ..strokeWidth = 3
          ..strokeCap = StrokeCap.round,
      );
    }

    _text(
      canvas,
      n.title,
      n.header.topLeft + const Offset(12, 6),
      FontWeight.w600,
      12.5,
      tokens.textPrimary,
      maxWidth: n.rect.width - (n.declared ? 78 : 24),
    );
    // The header's right word is object state in words only where the
    // geometry cannot carry it: a declared mapping, an open or contested
    // sink, a required sink, a port-backed relationship in a component's
    // source.
    final headerWord = n.declared
        ? 'declared'
        : n.headerWord.isNotEmpty
        ? n.headerWord
        : switch (n.sink) {
            SinkState.open => 'no domain',
            SinkState.contested => 'contested',
            SinkState.illFormed => 'ill-formed',
            _ => n.required ? 'required' : '',
          };
    if (headerWord.isNotEmpty) {
      _text(
        canvas,
        headerWord,
        n.header.topRight + const Offset(-10, 8),
        FontWeight.w400,
        10,
        n.sink == SinkState.contested || n.sink == SinkState.illFormed
            ? tokens.error
            : tokens.textSecondary,
        alignRight: true,
      );
    }
    // The timing domain, quietly, at the body's right edge.
    if (n.timing.isNotEmpty) {
      final at = n.ref.kind == NodeKind.mapping
          ? n.definitionRegion.topRight + const Offset(-10, 5)
          : n.rect.bottomRight + const Offset(-12, -NodeMetrics.rowHeight + 5);
      _text(
        canvas,
        '↻ ${n.timing}',
        at,
        FontWeight.w400,
        10,
        tokens.textTertiary,
        alignRight: true,
      );
    }

    for (final s in n.sockets) {
      socket(canvas, s.center, s.kind, conceptColor(s.ref.concept), ref: s.ref, open: s.open);
      final label = n.socketLabels[s.ref];
      if (label != null) {
        if (s.ref.side == SocketSide.input) {
          _text(
            canvas,
            label,
            s.center + const Offset(12, -7),
            FontWeight.w400,
            11,
            tokens.textPrimary,
            maxWidth: n.rect.width - 24,
          );
        } else {
          _text(
            canvas,
            label,
            s.center + const Offset(-12, -7),
            FontWeight.w400,
            11,
            tokens.textPrimary,
            alignRight: true,
            maxWidth: n.rect.width / 2,
          );
        }
      }
    }

    // An instance's body row names its component; a collapsed group's, its
    // size.  A body that no longer keeps its promise gets the red mark.
    if (n.ref.kind == NodeKind.instance || n.ref.kind == NodeKind.group) {
      final region = n.definitionRegion;
      canvas.drawLine(
        region.topLeft + const Offset(1, 0),
        region.topRight + const Offset(-1, 0),
        Paint()..color = tokens.hairline,
      );
      var left = region.left + 10;
      if (n.unrealized) {
        canvas.drawCircle(Offset(left + 3, region.center.dy), 3, Paint()..color = tokens.error);
        left += 12;
      }
      _text(
        canvas,
        n.subtitle,
        Offset(left, region.top + 5),
        FontWeight.w400,
        11,
        tokens.textSecondary,
        italic: n.ref.kind == NodeKind.group,
        maxWidth: region.right - 10 - left - (n.timing.isEmpty ? 0 : 64),
      );
    }

    // Definition region: the summary line, or nothing while declared.  A
    // definition that does not check gets the red mark here — where the
    // problem lives — and nowhere else on the node.
    if (n.ref.kind == NodeKind.mapping) {
      final region = n.definitionRegion;
      canvas.drawLine(
        region.topLeft + const Offset(1, 0),
        region.topRight + const Offset(-1, 0),
        Paint()..color = tokens.hairline,
      );
      final definition = n.definition;
      if (definition != null) {
        var left = region.left + 10;
        if (n.wrong) {
          canvas.drawCircle(Offset(left + 3, region.center.dy), 3, Paint()..color = tokens.error);
          left += 12;
        }
        _text(
          canvas,
          definition,
          Offset(left, region.top + 5),
          FontWeight.w400,
          11,
          n.wrong ? tokens.textPrimary : tokens.textSecondary,
          maxWidth: region.right - 10 - left - (n.timing.isEmpty ? 0 : 64),
        );
      }
    }
  }

  /// One socket: shape by value form, hue by identity.  Reused by every
  /// widget that shows a concept (library rows, chips, toggles) so the mark
  /// is learned once.
  /// [open]: a port nobody has bound (or an open base relationship's
  /// realisation socket) — drawn hollow in its hue: the value form is
  /// known, the value is not supplied.
  void socket(
    Canvas canvas,
    Offset c,
    SocketKind kind,
    Color color, {
    SocketRef? ref,
    bool open = false,
  }) {
    const r = NodeMetrics.socketRadius;
    if (ref != null) {
      if (dropOk == ref) {
        canvas.drawCircle(c, r + 4, Paint()..color = color.withValues(alpha: 0.35));
      } else if (compatible.contains(ref)) {
        canvas.drawCircle(c, r + 3, Paint()..color = color.withValues(alpha: 0.16));
      }
    }
    final path = socketPath(c, kind, r);
    canvas.drawPath(path, Paint()..color = tokens.content);
    final hollow = kind == SocketKind.open || open;
    if (!hollow) {
      canvas.drawPath(path, Paint()..color = color);
    }
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = hollow ? color : tokens.content.withValues(alpha: 0.9),
    );
  }

  /// An expanded group region: a tinted, rounded background with a title
  /// band; members sit inside by position.  A picture, not a box the
  /// compiler knows.
  void region(Canvas canvas, GroupShape g, {bool selected = false, bool hovered = false}) {
    final rrect = RRect.fromRectAndRadius(g.rect, const Radius.circular(10));
    canvas.drawRRect(
      rrect,
      Paint()
        ..color = (tokens.isDark ? const Color(0xFF7A66A8) : const Color(0xFF8C6FC2)).withValues(
          alpha: 0.12,
        ),
    );
    canvas.drawRRect(
      rrect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = selected ? 2 : 1
        ..color = selected
            ? tokens.accent
            : hovered
            ? tokens.textSecondary
            : (tokens.isDark ? const Color(0xFF7A66A8) : const Color(0xFF8C6FC2)).withValues(
                alpha: 0.5,
              ),
    );
    _text(
      canvas,
      g.title,
      g.rect.topLeft + const Offset(10, 4),
      FontWeight.w600,
      11.5,
      tokens.textSecondary,
      maxWidth: g.rect.width - 20,
    );
  }

  /// A transport gate on a link: a short vertical bar with the initial
  /// value beside it — the value is carried across timing domains and
  /// starts there.
  void gate(Canvas canvas, Offset at, String init, Color color) {
    canvas.drawLine(
      at + const Offset(0, -7),
      at + const Offset(0, 7),
      Paint()
        ..color = color
        ..strokeWidth = 3
        ..strokeCap = StrokeCap.round,
    );
    canvas.drawLine(
      at + const Offset(4, -7),
      at + const Offset(4, 7),
      Paint()
        ..color = tokens.content
        ..strokeWidth = 1.5,
    );
    _text(canvas, init, at + const Offset(8, -14), FontWeight.w400, 10, tokens.textSecondary);
  }

  /// The socket outline for a value form, centred on [c] with radius [r].
  static Path socketPath(Offset c, SocketKind kind, double r) {
    switch (kind) {
      case SocketKind.open:
      case SocketKind.quantity:
        return Path()..addOval(Rect.fromCircle(center: c, radius: r));
      case SocketKind.onOff:
        final d = r * 1.25;
        return Path()
          ..moveTo(c.dx, c.dy - d)
          ..lineTo(c.dx + d, c.dy)
          ..lineTo(c.dx, c.dy + d)
          ..lineTo(c.dx - d, c.dy)
          ..close();
      case SocketKind.count:
        final h = r * 0.92;
        return Path()..addRect(Rect.fromCenter(center: c, width: 2 * h, height: 2 * h));
    }
  }

  void _dashedRRect(Canvas canvas, RRect r, Paint paint) {
    final path = Path()..addRRect(r);
    for (final metric in path.computeMetrics()) {
      var d = 0.0;
      while (d < metric.length) {
        canvas.drawPath(metric.extractPath(d, d + 5), paint);
        d += 9;
      }
    }
  }

  void _text(
    Canvas canvas,
    String s,
    Offset at,
    FontWeight weight,
    double size,
    Color color, {
    double maxWidth = 200,
    bool alignRight = false,
    bool italic = false,
  }) {
    final tp = TextPainter(
      text: TextSpan(
        text: s,
        style: TextStyle(
          fontFamily: '.AppleSystemUIFont',
          fontSize: size,
          fontWeight: weight,
          color: color,
          fontStyle: italic ? FontStyle.italic : FontStyle.normal,
        ),
      ),
      textDirection: TextDirection.ltr,
      maxLines: 1,
      ellipsis: '…',
    )..layout(maxWidth: maxWidth);
    tp.paint(canvas, alignRight ? at - Offset(tp.width, 0) : at);
  }
}

/// A single node rendered at canvas fidelity, for sheets and inspectors.
/// Build it from a synthetic projection so the same `buildScene` geometry
/// applies.
class NodePreview extends StatelessWidget {
  const NodePreview({
    super.key,
    required this.projection,
    required this.node,
    this.height = 96,
    this.neutralConcepts = const {},
  });
  final pb.ProjectProjection projection;
  final NodeRef node;
  final double height;

  /// Concept ids whose colour is not yet known (not created yet).
  final Set<int> neutralConcepts;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return SizedBox(
      height: height,
      child: CustomPaint(painter: _PreviewPainter(t, projection, node, neutralConcepts)),
    );
  }
}

class _PreviewPainter extends CustomPainter {
  _PreviewPainter(this.t, this.p, this.node, this.neutral);
  final MacTokens t;
  final pb.ProjectProjection p;
  final NodeRef node;
  final Set<int> neutral;

  Color _color(int id) => neutral.contains(id) ? t.textTertiary : t.conceptColor(id);

  @override
  void paint(Canvas canvas, Size size) {
    final scene = buildScene(p, const {});
    final shape = scene.nodes.where((n) => n.ref == node).firstOrNull;
    if (shape == null) return;
    // centre the node
    final dx = (size.width - shape.rect.width) / 2 - shape.rect.left;
    final dy = (size.height - shape.rect.height) / 2 - shape.rect.top;
    canvas.translate(dx, dy);
    // short link stubs so the sockets read as connectable
    final stub = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round;
    for (final s in shape.sockets) {
      final c = _color(s.ref.concept).withValues(alpha: 0.6);
      final dir = s.ref.side == SocketSide.input ? -1.0 : 1.0;
      canvas.drawLine(
        s.center + Offset(dir * 9, 0),
        s.center + Offset(dir * 28, 0),
        stub..color = c,
      );
    }
    NodePainter(t, conceptColor: _color).node(canvas, shape);
  }

  @override
  bool shouldRepaint(_PreviewPainter old) =>
      old.p != p || old.node != node || old.t != t || old.neutral != neutral;
}
