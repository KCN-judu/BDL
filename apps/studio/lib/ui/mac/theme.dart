import 'package:flutter/material.dart';

import 'tokens.dart';

/// A Material `ThemeData` tuned to read as a native macOS app: system font,
/// 13 pt body, flat controls, no ripples, hairline borders, accent only for
/// selection and the default action.
ThemeData macTheme(Brightness brightness) {
  final t = brightness == Brightness.dark ? MacTokens.dark : MacTokens.light;
  final scheme = ColorScheme.fromSeed(
    seedColor: t.accent,
    brightness: brightness,
    surface: t.content,
    primary: t.accent,
    error: t.error,
  );
  final base = ThemeData(
    brightness: brightness,
    colorScheme: scheme,
    useMaterial3: true,
    fontFamily: '.AppleSystemUIFont',
    fontFamilyFallback: const ['SF Pro Text', 'Helvetica Neue', 'Inter', 'Segoe UI', 'sans-serif'],
    visualDensity: VisualDensity.compact,
    splashFactory: NoSplash.splashFactory,
    highlightColor: Colors.transparent,
    hoverColor: t.controlHover,
    scaffoldBackgroundColor: t.window,
    canvasColor: t.content,
    dividerColor: t.hairline,
  );
  final text = base.textTheme.apply(bodyColor: t.textPrimary, displayColor: t.textPrimary);
  const body = 13.0;
  final states = MacStates(t);
  return base.copyWith(
    extensions: [t],
    textTheme: text.copyWith(
      bodyLarge: text.bodyLarge?.copyWith(fontSize: body, height: 1.25),
      bodyMedium: text.bodyMedium?.copyWith(fontSize: body, height: 1.25),
      bodySmall: text.bodySmall?.copyWith(fontSize: 11, height: 1.2, color: t.textSecondary),
      titleMedium: text.titleMedium?.copyWith(fontSize: body, fontWeight: FontWeight.w600),
      titleSmall: text.titleSmall?.copyWith(
        fontSize: 11,
        fontWeight: FontWeight.w600,
        color: t.textSecondary,
      ),
      labelLarge: text.labelLarge?.copyWith(fontSize: body, fontWeight: FontWeight.w500),
      labelMedium: text.labelMedium?.copyWith(fontSize: 11, color: t.textSecondary),
    ),
    dividerTheme: DividerThemeData(color: t.hairline, thickness: 1, space: 1),
    iconTheme: IconThemeData(size: 16, color: t.textSecondary),
    tooltipTheme: TooltipThemeData(
      waitDuration: const Duration(milliseconds: 600),
      textStyle: TextStyle(fontSize: 11, color: t.textPrimary),
      decoration: BoxDecoration(
        color: t.content,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: t.hairline),
        boxShadow: const [BoxShadow(color: Color(0x33000000), blurRadius: 8, offset: Offset(0, 2))],
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      isDense: true,
      filled: true,
      fillColor: t.control,
      contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(5),
        borderSide: BorderSide(color: t.hairline),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(5),
        borderSide: BorderSide(color: t.hairline),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(5),
        borderSide: BorderSide(color: t.accent, width: 2),
      ),
      hintStyle: TextStyle(fontSize: body, color: t.textTertiary),
      labelStyle: TextStyle(fontSize: 11, color: t.textSecondary),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style:
          FilledButton.styleFrom(
            minimumSize: const Size(0, MacMetrics.controlHeight),
            padding: const EdgeInsets.symmetric(horizontal: 12),
            textStyle: const TextStyle(fontSize: body, fontWeight: FontWeight.w500),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(5)),
            animationDuration: MacStates.duration,
          ).copyWith(
            overlayColor: states.overlay(onAccent: true),
            side: states.focusRing(),
            mouseCursor: states.cursor,
          ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style:
          OutlinedButton.styleFrom(
            minimumSize: const Size(0, MacMetrics.controlHeight),
            padding: const EdgeInsets.symmetric(horizontal: 12),
            foregroundColor: t.textPrimary,
            backgroundColor: t.control,
            textStyle: const TextStyle(fontSize: body),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(5)),
            animationDuration: MacStates.duration,
          ).copyWith(
            overlayColor: states.overlay(),
            side: states.focusRing(rest: BorderSide(color: t.hairline)),
            mouseCursor: states.cursor,
          ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        minimumSize: const Size(0, MacMetrics.controlHeight),
        padding: const EdgeInsets.symmetric(horizontal: 8),
        foregroundColor: t.accent,
        textStyle: const TextStyle(fontSize: body),
        animationDuration: MacStates.duration,
      ).copyWith(overlayColor: states.overlay(), mouseCursor: states.cursor),
    ),
    iconButtonTheme: IconButtonThemeData(
      style: IconButton.styleFrom(
        foregroundColor: t.textSecondary,
        minimumSize: const Size(28, 28),
        padding: EdgeInsets.zero,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(5)),
        animationDuration: MacStates.duration,
      ).copyWith(overlayColor: states.overlay(), mouseCursor: states.cursor),
    ),
    sliderTheme: SliderThemeData(
      trackHeight: 4,
      activeTrackColor: t.accent,
      inactiveTrackColor: t.hairline,
      thumbColor: t.control,
      overlayColor: t.accent.withValues(alpha: 0.12),
      overlayShape: const RoundSliderOverlayShape(overlayRadius: 14),
      thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 8, elevation: 1),
    ),
    checkboxTheme: CheckboxThemeData(
      fillColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? t.accent : t.control,
      ),
      checkColor: const WidgetStatePropertyAll(Colors.white),
      side: BorderSide(color: t.hairline),
      overlayColor: states.overlay(),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(3)),
      visualDensity: VisualDensity.compact,
    ),
    switchTheme: SwitchThemeData(
      thumbColor: const WidgetStatePropertyAll(Colors.white),
      trackColor: WidgetStateProperty.resolveWith(
        (s) => s.contains(WidgetState.selected) ? t.accent : t.hairline,
      ),
      trackOutlineColor: const WidgetStatePropertyAll(Colors.transparent),
      overlayColor: states.overlay(),
    ),
    popupMenuTheme: PopupMenuThemeData(
      color: t.content,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(6),
        side: BorderSide(color: t.hairline),
      ),
      textStyle: TextStyle(fontSize: body, color: t.textPrimary),
      menuPadding: const EdgeInsets.symmetric(vertical: 4),
    ),
    focusColor: t.accent.withValues(alpha: 0.25),
    listTileTheme: ListTileThemeData(
      dense: true,
      minVerticalPadding: 2,
      horizontalTitleGap: 6,
      contentPadding: const EdgeInsets.symmetric(horizontal: 10),
      selectedTileColor: t.selection,
      selectedColor: t.textPrimary,
      titleTextStyle: TextStyle(fontSize: body, color: t.textPrimary),
      subtitleTextStyle: TextStyle(fontSize: 11, color: t.textSecondary),
    ),
    dialogTheme: DialogThemeData(
      backgroundColor: t.window,
      surfaceTintColor: Colors.transparent,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      titleTextStyle: TextStyle(fontSize: body, fontWeight: FontWeight.w600, color: t.textPrimary),
      contentTextStyle: TextStyle(fontSize: body, color: t.textPrimary),
    ),
    dropdownMenuTheme: DropdownMenuThemeData(
      textStyle: TextStyle(fontSize: body, color: t.textPrimary),
    ),
    bannerTheme: MaterialBannerThemeData(
      backgroundColor: t.sidebar,
      contentTextStyle: TextStyle(fontSize: body, color: t.textPrimary),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 2),
    ),
  );
}

/// The interaction-state standard (docs/architecture/studio-ui.md §6), as
/// `WidgetStateProperty`s every control theme shares:
///
/// | state    | effect                                     |
/// |----------|--------------------------------------------|
/// | hover    | overlay 6 % (black on light, white on dark) |
/// | pressed  | overlay 12 %                               |
/// | focused  | 2 px accent ring (keyboard focus only)     |
/// | disabled | 40 % opacity, no hover                     |
/// | motion   | 120 ms ease-out                            |
class MacStates {
  const MacStates(this.t);
  final MacTokens t;

  static const Duration duration = Duration(milliseconds: 120);
  static const Curve curve = Curves.easeOut;
  static const double hoverAlpha = 0.06;
  static const double pressedAlpha = 0.12;
  static const double disabledOpacity = 0.4;

  Color get _ink => t.isDark ? Colors.white : Colors.black;

  /// Hover/pressed overlays.  On an accent-filled control the overlay is
  /// white so it lightens instead of muddying.
  WidgetStateProperty<Color?> overlay({bool onAccent = false}) {
    final ink = onAccent ? Colors.white : _ink;
    return WidgetStateProperty.resolveWith((s) {
      if (s.contains(WidgetState.disabled)) return null;
      if (s.contains(WidgetState.pressed)) return ink.withValues(alpha: pressedAlpha);
      if (s.contains(WidgetState.hovered)) return ink.withValues(alpha: hoverAlpha);
      return null;
    });
  }

  /// Keyboard-focus ring; pointer clicks do not show it.
  WidgetStateProperty<BorderSide?> focusRing({BorderSide? rest}) =>
      WidgetStateProperty.resolveWith((s) {
        if (s.contains(WidgetState.focused)) return BorderSide(color: t.accent, width: 2);
        return rest;
      });

  /// Arrow everywhere (macOS), except a hand on text links (see `MacLink`).
  WidgetStateProperty<MouseCursor> get cursor => WidgetStateProperty.resolveWith(
    (s) => s.contains(WidgetState.disabled) ? SystemMouseCursors.basic : SystemMouseCursors.basic,
  );

  /// Background for hoverable rows and links.
  Color rowHover() => _ink.withValues(alpha: hoverAlpha);
  Color rowPressed() => _ink.withValues(alpha: pressedAlpha);
}
