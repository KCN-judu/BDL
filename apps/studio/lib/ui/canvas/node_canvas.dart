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
  });

  final pb.ProjectProjection project;
  final Map<NodeRef, Offset> layout;
  final Selection selection;
  final void Function(AppAction) dispatch;

  /// Compiler verdicts per mapping id, when an analysis of this revision
  /// exists.
  final Map<int, pb.MappingStatus> statuses;

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

  Map<NodeRef, Offset> get _effectiveLayout {
    if (_draggingNode == null) return widget.layout;
    final scene = buildScene(widget.project, widget.layout, statuses: widget.statuses);
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
    final scene = buildScene(widget.project, _effectiveLayout, statuses: widget.statuses);
    final hit = hitTest(scene, _toScene(e.localPosition));
    final (NodeRef? node, SocketRef? socket) = switch (hit) {
      HitSocket(:final socket, :final node) => (node.ref, socket.ref),
      HitNode(:final node) => (node.ref, null),
      HitNothing() => (null, null),
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
    final scene = buildScene(widget.project, widget.layout, statuses: widget.statuses);
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
        final hit = hitTest(
          buildScene(widget.project, widget.layout, statuses: widget.statuses),
          p,
        );
        _hoverSocket = hit is HitSocket ? hit.socket.ref : null;
      } else if (_draggingNode != null) {
        _dragDelta += d.delta / _zoom;
      } else if (_panning) {
        _pan += d.delta;
      }
    });
  }

  void _onPanEnd(DragEndDetails d) {
    final link = _linkDrag;
    if (link != null) {
      final scene = buildScene(widget.project, widget.layout, statuses: widget.statuses);
      final target = dropTarget(scene, link.from, link.current);
      if (target != null) {
        _makeLink(link.from, target.ref);
      } else if (link.fromConnectedInput && hitTest(scene, link.current) is HitNothing) {
        widget.dispatch(
          UnlinkMappingInput(mappingId: link.from.node.id, conceptId: link.from.concept),
        );
      }
    }
    final node = _draggingNode;
    if (node != null && _dragDelta != Offset.zero) {
      widget.dispatch(NodeMoved(node, _effectiveLayout[node]!));
    }
    setState(() {
      _linkDrag = null;
      _draggingNode = null;
      _dragDelta = Offset.zero;
      _panning = false;
    });
  }

  /// A link is always output → input in data-flow terms, whichever end was
  /// dragged first.
  void _makeLink(SocketRef a, SocketRef b) {
    final (out, inp) = a.side == SocketSide.output ? (a, b) : (b, a);
    if (out.node.kind == NodeKind.concept && inp.node.kind == NodeKind.mapping) {
      widget.dispatch(LinkConceptToMappingInput(conceptId: out.concept, mappingId: inp.node.id));
    } else if (out.node.kind == NodeKind.mapping && inp.node.kind == NodeKind.concept) {
      widget.dispatch(LinkMappingOutputToConcept(mappingId: out.node.id, conceptId: inp.concept));
    }
  }

  Selection _select(NodeRef ref) => switch (ref.kind) {
    NodeKind.concept => ConceptSelected(ref.id),
    NodeKind.mapping => MappingSelected(ref.id),
  };

  void _frameAll(Size viewport) {
    final scene = buildScene(widget.project, widget.layout, statuses: widget.statuses);
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
    final scene = buildScene(widget.project, _effectiveLayout, statuses: widget.statuses);
    final selected = switch (widget.selection) {
      ConceptSelected(:final id) => NodeRef.concept(id),
      MappingSelected(:final id) => NodeRef.mapping(id),
      NoSelection() => null,
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
                child: ClipRect(
                  child: CustomPaint(
                    painter: _CanvasPainter(
                      scene: scene,
                      tokens: t,
                      pan: _pan,
                      zoom: _zoom,
                      selected: selected,
                      hovered: _hoverNode,
                      hoveredSocket: _hoverSocket,
                      linkDrag: _linkDrag,
                      dropOk: _linkDrag == null
                          ? null
                          : dropTarget(scene, _linkDrag!.from, _linkDrag!.current)?.ref,
                    ),
                    size: Size.infinite,
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

class _CanvasPainter extends CustomPainter {
  _CanvasPainter({
    required this.scene,
    required this.tokens,
    required this.pan,
    required this.zoom,
    required this.selected,
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

    for (final l in scene.links) {
      canvas.drawPath(
        l.path,
        Paint()
          ..color = tokens.conceptColor(l.concept)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..strokeCap = StrokeCap.round,
      );
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
    final painter = NodePainter(
      tokens,
      hoveredSocket: hoveredSocket,
      dropOk: dropOk,
      compatible: drag == null ? const {} : compatibleSockets(scene, drag.from),
    );
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

    // Category tint: the whole concept object, the mapping's header strip.
    final headerColor = switch (n.ref.kind) {
      NodeKind.concept => tokens.isDark ? const Color(0xFF3A4556) : const Color(0xFFDCE3EE),
      NodeKind.mapping => tokens.isDark ? const Color(0xFF2E4A6B) : const Color(0xFFCFE0F5),
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
    if (n.declared) {
      _dashedRRect(canvas, rrect, outline);
    } else {
      canvas.drawRRect(rrect, outline);
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
    if (n.declared) {
      _text(
        canvas,
        'declared',
        n.header.topRight + const Offset(-10, 8),
        FontWeight.w400,
        10,
        tokens.textSecondary,
        alignRight: true,
      );
    }

    for (final s in n.sockets) {
      socket(canvas, s.center, s.kind, conceptColor(s.ref.concept), ref: s.ref);
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
          maxWidth: region.right - 10 - left,
        );
      }
    }
  }

  /// One socket: shape by value form, hue by identity.  Reused by every
  /// widget that shows a concept (library rows, chips, toggles) so the mark
  /// is learned once.
  void socket(Canvas canvas, Offset c, SocketKind kind, Color color, {SocketRef? ref}) {
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
    if (kind != SocketKind.open) {
      canvas.drawPath(path, Paint()..color = color);
    }
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = kind == SocketKind.open ? color : tokens.content.withValues(alpha: 0.9),
    );
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
