/// Semantic highlighting transitions (`docs/architecture/syntax-highlighting.md`).
///
/// The IDE service classifies; Studio colours.  Every request carries a
/// generation per document and only the latest answer is applied, so an
/// answer to an older text never lands on a newer one.  The spans are
/// kept by text, not by revision: a pushed projection does not drop them,
/// and while the designer types the editor shifts the spans of the
/// unchanged prefix and suffix ([shiftSpans]) and shows the changed middle
/// plain until the service answers again.  No keyword, name or unit
/// table exists here; no text is tokenized in Dart.
library;

import 'dart:convert';
import 'dart:math' as math;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition;
import 'state.dart';

Transition semanticTokensRequested(
  AppState s, {
  required String? path,
  required int? mappingId,
  required int? component,
  required String text,
}) {
  if (s.project == null) return Transition(s);
  final key = path != null
      ? HighlightState.fileKey(path)
      : HighlightState.formulaKey(mappingId ?? -1, component);
  if (path == null && mappingId == null) return Transition(s);
  final held = s.editor.highlights[key];
  // The same text again: the answer on show (or in flight) is the answer.
  if (held != null && held.requested == text && (held.pending || held.text == text)) {
    return Transition(s);
  }
  final generation = (held?.generation ?? 0) + 1;
  final next =
      held?.copyWith(generation: generation, requested: text, pending: true) ??
      HighlightState(key: key, generation: generation, requested: text, pending: true);
  return Transition(
    s.copyWith(editor: s.editor.copyWith(highlights: {...s.editor.highlights, key: next})),
    [
      FetchSemanticTokens(
        key: key,
        revision: s.revision,
        generation: generation,
        text: text,
        path: path,
        mappingId: mappingId,
        component: component,
      ),
    ],
  );
}

/// The answer to the latest request lands; any other is dropped.  The
/// service's byte ranges become code-unit ranges of the requested text
/// once, here, and the legend's indices become names.
Transition semanticTokensReceived(AppState s, String key, pb.SemanticTokensResponse r) {
  final held = s.editor.highlights[key];
  if (held == null || r.generation.toInt() != held.generation) return Transition(s);
  final text = held.requested;
  if (r.textLen != utf8.encode(text).length) {
    // not the text that was asked about: keep what is on show
    return Transition(
      s.copyWith(
        editor: s.editor.copyWith(
          highlights: {...s.editor.highlights, key: held.copyWith(pending: false)},
        ),
      ),
    );
  }
  final spans = spansOf(text, r);
  final next = held.copyWith(
    text: text,
    spans: spans,
    legendVersion: r.legend.version,
    pending: false,
  );
  return Transition(
    s.copyWith(editor: s.editor.copyWith(highlights: {...s.editor.highlights, key: next})),
  );
}

Transition semanticTokensFailed(AppState s, String key, int generation) {
  final held = s.editor.highlights[key];
  if (held == null || generation != held.generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        highlights: {...s.editor.highlights, key: held.copyWith(pending: false)},
      ),
    ),
  );
}

/// The service's tokens as spans over [text]: byte offsets to code units
/// in one pass (the tokens are sorted), indices to the legend's names.
List<HighlightSpan> spansOf(String text, pb.SemanticTokensResponse r) {
  final legend = r.legend;
  final bytes = utf8.encode(text);
  final out = <HighlightSpan>[];
  var byte = 0;
  var unit = 0;
  int unitOf(int target) {
    // advance from the last position: tokens come in text order
    if (target < byte) {
      byte = 0;
      unit = 0;
    }
    while (byte < target && byte < bytes.length) {
      final b = bytes[byte];
      final width = b < 0x80
          ? 1
          : b < 0xE0
          ? 2
          : b < 0xF0
          ? 3
          : 4;
      unit += width == 4 ? 2 : 1;
      byte += width;
    }
    return unit;
  }

  for (final t in r.tokens) {
    if (t.tokenType >= legend.types.length) continue;
    final start = unitOf(t.start);
    final end = unitOf(t.end);
    if (end <= start || end > text.length) continue;
    final mods = <String>{
      for (var i = 0; i < legend.modifiers.length && i < 32; i++)
        if (t.tokenModifiers & (1 << i) != 0) legend.modifiers[i],
    };
    out.add(HighlightSpan(start, end, legend.types[t.tokenType], mods));
  }
  return out;
}

/// The spans of [from] carried onto [to] without reclassifying anything:
/// those strictly inside the unchanged prefix stay, those strictly inside
/// the unchanged suffix move by the change in length, those touching the
/// changed middle go — a span that ends or starts exactly at the edit
/// point may have grown into a different word, so it goes too.  A stale
/// span is never drawn over text it was not computed for.
List<HighlightSpan> shiftSpans(String from, String to, List<HighlightSpan> spans) {
  if (from == to) return spans;
  final prefix = _commonPrefix(from, to);
  final suffix = math.min(_commonSuffix(from, to), math.min(from.length, to.length) - prefix);
  final delta = to.length - from.length;
  final fromSuffixStart = from.length - suffix;
  final out = <HighlightSpan>[];
  for (final s in spans) {
    if (s.end < prefix) {
      out.add(s);
    } else if (s.start > fromSuffixStart) {
      out.add(s.shifted(delta));
    }
  }
  return out;
}

int _commonPrefix(String a, String b) {
  final n = math.min(a.length, b.length);
  var i = 0;
  while (i < n && a.codeUnitAt(i) == b.codeUnitAt(i)) {
    i++;
  }
  return i;
}

int _commonSuffix(String a, String b) {
  final n = math.min(a.length, b.length);
  var i = 0;
  while (i < n && a.codeUnitAt(a.length - 1 - i) == b.codeUnitAt(b.length - 1 - i)) {
    i++;
  }
  return i;
}
