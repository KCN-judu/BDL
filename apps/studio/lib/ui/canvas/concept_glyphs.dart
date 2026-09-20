/// The marks that stand for a concept and a mapping outside the canvas —
/// library rows, chips, toggles, pop-up items — painted by the same code as
/// the canvas sockets so identity (hue) and value form (shape) are learned
/// once and recognised everywhere (docs/architecture/studio-ui.md §7).
library;

import 'package:flutter/material.dart';

import '../../l10n/l10n.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../mac/tokens.dart';
import 'canvas_geometry.dart';
import 'node_canvas.dart' show NodePainter;

/// A concept's socket at row size: its hue, its shape, hollow while the
/// value form is undecided.
class SocketGlyph extends StatelessWidget {
  const SocketGlyph({super.key, required this.kind, required this.color, this.size = 12});

  SocketGlyph.of(pb.ConceptView concept, MacTokens t, {super.key, this.size = 12})
    : kind = socketKind(concept),
      color = t.conceptColor(concept.id.toInt());

  final SocketKind kind;
  final Color color;
  final double size;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Semantics(
      label: switch (kind) {
        SocketKind.open => context.l10n.valueNotDecided,
        SocketKind.quantity => 'quantity',
        SocketKind.onOff => context.l10n.onOrOff,
        SocketKind.count => 'count',
        SocketKind.collection => 'collection',
        SocketKind.grouped => context.l10n.groupedValue,
        SocketKind.optional => context.l10n.optionalValue,
      },
      child: CustomPaint(size: Size.square(size), painter: _SocketGlyphPainter(t, kind, color)),
    );
  }
}

class _SocketGlyphPainter extends CustomPainter {
  _SocketGlyphPainter(this.t, this.kind, this.color);
  final MacTokens t;
  final SocketKind kind;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final c = size.center(Offset.zero);
    final r = size.shortestSide / 2 - 1;
    final path = NodePainter.socketPath(c, kind, r);
    if (kind != SocketKind.open) canvas.drawPath(path, Paint()..color = color);
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = kind == SocketKind.open ? color : t.content.withValues(alpha: 0.9),
    );
  }

  @override
  bool shouldRepaint(_SocketGlyphPainter old) =>
      old.kind != kind || old.color != color || old.t != t;
}

/// A mapping at row size: the node's silhouette — dashed while declared,
/// with the red mark when its definition does not check; a Source wears
/// the canvas's Source header and its left boundary bar (ADR-0032).
class MappingGlyph extends StatelessWidget {
  const MappingGlyph({
    super.key,
    required this.declared,
    required this.wrong,
    this.source = false,
    this.size = 12,
  });
  final bool declared;
  final bool wrong;
  final bool source;
  final double size;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Semantics(
      label: source
          ? context.l10n.roleSource
          : declared
          ? context.l10n.declaredNotYetDefined
          : wrong
          ? context.l10n.definitionDoesNotCheck
          : 'defined',
      child: CustomPaint(
        size: Size(size + 2, size),
        painter: _MappingGlyphPainter(
          t,
          declared: declared && !source,
          wrong: wrong,
          source: source,
        ),
      ),
    );
  }
}

class _MappingGlyphPainter extends CustomPainter {
  _MappingGlyphPainter(this.t, {required this.declared, required this.wrong, this.source = false});
  final MacTokens t;
  final bool declared;
  final bool wrong;
  final bool source;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0.75, 0.75, size.width - 1.5, size.height - 1.5),
      const Radius.circular(2.5),
    );
    final header = source
        ? (t.isDark ? const Color(0xFF2F5A4A) : const Color(0xFFD2ECDD))
        : (t.isDark ? const Color(0xFF2E4A6B) : const Color(0xFFCFE0F5));
    canvas.save();
    canvas.clipRRect(rect);
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, size.height * 0.45), Paint()..color = header);
    canvas.restore();
    if (source) {
      canvas.drawLine(
        Offset(1.25, 2.5),
        Offset(1.25, size.height - 2.5),
        Paint()
          ..color = t.textSecondary
          ..strokeWidth = 2
          ..strokeCap = StrokeCap.round,
      );
    }
    final outline = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = declared ? t.textTertiary : t.textSecondary;
    if (declared) {
      final path = Path()..addRRect(rect);
      for (final m in path.computeMetrics()) {
        var d = 0.0;
        while (d < m.length) {
          canvas.drawPath(m.extractPath(d, d + 2.5), outline);
          d += 4.5;
        }
      }
    } else {
      canvas.drawRRect(rect, outline);
    }
    if (wrong) {
      canvas.drawCircle(
        Offset(size.width * 0.3, size.height * 0.74),
        1.6,
        Paint()..color = t.error,
      );
    }
  }

  @override
  bool shouldRepaint(_MappingGlyphPainter old) =>
      old.declared != declared || old.wrong != wrong || old.source != source || old.t != t;
}

/// A concept as a removable chip (a mapping's Reads): the socket glyph and
/// the name.  Filled in the concept's hue means *member of this signature*.
class ConceptChip extends StatelessWidget {
  const ConceptChip({super.key, required this.concept, this.onRemove});
  final pb.ConceptView concept;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      height: MacMetrics.controlHeight,
      padding: const EdgeInsets.only(left: 6, right: 2),
      decoration: BoxDecoration(
        color: t.control,
        borderRadius: BorderRadius.circular(5),
        border: Border.all(color: t.hairline),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 5,
        children: [
          SocketGlyph.of(concept, t, size: 11),
          Text(
            concept.name,
            style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
          ),
          if (onRemove != null)
            IconButton(
              icon: const Icon(Icons.close, size: 12),
              onPressed: onRemove,
              constraints: const BoxConstraints.tightFor(width: 18, height: 18),
              padding: EdgeInsets.zero,
              tooltip: 'Stop reading ${concept.name}',
            ),
        ],
      ),
    );
  }
}

/// A physical output in a list: the sink's boundary bar and its state —
/// dashed while open or undriven, solid when driven, a red mark when
/// contested or ill-formed.
class OutputGlyph extends StatelessWidget {
  const OutputGlyph({super.key, required this.state, required this.open, this.size = 12});
  final pb.OutputState? state;
  final bool open;
  final double size;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final wrong =
        state == pb.OutputState.OUTPUT_STATE_CONFLICT ||
        state == pb.OutputState.OUTPUT_STATE_ILL_FORMED;
    final incomplete = open || state == null || state == pb.OutputState.OUTPUT_STATE_UNDRIVEN;
    return SizedBox(
      width: size + 4,
      height: size,
      child: CustomPaint(
        painter: _OutputGlyphPainter(
          border: wrong ? t.error : (incomplete ? t.textTertiary : t.textSecondary),
          bar: wrong ? t.error : t.textSecondary,
          dashed: incomplete && !wrong,
        ),
      ),
    );
  }
}

class _OutputGlyphPainter extends CustomPainter {
  _OutputGlyphPainter({required this.border, required this.bar, required this.dashed});
  final Color border;
  final Color bar;
  final bool dashed;

  @override
  void paint(Canvas canvas, Size size) {
    final r = RRect.fromRectAndRadius(
      Rect.fromLTWH(0.5, 0.5, size.width - 3, size.height - 1),
      const Radius.circular(2),
    );
    final outline = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = border;
    if (dashed) {
      final path = Path()..addRRect(r);
      for (final m in path.computeMetrics()) {
        var d = 0.0;
        while (d < m.length) {
          canvas.drawPath(m.extractPath(d, d + 2), outline);
          d += 4;
        }
      }
    } else {
      canvas.drawRRect(r, outline);
    }
    canvas.drawLine(
      Offset(size.width - 1, 1),
      Offset(size.width - 1, size.height - 1),
      Paint()
        ..color = bar
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(_OutputGlyphPainter old) =>
      old.border != border || old.bar != bar || old.dashed != dashed;
}
