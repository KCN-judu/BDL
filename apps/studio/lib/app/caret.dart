/// The structural caret of the Formula view (docs/architecture/studio-ui.md
/// §4b, *Typed structure*): where a typed character lands and what each
/// key does there, as pure functions over the compiler's projection.
///
/// The compiler's projection states every part with its byte range; the
/// **carets** of a formula are read off it — `before` and `after` each
/// part, `in` an empty slot, `open` and `close` just inside a
/// parenthesised part's parentheses — in reading order, each with the
/// part it belongs to and the part enclosing it (the same positions the
/// compiler's `NavigateFormula` moves between, protocol 0.27).  Studio
/// adds the **character positions inside a leaf** (a name, a number, a
/// unit: text edited as text) and nothing else: the result is the list of
/// *stops* the caret moves along.  Left and Right
/// walk the list — the last stop inside a denominator is followed by the
/// stop after the fraction, which is how a nested part is left; Home and
/// End move among the stops of one enclosing part; Tab among the slots.
/// Where a stop stands on screen is the widget's layout; what it means is
/// here.
///
/// What a key does is one [KeyPlan].  The principle is a formula editor's
/// (GeoGebra's `EditorState` / `InputController`: the keys mutate the
/// editor's own sequence at its own cursor, synchronously; the parser
/// reads the result afterwards and never moves the cursor): **a key acts
/// on the text Studio holds, at the caret's byte, now** — a letter, a
/// digit, an operator with its slot, a parenthesis, a deletion are text
/// edits of the draft ([textCharacterAt], [textBackspaceAt],
/// [textDeleteAt]), computed without the compiler and never waiting for
/// it.  The compiler's tree, when it is current, adds what text cannot
/// know: where a whole part begins and ends (⌫ after a fraction removes
/// the fraction), what a name applies to (`(` after `clamp` is the
/// compiler's arity), what a part negated is; those are structured
/// actions the compiler answers with text, and a key that needs them
/// while the tree is not current is [NeedsStructure] — the composer keeps
/// it, in order, until the reading arrives.  Nothing typed is ever
/// dropped, and nothing here parses, types or decides what fits: text
/// that does not parse is shown as text until it does.
library;

import 'dart:convert' show utf8;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'state.dart' show CaretState;

/// Where a caret may stand.
enum StopSide {
  before,
  after,
  inSlot,
  open,
  close,

  /// Between two characters of a leaf's text (Studio's own).
  inside,
}

/// One place the caret can be: a byte [offset] into the source, the
/// [node] it belongs to, the side, the enclosing [parent] (empty at the
/// root).  [id] is stable while the text is: the compiler's caret id, or
/// `<node>@<char>` for a position inside a leaf.
class CaretStop {
  const CaretStop({
    required this.id,
    required this.offset,
    required this.node,
    required this.side,
    required this.parent,
  });
  final String id;
  final int offset;
  final String node;
  final StopSide side;
  final String parent;

  @override
  String toString() => '$id@$offset';
}

/// The parts whose text is edited character by character.
bool isTextLeaf(pb.FormulaNode n) =>
    n.kind == 'reference' || n.kind == 'number' || n.kind == 'quantity';

/// Whether the node's text is wholly parenthesised (its range includes
/// the parentheses): the leaf's own text lies one byte in on each side.
bool wrappedInParens(pb.FormulaNode n) {
  final t = n.text;
  if (!t.startsWith('(') || !t.endsWith(')')) return false;
  var depth = 0;
  for (var i = 0; i < t.length; i++) {
    final c = t[i];
    if (c == '(') depth++;
    if (c == ')') {
      depth--;
      if (depth == 0) return i == t.length - 1;
    }
  }
  return false;
}

pb.FormulaNode? findNode(pb.FormulaNode? root, String id) {
  if (root == null) return null;
  if (root.id == id) return root;
  for (final c in root.children) {
    final f = findNode(c, id);
    if (f != null) return f;
  }
  return null;
}

/// The parent's id of a tree path (`r.0.1` → `r.0`; `r` → '').
String parentId(String id) {
  final i = id.lastIndexOf('.');
  return i < 0 ? '' : id.substring(0, i);
}

/// Every stop of the projection, in reading order, read off the
/// compiler's tree: each part's `before` and `after` at its byte range
/// (its parentheses included), `open` and `close` just inside a
/// parenthesised part, `in` for a slot; and, Studio's own, one stop per
/// character boundary strictly inside a text leaf.  At one byte the order
/// is outer-before, inner-before, …, inner-after, outer-after: after the
/// denominator comes after the fraction, which is how Right leaves it.
/// Empty when the projection has no tree.  (The compiler's `NavigateFormula`
/// answers the same moves over the wire; Studio walks the tree it was
/// given so an arrow key costs no round trip.)
List<CaretStop> caretStops(pb.FormulaProjection? p) {
  if (p == null || !p.hasRoot()) return const [];
  final out = <(int, int, CaretStop)>[];
  void walk(pb.FormulaNode n, String parent, int depth) {
    CaretStop stop(StopSide side, String word, int offset) =>
        CaretStop(id: '${n.id}:$word', offset: offset, node: n.id, side: side, parent: parent);
    final opening = depth;
    final closing = 1000 - depth;
    if (n.kind == 'slot') {
      out.add((n.range.start, 500, stop(StopSide.inSlot, 'in', n.range.start)));
      return;
    }
    out.add((n.range.start, opening, stop(StopSide.before, 'before', n.range.start)));
    out.add((n.range.end, closing, stop(StopSide.after, 'after', n.range.end)));
    final grouped = n.kind != 'call' && n.kind != 'opaque' && wrappedInParens(n);
    if (grouped && n.range.end > n.range.start + 1) {
      out.add((n.range.start + 1, opening + 1, stop(StopSide.open, 'open', n.range.start + 1)));
      out.add((n.range.end - 1, closing - 1, stop(StopSide.close, 'close', n.range.end - 1)));
    }
    if (isTextLeaf(n)) {
      final paren = wrappedInParens(n);
      final start = n.range.start + (paren ? 1 : 0);
      final end = n.range.end - (paren ? 1 : 0);
      for (var k = start + 1; k < end; k++) {
        out.add((
          k,
          500,
          CaretStop(
            id: '${n.id}@${k - start}',
            offset: k,
            node: n.id,
            side: StopSide.inside,
            parent: parent,
          ),
        ));
      }
    }
    for (final c in n.children) {
      walk(c, n.id, depth + 2);
    }
  }

  walk(p.root, '', 0);
  out.sort((a, b) {
    final d = a.$1.compareTo(b.$1);
    return d != 0 ? d : a.$2.compareTo(b.$2);
  });
  return [for (final e in out) e.$3];
}

/// The stop a caret state names: by id when the text still has it, else
/// the best stop at its byte offset — a slot's `in` first, then the
/// innermost `after` (typing continues the part just written), then any
/// — else `null`.
CaretStop? resolveStop(List<CaretStop> stops, CaretState? caret) {
  if (caret == null || stops.isEmpty) return null;
  if (caret.id != null) {
    for (final s in stops) {
      if (s.id == caret.id) return s;
    }
  }
  final at = stops.where((s) => s.offset == caret.offset).toList();
  if (at.isEmpty) {
    // between stops (a text edit moved the byte): the nearest before it
    CaretStop? best;
    for (final s in stops) {
      if (s.offset <= caret.offset && (best == null || s.offset > best.offset)) best = s;
    }
    return best ?? stops.first;
  }
  for (final s in at) {
    if (s.side == StopSide.inSlot) return s;
  }
  for (final s in at) {
    if (s.side == StopSide.after || s.side == StopSide.inside) return s;
  }
  return at.first;
}

CaretState stateOf(CaretStop s) => CaretState(s.offset, id: s.id);

int indexOf(List<CaretStop> stops, CaretStop s) => stops.indexWhere((x) => x.id == s.id);

CaretStop? nextStop(List<CaretStop> stops, CaretStop current) {
  final i = indexOf(stops, current);
  return i < 0 || i + 1 >= stops.length ? null : stops[i + 1];
}

CaretStop? previousStop(List<CaretStop> stops, CaretStop current) {
  final i = indexOf(stops, current);
  return i <= 0 ? null : stops[i - 1];
}

/// The stops within the part enclosing [current]: for a stop inside a
/// part (a leaf's characters, just inside parentheses) the part itself;
/// for a stop before or after a part, or in a slot, the part's parent.
/// The stops within a part are its own — its start and end included —
/// and every stop of its descendants.
List<CaretStop> _enclosed(List<CaretStop> stops, CaretStop current) {
  final e = switch (current.side) {
    StopSide.before || StopSide.after || StopSide.inSlot => current.parent,
    _ => current.node,
  };
  if (e.isEmpty) return stops;
  bool within(CaretStop s) => s.node == e || s.node.startsWith('$e.');
  return stops.where(within).toList();
}

/// Home: the first stop of the current stop's enclosing part; when
/// already there, the first stop of all.
CaretStop homeStop(List<CaretStop> stops, CaretStop current) {
  final own = _enclosed(stops, current);
  final first = own.isEmpty ? stops.first : own.first;
  return first.id == current.id ? stops.first : first;
}

/// End: the last stop of the current stop's enclosing part; when already
/// there, the last stop of all.
CaretStop endStop(List<CaretStop> stops, CaretStop current) {
  final own = _enclosed(stops, current);
  final last = own.isEmpty ? stops.last : own.last;
  return last.id == current.id ? stops.last : last;
}

/// The next / previous empty slot after the current stop (the Tab order),
/// wrapping around.
CaretStop? nextSlot(List<CaretStop> stops, CaretStop? current, {bool backwards = false}) {
  final slots = stops.where((s) => s.side == StopSide.inSlot).toList();
  if (slots.isEmpty) return null;
  if (current == null) return backwards ? slots.last : slots.first;
  final i = indexOf(stops, current);
  if (backwards) {
    for (var k = slots.length - 1; k >= 0; k--) {
      if (indexOf(stops, slots[k]) < i) return slots[k];
    }
    return slots.last;
  }
  for (final s in slots) {
    if (indexOf(stops, s) > i) return s;
  }
  return slots.first;
}

/// The stop the Enter-a-group key `)` leaves to: the `after` stop of the
/// nearest enclosing parenthesised part or call, else the last stop.
CaretStop exitGroup(List<CaretStop> stops, pb.FormulaNode root, CaretStop current) {
  var id = current.side == StopSide.inSlot || current.side == StopSide.inside
      ? current.node
      : current.side == StopSide.before || current.side == StopSide.after
      ? current.parent
      : current.node;
  // `open`/`close` of a group and `before`/`after` of a child: the group
  // itself; climb until a part with parentheses or a call
  while (id.isNotEmpty) {
    final n = findNode(root, id);
    if (n == null) break;
    if (n.kind == 'call' || wrappedInParens(n)) {
      final after = stops.where((s) => s.node == id && s.side == StopSide.after).firstOrNull;
      if (after != null && indexOf(stops, after) > indexOf(stops, current)) return after;
    }
    id = parentId(id);
  }
  return stops.last;
}

/// The stop `,` moves to: the first stop of the next argument of the
/// enclosing call (or the next arm / binding of a match or block), else
/// the call's `after` stop.
CaretStop? nextArgument(List<CaretStop> stops, pb.FormulaNode root, CaretStop current) {
  var child = current.side == StopSide.before || current.side == StopSide.after
      ? current.node
      : current.side == StopSide.inSlot || current.side == StopSide.inside
      ? current.node
      : current.node;
  var parent = parentId(child);
  while (parent.isNotEmpty) {
    final p = findNode(root, parent);
    if (p == null) return null;
    if (const {'call', 'match', 'block', 'delay', 'sync', 'list', 'tuple'}.contains(p.kind)) {
      final i = p.children.indexWhere((c) => c.id == child);
      if (i >= 0 && i + 1 < p.children.length) {
        final next = p.children[i + 1];
        return stops
            .where((s) => s.node == next.id || s.node.startsWith('${next.id}.'))
            .firstOrNull;
      }
      return stops.where((s) => s.node == parent && s.side == StopSide.after).firstOrNull;
    }
    child = parent;
    parent = parentId(parent);
  }
  return null;
}

// ---- what a key does ----------------------------------------------------------

/// A text edit of the draft: the bytes [start, end) become [text].  The
/// caret lands at [caret] (a byte into the new text), else after the
/// inserted text.
class TextEditPlan {
  const TextEditPlan(this.start, this.end, this.text, {this.caret});
  final int start;
  final int end;
  final String text;

  /// Where the caret lands when not after the inserted text.
  final int? caret;

  /// Where the caret lands.
  int get caretAfter => caret ?? start + utf8.encode(text).length;
}

/// What one key does at one place.
sealed class KeyPlan {
  const KeyPlan();
}

class EditText extends KeyPlan {
  const EditText(this.edit);
  final TextEditPlan edit;
}

class Structural extends KeyPlan {
  const Structural(this.action);
  final pb.ComposeAction action;
}

class MoveTo extends KeyPlan {
  const MoveTo(this.stop);
  final CaretStop stop;
}

/// The key acts on structure the text alone does not tell (what a name
/// applies to, where a whole part ends) and the compiler's reading of the
/// text is not current: the composer keeps the key until it is.
class NeedsStructure extends KeyPlan {
  const NeedsStructure();
}

/// The key does nothing here; [reason] says why when it is worth saying
/// (a value typed where an operator is needed first).
class Refused extends KeyPlan {
  const Refused([this.reason]);
  final String? reason;

  static const String needsOperator = 'needs-operator';
}

/// The operator a typed character stands for, or `null`.
String? operatorFor(String char) => switch (char) {
  '+' => '+',
  '-' => '-',
  '*' => '*',
  '/' => '/',
  '<' => '<',
  '>' => '>',
  '=' => '==',
  '&' => '&&',
  '|' => '||',
  '!' => '!',
  _ => null,
};

bool _isWordChar(String c) => RegExp(r'^[A-Za-z0-9_.]$').hasMatch(c);

// ---- the sequence, as text ------------------------------------------------------
//
// GeoGebra edits a sequence of characters at a cursor offset; a BDL
// formula's sequence is its text and the cursor a byte into it.  These
// functions are that editor: every case is a byte-range edit decided from
// the characters around the caret — a `?` is a slot (typing next to it
// replaces it, as typing over a placeholder does), ` op ` an operator —
// and none needs the compiler.

/// Byte offset → code unit in [source] (clamped, never inside a character).
int _cu(String source, int byte) {
  var bytes = 0;
  var units = 0;
  for (final rune in source.runes) {
    if (bytes >= byte) return units;
    bytes += rune < 0x80
        ? 1
        : rune < 0x800
        ? 2
        : rune < 0x10000
        ? 3
        : 4;
    units += rune > 0xFFFF ? 2 : 1;
  }
  return units;
}

/// Code unit → byte offset in [source].
int _by(String source, int cu) =>
    utf8.encode(source.substring(0, cu.clamp(0, source.length))).length;

/// The byte where the character before byte [at] starts; `null` at 0.
int? previousCharStart(String source, int at) {
  final cu = _cu(source, at);
  if (cu == 0) return null;
  var back = 1;
  // a surrogate pair is one character
  if (cu >= 2 && _isLowSurrogate(source.codeUnitAt(cu - 1))) back = 2;
  return _by(source, cu - back);
}

/// The byte after the character at byte [at]; `null` at the end.
int? nextCharEnd(String source, int at) {
  final cu = _cu(source, at);
  if (cu >= source.length) return null;
  var forward = 1;
  if (cu + 1 < source.length && _isHighSurrogate(source.codeUnitAt(cu))) forward = 2;
  return _by(source, cu + forward);
}

bool _isHighSurrogate(int u) => u >= 0xD800 && u <= 0xDBFF;
bool _isLowSurrogate(int u) => u >= 0xDC00 && u <= 0xDFFF;

/// The character right before / at byte [at] ('' at an edge).
String _before(String source, int at) {
  final cu = _cu(source, at);
  return cu == 0 ? '' : source[cu - 1];
}

String _at(String source, int at) {
  final cu = _cu(source, at);
  return cu >= source.length ? '' : source[cu];
}

/// The text up to byte [at].
String _head(String source, int at) => source.substring(0, _cu(source, at));

/// The slot the caret touches: the byte of a `?` right at or right before
/// the caret, else `null`.
int? _slotAt(String source, int at) {
  if (_at(source, at) == '?') return at;
  if (_before(source, at) == '?') return at - 1;
  return null;
}

/// Where a value may begin at byte [at]: the start of the text, or after
/// `(`, `[`, `,`, an operator or a keyword — the previous non-blank
/// character read backwards.
bool _valueMayStart(String source, int at) {
  final head = _head(source, at).trimRight();
  if (head.isEmpty) return true;
  final last = head[head.length - 1];
  if ('([,+-*/<>=&|!'.contains(last)) return true;
  final word = RegExp(r'([A-Za-z_][A-Za-z0-9_]*)$').firstMatch(head)?.group(1);
  return word != null && _keywords.contains(word);
}

const Set<String> _keywords = {
  'if',
  'then',
  'else',
  'in',
  'match',
  'with',
  'let',
  'and',
  'or',
  'not',
};

/// The number the caret is right after, blank included (`90 |`): a unit
/// may follow.
bool _afterNumber(String source, int at) =>
    RegExp(r'[0-9.](\s[A-Za-z_][A-Za-z0-9_/^]*)?\s?$').hasMatch(_head(source, at)) &&
    !_head(source, at).endsWith('  ');

/// A character typed at byte [at] of [source], as text: a word character
/// replaces the slot it touches, extends the word it touches, or begins a
/// value where one may begin; a space is only meaningful after a number
/// (its unit follows); an operator is [textOperatorAt]; `(` groups a slot
/// or opens a group where a value may begin, and after a name needs the
/// compiler (what the name applies to, its arity); `)` steps past the
/// parenthesis it stands before; `,` needs the compiler (the next
/// argument).  Elsewhere a value cannot begin without an operator first.
///
/// With [plain], the field is text the compiler could not read: every
/// character is inserted where the caret is, so what was typed can be
/// mended.
KeyPlan textCharacterAt(String source, int at, String char, {bool plain = false}) {
  if (plain) return EditText(TextEditPlan(at, at, char));
  final op = operatorFor(char);
  if (op != null) return textOperatorAt(source, at, char, plain: plain);
  final slot = _slotAt(source, at);
  if (char == '(') {
    if (slot != null) return EditText(TextEditPlan(slot, slot + 1, '(?)', caret: slot + 1));
    if (_isWordChar(_before(source, at)) && !RegExp(r'[0-9.]').hasMatch(_before(source, at))) {
      return const NeedsStructure();
    }
    if (_valueMayStart(source, at)) return EditText(TextEditPlan(at, at, '(?)', caret: at + 1));
    return const Refused(Refused.needsOperator);
  }
  if (char == ')') {
    return _at(source, at) == ')'
        ? EditText(TextEditPlan(at, at, '', caret: at + 1))
        : const NeedsStructure();
  }
  if (char == ',') return const NeedsStructure();
  if (char == ' ') {
    if (slot == null && _afterNumber(source, at) && _before(source, at) != ' ') {
      return EditText(TextEditPlan(at, at, ' '));
    }
    return const Refused();
  }
  if (!_isWordChar(char)) return const Refused();
  if (slot != null) return EditText(TextEditPlan(slot, slot + 1, char));
  final before = _before(source, at);
  final after = _at(source, at);
  if (_isWordChar(before) || _isWordChar(after)) return EditText(TextEditPlan(at, at, char));
  if (before == ' ' && _afterNumber(source, at)) return EditText(TextEditPlan(at, at, char));
  if (_valueMayStart(source, at)) return EditText(TextEditPlan(at, at, char));
  return const Refused(Refused.needsOperator);
}

/// An operator typed at byte [at], as text — what the text alone can
/// decide: in a slot, `-` and `!` are a sign (`-?`: a negative number is
/// typed as one) and the others split it (`? + ?`, the caret in the
/// second); `=` after `<` or `>` makes `<=`, `>=`; with [plain] the
/// character is inserted.  Anywhere else an operator applies to a part
/// — the value before the caret, kept whole, parenthesised where the
/// operator binds looser than the part's place (a `+` in a denominator
/// stays in the denominator) — which only the compiler's tree tells:
/// [NeedsStructure].
KeyPlan textOperatorAt(String source, int at, String char, {bool plain = false}) {
  if (plain) return EditText(TextEditPlan(at, at, char));
  final op = operatorFor(char)!;
  final slot = _slotAt(source, at);
  final prefix = op == '-' || op == '!';
  // `<` then `=`: one operator
  if (char == '=') {
    final head = _head(source, slot ?? at);
    final m = RegExp(r' ([<>]) $').firstMatch(head);
    if (m != null) {
      final opAt = _by(source, m.start + 1);
      return EditText(TextEditPlan(opAt, opAt + 1, '${m.group(1)}=', caret: (slot ?? at) + 1));
    }
  }
  if (slot != null) {
    if (prefix) return EditText(TextEditPlan(slot, slot + 1, '$op?', caret: slot + op.length));
    final text = '? $op ?';
    return EditText(TextEditPlan(slot, slot + 1, text, caret: slot + text.length - 1));
  }
  return const NeedsStructure();
}

/// Backspace at byte [at], as text: a slot with the operator that opened
/// it goes together (`a + ?` → `a`); the last character of a word leaves
/// a slot in its place (`a + b` → `a + ?`), so the text stays readable;
/// otherwise the character before the caret goes.  With [plain] (text
/// the compiler could not read) always the character.
KeyPlan textBackspaceAt(String source, int at, {bool plain = false}) {
  final prev = previousCharStart(source, at);
  if (prev == null) return const Refused();
  if (plain) return EditText(TextEditPlan(prev, at, ''));
  final slot = _slotAt(source, at);
  if (slot != null) {
    final head = _head(source, slot);
    // `a + ?`: the operator and its blank go with the slot
    final m = RegExp(r' (\+|-|\*|/|<=|>=|==|!=|<|>|&&|\|\|) $').firstMatch(head);
    if (m != null) {
      final start = _by(source, m.start);
      return EditText(TextEditPlan(start, slot + 1, '', caret: start));
    }
    // `-?` / `!?`: the prefix
    final p = RegExp(r'[-!]$').firstMatch(head);
    if (p != null) {
      final start = _by(source, p.start);
      return EditText(TextEditPlan(start, slot, '', caret: start));
    }
    // `? + b`, `(?)`: the slot stays, the caret steps back
    return slot < at ? EditText(TextEditPlan(at, at, '', caret: slot)) : const Refused();
  }
  final before = _before(source, at);
  final after = _at(source, at);
  if (_isSeparator(before) && (_isWordChar(after) || after == '?' || after == '(')) {
    // before a part (`a + |b`, `(?, |1)`): the caret steps back over the
    // separators to the part before — an operator is removed with the
    // operand it opened, never on its own, so the text stays readable
    var to = prev;
    while (to > 0 && _isSeparator(_before(source, to))) {
      to = previousCharStart(source, to) ?? 0;
    }
    return EditText(TextEditPlan(at, at, '', caret: to));
  }
  if (_isWordChar(before) && !_isWordChar(_before(source, prev)) && !_isWordChar(_at(source, at))) {
    // the only character of a value: a slot in its place
    return EditText(TextEditPlan(prev, at, '?', caret: prev));
  }
  return EditText(TextEditPlan(prev, at, '', caret: prev));
}

/// The characters between parts: blanks, commas, the operators and the
/// parentheses.
bool _isSeparator(String c) => c.isNotEmpty && ' ,+-*/<>=&|!()[]'.contains(c);

/// Delete (forward) at byte [at], as text: the mirror of
/// [textBackspaceAt].
KeyPlan textDeleteAt(String source, int at, {bool plain = false}) {
  final next = nextCharEnd(source, at);
  if (next == null) return const Refused();
  if (plain) return EditText(TextEditPlan(at, next, '', caret: at));
  final slot = _slotAt(source, at);
  if (slot != null && slot == at) {
    final tail = source.substring(_cu(source, slot + 1));
    // `? + b`: the slot and the operator after it
    final m = RegExp(r'^ (\+|-|\*|/|<=|>=|==|!=|<|>|&&|\|\|) ').firstMatch(tail);
    if (m != null) return EditText(TextEditPlan(slot, slot + 1 + m.end, '', caret: slot));
    return const Refused();
  }
  final c = _at(source, at);
  final before = _before(source, at);
  if (_isSeparator(c) && (_isWordChar(before) || before == '?' || before == ')')) {
    // after a part: the caret steps forward over the separators
    var to = next;
    final end = utf8.encode(source).length;
    while (to < end && _isSeparator(_at(source, to))) {
      to = nextCharEnd(source, to) ?? end;
    }
    return EditText(TextEditPlan(at, at, '', caret: to));
  }
  if (_isWordChar(c) && !_isWordChar(before) && !_isWordChar(_at(source, next))) {
    return EditText(TextEditPlan(at, next, '?', caret: at));
  }
  return EditText(TextEditPlan(at, next, '', caret: at));
}

// ---- the keys over the compiler's tree ------------------------------------------

/// The leaf the stop touches — the leaf itself for `inside`, or the leaf
/// the stop is right after / before — else `null`.
pb.FormulaNode? _touchedLeaf(pb.FormulaNode root, CaretStop stop) {
  final n = findNode(root, stop.node);
  if (n == null) return null;
  if (stop.side == StopSide.inside) return n;
  if ((stop.side == StopSide.after || stop.side == StopSide.before) && isTextLeaf(n)) return n;
  return null;
}

/// A character typed at byte [at] of [p]'s source, the caret at [stop]:
/// the text edit [textCharacterAt] decides, with the tree adding what the
/// text does not tell — an operator applies to the part the caret
/// touches ([operatorAt]), `(` after a name applies the name (the
/// compiler's arity).
///
/// [source] is the text on screen — the projection's own unless
/// characters were typed since — and [at] the caret's byte into it.
KeyPlan characterAt(
  pb.FormulaProjection p,
  CaretStop stop,
  String char, {
  int? at,
  String? source,
}) {
  final text = source ?? p.source;
  final offset = at ?? stop.offset;
  final plan = textCharacterAt(text, offset, char);
  if (plan is! NeedsStructure) return plan;
  if (operatorFor(char) != null) return operatorAt(p, stop, char, at: offset, source: text);
  if (char == '(') {
    final leaf = _touchedLeaf(p.root, stop);
    if (leaf != null && leaf.kind == 'reference' && stop.side != StopSide.before) {
      return Structural(pb.ComposeAction(nodeId: leaf.id, apply: pb.Unit()));
    }
    return const Refused(Refused.needsOperator);
  }
  return plan;
}

/// An operator at a stop over the tree: the compiler puts it after (or
/// before) the part the stop belongs to, with a slot for the other side,
/// parenthesising where the part's place needs it.  Inside a leaf the
/// operator applies to the whole leaf; `open` puts the operator before
/// the part; `!` negates the part in place.  A slot's own cases are the
/// text's ([textOperatorAt]).
KeyPlan operatorAt(pb.FormulaProjection p, CaretStop stop, String char, {int? at, String? source}) {
  final text = textOperatorAt(source ?? p.source, at ?? stop.offset, char);
  if (text is! NeedsStructure) return text;
  final op = operatorFor(char)!;
  final before = stop.side == StopSide.before || stop.side == StopSide.open;
  if (op == '!' && before) {
    return Structural(
      pb.ComposeAction(
        nodeId: stop.node,
        operator: pb.ComposeOperator(op: '!', before: false),
      ),
    );
  }
  // a stop after a child of a parenthesised group's last part and the
  // group's own close stop mean different things: `(a + b|)` extends the
  // sum, `(a + b)|` operates on the group — both the compiler's
  return Structural(
    pb.ComposeAction(
      nodeId: stop.node,
      operator: pb.ComposeOperator(op: op, before: before),
    ),
  );
}

/// `(` typed at a stop: after a name it applies the name (a call with one
/// slot per argument, the compiler's arity); in a slot it groups the slot
/// (`(?)`); elsewhere as text.
KeyPlan openParenAt(pb.FormulaProjection p, CaretStop stop) => characterAt(p, stop, '(');

/// Backspace at a stop over the tree: right after a whole structure (a
/// fraction, a call, a choice) the structure is removed — the compiler's
/// `remove`, which takes an operator whose operand it was; just inside a
/// closing parenthesis the caret steps back to the last part; elsewhere
/// the text rule ([textBackspaceAt]) at the caret's own byte.
KeyPlan backspaceAt(
  pb.FormulaProjection p,
  List<CaretStop> stops,
  CaretStop stop, {
  int? at,
  String? source,
}) {
  final root = p.root;
  final text = source ?? p.source;
  final n = findNode(root, stop.node);
  if (n == null) return const Refused();
  final offset = at ?? stop.offset;
  if (offset == stop.offset) {
    switch (stop.side) {
      case StopSide.after:
        if (!isTextLeaf(n) && n.kind != 'slot') {
          return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
        }
      case StopSide.close:
        final last = n.children.isEmpty ? null : n.children.last;
        if (last == null) return const Refused();
        final after = stops.where((s) => s.node == last.id && s.side == StopSide.after).firstOrNull;
        return after == null ? const Refused() : MoveTo(after);
      case StopSide.inSlot:
        // the slot's own removal is the compiler's when it stands alone in
        // a structure (`clamp(?, 1)`: the argument goes); with an
        // operator, the text rule takes both
        final plan = textBackspaceAt(text, offset);
        if (plan is EditText) return plan;
        return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
      case StopSide.before:
      case StopSide.open:
        // only a move back: an operator goes with the operand it opened
        final prev = previousStop(stops, stop);
        return prev == null ? const Refused() : MoveTo(prev);
      case StopSide.inside:
        break;
    }
  }
  return textBackspaceAt(text, offset);
}

/// Delete (forward) at a stop: the mirror of backspace.
KeyPlan deleteAt(
  pb.FormulaProjection p,
  List<CaretStop> stops,
  CaretStop stop, {
  int? at,
  String? source,
}) {
  final root = p.root;
  final text = source ?? p.source;
  final n = findNode(root, stop.node);
  if (n == null) return const Refused();
  final offset = at ?? stop.offset;
  if (offset == stop.offset) {
    switch (stop.side) {
      case StopSide.before:
        if (!isTextLeaf(n) && n.kind != 'slot') {
          return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
        }
      case StopSide.inSlot:
        final plan = textDeleteAt(text, offset);
        if (plan is EditText) return plan;
        return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
      case StopSide.open:
        final first = n.children.isEmpty ? null : n.children.first;
        if (first == null) return const Refused();
        final before = stops
            .where((s) => s.node == first.id && s.side == StopSide.before)
            .firstOrNull;
        return before == null ? const Refused() : MoveTo(before);
      case StopSide.after:
      case StopSide.close:
        final next = nextStop(stops, stop);
        return next == null ? const Refused() : MoveTo(next);
      case StopSide.inside:
        break;
    }
  }
  return textDeleteAt(text, offset);
}

/// The region of [current] that differs from [old], as a byte range into
/// [old] and the replacement's byte length: `null` when the texts are
/// equal.  (Bytes, as the compiler's ranges are.)
({int start, int end, int newLength})? changedRegion(String old, String current) {
  if (old == current) return null;
  final a = utf8.encode(old);
  final b = utf8.encode(current);
  var prefix = 0;
  final max = a.length < b.length ? a.length : b.length;
  while (prefix < max && a[prefix] == b[prefix]) {
    prefix++;
  }
  var suffix = 0;
  while (suffix < max - prefix && a[a.length - 1 - suffix] == b[b.length - 1 - suffix]) {
    suffix++;
  }
  return (start: prefix, end: a.length - suffix, newLength: b.length - prefix - suffix);
}

/// The stops of a reading, moved to the text as it is now: a text edit
/// of [region] bytes (into the reading's source) that made the text
/// [delta] bytes longer shifts every stop past it; the character stops
/// strictly inside the edited region belong to text that no longer exists
/// and go.  What the caret walks while the compiler reads.
List<CaretStop> shiftedStops(List<CaretStop> stops, ({int start, int end, int newLength}) region) {
  final delta = region.newLength - (region.end - region.start);
  final out = <CaretStop>[];
  for (final s in stops) {
    if (s.offset <= region.start) {
      out.add(s);
    } else if (s.offset >= region.end) {
      out.add(
        CaretStop(id: s.id, offset: s.offset + delta, node: s.node, side: s.side, parent: s.parent),
      );
    }
  }
  return out;
}

/// The innermost node of [root] whose range contains the byte range
/// [start, end) (a text edit's region in the projection's own source),
/// else `null`.  The part the composer shows as text until the compiler
/// has read the edit.
pb.FormulaNode? nodeContaining(pb.FormulaNode? root, int start, int end) {
  if (root == null || !root.hasRange()) return null;
  if (root.range.start > start || root.range.end < end) return null;
  for (final c in root.children) {
    final f = nodeContaining(c, start, end);
    if (f != null) return f;
  }
  return root;
}
