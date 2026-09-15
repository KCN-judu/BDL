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
      style: FilledButton.styleFrom(
        minimumSize: const Size(0, MacMetrics.controlHeight),
        padding: const EdgeInsets.symmetric(horizontal: 12),
        textStyle: const TextStyle(fontSize: body, fontWeight: FontWeight.w500),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(5)),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        minimumSize: const Size(0, MacMetrics.controlHeight),
        padding: const EdgeInsets.symmetric(horizontal: 12),
        foregroundColor: t.textPrimary,
        backgroundColor: t.control,
        side: BorderSide(color: t.hairline),
        textStyle: const TextStyle(fontSize: body),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(5)),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: TextButton.styleFrom(
        minimumSize: const Size(0, MacMetrics.controlHeight),
        padding: const EdgeInsets.symmetric(horizontal: 8),
        foregroundColor: t.accent,
        textStyle: const TextStyle(fontSize: body),
      ),
    ),
    iconButtonTheme: IconButtonThemeData(
      style: IconButton.styleFrom(
        foregroundColor: t.textSecondary,
        minimumSize: const Size(28, 28),
        padding: EdgeInsets.zero,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(5)),
      ),
    ),
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
