/// The Studio hero: the wordmark in Chakra Petch (square sans, 45° chamfers)
/// over a field of 45°-routed traces — the way a PCB is routed and the way
/// the paper draws behaviour flowing left to right.  Pure painting; the
/// trace field is deterministic (seeded), so it is the same on every launch.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../brand/compass_mark.dart';
import '../mac/tokens.dart';

const String kHeroFont = 'ChakraPetch';

class HeroMark extends StatelessWidget {
  const HeroMark({super.key, required this.version});
  final String version;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return ClipRRect(
      borderRadius: BorderRadius.circular(10),
      child: Stack(
        fit: StackFit.expand,
        children: [
          CustomPaint(painter: _TraceFieldPainter(t)),
          // The lock-up is authored at one size and scaled uniformly to the
          // space available, so the wordmark never wraps and the mark keeps
          // its splash proportion at every window size.
          LayoutBuilder(
            builder: (context, c) => Padding(
              padding: const EdgeInsets.fromLTRB(28, 28, 28, 26),
              child: Align(
                alignment: Alignment.bottomLeft,
                // The lock-up may take 85 % of the width and 50 % of the height;
                // it scales both ways within that box.
                child: SizedBox(
                  width: (c.maxWidth - 56) * 0.85,
                  height: (c.maxHeight - 54) * 0.5,
                  child: FittedBox(
                    fit: BoxFit.contain,
                    alignment: Alignment.bottomLeft,
                    child: _LockUp(version: version, tokens: t),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Mark + wordmark + tagline at their designed size (about 470 × 170).
class _LockUp extends StatelessWidget {
  const _LockUp({required this.version, required this.tokens});
  final String version;
  final MacTokens tokens;

  @override
  Widget build(BuildContext context) {
    final t = tokens;
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        const CompassMark(size: 168),
        const SizedBox(width: 22),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              'Behavior\nDesigner',
              softWrap: false,
              style: TextStyle(
                fontFamily: kHeroFont,
                fontSize: 50,
                height: 1.0,
                fontWeight: FontWeight.w700,
                letterSpacing: -0.5,
                color: t.textPrimary,
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'Product behavior as a design material.',
              softWrap: false,
              style: TextStyle(fontSize: 13, color: t.textSecondary),
            ),
            const SizedBox(height: 4),
            Text(
              'Studio $version',
              style: TextStyle(
                fontFamily: kHeroFont,
                fontSize: 11,
                letterSpacing: 1,
                color: t.textTertiary,
              ),
            ),
          ],
        ),
      ],
    );
  }
}

/// Traces routed on a grid with 45° bends, fading toward the wordmark.
class _TraceFieldPainter extends CustomPainter {
  _TraceFieldPainter(this.t);
  final MacTokens t;

  @override
  void paint(Canvas canvas, Size size) {
    final bg = Paint()
      ..shader = LinearGradient(
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
        colors: t.isDark
            ? [const Color(0xFF1B2230), const Color(0xFF15181E)]
            : [const Color(0xFFE9EEF6), const Color(0xFFF6F7FA)],
      ).createShader(Offset.zero & size);
    canvas.drawRect(Offset.zero & size, bg);

    const grid = 18.0;
    final rnd = math.Random(20260915);
    final base = t.accent;
    for (var i = 0; i < 26; i++) {
      final path = Path();
      var x = (rnd.nextInt((size.width / grid).ceil() + 8) - 4) * grid;
      var y = -grid * rnd.nextInt(6);
      path.moveTo(x, y);
      // Each trace runs downward, turning by ±45° for a while then straight.
      var dirX = 0.0;
      for (var s = 0; s < 40 && y < size.height + grid; s++) {
        final turn = rnd.nextInt(5);
        if (turn == 0) dirX = -1;
        if (turn == 1) dirX = 1;
        if (turn >= 2 && rnd.nextInt(3) == 0) dirX = 0;
        final len = grid * (1 + rnd.nextInt(3));
        x += dirX * len;
        y += len;
        path.lineTo(x, y);
      }
      // Fade with the trace's horizontal position so the left, where the
      // wordmark sits, stays quiet.
      final alpha = (0.05 + 0.22 * (x / size.width).clamp(0.0, 1.0)) * (t.isDark ? 1.0 : 0.8);
      canvas.drawPath(
        path,
        Paint()
          ..color = base.withValues(alpha: alpha)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.5
          ..strokeJoin = StrokeJoin.miter,
      );
      // A pad at the end of the trace.
      canvas.drawCircle(Offset(x, y), 2.5, Paint()..color = base.withValues(alpha: alpha + 0.1));
    }
  }

  @override
  bool shouldRepaint(_TraceFieldPainter old) => old.t != t;
}
