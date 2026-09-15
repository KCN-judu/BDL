/// macOS look tokens (docs/STUDIO_UI.md §3).  One place for every colour,
/// size and radius so the chrome stays consistent and swappable.
library;

import 'package:flutter/material.dart';

@immutable
class MacTokens extends ThemeExtension<MacTokens> {
  const MacTokens({
    required this.window,
    required this.sidebar,
    required this.content,
    required this.canvas,
    required this.canvasGrid,
    required this.hairline,
    required this.control,
    required this.controlHover,
    required this.textPrimary,
    required this.textSecondary,
    required this.textTertiary,
    required this.accent,
    required this.selection,
    required this.settled,
    required this.open,
    required this.error,
    required this.isDark,
  });

  final Color window;
  final Color sidebar;
  final Color content;
  final Color canvas;
  final Color canvasGrid;
  final Color hairline;
  final Color control;
  final Color controlHover;
  final Color textPrimary;
  final Color textSecondary;
  final Color textTertiary;
  final Color accent;
  final Color selection;
  final Color settled;
  final Color open;
  final Color error;
  final bool isDark;

  static const light = MacTokens(
    window: Color(0xFFECECEC),
    sidebar: Color(0xFFF5F5F5),
    content: Color(0xFFFFFFFF),
    canvas: Color(0xFFF2F2F4),
    canvasGrid: Color(0x14000000),
    hairline: Color(0x1F000000),
    control: Color(0xFFFFFFFF),
    controlHover: Color(0xFFF0F0F0),
    textPrimary: Color(0xD9000000),
    textSecondary: Color(0x80000000),
    textTertiary: Color(0x40000000),
    accent: Color(0xFF0A60FF),
    selection: Color(0x330A60FF),
    settled: Color(0xFF28A745),
    open: Color(0xFFE8912D),
    error: Color(0xFFD93025),
    isDark: false,
  );

  static const dark = MacTokens(
    window: Color(0xFF1E1E1E),
    sidebar: Color(0xFF2B2B2B),
    content: Color(0xFF262626),
    canvas: Color(0xFF202124),
    canvasGrid: Color(0x14FFFFFF),
    hairline: Color(0x1FFFFFFF),
    control: Color(0xFF363636),
    controlHover: Color(0xFF404040),
    textPrimary: Color(0xD9FFFFFF),
    textSecondary: Color(0x8CFFFFFF),
    textTertiary: Color(0x47FFFFFF),
    accent: Color(0xFF3B82F6),
    selection: Color(0x403B82F6),
    settled: Color(0xFF34C759),
    open: Color(0xFFFF9F0A),
    error: Color(0xFFFF453A),
    isDark: true,
  );

  static MacTokens of(BuildContext context) =>
      Theme.of(context).extension<MacTokens>() ?? MacTokens.light;

  /// Stable hue per semantic identity: socket and link colour *is* the
  /// nominal type (docs/STUDIO_UI.md §2).  Golden-angle spacing keeps
  /// neighbouring ids visually distinct.
  ///
  /// Lightness is chosen per hue so every identity clears the 3:1
  /// non-text contrast floor against the canvas in both appearances — a
  /// fixed HSL lightness leaves yellows and cyans at ~2:1 on the light
  /// canvas.  Deterministic: the same id always gets the same colour.
  Color conceptColor(int id) {
    final hue = (id * 137.508) % 360.0;
    var l = isDark ? 0.62 : 0.48;
    for (var i = 0; i < 20; i++) {
      final c = HSLColor.fromAHSL(1, hue, 0.55, l).toColor();
      final ratio = _contrast(c.computeLuminance(), canvas.computeLuminance());
      if (ratio >= 3.4) return c;
      l += isDark ? 0.02 : -0.02;
    }
    return HSLColor.fromAHSL(1, hue, 0.55, l).toColor();
  }

  static double _contrast(double a, double b) {
    final hi = a > b ? a : b;
    final lo = a > b ? b : a;
    return (hi + 0.05) / (lo + 0.05);
  }

  @override
  MacTokens copyWith() => this;

  @override
  MacTokens lerp(ThemeExtension<MacTokens>? other, double t) =>
      t < 0.5 ? this : (other as MacTokens);
}

/// Sizes shared by chrome and canvas.
abstract final class MacMetrics {
  static const double sidebarWidth = 220;
  static const double inspectorWidth = 290;
  static const double toolbarHeight = 44;
  static const double statusHeight = 22;
  static const double pageBarHeight = 44;
  static const double rowHeight = 24;
  static const double controlHeight = 22;
  static const double radius = 6;
  static const double grid = 8;

  // Separation is whitespace (STUDIO_UI.md §3a).
  static const double gapTight = 4;
  static const double gap = 8;
  static const double gapGroup = 16;
  static const double gapSection = 24;
  static const double formLabelWidth = 78;
  static const double gutter = 16;
}
