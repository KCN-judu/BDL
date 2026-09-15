/// The BDL language mark, painted from the same geometry as
/// `assets/brand/gen_logo.py`: a drafting compass reduced to straight
/// lines — two equal legs opened symmetrically (tips level), the stalk
/// continuing the right leg so the instrument leans into a λ.  Parallel
/// edges, chamfered tips, square blocks aligned with the stalk.  Vector
/// code, crisp at any size, no SVG runtime.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';

abstract final class BrandColors {
  static const black = Color(0xFF141414);
  static const white = Color(0xFFF4F4F2);
  static const red = Color(0xFFE5322D);
  static const cream = Color(0xFFF2EBDD);
  static const blue = Color(0xFF2B5DD1);
  static const darkPin = Color(0xFF1E1E1E);
}

/// The mark in the theme's version: black body on light themes, white body
/// on dark ones (the red blocks stay red).  Pass [monochrome] for a
/// single-colour toolbar glyph, or [tile] for the app-icon variant.
class CompassMark extends StatelessWidget {
  const CompassMark({super.key, this.size = 64, this.tile = false, this.monochrome, this.dark});

  final double size;

  /// Draw the cream tile behind the compass (the app-icon variant; always
  /// the black body).
  final bool tile;

  /// Single-colour variant (for toolbars); ignores the palette.
  final Color? monochrome;

  /// Force the dark-theme (white body) version; defaults to the theme.
  final bool? dark;

  @override
  Widget build(BuildContext context) {
    final onDark = dark ?? Theme.of(context).brightness == Brightness.dark;
    return CustomPaint(
      size: Size.square(size),
      painter: _CompassPainter(tile: tile, mono: monochrome, dark: onDark && !tile),
    );
  }
}

class _CompassPainter extends CustomPainter {
  _CompassPainter({required this.tile, required this.mono, required this.dark});
  final bool tile;
  final Color? mono;
  final bool dark;

  // viewBox 0 0 256 256 — identical numbers to gen_logo.py
  static const Offset _h = Offset(128, 102);
  static final double _theta = 24 * math.pi / 180;
  static const double _leg = 138;
  static const double _stalk = 60;
  static const double _w = 20;
  static const double _knob = 30;
  static const double _hinge = 36;
  static const double _leadLen = 42; // > _chamfer * _w, so the lead contains the chamfer
  static const double _chamfer = 1.5;

  static final Offset _dr = Offset(math.sin(_theta), math.cos(_theta));
  static final Offset _dl = Offset(-math.sin(_theta), math.cos(_theta));
  static final Offset _pencilTip = _h + _dr * _leg;
  static final Offset _needleTip = _h + _dl * _leg;
  static final Offset _knobC = _h - _dr * _stalk;

  static Path _bar(Offset a, Offset b, double w, {bool tipAtB = false}) {
    final d = (b - a) / (b - a).distance;
    final n = Offset(-d.dy, d.dx);
    final p = Path()..moveTo((a + n * (w / 2)).dx, (a + n * (w / 2)).dy);
    if (tipAtB) {
      final c = b - d * (_chamfer * w);
      p
        ..lineTo((c + n * (w / 2)).dx, (c + n * (w / 2)).dy)
        ..lineTo(b.dx, b.dy)
        ..lineTo((c - n * (w / 2)).dx, (c - n * (w / 2)).dy);
    } else {
      p
        ..lineTo((b + n * (w / 2)).dx, (b + n * (w / 2)).dy)
        ..lineTo((b - n * (w / 2)).dx, (b - n * (w / 2)).dy);
    }
    return p
      ..lineTo((a - n * (w / 2)).dx, (a - n * (w / 2)).dy)
      ..close();
  }

  /// Square centred at [c] with one axis along [d].
  static Path _square(Offset c, double side, Offset d) {
    final n = Offset(-d.dy, d.dx);
    final h = side / 2;
    final corners = [c + d * h + n * h, c + d * h - n * h, c - d * h - n * h, c - d * h + n * h];
    final p = Path()..moveTo(corners[0].dx, corners[0].dy);
    for (final k in corners.skip(1)) {
      p.lineTo(k.dx, k.dy);
    }
    return p..close();
  }

  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / 256);
    final body = mono ?? (dark ? BrandColors.white : BrandColors.black);
    final red = mono ?? BrandColors.red;
    final pin = dark ? BrandColors.darkPin : BrandColors.cream;
    final fill = Paint()..style = PaintingStyle.fill;

    if (tile) {
      canvas.drawRRect(
        RRect.fromRectAndRadius(const Rect.fromLTWH(0, 0, 256, 256), const Radius.circular(56)),
        fill..color = BrandColors.cream,
      );
    }
    // one black chamfered bar, the red lead painted over its end: no seams
    final leadStart = _pencilTip + (_h - _pencilTip) * (_leadLen / _leg);
    canvas.drawPath(_bar(_knobC, _pencilTip, _w, tipAtB: true), fill..color = body);
    canvas.drawPath(_bar(leadStart, _pencilTip, _w, tipAtB: true), fill..color = red);
    canvas.drawPath(_bar(_h, _needleTip, _w, tipAtB: true), fill..color = body);
    canvas.drawPath(_square(_knobC, _knob, _dr), fill..color = body);
    canvas.drawPath(_square(_h, _hinge, _dr), fill..color = red);
    if (mono == null) canvas.drawPath(_square(_h, 10, _dr), fill..color = pin);
  }

  @override
  bool shouldRepaint(_CompassPainter old) =>
      old.tile != tile || old.mono != mono || old.dark != dark;
}
