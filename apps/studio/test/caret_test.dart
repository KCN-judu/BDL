/// The structural caret model (`app/caret.dart`): the stops a formula
/// has — the compiler's carets with the character positions inside each
/// text leaf — and what each key does at one: a text edit, a structured
/// action, a move, or nothing.  Pure functions over a scripted projection;
/// the carets are derived as the compiler derives them (`support/carets`).
library;

import 'package:bdl_studio/app/caret.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:flutter_test/flutter_test.dart';

pb.FormulaNode node(
  String id,
  String kind,
  String text,
  int start,
  int end, {
  String name = '',
  List<pb.FormulaNode> children = const [],
  String coordinate = '',
  String unit = '',
  bool equation = false,
}) => pb.FormulaNode(
  id: id,
  kind: kind,
  text: text,
  name: name,
  range: pb.SourceSpan(start: start, end: end),
  children: children,
  coordinate: coordinate,
  unit: unit,
  equation: equation,
);

/// `clamp(Tilt / (90 deg), ?, 1)` as the compiler projects it, with carets.
pb.FormulaProjection clampOf() => pb.FormulaProjection(
  source: 'clamp(Tilt / (90 deg), ?, 1)',
  parseOk: true,
  slots: ['r.1'],
  root: node(
    'r',
    'call',
    'clamp(Tilt / (90 deg), ?, 1)',
    0,
    28,
    name: 'clamp',
    equation: true,
    children: [
      node(
        'r.0',
        'binary',
        'Tilt / (90 deg)',
        6,
        21,
        name: '/',
        children: [
          node('r.0.0', 'reference', 'Tilt', 6, 10, name: 'Tilt'),
          node('r.0.1', 'quantity', '(90 deg)', 13, 21, coordinate: '90', unit: 'deg'),
        ],
      ),
      node('r.1', 'slot', '?', 23, 24),
      node('r.2', 'number', '1', 26, 27, coordinate: '1'),
    ],
  ),
);

String ids(Iterable<CaretStop> s) => s.map((x) => x.id).join(' ');

void main() {
  group('stops', () {
    test('the compiler\'s carets, with the character positions inside each text leaf', () {
      final p = clampOf();
      final stops = caretStops(p);
      expect(
        ids(stops),
        'r:before '
        'r.0:before r.0.0:before r.0.0@1 r.0.0@2 r.0.0@3 r.0.0:after '
        'r.0.1:before r.0.1:open r.0.1@1 r.0.1@2 r.0.1@3 r.0.1@4 r.0.1@5 r.0.1:close r.0.1:after r.0:after '
        'r.1:in '
        'r.2:before r.2:after '
        'r:after',
      );
      // offsets never decrease; a position inside a leaf is strictly inside
      for (var i = 1; i < stops.length; i++) {
        expect(stops[i].offset, greaterThanOrEqualTo(stops[i - 1].offset));
      }
      expect(stops.firstWhere((s) => s.id == 'r.0.0@2').offset, 8);
      expect(stops.firstWhere((s) => s.id == 'r.0.1@1').offset, 15, reason: 'after the paren');
      // a slot is one stop; a structure has no character positions
      expect(stops.where((s) => s.node == 'r.1').map((s) => s.side), [StopSide.inSlot]);
      expect(stops.where((s) => s.node == 'r.0').map((s) => s.side), [
        StopSide.before,
        StopSide.after,
      ]);
      // no tree: no stops
      expect(caretStops(pb.FormulaProjection(source: '', parseOk: true)), isEmpty);
      expect(caretStops(null), isEmpty);
    });

    test('a caret state resolves by id, else by offset — a slot first, then after a part', () {
      final stops = caretStops(clampOf());
      expect(resolveStop(stops, const CaretState(23, id: 'r.1:in'))?.id, 'r.1:in');
      // an id the text no longer has: the offset decides
      expect(resolveStop(stops, const CaretState(23, id: 'gone'))?.id, 'r.1:in');
      // after the denominator and after the fraction share a byte: the
      // inner `after` is the one typing continues from
      expect(resolveStop(stops, const CaretState(21))?.id, 'r.0.1:after');
      expect(resolveStop(stops, const CaretState(21, id: 'r.0:after'))?.id, 'r.0:after');
      // between stops (mid-edit): the nearest before
      expect(resolveStop(stops, const CaretState(22))?.id, 'r.0.1:after');
      expect(resolveStop(stops, null), isNull);
      expect(resolveStop(const [], const CaretState(0)), isNull);
    });
  });

  group('moves', () {
    final p = clampOf();
    final stops = caretStops(p);
    CaretStop at(String id) => stops.firstWhere((s) => s.id == id);

    test('Right walks the list: out of the denominator, past the fraction, into the slot', () {
      expect(nextStop(stops, at('r.0.1@5'))?.id, 'r.0.1:close');
      expect(nextStop(stops, at('r.0.1:close'))?.id, 'r.0.1:after');
      expect(nextStop(stops, at('r.0.1:after'))?.id, 'r.0:after', reason: 'leaving the fraction');
      expect(nextStop(stops, at('r.0:after'))?.id, 'r.1:in');
      expect(nextStop(stops, at('r:after')), isNull);
      expect(previousStop(stops, at('r.1:in'))?.id, 'r.0:after');
      expect(previousStop(stops, at('r:before')), isNull);
    });

    test('Home and End move among the stops of one enclosing part, then the whole', () {
      // inside a name or a number: its first and last character positions
      expect(homeStop(stops, at('r.0.1@3')).id, 'r.0.1:before');
      expect(endStop(stops, at('r.0.0@2')).id, 'r.0.0:after');
      // after a part of the fraction: the fraction's first and last stops
      expect(homeStop(stops, at('r.0.1:after')).id, 'r.0:before');
      expect(endStop(stops, at('r.0.0:after')).id, 'r.0:after');
      // in a slot of the call: the call's own start and end
      expect(homeStop(stops, at('r.1:in')).id, 'r:before');
      expect(endStop(stops, at('r.1:in')).id, 'r:after');
      // already at the part's start: the formula's own start
      expect(homeStop(stops, at('r.0:before')).id, 'r:before');
      expect(endStop(stops, at('r.0.1:after')).id, 'r.0:after');
    });

    test('Tab moves among the slots, wrapping; from nowhere, the first', () {
      expect(nextSlot(stops, at('r:before'))?.id, 'r.1:in');
      expect(nextSlot(stops, at('r.1:in'))?.id, 'r.1:in', reason: 'the only slot, again');
      expect(nextSlot(stops, at('r.2:after'), backwards: true)?.id, 'r.1:in');
      expect(nextSlot(stops, null)?.id, 'r.1:in');
      // no slot at all
      final full = pb.FormulaProjection(
        source: 'Tilt',
        parseOk: true,
        root: node('r', 'reference', 'Tilt', 0, 4, name: 'Tilt'),
      );
      expect(nextSlot(caretStops(full), null), isNull);
    });

    test('`)` leaves the enclosing parentheses or call; `,` goes to the next argument', () {
      expect(exitGroup(stops, p.root, at('r.0.1@3')).id, 'r.0.1:after');
      expect(exitGroup(stops, p.root, at('r.0.0@2')).id, 'r:after', reason: 'the call');
      expect(exitGroup(stops, p.root, at('r:after')).id, 'r:after');
      expect(nextArgument(stops, p.root, at('r.0.0@2'))?.id, 'r.1:in');
      expect(nextArgument(stops, p.root, at('r.1:in'))?.id, 'r.2:before');
      expect(nextArgument(stops, p.root, at('r.2:after'))?.id, 'r:after', reason: 'last argument');
      expect(nextArgument(stops, p.root, at('r:before')), isNull);
    });
  });

  group('keys', () {
    final p = clampOf();
    final stops = caretStops(p);
    CaretStop at(String id) => stops.firstWhere((s) => s.id == id);

    test('a letter or digit types into a slot or extends the name or number it touches', () {
      // in a slot: the `?` is replaced
      final inSlot = characterAt(p, at('r.1:in'), '0') as EditText;
      expect((inSlot.edit.start, inSlot.edit.end, inSlot.edit.text), (23, 24, '0'));
      expect(inSlot.edit.caretAfter, 24);
      // after a name: appended
      final after = characterAt(p, at('r.0.0:after'), 's') as EditText;
      expect((after.edit.start, after.edit.end, after.edit.text), (10, 10, 's'));
      // inside a name
      final inside = characterAt(p, at('r.0.0@2'), 'x') as EditText;
      expect((inside.edit.start, inside.edit.end), (8, 8));
      // before a number
      final before = characterAt(p, at('r.2:before'), '2') as EditText;
      expect((before.edit.start, before.edit.end, before.edit.text), (26, 26, '2'));
      // a space after a number starts its unit; elsewhere it does nothing
      expect(characterAt(p, at('r.2:after'), ' '), isA<EditText>());
      expect(characterAt(p, at('r.0.0:after'), ' '), isA<Refused>());
      // after a whole structure a value cannot start: an operator first
      final refused = characterAt(p, at('r.0:after'), 'x') as Refused;
      expect(refused.reason, Refused.needsOperator);
      expect(characterAt(p, at('r:after'), '7'), isA<Refused>());
    });

    test('an operator is the compiler\'s action on the part the caret touches', () {
      final after = characterAt(p, at('r.0.0:after'), '/') as Structural;
      expect(
        (after.action.nodeId, after.action.operator.op, after.action.operator.before),
        ('r.0.0', '/', false),
      );
      // after the fraction: on the fraction; before a part: `? op part`
      expect((characterAt(p, at('r.0:after'), '+') as Structural).action.nodeId, 'r.0');
      final before = characterAt(p, at('r.2:before'), '*') as Structural;
      expect((before.action.nodeId, before.action.operator.before), ('r.2', true));
      // inside a name: the whole name
      expect((characterAt(p, at('r.0.0@2'), '-') as Structural).action.nodeId, 'r.0.0');
      // in a slot: the text's rule — the slot splits around the operator,
      // the caret in the second; `-` and `!` are a sign
      final split = characterAt(p, at('r.1:in'), '<') as EditText;
      expect(
        (split.edit.start, split.edit.end, split.edit.text, split.edit.caretAfter),
        (23, 24, '? < ?', 27),
      );
      final sign = characterAt(p, at('r.1:in'), '-') as EditText;
      expect((sign.edit.text, sign.edit.caretAfter), ('-?', 24));
      // the two-character operators by their first key
      expect((characterAt(p, at('r.2:after'), '=') as Structural).action.operator.op, '==');
      expect((characterAt(p, at('r.2:after'), '&') as Structural).action.operator.op, '&&');
      expect((characterAt(p, at('r.2:after'), '!') as Structural).action.operator.op, '!');
    });

    test('`(` applies a name (the compiler\'s arity) or groups a slot', () {
      final apply = characterAt(p, at('r.0.0:after'), '(') as Structural;
      expect(apply.action.nodeId, 'r.0.0');
      expect(apply.action.hasApply(), isTrue);
      final group = characterAt(p, at('r.1:in'), '(') as EditText;
      expect((group.edit.start, group.edit.end, group.edit.text), (23, 24, '(?)'));
      expect(characterAt(p, at('r.2:after'), '('), isA<Refused>());
    });

    test('Backspace: a character, a one-character leaf as a whole, a structure as a whole', () {
      // inside or after a name: one character
      final mid = backspaceAt(p, stops, at('r.0.0@2')) as EditText;
      expect((mid.edit.start, mid.edit.end, mid.edit.text), (7, 8, ''));
      final end = backspaceAt(p, stops, at('r.0.0:after')) as EditText;
      expect((end.edit.start, end.edit.end), (9, 10));
      // the last character of a one-character number: a slot in its place
      // (the text stays readable)
      final one = backspaceAt(p, stops, at('r.2:after')) as EditText;
      expect((one.edit.start, one.edit.end, one.edit.text), (26, 27, '?'));
      // after a structure: the structure, the compiler's remove
      final whole = backspaceAt(p, stops, at('r.0:after')) as Structural;
      expect(whole.action.nodeId, 'r.0');
      // in a slot that is an argument: the compiler's remove (the argument
      // goes); a slot an operator opened goes with the operator, as text
      expect((backspaceAt(p, stops, at('r.1:in')) as Structural).action.nodeId, 'r.1');
      final opened = textBackspaceAt('Tilt / ?', 7) as EditText;
      expect((opened.edit.start, opened.edit.end, opened.edit.caretAfter), (4, 8, 4));
      // before a part or just inside a parenthesis: only a move back
      expect((backspaceAt(p, stops, at('r.2:before')) as MoveTo).stop.id, 'r.1:in');
      expect((backspaceAt(p, stops, at('r.0.1:open')) as MoveTo).stop.id, 'r.0.1:before');
      // Delete is the mirror
      final del = deleteAt(p, stops, at('r.0.0:before')) as EditText;
      expect((del.edit.start, del.edit.end), (6, 7));
      expect((deleteAt(p, stops, at('r.0.0:after')) as MoveTo).stop.id, 'r.0.1:before');
      final delOne = deleteAt(p, stops, at('r.2:before')) as EditText;
      expect((delOne.edit.start, delOne.edit.end, delOne.edit.text), (26, 27, '?'));
    });
  });

  group('the sequence, as text', () {
    (int, int, String, int) edit(KeyPlan k) {
      final e = (k as EditText).edit;
      return (e.start, e.end, e.text, e.caretAfter);
    }

    test('a character replaces the slot it touches, extends the word it touches, begins a '
        'value where one may begin, and is refused after a complete part', () {
      expect(edit(textCharacterAt('Tilt / ?', 7, '9')), (7, 8, '9', 8));
      expect(edit(textCharacterAt('Tilt / ?', 8, '9')), (7, 8, '9', 8), reason: 'after the ?');
      expect(edit(textCharacterAt('Tilt / 9', 8, '0')), (8, 8, '0', 9));
      expect(edit(textCharacterAt('Tilt / 9', 7, '4')), (7, 7, '4', 8));
      expect(edit(textCharacterAt('Tilt / 90', 8, '.')), (8, 8, '.', 9), reason: 'inside');
      expect(edit(textCharacterAt('clamp(Tilt, ?)', 6, 'x')), (6, 6, 'x', 7), reason: 'after (');
      expect(edit(textCharacterAt('if ? then 1 else 0', 3, 'a')), (3, 4, 'a', 4));
      expect(edit(textCharacterAt('', 0, 'a')), (0, 0, 'a', 1));
      final refused = textCharacterAt('Tilt / (90 deg)', 15, 'x') as Refused;
      expect(refused.reason, Refused.needsOperator);
      expect(
        textCharacterAt('Tilt / 90', 5, 'x'),
        isA<Refused>(),
        reason: 'after the operator, before the blank',
      );
    });

    test('a space starts a unit after a number and nothing elsewhere', () {
      expect(edit(textCharacterAt('Tilt / 90', 9, ' ')), (9, 9, ' ', 10));
      expect(edit(textCharacterAt('Tilt / 90 ', 10, 'd')), (10, 10, 'd', 11));
      expect(textCharacterAt('Tilt / 90 ', 10, ' '), isA<Refused>());
      expect(textCharacterAt('Tilt', 4, ' '), isA<Refused>());
      expect(textCharacterAt('Tilt / ?', 7, ' '), isA<Refused>());
    });

    test('an operator in a slot is a sign or a split; `<` then `=` is one operator; '
        'elsewhere it needs the tree', () {
      expect(edit(textOperatorAt('Tilt / ?', 7, '-')), (7, 8, '-?', 8));
      expect(edit(textCharacterAt('Tilt / -?', 8, '5')), (8, 9, '5', 9), reason: 'then the number');
      expect(edit(textOperatorAt('if ? then 1 else 0', 3, '!')), (3, 4, '!?', 4));
      expect(edit(textOperatorAt('Tilt / ?', 7, '+')), (7, 8, '? + ?', 11));
      expect(edit(textOperatorAt('a < ?', 4, '=')), (2, 3, '<=', 5));
      expect(edit(textOperatorAt('a > ?', 4, '=')), (2, 3, '>=', 5));
      expect(textOperatorAt('Tilt', 4, '/'), isA<NeedsStructure>());
      expect(textOperatorAt('Tilt / 90', 9, '+'), isA<NeedsStructure>());
      expect(
        textOperatorAt('Tilt / 90', 2, '+'),
        isA<NeedsStructure>(),
        reason: 'inside a word: the word',
      );
      expect(edit(textOperatorAt('Tilt /', 6, '+', plain: true)), (6, 6, '+', 7));
    });

    test('`(` groups a slot, opens a group where a value may begin, and after a name needs the '
        'tree; `)` steps past the parenthesis before it', () {
      expect(edit(textCharacterAt('Tilt / ?', 7, '(')), (7, 8, '(?)', 8));
      expect(edit(textCharacterAt('Tilt / ', 7, '(')), (7, 7, '(?)', 8));
      expect(textCharacterAt('clamp', 5, '('), isA<NeedsStructure>());
      expect(textCharacterAt('Tilt / 90', 9, '('), isA<Refused>());
      expect(edit(textCharacterAt('(Tilt)', 5, ')')), (5, 5, '', 6));
      expect(textCharacterAt('Tilt', 4, ')'), isA<NeedsStructure>());
    });

    test('Backspace: a character; the last of a value leaves a slot; a slot goes with the '
        'operator that opened it; before a part it steps back over the separators', () {
      expect(edit(textBackspaceAt('Tilt / 90', 9)), (8, 9, '', 8));
      expect(edit(textBackspaceAt('Tilt / 9', 8)), (7, 8, '?', 7));
      expect(edit(textBackspaceAt('Tilt / ?', 7)), (4, 8, '', 4));
      expect(edit(textBackspaceAt('Tilt / ?', 8)), (4, 8, '', 4));
      expect(edit(textBackspaceAt('Tilt / -?', 8)), (7, 8, '', 7));
      expect(edit(textBackspaceAt('a + b', 4)), (4, 4, '', 1));
      expect(edit(textBackspaceAt('clamp(?, 1)', 9)), (9, 9, '', 7));
      expect(textBackspaceAt('Tilt', 0), isA<Refused>());
      expect(edit(textBackspaceAt('Tilt /', 6, plain: true)), (5, 6, '', 5));
      // a multi-byte character is one character
      expect(edit(textBackspaceAt('90 °', 5)), (3, 5, '', 3));
    });

    test('Delete is the mirror', () {
      expect(edit(textDeleteAt('Tilt / 90', 7)), (7, 8, '', 7));
      expect(edit(textDeleteAt('Tilt / 9', 7)), (7, 8, '?', 7));
      expect(edit(textDeleteAt('? + b', 0)), (0, 4, '', 0));
      expect(edit(textDeleteAt('a + b', 1)), (1, 1, '', 4));
      expect(textDeleteAt('Tilt', 4), isA<Refused>());
    });

    test('the stops of a reading move with the text typed since', () {
      final p = clampOf();
      final stops = caretStops(p);
      // `Tilt` became `Tilts`: everything after byte 10 is one byte later
      final r = changedRegion(p.source, 'clamp(Tilts / (90 deg), ?, 1)')!;
      final shifted = shiftedStops(stops, (start: 6, end: 10, newLength: 5));
      expect(r.start, 10);
      expect(shifted.firstWhere((s) => s.id == 'r.1:in').offset, 24);
      expect(shifted.firstWhere((s) => s.id == 'r.0.0:before').offset, 6);
      expect(shifted.firstWhere((s) => s.id == 'r.0.0:after').offset, 11);
      expect(shifted.any((s) => s.id == 'r.0.0@2'), isFalse, reason: 'the edited text\'s own');
    });
  });

  group('the pending region', () {
    test('the region a text edit changed, and the innermost part holding it', () {
      final p = clampOf();
      final r = changedRegion(p.source, 'clamp(Tilts / (90 deg), ?, 1)')!;
      expect((r.start, r.end, r.newLength), (10, 10, 1));
      expect(nodeContaining(p.root, r.start, r.end)?.id, 'r.0.0');
      final r2 = changedRegion(p.source, 'clamp(Tilt / (9 deg), ?, 1)')!;
      expect((r2.start, r2.end, r2.newLength), (15, 16, 0));
      expect(nodeContaining(p.root, r2.start, r2.end)?.id, 'r.0.1');
      // a change across parts: the innermost structure holding it
      final r3 = changedRegion(p.source, 'clamp(X, ?, 1)')!;
      expect(nodeContaining(p.root, r3.start, r3.end)?.id, 'r.0');
      final r4 = changedRegion(p.source, 'clamp(X, 1)')!;
      expect(nodeContaining(p.root, r4.start, r4.end)?.id, 'r');
      expect(changedRegion(p.source, p.source), isNull);
    });
  });
}
