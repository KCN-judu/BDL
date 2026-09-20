/// The one menu primitive of Studio's macOS look (docs/architecture/studio-ui.md
/// §3, "Menus"): a compact command menu on the content colour with a
/// hairline, 24 pt rows in the body size, a secondary shortcut column, a
/// submenu chevron, the accent tint on the focused row, a disabled row at
/// 40 % with its reason beneath it, a destructive row in the error colour.
/// Contextual menus, choosers and any compact command menu are built from
/// these; nothing styles a menu anywhere else.
///
/// Flutter's `MenuAnchor` family underneath — it owns the overlay, focus
/// traversal (↑ ↓ ← → ⏎ Esc) and accessibility roles; only its look and
/// its lifecycle callbacks are adapted here.
library;

import 'package:flutter/material.dart';

import 'theme.dart';
import 'tokens.dart';

/// The anchor of a menu: opens at a position on its controller, closes on
/// Esc, a command, or a tap outside (which it consumes — the tap edits
/// nothing beneath).  [onClose] fires whenever the menu closes, so the
/// owner's state can follow the overlay's.
class MacMenuAnchor extends StatelessWidget {
  const MacMenuAnchor({
    super.key,
    required this.controller,
    required this.items,
    required this.child,
    this.onClose,
    this.onOpen,
  });
  final MenuController controller;
  final List<Widget> items;
  final Widget child;
  final VoidCallback? onClose;
  final VoidCallback? onOpen;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    // A menu opened at the pointer has no button to hold focus: its first
    // row takes it, so ↑ ↓ ⏎ work from the start.
    final focused = [
      for (var i = 0; i < items.length; i++)
        if (i == 0 && items[i] is MacMenuItem)
          (items[i] as MacMenuItem).withAutofocus()
        else if (i == 0 && items[i] is MacSubmenu)
          (items[i] as MacSubmenu).withAutofocus()
        else
          items[i],
    ];
    return MenuAnchor(
      controller: controller,
      consumeOutsideTap: true,
      onClose: onClose,
      onOpen: onOpen,
      style: menuStyle(t),
      menuChildren: focused,
      child: child,
    );
  }
}

/// The look of every menu surface (the anchor's and each submenu's).
MenuStyle menuStyle(MacTokens t) => MenuStyle(
  backgroundColor: WidgetStatePropertyAll(t.content),
  surfaceTintColor: const WidgetStatePropertyAll(Colors.transparent),
  shadowColor: const WidgetStatePropertyAll(Color(0x40000000)),
  elevation: const WidgetStatePropertyAll(8),
  padding: const WidgetStatePropertyAll(EdgeInsets.symmetric(vertical: MacMetrics.gapTight)),
  shape: WidgetStatePropertyAll(
    RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(MacMetrics.radius),
      side: BorderSide(color: t.hairline),
    ),
  ),
);

/// The look of a menu row: the standard hover/pressed overlays, the accent
/// tint when focused by keyboard, disabled at 40 %, destructive in the
/// error colour.
ButtonStyle menuRowStyle(MacTokens t, {bool destructive = false}) {
  final states = MacStates(t);
  final fg = destructive ? t.error : t.textPrimary;
  return ButtonStyle(
    minimumSize: const WidgetStatePropertyAll(Size(160, MacMetrics.rowHeight)),
    padding: const WidgetStatePropertyAll(EdgeInsets.symmetric(horizontal: 12)),
    textStyle: WidgetStatePropertyAll(TextStyle(fontSize: MacType.body)),
    foregroundColor: WidgetStateProperty.resolveWith(
      (s) =>
          s.contains(WidgetState.disabled) ? fg.withValues(alpha: MacStates.disabledOpacity) : fg,
    ),
    backgroundColor: WidgetStateProperty.resolveWith(
      (s) => s.contains(WidgetState.focused) ? t.selection : null,
    ),
    overlayColor: states.overlay(),
    iconColor: WidgetStatePropertyAll(t.textSecondary),
    iconSize: const WidgetStatePropertyAll(14),
    shape: const WidgetStatePropertyAll(RoundedRectangleBorder()),
    animationDuration: MacStates.duration,
    mouseCursor: const WidgetStatePropertyAll(SystemMouseCursors.basic),
  );
}

/// One command.  [shortcut] is drawn in the secondary column; [detail]
/// (a reason, what a choice connects to) beneath the label in the
/// secondary style.  A `null` [onPressed] is a disabled row — shown only
/// when its reason is worth reading in place.
class MacMenuItem extends StatelessWidget {
  const MacMenuItem({
    super.key,
    required this.label,
    required this.onPressed,
    this.shortcut,
    this.detail,
    this.destructive = false,
    this.autofocus = false,
  });
  final String label;
  final VoidCallback? onPressed;
  final String? shortcut;
  final String? detail;
  final bool destructive;
  final bool autofocus;

  MacMenuItem withAutofocus() => MacMenuItem(
    key: key,
    label: label,
    onPressed: onPressed,
    shortcut: shortcut,
    detail: detail,
    destructive: destructive,
    autofocus: true,
  );

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final secondary = Theme.of(context).textTheme.bodySmall;
    return MenuItemButton(
      onPressed: onPressed,
      autofocus: autofocus && onPressed != null,
      style: menuRowStyle(t, destructive: destructive),
      trailingIcon: shortcut == null
          ? null
          : Padding(
              padding: const EdgeInsets.only(left: MacMetrics.gapGroup),
              child: Text(shortcut!, style: secondary),
            ),
      child: detail == null
          ? Text(label)
          : Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(label),
                Text(detail!, style: secondary),
              ],
            ),
    );
  }
}

/// A submenu: the chevron is the platform's; the children are built from
/// the same primitives.  [autofocus] gives the row focus when its menu
/// opens (the first row of a menu opened at the pointer).
class MacSubmenu extends StatefulWidget {
  const MacSubmenu({
    super.key,
    required this.label,
    required this.children,
    this.autofocus = false,
  });
  final String label;
  final List<Widget> children;
  final bool autofocus;

  MacSubmenu withAutofocus() =>
      MacSubmenu(key: key, label: label, autofocus: true, children: children);

  @override
  State<MacSubmenu> createState() => _MacSubmenuState();
}

class _MacSubmenuState extends State<MacSubmenu> {
  final FocusNode _focus = FocusNode(debugLabel: 'submenu');

  @override
  void initState() {
    super.initState();
    if (widget.autofocus) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) _focus.requestFocus();
      });
    }
  }

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return SubmenuButton(
      focusNode: _focus,
      style: menuRowStyle(t),
      menuStyle: menuStyle(t),
      menuChildren: widget.children,
      child: Text(widget.label),
    );
  }
}

/// A hairline between groups of commands — between groups only, never
/// between commands of one group.
class MacMenuDivider extends StatelessWidget {
  const MacMenuDivider({super.key});

  @override
  Widget build(BuildContext context) => const Divider(height: MacMetrics.gap + 1);
}
