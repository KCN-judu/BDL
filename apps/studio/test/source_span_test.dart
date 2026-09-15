import 'package:bdl_studio/ui/source_span.dart';
import 'package:flutter/painting.dart' show TextRange;
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('ASCII spans map one to one', () {
    expect(codeUnitRange('Tilt / 90 deg', 7, 9), const TextRange(start: 7, end: 9));
    expect(excerptOf('Tilt / 90 deg', 7, 9), '90');
    expect(excerptOf('Tilt / 90 deg', 0, 13), 'Tilt / 90 deg');
  });

  test('multi-byte characters shift code units without splitting them', () {
    // `×` is 2 bytes, 1 code unit; `°` is 2 bytes, 1 code unit.
    const src = 'Tilt × 90°';
    // bytes: T0 i1 l2 t3 ' '4 ×5-6 ' '7 98 09 °10-11  → 12 bytes, 10 code units
    expect(codeUnitRange(src, 5, 7), const TextRange(start: 5, end: 6));
    expect(excerptOf(src, 5, 7), '×');
    expect(codeUnitRange(src, 8, 12), const TextRange(start: 7, end: 10));
    expect(excerptOf(src, 8, 12), '90°');
    // an astral character is 4 bytes and 2 code units
    const emoji = 'a😀b';
    expect(codeUnitRange(emoji, 1, 5), const TextRange(start: 1, end: 3));
    expect(excerptOf(emoji, 5, 6), 'b');
  });

  test('spans past the end collapse at the end; inverted spans collapse', () {
    expect(codeUnitRange('ab', 2, 2), const TextRange(start: 2, end: 2));
    expect(codeUnitRange('ab', 5, 9), const TextRange(start: 2, end: 2));
    expect(codeUnitRange('ab', 1, 0), const TextRange(start: 1, end: 1));
    expect(excerptOf('ab', 2, 2), '');
    expect(excerptOf('', 0, 3), '');
  });
}
