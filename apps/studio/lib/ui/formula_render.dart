/// The rendered formula: the compiler's `FormulaProjection` drawn as
/// mathematical structure — a quotient as a fraction, a call with tall
/// parentheses around its arguments, a choice as a branch, a match as its
/// cases, a block as its bindings over its result, a temporal boundary as
/// a marked region — the same picture in the inspector's Formula view
/// (editable) and on an expanded canvas node (read only).
///
/// Nothing here parses or decides: the tree, every kind, every text and
/// every expected type is the compiler's (ADR-0028); this file owns layout,
/// hit-testing and the reading assistive technology hears.  Layout is
/// derived from the tree on every build and never persisted (§21).
///
/// Encodings, by channel (docs/architecture/studio-ui.md §4b): a **slot** is
/// a dashed hollow chip; a **reference** a chip with its concept's socket
/// glyph, a local in italics; a **literal** its coordinate with the unit
/// in the secondary colour; **operators** their glyphs, the logical ones
/// their words in the keyword weight; a **fraction** numerator over a rule
/// over denominator; a **choice** the condition in a tinted row behind a
/// ◇, then the `then` and `else` outcomes on their own rows joined by a
/// connector to a result row saying what both give; a **match** the value
/// and one row per case (`pattern ⇒ outcome`); a **block** one row per
/// `let` and the result under a rule; a **temporal** boundary a region with
/// a bar on its left and its word.  Selection is the selection tint; a
/// finding a red underline on the part.
library;

import 'package:flutter/material.dart';

import '../app/caret.dart' show wrappedInParens;
import '../l10n/l10n.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

/// Where each part was laid out, for hit-testing and the caret: one key
/// per node id, plus `<id>/inner` for a parenthesised part's inside and
/// `<id>/text` for a text leaf's own text.  Filled by the render, read by
/// the composer after layout.
class FormulaGeometry {
  final Map<String, GlobalKey> _keys = {};

  GlobalKey keyFor(String id) => _keys.putIfAbsent(id, () => GlobalKey(debugLabel: id));

  /// The rect of a laid-out part in [ancestor]'s coordinates; `null`
  /// before layout or for a part not drawn.
  Rect? rectOf(String id, RenderBox ancestor) {
    final ctx = _keys[id]?.currentContext;
    final box = ctx?.findRenderObject() as RenderBox?;
    if (box == null || !box.hasSize || !box.attached) return null;
    final origin = box.localToGlobal(Offset.zero, ancestor: ancestor);
    return origin & box.size;
  }

  /// The text style a leaf's own text was drawn with, so a caret inside
  /// it can be measured with the same metrics.
  final Map<String, TextStyle> textStyles = {};
}

/// The text of a part being typed into, shown in place of the part until
/// the compiler has read it (`FormulaComposer`'s pending region).
class PendingText {
  const PendingText({required this.nodeId, required this.text});
  final String nodeId;
  final String text;
}

/// The reading of a projection node for assistive technology, in product
/// language: what the part is, said in the order it is read.
String describeNode(pb.FormulaNode n, AppLocalizations l10n) {
  String c(int i) => n.children.length > i ? describeNode(n.children[i], l10n) : '';
  return switch (n.kind) {
    'slot' => n.hasExpected() ? '${l10n.caretIn}, ${n.expected.description}' : l10n.caretIn,
    'reference' => n.local ? 'local ${n.name}' : n.name,
    'number' => n.coordinate,
    'quantity' => '${n.coordinate} ${unitTextOf(n)}',
    'bool' => n.name,
    'binary' when n.name == '/' => l10n.fractionOf(c(0), c(1)),
    'binary' || 'compare' => '${c(0)} ${operatorWord(n.name)} ${c(1)}',
    'unary' => '${operatorWord(n.name)} ${c(0)}',
    'call' => '${n.name} of ${[for (var i = 0; i < n.children.length; i++) c(i)].join(', ')}',
    'if' => l10n.choiceSemantics(c(0), c(1), c(2)),
    'match' => l10n.matchSemantics(c(0), n.children.length - 1),
    'arm' => '${n.name} gives ${c(0)}',
    'block' => l10n.blockSemantics(n.children.length - 1),
    'let' => 'let ${n.name} be ${c(0)}',
    'rule' => '${n.binds.join(', ')} gives ${c(0)}',
    'list' => 'a collection of ${[for (var i = 0; i < n.children.length; i++) c(i)].join(', ')}',
    'tuple' => 'a group of ${[for (var i = 0; i < n.children.length; i++) c(i)].join(', ')}',
    'delay' || 'sync' =>
      '${l10n.temporalSemantics(n.kind)} of ${[for (var i = 0; i < n.children.length; i++) c(i)].join(', ')}',
    'binder' => '${n.name} ${n.param} in ${c(0)}: ${c(1)}',
    'range' => '${c(0)} to ${c(1)}',
    _ => n.text,
  };
}

/// The unit of a quantity literal as the compiler renders it: a composite
/// (`m/s²`) comes as `unit_display`; a plain symbol as `unit`.
String unitTextOf(pb.FormulaNode n) => n.unitDisplay.isNotEmpty ? n.unitDisplay : n.unit;

String operatorGlyph(String op) => switch (op) {
  '*' => '×',
  '/' => '÷',
  '-' => '−',
  '<=' => '≤',
  '>=' => '≥',
  '!=' => '≠',
  '&&' => 'and',
  '||' => 'or',
  '!' => 'not',
  _ => op,
};

/// The operator's reading, for assistive technology.
String operatorWord(String op) => switch (op) {
  '*' => 'times',
  '/' => 'over',
  '-' => 'minus',
  '+' => 'plus',
  '<' => 'less than',
  '<=' => 'at most',
  '>' => 'greater than',
  '>=' => 'at least',
  '==' => 'equals',
  '!=' => 'differs from',
  '&&' => 'and',
  '||' => 'or',
  '!' => 'not',
  'in' => 'in',
  '??' => 'or else',
  _ => op,
};

/// The operators drawn as words of the language rather than symbols.
bool isWordOperator(String op) => op == '&&' || op == '||' || op == '!';

/// The rendered tree.  [onTapNode] receives the node and the tap's
/// position in the render's coordinates (the composer places the caret
/// from it); `null` makes the render read only.
class FormulaRender extends StatelessWidget {
  const FormulaRender({
    super.key,
    required this.projection,
    required this.concepts,
    required this.geometry,
    this.selected,
    this.pending,
    this.onTapNode,
    this.dense = false,
    this.resultDescription,
    this.units = const [],
    this.onUnit,
  });

  final pb.FormulaProjection projection;
  final Map<int, pb.ConceptView> concepts;
  final FormulaGeometry geometry;
  final String? selected;

  /// A part shown as the text being typed into it.
  final PendingText? pending;
  final void Function(pb.FormulaNode node, Offset at)? onTapNode;

  /// The compiler's units for the selected literal's own dimension (its
  /// pop-up switches the unit and keeps the quantity), and what a chosen
  /// unit id does.
  final List<pb.UnitCandidate> units;
  final void Function(String nodeId, String unitId)? onUnit;

  /// A smaller, tighter picture (the canvas node).
  final bool dense;

  /// What the whole formula gives, for the result rows of choices at the
  /// root (`projection.result`'s description when absent).
  final String? resultDescription;

  @override
  Widget build(BuildContext context) {
    if (!projection.hasRoot()) return const SizedBox.shrink();
    return _Node(
      node: projection.root,
      ctx: _RenderContext(
        concepts: concepts,
        geometry: geometry,
        selected: selected,
        pending: pending,
        onTapNode: onTapNode,
        dense: dense,
        l10n: context.l10n,
        units: units,
        onUnit: onUnit,
        result:
            resultDescription ?? (projection.hasResult() ? projection.result.description : null),
      ),
    );
  }
}

class _RenderContext {
  const _RenderContext({
    required this.concepts,
    required this.geometry,
    required this.selected,
    required this.pending,
    required this.onTapNode,
    required this.dense,
    required this.l10n,
    required this.result,
    this.units = const [],
    this.onUnit,
  });
  final Map<int, pb.ConceptView> concepts;
  final FormulaGeometry geometry;
  final String? selected;
  final PendingText? pending;
  final void Function(pb.FormulaNode node, Offset at)? onTapNode;
  final bool dense;
  final AppLocalizations l10n;
  final String? result;
  final List<pb.UnitCandidate> units;
  final void Function(String nodeId, String unitId)? onUnit;

  double get body => dense ? MacType.secondary : MacType.body;
  double get small => dense ? MacType.caption : MacType.secondary;
}

/// One node, dispatched by kind; parentheses in its text draw as a group
/// around it.
class _Node extends StatelessWidget {
  const _Node({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final n = node;
    final pending = ctx.pending;
    if (pending != null && pending.nodeId == n.id) {
      return _Pending(node: n, text: pending.text, ctx: ctx);
    }
    final grouped =
        n.kind != 'call' && n.kind != 'opaque' && n.kind != 'slot' && wrappedInParens(n);
    Widget inner = switch (n.kind) {
      'slot' => _Slot(node: n, ctx: ctx),
      'reference' => _Reference(node: n, ctx: ctx),
      'number' || 'quantity' => _Literal(node: n, ctx: ctx),
      'bool' => _Word(node: n, text: n.name, ctx: ctx),
      'binary' when n.name == '/' => _Fraction(node: n, ctx: ctx),
      'binary' || 'compare' => _Infix(node: n, ctx: ctx),
      'unary' => _Prefix(node: n, ctx: ctx),
      'call' => _Call(node: n, ctx: ctx),
      'if' => _Choice(node: n, ctx: ctx),
      'match' => _Match(node: n, ctx: ctx),
      'block' => _Block(node: n, ctx: ctx),
      'delay' || 'sync' => _Temporal(node: n, ctx: ctx),
      'rule' => _Rule(node: n, ctx: ctx),
      'list' || 'tuple' => _Sequence(node: n, ctx: ctx),
      'binder' => _Binder(node: n, ctx: ctx),
      'range' => _Range(node: n, ctx: ctx),
      _ => _Opaque(node: n, ctx: ctx),
    };
    if (grouped) {
      inner = _Parens(
        innerKey: ctx.geometry.keyFor('${n.id}/inner'),
        selected: ctx.selected == n.id,
        ctx: ctx,
        child: inner,
      );
    }
    final t = MacTokens.of(context);
    final hasError = n.diagnostics.any(
      (d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
    );
    // a leaf carries its own key (its chip); a structure's key is the box
    // around everything it holds
    final leaf = _isLeaf(n.kind);
    Widget box = KeyedSubtree(key: leaf ? null : ctx.geometry.keyFor(n.id), child: inner);
    if (hasError && !leaf) {
      box = DecoratedBox(
        decoration: BoxDecoration(
          border: Border(bottom: BorderSide(color: t.error, width: 2)),
        ),
        child: box,
      );
    }
    return KeyedSubtree(
      key: ValueKey(leaf ? 'leaf-${n.id}' : 'struct-${n.id}'),
      child: Semantics(
        label: describeNode(n, ctx.l10n),
        selected: ctx.selected == n.id,
        container: !leaf,
        child: box,
      ),
    );
  }
}

bool _isLeaf(String kind) =>
    kind == 'slot' ||
    kind == 'reference' ||
    kind == 'number' ||
    kind == 'quantity' ||
    kind == 'bool' ||
    kind == 'opaque';

/// A tap on a part: the composer decides caret or selection from where
/// it landed.
void _tap(BuildContext context, _RenderContext ctx, pb.FormulaNode n, TapUpDetails d) {
  final f = ctx.onTapNode;
  if (f == null) return;
  f(n, d.globalPosition);
}

/// One selectable chip: a slot, a reference, a literal, a word.
class _Chip extends StatelessWidget {
  const _Chip({
    required this.node,
    required this.ctx,
    required this.child,
    this.dashed = false,
    this.local = false,
  });
  final pb.FormulaNode node;
  final _RenderContext ctx;
  final Widget child;
  final bool dashed;
  final bool local;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final selected = ctx.selected == node.id;
    final hasError = node.diagnostics.any(
      (d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
    );
    final pad = ctx.dense
        ? const EdgeInsets.symmetric(horizontal: 4, vertical: 1)
        : const EdgeInsets.symmetric(horizontal: 6, vertical: 3);
    final body = Container(
      key: ctx.geometry.keyFor(node.id),
      padding: pad,
      decoration: BoxDecoration(
        color: selected ? t.selection : null,
        borderRadius: BorderRadius.circular(4),
        border: dashed ? null : Border.all(color: local ? t.textTertiary : t.hairline),
      ),
      child: DecoratedBox(
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(color: hasError ? t.error : Colors.transparent, width: 2),
          ),
        ),
        child: dashed ? CustomPaint(painter: _DashedBorder(t.textTertiary), child: child) : child,
      ),
    );
    if (ctx.onTapNode == null) return KeyedSubtree(key: ValueKey('node-${node.id}'), child: body);
    return MouseRegion(
      key: ValueKey('node-${node.id}'),
      cursor: SystemMouseCursors.text,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: (d) => _tap(context, ctx, node, d),
        child: body,
      ),
    );
  }
}

class _Slot extends StatelessWidget {
  const _Slot({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return _Chip(
      node: node,
      ctx: ctx,
      dashed: true,
      child: Text(
        '?',
        key: ctx.geometry.keyFor('${node.id}/text'),
        style: TextStyle(fontSize: ctx.small, color: t.textSecondary),
      ),
    );
  }
}

TextStyle _nameStyle(MacTokens t, _RenderContext ctx, {required bool local}) => TextStyle(
  fontSize: ctx.body,
  color: t.textPrimary,
  fontStyle: local ? FontStyle.italic : FontStyle.normal,
);

class _Reference extends StatelessWidget {
  const _Reference({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final concept = n.hasEntity() && n.entity.hasConceptId()
        ? ctx.concepts[n.entity.conceptId.toInt()]
        : null;
    final style = _nameStyle(t, ctx, local: n.local);
    ctx.geometry.textStyles[n.id] = style;
    return _Chip(
      node: n,
      ctx: ctx,
      local: n.local,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        spacing: 4,
        children: [
          if (concept != null) SocketGlyph.of(concept, t, size: ctx.dense ? 7 : 9),
          Text(n.name, key: ctx.geometry.keyFor('${n.id}/text'), style: style),
        ],
      ),
    );
  }
}

/// A number, with its unit in the secondary colour beside the coordinate.
class _Literal extends StatelessWidget {
  const _Literal({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final style = TextStyle(
      fontSize: ctx.body,
      color: t.textPrimary,
      fontFeatures: const [FontFeature.tabularFigures()],
    );
    ctx.geometry.textStyles[n.id] = style;
    final inner = n.kind == 'quantity' ? '${n.coordinate} ${unitTextOf(n)}' : n.coordinate;
    final selected = ctx.selected == n.id;
    final chip = _Chip(
      node: n,
      ctx: ctx,
      child: Text.rich(
        TextSpan(
          children: [
            TextSpan(text: n.coordinate),
            if (n.kind == 'quantity')
              TextSpan(
                text: ' ${unitTextOf(n)}',
                style: TextStyle(color: t.textSecondary),
              ),
          ],
        ),
        key: ctx.geometry.keyFor('${n.id}/text'),
        style: style,
        semanticsLabel: inner,
      ),
    );
    final onUnit = ctx.onUnit;
    if (!selected || onUnit == null || ctx.onTapNode == null) return chip;
    // the selected literal's unit: a pop-up of the units of its own
    // dimension (the compiler's candidates), switching keeps the quantity
    final options = [for (final u in ctx.units) u.id];
    return Wrap(
      crossAxisAlignment: WrapCrossAlignment.center,
      spacing: 2,
      runSpacing: 2,
      children: [
        chip,
        MacDropdown<String>(
          key: ValueKey('unit-${n.id}'),
          compact: true,
          value: n.unitId.isEmpty ? null : n.unitId,
          hint: n.kind == 'quantity' ? unitTextOf(n) : 'unit',
          items: options.isEmpty && n.unitId.isNotEmpty ? [n.unitId] : options,
          labelOf: (id) =>
              ctx.units
                  .where((u) => u.id == id)
                  .map((u) => u.display.isNotEmpty ? u.display : u.symbol)
                  .firstOrNull ??
              (n.kind == 'quantity' ? unitTextOf(n) : id),
          onChanged: (id) => onUnit(n.id, id),
        ),
      ],
    );
  }
}

class _Word extends StatelessWidget {
  const _Word({required this.node, required this.text, required this.ctx});
  final pb.FormulaNode node;
  final String text;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return _Chip(
      node: node,
      ctx: ctx,
      child: Text(
        text,
        key: ctx.geometry.keyFor('${node.id}/text'),
        style: TextStyle(fontSize: ctx.body, color: t.textPrimary),
      ),
    );
  }
}

/// The pattern of a match arm or a let, as written (the part's `name`),
/// in the local's italics: the names it binds are the formula's own.
/// It is not a part of its own; tapping it selects the arm or the let.
class _Pattern extends StatelessWidget {
  const _Pattern({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final style = TextStyle(
      fontSize: ctx.body,
      fontFamily: 'Menlo',
      fontStyle: FontStyle.italic,
      color: t.textPrimary,
    );
    final box = Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: t.textTertiary),
      ),
      child: Text(node.name, key: ValueKey('pattern-${node.id}'), style: style),
    );
    final tap = ctx.onTapNode;
    if (tap == null) return box;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapUp: (d) => tap(node, d.globalPosition),
      child: box,
    );
  }
}

class _Opaque extends StatelessWidget {
  const _Opaque({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return _Chip(
      node: node,
      ctx: ctx,
      child: Text(
        node.text,
        key: ctx.geometry.keyFor('${node.id}/text'),
        style: TextStyle(fontSize: ctx.body, fontFamily: 'Menlo', color: t.textPrimary),
      ),
    );
  }
}

/// The part being typed into: its current text, monospace, until the
/// compiler has read it.
class _Pending extends StatelessWidget {
  const _Pending({required this.node, required this.text, required this.ctx});
  final pb.FormulaNode node;
  final String text;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final style = TextStyle(fontSize: ctx.body, fontFamily: 'Menlo', color: t.textPrimary);
    ctx.geometry.textStyles[node.id] = style;
    return Container(
      key: ctx.geometry.keyFor(node.id),
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: t.accent),
      ),
      child: Text(
        text.isEmpty ? ' ' : text,
        key: ctx.geometry.keyFor('${node.id}/text'),
        style: style,
      ),
    );
  }
}

/// An operator or keyword glyph; selectable when it names a node.
class _Glyph extends StatelessWidget {
  const _Glyph(this.text, {required this.ctx, this.node, this.keyword = false, this.key_});
  final String text;
  final _RenderContext ctx;
  final pb.FormulaNode? node;
  final bool keyword;
  final Key? key_;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final selected = n != null && ctx.selected == n.id;
    final hasError =
        n != null &&
        n.diagnostics.any((d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR);
    final label = Text(
      text,
      key: key_,
      style: TextStyle(
        fontSize: ctx.body,
        color: hasError ? t.error : (keyword ? t.textPrimary : t.textSecondary),
        fontWeight: selected || keyword ? FontWeight.w600 : FontWeight.w400,
      ),
    );
    if (n == null || ctx.onTapNode == null) {
      return Padding(padding: const EdgeInsets.symmetric(horizontal: 2), child: label);
    }
    return MacInteractive(
      selected: selected,
      radius: 4,
      onTap: () {},
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: (d) => _tap(context, ctx, n, d),
        child: label,
      ),
    );
  }
}

/// Parts side by side, centred on the math axis, wrapping when the field
/// is narrow.
class _MathRow extends StatelessWidget {
  const _MathRow({required this.children});
  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Wrap(
    crossAxisAlignment: WrapCrossAlignment.center,
    spacing: 4,
    runSpacing: 4,
    children: children,
  );
}

class _Infix extends StatelessWidget {
  const _Infix({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) => _MathRow(
    children: [
      _Node(node: node.children[0], ctx: ctx),
      _Glyph(
        operatorGlyph(node.name),
        ctx: ctx,
        node: node,
        keyword: isWordOperator(node.name),
        key_: ValueKey('node-${node.id}'),
      ),
      _Node(node: node.children[1], ctx: ctx),
    ],
  );
}

class _Prefix extends StatelessWidget {
  const _Prefix({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) => _MathRow(
    children: [
      _Glyph(
        operatorGlyph(node.name),
        ctx: ctx,
        node: node,
        keyword: isWordOperator(node.name),
        key_: ValueKey('node-${node.id}'),
      ),
      _Node(node: node.children[0], ctx: ctx),
    ],
  );
}

/// `a / b` as a fraction: numerator over a rule over denominator.  The
/// rule is the quotient's own selectable mark.
class _Fraction extends StatelessWidget {
  const _Fraction({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final selected = ctx.selected == node.id;
    final rule = Container(
      key: ValueKey('fraction-${node.id}'),
      height: selected ? 2 : 1.2,
      margin: const EdgeInsets.symmetric(vertical: 3),
      color: selected ? t.accent : t.textSecondary,
    );
    final column = IntrinsicWidth(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Center(
            child: _Node(node: node.children[0], ctx: ctx),
          ),
          ctx.onTapNode == null
              ? rule
              : GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTapUp: (d) => _tap(context, ctx, node, d),
                  child: Padding(padding: const EdgeInsets.symmetric(vertical: 2), child: rule),
                ),
          Center(
            child: _Node(node: node.children[1], ctx: ctx),
          ),
        ],
      ),
    );
    return Padding(padding: const EdgeInsets.symmetric(horizontal: 4), child: column);
  }
}

/// Tall parentheses around a part, drawn to its height.
class _Parens extends StatelessWidget {
  const _Parens({
    required this.child,
    required this.ctx,
    required this.innerKey,
    this.selected = false,
    this.node,
  });
  final Widget child;
  final _RenderContext ctx;
  final GlobalKey innerKey;
  final bool selected;

  /// The node whose parentheses these are (a call's): a tap selects it.
  final pb.FormulaNode? node;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final color = selected ? t.accent : t.textSecondary;
    Widget paren(bool open) {
      final p = CustomPaint(
        size: Size(ctx.dense ? 5 : 7, 1),
        painter: _ParenPainter(color: color, open: open, width: ctx.dense ? 5 : 7),
      );
      final n = node;
      if (n == null || ctx.onTapNode == null) return p;
      return GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: (d) => _tap(context, ctx, n, d),
        child: p,
      );
    }

    return IntrinsicHeight(
      child: Row(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          paren(true),
          Flexible(
            child: KeyedSubtree(
              key: innerKey,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 2),
                child: Center(child: child),
              ),
            ),
          ),
          paren(false),
        ],
      ),
    );
  }
}

class _ParenPainter extends CustomPainter {
  const _ParenPainter({required this.color, required this.open, required this.width});
  final Color color;
  final bool open;
  final double width;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;
    final h = size.height;
    final bow = width * 0.9;
    final path = Path();
    if (open) {
      path.moveTo(width, 1);
      path.quadraticBezierTo(width - bow - 1, h / 2, width, h - 1);
    } else {
      path.moveTo(0, 1);
      path.quadraticBezierTo(bow + 1, h / 2, 0, h - 1);
    }
    canvas.drawPath(path, paint);
  }

  @override
  bool shouldRepaint(_ParenPainter old) =>
      old.color != color || old.open != open || old.width != width;
}

/// `name(a, b, c)`: the name, then the arguments between tall
/// parentheses, separated by commas.
class _Call extends StatelessWidget {
  const _Call({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final selected = ctx.selected == n.id;
    final name = Text(
      n.name,
      key: ValueKey('node-${n.id}'),
      style: TextStyle(
        fontSize: ctx.body,
        color: t.textPrimary,
        fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
      ),
    );
    final head = ctx.onTapNode == null
        ? name
        : MacInteractive(
            selected: selected,
            radius: 4,
            onTap: () {},
            padding: const EdgeInsets.symmetric(horizontal: 3, vertical: 2),
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTapUp: (d) => _tap(context, ctx, n, d),
              child: name,
            ),
          );
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        head,
        // the arguments wrap inside the parentheses when the field is narrow
        Flexible(
          child: _Parens(
            innerKey: ctx.geometry.keyFor('${n.id}/inner'),
            selected: selected,
            ctx: ctx,
            node: n,
            child: _MathRow(
              children: [
                for (var i = 0; i < n.children.length; i++) ...[
                  if (i > 0) _Glyph(',', ctx: ctx),
                  _Node(node: n.children[i], ctx: ctx),
                ],
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// A connector along the left of a structured form's rows: a spine from
/// the head to each branch and on to the result.
class _SpinePainter extends CustomPainter {
  const _SpinePainter({required this.color, required this.rows});
  final Color color;

  /// The y of each row's centre, in order: the head, the branches, the
  /// result last.
  final List<double> rows;

  @override
  void paint(Canvas canvas, Size size) {
    if (rows.length < 2) return;
    final paint = Paint()
      ..color = color
      ..strokeWidth = 1.2
      ..style = PaintingStyle.stroke;
    const x = 8.0;
    canvas.drawLine(Offset(x, rows.first), Offset(x, rows.last), paint);
    for (var i = 1; i < rows.length; i++) {
      canvas.drawLine(Offset(x, rows[i]), Offset(size.width, rows[i]), paint);
    }
    // the result: an arrowhead into the last row
    canvas.drawCircle(Offset(x, rows.last), 2.2, Paint()..color = color);
  }

  @override
  bool shouldRepaint(_SpinePainter old) => old.color != color || old.rows != rows;
}

/// Rows joined by a spine on the left: the head (its own glyph), then
/// each branch behind a word, then the result row.
class _Structured extends StatefulWidget {
  const _Structured({super.key, required this.head, required this.branches, required this.result});
  final Widget head;
  final List<Widget> branches;
  final Widget? result;

  @override
  State<_Structured> createState() => _StructuredState();
}

class _StructuredState extends State<_Structured> {
  final List<GlobalKey> _rowKeys = [];
  List<double> _rows = const [];

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final rows = [widget.head, ...widget.branches, if (widget.result != null) widget.result!];
    while (_rowKeys.length < rows.length) {
      _rowKeys.add(GlobalKey());
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final box = context.findRenderObject() as RenderBox?;
      if (box == null || !box.hasSize) return;
      final ys = <double>[];
      for (var i = 0; i < rows.length; i++) {
        final r = _rowKeys[i].currentContext?.findRenderObject() as RenderBox?;
        if (r == null || !r.hasSize) return;
        final o = r.localToGlobal(Offset.zero, ancestor: box);
        ys.add(o.dy + r.size.height / 2);
      }
      if (ys.length != _rows.length || Iterable.generate(ys.length).any((i) => ys[i] != _rows[i])) {
        setState(() => _rows = ys);
      }
    });
    return CustomPaint(
      painter: _SpinePainter(color: t.textTertiary, rows: _rows),
      child: Padding(
        padding: const EdgeInsets.only(left: 18),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            for (var i = 0; i < rows.length; i++)
              Padding(
                key: _rowKeys[i],
                padding: EdgeInsets.only(top: i == 0 ? 0 : 4),
                child: rows[i],
              ),
          ],
        ),
      ),
    );
  }
}

/// The result row of a structured form: what every branch gives.
Widget _resultRow(BuildContext context, _RenderContext ctx, pb.FormulaNode n) {
  final t = MacTokens.of(context);
  final what = n.hasExpected()
      ? n.expected.description
      : (n.id == 'r' ? ctx.result : null) ?? (n.hasActual() ? n.actual.description : null);
  if (what == null) return const SizedBox.shrink();
  return Text.rich(
    TextSpan(
      children: [
        TextSpan(
          text: ctx.l10n.resultLabel,
          style: TextStyle(fontSize: ctx.small, color: t.textTertiary),
        ),
        const TextSpan(text: '  '),
        TextSpan(
          text: what,
          style: TextStyle(fontSize: ctx.small, color: t.textSecondary),
        ),
      ],
    ),
    key: ValueKey('result-${n.id}'),
    softWrap: true,
  );
}

/// `if c then a else b`: the condition behind a ◇ in a tinted row, the
/// two outcomes behind their words, the result they both give.
class _Choice extends StatelessWidget {
  const _Choice({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final selected = ctx.selected == n.id;
    final head = Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: selected ? t.selection : t.open.withValues(alpha: 0.10),
        borderRadius: BorderRadius.circular(5),
      ),
      child: _MathRow(
        children: [
          _Glyph('◇', ctx: ctx, node: n, key_: ValueKey('node-${n.id}')),
          _Glyph('if', ctx: ctx, node: n, keyword: true),
          _Node(node: n.children[0], ctx: ctx),
        ],
      ),
    );
    Widget branch(String word, pb.FormulaNode child) => _MathRow(
      children: [
        _Glyph(word, ctx: ctx, node: n, keyword: true),
        _Node(node: child, ctx: ctx),
      ],
    );
    return _Structured(
      key: ValueKey('if-${n.id}'),
      head: head,
      branches: [branch('then', n.children[1]), branch('else', n.children[2])],
      result: _resultRow(context, ctx, n),
    );
  }
}

/// `match v { p ⇒ a, … }`: the matched value behind the word, one row per
/// case, the result they all give.
class _Match extends StatelessWidget {
  const _Match({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final selected = ctx.selected == n.id;
    final head = Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: selected ? t.selection : t.open.withValues(alpha: 0.10),
        borderRadius: BorderRadius.circular(5),
      ),
      child: _MathRow(
        children: [
          _Glyph('match', ctx: ctx, node: n, keyword: true, key_: ValueKey('node-${n.id}')),
          _Node(node: n.children[0], ctx: ctx),
        ],
      ),
    );
    return _Structured(
      key: ValueKey('match-${n.id}'),
      head: head,
      branches: [
        for (final arm in n.children.skip(1))
          Semantics(
            label: describeNode(arm, ctx.l10n),
            container: true,
            child: KeyedSubtree(
              key: ctx.geometry.keyFor(arm.id),
              child: _MathRow(
                children: [
                  _Pattern(node: arm, ctx: ctx),
                  _Glyph('⇒', ctx: ctx, node: arm, key_: ValueKey('node-${arm.id}')),
                  if (arm.children.isNotEmpty) _Node(node: arm.children[0], ctx: ctx),
                ],
              ),
            ),
          ),
      ],
      result: _resultRow(context, ctx, n),
    );
  }
}

/// `{ let p = v; … r }`: one row per binding, the result under a rule.
class _Block extends StatelessWidget {
  const _Block({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final lets = n.children.take(n.children.length - 1);
    final tail = n.children.last;
    return Container(
      key: ValueKey('block-${n.id}'),
      padding: const EdgeInsets.fromLTRB(8, 4, 8, 4),
      decoration: BoxDecoration(
        border: Border(left: BorderSide(color: t.textTertiary, width: 2)),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (final l in lets)
            Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: Semantics(
                label: describeNode(l, ctx.l10n),
                container: true,
                child: KeyedSubtree(
                  key: ctx.geometry.keyFor(l.id),
                  child: _MathRow(
                    children: [
                      _Glyph(
                        'let',
                        ctx: ctx,
                        node: l,
                        keyword: true,
                        key_: ValueKey('node-${l.id}'),
                      ),
                      _Pattern(node: l, ctx: ctx),
                      _Glyph('=', ctx: ctx, node: l),
                      if (l.children.isNotEmpty) _Node(node: l.children[0], ctx: ctx),
                    ],
                  ),
                ),
              ),
            ),
          Container(
            height: 1,
            width: 48,
            color: t.textTertiary,
            margin: const EdgeInsets.only(bottom: 4),
          ),
          _Node(node: tail, ctx: ctx),
          Padding(padding: const EdgeInsets.only(top: 2), child: _resultRow(context, ctx, n)),
        ],
      ),
    );
  }
}

/// `delay(init, e)` / `sync(domain, init, e)`: a region with a bar on its
/// left — the temporal boundary — its word and its arguments.
class _Temporal extends StatelessWidget {
  const _Temporal({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final selected = ctx.selected == n.id;
    return Container(
      key: ValueKey('temporal-${n.id}'),
      padding: const EdgeInsets.fromLTRB(8, 3, 6, 3),
      decoration: BoxDecoration(
        color: selected ? t.selection : t.settled.withValues(alpha: 0.08),
        borderRadius: BorderRadius.circular(5),
        border: Border(left: BorderSide(color: t.settled, width: 3)),
      ),
      child: _MathRow(
        children: [
          _Glyph(n.kind, ctx: ctx, node: n, keyword: true, key_: ValueKey('node-${n.id}')),
          for (var i = 0; i < n.children.length; i++) ...[
            if (i > 0) _Glyph(',', ctx: ctx),
            _Node(node: n.children[i], ctx: ctx),
          ],
        ],
      ),
    );
  }
}

/// `x => body` — a rule given to an equation: its parameters in the
/// local's italics, the arrow, the body.
class _Rule extends StatelessWidget {
  const _Rule({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    return _MathRow(
      children: [
        for (final p in n.binds)
          Text(
            p,
            style: TextStyle(
              fontSize: ctx.body,
              fontFamily: 'Menlo',
              fontStyle: FontStyle.italic,
              color: t.textPrimary,
            ),
          ),
        _Glyph('⇒', ctx: ctx, node: n, key_: ValueKey('node-${n.id}')),
        if (n.children.isNotEmpty) _Node(node: n.children[0], ctx: ctx),
      ],
    );
  }
}

/// `[a, b, c]` — a collection; `(a, b)` — a group: the items between
/// their delimiters, separated by commas.
class _Sequence extends StatelessWidget {
  const _Sequence({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final n = node;
    final (open, close) = n.kind == 'list' ? ('[', ']') : ('(', ')');
    return _MathRow(
      children: [
        _Glyph(open, ctx: ctx, node: n, key_: ValueKey('node-${n.id}')),
        for (var i = 0; i < n.children.length; i++) ...[
          if (i > 0) _Glyph(',', ctx: ctx),
          _Node(node: n.children[i], ctx: ctx),
        ],
        _Glyph(close, ctx: ctx, node: n),
      ],
    );
  }
}

/// `all x in xs: body` — the head on one line, the body indented under.
class _Binder extends StatelessWidget {
  const _Binder({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = node;
    final selected = ctx.selected == n.id;
    final local = Container(
      key: ValueKey('binder-local-${n.id}'),
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
      decoration: BoxDecoration(
        color: selected ? t.selection : null,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: t.textTertiary),
      ),
      child: Text(n.param, style: _nameStyle(t, ctx, local: true)),
    );
    return Column(
      key: ValueKey('binder-${n.id}'),
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _MathRow(
          children: [
            _Glyph(n.name, ctx: ctx, node: n, keyword: true, key_: ValueKey('node-${n.id}')),
            ctx.onTapNode == null
                ? local
                : GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTapUp: (d) => _tap(context, ctx, n, d),
                    child: local,
                  ),
            _Glyph('in', ctx: ctx, keyword: true),
            _Node(node: n.children[0], ctx: ctx),
            _Glyph(':', ctx: ctx, node: n),
          ],
        ),
        Padding(
          padding: const EdgeInsets.only(left: 16, top: 4),
          child: _Node(node: n.children[1], ctx: ctx),
        ),
      ],
    );
  }
}

class _Range extends StatelessWidget {
  const _Range({required this.node, required this.ctx});
  final pb.FormulaNode node;
  final _RenderContext ctx;

  @override
  Widget build(BuildContext context) => _MathRow(
    children: [
      _Node(node: node.children[0], ctx: ctx),
      _Glyph('..', ctx: ctx, node: node, key_: ValueKey('node-${node.id}')),
      _Node(node: node.children[1], ctx: ctx),
    ],
  );
}

class _DashedBorder extends CustomPainter {
  _DashedBorder(this.color);
  final Color color;
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    const dash = 3.0;
    final rect = Rect.fromLTWH(-6, -3, size.width + 12, size.height + 6);
    final path = Path()..addRRect(RRect.fromRectAndRadius(rect, const Radius.circular(4)));
    for (final metric in path.computeMetrics()) {
      var d = 0.0;
      while (d < metric.length) {
        canvas.drawPath(metric.extractPath(d, d + dash), paint);
        d += dash * 2;
      }
    }
  }

  @override
  bool shouldRepaint(_DashedBorder old) => old.color != color;
}
