/// The BDL language mark, painted from the same geometry as
/// `assets/brand/gen_logo.py`: a constructivist drafting compass whose
/// stalk, pencil leg and needle leg form a λ.  Kept as vector code so it is
/// crisp at any size and needs no SVG runtime.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';

abstract final class BrandColors {
  static const black = Color(0xFF141414);
  static const red = Color(0xFFE5322D);
  static const cream = Color(0xFFF2EBDD);
  static const blue = Color(0xFF2B5DD1);
}

class CompassMark extends StatelessWidget {
  const CompassMark({super.key, this.size = 64, this.tile = false, this.monochrome});

  final double size;

  /// Draw the cream tile with the pale red disc behind the compass (the
  /// app-icon variant).
  final bool tile;

  /// Single-colour variant (for toolbars); ignores the palette.
  final Color? monochrome;

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      size: Size.square(size),
      painter: _CompassPainter(tile: tile, mono: monochrome),
    );
  }
}

class _CompassPainter extends CustomPainter {
  _CompassPainter({required this.tile, required this.mono});
  final bool tile;
  final Color? mono;

  // viewBox 0 0 256 256 — identical numbers to gen_logo.py
  static const _t = Offset(92, 22);
  static const _b = Offset(192, 238);
  static const _l = Offset(34, 232);

  static Offset _lerp(Offset a, Offset b, double t) => a + (b - a) * t;

  static Path _tapered(Offset a, Offset b, double wa, double wb) {
    final d = (b - a) / (b - a).distance;
    final n = Offset(-d.dy, d.dx);
    return Path()
      ..moveTo((a + n * (wa / 2)).dx, (a + n * (wa / 2)).dy)
      ..lineTo((b + n * (wb / 2)).dx, (b + n * (wb / 2)).dy)
      ..lineTo((b - n * (wb / 2)).dx, (b - n * (wb / 2)).dy)
      ..lineTo((a - n * (wa / 2)).dx, (a - n * (wa / 2)).dy)
      ..close();
  }

  @override
  void paint(Canvas canvas, Size size) {
    final s = size.width / 256;
    canvas.scale(s);
    final black = mono ?? BrandColors.black;
    final red = mono ?? BrandColors.red;
    final blue = mono ?? BrandColors.blue;

    // hinge projected onto the long stroke T→B so the λ is one straight line
    final d = (_b - _t) / (_b - _t).distance;
    const hRaw = Offset(116, 104);
    final h = _t + d * ((hRaw - _t).dx * d.dx + (hRaw - _t).dy * d.dy);
    final leadStart = _lerp(h, _b, 0.82);
    final needleStart = _lerp(h, _l, 0.86);

    if (tile) {
      canvas.drawRRect(
        RRect.fromRectAndRadius(const Rect.fromLTWH(0, 0, 256, 256), const Radius.circular(56)),
        Paint()..color = BrandColors.cream,
      );
      canvas.drawCircle(h, 78, Paint()..color = BrandColors.red.withValues(alpha: 0.14));
    }

    // the circle the pencil has just begun
    final r = (_b - _l).distance;
    final a0 = math.atan2(_b.dy - _l.dy, _b.dx - _l.dx) - 5 * math.pi / 180;
    final arc = Path()..addArc(Rect.fromCircle(center: _l, radius: r), a0, -28 * math.pi / 180);
    canvas.drawPath(
      arc,
      Paint()
        ..color = blue
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4.5
        ..strokeCap = StrokeCap.round,
    );

    final fill = Paint()..style = PaintingStyle.fill;
    canvas.drawPath(_tapered(_t, h, 11, 24), fill..color = black);
    canvas.drawPath(_tapered(h, leadStart, 24, 9), fill..color = black);
    canvas.drawPath(_tapered(leadStart, _b, 9, 2), fill..color = red);
    canvas.drawPath(_tapered(h, needleStart, 22, 6), fill..color = black);
    canvas.drawPath(_tapered(needleStart, _l, 6, 1), fill..color = black);
    canvas.drawCircle(h, 19, fill..color = red);
    if (mono == null) canvas.drawCircle(h, 6, fill..color = BrandColors.cream);
    canvas.drawCircle(_t, 12, fill..color = black);
  }

  @override
  bool shouldRepaint(_CompassPainter old) => old.tile != tile || old.mono != mono;
}
