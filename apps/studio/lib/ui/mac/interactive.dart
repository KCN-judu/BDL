/// Shared interactive primitives that implement the interaction standard
/// (docs/architecture/studio-ui.md §6) for things Material has no native control for:
/// text links and hoverable rows.  Every hover/pressed/focus/disabled
/// treatment in Studio comes from here or from `MacStates` in the theme.
library;

import 'package:flutter/material.dart';

import 'theme.dart';
import 'tokens.dart';

/// A surface that reacts to hover, press and keyboard focus with the
/// standard overlays and a rounded highlight.  Wrap rows, links, chips.
class MacInteractive extends StatefulWidget {
  const MacInteractive({
    super.key,
    required this.child,
    this.onTap,
    this.onDoubleTap,
    this.selected = false,
    this.radius = 5,
    this.cursor = SystemMouseCursors.basic,
    this.padding = EdgeInsets.zero,
    this.onHoverChanged,
  });

  final Widget child;
  final VoidCallback? onTap;
  final VoidCallback? onDoubleTap;
  final bool selected;
  final double radius;
  final MouseCursor cursor;
  final EdgeInsets padding;
  final ValueChanged<bool>? onHoverChanged;

  bool get enabled => onTap != null || onDoubleTap != null;

  @override
  State<MacInteractive> createState() => _MacInteractiveState();
}

class _MacInteractiveState extends State<MacInteractive> {
  bool _hover = false;
  bool _pressed = false;
  bool _focused = false;

  void _setHover(bool v) {
    if (_hover == v) return;
    setState(() => _hover = v);
    widget.onHoverChanged?.call(v);
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final states = MacStates(t);
    final enabled = widget.enabled;
    Color? bg;
    if (widget.selected) {
      bg = t.selection;
    } else if (enabled && _pressed) {
      bg = states.rowPressed();
    } else if (enabled && _hover) {
      bg = states.rowHover();
    }
    return FocusableActionDetector(
      enabled: enabled,
      mouseCursor: enabled ? widget.cursor : SystemMouseCursors.basic,
      onShowHoverHighlight: _setHover,
      onShowFocusHighlight: (v) => setState(() => _focused = v),
      actions: {
        ActivateIntent: CallbackAction<ActivateIntent>(
          onInvoke: (_) {
            widget.onTap?.call();
            return null;
          },
        ),
      },
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: enabled ? (_) => setState(() => _pressed = true) : null,
        onTapUp: enabled ? (_) => setState(() => _pressed = false) : null,
        onTapCancel: enabled ? () => setState(() => _pressed = false) : null,
        onTap: widget.onTap,
        onDoubleTap: widget.onDoubleTap,
        child: AnimatedContainer(
          duration: MacStates.duration,
          curve: MacStates.curve,
          padding: widget.padding,
          decoration: BoxDecoration(
            color: bg,
            borderRadius: BorderRadius.circular(widget.radius),
            border: _focused ? Border.all(color: t.accent, width: 2) : null,
          ),
          child: Opacity(opacity: enabled ? 1 : MacStates.disabledOpacity, child: widget.child),
        ),
      ),
    );
  }
}

/// A text link: accent colour, hand cursor, hover pill, no underline
/// (macOS), optional leading icon and trailing shortcut hint.
class MacLink extends StatelessWidget {
  const MacLink({
    super.key,
    required this.label,
    required this.onTap,
    this.icon,
    this.shortcut,
    this.enabled = true,
  });
  final String label;
  final VoidCallback onTap;
  final IconData? icon;
  final String? shortcut;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Align(
      alignment: Alignment.centerLeft,
      child: MacInteractive(
        onTap: enabled ? onTap : null,
        cursor: SystemMouseCursors.click,
        padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 6),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (icon != null) ...[Icon(icon, size: 16, color: t.accent), const SizedBox(width: 8)],
            Text(label, style: TextStyle(fontSize: 13, color: t.accent)),
            if (shortcut != null) ...[
              const SizedBox(width: 10),
              Text(shortcut!, style: TextStyle(fontSize: 11, color: t.textTertiary)),
            ],
          ],
        ),
      ),
    );
  }
}
