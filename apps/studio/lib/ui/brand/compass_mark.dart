/// The BDL language mark, painted from the same geometry as
/// `assets/brand/gen_logo.py`: a drafting compass reduced to straight
/// lines — an upright head, one vertical bar (stalk and needle leg), one
/// diagonal (pencil leg), parallel edges, chamfered tips.  The λ is what
/// remains.  Vector code, crisp at any size, no SVG runtime.
library;

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

  /// Draw the cream tile behind the compass (the app-icon variant).
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
  static const double _x = 150;
  static const double _w = 20;
  static const Rect _knob = Rect.fromLTWH(_x - 15, 18, 30, 30);
  static const Offset _hinge = Offset(_x, 104);
  static const double _hingeSide = 36;
  static const Offset _needleTip = Offset(_x, 242);
  static const Offset _pencilTip = Offset(52, 232);
  static const double _lead = 0.80;
  static const double _chamfer = 1.5;

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

  @override
  void paint(Canvas canvas, Size size) {
    canvas.scale(size.width / 256);
    final black = mono ?? BrandColors.black;
    final red = mono ?? BrandColors.red;
    final fill = Paint()..style = PaintingStyle.fill;

    if (tile) {
      canvas.drawRRect(
        RRect.fromRectAndRadius(const Rect.fromLTWH(0, 0, 256, 256), const Radius.circular(56)),
        fill..color = BrandColors.cream,
      );
    }

    canvas.drawPath(_bar(Offset(_x, _knob.top), _needleTip, _w, tipAtB: true), fill..color = black);
    canvas.drawRect(_knob, fill..color = black);
    final leadStart = _hinge + (_pencilTip - _hinge) * _lead;
    canvas.drawPath(_bar(_hinge, leadStart, _w - 2), fill..color = black);
    canvas.drawPath(_bar(leadStart, _pencilTip, _w - 2, tipAtB: true), fill..color = red);
    canvas.drawRect(
      Rect.fromCenter(center: _hinge, width: _hingeSide, height: _hingeSide),
      fill..color = red,
    );
    if (mono == null) {
      canvas.drawRect(
        Rect.fromCenter(center: _hinge, width: 10, height: 10),
        fill..color = BrandColors.cream,
      );
    }
  }

  @override
  bool shouldRepaint(_CompassPainter old) => old.tile != tile || old.mono != mono;
}
