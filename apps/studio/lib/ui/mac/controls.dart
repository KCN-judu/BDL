/// macOS-style text field and push buttons (docs/architecture/studio-ui.md §3, §6).
///
/// `MacTextField`: 1 px hairline, 5 pt radius, 24 pt tall; focus shows the
/// macOS focus *glow* (a soft 3 pt accent halo outside the border) instead
/// of a thicker border, so nothing shifts.
/// `MacButton`: 22 pt tall, ≥ 72 pt wide, primary is the accent fill.
library;

import 'package:flutter/material.dart';

import 'theme.dart';
import 'tokens.dart';

class MacTextField extends StatefulWidget {
  const MacTextField({
    super.key,
    this.controller,
    this.focusNode,
    this.hint,
    this.autofocus = false,
    this.maxLines = 1,
    this.monospace = false,
    this.onSubmitted,
    this.onChanged,
  });

  final TextEditingController? controller;
  final FocusNode? focusNode;
  final String? hint;
  final bool autofocus;
  final int maxLines;
  final bool monospace;
  final ValueChanged<String>? onSubmitted;
  final ValueChanged<String>? onChanged;

  @override
  State<MacTextField> createState() => _MacTextFieldState();
}

class _MacTextFieldState extends State<MacTextField> {
  late final FocusNode _focus = widget.focusNode ?? FocusNode();
  bool _focused = false;

  @override
  void initState() {
    super.initState();
    _focus.addListener(_onFocus);
  }

  void _onFocus() => setState(() => _focused = _focus.hasFocus);

  @override
  void dispose() {
    _focus.removeListener(_onFocus);
    if (widget.focusNode == null) _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return AnimatedContainer(
      duration: MacStates.duration,
      curve: MacStates.curve,
      decoration: BoxDecoration(
        color: t.control,
        borderRadius: BorderRadius.circular(5),
        border: Border.all(color: _focused ? t.accent : t.hairline),
        boxShadow: _focused
            ? [BoxShadow(color: t.accent.withValues(alpha: 0.35), blurRadius: 0, spreadRadius: 3)]
            : null,
      ),
      child: TextField(
        controller: widget.controller,
        focusNode: _focus,
        autofocus: widget.autofocus,
        maxLines: widget.maxLines,
        onSubmitted: widget.onSubmitted,
        onChanged: widget.onChanged,
        cursorColor: t.accent,
        cursorWidth: 1.5,
        style: TextStyle(
          fontSize: 13,
          height: 1.3,
          color: t.textPrimary,
          fontFamily: widget.monospace ? 'Menlo' : null,
        ),
        decoration: InputDecoration(
          isDense: true,
          filled: false,
          border: InputBorder.none,
          enabledBorder: InputBorder.none,
          focusedBorder: InputBorder.none,
          contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          hintText: widget.hint,
          hintStyle: TextStyle(fontSize: 13, color: t.textTertiary),
        ),
      ),
    );
  }
}

enum MacButtonKind { primary, secondary, destructive }

class MacButton extends StatefulWidget {
  const MacButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.kind = MacButtonKind.secondary,
    this.tooltip,
  });

  const MacButton.primary({super.key, required this.label, required this.onPressed, this.tooltip})
    : kind = MacButtonKind.primary;

  const MacButton.destructive({
    super.key,
    required this.label,
    required this.onPressed,
    this.tooltip,
  }) : kind = MacButtonKind.destructive;

  final String label;
  final VoidCallback? onPressed;
  final MacButtonKind kind;
  final String? tooltip;

  @override
  State<MacButton> createState() => _MacButtonState();
}

class _MacButtonState extends State<MacButton> {
  bool _hover = false;
  bool _pressed = false;
  bool _focused = false;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final enabled = widget.onPressed != null;
    final primary = widget.kind == MacButtonKind.primary;

    final Color fill = primary ? t.accent : t.control;
    final Color fg = switch (widget.kind) {
      MacButtonKind.primary => Colors.white,
      MacButtonKind.secondary => t.textPrimary,
      MacButtonKind.destructive => t.error,
    };
    final ink = primary ? Colors.white : (t.isDark ? Colors.white : Colors.black);
    final overlay = !enabled
        ? null
        : _pressed
        ? ink.withValues(alpha: MacStates.pressedAlpha)
        : _hover
        ? ink.withValues(alpha: MacStates.hoverAlpha)
        : null;

    Widget button = FocusableActionDetector(
      enabled: enabled,
      onShowHoverHighlight: (v) => setState(() => _hover = v),
      onShowFocusHighlight: (v) => setState(() => _focused = v),
      actions: {
        ActivateIntent: CallbackAction<ActivateIntent>(
          onInvoke: (_) {
            widget.onPressed?.call();
            return null;
          },
        ),
      },
      child: GestureDetector(
        onTapDown: enabled ? (_) => setState(() => _pressed = true) : null,
        onTapUp: enabled ? (_) => setState(() => _pressed = false) : null,
        onTapCancel: enabled ? () => setState(() => _pressed = false) : null,
        onTap: widget.onPressed,
        child: Opacity(
          opacity: enabled ? 1 : MacStates.disabledOpacity,
          child: AnimatedContainer(
            duration: MacStates.duration,
            curve: MacStates.curve,
            height: MacMetrics.controlHeight,
            constraints: const BoxConstraints(minWidth: 72),
            padding: const EdgeInsets.symmetric(horizontal: 14),
            decoration: BoxDecoration(
              color: overlay == null ? fill : Color.alphaBlend(overlay, fill),
              borderRadius: BorderRadius.circular(5),
              border: primary ? null : Border.all(color: t.hairline),
              boxShadow: _focused
                  ? [BoxShadow(color: t.accent.withValues(alpha: 0.35), spreadRadius: 3)]
                  : primary
                  ? const [
                      BoxShadow(color: Color(0x22000000), blurRadius: 1, offset: Offset(0, 0.5)),
                    ]
                  : null,
            ),
            // Sized to the label (≥ 72 pt), never to the space offered: a
            // push button in a stretching column or a wrap stays a button.
            child: Center(
              widthFactor: 1,
              child: Text(
                widget.label,
                style: TextStyle(
                  fontSize: 13,
                  fontWeight: primary ? FontWeight.w600 : FontWeight.w400,
                  color: fg,
                ),
              ),
            ),
          ),
        ),
      ),
    );
    if (widget.tooltip != null) button = Tooltip(message: widget.tooltip!, child: button);
    return button;
  }
}
