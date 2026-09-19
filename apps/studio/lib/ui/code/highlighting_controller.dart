/// One text controller for every BDL text field: the Code view's file
/// and the definition editor's formula.  It draws the IDE service's
/// spans through the [SyntaxTheme] and the diagnostics' wavy marks on
/// top; it classifies nothing.  While its text differs from the text the
/// spans are over (the designer is typing, the answer is on its way) it
/// shifts the spans of the unchanged prefix and suffix and leaves the
/// changed middle plain — never a stale span over new text, never a
/// flash to plain and back.
library;

import 'package:flutter/foundation.dart' show listEquals;
import 'package:flutter/material.dart';

import '../../app/highlighting.dart' show shiftSpans;
import '../../app/state.dart';
import 'syntax_theme.dart';

/// How long typing pauses before a text's tokens are asked for again;
/// shorter than a source send, so the colours settle before the graph
/// moves.  Between, the spans on show are shifted.
const Duration kHighlightPause = Duration(milliseconds: 150);

class HighlightingController extends TextEditingController {
  HighlightingController({super.text});

  /// The spans on show and the text they are over.
  HighlightState? _highlight;

  /// Diagnostic marks in code units of the current text, with their colour.
  List<(TextRange, Color)> marks = const [];

  SyntaxTheme? theme;

  /// A new answer (or none: the document changed).  A rebuild follows when
  /// the spans differ.
  void setHighlight(HighlightState? h) {
    if (identical(h, _highlight)) return;
    final same =
        h != null &&
        _highlight != null &&
        h.text == _highlight!.text &&
        listEquals(h.spans, _highlight!.spans);
    _highlight = h;
    if (!same) notifyListeners();
  }

  /// The spans as they apply to the text on screen now.
  List<HighlightSpan> get spans {
    final h = _highlight;
    if (h == null || h.spans.isEmpty) return const [];
    return shiftSpans(h.text, text, h.spans);
  }

  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    final text = this.text;
    final base = style ?? const TextStyle();
    final theme = this.theme;
    // Cut points: every span and mark boundary, in order.
    final spans = theme == null ? const <HighlightSpan>[] : this.spans;
    final marks = this.marks.where((m) => !m.$1.isCollapsed && m.$1.end <= text.length).toList();
    if (spans.isEmpty && marks.isEmpty) {
      return super.buildTextSpan(context: context, style: style, withComposing: withComposing);
    }
    final cuts = <int>{0, text.length};
    for (final s in spans) {
      cuts.addAll([s.start, s.end]);
    }
    for (final m in marks) {
      cuts.addAll([m.$1.start, m.$1.end]);
    }
    final points = cuts.where((c) => c >= 0 && c <= text.length).toList()..sort();
    final children = <TextSpan>[];
    var si = 0;
    for (var i = 0; i + 1 < points.length; i++) {
      final a = points[i];
      final b = points[i + 1];
      if (b <= a) continue;
      while (si < spans.length && spans[si].end <= a) {
        si++;
      }
      var piece = base;
      if (si < spans.length && spans[si].start <= a && spans[si].end >= b) {
        piece = theme!.styleOf(spans[si], base);
      }
      for (final (range, color) in marks) {
        if (range.start <= a && range.end >= b) {
          piece = piece.copyWith(
            decoration: TextDecoration.underline,
            decorationStyle: TextDecorationStyle.wavy,
            decorationColor: color,
            decorationThickness: 1.5,
          );
          break;
        }
      }
      children.add(TextSpan(text: text.substring(a, b), style: piece));
    }
    return TextSpan(style: style, children: children);
  }
}
