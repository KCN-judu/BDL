/// The Formula Composer: the structured projection of a definition draft,
/// rendered as the expression it is (`formula_render.dart`) and edited
/// two ways at once over the one draft — a **structural caret** the
/// keyboard types at, and a **selected part** the palette acts on —
/// both the compiler's reading of the same text (ADR-0028; the model is
/// `app/caret.dart`, the rationale docs/architecture/studio-ui.md §4b
/// *Typed structure*).
///
/// Studio draws what the compiler projected (`FormulaProjection`), asks
/// it for every candidate (`GetFormulaSlot`, `CompleteDefinitionDraft`)
/// and sends it every structured action (`ComposeFormula`); a typed
/// character is a text edit of the draft at the caret's byte offset,
/// which the compiler reads like any typing in the Text view.  Nothing in
/// this file parses, types, converts a unit or decides what fits.
///
/// Keys (the primary path; the palette is the pointer's):
///
///   ← →            the previous / next caret stop — out of a
///                  denominator, past a parenthesis, into the next part;
///   ↑ ↓            the nearest stop on the row above / below (a
///                  numerator from its denominator, a branch from the
///                  next);
///   Home / End     the first / last stop of the enclosing part; again,
///                  of the whole formula;
///   Tab / ⇧Tab     the next / previous empty slot;
///   letters digits type into a slot or extend the name or number the
///                  caret touches; a space after a number starts its unit;
///   + − * / < > = & | !   the operator on the part the caret touches,
///                  with a slot for the other side; `!` negates;
///   (              applies the name before the caret (`clamp` →
///                  `clamp(?, ?, ?)`), or groups a slot;
///   )  ,           leave the enclosing parentheses / move to the next
///                  argument;
///   ⌫ ⌦            a character, or a whole part (a slot's operator goes
///                  with it);
///   ⌃Space         completion at the caret; ↑ ↓ ⏎ Tab in the list;
///   Esc            close completion, then clear the caret and selection.
///
/// While the text differs from what the compiler last read — a character
/// just typed — the part being typed into shows as text in place
/// (`PendingText`) and the rest keeps its structure; when the compiler's
/// reading arrives the picture follows.  A structured action is refused
/// until then (the stale-projection policy, `app/composer.dart`).
library;

import 'dart:convert' show utf8;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/caret.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/canvas_geometry.dart';
import 'code/completion_popup.dart';
import 'formula_render.dart';
import 'mac/controls.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'source_span.dart';

class FormulaComposer extends StatefulWidget {
  const FormulaComposer({
    super.key,
    required this.mappingId,
    required this.source,
    required this.projection,
    required this.composer,
    required this.concepts,
    required this.dispatch,
    this.completion,
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

  /// The completion pop-up's state when it is this mapping's.
  final CompletionState? completion;

  /// The text on screen is not what the projection is of (it did not
  /// parse, or the verdict has not arrived).
  final bool outOfSync;
  final VoidCallback? onEditAsText;

  @override
  State<FormulaComposer> createState() => _FormulaComposerState();
}

class _FormulaComposerState extends State<FormulaComposer> {
  final FocusNode _focus = FocusNode(debugLabel: 'composer');
  final FormulaGeometry _geometry = FormulaGeometry();
  final GlobalKey _fieldKey = GlobalKey(debugLabel: 'composer-field');

  /// The last projection of this mapping that parsed: what the picture
  /// is drawn from while the compiler reads a newer text.
  pb.FormulaProjection? _lastGood;

  /// Where the caret is drawn, in the field's coordinates; computed
  /// after layout from the geometry.
  Rect? _caretRect;

  /// A transient word under the field: why a key did nothing.
  String? _hint;

  /// The text the composer's own last edit produced (a typed character, a
  /// structured action's answer): while the compiler reads exactly this
  /// text the picture stays live with the edited part as text.  Text that
  /// changed any other way (the Text view, a reload) waits for its
  /// reading as before.
  String? _typedSource;

  @override
  void initState() {
    super.initState();
    _remember(widget.projection);
  }

  @override
  void didUpdateWidget(FormulaComposer old) {
    super.didUpdateWidget(old);
    if (old.mappingId != widget.mappingId) {
      _lastGood = null;
      _typedSource = null;
    }
    // a structured action's answer landed: the composer's own edit
    if (old.composer.pendingCompose &&
        !widget.composer.pendingCompose &&
        old.source != widget.source) {
      _typedSource = widget.source;
    }
    _remember(widget.projection);
    if (old.composer.caret != widget.composer.caret || old.source != widget.source) {
      _hint = null;
    }
  }

  void _remember(pb.FormulaProjection? p) {
    if (p != null && p.parseOk && p.hasRoot() && p.source == widget.source) _lastGood = p;
  }

  @override
  void dispose() {
    _focus.dispose();
    super.dispose();
  }

  // ---- what is on screen ---------------------------------------------------

  bool get _inSync {
    final p = widget.projection;
    if (widget.source.trim().isEmpty) return true;
    return p != null && p.source == widget.source && p.parseOk;
  }

  /// The projection the picture is drawn from, and the part shown as the
  /// text being typed into it when the text has moved on from it.
  ///
  /// Three states, one policy (app/composer.dart `composerInSync`): the
  /// compiler read this text (in sync: the picture is current); it has not
  /// read it yet (awaiting: the last picture with the edited part as text,
  /// live — keys keep working on it); it read it and could not (stale:
  /// the last picture dimmed, the notice, *Edit as text*).
  ({pb.FormulaProjection? projection, PendingText? pending, bool stale}) get _picture {
    final p = widget.projection;
    if (_inSync) return (projection: p, pending: null, stale: false);
    final refused = p != null && p.source == widget.source && !p.parseOk;
    final good = _lastGood;
    // refused, or changed by another hand: no tree is invented — the
    // notice says which (the field shows the text)
    if (refused || widget.source != _typedSource) {
      return (projection: null, pending: null, stale: true);
    }
    // the composer's own first characters into an empty formula: text,
    // live, until read
    if (good == null || !good.hasRoot()) return (projection: null, pending: null, stale: false);
    final region = changedRegion(good.source, widget.source);
    if (region == null) return (projection: good, pending: null, stale: false);
    final node = nodeContaining(good.root, region.start, region.end);
    if (node == null) return (projection: null, pending: null, stale: true);
    final delta = region.newLength - (region.end - region.start);
    final text = excerptOf(widget.source, node.range.start, node.range.end + delta);
    return (projection: good, pending: PendingText(nodeId: node.id, text: text), stale: false);
  }

  List<CaretStop> get _stops => caretStops(_picture.projection);

  CaretStop? get _stop => resolveStop(_stops, widget.composer.caret);

  String? get _selected => widget.composer.selectedNode;

  // ---- moves ---------------------------------------------------------------

  void _select(String? id) =>
      widget.dispatch(FormulaNodeSelected(mappingId: widget.mappingId, nodeId: id));

  /// The caret lands at a stop; the selection follows it to the part the
  /// stop belongs to, so the palette is about the same part the keys are.
  void _moveTo(CaretStop stop) {
    widget.dispatch(FormulaCaretMoved(mappingId: widget.mappingId, caret: stateOf(stop)));
    if (_selected != stop.node) _select(stop.node);
    _focus.requestFocus();
  }

  void _clear() {
    widget.dispatch(FormulaCaretMoved(mappingId: widget.mappingId, caret: null));
    _select(null);
  }

  void _compose(pb.ComposeAction a) =>
      widget.dispatch(ComposeRequested(mappingId: widget.mappingId, action: a));

  /// A text edit of the draft at byte offsets: the same path the Text
  /// view's typing takes.  The caret lands after the inserted text; a
  /// word character asks for completion there.
  void _applyEdit(TextEditPlan e, {bool complete = false}) {
    final source = widget.source;
    final r = codeUnitRange(source, e.start, e.end);
    final next = source.replaceRange(r.start, r.end, e.text);
    final caretBytes = e.start + utf8.encode(e.text).length;
    _typedSource = next;
    widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: next));
    widget.dispatch(FormulaCaretMoved(mappingId: widget.mappingId, caret: CaretState(caretBytes)));
    if (complete) {
      widget.dispatch(
        CompletionRequested(mappingId: widget.mappingId, source: next, offset: caretBytes),
      );
    } else if (widget.completion != null) {
      widget.dispatch(const CompletionDismissed());
    }
  }

  void _run(KeyPlan plan, {bool complete = false}) {
    switch (plan) {
      case EditText(:final edit):
        _applyEdit(edit, complete: complete);
      case Structural(:final action):
        if (widget.completion != null) widget.dispatch(const CompletionDismissed());
        _compose(action);
      case MoveTo(:final stop):
        _moveTo(stop);
      case Refused(:final reason):
        if (reason == Refused.needsOperator) {
          setState(() => _hint = context.l10n.typeAnOperatorFirst);
        }
    }
  }

  /// Accept the completion's selected candidate: replace the service's
  /// byte range with its insert text; the caret lands after it, or in the
  /// first slot it wrote.  False when nothing changed (the candidate was
  /// already written).
  bool _acceptCompletion() {
    final c = widget.completion;
    if (c == null || c.items.isEmpty) return false;
    final item = c.items[c.selected.clamp(0, c.items.length - 1)];
    final source = widget.source;
    if (c.source != source) return false;
    final r = codeUnitRange(source, item.replaceStart, item.replaceEnd);
    final next = source.replaceRange(r.start, r.end, item.insert);
    if (next == source) {
      widget.dispatch(const CompletionDismissed());
      return false;
    }
    final slot = item.insert.indexOf('?');
    final caretBytes =
        item.replaceStart +
        (slot >= 0
            ? utf8.encode(item.insert.substring(0, slot)).length
            : utf8.encode(item.insert).length);
    _typedSource = next;
    widget.dispatch(const CompletionDismissed());
    widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: next));
    widget.dispatch(FormulaCaretMoved(mappingId: widget.mappingId, caret: CaretState(caretBytes)));
    return true;
  }

  void _requestCompletion() {
    final stop = _stop;
    final offset = stop?.offset ?? widget.composer.caret?.offset;
    if (offset == null) return;
    widget.dispatch(
      CompletionRequested(mappingId: widget.mappingId, source: widget.source, offset: offset),
    );
  }

  // ---- keys ----------------------------------------------------------------

  KeyEventResult _onKey(FocusNode node, KeyEvent e) {
    if (e is! KeyDownEvent && e is! KeyRepeatEvent) return KeyEventResult.ignored;
    // a text entry inside the field (a coordinate) owns its own keys
    if (FocusManager.instance.primaryFocus?.context?.widget is EditableText) {
      return KeyEventResult.ignored;
    }
    final k = e.logicalKey;
    final completion = widget.completion;
    if (completion != null) {
      // the pop-up takes the navigation keys while it is open
      if (k == LogicalKeyboardKey.arrowDown) {
        widget.dispatch(const CompletionMoved(1));
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.arrowUp) {
        widget.dispatch(const CompletionMoved(-1));
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.tab) {
        final changed = _acceptCompletion();
        // Tab on a candidate already written moves on to the next slot
        if (k == LogicalKeyboardKey.tab && !changed) {
          final stops = caretStops(_picture.projection);
          final next = nextSlot(
            stops,
            resolveStop(stops, widget.composer.caret),
            backwards: HardwareKeyboard.instance.isShiftPressed,
          );
          if (next != null) _moveTo(next);
        }
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.escape) {
        widget.dispatch(const CompletionDismissed());
        return KeyEventResult.handled;
      }
    }
    if (k == LogicalKeyboardKey.space && HardwareKeyboard.instance.isControlPressed) {
      _requestCompletion();
      return KeyEventResult.handled;
    }
    final picture = _picture;
    final p = picture.projection;
    final stops = caretStops(p);
    final stop = resolveStop(stops, widget.composer.caret);
    final selected = _selected;
    final pending = widget.composer.pendingCompose;

    if (k == LogicalKeyboardKey.escape) {
      // the innermost thing first: a caret or selection; then the editor's
      // own Esc (revert) sees it
      if (stop == null && selected == null) return KeyEventResult.ignored;
      _clear();
      return KeyEventResult.handled;
    }
    // no tree yet: an empty formula (one slot of Studio's: typing writes
    // it), or the composer's own first characters the compiler has not
    // read — plain text at the caret until it has
    if (p == null || !p.hasRoot()) {
      if (picture.stale || e.character == null) return KeyEventResult.ignored;
      final c = e.character!;
      final bytes = utf8.encode(widget.source).length;
      if (widget.source.trim().isEmpty) {
        if (RegExp(r'^[A-Za-z0-9_(]$').hasMatch(c)) {
          _applyEdit(
            TextEditPlan(0, bytes, c == '(' ? '(?)' : c),
            complete: RegExp(r'^[A-Za-z_]$').hasMatch(c),
          );
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      }
      final at = (widget.composer.caret?.offset ?? bytes).clamp(0, bytes);
      if (k == LogicalKeyboardKey.backspace) {
        if (at > 0) _applyEdit(TextEditPlan(at - 1, at, ''));
        return KeyEventResult.handled;
      }
      if (RegExp(r'^[A-Za-z0-9_. ]$').hasMatch(c)) {
        _applyEdit(TextEditPlan(at, at, c), complete: RegExp(r'^[A-Za-z_]$').hasMatch(c));
        return KeyEventResult.handled;
      }
      final op = operatorFor(c);
      if (op != null && at == bytes) {
        // an operator at the end of unread text: the text it would make,
        // the caret in the slot it opens
        final text = op == '!' ? '' : ' $op ?';
        if (text.isEmpty) return KeyEventResult.handled;
        _applyEdit(TextEditPlan(at, at, text));
        widget.dispatch(
          FormulaCaretMoved(
            mappingId: widget.mappingId,
            caret: CaretState(at + utf8.encode(text).length - 1),
          ),
        );
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    }
    // Tab: the next slot, from the caret or the selection
    if (k == LogicalKeyboardKey.tab) {
      final from = stop ?? stops.where((s) => s.node == selected).firstOrNull;
      final next = nextSlot(stops, from, backwards: HardwareKeyboard.instance.isShiftPressed);
      if (next != null) _moveTo(next);
      return KeyEventResult.handled;
    }
    if (stop == null) {
      // keys on a selected part, without a caret: the operators and ⌫
      // as the palette does them
      if (selected == null || pending || picture.stale) return KeyEventResult.ignored;
      final op = e.character == null ? null : operatorFor(e.character!);
      if (op != null) {
        _compose(
          pb.ComposeAction(
            nodeId: selected,
            operator: pb.ComposeOperator(op: op, before: false),
          ),
        );
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.backspace || k == LogicalKeyboardKey.delete) {
        _compose(pb.ComposeAction(nodeId: selected, remove: pb.Unit()));
        return KeyEventResult.handled;
      }
      if (k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowLeft) {
        final own = stops.where((s) => s.node == selected).toList();
        if (own.isNotEmpty) _moveTo(k == LogicalKeyboardKey.arrowRight ? own.last : own.first);
        return KeyEventResult.handled;
      }
      return KeyEventResult.ignored;
    }
    // moves: always allowed, also while the compiler reads
    if (k == LogicalKeyboardKey.arrowRight) {
      final n = nextStop(stops, stop);
      if (n != null) _moveTo(n);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowLeft) {
      final n = previousStop(stops, stop);
      if (n != null) _moveTo(n);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.arrowUp || k == LogicalKeyboardKey.arrowDown) {
      final n = _verticalNeighbour(stops, stop, up: k == LogicalKeyboardKey.arrowUp);
      if (n != null) _moveTo(n);
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.home) {
      _moveTo(homeStop(stops, stop));
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.end) {
      _moveTo(endStop(stops, stop));
      return KeyEventResult.handled;
    }
    if (e.character == ')') {
      _moveTo(exitGroup(stops, p.root, stop));
      return KeyEventResult.handled;
    }
    if (e.character == ',') {
      final n = nextArgument(stops, p.root, stop);
      if (n != null) _moveTo(n);
      return KeyEventResult.handled;
    }
    // edits: only over a picture the compiler has read
    if (picture.stale || pending) return KeyEventResult.ignored;
    final at = widget.composer.caret?.offset;
    if (k == LogicalKeyboardKey.backspace) {
      _run(backspaceAt(p, stops, stop, at: at));
      return KeyEventResult.handled;
    }
    if (k == LogicalKeyboardKey.delete) {
      _run(deleteAt(p, stops, stop));
      return KeyEventResult.handled;
    }
    final c = e.character;
    if (c != null && c.isNotEmpty && !HardwareKeyboard.instance.isMetaPressed) {
      if (picture.pending != null && operatorFor(c) != null) {
        // an operator while the compiler reads the part being typed: wait
        return KeyEventResult.handled;
      }
      final plan = characterAt(p, stop, c, at: at);
      _run(plan, complete: RegExp(r'^[A-Za-z_]$').hasMatch(c) || c == ' ');
      return KeyEventResult.handled;
    }
    return KeyEventResult.ignored;
  }

  // ---- geometry ------------------------------------------------------------

  RenderBox? get _fieldBox => _fieldKey.currentContext?.findRenderObject() as RenderBox?;

  /// Where a stop stands in the field: a vertical span at an x.
  Rect? _stopRect(CaretStop s) {
    final box = _fieldBox;
    if (box == null) return null;
    Rect? r(String id) => _geometry.rectOf(id, box);
    switch (s.side) {
      case StopSide.before:
        final n = r(s.node);
        return n == null ? null : Rect.fromLTWH(n.left, n.top, 0, n.height);
      case StopSide.after:
        final n = r(s.node);
        return n == null ? null : Rect.fromLTWH(n.right, n.top, 0, n.height);
      case StopSide.inSlot:
        final n = r('${s.node}/text') ?? r(s.node);
        return n == null ? null : Rect.fromLTWH(n.center.dx, n.top, 0, n.height);
      case StopSide.open:
        final n = r('${s.node}/inner') ?? r(s.node);
        return n == null ? null : Rect.fromLTWH(n.left, n.top, 0, n.height);
      case StopSide.close:
        final n = r('${s.node}/inner') ?? r(s.node);
        return n == null ? null : Rect.fromLTWH(n.right, n.top, 0, n.height);
      case StopSide.inside:
        final n = r('${s.node}/text');
        final node = findNode(_picture.projection?.root, s.node);
        if (n == null || node == null) return null;
        final k = int.tryParse(s.id.split('@').last) ?? 0;
        final paren = wrappedInParens(node);
        final text = paren ? node.text.substring(1, node.text.length - 1) : node.text;
        final w = _textWidth(text.substring(0, k.clamp(0, text.length)), s.node);
        return Rect.fromLTWH(n.left + w, n.top, 0, n.height);
    }
  }

  double _textWidth(String text, String nodeId) {
    final style = _geometry.textStyles[nodeId] ?? TextStyle(fontSize: MacType.body);
    final painter = TextPainter(
      text: TextSpan(text: text, style: style),
      textDirection: TextDirection.ltr,
    )..layout();
    return painter.width;
  }

  /// The stop nearest a point in the field: the same row first.
  CaretStop? _stopNear(Offset local, List<CaretStop> stops) {
    CaretStop? best;
    var bestScore = double.infinity;
    for (final s in stops) {
      final r = _stopRect(s);
      if (r == null) continue;
      final dy = local.dy < r.top
          ? r.top - local.dy
          : local.dy > r.bottom
          ? local.dy - r.bottom
          : 0.0;
      final score = (local.dx - r.left).abs() + dy * 4;
      if (score < bestScore) {
        bestScore = score;
        best = s;
      }
    }
    return best;
  }

  CaretStop? _verticalNeighbour(List<CaretStop> stops, CaretStop from, {required bool up}) {
    final here = _stopRect(from);
    if (here == null) return null;
    CaretStop? best;
    var bestScore = double.infinity;
    for (final s in stops) {
      if (s.id == from.id) continue;
      final r = _stopRect(s);
      if (r == null) continue;
      final above = r.bottom <= here.top + 1;
      final below = r.top >= here.bottom - 1;
      if (up ? !above : !below) continue;
      final dy = up ? here.top - r.bottom : r.top - here.bottom;
      final score = dy * 2 + (r.left - here.left).abs();
      if (score < bestScore) {
        bestScore = score;
        best = s;
      }
    }
    return best;
  }

  /// A tap on a part: the caret goes where the tap was — between two
  /// characters of a name or number, before or after any other part, in
  /// a slot — and the part is selected.
  void _tapNode(pb.FormulaNode n, Offset global) {
    final box = _fieldBox;
    if (box == null) return;
    final local = box.globalToLocal(global);
    final stops = _stops;
    final own = stops.where((s) => s.node == n.id).toList();
    if (own.isEmpty) {
      _select(n.id);
      _focus.requestFocus();
      return;
    }
    final near = _stopNear(local, own);
    if (near != null) _moveTo(near);
  }

  void _tapField(TapUpDetails d) {
    final box = _fieldBox;
    if (box == null) return;
    final near = _stopNear(box.globalToLocal(d.globalPosition), _stops);
    if (near != null) {
      _moveTo(near);
    } else {
      _focus.requestFocus();
    }
  }

  final GlobalKey _rawKey = GlobalKey(debugLabel: 'composer-raw');

  void _placeCaret() {
    final stop = _stop;
    var rect = stop == null ? null : _stopRect(stop);
    // raw text (not read yet): the caret between its characters
    final caret = widget.composer.caret;
    final box = _fieldBox;
    final raw = _rawKey.currentContext?.findRenderObject() as RenderBox?;
    if (rect == null && caret != null && box != null && raw != null && raw.hasSize) {
      final origin = raw.localToGlobal(Offset.zero, ancestor: box);
      final r = codeUnitRange(widget.source, 0, caret.offset);
      final painter = TextPainter(
        text: TextSpan(
          text: widget.source.substring(0, r.end),
          style: const TextStyle(fontSize: MacType.code, fontFamily: 'Menlo'),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      rect = Rect.fromLTWH(origin.dx + painter.width, origin.dy, 0, raw.size.height);
    }
    if (rect != _caretRect && mounted) setState(() => _caretRect = rect);
  }

  // ---- build ---------------------------------------------------------------

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final picture = _picture;
    final p = picture.projection;
    final root = p != null && p.hasRoot() ? p.root : null;
    final selected = _selected;
    final selectedNode = selected == null ? null : findNode(root, selected);
    final pending = widget.composer.pendingCompose;
    final stale = picture.stale && widget.source.trim().isNotEmpty;
    final enabled = !stale && !pending;
    final stop = _stop;
    WidgetsBinding.instance.addPostFrameCallback((_) => _placeCaret());

    final Widget content;
    if (root == null) {
      content = stale
          // nothing readable to show: the notice below says why
          ? Text(
              widget.source,
              style: TextStyle(fontSize: MacType.code, fontFamily: 'Menlo', color: t.textSecondary),
            )
          : widget.source.trim().isEmpty
          ? _EmptySlot(
              selected: selected == 'r' || widget.composer.caret != null,
              onTap: () {
                widget.dispatch(
                  FormulaCaretMoved(mappingId: widget.mappingId, caret: const CaretState(0)),
                );
                _select('r');
                _focus.requestFocus();
              },
              hint: widget.projection?.hasResult() == true
                  ? l10n.producesDescription(widget.projection!.result.description)
                  : l10n.typeToWrite,
            )
          // text just typed into an empty formula: shown as text until read
          : Text(
              widget.source,
              key: _rawKey,
              style: TextStyle(fontSize: MacType.code, fontFamily: 'Menlo', color: t.textPrimary),
            );
    } else {
      content = FormulaRender(
        projection: p!,
        concepts: widget.concepts,
        geometry: _geometry,
        selected: selected,
        pending: picture.pending,
        onTapNode: enabled ? _tapNode : null,
        units: widget.composer.slot?.units ?? const [],
        onUnit: enabled
            ? (id, unitId) => _compose(
                pb.ComposeAction(
                  nodeId: id,
                  setUnit: pb.ComposeSetUnit(unitId: unitId, preserveValue: true),
                ),
              )
            : null,
      );
    }

    final caret = _caretRect;
    final field = Focus(
      focusNode: _focus,
      onKeyEvent: _onKey,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapUp: enabled ? _tapField : null,
        child: MouseRegion(
          cursor: SystemMouseCursors.text,
          child: Container(
            key: _fieldKey,
            constraints: const BoxConstraints(minHeight: 34),
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 5),
            decoration: BoxDecoration(
              color: t.control,
              borderRadius: BorderRadius.circular(5),
              border: Border.all(color: _focus.hasFocus ? t.accent : t.hairline),
            ),
            child: Stack(
              children: [
                Opacity(opacity: stale ? 0.5 : 1, child: content),
                if (caret != null && _focus.hasFocus)
                  Positioned(
                    left: caret.left - 6 - 1,
                    top: caret.top - 5,
                    child: Semantics(
                      label: stop == null ? null : _caretReading(stop, root, l10n),
                      liveRegion: true,
                      child: Container(
                        key: const ValueKey('composer-caret'),
                        width: 2,
                        height: caret.height,
                        color: t.accent,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );

    final completion = widget.completion;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Semantics(
          key: const ValueKey('composer-field'),
          textField: true,
          label: l10n.formula,
          child: field,
        ),
        if (completion != null && (completion.items.isNotEmpty || completion.pending))
          CompletionPopup(
            completion: completion,
            onPick: (i) {
              widget.dispatch(CompletionMoved(i - completion.selected));
              _acceptCompletion();
            },
          ),
        if (_hint case final h?)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gapTight),
            child: Text(
              h,
              key: const ValueKey('composer-hint'),
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
            ),
          ),
        if (stale)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gapTight),
            child: Row(
              spacing: MacMetrics.gap,
              children: [
                Expanded(
                  child: Text(
                    // three stale states, one policy: no answer yet, an
                    // answer for older text, text the compiler cannot read
                    widget.projection == null || widget.projection!.source != widget.source
                        ? l10n.waitingForTheCompilerToReadThe
                        : l10n.theTextCannotBeReadAsA,
                    key: const ValueKey('composer-out-of-sync'),
                    style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                  ),
                ),
                if (widget.onEditAsText != null)
                  MacButton(label: l10n.editAsText, onPressed: widget.onEditAsText),
              ],
            ),
          ),
        if (selected != null && !stale)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: _SlotPanel(
              key: ValueKey('slot-panel-$selected'),
              mappingId: widget.mappingId,
              nodeId: selected,
              node: selectedNode,
              slot: widget.composer.slot,
              pending: pending,
              truthValued: _truthValued(selectedNode),
              onCompose: _compose,
              onDeselect: _clear,
            ),
          ),
      ],
    );
  }

  /// What assistive technology hears where the caret stands.
  String _caretReading(CaretStop s, pb.FormulaNode? root, AppLocalizations l10n) {
    final n = findNode(root, s.node);
    final what = n == null ? s.node : describeNode(n, l10n);
    return switch (s.side) {
      StopSide.before => l10n.caretBefore(what),
      StopSide.after => l10n.caretAfter(what),
      StopSide.inSlot => n == null ? l10n.caretIn : describeNode(n, l10n),
      StopSide.open || StopSide.close || StopSide.inside => l10n.caretInside(what),
    };
  }

  /// Whether a component is a truth value, by what the compiler found it
  /// to be: true or false, or a concept represented by one; `null` while
  /// nothing is known.  Presentation only — it decides which actions are
  /// offered, never whether they are right.
  bool? _truthValued(pb.FormulaNode? n) {
    if (n == null || !n.hasActual()) return null;
    final a = n.actual;
    switch (a.kind) {
      case 'boolean':
        return true;
      case 'unknown':
        return null;
      case 'concept':
        final c = a.hasConceptId() ? widget.concepts[a.conceptId.toInt()] : null;
        if (c == null || !c.hasRepresentation()) return null;
        return c.representation.hasBoolean();
      default:
        return false;
    }
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
        Semantics(
          label: 'empty slot, $hint',
          selected: selected,
          button: onTap != null,
          child: MacInteractive(
            selected: selected,
            radius: 4,
            onTap: onTap,
            child: Container(
              key: const ValueKey('node-r'),
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 3),
              child: CustomPaint(
                painter: _DashedBorder(t.textTertiary),
                child: Text(
                  '?',
                  style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                ),
              ),
            ),
          ),
        ),
        Expanded(
          child: Text(
            hint,
            style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
          ),
        ),
      ],
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
    required this.truthValued,
    required this.onCompose,
    required this.onDeselect,
  });
  final int mappingId;
  final String nodeId;
  final pb.FormulaNode? node;
  final pb.FormulaSlotResponse? slot;
  final bool pending;

  /// Whether the selected component is a truth value (`null`: not known):
  /// the logical operators are offered unless it is not one, a range
  /// unless it is one.
  final bool? truthValued;
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
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
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
                s == null ? context.l10n.askingWhatFitsHere : s.explanation,
                key: const ValueKey('slot-explanation'),
                style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
              ),
            ),
            if (s != null && s.technical.isNotEmpty)
              MacLink(
                label: _explain ? context.l10n.hideDetail : context.l10n.explain,
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
              style: TextStyle(
                fontSize: MacType.secondary,
                fontFamily: 'Menlo',
                color: t.textSecondary,
              ),
            ),
          ),
        if (n != null && n.hasActual() && !_isSlot)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gapTight),
            child: Text(context.l10n.thisIs(n.actual.description), style: small),
          ),
        const SizedBox(height: MacMetrics.gap),
        // 2. what to put here: a number (with its unit, 18C), a reference,
        //    an equation — or, on a component, an operator around it
        if (_isSlot) ...[
          if (s != null && s.booleans.isNotEmpty)
            // a truth value: the two literals, as written
            Row(
              spacing: MacMetrics.gapTight,
              children: [
                for (final b in s.booleans)
                  MacButton(
                    key: ValueKey('bool-$b'),
                    label: b,
                    onPressed: widget.pending ? null : () => _fill(b),
                  ),
              ],
            )
          else
            // a number: the coordinate, its unit (the compiler's list for
            // the slot's dimension), Insert — the literal's construction
            // (18B.1)
            Row(
              spacing: MacMetrics.gapTight,
              children: [
                Expanded(
                  child: MacTextField(
                    key: const ValueKey('slot-number'),
                    controller: _number,
                    focusNode: _numberFocus,
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
                MacButton(
                  label: context.l10n.insert,
                  onPressed: widget.pending ? null : _insertNumber,
                ),
              ],
            ),
          if (s != null && s.insufficient)
            Padding(
              padding: const EdgeInsets.only(top: MacMetrics.gapTight),
              child: Text(context.l10n.noUnitIsSuggestedFillTheOther, style: small),
            ),
          if (s != null && s.references.isNotEmpty) ...[
            const SizedBox(height: MacMetrics.gap),
            Text(context.l10n.references, style: small),
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
              title: context.l10n.equationsCount(s.equations.length),
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
          if (s != null) ...[
            const SizedBox(height: MacMetrics.gap),
            // the forms a slot can open: a choice (any kind of value), a
            // negation (a truth value, or not yet known)
            Wrap(
              spacing: MacMetrics.gapTight,
              runSpacing: MacMetrics.gapTight,
              children: [
                MacButton(
                  key: const ValueKey('op-choose'),
                  label: context.l10n.composeChoose,
                  tooltip: context.l10n.aChoiceIfThenElse,
                  onPressed: widget.pending
                      ? null
                      : () => widget.onCompose(
                          pb.ComposeAction(nodeId: widget.nodeId, choose: pb.Unit()),
                        ),
                ),
                if (s.booleans.isNotEmpty || !s.hasExpected())
                  MacButton(
                    key: const ValueKey('op-not'),
                    label: 'not',
                    tooltip: context.l10n.negateThis,
                    onPressed: widget.pending
                        ? null
                        : () => widget.onCompose(
                            pb.ComposeAction(
                              nodeId: widget.nodeId,
                              operator: pb.ComposeOperator(op: '!', before: false),
                            ),
                          ),
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
                  tooltip: context.l10n.insertAfterThis(glyph),
                  onPressed: widget.pending
                      ? null
                      : () => widget.onCompose(
                          pb.ComposeAction(
                            nodeId: widget.nodeId,
                            operator: pb.ComposeOperator(op: op, before: false),
                          ),
                        ),
                ),
              // the logical operators, on what may be a truth value: `and`
              // and `or` take a slot after it, `not` wraps it in place
              if (widget.truthValued != false) ...[
                for (final op in const ['&&', '||'])
                  MacButton(
                    key: ValueKey('op-$op'),
                    label: operatorGlyph(op),
                    tooltip: context.l10n.insertAfterThis(op),
                    onPressed: widget.pending
                        ? null
                        : () => widget.onCompose(
                            pb.ComposeAction(
                              nodeId: widget.nodeId,
                              operator: pb.ComposeOperator(op: op, before: false),
                            ),
                          ),
                  ),
                MacButton(
                  key: const ValueKey('op-not'),
                  label: operatorGlyph('!'),
                  tooltip: context.l10n.negateThis,
                  onPressed: widget.pending
                      ? null
                      : () => widget.onCompose(
                          pb.ComposeAction(
                            nodeId: widget.nodeId,
                            operator: pb.ComposeOperator(op: '!', before: false),
                          ),
                        ),
                ),
              ],
              MacDropdown<String>(
                key: const ValueKey('op-compare'),
                compact: true,
                value: null,
                hint: context.l10n.compare,
                items: const ['<', '<=', '>', '>=', '==', '!='],
                labelOf: operatorGlyph,
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
                  hint: context.l10n.function,
                  items: s.equations,
                  labelOf: (e) => e.shape,
                  onChanged: (e) => widget.onCompose(
                    pb.ComposeAction(
                      nodeId: widget.nodeId,
                      call: pb.ComposeCall(name: e.name, arity: '?'.allMatches(e.insert).length),
                    ),
                  ),
                ),
              // the natural forms: read every element of a collection,
              // or ask whether a value lies between two ends
              if (n == null || !n.hasActual() || n.actual.hasElement())
                MacDropdown<String>(
                  key: const ValueKey('op-binder'),
                  compact: true,
                  value: null,
                  hint: context.l10n.eachElement,
                  items: const ['all', 'any', 'map', 'filter'],
                  labelOf: (f) => switch (f) {
                    'all' => context.l10n.allSatisfy,
                    'any' => context.l10n.anySatisfies,
                    'map' => context.l10n.mapEach,
                    _ => context.l10n.filter,
                  },
                  onChanged: (f) => widget.onCompose(
                    pb.ComposeAction(
                      nodeId: widget.nodeId,
                      binder: pb.ComposeBinder(form: f),
                    ),
                  ),
                ),
              if (widget.truthValued != true)
                MacButton(
                  key: const ValueKey('op-range'),
                  label: context.l10n.range,
                  tooltip: context.l10n.betweenTwoEndsIn,
                  onPressed: widget.pending
                      ? null
                      : () => widget.onCompose(
                          pb.ComposeAction(nodeId: widget.nodeId, range: pb.Unit()),
                        ),
                ),
              // a choice around it: the component becomes one outcome
              MacButton(
                key: const ValueKey('op-choose'),
                label: context.l10n.composeChoose,
                tooltip: context.l10n.aChoiceIfThenElse,
                onPressed: widget.pending
                    ? null
                    : () => widget.onCompose(
                        pb.ComposeAction(nodeId: widget.nodeId, choose: pb.Unit()),
                      ),
              ),
              MacButton(
                key: const ValueKey('op-remove'),
                label: context.l10n.remove,
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
              child: Text(context.l10n.thisPartIsEditedAsText, style: small),
            ),
          if (n != null && n.kind == 'binder')
            Padding(
              padding: const EdgeInsets.only(top: MacMetrics.gapTight),
              child: Text(
                n.hasParamType()
                    ? context.l10n.paramIsEachElementOf(n.param, n.paramType.description)
                    : context.l10n.paramIsEachElement(n.param),
                key: const ValueKey('binder-local-note'),
                style: small,
              ),
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
          Text(
            label,
            style: TextStyle(fontSize: MacType.body, color: t.textPrimary),
          ),
          Expanded(
            child: Text(
              detail,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
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
