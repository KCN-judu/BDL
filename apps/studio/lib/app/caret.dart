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
/// What a key does at a stop is one [KeyPlan]: a text edit of the draft
/// at the stop's byte offset (letters, digits, a unit after a number — the
/// compiler reads the result), a structured action the compiler answers
/// with text (an operator, `(` on a name, deleting a whole part), or a
/// move.  Nothing here parses, types or decides what fits: a text edit
/// that does not parse is shown as text until it does (the composer's
/// pending region), and a structured action is the compiler's.
library;

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

/// A text edit of the draft: the bytes [start, end) become [text].
class TextEditPlan {
  const TextEditPlan(this.start, this.end, this.text);
  final int start;
  final int end;
  final String text;

  /// Where the caret lands: after the inserted text.
  int get caretAfter => start + text.length;
}

/// What one key does at one stop.
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

/// The leaf the stop touches — the leaf itself for `inside`, or the leaf
/// the stop is right after / before — else `null`.
pb.FormulaNode? _touchedLeaf(pb.FormulaNode root, CaretStop stop) {
  final n = findNode(root, stop.node);
  if (n == null) return null;
  if (stop.side == StopSide.inside) return n;
  if ((stop.side == StopSide.after || stop.side == StopSide.before) && isTextLeaf(n)) return n;
  return null;
}

/// A character typed at a stop.  In a slot it replaces the `?`; next to
/// a text leaf it extends the leaf (the compiler reads the result);
/// elsewhere a value cannot start without an operator between it and the
/// part before it, so it is refused with the reason.  A space is only
/// meaningful after a number (a unit follows).
///
/// [at] is the caret's own byte when it stands past the stop — in the
/// whitespace a space just typed after a number opened (`90 |`): the
/// unit's letters go there, not back against the number.
KeyPlan characterAt(pb.FormulaProjection p, CaretStop stop, String char, {int? at}) {
  final root = p.root;
  final op = operatorFor(char);
  if (op != null) return operatorAt(p, stop, op);
  if (char == '(') return openParenAt(p, stop);
  final past = at != null && at > stop.offset && stop.side == StopSide.after;
  if (char == ' ') {
    final leaf = _touchedLeaf(root, stop);
    if (leaf != null && leaf.kind == 'number' && stop.side == StopSide.after && !past) {
      return EditText(TextEditPlan(stop.offset, stop.offset, ' '));
    }
    return const Refused();
  }
  if (!_isWordChar(char)) return const Refused();
  if (stop.side == StopSide.inSlot) {
    final slot = findNode(root, stop.node);
    if (slot == null) return const Refused();
    return EditText(TextEditPlan(slot.range.start, slot.range.end, char));
  }
  final leaf = _touchedLeaf(root, stop);
  if (leaf != null) {
    final offset = past && leaf.kind == 'number' ? at : stop.offset;
    return EditText(TextEditPlan(offset, offset, char));
  }
  // just inside an empty pair of parentheses, or right after `(`: a
  // value may start (the parser reads `(x`… once complete)
  if (stop.side == StopSide.open) {
    return EditText(TextEditPlan(stop.offset, stop.offset, char));
  }
  return const Refused(Refused.needsOperator);
}

/// An operator typed at a stop: the compiler puts it after (or before)
/// the part the stop belongs to, with a slot for the other side.  Inside
/// a leaf the operator applies to the whole leaf; `open` puts the
/// operator before the part; `!` negates the part in place.
KeyPlan operatorAt(pb.FormulaProjection p, CaretStop stop, String op) {
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
/// (`(?)`); elsewhere nothing.
KeyPlan openParenAt(pb.FormulaProjection p, CaretStop stop) {
  final n = findNode(p.root, stop.node);
  if (n == null) return const Refused();
  if (n.kind == 'reference' && stop.side != StopSide.before) {
    return Structural(pb.ComposeAction(nodeId: n.id, apply: pb.Unit()));
  }
  if (stop.side == StopSide.inSlot) {
    return EditText(TextEditPlan(n.range.start, n.range.end, '(?)'));
  }
  return const Refused();
}

/// Backspace at a stop: the character before the caret inside or right
/// after a text leaf (the last character of a one-character leaf removes
/// the leaf: it becomes a slot, the compiler's `remove`); a whole
/// structural part right after the caret is removed the same way; in a
/// slot, its operator goes with it (the compiler's rule); before a part
/// or just inside a parenthesis the caret only moves back.
KeyPlan backspaceAt(pb.FormulaProjection p, List<CaretStop> stops, CaretStop stop, {int? at}) {
  final root = p.root;
  final n = findNode(root, stop.node);
  if (n == null) return const Refused();
  // past the stop, in whitespace a space opened: that character goes
  if (at != null && at > stop.offset && stop.side == StopSide.after) {
    return EditText(TextEditPlan(at - 1, at, ''));
  }
  switch (stop.side) {
    case StopSide.inside:
      final paren = wrappedInParens(n);
      final start = n.range.start + (paren ? 1 : 0);
      if (stop.offset - 1 == start && n.range.end - (paren ? 1 : 0) - start == 1) {
        return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
      }
      return EditText(TextEditPlan(stop.offset - 1, stop.offset, ''));
    case StopSide.after:
      if (isTextLeaf(n)) {
        final paren = wrappedInParens(n);
        final start = n.range.start + (paren ? 1 : 0);
        final end = n.range.end - (paren ? 1 : 0);
        if (end - start <= 1) {
          return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
        }
        return EditText(TextEditPlan(end - 1, end, ''));
      }
      return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
    case StopSide.inSlot:
      return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
    case StopSide.close:
      // the last part inside the parentheses, if any, is what precedes
      final last = n.children.isEmpty ? null : n.children.last;
      if (last == null) return const Refused();
      final after = stops.where((s) => s.node == last.id && s.side == StopSide.after).firstOrNull;
      return after == null ? const Refused() : MoveTo(after);
    case StopSide.before:
    case StopSide.open:
      final prev = previousStop(stops, stop);
      return prev == null ? const Refused() : MoveTo(prev);
  }
}

/// Delete (forward) at a stop: the mirror of backspace.
KeyPlan deleteAt(pb.FormulaProjection p, List<CaretStop> stops, CaretStop stop) {
  final root = p.root;
  final n = findNode(root, stop.node);
  if (n == null) return const Refused();
  switch (stop.side) {
    case StopSide.inside:
      final paren = wrappedInParens(n);
      final start = n.range.start + (paren ? 1 : 0);
      final end = n.range.end - (paren ? 1 : 0);
      if (end - start == 1) {
        return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
      }
      return EditText(TextEditPlan(stop.offset, stop.offset + 1, ''));
    case StopSide.before:
      if (isTextLeaf(n)) {
        final paren = wrappedInParens(n);
        final start = n.range.start + (paren ? 1 : 0);
        final end = n.range.end - (paren ? 1 : 0);
        if (end - start <= 1) {
          return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
        }
        return EditText(TextEditPlan(start, start + 1, ''));
      }
      return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
    case StopSide.inSlot:
      return Structural(pb.ComposeAction(nodeId: n.id, remove: pb.Unit()));
    case StopSide.after:
    case StopSide.close:
    case StopSide.open:
      final next = nextStop(stops, stop);
      return next == null ? const Refused() : MoveTo(next);
  }
}

/// The region of [current] that differs from [old], as a byte range into
/// [old] and the replacement's length: `null` when the texts are equal.
({int start, int end, int newLength})? changedRegion(String old, String current) {
  if (old == current) return null;
  var prefix = 0;
  final max = old.length < current.length ? old.length : current.length;
  while (prefix < max && old.codeUnitAt(prefix) == current.codeUnitAt(prefix)) {
    prefix++;
  }
  var suffix = 0;
  while (suffix < max - prefix &&
      old.codeUnitAt(old.length - 1 - suffix) == current.codeUnitAt(current.length - 1 - suffix)) {
    suffix++;
  }
  return (start: prefix, end: old.length - suffix, newLength: current.length - prefix - suffix);
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
