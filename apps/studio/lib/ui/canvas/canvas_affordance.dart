/// The contextual affordance of a selected canvas object
/// (docs/architecture/studio-ui.md §2, "Contextual affordances"): a
/// compact row of icon buttons that appears when the selected object is
/// hovered, at a deterministic anchor beside it, and offers the object's
/// menu and at most two of its own commands.  It reveals nothing on hover
/// alone (hover says "interactive"; selection says "this one"), it never
/// holds a command the contextual menu lacks, and every icon carries its
/// name as tooltip and accessibility label.  One widget for edges, nodes
/// and groups; what differs is the owner and its actions.
library;

import 'package:flutter/material.dart';

import '../mac/theme.dart' show MacStates;
import '../mac/tokens.dart';

/// One quick action: its icon, its name (the tooltip and the label a
/// screen reader speaks), what it does.  A destructive action is drawn in
/// the error tone only while hovered — never the dominant thing on show.
class AffordanceAction {
  const AffordanceAction({
    required this.icon,
    required this.label,
    required this.onPressed,
    this.destructive = false,
  });
  final IconData icon;
  final String label;
  final VoidCallback onPressed;
  final bool destructive;
}

/// Where the row sits relative to its [CanvasAffordance.anchor] (local
/// pixels): centred above it (an edge's midpoint), or ending at it (a
/// node's top-right corner).
enum AffordanceAlignment { centerAbove, endAbove }

/// The affordance of one object.  [owner] names it (the key: a new owner
/// is a new widget, so no hover state carries over); [anchor] is in the
/// canvas's local pixels; the row's size is fixed in pixels, whatever the
/// zoom — it is chrome, not a thing on the canvas.
class CanvasAffordance extends StatelessWidget {
  const CanvasAffordance({
    super.key,
    required this.owner,
    required this.actions,
    required this.anchor,
    this.alignment = AffordanceAlignment.centerAbove,
  });
  final Object owner;
  final List<AffordanceAction> actions;
  final Offset anchor;
  final AffordanceAlignment alignment;

  static const double buttonSize = 22;
  static const double gap = 2;
  static const double padding = 3;
  static const double height = buttonSize + padding * 2;

  /// The gap between the anchor and the row's bottom edge: clear of the
  /// stroke or the header, close enough to reach without leaving the
  /// combined hover region.
  static const double lift = 8;

  /// How far outside the row the pointer still counts as on it: the
  /// combined hover region of the object and its affordance stays
  /// contiguous while the pointer moves from one to the other.
  static const double hoverInflate = 12;

  static double widthFor(int actions) => padding * 2 + actions * buttonSize + (actions - 1) * gap;

  /// The row's rectangle for [anchor] and [alignment]: the same geometry
  /// the canvas uses to keep the pointer's hover on the row.
  static Rect rectFor(Offset anchor, int actions, AffordanceAlignment alignment) {
    final w = widthFor(actions);
    final left = switch (alignment) {
      AffordanceAlignment.centerAbove => anchor.dx - w / 2,
      AffordanceAlignment.endAbove => anchor.dx - w,
    };
    return Rect.fromLTWH(left, anchor.dy - lift - height, w, height);
  }

  Rect get rect => rectFor(anchor, actions.length, alignment);

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Positioned.fromRect(
      rect: rect,
      child: Semantics(
        container: true,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: t.content,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: t.hairline),
            boxShadow: const [
              BoxShadow(color: Color(0x1F000000), blurRadius: 4, offset: Offset(0, 1)),
            ],
          ),
          child: Padding(
            padding: const EdgeInsets.all(padding),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (var i = 0; i < actions.length; i++) ...[
                  if (i > 0) const SizedBox(width: gap),
                  _AffordanceButton(action: actions[i]),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _AffordanceButton extends StatefulWidget {
  const _AffordanceButton({required this.action});
  final AffordanceAction action;

  @override
  State<_AffordanceButton> createState() => _AffordanceButtonState();
}

class _AffordanceButtonState extends State<_AffordanceButton> {
  bool _hover = false;
  bool _pressed = false;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final a = widget.action;
    final ink = t.isDark ? Colors.white : Colors.black;
    // The icon is secondary until the pointer is on it; a destructive one
    // then takes the error tone — a warning at the moment of choice, not
    // a red button sitting on the canvas.
    final fg = _hover ? (a.destructive ? t.error : t.textPrimary) : t.textSecondary;
    final overlay = _pressed
        ? ink.withValues(alpha: MacStates.pressedAlpha)
        : _hover
        ? ink.withValues(alpha: MacStates.hoverAlpha)
        : null;
    return Tooltip(
      message: a.label,
      waitDuration: const Duration(milliseconds: 500),
      child: Semantics(
        button: true,
        label: a.label,
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          onEnter: (_) => setState(() => _hover = true),
          onExit: (_) => setState(() {
            _hover = false;
            _pressed = false;
          }),
          child: GestureDetector(
            behavior: HitTestBehavior.opaque,
            onTapDown: (_) => setState(() => _pressed = true),
            onTapUp: (_) => setState(() => _pressed = false),
            onTapCancel: () => setState(() => _pressed = false),
            onTap: a.onPressed,
            child: AnimatedContainer(
              duration: MacStates.duration,
              curve: MacStates.curve,
              width: CanvasAffordance.buttonSize,
              height: CanvasAffordance.buttonSize,
              decoration: BoxDecoration(color: overlay, borderRadius: BorderRadius.circular(4)),
              child: Icon(a.icon, size: 14, color: fg),
            ),
          ),
        ),
      ),
    );
  }
}
