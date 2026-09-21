/// The node canvas: one interaction state machine over [buildScene]
/// (docs/architecture/studio-ui.md §2, "Interaction").
///
/// Every pointer sequence has exactly one owner.  A primary press is a
/// *candidate* until it moves past the drag threshold; then it becomes one
/// of: a marquee (on empty canvas — a window when dragged left → right,
/// a crossing when dragged right → left, the CAD convention), a node move
/// (the selected set moves as one), a group move, a link drag (from a
/// socket), or a pan (middle button, Space held, or a trackpad).  A click
/// that never moved selects.  An open menu is a modal input state: the
/// canvas takes no pointer, hover or wheel until it is dismissed, and the
/// pointer that dismisses it edits nothing.
///
/// High-frequency state (pan, zoom, an in-progress gesture) lives here as
/// widget state and never reaches the reducer.  Only *results* are
/// dispatched: positions on release, a link made, a selection.
library;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart' show SemanticsProperties;
import 'package:flutter/services.dart';

import '../../l10n/l10n.dart';
import '../../app/actions.dart';
import '../../app/state.dart';
import '../../platform/desktop.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../../l10n/library_strings.dart';
import '../library_panel.dart' show LibraryItemDrag, categoryLabel, itemName;
import '../mac/menus.dart';
import '../mac/tokens.dart';
import '../expanded_formula.dart';
import 'canvas_affordance.dart';
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
    this.refs = const {},
    this.slots = const {},
    this.templates = const [],
    this.sources = const [],
    this.recentTemplates = const [],
    this.renaming,
    this.canInsert = true,
    this.system = const SystemSceneInput(),
    this.context = const SystemContext(),
    this.components = const [],
    this.groups = const [],
    this.viewport,
    this.groupsEnabled = false,
    this.actions,
    this.hasSources = false,
    this.expanded = const {},
    this.previews = const {},
    this.analyses = const {},
    this.frameRequest = 0,
    this.canUndoArrange = false,
  });

  /// Bumped when the canvas should frame the whole design once (after an
  /// arrangement, ADR-0023 §7): never on the designer's own moves.
  final int frameRequest;

  /// Whether a layout from before the last arrangement is kept: the
  /// canvas menu's _Undo Arrange_.
  final bool canUndoArrange;

  /// The mappings whose saved formula is shown on the node, with the
  /// height of the picture (measured once drawn; the initial height
  /// before), the committed definitions' projections fetched for them
  /// (`FormulaPreview`), and the analyses their findings come from.
  final Map<int, double> expanded;
  final Map<int, FormulaPreview> previews;
  final Map<int, pb.MappingAnalysis> analyses;

  final pb.ProjectProjection project;
  final Map<NodeRef, Offset> layout;
  final Selection selection;
  final void Function(AppAction) dispatch;

  /// Per mapping id, the open positions of its definition
  /// (`MappingAnalysis.slots`, the same analysis): the mapping block's
  /// slot sockets, where a dropped Sem block goes.
  final Map<int, List<String>> slots;

  /// A system project's instances, bindings, groups and their verdicts;
  /// empty for a flat project and inside a component's source.
  final SystemSceneInput system;
  final DesignContext context;

  /// The components an instance can be placed of (right-click menu).
  final List<pb.ComponentView> components;

  /// The groups a relationship can be added to (those of the design on
  /// screen).
  final List<pb.BehaviorGroupView> groups;

  /// Where the designer left this canvas; `null` on a fresh canvas.
  final CanvasViewport? viewport;

  /// Whether behaviour groups exist here (a system project, in any
  /// context).
  final bool groupsEnabled;

  /// Compiler verdicts per mapping id, when an analysis of this revision
  /// exists.
  final Map<int, pb.MappingStatus> statuses;

  /// The output pass per sink id, when an analysis of this revision exists.
  final Map<int, pb.OutputState> outputStates;

  /// Per mapping id, the mappings its definition references
  /// (`MappingAnalysis.references`, the same analysis): the reference edges.
  final Map<int, List<int>> refs;

  /// The concept templates the right-click menu offers, and the recently
  /// used ones (most recent first).
  final List<pb.ConceptTemplateView> templates;

  /// The Source items of the library, for the Add Source menu.
  final List<pb.LibraryItemView> sources;
  final List<String> recentTemplates;

  /// The node whose name is open for inline editing, if any.
  final NodeRef? renaming;

  /// False while an insertion is awaiting the daemon.
  final bool canInsert;

  /// The IDE service's actions for the selected object (the reducer asks on
  /// every selection change): the contextual menu's *Fix* section.
  final SemanticActionsState? actions;

  /// Whether the project's source text is authored (a text project): the
  /// menu then offers *Reveal in Code*.
  final bool hasSources;

  @override
  State<NodeCanvas> createState() => _NodeCanvasState();
}

/// Movement past this many logical pixels makes a press a drag; below it
/// a press is a click.  One threshold for marquee, move and pan.
const double kDragThreshold = 4;

/// Two primary clicks on the same spot within this interval are a
/// double-click (rename / enter a component).
const Duration kDoubleClickInterval = Duration(milliseconds: 350);

/// Below this zoom every group reads as its summary box (semantic zoom);
/// the authored collapse state is untouched.
const double kSummarizeBelowZoom = 0.5;

// ---------------------------------------------------------------------------
// Menu contexts
// ---------------------------------------------------------------------------

/// What a contextual menu is about — one explicit thing, never inferred
/// from a nullable node and a stale selection.
sealed class MenuContext {
  const MenuContext();
}

/// Empty canvas: creation and canvas commands only.
class CanvasMenuContext extends MenuContext {
  const CanvasMenuContext(this.scene);
  final Offset scene;
}

/// One node (a concept, a relationship, a sink, an instance, a collapsed
/// group box): the object's own commands.
class NodeMenuContext extends MenuContext {
  const NodeMenuContext(this.node);
  final NodeRef node;
}

/// An expanded group's title band.
class GroupMenuContext extends MenuContext {
  const GroupMenuContext(this.group);
  final int group;
}

/// A signature or binding link: its ends and its disconnection.
class LinkMenuContext extends MenuContext {
  const LinkMenuContext(this.link);
  final LinkShape link;
}

/// Several selected nodes, the right-click on one of them: commands about
/// the set.
class SelectionMenuContext extends MenuContext {
  const SelectionMenuContext(this.nodes, {this.active});
  final Set<NodeRef> nodes;
  final NodeRef? active;
}

// ---------------------------------------------------------------------------
// Gesture states
// ---------------------------------------------------------------------------

/// The one owner of the pointer.  Transitions happen in the pointer
/// handlers only; every state knows how to cancel itself (Esc).
sealed class _Gesture {
  const _Gesture();
}

class _Idle extends _Gesture {
  const _Idle();
}

/// A button went down; nothing is decided until the pointer moves past
/// [kDragThreshold] or comes up.
class _PressCandidate extends _Gesture {
  const _PressCandidate({
    required this.downLocal,
    required this.downScene,
    required this.hit,
    required this.buttons,
    required this.pan,
  });
  final Offset downLocal;
  final Offset downScene;
  final CanvasHit hit;
  final int buttons;

  /// Space was held, or the middle button: a drag will pan.
  final bool pan;
}

/// A rectangle selection.  [mode] follows the horizontal direction of the
/// drag on every update.  [add] (⌘/Ctrl) unions the result into the
/// selection, [subtract] (⇧) removes it; otherwise it replaces.
class _Marquee extends _Gesture {
  const _Marquee({
    required this.anchor,
    required this.current,
    required this.add,
    required this.subtract,
    required this.base,
  });
  final Offset anchor;
  final Offset current;
  final bool add;
  final bool subtract;

  /// The selection when the drag started (what add / subtract act on).
  final Set<NodeRef> base;

  Rect get rect => Rect.fromPoints(anchor, current);
  MarqueeMode get mode => marqueeMode(anchor, current);
}

class _Pan extends _Gesture {
  const _Pan();
}

/// The selected movable nodes travel together with the one under the
/// pointer; their positions at the press are the base.
class _DragNodes extends _Gesture {
  const _DragNodes({required this.grabbed, required this.base, required this.delta});
  final NodeRef grabbed;
  final Map<NodeRef, Offset> base;
  final Offset delta;
  _DragNodes moved(Offset d) => _DragNodes(grabbed: grabbed, base: base, delta: d);
}

/// An expanded group's title band dragged: every member moves.
class _DragGroup extends _Gesture {
  const _DragGroup({required this.group, required this.base, required this.delta});
  final int group;
  final Map<NodeRef, Offset> base;
  final Offset delta;
  _DragGroup moved(Offset d) => _DragGroup(group: group, base: base, delta: d);
}

/// A link being drawn from a socket.
class _DragLink extends _Gesture {
  _DragLink(this.from, this.start, {required this.fromConnectedInput}) : current = start;
  final SocketRef from;
  final Offset start;
  Offset current;

  /// The drag started on an aggregate socket standing for several
  /// concrete sources; the drop asks which.
  List<ProxyTarget>? proxyChoices;

  /// Dragging away from a connected mapping input: releasing on empty space
  /// disconnects (Blender: "drag the link away from its input socket").
  final bool fromConnectedInput;
}

/// The pointer that dismissed a menu: nothing else happens until it is up.
class _ConsumedByMenu extends _Gesture {
  const _ConsumedByMenu();
}

/// Which overlay menu owns the pointer, if any.
enum _MenuKind { context, chooser }

class _NodeCanvasState extends State<NodeCanvas> {
  late Offset _pan = widget.viewport?.pan ?? Offset.zero;
  late double _zoom = widget.viewport?.zoom ?? 1;

  _Gesture _gesture = const _Idle();

  /// The menu on show, its context, and where it opened (local).  The
  /// context is set with the menu and cleared with it; the menu's items
  /// are built from it on every rebuild.
  _MenuKind? _menuOpen;
  MenuContext? _menuContext;
  List<Widget> _chooser = const [];
  final MenuController _menu = MenuController();
  final MenuController _chooserMenu = MenuController();

  /// The expanded group the dragged relationship would join on release.
  int? _dragOverGroup;

  NodeRef? _hoverNode;
  SocketRef? _hoverSocket;
  LinkId? _hoverLink;

  /// The pointer is on the selected object's affordance (or just around
  /// it): the object stays hovered while the pointer crosses over to the
  /// icons.  The rectangle is what the last build placed, in local pixels;
  /// none when no affordance is on show.
  bool _overAffordance = false;
  Rect? _affordanceRect;
  bool _spaceHeld = false;
  final FocusNode _focus = FocusNode(debugLabel: 'canvas');

  /// The last primary click, for double-click detection.
  Duration? _lastClickAt;
  Offset? _lastClickLocal;

  @override
  void didUpdateWidget(NodeCanvas old) {
    super.didUpdateWidget(old);
    // Another canvas: its own viewport.
    if (old.context != widget.context) {
      _pan = widget.viewport?.pan ?? Offset.zero;
      _zoom = widget.viewport?.zoom ?? 1;
    }
    // A menu about something that is gone (a deleted node, another
    // canvas) closes: its commands would act on nothing.
    final ctx = _menuContext;
    if (ctx != null && !_contextStillValid(ctx)) _closeMenus();
    // An arrangement arrived: the whole design in view, once.
    if (widget.frameRequest != old.frameRequest) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _frameAll(_viewportSize);
      });
    }
    // A selected edge that is no longer drawn (its ends are, the edge
    // went with an edit made elsewhere) is no selection: the reducer only
    // knows the ends; the canvas knows the edges.
    if (widget.selection case LinkSelected(:final link)) {
      if (!_scene(widget.layout).links.any((l) => l.id == link)) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && widget.selection == LinkSelected(link)) {
            widget.dispatch(const SelectionChanged(NoSelection()));
          }
        });
      }
    }
  }

  bool _contextStillValid(MenuContext ctx) {
    final scene = _scene(widget.layout);
    return switch (ctx) {
      CanvasMenuContext() => true,
      NodeMenuContext(:final node) => scene.nodes.any((n) => n.ref == node),
      GroupMenuContext(:final group) => scene.groups.any((g) => g.id == group),
      LinkMenuContext(:final link) => scene.links.any(
        (l) => l.from == link.from && l.to == link.to,
      ),
      SelectionMenuContext(:final nodes) => nodes.every((n) => scene.nodes.any((s) => s.ref == n)),
    };
  }

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  void _viewportMoved() {
    widget.dispatch(ViewportChanged(pan: _pan, zoom: _zoom));
  }

  SystemSceneInput get _sceneInput => _zoom < kSummarizeBelowZoom
      ? SystemSceneInput(
          system: widget.system.system,
          analysis: widget.system.analysis,
          groups: widget.system.groups,
          boundaries: widget.system.boundaries,
          groupBoxes: widget.system.groupBoxes,
          portWords: widget.system.portWords,
          summarize: true,
        )
      : widget.system;

  /// The scene's system input with [decl] taken out of every group: the
  /// regions measured without the relationship being dragged (and without
  /// its mapping block), so dragging out of a group is possible.
  SystemSceneInput _sceneInputWithout(int decl) {
    final s = _sceneInput;
    return SystemSceneInput(
      system: s.system,
      analysis: s.analysis,
      groups: [
        for (final g in s.groups) g.deepCopy()..members.removeWhere((m) => m.toInt() == decl),
      ],
      boundaries: s.boundaries,
      groupBoxes: s.groupBoxes,
      portWords: s.portWords,
      summarize: s.summarize,
    );
  }

  CanvasScene _scene(Map<NodeRef, Offset> layout) => buildScene(
    widget.project,
    layout,
    statuses: widget.statuses,
    outputStates: widget.outputStates,
    refs: widget.refs,
    slots: widget.slots,
    system: _sceneInput,
    expanded: widget.expanded,
  );

  bool get _isSystemCanvas => widget.system.system != null && widget.context is SystemContext;
  bool get _groupsEnabled => widget.groupsEnabled;

  Offset _toScene(Offset local) => (local - _pan) / _zoom;
  Offset _toLocal(Offset scene) => scene * _zoom + _pan;

  /// Top-left of a new Sem block centred on a scene point.
  static Offset nodeOriginFor(Offset scenePoint) =>
      scenePoint - const Offset(NodeMetrics.semWidth / 2, NodeMetrics.headerHeight / 2);

  /// A value category opens the concept sheet at the point: the concept
  /// is named before it is created (ADR-0041), and lands here.
  void _newConceptAt(String presetId, Offset scenePoint) {
    widget.dispatch(NewConceptRequested(presetId: presetId, position: nodeOriginFor(scenePoint)));
  }

  /// A Source item is a preset for the Source sheet: nothing is created
  /// until the concept is chosen there (docs/spec/concept-library.md).
  void _newSourceAt(String presetId, Offset scenePoint) {
    widget.dispatch(NewSourceRequested(presetId: presetId, position: nodeOriginFor(scenePoint)));
  }

  // ---- selection algebra ---------------------------------------------------

  Set<NodeRef> get _selectedSet => selectedNodes(widget.selection);
  NodeRef? get _active => activeNode(widget.selection);

  /// The nodes the selection is drawn on: a selected Sem block's mapping
  /// block wears the outline too — one declaration, two nodes (ADR-0044).
  Set<NodeRef> get _selectedShapes => {
    for (final r in _selectedSet) ...[r, if (r.kind == NodeKind.mapping) NodeRef.definition(r.id)],
  };

  void _setSelection(Set<NodeRef> nodes, {NodeRef? active}) {
    final next = selectionOfNodes(nodes, active: active);
    if (next != widget.selection) widget.dispatch(SelectionChanged(next));
  }

  /// ⌘/Ctrl-click: toggle one node; the toggled-in node becomes active.
  void _toggle(NodeRef ref) {
    final next = {..._selectedSet};
    if (next.remove(ref)) {
      _setSelection(next, active: _active == ref ? null : _active);
    } else {
      _setSelection(next..add(ref), active: ref);
    }
  }

  /// A plain click on a node: it alone, unless it is already in the set
  /// (then the set stays and it becomes the active object).
  void _clickNode(NodeRef ref) {
    final current = _selectedSet;
    if (current.contains(ref)) {
      if (_active != ref) _setSelection(current, active: ref);
      return;
    }
    _setSelection({ref}, active: ref);
  }

  /// The nodes a marquee would produce, previewed and committed alike.
  Set<NodeRef> _marqueeResult(_Marquee m) {
    final taken = marqueeNodes(_scene(widget.layout), m.rect, m.mode);
    if (m.subtract) return m.base.difference(taken);
    if (m.add) return m.base.union(taken);
    return taken;
  }

  /// Everything a marquee or ⌘A may take: the scene's visible nodes.
  Set<NodeRef> get _eligibleNodes => {for (final n in _scene(widget.layout).nodes) n.ref};

  // ---- pointer ---------------------------------------------------------------

  bool get _menuIsOpen => _menuOpen != null;

  /// ⌘ on macOS, Ctrl elsewhere.
  bool get _primaryModifier => primaryModifierIsControl
      ? HardwareKeyboard.instance.isControlPressed
      : HardwareKeyboard.instance.isMetaPressed;

  /// The secondary button; on macOS also Control + the primary button
  /// (Control is free there — ⌘ is the selection modifier).
  bool _isContextButton(PointerDownEvent e) =>
      e.buttons & kSecondaryButton != 0 ||
      (!primaryModifierIsControl &&
          e.buttons & kPrimaryButton != 0 &&
          HardwareKeyboard.instance.isControlPressed);

  void _onPointerDown(PointerDownEvent e) {
    if (_menuIsOpen) {
      // A context press elsewhere retargets the menu: the old one closes,
      // the new one opens for what is under the pointer.  Any other
      // outside pointer dismisses the menu and does nothing else; the
      // menu's own surface is in the overlay and never reaches this
      // listener.
      if (_isContextButton(e)) {
        _openContextMenuAt(e.localPosition);
        return;
      }
      _closeMenus();
      setState(() => _gesture = const _ConsumedByMenu());
      return;
    }
    if (_gesture is! _Idle) return;
    // The affordance's buttons take their own press; the canvas under
    // them does not start a gesture on it.
    if (_affordanceRect?.contains(e.localPosition) ?? false) return;
    _focus.requestFocus();
    if (_isContextButton(e)) {
      _openContextMenuAt(e.localPosition);
      return;
    }
    final middle = e.buttons & kMiddleMouseButton != 0;
    if (e.buttons & kPrimaryButton == 0 && !middle) return;
    final scene = _scene(widget.layout);
    final p = _toScene(e.localPosition);
    setState(() {
      _gesture = _PressCandidate(
        downLocal: e.localPosition,
        downScene: p,
        hit: hitTest(scene, p, linkHitTolerance: linkTolerance(_zoom)),
        buttons: e.buttons,
        pan: middle || _spaceHeld,
      );
    });
  }

  void _onPointerMove(PointerMoveEvent e) {
    switch (_gesture) {
      case _PressCandidate(:final downLocal)
          when (e.localPosition - downLocal).distance >= kDragThreshold:
        _beginDrag(_gesture as _PressCandidate, e);
      case _PressCandidate() || _Idle() || _ConsumedByMenu():
        break;
      case _Marquee():
        final m = _gesture as _Marquee;
        setState(() {
          _gesture = _Marquee(
            anchor: m.anchor,
            current: _toScene(e.localPosition),
            add: m.add,
            subtract: m.subtract,
            base: m.base,
          );
        });
      case _Pan():
        setState(() => _pan += e.delta);
      case _DragNodes():
        final d = _gesture as _DragNodes;
        setState(() {
          _gesture = d.moved(d.delta + e.delta / _zoom);
          _updateDragOverGroup(d.grabbed);
        });
      case _DragGroup():
        final d = _gesture as _DragGroup;
        setState(() => _gesture = d.moved(d.delta + e.delta / _zoom));
      case _DragLink():
        final l = _gesture as _DragLink;
        final p = _toScene(e.localPosition);
        setState(() {
          l.current = p;
          // Hover is not reported while a button is down; track the socket
          // under the dragged link end so the cursor can refuse an illegal one.
          final hit = hitTest(_scene(widget.layout), p);
          _hoverSocket = hit is HitSocket ? hit.socket.ref : null;
        });
    }
  }

  /// The press moved past the threshold: decide what it is, once.
  void _beginDrag(_PressCandidate c, PointerMoveEvent e) {
    if (c.pan) {
      setState(() => _gesture = const _Pan());
      return;
    }
    final scene = _scene(widget.layout);
    switch (c.hit) {
      case HitSocket(:final socket):
        // An aggregate socket is a proxy: the drag starts from the one
        // concrete socket it stands for (several: the drop asks).
        final targets = scene.resolve(socket.ref);
        final from = targets.length == 1 && targets.single.socket != null
            ? targets.single.socket!
            : targets.isNotEmpty && targets.every((t) => t.socket != null)
            ? targets.first.socket!
            : socket.ref;
        final connected = from.side == SocketSide.input && scene.links.any((l) => l.to == from);
        setState(() {
          _gesture = _DragLink(from, socket.center, fromConnectedInput: connected)
            ..current = _toScene(e.localPosition)
            ..proxyChoices = targets.length > 1 ? targets : null;
        });
      case HitNode(:final node) || HitDisclosure(:final node):
        // Dragging a selected node moves the selected set; an unselected
        // one becomes the selection first (⌘/Ctrl adds it instead).  A
        // Sem block takes its mapping block along; a mapping block grabbed
        // by itself moves alone (its own position, ADR-0044).
        final decl = asDeclaration(node.ref);
        var set = _selectedSet;
        if (!set.contains(decl)) {
          set = _primaryModifier ? {...set, decl} : {decl};
          _setSelection(set, active: decl);
        }
        final moving = node.ref.kind == NodeKind.definition
            ? {node.ref}
            : {
                for (final r in set) ...[
                  r,
                  if (r.kind == NodeKind.mapping) NodeRef.definition(r.id),
                ],
              };
        final base = <NodeRef, Offset>{
          for (final n in scene.nodes)
            if (moving.contains(n.ref)) n.ref: n.rect.topLeft,
        };
        setState(() {
          _gesture = _DragNodes(grabbed: node.ref, base: base, delta: e.delta / _zoom);
        });
      case HitGroup(:final group):
        widget.dispatch(SelectionChanged(GroupSelected(group.id)));
        final base = <NodeRef, Offset>{
          for (final n in scene.nodes)
            if (group.members.contains(n.ref.id) &&
                (n.ref.kind == NodeKind.mapping || n.ref.kind == NodeKind.definition))
              n.ref: n.rect.topLeft,
        };
        setState(() {
          _gesture = _DragGroup(group: group.id, base: base, delta: e.delta / _zoom);
        });
      case HitLink():
        // A link is not draggable; the press was a click that moved.
        setState(() => _gesture = const _Idle());
      case HitNothing():
        setState(() {
          _gesture = _Marquee(
            anchor: c.downScene,
            current: _toScene(e.localPosition),
            add: _primaryModifier,
            subtract: HardwareKeyboard.instance.isShiftPressed,
            base: _selectedSet,
          );
        });
    }
  }

  void _onPointerUp(PointerUpEvent e) {
    final g = _gesture;
    switch (g) {
      case _Idle():
        return;
      case _ConsumedByMenu():
        setState(() => _gesture = const _Idle());
        return;
      case _PressCandidate():
        setState(() => _gesture = const _Idle());
        if (g.pan || g.buttons & kPrimaryButton == 0) return;
        _click(g, e);
      case _Marquee():
        final result = _marqueeResult(g);
        final active = g.subtract
            ? (_active != null && result.contains(_active!) ? _active : null)
            : g.add
            ? _active
            : null;
        setState(() => _gesture = const _Idle());
        _setSelection(result, active: active);
      case _Pan():
        setState(() => _gesture = const _Idle());
        _viewportMoved();
      case _DragNodes():
        setState(() {
          _gesture = const _Idle();
          _dragOverGroup = null;
        });
        if (g.delta != Offset.zero) _commitNodeDrag(g);
      case _DragGroup():
        setState(() => _gesture = const _Idle());
        if (g.delta != Offset.zero) {
          widget.dispatch(NodesMoved({for (final b in g.base.entries) b.key: b.value + g.delta}));
        }
      case _DragLink():
        setState(() {
          _gesture = const _Idle();
          _hoverSocket = null;
        });
        _dropLink(_scene(widget.layout), g);
    }
  }

  /// A primary press that never moved: a click, or the second of a
  /// double-click.
  void _click(_PressCandidate c, PointerUpEvent e) {
    // The pointer's own clock, never the wall's: a test's taps are timed
    // by its binding, and a loaded machine does not turn a double-click
    // into two clicks.
    final now = e.timeStamp;
    final isDouble =
        _lastClickAt != null &&
        now - _lastClickAt! < kDoubleClickInterval &&
        _lastClickLocal != null &&
        (_lastClickLocal! - c.downLocal).distance < kDragThreshold;
    _lastClickAt = isDouble ? null : now;
    _lastClickLocal = c.downLocal;
    if (isDouble) {
      _doubleClick(c.hit);
      return;
    }
    final shift = HardwareKeyboard.instance.isShiftPressed;
    switch (c.hit) {
      case HitNode(:final node):
        if (_primaryModifier) {
          _toggle(node.ref);
        } else if (shift) {
          _chainSelect(node.ref);
        } else {
          _clickNode(node.ref);
        }
      case HitSocket(:final node):
        _primaryModifier ? _toggle(node.ref) : _clickNode(node.ref);
      case HitDisclosure(:final node):
        // the one control on a node: show or hide the saved formula; the
        // node is not selected by it (reading, not choosing)
        widget.dispatch(FormulaExpansionToggled(node.ref.id));
      case HitGroup(:final group):
        final ref = NodeRef.group(group.id);
        _primaryModifier
            ? _toggle(ref)
            : widget.dispatch(SelectionChanged(GroupSelected(group.id)));
      case HitLink(:final link):
        // A binding is its own object; any other edge is selected by its
        // ends (studio-ui §2): the selection persists, the inspector says
        // what the edge means, and the affordance appears on hover.
        _selectLink(link);
      case HitNothing():
        if (!_primaryModifier && !shift) _setSelection(const {});
    }
  }

  void _selectLink(LinkShape link) {
    final b = link.binding;
    final Selection next = b != null ? BindingSelected(b) : LinkSelected(link.id);
    if (widget.selection != next) widget.dispatch(SelectionChanged(next));
  }

  /// ⇧-click on a node: the displayed signature chain from the active
  /// object to it — taken only when there is exactly one such path over
  /// signature edges (concept ↔ relationship ↔ sink); a branched graph
  /// offers no chain and the click adds the node alone, never a guess.
  void _chainSelect(NodeRef target) {
    final anchor = _active;
    if (anchor == null || anchor == target) {
      _toggleOn(target);
      return;
    }
    final chain = uniqueSignatureChain(_scene(widget.layout), anchor, target);
    if (chain == null) {
      _toggleOn(target);
      return;
    }
    _setSelection({..._selectedSet, ...chain}, active: anchor);
  }

  void _toggleOn(NodeRef ref) => _setSelection({..._selectedSet, ref}, active: _active ?? ref);

  void _doubleClick(CanvasHit hit) {
    switch (hit) {
      case HitNode(:final node):
        if (node.ref.kind == NodeKind.instance) {
          _editSource(node.ref.id);
        } else if (node.ref.kind != NodeKind.output) {
          // a mapping block is renamed through its Sem block: one name
          widget.dispatch(InlineRenameStarted(asDeclaration(node.ref)));
        }
      case HitGroup(:final group):
        widget.dispatch(InlineRenameStarted(NodeRef.group(group.id)));
      default:
        break;
    }
  }

  /// Into the component's source.
  void _editSource(int instanceId) {
    final inst = widget.system.system?.instances
        .where((i) => i.id.toInt() == instanceId)
        .firstOrNull;
    if (inst != null) widget.dispatch(ContextChanged(ComponentContext(inst.component.toInt())));
  }

  void _onPointerCancel(PointerCancelEvent e) {
    if (_gesture is _Idle) return;
    setState(() {
      _gesture = const _Idle();
      _dragOverGroup = null;
    });
  }

  /// Esc: the innermost thing first — a menu, then a gesture, then the
  /// selection.  (A submenu's own Esc is the menu system's.)
  bool _escape() {
    if (_menuIsOpen) {
      _closeMenus();
      return true;
    }
    if (_gesture is! _Idle) {
      setState(() {
        _gesture = const _Idle();
        _dragOverGroup = null;
        _hoverSocket = null;
      });
      return true;
    }
    if (widget.selection is! NoSelection) {
      widget.dispatch(const SelectionChanged(NoSelection()));
      return true;
    }
    return false;
  }

  Map<NodeRef, Offset> get _effectiveLayout {
    switch (_gesture) {
      case _DragNodes(:final base, :final delta):
        return {...widget.layout, for (final b in base.entries) b.key: b.value + delta};
      case _DragGroup(:final base, :final delta):
        return {...widget.layout, for (final b in base.entries) b.key: b.value + delta};
      default:
        return widget.layout;
    }
  }

  void _updateDragOverGroup(NodeRef node) {
    // Insertion affordance: the expanded group under the dragged
    // relationship (its own group's region measured without it).  Only for
    // a single relationship — a Sem block with its mapping block is one —
    // a moved set keeps its memberships.
    if (!_groupsEnabled || node.kind != NodeKind.mapping) return;
    final g = _gesture;
    if (g is! _DragNodes || _oneDeclaration(g.base.keys) != node) return;
    final layout = _effectiveLayout;
    final shape = _scene(layout).nodes.where((n) => n.ref == node).firstOrNull;
    final without = buildScene(widget.project, layout, system: _sceneInputWithout(node.id));
    final over = shape == null ? null : groupAt(without, shape.rect.center);
    _dragOverGroup = over == null || (over.members.length == 1 && over.members.first == node.id)
        ? null
        : over.id;
  }

  void _commitNodeDrag(_DragNodes g) {
    final positions = {for (final b in g.base.entries) b.key: b.value + g.delta};
    if (positions.length == 1) {
      final e = positions.entries.single;
      widget.dispatch(NodeMoved(e.key, e.value));
    } else {
      widget.dispatch(NodesMoved(positions));
    }
    // Into or out of a group region: membership follows the drop of one
    // relationship (a Sem block, with or without its mapping block).  Only
    // the membership changes — a group is authoring metadata.
    final one = _oneDeclaration(positions.keys);
    if (_groupsEnabled &&
        one != null &&
        one.kind == NodeKind.mapping &&
        positions.containsKey(one)) {
      _membershipAfterDrop(one, {...widget.layout, ...positions});
    }
  }

  /// The one declaration a set of moved nodes is, when it is one: a Sem
  /// block alone or with its mapping block (ADR-0044); else none.
  static NodeRef? _oneDeclaration(Iterable<NodeRef> nodes) {
    final decls = {for (final n in nodes) asDeclaration(n)};
    return decls.length == 1 ? decls.single : null;
  }

  void _onPointerSignal(PointerSignalEvent e) {
    if (_menuIsOpen) return;
    if (e is PointerScrollEvent) {
      // A mouse wheel zooms about the pointer; a trackpad's two-finger
      // scroll pans, and zooms with ⌘/Ctrl held.
      final zoomIt =
          e.kind != PointerDeviceKind.trackpad ||
          HardwareKeyboard.instance.isMetaPressed ||
          HardwareKeyboard.instance.isControlPressed;
      if (zoomIt) {
        _zoomAbout(e.localPosition, e.scrollDelta.dy > 0 ? 0.9 : 1.1);
      } else {
        setState(() => _pan -= e.scrollDelta);
        _viewportMoved();
      }
    } else if (e is PointerScaleEvent) {
      _zoomAbout(e.localPosition, e.scale);
    }
  }

  void _onPanZoomUpdate(PointerPanZoomUpdateEvent e) {
    if (_menuIsOpen) return;
    // A trackpad gesture: pinch zooms about the fingers, the pan part pans.
    if (e.scale != 1) {
      _zoomAbout(e.localPosition, e.scale / (_lastScale ?? 1));
      _lastScale = e.scale;
    }
    if (e.panDelta != Offset.zero) setState(() => _pan += e.panDelta);
  }

  double? _lastScale;
  void _onPanZoomEnd(PointerPanZoomEndEvent e) {
    _lastScale = null;
    _viewportMoved();
  }

  void _zoomAbout(Offset local, double factor) {
    final before = _toScene(local);
    setState(() {
      _zoom = (_zoom * factor).clamp(0.25, 3.0);
      _pan = local - before * _zoom;
    });
    _viewportMoved();
  }

  void _onHover(PointerHoverEvent e) {
    // A menu owns the pointer: the nodes under it do not react.
    if (_menuIsOpen) return;
    // On the affordance (or just around it), the selected object stays
    // hovered: the pointer is on its way to an icon, not leaving.
    final onAffordance =
        _affordanceRect?.inflate(CanvasAffordance.hoverInflate).contains(e.localPosition) ?? false;
    if (onAffordance != _overAffordance) setState(() => _overAffordance = onAffordance);
    if (onAffordance) return;
    final scene = _scene(_effectiveLayout);
    final hit = hitTest(scene, _toScene(e.localPosition), linkHitTolerance: linkTolerance(_zoom));
    final (NodeRef? node, SocketRef? socket, LinkId? link) = switch (hit) {
      HitSocket(:final socket, :final node) => (node.ref, socket.ref, null),
      HitNode(:final node) || HitDisclosure(:final node) => (node.ref, null, null),
      HitGroup(:final group) => (NodeRef.group(group.id), null, null),
      HitLink(:final link) => (null, null, link.id),
      HitNothing() => (null, null, null),
    };
    if (node != _hoverNode || socket != _hoverSocket || link != _hoverLink) {
      setState(() {
        _hoverNode = node;
        _hoverSocket = socket;
        _hoverLink = link;
      });
    }
  }

  // ---- menus -----------------------------------------------------------------

  /// Open the contextual menu for what is under [local].  A right-click on
  /// one of several selected nodes keeps the selection (the menu is about
  /// all of them); on an unselected object it selects that object first.
  /// An open menu is closed and the new one opens after the rebuild that
  /// carries its context, so the items are never those of the last target.
  void _openContextMenuAt(Offset local) {
    final p = _toScene(local);
    final scene = _scene(widget.layout);
    final hit = hitTest(scene, p, linkHitTolerance: linkTolerance(_zoom));
    final MenuContext ctx;
    switch (hit) {
      case HitNode(:final node) || HitSocket(:final node) || HitDisclosure(:final node):
        // a mapping block's menu is its Sem block's: one declaration
        final decl = asDeclaration(node.ref);
        final set = _selectedSet;
        if (set.length > 1 && set.contains(decl)) {
          ctx = SelectionMenuContext(set, active: _active);
        } else {
          if (!set.contains(decl)) _setSelection({decl}, active: decl);
          ctx = NodeMenuContext(decl);
        }
      case HitGroup(:final group):
        widget.dispatch(SelectionChanged(GroupSelected(group.id)));
        ctx = GroupMenuContext(group.id);
      case HitLink(:final link):
        // The edge is selected first, as a node is: the menu is about
        // what the inspector then shows.
        _selectLink(link);
        ctx = LinkMenuContext(link);
      case HitNothing():
        // Blank canvas keeps the selection on show (nothing was clicked
        // away); the menu is about the canvas alone.
        ctx = CanvasMenuContext(p);
    }
    _openMenu(_MenuKind.context, ctx, local);
  }

  void _openMenu(_MenuKind kind, MenuContext? ctx, Offset local, {List<Widget>? chooser}) {
    // One overlay at a time: the other closes first.
    if (_menu.isOpen) _menu.close();
    if (_chooserMenu.isOpen) _chooserMenu.close();
    setState(() {
      _menuOpen = kind;
      _menuContext = ctx;
      _chooser = chooser ?? const [];
      _gesture = const _Idle();
      _hoverNode = null;
      _hoverSocket = null;
      _hoverLink = null;
      _overAffordance = false;
    });
    // The items are built from the context in the next frame; open then.
    // Until then the old overlay's own close (it closes itself on the
    // outside press that retargeted it) must not clear the new context.
    _reopenPending = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _reopenPending = false;
      if (!mounted || _menuOpen != kind) return;
      switch (kind) {
        case _MenuKind.context:
          _menu.open(position: local);
        case _MenuKind.chooser:
          _chooserMenu.open(position: local);
      }
    });
  }

  bool _reopenPending = false;

  void _closeMenus() {
    if (_menu.isOpen) _menu.close();
    if (_chooserMenu.isOpen) _chooserMenu.close();
    if (_menuOpen != null || _menuContext != null) {
      setState(() {
        _menuOpen = null;
        _menuContext = null;
        _chooser = const [];
      });
    }
  }

  /// The menu system closed itself (a command ran, Esc, a submenu's
  /// dismissal): the state follows.
  void _menuClosed(_MenuKind kind) {
    if (_reopenPending || _menuOpen != kind) return;
    setState(() {
      _menuOpen = null;
      if (kind == _MenuKind.context) _menuContext = null;
      if (kind == _MenuKind.chooser) _chooser = const [];
    });
  }

  // ---- menu contents -----------------------------------------------------------

  /// The contextual menu for [ctx] (docs/architecture/studio-ui.md §2,
  /// "Contextual menus"): stable groups — the object's primary command,
  /// IDE navigation, the service's fixes, structure, then the destructive
  /// command — each item present only when its command exists for that
  /// object.  Insertion lives on the empty-canvas menu alone.
  List<Widget> _menuItems(BuildContext context, MenuContext ctx) {
    final l10n = context.l10n;
    switch (ctx) {
      case CanvasMenuContext(:final scene):
        return _canvasMenu(context, scene);
      case NodeMenuContext(:final node):
        return _nodeMenu(context, node);
      case GroupMenuContext(:final group):
        return _groupMenu(context, group);
      case LinkMenuContext(:final link):
        return _linkMenu(context, link);
      case SelectionMenuContext(:final nodes):
        return _selectionMenu(context, nodes, l10n);
    }
  }

  pb.MappingView? _mapping(int id) =>
      widget.project.mappings.where((m) => m.id.toInt() == id).firstOrNull;
  String _conceptName(int id) =>
      widget.project.concepts.where((c) => c.id.toInt() == id).map((c) => c.name).firstOrNull ??
      '?';

  /// A node's name for a menu: a mapping block's is its Sem block's.
  String _nodeName(NodeRef ref) =>
      _scene(widget.layout).nodes
          .where((n) => n.ref == asDeclaration(ref))
          .map((n) => n.title)
          .firstOrNull ??
      '?';

  /// The service's actions for the object the menu is about — only when
  /// they are about it, at this revision (stale ones are not offered).
  List<pb.SemanticActionView> _fixesFor(NodeRef node) {
    final a = widget.actions;
    if (a == null || a.pending || a.revision != widget.project.revision.toInt()) return const [];
    final about = switch (node.kind) {
      NodeKind.concept => a.entity.hasConceptId() && a.entity.conceptId.toInt() == node.id,
      NodeKind.mapping ||
      NodeKind.definition => a.entity.hasMappingId() && a.entity.mappingId.toInt() == node.id,
      NodeKind.output => a.entity.hasOutputId() && a.entity.outputId.toInt() == node.id,
      _ => false,
    };
    return about ? a.actions : const [];
  }

  /// The *Fix* section: a ready action is a command; one that needs a
  /// choice is a submenu of the service's options; a blocked one is shown
  /// disabled with its reason, never clickable.
  List<Widget> _fixItems(BuildContext context, List<pb.SemanticActionView> fixes) {
    if (fixes.isEmpty) return const [];
    return [
      const MacMenuDivider(),
      MacSubmenu(
        label: context.l10n.fixMenu,
        children: [
          for (final x in fixes)
            switch (x.applicability) {
              pb.ActionApplicability.ACTION_APPLICABILITY_READY => MacMenuItem(
                label: x.title,
                onPressed: () => widget.dispatch(SemanticActionApplied(actionId: x.id)),
              ),
              pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE => MacSubmenu(
                label: x.title,
                children: [
                  for (var i = 0; i < x.options.length; i++)
                    MacMenuItem(
                      label: x.options[i].label,
                      onPressed: () =>
                          widget.dispatch(SemanticActionApplied(actionId: x.id, option: i)),
                    ),
                ],
              ),
              _ => MacMenuItem(label: x.title, detail: x.reason, onPressed: null),
            },
        ],
      ),
    ];
  }

  List<Widget> _navigationItems(BuildContext context, NodeRef node) => [
    if (widget.hasSources && node.kind != NodeKind.group)
      MacMenuItem(
        label: context.l10n.revealInCode,
        onPressed: () => widget.dispatch(RevealInCodeRequested(node)),
      ),
  ];

  List<Widget> _nodeMenu(BuildContext context, NodeRef node) {
    final l10n = context.l10n;
    final fixes = _fixItems(context, _fixesFor(node));
    switch (node.kind) {
      // a mapping block's menu is its Sem block's: one declaration
      case NodeKind.mapping || NodeKind.definition:
        final m = _mapping(node.id);
        final role = m == null ? RelationshipRole.value : relationshipRole(m);
        final groupOf = widget.groups
            .where((g) => g.members.any((x) => x.toInt() == node.id))
            .firstOrNull;
        final ported = widget.system.portWords.containsKey(node.id);
        return [
          // Primary: a rule or a value has a definition to edit; a Source
          // is provided by the environment and has none to open.
          if (role != RelationshipRole.source)
            MacMenuItem(
              label: l10n.editDefinition,
              onPressed: () => widget.dispatch(EditDefinitionRequested(node.id)),
            ),
          // reading: the saved formula on the node, folded or shown
          if (m != null && m.hasDefinition())
            MacMenuItem(
              label: widget.expanded.containsKey(node.id) ? l10n.hideFormula : l10n.showFormula,
              onPressed: () => widget.dispatch(FormulaExpansionToggled(node.id)),
            ),
          MacMenuItem(
            label: l10n.rename,
            onPressed: () => widget.dispatch(InlineRenameStarted(node)),
          ),
          ..._navigationItems(context, node),
          ...fixes,
          if (_groupsEnabled) ...[
            const MacMenuDivider(),
            if (groupOf == null) ...[
              MacMenuItem(
                label: l10n.groupAsBehavior,
                onPressed: () => widget.dispatch(
                  CreateGroupRequested(name: l10n.behavior, members: [node.id], renameAfter: true),
                ),
              ),
              if (widget.groups.isNotEmpty)
                MacSubmenu(
                  label: l10n.addToGroup,
                  children: [
                    for (final g in widget.groups)
                      MacMenuItem(
                        label: g.name,
                        onPressed: () => widget.dispatch(
                          AddGroupMemberRequested(group: g.id.toInt(), decl: node.id),
                        ),
                      ),
                  ],
                ),
            ] else
              MacMenuItem(
                label: l10n.removeFromGroup(groupOf.name),
                onPressed: () => widget.dispatch(
                  RemoveGroupMemberRequested(group: groupOf.id.toInt(), decl: node.id),
                ),
              ),
          ],
          const MacMenuDivider(),
          // A port-backed relationship of an open component is the port's:
          // it goes with the port, not with a delete here.
          if (!ported)
            MacMenuItem(
              label: l10n.deleteNamed(_nodeName(node)),
              destructive: true,
              onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
            ),
        ];
      case NodeKind.concept:
        return [
          MacMenuItem(
            label: l10n.rename,
            onPressed: () => widget.dispatch(InlineRenameStarted(node)),
          ),
          ..._navigationItems(context, node),
          ...fixes,
          const MacMenuDivider(),
          MacMenuItem(
            label: l10n.deleteNamed(_nodeName(node)),
            destructive: true,
            onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
          ),
        ];
      case NodeKind.output:
        final driver = widget.project.mappings
            .where((m) => m.hasDrivesOutputId() && m.drivesOutputId.toInt() == node.id)
            .toList();
        return [
          // The sink's driver is its one fact worth a jump: show it.
          for (final m in driver)
            MacMenuItem(
              label: l10n.showDriver(m.name),
              onPressed: () => _setSelection({NodeRef.mapping(m.id.toInt())}),
            ),
          MacMenuItem(
            label: l10n.rename,
            onPressed: () => widget.dispatch(InlineRenameStarted(node)),
          ),
          ..._navigationItems(context, node),
          ...fixes,
          const MacMenuDivider(),
          MacMenuItem(
            label: l10n.deleteNamed(_nodeName(node)),
            destructive: true,
            onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
          ),
        ];
      case NodeKind.instance:
        return [
          MacMenuItem(label: l10n.editSource, onPressed: () => _editSource(node.id)),
          MacMenuItem(
            label: l10n.rename,
            onPressed: () => widget.dispatch(InlineRenameStarted(node)),
          ),
          ..._navigationItems(context, node),
          const MacMenuDivider(),
          MacMenuItem(
            label: l10n.deleteNamed(_nodeName(node)),
            destructive: true,
            onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
          ),
        ];
      case NodeKind.group:
        // A collapsed box: the group's own commands.
        return _groupMenu(context, node.id);
    }
  }

  List<Widget> _groupMenu(BuildContext context, int group) {
    final l10n = context.l10n;
    final collapsed = widget.system.groupBoxes[group]?.collapsed ?? false;
    return [
      MacMenuItem(
        label: l10n.rename,
        onPressed: () => widget.dispatch(InlineRenameStarted(NodeRef.group(group))),
      ),
      MacMenuItem(
        label: collapsed ? l10n.expand : l10n.collapse,
        onPressed: () => _toggleCollapsed(group, collapsed),
      ),
      if (_isSystemCanvas)
        MacMenuItem(
          label: l10n.packageAsReusableComponent,
          onPressed: () => widget.dispatch(ExtractionSheetOpened(group)),
        ),
      const MacMenuDivider(),
      // Ungrouping keeps the relationships; the destructive delete is a
      // named action in the group's inspector.
      MacMenuItem(label: l10n.ungroup, onPressed: () => widget.dispatch(UngroupRequested(group))),
    ];
  }

  List<Widget> _linkMenu(BuildContext context, LinkShape link) {
    final l10n = context.l10n;
    final from = link.from.node;
    final to = link.to.node;
    return [
      if (link.binding case final b?)
        MacMenuItem(
          label: l10n.showBinding,
          onPressed: () => widget.dispatch(SelectionChanged(BindingSelected(b))),
        ),
      MacMenuItem(
        label: l10n.showEnd(_nodeName(from)),
        onPressed: () => _setSelection({from}, active: from),
      ),
      MacMenuItem(
        label: l10n.showEnd(_nodeName(to)),
        onPressed: () => _setSelection({to}, active: to),
      ),
      // Disconnect only where the model has a way to take this one edge
      // away (a binding; a relationship's read; a sink's driver).  A
      // relationship's produce edge and a collapsed group's edges have
      // none, and get no item — nothing greyed out, nothing implied.
      if (link.binding != null || link.id.disconnectable) ...[
        const MacMenuDivider(),
        MacMenuItem(
          label: l10n.disconnect,
          destructive: true,
          onPressed: () => _unlink(_scene(widget.layout), link.to, link: link),
        ),
      ],
    ];
  }

  List<Widget> _selectionMenu(BuildContext context, Set<NodeRef> nodes, AppLocalizations l10n) {
    final groupable = nodes
        .where((n) => n.kind == NodeKind.mapping)
        .where((n) => widget.groups.every((g) => g.members.every((x) => x.toInt() != n.id)))
        .length;
    return [
      if (_groupsEnabled && groupable > 0)
        MacMenuItem(
          label: l10n.groupAsBehaviorCount(groupable),
          onPressed: () => widget.dispatch(const GroupSelectionRequested()),
        ),
      if (_groupsEnabled && groupable > 0) const MacMenuDivider(),
      MacMenuItem(
        label: l10n.deleteObjects(nodes.length),
        destructive: true,
        onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
      ),
    ];
  }

  /// Empty canvas: creation, then the canvas itself.  The quick-insert
  /// tree is the library's — Recent, the value forms, the quantities in a
  /// submenu, then the Library tab for the rest.  Every category opens the
  /// concept sheet at the pointer: the concept is named there.
  List<Widget> _canvasMenu(BuildContext context, Offset at) {
    final l10n = context.l10n;
    final all = widget.templates;
    final byId = {for (final t in all) t.id: t};
    Widget item(pb.ConceptTemplateView t) => MacMenuItem(
      label: libraryItemStrings(l10n, t.id)?.name ?? t.displayName,
      onPressed: widget.canInsert ? () => _newConceptAt(t.id, at) : null,
    );
    List<Widget> group(Iterable<pb.ConceptTemplateView> ts) => [for (final t in ts) item(t)];
    Widget sourceItem(pb.LibraryItemView s) => MacMenuItem(
      label: itemName(l10n, s),
      onPressed: widget.canInsert ? () => _newSourceAt(s.id, at) : null,
    );
    final sourceById = {for (final s in widget.sources) s.id: s};
    final recent = [
      for (final id in widget.recentTemplates)
        if (byId[id] case final t?) item(t) else if (sourceById[id] case final s?) sourceItem(s),
    ];
    final forms = all.where((t) => t.category == 'form');
    final quantities = all.where((t) => t.category == 'quantity');
    // any other served library's categories, by their groups
    final otherGroups = <String>[];
    for (final t in all) {
      if (t.category != 'form' && t.category != 'quantity' && !otherGroups.contains(t.category)) {
        otherGroups.add(t.category);
      }
    }
    // The design's concepts, the templates a Sem block is created from
    // (ADR-0043): one block each per click, as many times as the product
    // has them.
    final conceptsOfDesign = [...widget.project.concepts]
      ..sort((a, b) => a.name.toLowerCase().compareTo(b.name.toLowerCase()));
    return [
      MacSubmenu(
        label: l10n.addBlock,
        children: [
          for (final c in conceptsOfDesign)
            MacMenuItem(
              label: l10n.blockOf(c.name),
              onPressed: widget.canInsert
                  ? () => widget.dispatch(AddBlockRequested(conceptId: c.id.toInt(), position: at))
                  : null,
            ),
          if (conceptsOfDesign.isNotEmpty) const MacMenuDivider(),
          // A new concept: the sheet names the template, and a block of it
          // lands where the pointer is.
          MacSubmenu(
            label: l10n.newConceptSubmenu,
            children: [
              if (recent.isNotEmpty) ...[
                MacSubmenu(label: l10n.recent, children: recent),
                const MacMenuDivider(),
              ],
              ...group(forms),
              if (quantities.isNotEmpty)
                MacSubmenu(label: l10n.libraryQuantities, children: group(quantities)),
              for (final g in otherGroups)
                MacSubmenu(
                  label: categoryLabel(l10n, g),
                  children: group(all.where((t) => t.category == g)),
                ),
              const MacMenuDivider(),
              MacMenuItem(
                label: l10n.more,
                onPressed: () => widget.dispatch(const SidebarTabSelected(SidebarTab.library)),
              ),
            ],
          ),
        ],
      ),
      // A Source is an ordinary relationship the environment provides
      // (ADR-0032), created over a concept the designer chooses on the
      // Source sheet: the generic entry, then any preset a served library
      // ships.
      MacSubmenu(
        label: l10n.addSource,
        children: [
          MacMenuItem(
            label: l10n.newSourceEllipsis,
            onPressed: widget.canInsert ? () => _newSourceAt('', at) : null,
          ),
          if (widget.sources.isNotEmpty) const MacMenuDivider(),
          ...widget.sources.map(sourceItem),
        ],
      ),
      if (_isSystemCanvas && widget.components.isNotEmpty)
        MacSubmenu(
          label: l10n.addInstance,
          children: [
            for (final c in widget.components)
              MacMenuItem(
                label: c.name,
                onPressed: () => widget.dispatch(
                  CreateInstanceRequested(
                    component: c.id.toInt(),
                    name: _freshInstanceName(c),
                    position: at,
                  ),
                ),
              ),
          ],
        ),
      if (_groupsEnabled)
        MacMenuItem(
          label: l10n.newBehaviorGroup,
          onPressed: () =>
              widget.dispatch(CreateGroupRequested(name: _freshGroupName(), renameAfter: true)),
        ),
      const MacMenuDivider(),
      MacMenuItem(label: l10n.selectAll, shortcut: shortcut('A'), onPressed: () => _selectAll()),
      MacMenuItem(label: l10n.frameAll, shortcut: shortcut('0'), onPressed: () => _frameAllNow()),
      const MacMenuDivider(),
      // Layout only (ADR-0023 §7): the service arranges every node again;
      // the layout from before is kept for one step back.
      MacMenuItem(
        label: l10n.arrangeAutomatically,
        onPressed: () => widget.dispatch(const AutoLayoutRequested()),
      ),
      MacMenuItem(
        label: l10n.undoArrange,
        onPressed: widget.canUndoArrange
            ? () => widget.dispatch(const RestoreLayoutRequested())
            : null,
      ),
    ];
  }

  /// `Behavior`, `Behavior 2`: a default name the designer renames inline.
  String _freshGroupName() {
    final taken = widget.groups.map((g) => g.name).toSet();
    if (!taken.contains(context.l10n.behavior)) return context.l10n.behavior;
    var i = 2;
    while (taken.contains('Behavior $i')) {
      i++;
    }
    return 'Behavior $i';
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

  // ---- links ---------------------------------------------------------------------

  /// The drop.  A hit on an aggregate socket resolves to the concrete
  /// endpoints it stands for: one → the link is made to it; several → a
  /// chooser names them (the member and its socket), and the choice makes
  /// the link.  Nothing is ever bound to the group.  A concept dropped on
  /// a sink is the authoring gesture: it resolves to the relationship that
  /// can drive the sink.
  void _dropLink(CanvasScene scene, _DragLink link) {
    final hit = hitTest(scene, link.current);
    if (hit is HitSocket && hit.socket.ref.role == SocketRole.aggregate) {
      final candidates = [
        for (final t in scene.resolve(hit.socket.ref))
          if (t.socket != null && canLink(link.from, t.socket!)) t,
      ];
      if (candidates.isEmpty) return;
      if (candidates.length == 1) {
        _linkToTarget(link.from, candidates.single);
        return;
      }
      _offerTargets(link.from, candidates, link.current);
      return;
    }
    // A Sem block dropped on a block as a whole: a Source gets it as its
    // definition; a mapping block with an open position gets it there.  A
    // text edit of the definition (ADR-0028), never a signature edit.
    if (blockDropTarget(scene, link.from, link.current) case final block?) {
      widget.dispatch(WireSemBlockRequested(mappingId: block.ref.id, semId: link.from.node.id));
      return;
    }
    final target = dropTarget(scene, link.from, link.current);
    if (target != null) {
      if (link.proxyChoices case final choices?) {
        // The drag started on an aggregate socket standing for several
        // sources: choose which one connects.
        final usable = [
          for (final t in choices)
            if (t.socket != null && canLink(t.socket!, target.ref)) t,
        ];
        if (usable.length == 1) {
          _makeLink(usable.single.socket!, target.ref);
        } else if (usable.length > 1) {
          _offerSources(usable, target.ref, link.current);
        }
        return;
      }
      _makeLink(link.from, target.ref);
    } else if (link.fromConnectedInput && hit is HitNothing) {
      _unlink(scene, link.from);
    }
  }

  void _linkToTarget(SocketRef from, ProxyTarget t) {
    if (t.socket case final to?) _makeLink(from, to);
  }

  void _offerTargets(SocketRef from, List<ProxyTarget> targets, Offset at) {
    _openMenu(
      _MenuKind.chooser,
      null,
      _toLocal(at),
      chooser: [
        for (final t in targets)
          if (t.socket case final socket?)
            MacMenuItem(
              label: t.label,
              detail: _socketWord(socket),
              onPressed: () => _linkToTarget(from, t),
            ),
      ],
    );
  }

  void _offerSources(List<ProxyTarget> sources, SocketRef to, Offset at) {
    _openMenu(
      _MenuKind.chooser,
      null,
      _toLocal(at),
      chooser: [
        for (final t in sources)
          MacMenuItem(label: t.label, onPressed: () => _makeLink(t.socket!, to)),
      ],
    );
  }

  String _socketWord(SocketRef s) {
    final name = _conceptName(s.concept);
    return switch (s.role) {
      SocketRole.realise => 'definition',
      SocketRole.slot => '?',
      _ =>
        s.side == SocketSide.input
            ? context.l10n.readsSocket(name)
            : context.l10n.producesSocket(name),
    };
  }

  void _membershipAfterDrop(NodeRef node, Map<NodeRef, Offset> layout) {
    final scene = _scene(layout);
    final shape = scene.nodes.where((n) => n.ref == node).firstOrNull;
    if (shape == null) return;
    final current = widget.groups
        .where((g) => g.members.any((m) => m.toInt() == node.id))
        .firstOrNull;
    // The region of the node's own group is measured without the node (and
    // its mapping block), so dragging out is possible.
    final sceneWithout = buildScene(widget.project, layout, system: _sceneInputWithout(node.id));
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
    // A Sem block into an open position of a mapping block: the text edit
    // that fills it (ADR-0043).
    if (inp.role == SocketRole.slot && out.node.kind == NodeKind.mapping) {
      widget.dispatch(
        WireSemBlockRequested(mappingId: inp.node.id, semId: out.node.id, slot: inp.index),
      );
      return;
    }
    if (out.node.kind == NodeKind.mapping && inp.node.kind == NodeKind.output) {
      widget.dispatch(SetMappingDriveRequested(mappingId: out.node.id, outputId: inp.node.id));
    }
  }

  /// Dragging a connected input away into empty space disconnects it: a
  /// mapping's read, or a sink's driver (the mapping stops driving).  From
  /// a link's menu, [link] names the one edge to undo.
  void _unlink(CanvasScene scene, SocketRef input, {LinkShape? link}) {
    // A bound port or realised relationship: the binding goes.
    if (input.role == SocketRole.port || input.role == SocketRole.realise) {
      for (final l in scene.links.where((l) => l.to == input && l.binding != null)) {
        if (link == null || l.binding == link.binding) widget.dispatch(UnbindRequested(l.binding!));
      }
      return;
    }
    // Every other edge goes through the one disconnect (the reducer
    // refuses what the model cannot take away alone: a read edge, which is
    // a name in a formula; a collapsed group's aggregate edge).
    for (final l in scene.links.where((l) => l.to == input)) {
      if (link == null || l.from == link.from) widget.dispatch(DisconnectLinkRequested(l.id));
    }
  }

  /// The contextual menu for the selection, from the keyboard (the menu
  /// key, ⇧F10): what the affordance's menu icon opens, at the same place.
  bool _openMenuForSelection() {
    final scene = _scene(widget.layout);
    final (MenuContext, Offset)? target = switch (widget.selection) {
      LinkSelected(:final link) => switch (scene.links.where((l) => l.id == link).firstOrNull) {
        final l? => (LinkMenuContext(l), _toLocal(l.midpoint)),
        null => null,
      },
      BindingSelected(:final id) => switch (scene.links.where((l) => l.binding == id).firstOrNull) {
        final l? => (LinkMenuContext(l), _toLocal(l.midpoint)),
        null => null,
      },
      GroupSelected(:final id) => switch (scene.groups.where((g) => g.id == id).firstOrNull) {
        final g? => (GroupMenuContext(id), _toLocal(g.titleBand.center)),
        null => null,
      },
      MultiSelected(:final nodes) => (
        SelectionMenuContext(nodes, active: _active),
        _toLocal(scene.bounds.center),
      ),
      _ => switch (_selectedSet.singleOrNull) {
        final ref? => switch (scene.nodes.where((n) => n.ref == ref).firstOrNull) {
          final n? => (NodeMenuContext(ref), _toLocal(n.header.center)),
          null => null,
        },
        null => null,
      },
    };
    if (target == null) return false;
    _openMenu(_MenuKind.context, target.$1, target.$2);
    return true;
  }

  // ---- affordance ----------------------------------------------------------------

  /// The affordance of the selected object while it is hovered
  /// (studio-ui §2): the object's menu, then its own quick actions — an
  /// edge's disconnect, a node's delete, a group's collapse — only those
  /// its menu has.  Nothing while a menu is open, a gesture is under way
  /// or a name is being edited.  One object at a time; a set has none.
  CanvasAffordance? _affordance(BuildContext context, CanvasScene scene) {
    if (_menuIsOpen || widget.renaming != null) return null;
    if (_gesture is! _Idle && _gesture is! _PressCandidate) return null;
    final l10n = context.l10n;
    switch (widget.selection) {
      case LinkSelected(:final link):
        final shape = scene.links.where((l) => l.id == link).firstOrNull;
        if (shape == null || !(_hoverLink == link || _overAffordance)) return null;
        return CanvasAffordance(
          key: ValueKey(link),
          owner: link,
          anchor: _toLocal(shape.midpoint),
          actions: [
            AffordanceAction(
              icon: Icons.more_horiz,
              label: l10n.connectionMenu,
              onPressed: () => _openMenuFromAffordance(LinkMenuContext(shape)),
            ),
            if (link.disconnectable)
              AffordanceAction(
                icon: Icons.close,
                label: l10n.disconnect,
                destructive: true,
                onPressed: () => widget.dispatch(DisconnectLinkRequested(link)),
              ),
          ],
        );
      case BindingSelected(:final id):
        final shape = scene.links.where((l) => l.binding == id).firstOrNull;
        if (shape == null || !(_hoverLink == shape.id || _overAffordance)) return null;
        return CanvasAffordance(
          key: ValueKey('binding$id'),
          owner: shape.id,
          anchor: _toLocal(shape.midpoint),
          actions: [
            AffordanceAction(
              icon: Icons.more_horiz,
              label: l10n.connectionMenu,
              onPressed: () => _openMenuFromAffordance(LinkMenuContext(shape)),
            ),
            AffordanceAction(
              icon: Icons.close,
              label: l10n.disconnect,
              destructive: true,
              onPressed: () => widget.dispatch(UnbindRequested(id)),
            ),
          ],
        );
      case GroupSelected(:final id):
        final ref = NodeRef.group(id);
        if (!(_hoverNode == ref || _overAffordance)) return null;
        final region = scene.groups.where((g) => g.id == id).firstOrNull;
        final box = scene.nodes.where((n) => n.ref == ref).firstOrNull;
        final rect = region?.rect ?? box?.rect;
        if (rect == null) return null;
        final collapsed = widget.system.groupBoxes[id]?.collapsed ?? false;
        return CanvasAffordance(
          key: ValueKey(ref),
          owner: ref,
          anchor: _toLocal(rect.topRight),
          alignment: AffordanceAlignment.endAbove,
          actions: [
            AffordanceAction(
              icon: Icons.more_horiz,
              label: l10n.groupMenu,
              onPressed: () => _openMenuFromAffordance(GroupMenuContext(id)),
            ),
            AffordanceAction(
              icon: collapsed ? Icons.unfold_more : Icons.unfold_less,
              label: collapsed ? l10n.expand : l10n.collapse,
              onPressed: () => _toggleCollapsed(id, collapsed),
            ),
          ],
        );
      case MultiSelected():
        return null;
      default:
        final ref = _selectedSet.singleOrNull;
        if (ref == null || ref.kind == NodeKind.group) return null;
        // the affordance shows over the Sem block while either of its
        // nodes is hovered
        final block = ref.kind == NodeKind.mapping ? NodeRef.definition(ref.id) : null;
        if (!(_hoverNode == ref || _hoverNode == block || _overAffordance)) return null;
        final shape = scene.nodes.where((n) => n.ref == ref).firstOrNull;
        if (shape == null) return null;
        // A ported relationship is part of the component's promise: its
        // menu has no delete, and neither does the affordance.
        final deletable =
            ref.kind != NodeKind.mapping || !widget.system.portWords.containsKey(ref.id);
        return CanvasAffordance(
          key: ValueKey(ref),
          owner: ref,
          anchor: _toLocal(shape.rect.topRight),
          alignment: AffordanceAlignment.endAbove,
          actions: [
            AffordanceAction(
              icon: Icons.more_horiz,
              label: l10n.nodeMenu,
              onPressed: () => _openMenuFromAffordance(NodeMenuContext(ref)),
            ),
            if (deletable)
              AffordanceAction(
                icon: Icons.close,
                label: l10n.deleteNamed(shape.title),
                destructive: true,
                onPressed: () => widget.dispatch(const DeleteSelectionRequested()),
              ),
          ],
        );
    }
  }

  /// The menu icon: the same contextual menu the right-click opens, under
  /// the affordance.
  void _openMenuFromAffordance(MenuContext ctx) {
    final r = _affordanceRect;
    _openMenu(_MenuKind.context, ctx, r == null ? Offset.zero : r.bottomLeft);
  }

  void _toggleCollapsed(int group, bool collapsed) {
    // Collapsing: the box starts where the region was.
    if (!collapsed) {
      final region = _scene(widget.layout).groups.where((g) => g.id == group).firstOrNull;
      if (region != null) widget.dispatch(GroupBoxChanged(id: group, rect: region.rect));
    }
    widget.dispatch(GroupCollapsedChanged(id: group, collapsed: !collapsed));
  }

  // ---- keyboard ------------------------------------------------------------------

  void _selectAll() => _setSelection(_eligibleNodes, active: _active);

  Size _viewportSize = Size.zero;
  void _frameAllNow() => _frameAll(_viewportSize);

  void _frameAll(Size viewport) {
    if (viewport.isEmpty) return;
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
    _viewportMoved();
  }

  /// Arrow keys nudge the selected set by the grid (⇧: one point).
  void _nudge(Offset by) {
    final set = _selectedShapes;
    if (set.isEmpty) return;
    final scene = _scene(widget.layout);
    final positions = {
      for (final n in scene.nodes)
        if (set.contains(n.ref)) n.ref: n.rect.topLeft + by,
    };
    if (positions.isEmpty) return;
    if (positions.length == 1) {
      final e = positions.entries.single;
      widget.dispatch(NodeMoved(e.key, e.value));
    } else {
      widget.dispatch(NodesMoved(positions));
    }
  }

  KeyEventResult _onKey(FocusNode node, KeyEvent event) {
    // Only the canvas itself: a field inside it (the inline rename) owns
    // its own keys, and a shortcut never fires through a text field.
    if (!_focus.hasPrimaryFocus) return KeyEventResult.ignored;
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.space) {
      // Held: the next primary drag pans.
      final held = event is! KeyUpEvent;
      if (held != _spaceHeld) setState(() => _spaceHeld = held);
      return KeyEventResult.handled;
    }
    if (event is! KeyDownEvent) return KeyEventResult.ignored;
    if (key == LogicalKeyboardKey.escape) {
      return _escape() ? KeyEventResult.handled : KeyEventResult.ignored;
    }
    if (_menuIsOpen) return KeyEventResult.ignored;
    if (key == LogicalKeyboardKey.backspace || key == LogicalKeyboardKey.delete) {
      widget.dispatch(const DeleteSelectionRequested());
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.contextMenu ||
        (key == LogicalKeyboardKey.f10 && HardwareKeyboard.instance.isShiftPressed)) {
      return _openMenuForSelection() ? KeyEventResult.handled : KeyEventResult.ignored;
    }
    if (key == LogicalKeyboardKey.home || (key == LogicalKeyboardKey.digit0 && _primaryModifier)) {
      _frameAllNow();
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.keyA && _primaryModifier) {
      _selectAll();
      return KeyEventResult.handled;
    }
    final step = HardwareKeyboard.instance.isShiftPressed ? 1.0 : MacTokens.gridStep;
    final nudge = switch (key) {
      LogicalKeyboardKey.arrowLeft => Offset(-step, 0),
      LogicalKeyboardKey.arrowRight => Offset(step, 0),
      LogicalKeyboardKey.arrowUp => Offset(0, -step),
      LogicalKeyboardKey.arrowDown => Offset(0, step),
      _ => null,
    };
    if (nudge != null) {
      _nudge(nudge);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  MouseCursor get _cursor {
    if (_menuIsOpen) return SystemMouseCursors.basic;
    switch (_gesture) {
      case _Pan():
        return SystemMouseCursors.grabbing;
      case _DragNodes() || _DragGroup():
        return SystemMouseCursors.move;
      case _DragLink(:final from):
        // Over a socket the link cannot reach, the pointer says so before
        // the drop: the typing rule is refused, not diagnosed.
        final s = _hoverSocket;
        if (s == null) return SystemMouseCursors.precise;
        return canLink(from, s) ? SystemMouseCursors.precise : SystemMouseCursors.forbidden;
      case _Marquee():
        return SystemMouseCursors.precise;
      default:
        if (_spaceHeld) return SystemMouseCursors.grab;
        if (_hoverSocket != null) return SystemMouseCursors.precise;
        // An edge is something to click: the pointer says so before
        // anything is drawn on it.
        if (_hoverLink != null && !_overAffordance) return SystemMouseCursors.click;
        return SystemMouseCursors.basic;
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final scene = _scene(_effectiveLayout);
    final selectedSet = _selectedShapes;
    final active = _active;
    final selectedBinding = switch (widget.selection) {
      BindingSelected(:final id) => id,
      _ => null,
    };
    final selectedLink = switch (widget.selection) {
      LinkSelected(:final link) => link,
      _ => null,
    };
    final affordance = _affordance(context, scene);
    _affordanceRect = affordance?.rect;
    final g = _gesture;
    final marquee = g is _Marquee ? g : null;
    final preview = marquee == null ? null : _marqueeResult(marquee);
    final linkDrag = g is _DragLink ? g : null;
    final menuItems = _menuContext == null ? const <Widget>[] : _menuItems(context, _menuContext!);

    return LayoutBuilder(
      builder: (context, constraints) {
        _viewportSize = constraints.biggest;
        return Focus(
          focusNode: _focus,
          onKeyEvent: _onKey,
          child: DragTarget<LibraryItemDrag>(
            onWillAcceptWithDetails: (_) => widget.canInsert,
            onAcceptWithDetails: (d) {
              final box = context.findRenderObject() as RenderBox?;
              if (box == null) return;
              final scene = _toScene(box.globalToLocal(d.offset));
              if (d.data.source) {
                _newSourceAt(d.data.itemId, scene);
              } else {
                _newConceptAt(d.data.itemId, scene);
              }
            },
            builder: (context, candidates, _) => MacMenuAnchor(
              controller: _chooserMenu,
              items: _chooser,
              onClose: () => _menuClosed(_MenuKind.chooser),
              child: MacMenuAnchor(
                controller: _menu,
                items: menuItems,
                onClose: () => _menuClosed(_MenuKind.context),
                child: Listener(
                  behavior: HitTestBehavior.opaque,
                  onPointerDown: _onPointerDown,
                  onPointerMove: _onPointerMove,
                  onPointerUp: _onPointerUp,
                  onPointerCancel: _onPointerCancel,
                  onPointerSignal: _onPointerSignal,
                  onPointerHover: _onHover,
                  onPointerPanZoomUpdate: _onPanZoomUpdate,
                  onPointerPanZoomEnd: _onPanZoomEnd,
                  child: MouseRegion(
                    cursor: _cursor,
                    onExit: (_) => setState(() {
                      _hoverNode = null;
                      _hoverLink = null;
                      _overAffordance = false;
                      if (g is! _DragLink) _hoverSocket = null;
                    }),
                    child: ClipRect(
                      child: Stack(
                        children: [
                          CustomPaint(
                            painter: _CanvasPainter(
                              scene: scene,
                              tokens: t,
                              l10n: context.l10n,
                              pan: _pan,
                              zoom: _zoom,
                              selectedSet: selectedSet,
                              active: _selectedSet.length > 1 ? active : null,
                              selectedBinding: selectedBinding,
                              selectedLink: selectedLink,
                              hoveredLink: _hoverLink,
                              marquee: marquee?.rect,
                              marqueeMode: marquee?.mode,
                              marqueeAdd: marquee?.add ?? false,
                              marqueeSubtract: marquee?.subtract ?? false,
                              preview: preview,
                              dragOverGroup: _dragOverGroup,
                              hovered: _hoverNode,
                              hoveredSocket: _hoverSocket,
                              linkDrag: linkDrag,
                              dropOk: linkDrag == null
                                  ? null
                                  : dropTarget(scene, linkDrag.from, linkDrag.current)?.ref,
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
                          // Expanded formulas: the saved definition rendered
                          // on the node, read only (a deliberate action edits
                          // it in the inspector).
                          for (final shape in scene.nodes)
                            if (shape.expanded && shape.ref.kind == NodeKind.definition)
                              ExpandedFormula(
                                key: ValueKey('expanded-${shape.ref.id}'),
                                mappingId: shape.ref.id,
                                rect: Rect.fromLTWH(
                                  shape.formulaRegion.left * _zoom + _pan.dx,
                                  shape.formulaRegion.top * _zoom + _pan.dy,
                                  shape.formulaRegion.width * _zoom,
                                  shape.formulaRegion.height * _zoom,
                                ),
                                sceneWidth: shape.formulaRegion.width,
                                zoom: _zoom,
                                preview: widget.previews[shape.ref.id],
                                analysis: widget.analyses[shape.ref.id],
                                concepts: {
                                  for (final c in widget.project.concepts) c.id.toInt(): c,
                                },
                                selected: selectedSet.contains(shape.ref),
                                dispatch: widget.dispatch,
                              ),
                          // The selected object's affordance, over the
                          // picture, at its anchor (studio-ui §2).
                          ?affordance,
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
                            for (final gr in scene.groups.where(
                              (gr) => NodeRef.group(gr.id) == node,
                            ))
                              _InlineRename(
                                key: ValueKey(node),
                                rect: Rect.fromLTWH(
                                  gr.rect.left * _zoom + _pan.dx,
                                  gr.rect.top * _zoom + _pan.dy,
                                  (gr.rect.width / 2).clamp(120, 320) * _zoom,
                                  NodeMetrics.regionTitle * _zoom,
                                ),
                                zoom: _zoom,
                                initial: gr.title,
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
            fontSize: MacType.nodeTitle * widget.zoom,
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
    required this.l10n,
    required this.pan,
    required this.zoom,
    required this.selectedSet,
    required this.active,
    required this.selectedBinding,
    required this.selectedLink,
    required this.hoveredLink,
    required this.hovered,
    required this.hoveredSocket,
    required this.linkDrag,
    required this.dropOk,
    this.marquee,
    this.marqueeMode,
    this.marqueeAdd = false,
    this.marqueeSubtract = false,
    this.preview,
    this.dragOverGroup,
  });

  final CanvasScene scene;
  final MacTokens tokens;
  final AppLocalizations l10n;
  final Offset pan;
  final double zoom;

  /// The nodes the selection is drawn on and, among several selected
  /// declarations, the active one (none while one alone is selected).
  final Set<NodeRef> selectedSet;
  final NodeRef? active;
  final int? selectedBinding;

  /// The selected edge and the hovered one (by their ends).
  final LinkId? selectedLink;
  final LinkId? hoveredLink;

  /// A marquee in progress: its rectangle, its mode (window: solid;
  /// crossing: dashed), whether it adds to or subtracts from the
  /// selection, and the selection it would produce ([preview]) — shown
  /// before the pointer comes up, so the gesture means what it shows.
  final Rect? marquee;
  final MarqueeMode? marqueeMode;
  final bool marqueeAdd;
  final bool marqueeSubtract;
  final Set<NodeRef>? preview;
  final int? dragOverGroup;
  final NodeRef? hovered;
  final SocketRef? hoveredSocket;
  final _DragLink? linkDrag;
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
      l10n: l10n,
      hoveredSocket: hoveredSocket,
      dropOk: dropOk,
      compatible: linkDrag == null ? const {} : compatibleSockets(scene, linkDrag!.from),
    );
    for (final g in scene.groups) {
      painter.region(
        canvas,
        g,
        selected: selectedSet.contains(NodeRef.group(g.id)),
        hovered: hovered == NodeRef.group(g.id),
        receiving: dragOverGroup == g.id,
      );
    }

    // Every edge in the hue of the concept it carries (ADR-0043: a read
    // edge carries the read Sem block's; a drive edge the driver's).
    for (final l in scene.links) {
      final isSelected =
          (l.binding != null && l.binding == selectedBinding) ||
          (l.binding == null && selectedLink != null && l.id == selectedLink);
      final isHovered = !isSelected && hoveredLink != null && l.id == hoveredLink;
      // Hovered: a soft halo under the same stroke — "this can be
      // clicked", nothing more.  Selected: the accent, the heavier stroke
      // and a ring at each end, so the state is told by shape as well as
      // by colour (a binding's selection has looked like this all along).
      if (isHovered || isSelected) {
        canvas.drawPath(
          l.path,
          Paint()
            ..color = (isSelected ? tokens.accent : tokens.conceptColor(l.concept)).withValues(
              alpha: isSelected ? 0.22 : 0.28,
            )
            ..style = PaintingStyle.stroke
            ..strokeWidth = 8
            ..strokeCap = StrokeCap.round,
        );
      }
      canvas.drawPath(
        l.path,
        Paint()
          ..color = isSelected ? tokens.accent : tokens.conceptColor(l.concept)
          ..style = PaintingStyle.stroke
          ..strokeWidth = isSelected ? 3 : 2
          ..strokeCap = StrokeCap.round,
      );
      if (isSelected) {
        for (final end in _pathEnds(l.path)) {
          canvas.drawCircle(end, 4, Paint()..color = tokens.canvas);
          canvas.drawCircle(
            end,
            4,
            Paint()
              ..color = tokens.accent
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2,
          );
        }
      }
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
    // While a marquee is drawn, a node's outline says what the release
    // would do to it: taken (accent), let go (secondary), or unchanged.
    final p = preview;
    for (final n in scene.nodes) {
      final isSelected = selectedSet.contains(n.ref);
      final taken = p != null && (p.contains(n.ref) || p.contains(asDeclaration(n.ref)));
      final MarqueePreview? pv = p == null
          ? null
          : taken && !isSelected
          ? MarqueePreview.take
          : !taken && isSelected
          ? MarqueePreview.release
          : null;
      painter.node(
        canvas,
        n,
        selected: isSelected,
        active: n.ref == active,
        hovered: n.ref == hovered,
        preview: pv,
      );
    }
    if (marquee case final m?) {
      // Window: a solid outline and a restrained fill.  Crossing: a dashed
      // outline and a fainter fill — told apart by the line, not the hue.
      final crossing = marqueeMode == MarqueeMode.crossing;
      canvas.drawRect(m, Paint()..color = tokens.accent.withValues(alpha: crossing ? 0.05 : 0.1));
      final stroke = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1 / zoom
        ..color = tokens.accent;
      if (crossing) {
        painter._dashedRect(canvas, m, stroke, dash: 6 / zoom);
      } else {
        canvas.drawRect(m, stroke);
      }
      // Adding to or subtracting from the selection: the sign at the
      // marquee's moving corner, beside the pointer.
      if (marqueeAdd || marqueeSubtract) {
        final corner = m.bottomRight + Offset(6 / zoom, 2 / zoom);
        painter._text(
          canvas,
          marqueeSubtract ? '−' : '+',
          corner,
          FontWeight.w600,
          MacType.body / zoom,
          tokens.accent,
        );
      }
    }
    canvas.restore();

    // An empty design names its first step (the canvas is the entry point).
    if (scene.nodes.isEmpty) {
      final tp = TextPainter(
        text: TextSpan(
          text: l10n.addAConceptFromTheLibraryTo,
          style: TextStyle(
            fontFamily: '.AppleSystemUIFont',
            fontSize: MacType.body,
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
            selected: selectedSet.contains(n.ref),
            button: true,
          ),
        ),
    ];
  };

  String _describe(NodeShape n) {
    switch (n.ref.kind) {
      case NodeKind.concept:
        // not drawn (ADR-0043: a concept is a template, not a node)
        return '${n.title}, concept';
      case NodeKind.mapping:
        final produces = n.sockets
            .where((s) => s.ref.side == SocketSide.output)
            .map((s) => n.socketLabels[s.ref] ?? '')
            .join(', ');
        // A Source is named as what it is, not as a relationship missing
        // its reads: the environment provides what it produces.
        if (n.source) return l10n.sourceNodeSemantics(n.title, produces);
        final state = n.wrong ? l10n.definitionDoesNotCheck : l10n.stateDefined;
        // A Sem block with its mapping block: what the block reads is its
        // read sockets, what it applies is a word.  A port-backed
        // relationship of an open component is named by its port, the
        // word the header wears.
        final shape = n.headerWord.isNotEmpty ? n.headerWord : l10n.valueNodeSemantics;
        final depends = n.dependsOn.isEmpty
            ? ''
            : ', ${l10n.dependsOnList(n.dependsOn.join(', '))}';
        final applies = n.applies.isEmpty ? '' : ', ${l10n.appliesList(n.applies.join(', '))}';
        return '${n.title}, $shape, produces $produces$depends$applies, $state';
      case NodeKind.definition:
        // the mapping block: the definition of its Sem block, what it
        // reads and applies, and whether it checks
        final state = n.wrong ? l10n.definitionDoesNotCheck : l10n.stateDefined;
        final depends = n.dependsOn.isEmpty
            ? ''
            : ', ${l10n.dependsOnList(n.dependsOn.join(', '))}';
        final applies = n.applies.isEmpty ? '' : ', ${l10n.appliesList(n.applies.join(', '))}';
        return '${l10n.mappingBlockSemantics(n.subtitle)}$depends$applies, $state';
      case NodeKind.output:
        final accepts = n.socketLabels.values.join(', ');
        final state = switch (n.sink) {
          SinkState.open => l10n.noTimingDomainYet,
          SinkState.undriven => l10n.stateUndriven,
          SinkState.driven => l10n.stateDriven,
          SinkState.illFormed => l10n.drivenByAnIllFormedConnection,
          SinkState.contested => l10n.contestedBySeveralDrivers,
          null => '',
        };
        return '${n.title}, physical output, accepts $accepts, '
            '${n.required ? l10n.stateRequired : l10n.stateOptional}, $state';
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

  /// Where a path starts and ends.
  static List<Offset> _pathEnds(Path path) {
    final metrics = path.computeMetrics().toList();
    if (metrics.isEmpty) return const [];
    final first = metrics.first.getTangentForOffset(0)?.position;
    final last = metrics.last.getTangentForOffset(metrics.last.length)?.position;
    return [?first, ?last];
  }

  @override
  bool shouldRepaint(_CanvasPainter old) => true;
}

/// Paints one node the way the canvas does — shared with previews, chips
/// and library rows so a concept looks the same wherever it appears.
///
/// What the geometry means (docs/architecture/studio-ui.md §7): socket hue = identity, socket
/// shape = value form (○ quantity, ◇ on–off, □ count, hollow ring while
/// undecided), dashed outline = declared-not-defined, a red mark at the
/// definition line = the definition does not check.  No other state is
/// written on the node.
/// What a marquee in progress would do to a node on release.
enum MarqueePreview { take, release }

class NodePainter {
  NodePainter(
    this.tokens, {
    AppLocalizations? l10n,
    this.hoveredSocket,
    this.dropOk,
    this.compatible = const {},
    Color Function(int)? conceptColor,
  }) : conceptColor = conceptColor ?? tokens.conceptColor,
       l10n = l10n ?? kEnglish;
  final MacTokens tokens;

  /// The words drawn on nodes (a declared mapping's *declared*, a sink's
  /// *contested*); English when no locale is in scope (previews, tests).
  final AppLocalizations l10n;
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

  /// [active]: the active object of a multi-selection wears a second,
  /// outer accent ring (the platform's focus-ring idiom) — a shape, not a
  /// hue, so it reads beside the others' single outline.  [preview]: what
  /// a marquee in progress would do to the node.
  void node(
    Canvas canvas,
    NodeShape n, {
    bool selected = false,
    bool active = false,
    bool hovered = false,
    MarqueePreview? preview,
  }) {
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
      // A Source: the environment's side of the model — a cooler, greener
      // strip than a relationship's, paired below with the boundary bar
      // and the entry glyph so the category survives without colour.
      NodeKind.mapping when n.source =>
        tokens.isDark ? const Color(0xFF2F5A4A) : const Color(0xFFD2ECDD),
      NodeKind.mapping => tokens.isDark ? const Color(0xFF2E4A6B) : const Color(0xFFCFE0F5),
      // A mapping block: the definition's strip, a shade quieter than the
      // Sem block's it joins — the same family, the dependent object.
      NodeKind.definition => tokens.isDark ? const Color(0xFF2A3D55) : const Color(0xFFDDE8F7),
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
          : tokens.hairline;
    // An undriven or open sink is incomplete, not wrong: dashed.
    final dashed = n.sink == SinkState.open || n.sink == SinkState.undriven;
    if (dashed) {
      _dashedRRect(canvas, rrect, outline..color = selected ? tokens.accent : tokens.textTertiary);
    } else {
      canvas.drawRRect(rrect, outline);
    }
    if (active) {
      canvas.drawRRect(
        rrect.inflate(3),
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = tokens.accent,
      );
    }
    if (preview != null) {
      final ring = Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = preview == MarqueePreview.take
            ? tokens.accent.withValues(alpha: 0.6)
            : tokens.textSecondary;
      if (preview == MarqueePreview.release) {
        _dashedRRect(canvas, rrect.inflate(2), ring);
      } else {
        canvas.drawRRect(rrect.inflate(2), ring);
      }
    }
    // The environment boundary: a solid bar on a Source's left edge — to the
    // left of it is the environment, which provides the value; nothing in
    // the model feeds it.  The mirror of the sink's bar on the right, drawn
    // the same way; the two are opposite boundaries, never symmetric types.
    if (n.source) {
      canvas.drawLine(
        n.rect.topLeft + const Offset(1, NodeMetrics.cornerRadius),
        n.rect.bottomLeft + const Offset(1, -NodeMetrics.cornerRadius),
        Paint()
          ..color = selected ? tokens.accent : tokens.textSecondary
          ..strokeWidth = 3
          ..strokeCap = StrokeCap.round,
      );
      // The entry glyph: a value coming in from the left — an arrow into
      // the header, meaning "enters the behavior model", not a sensor.
      _entryGlyph(canvas, n.header.topLeft + const Offset(9, 8), tokens.textSecondary);
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

    // A mapping block's header names the rules it applies; one applying
    // none is a formula of its own and says so, quietly.
    final plainFormula = n.ref.kind == NodeKind.definition && n.title.isEmpty;
    _text(
      canvas,
      plainFormula ? l10n.formula : n.title,
      n.header.topLeft + Offset(n.source ? 26 : 12, 6),
      plainFormula ? FontWeight.w400 : FontWeight.w600,
      12.5,
      plainFormula ? tokens.textSecondary : tokens.textPrimary,
      maxWidth: n.rect.width - (n.source ? 78 : 24) - (n.source ? 14 : 0),
    );
    // The header's right word is object state in words only where the
    // geometry cannot carry it: a declared mapping, an open or contested
    // sink, a required sink, a port-backed relationship in a component's
    // source.
    // A defined rule that carries no other word says *rule*: it is applied
    // by a value's formula and has no value of its own (ADR-0034); the
    // input sockets are the shape, the word the consequence.  A rule
    // nothing applies says *not applied* instead — the state before the
    // shape, as *declared* comes before both — and its hollow output socket
    // says no value comes out of it.
    final headerWord = n.source
        ? l10n.roleSource
        : n.headerWord.isNotEmpty
        ? n.headerWord
        : switch (n.sink) {
            SinkState.open => l10n.noDomain,
            SinkState.contested => l10n.stateContested,
            SinkState.illFormed => l10n.stateIllFormed,
            _ => n.required ? l10n.stateRequired : '',
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
    // The timing domain, quietly, at the body's right edge — left of the
    // disclosure when the definition line carries one.
    // A Sem block's domain is the declaration's: at the left of its
    // concept row, unless a realisation's word is there already.
    final semRowFree =
        n.ref.kind == NodeKind.mapping &&
        n.sockets.every(
          (s) => s.ref.side == SocketSide.output || !n.socketLabels.containsKey(s.ref),
        );
    if (n.timing.isNotEmpty && (n.ref.kind != NodeKind.mapping || semRowFree)) {
      final sem = n.ref.kind == NodeKind.mapping;
      final at = n.ref.kind == NodeKind.definition
          ? n.definitionRegion.topRight + Offset(n.definition != null ? -28 : -10, 5)
          : sem
          ? n.rect.bottomLeft + const Offset(12, -NodeMetrics.rowHeight + 5)
          : n.rect.bottomRight + const Offset(-12, -NodeMetrics.rowHeight + 5);
      _text(
        canvas,
        '↻ ${n.timing}',
        at,
        FontWeight.w400,
        10,
        tokens.textTertiary,
        alignRight: !sem,
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

    // Definition region of a mapping block: the summary line.  A
    // definition that does not check gets the red mark here — where the
    // problem lives — and nowhere else on the node.
    if (n.ref.kind == NodeKind.definition) {
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
          maxWidth: region.right - 10 - left - 18 - (n.timing.isEmpty ? 0 : 64),
        );
        // the disclosure: a chevron at the line's right end — right while
        // the formula is folded, down while it is shown on the node
        final d = n.disclosure;
        final c = d.center;
        final chevron = Path();
        if (n.expanded) {
          chevron
            ..moveTo(c.dx - 3.5, c.dy - 2)
            ..lineTo(c.dx, c.dy + 2)
            ..lineTo(c.dx + 3.5, c.dy - 2);
        } else {
          chevron
            ..moveTo(c.dx - 2, c.dy - 3.5)
            ..lineTo(c.dx + 2, c.dy)
            ..lineTo(c.dx - 2, c.dy + 3.5);
        }
        canvas.drawPath(
          chevron,
          Paint()
            ..color = tokens.textSecondary
            ..style = PaintingStyle.stroke
            ..strokeWidth = 1.4
            ..strokeCap = StrokeCap.round
            ..strokeJoin = StrokeJoin.round,
        );
        if (n.expanded) {
          final f = n.formulaRegion;
          canvas.drawLine(
            f.topLeft + const Offset(1, 0),
            f.topRight + const Offset(-1, 0),
            Paint()..color = tokens.hairline,
          );
        }
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
  /// [receiving]: a relationship dragged over the region would join it on
  /// release — the insertion affordance.
  void region(
    Canvas canvas,
    GroupShape g, {
    bool selected = false,
    bool hovered = false,
    bool receiving = false,
  }) {
    final rrect = RRect.fromRectAndRadius(g.rect, const Radius.circular(10));
    final tint = tokens.isDark ? const Color(0xFF7A66A8) : const Color(0xFF8C6FC2);
    canvas.drawRRect(rrect, Paint()..color = tint.withValues(alpha: receiving ? 0.22 : 0.12));
    canvas.drawRRect(
      rrect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = selected || receiving ? 2 : 1
        ..color = selected || receiving
            ? tokens.accent
            : hovered
            ? tokens.textSecondary
            : tint.withValues(alpha: 0.5),
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
      case SocketKind.collection:
        // a stack: two rings, the second offset behind the first
        final d = r * 0.45;
        return Path()
          ..addOval(Rect.fromCircle(center: c + Offset(d, -d), radius: r * 0.85))
          ..addOval(Rect.fromCircle(center: c + Offset(-d, d), radius: r * 0.85));
      case SocketKind.grouped:
        // two cells side by side: a wide box with a divider
        final h = r * 0.85;
        final w = r * 1.35;
        return Path()
          ..addRRect(
            RRect.fromRectAndRadius(
              Rect.fromCenter(center: c, width: 2 * w, height: 2 * h),
              const Radius.circular(1.5),
            ),
          )
          ..moveTo(c.dx, c.dy - h)
          ..lineTo(c.dx, c.dy + h);
      case SocketKind.optional:
        // a ring with a hole: the value may be absent
        return Path()
          ..fillType = PathFillType.evenOdd
          ..addOval(Rect.fromCircle(center: c, radius: r))
          ..addOval(Rect.fromCircle(center: c, radius: r * 0.42));
    }
  }

  /// The Source glyph: a short shaft and an arrowhead pointing into the
  /// node, drawn at [at] (the tip's row), 12 px wide.
  void _entryGlyph(Canvas canvas, Offset at, Color color) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = 1.6
      ..strokeCap = StrokeCap.round
      ..style = PaintingStyle.stroke;
    final y = at.dy + 5;
    canvas.drawLine(Offset(at.dx, y), Offset(at.dx + 9, y), paint);
    canvas.drawLine(Offset(at.dx + 5, y - 4), Offset(at.dx + 9, y), paint);
    canvas.drawLine(Offset(at.dx + 5, y + 4), Offset(at.dx + 9, y), paint);
    // the boundary tick the arrow crosses
    canvas.drawLine(Offset(at.dx + 11.5, y - 5), Offset(at.dx + 11.5, y + 5), paint);
  }

  /// A dashed rectangle (the crossing marquee), with the dash length given
  /// in scene units so it reads the same at every zoom.
  void _dashedRect(Canvas canvas, Rect r, Paint paint, {double dash = 6}) {
    final path = Path()..addRect(r);
    for (final metric in path.computeMetrics()) {
      var d = 0.0;
      while (d < metric.length) {
        canvas.drawPath(metric.extractPath(d, d + dash), paint);
        d += dash * 2;
      }
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
