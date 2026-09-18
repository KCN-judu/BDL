/// The Formula Composer: the structured projection of a definition draft,
/// rendered as the expression it is — `clamp( [Tilt] ÷ [90][deg ▾], [0], [1] )`
/// — with what the selected position expects and what fits beneath it.
///
/// Studio draws what the compiler projected (`FormulaProjection`) and asks
/// it for every candidate (`GetFormulaSlot`); every action here becomes a
/// `ComposeFormula` request whose answer is ordinary draft text.  Nothing
/// in this file parses, types, converts a unit or decides what fits.
///
/// What the designer sees, ranked: the selected slot's expectation and the
/// candidates for it (the decision at each step); the structure of the
/// expression (operators, calls, literals, references, slots — by shape);
/// the compiler's objections on the component they concern; the kinds of
/// the components (on selection).  Encodings: a slot is a dashed hollow
/// box (dashed = not decided, as on the canvas); a reference is a chip with
/// its concept's socket glyph (shape = kind of value); a literal is two
/// fields, the coordinate and its unit pop-up (18A: only a literal owns a
/// unit picker; a reference's kind comes from its declaration); an
/// operator is its glyph; a call is its name and parentheses; an
/// unsupported form is its text.  Selection is the selection tint;
/// keyboard focus the accent ring; an error the diagnostic row's words and
/// the red underline — never colour alone.
///
/// Keys: Tab moves between components in reading order; on a selected
/// component `+ − * /` insert that operator after it, ⌫ removes it; in a
/// slot's number entry digits type and Return inserts; Esc clears the
/// selection.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/canvas_geometry.dart';
import 'canvas/concept_glyphs.dart';
import 'mac/controls.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

class FormulaComposer extends StatefulWidget {
  const FormulaComposer({
    super.key,
    required this.mappingId,
    required this.source,
    required this.projection,
    required this.composer,
    required this.concepts,
    required this.dispatch,
    this.outOfSync = false,
    this.onEditAsText,
  });

  final int mappingId;

  /// The text the projection is of (the draft's, else the committed).
  final String source;

  /// The compiler's structured view; `null` while none has arrived.
  final pb.FormulaProjection? projection;
  final ComposerState composer;

  /// The concepts of the design, for a reference's socket glyph.
  final Map<int, pb.ConceptView> concepts;
  final void Function(AppAction) dispatch;

  /// The text on screen is not what the projection is of (it did not
  /// parse, or the verdict has not arrived): the tree is shown dimmed and
  /// read-only.
  final bool outOfSync;
  final VoidCallback? onEditAsText;

  @override
  State<FormulaComposer> createState() => _FormulaComposerState();
}

class _FormulaComposerState extends State<FormulaComposer> {
  final FocusNode _focus = FocusNode(debugLabel: 'composer');

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  String? get _selected => widget.composer.selectedNode;

  void _select(String? id) =>
      widget.dispatch(FormulaNodeSelected(mappingId: widget.mappingId, nodeId: id));

  void _compose(pb.ComposeAction a) =>
      widget.dispatch(ComposeRequested(mappingId: widget.mappingId, action: a));

  pb.FormulaNode? _find(pb.FormulaNode? n, String id) {
    if (n == null) return null;
    if (n.id == id) return n;
    for (final c in n.children) {
      final f = _find(c, id);
      if (f != null) return f;
    }
    return null;
  }

  /// Keys on the field while a component is selected and no text entry
  /// inside it has focus.
  KeyEventResult _onKey(FocusNode node, KeyEvent e) {
    if (e is! KeyDownEvent) return KeyEventResult.ignored;
    final id = _selected;
    if (id == null || widget.outOfSync || widget.composer.pendingCompose) {
      return KeyEventResult.ignored;
    }
    // a text entry (a coordinate) owns its own keys
    if (FocusManager.instance.primaryFocus?.context?.widget is EditableText) {
      return KeyEventResult.ignored;
    }
    final ch = e.character;
    final op = switch (ch) {
      '+' => '+',
      '-' => '-',
      '*' => '*',
      '/' => '/',
      _ => null,
    };
    if (op != null) {
      _compose(
        pb.ComposeAction(
          nodeId: id,
          operator: pb.ComposeOperator(op: op, before: false),
        ),
      );
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.backspace || e.logicalKey == LogicalKeyboardKey.delete) {
      _compose(pb.ComposeAction(nodeId: id, remove: pb.Unit()));
      return KeyEventResult.handled;
    }
    if (e.logicalKey == LogicalKeyboardKey.escape) {
      _select(null);
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = widget.projection;
    final root = p?.hasRoot() == true ? p!.root : null;
    final selected = _selected;
    final selectedNode = selected == null ? null : _find(root, selected);
    final pending = widget.composer.pendingCompose;
    final body = Focus(
      focusNode: _focus,
      onKeyEvent: _onKey,
      child: Container(
        key: const ValueKey('composer-field'),
        constraints: const BoxConstraints(minHeight: 34),
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 5),
        decoration: BoxDecoration(
          color: t.control,
          borderRadius: BorderRadius.circular(5),
          border: Border.all(color: t.hairline),
        ),
        child: Opacity(
          opacity: widget.outOfSync ? 0.5 : 1,
          child: root == null
              // an empty formula is one slot: the first action is to fill it
              ? _EmptySlot(
                  selected: selected == 'r',
                  onTap: widget.outOfSync ? null : () => _select('r'),
                  hint: p?.hasResult() == true ? 'produces ${p!.result.description}' : 'expression',
                )
              : Wrap(
                  crossAxisAlignment: WrapCrossAlignment.center,
                  spacing: 4,
                  runSpacing: 4,
                  children: _pieces(root, selected, !widget.outOfSync && !pending),
                ),
        ),
      ),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        body,
        if (widget.outOfSync)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gapTight),
            child: Row(
              spacing: MacMetrics.gap,
              children: [
                Expanded(
                  child: Text(
                    p == null
                        ? 'Waiting for the compiler to read the formula…'
                        : 'The text cannot be read as a formula; the last readable form is shown.',
                    key: const ValueKey('composer-out-of-sync'),
                    style: TextStyle(fontSize: 11, color: t.textSecondary),
                  ),
                ),
                if (widget.onEditAsText != null)
                  MacButton(label: 'Edit as text', onPressed: widget.onEditAsText),
              ],
            ),
          ),
        if (selected != null && !widget.outOfSync)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: _SlotPanel(
              key: ValueKey('slot-panel-$selected'),
              mappingId: widget.mappingId,
              nodeId: selected,
              node: selectedNode,
              slot: widget.composer.slot,
              pending: pending,
              onCompose: _compose,
              onDeselect: () => _select(null),
            ),
          ),
      ],
    );
  }

  // ---- pieces ------------------------------------------------------------

  List<Widget> _pieces(pb.FormulaNode n, String? selected, bool enabled) {
    final t = MacTokens.of(context);
    final isSelected = n.id == selected;
    final hasError = n.diagnostics.any(
      (d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
    );
    final paren = n.text.startsWith('(') && n.kind != 'call' && n.kind != 'opaque';
    List<Widget> inner;
    switch (n.kind) {
      case 'slot':
        inner = [
          _Chip(
            key: ValueKey('node-${n.id}'),
            selected: isSelected,
            error: hasError,
            dashed: true,
            onTap: enabled ? () => _select(n.id) : null,
            semantics: 'empty slot${n.hasExpected() ? ', expects ${n.expected.description}' : ''}',
            child: Text('?', style: TextStyle(fontSize: 12, color: t.textSecondary)),
          ),
        ];
      case 'reference':
        final concept = n.hasEntity() && n.entity.hasConceptId()
            ? widget.concepts[n.entity.conceptId.toInt()]
            : null;
        inner = [
          _Chip(
            key: ValueKey('node-${n.id}'),
            selected: isSelected,
            error: hasError,
            onTap: enabled ? () => _select(n.id) : null,
            semantics: '${n.name}${n.hasActual() ? ', ${n.actual.description}' : ''}',
            child: Row(
              mainAxisSize: MainAxisSize.min,
              spacing: 4,
              children: [
                if (concept != null) SocketGlyph.of(concept, t, size: 9),
                Text(n.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
              ],
            ),
          ),
        ];
      case 'number' || 'quantity':
        inner = [
          _LiteralChip(
            key: ValueKey('node-${n.id}'),
            node: n,
            selected: isSelected,
            error: hasError,
            enabled: enabled,
            units: isSelected ? widget.composer.slot?.units ?? const [] : const [],
            onSelect: () => _select(n.id),
            onCoordinate: (v) => _compose(pb.ComposeAction(nodeId: n.id, setCoordinate: v)),
            onUnit: (id) => _compose(
              pb.ComposeAction(
                nodeId: n.id,
                setUnit: pb.ComposeSetUnit(unitId: id, preserveValue: true),
              ),
            ),
          ),
        ];
      case 'bool':
        inner = [
          _Chip(
            key: ValueKey('node-${n.id}'),
            selected: isSelected,
            error: hasError,
            onTap: enabled ? () => _select(n.id) : null,
            semantics: n.name,
            child: Text(n.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
          ),
        ];
      case 'unary':
        inner = [
          _Glyph(n.name, selected: isSelected, onTap: enabled ? () => _select(n.id) : null),
          ..._pieces(n.children[0], selected, enabled),
        ];
      case 'binary' || 'compare':
        inner = [
          ..._pieces(n.children[0], selected, enabled),
          _Glyph(
            _operatorGlyph(n.name),
            key: ValueKey('node-${n.id}'),
            selected: isSelected,
            error: hasError,
            onTap: enabled ? () => _select(n.id) : null,
          ),
          ..._pieces(n.children[1], selected, enabled),
        ];
      case 'call':
        inner = [
          _Glyph(
            '${n.name}(',
            key: ValueKey('node-${n.id}'),
            selected: isSelected,
            error: hasError,
            onTap: enabled ? () => _select(n.id) : null,
          ),
          for (var i = 0; i < n.children.length; i++) ...[
            if (i > 0) _Glyph(',', selected: false),
            ..._pieces(n.children[i], selected, enabled),
          ],
          _Glyph(')', selected: isSelected, onTap: enabled ? () => _select(n.id) : null),
        ];
      default:
        inner = [
          _Chip(
            key: ValueKey('node-${n.id}'),
            selected: isSelected,
            error: hasError,
            onTap: enabled ? () => _select(n.id) : null,
            semantics: '${n.name}, edited as text',
            child: Text(
              n.text,
              style: TextStyle(fontSize: 12, fontFamily: 'Menlo', color: t.textPrimary),
            ),
          ),
        ];
    }
    if (paren) {
      return [_Glyph('(', selected: isSelected), ...inner, _Glyph(')', selected: isSelected)];
    }
    return inner;
  }
}

String _operatorGlyph(String op) => switch (op) {
  '*' => '×',
  '/' => '÷',
  '-' => '−',
  '<=' => '≤',
  '>=' => '≥',
  '!=' => '≠',
  _ => op,
};

/// One selectable component.
class _Chip extends StatelessWidget {
  const _Chip({
    super.key,
    required this.child,
    required this.selected,
    required this.semantics,
    this.error = false,
    this.dashed = false,
    this.onTap,
  });
  final Widget child;
  final bool selected;
  final bool error;
  final bool dashed;
  final String semantics;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Semantics(
      label: semantics,
      selected: selected,
      button: onTap != null,
      child: MacInteractive(
        selected: selected,
        radius: 4,
        onTap: onTap,
        child: DecoratedBox(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(4),
            border: dashed ? null : Border.all(color: t.hairline),
          ),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
            // an objection: a red underline, with the diagnostic row's words
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(color: error ? t.error : Colors.transparent, width: 2),
              ),
            ),
            child: dashed
                ? CustomPaint(painter: _DashedBorder(t.textTertiary), child: child)
                : child,
          ),
        ),
      ),
    );
  }
}

/// An operator or punctuation glyph, selectable when it names a node.
class _Glyph extends StatelessWidget {
  const _Glyph(this.text, {super.key, required this.selected, this.error = false, this.onTap});
  final String text;
  final bool selected;
  final bool error;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final label = Text(
      text,
      style: TextStyle(
        fontSize: 13,
        color: error ? t.error : t.textSecondary,
        fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
      ),
    );
    if (onTap == null) {
      return Padding(padding: const EdgeInsets.symmetric(horizontal: 2), child: label);
    }
    return MacInteractive(
      selected: selected,
      radius: 4,
      onTap: onTap,
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
      child: label,
    );
  }
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

/// A quantity literal: `[ coordinate ] [ unit ▾ ]`.  The coordinate is
/// typed and committed with Return (a new quantity, the same unit); the
/// unit pop-up switches the unit and keeps the quantity (18D).  Only a
/// literal has this pop-up (18A).
class _LiteralChip extends StatefulWidget {
  const _LiteralChip({
    super.key,
    required this.node,
    required this.selected,
    required this.error,
    required this.enabled,
    required this.units,
    required this.onSelect,
    required this.onCoordinate,
    required this.onUnit,
  });
  final pb.FormulaNode node;
  final bool selected;
  final bool error;
  final bool enabled;
  final List<pb.UnitCandidate> units;
  final VoidCallback onSelect;
  final ValueChanged<String> onCoordinate;
  final ValueChanged<String> onUnit;

  @override
  State<_LiteralChip> createState() => _LiteralChipState();
}

class _LiteralChipState extends State<_LiteralChip> {
  late final TextEditingController _c = TextEditingController(text: widget.node.coordinate);
  final FocusNode _focus = FocusNode(debugLabel: 'coordinate');

  @override
  void didUpdateWidget(_LiteralChip old) {
    super.didUpdateWidget(old);
    if (old.node.coordinate != widget.node.coordinate && _c.text != widget.node.coordinate) {
      _c.text = widget.node.coordinate;
    }
  }

  @override
  void dispose() {
    _c.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _commit() {
    final v = _c.text.trim();
    if (v.isEmpty || v == widget.node.coordinate) return;
    widget.onCoordinate(v);
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final n = widget.node;
    final unit = n.kind == 'quantity' ? n.unit : null;
    final options = [for (final u in widget.units) u.id];
    return Semantics(
      label: 'number ${n.coordinate}${unit == null ? '' : ' $unit'}',
      selected: widget.selected,
      child: Container(
        decoration: BoxDecoration(
          color: widget.selected ? t.selection : null,
          border: Border(
            bottom: BorderSide(color: widget.error ? t.error : Colors.transparent, width: 2),
          ),
        ),
        padding: const EdgeInsets.all(1),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          spacing: 2,
          children: [
            SizedBox(
              width: 12.0 * (n.coordinate.length.clamp(2, 12)) + 14,
              child: Focus(
                onFocusChange: (f) {
                  if (f) {
                    widget.onSelect();
                  } else {
                    _commit();
                  }
                },
                child: MacTextField(
                  key: ValueKey('coordinate-${n.id}'),
                  controller: _c,
                  focusNode: _focus,
                  monospace: true,
                  onSubmitted: (_) => _commit(),
                ),
              ),
            ),
            if (unit != null || widget.selected)
              // the unit: a pop-up of the units of the literal's own dimension
              // (the compiler's candidates), the current one shown
              MacDropdown<String>(
                key: ValueKey('unit-${n.id}'),
                compact: true,
                value: n.unitId.isEmpty ? null : n.unitId,
                hint: unit ?? 'unit',
                items: options.isEmpty && n.unitId.isNotEmpty ? [n.unitId] : options,
                labelOf: (id) =>
                    widget.units.where((u) => u.id == id).map((u) => u.symbol).firstOrNull ??
                    (unit ?? id),
                onChanged: widget.enabled ? widget.onUnit : (_) {},
              ),
          ],
        ),
      ),
    );
  }
}

/// The empty formula: one slot, and what it must produce.
class _EmptySlot extends StatelessWidget {
  const _EmptySlot({required this.selected, required this.onTap, required this.hint});
  final bool selected;
  final VoidCallback? onTap;
  final String hint;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Row(
      spacing: MacMetrics.gap,
      children: [
        _Chip(
          key: const ValueKey('node-r'),
          selected: selected,
          dashed: true,
          onTap: onTap,
          semantics: 'empty slot, $hint',
          child: Text('?', style: TextStyle(fontSize: 12, color: t.textSecondary)),
        ),
        Expanded(
          child: Text(hint, style: TextStyle(fontSize: 11, color: t.textTertiary)),
        ),
      ],
    );
  }
}

/// What the selected position expects and what fits: the compiler's
/// answer, with the actions on the selected component.
class _SlotPanel extends StatefulWidget {
  const _SlotPanel({
    super.key,
    required this.mappingId,
    required this.nodeId,
    required this.node,
    required this.slot,
    required this.pending,
    required this.onCompose,
    required this.onDeselect,
  });
  final int mappingId;
  final String nodeId;
  final pb.FormulaNode? node;
  final pb.FormulaSlotResponse? slot;
  final bool pending;
  final void Function(pb.ComposeAction) onCompose;
  final VoidCallback onDeselect;

  @override
  State<_SlotPanel> createState() => _SlotPanelState();
}

class _SlotPanelState extends State<_SlotPanel> {
  final TextEditingController _number = TextEditingController();
  final FocusNode _numberFocus = FocusNode(debugLabel: 'slot-number');
  String? _unit;
  bool _explain = false;

  @override
  void dispose() {
    _number.dispose();
    _numberFocus.dispose();
    super.dispose();
  }

  bool get _isSlot => widget.node == null || widget.node!.kind == 'slot';

  void _fill(String text) => widget.onCompose(pb.ComposeAction(nodeId: widget.nodeId, fill: text));

  void _insertNumber() {
    final v = _number.text.trim();
    if (v.isEmpty || double.tryParse(v) == null) return;
    final units = widget.slot?.units ?? const <pb.UnitCandidate>[];
    final unit = _unit ?? (units.isEmpty ? null : units.first.symbol);
    _fill(unit == null ? v : '$v $unit');
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final s = widget.slot;
    final n = widget.node;
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final units = s?.units ?? const <pb.UnitCandidate>[];
    final unitSymbol = _unit ?? (units.isEmpty ? null : units.first.symbol);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // 1. the expectation, in the designer's words
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: MacMetrics.gap,
          children: [
            Expanded(
              child: Text(
                s == null ? 'Asking what fits here…' : s.explanation,
                key: const ValueKey('slot-explanation'),
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
            ),
            if (s != null && s.technical.isNotEmpty)
              MacLink(
                label: _explain ? 'Hide detail' : 'Explain',
                onTap: () => setState(() => _explain = !_explain),
              ),
          ],
        ),
        if (_explain && s != null)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gapTight),
            child: Text(
              s.technical,
              key: const ValueKey('slot-technical'),
              style: TextStyle(fontSize: 11, fontFamily: 'Menlo', color: t.textSecondary),
            ),
          ),
        if (n != null && n.hasActual() && !_isSlot)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gapTight),
            child: Text('This is ${n.actual.description}.', style: small),
          ),
        const SizedBox(height: MacMetrics.gap),
        // 2. what to put here: a number (with its unit, 18C), a reference,
        //    an equation — or, on a component, an operator around it
        if (_isSlot) ...[
          // a number: the coordinate, its unit (the compiler's list for the
          // slot's dimension), Insert — the literal's construction (18B.1)
          Row(
            spacing: MacMetrics.gapTight,
            children: [
              Expanded(
                child: MacTextField(
                  key: const ValueKey('slot-number'),
                  controller: _number,
                  focusNode: _numberFocus,
                  autofocus: true,
                  monospace: true,
                  hint: 'number',
                  onSubmitted: (_) => _insertNumber(),
                ),
              ),
              if (units.isNotEmpty)
                MacDropdown<String>(
                  key: const ValueKey('slot-unit'),
                  compact: true,
                  value: unitSymbol,
                  hint: unitSymbol,
                  items: [for (final u in units) u.symbol],
                  onChanged: (v) => setState(() => _unit = v),
                ),
              MacButton(label: 'Insert', onPressed: widget.pending ? null : _insertNumber),
            ],
          ),
          if (s != null && s.insufficient)
            Padding(
              padding: const EdgeInsets.only(top: MacMetrics.gapTight),
              child: Text('No unit is suggested: fill the other side first.', style: small),
            ),
          if (s != null && s.references.isNotEmpty) ...[
            const SizedBox(height: MacMetrics.gap),
            Text('References', style: small),
            for (final r in s.references)
              _CandidateRow(
                key: ValueKey('ref-${r.label}'),
                label: r.label,
                detail: r.produces,
                onTap: widget.pending ? null : () => _fill(r.insert),
              ),
          ],
          if (s != null && s.equations.isNotEmpty) ...[
            const SizedBox(height: MacMetrics.gap),
            // the library is long: folded until asked for
            MacDisclosure(
              key: const ValueKey('slot-equations'),
              title: 'Equations (${s.equations.length})',
              children: [
                for (final e in s.equations)
                  _CandidateRow(
                    key: ValueKey('eq-${e.name}'),
                    label: e.shape,
                    detail: e.summary,
                    onTap: widget.pending ? null : () => _fill(e.insert),
                  ),
              ],
            ),
          ],
        ] else ...[
          Wrap(
            spacing: MacMetrics.gapTight,
            runSpacing: MacMetrics.gapTight,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: [
              for (final (op, glyph) in const [('+', '+'), ('-', '−'), ('*', '×'), ('/', '÷')])
                MacButton(
                  key: ValueKey('op-$op'),
                  label: glyph,
                  tooltip: 'Insert $glyph after this',
                  onPressed: widget.pending
                      ? null
                      : () => widget.onCompose(
                          pb.ComposeAction(
                            nodeId: widget.nodeId,
                            operator: pb.ComposeOperator(op: op, before: false),
                          ),
                        ),
                ),
              MacDropdown<String>(
                key: const ValueKey('op-compare'),
                compact: true,
                value: null,
                hint: 'Compare',
                items: const ['<', '<=', '>', '>=', '==', '!='],
                labelOf: _operatorGlyph,
                onChanged: (op) => widget.onCompose(
                  pb.ComposeAction(
                    nodeId: widget.nodeId,
                    operator: pb.ComposeOperator(op: op, before: false),
                  ),
                ),
              ),
              if (s != null && s.equations.isNotEmpty)
                MacDropdown<pb.EquationCandidate>(
                  key: const ValueKey('op-function'),
                  compact: true,
                  value: null,
                  hint: 'Function',
                  items: s.equations,
                  labelOf: (e) => e.shape,
                  onChanged: (e) => widget.onCompose(
                    pb.ComposeAction(
                      nodeId: widget.nodeId,
                      call: pb.ComposeCall(name: e.name, arity: '?'.allMatches(e.insert).length),
                    ),
                  ),
                ),
              MacButton(
                key: const ValueKey('op-remove'),
                label: 'Remove',
                onPressed: widget.pending
                    ? null
                    : () => widget.onCompose(
                        pb.ComposeAction(nodeId: widget.nodeId, remove: pb.Unit()),
                      ),
              ),
            ],
          ),
          if (n != null && n.kind == 'opaque')
            Padding(
              padding: const EdgeInsets.only(top: MacMetrics.gapTight),
              child: Text('This part is edited as text.', style: small),
            ),
        ],
        // the objections on this component are the red underline on it
        // and the diagnostic rows under the field: one finding, two places
      ],
    );
  }
}

class _CandidateRow extends StatelessWidget {
  const _CandidateRow({super.key, required this.label, required this.detail, this.onTap});
  final String label;
  final String detail;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return MacInteractive(
      onTap: onTap,
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
      child: Row(
        spacing: MacMetrics.gap,
        children: [
          Text(label, style: TextStyle(fontSize: 12, color: t.textPrimary)),
          Expanded(
            child: Text(
              detail,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(fontSize: 11, color: t.textSecondary),
            ),
          ),
        ],
      ),
    );
  }
}

/// The kind of value a projection node is, as the canvas draws it.
SocketKind socketKindOfType(pb.TypeView t) => switch (t.kind) {
  'quantity' => SocketKind.quantity,
  'boolean' => SocketKind.onOff,
  'count' => SocketKind.count,
  _ => SocketKind.open,
};
