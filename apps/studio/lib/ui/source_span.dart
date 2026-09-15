/// Compiler spans are UTF-8 byte ranges into the formula source; Flutter
/// text is UTF-16 code units.  This is the one place the two meet.
library;

import 'dart:convert' show utf8;

import 'package:flutter/painting.dart' show TextRange;

/// The code-unit range of the byte span `[byteStart, byteEnd)` in [source].
/// Offsets are clamped to the text and never split a character; a span
/// beyond the end collapses at the end (a parser's "expected more here").
TextRange codeUnitRange(String source, int byteStart, int byteEnd) {
  var bytes = 0;
  var units = 0;
  int? start;
  int? end;
  for (final rune in source.runes) {
    if (start == null && bytes >= byteStart) start = units;
    if (end == null && bytes >= byteEnd) end = units;
    if (start != null && end != null) break;
    bytes += utf8.encode(String.fromCharCode(rune)).length;
    units += rune > 0xFFFF ? 2 : 1;
  }
  start ??= units;
  end ??= units;
  if (end < start) end = start;
  return TextRange(start: start, end: end);
}

/// The text a byte span covers, `''` when it covers nothing.
String excerptOf(String source, int byteStart, int byteEnd) {
  final r = codeUnitRange(source, byteStart, byteEnd);
  return r.isCollapsed ? '' : source.substring(r.start, r.end);
}

/// The byte offset of code-unit offset [codeUnit] in [source] (clamped).
int byteOffsetOf(String source, int codeUnit) {
  final end = codeUnit.clamp(0, source.length);
  return utf8.encode(source.substring(0, end)).length;
}
