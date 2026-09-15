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
  static const Offset _h = Offset(128, 102);
  static final double _theta = 24 * math.pi / 180;
  static const double _leg = 138;
  static const double _stalk = 60;
  static const double _w = 20;
  static const double _knob = 30;
  static const double _hinge = 36;
  static const double _lead = 0.80;
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
    final black = mono ?? BrandColors.black;
    final red = mono ?? BrandColors.red;
    final fill = Paint()..style = PaintingStyle.fill;

    if (tile) {
      canvas.drawRRect(
        RRect.fromRectAndRadius(const Rect.fromLTWH(0, 0, 256, 256), const Radius.circular(56)),
        fill..color = BrandColors.cream,
      );
    }
    final leadStart = _h + (_pencilTip - _h) * _lead;
    canvas.drawPath(_bar(_knobC, leadStart, _w), fill..color = black);
    canvas.drawPath(_bar(leadStart, _pencilTip, _w, tipAtB: true), fill..color = red);
    canvas.drawPath(_bar(_h, _needleTip, _w, tipAtB: true), fill..color = black);
    canvas.drawPath(_square(_knobC, _knob, _dr), fill..color = black);
    canvas.drawPath(_square(_h, _hinge, _dr), fill..color = red);
    if (mono == null) canvas.drawPath(_square(_h, 10, _dr), fill..color = BrandColors.cream);
  }

  @override
  bool shouldRepaint(_CompassPainter old) => old.tile != tile || old.mono != mono;
}
