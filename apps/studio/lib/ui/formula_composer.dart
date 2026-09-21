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
/// and sends it the structured actions (`ComposeFormula`) the text
/// cannot decide alone.  A key acts on **the editor's own state** — the
/// text and the caret as of the last key, kept here ahead of the rebuild
/// a dispatch schedules, the way a formula editor keeps its sequence and
/// cursor (GeoGebra's `EditorState`) — and never on what the compiler
/// last read: a character is a text edit of the draft at the caret's
/// byte, which the compiler reads like any typing in the Text view.  A
/// key that needs the compiler's tree while the tree is not current
/// waits in a queue, in order, until the reading arrives (the debounced
/// check is flushed for it); nothing typed is ever dropped.  Nothing in
/// this file parses, types, converts a unit or decides what fits.
///
/// Keys (the primary path; the palette is the pointer's):
///
///   ← →            the previous / next caret stop — out of a
///                  denominator, past a parenthesis, into the next part;
///                  one character at a time inside a name or a number;
///   ↑ ↓            the nearest stop on the row above / below (a
///                  numerator from its denominator, a branch from the
///                  next);
///   Home / End     the first / last stop of the enclosing part; again,
///                  of the whole formula;
///   Tab / ⇧Tab     the next / previous empty slot;
///   letters digits type into a slot or extend the name or number the
///                  caret touches; a space after a number starts its unit;
///   + − * / < > = & |   the operator where the caret is, with a slot for
///                  the operand not yet written (`-` in a slot is a sign);
///   !              negates the part the caret touches;
///   (              applies the name before the caret (`clamp` →
///                  `clamp(?, ?, ?)`), or groups a slot;
///   )  ,           leave the enclosing parentheses / move to the next
///                  argument;
///   ⌫ ⌦            a character; the last character of a value leaves a
///                  slot; a slot goes with its operator; a whole
///                  structure after the caret goes as a whole;
///   ⌃Space         completion at the caret; ↑ ↓ ⏎ Tab in the list;
///   Esc            close completion, then clear the caret and selection.
///
/// While the text differs from what the compiler last read — characters
/// just typed — the part being typed into shows as text in place
/// (`PendingText`), the caret inside it, and the rest keeps its
/// structure; when the compiler's reading arrives the picture follows
/// and the part under the caret is selected.  Text the compiler could
/// not read is shown as text and edited as text, so it can be mended.
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
    this.large = false,
    this.palette = true,
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

  /// The formula sheet's field: the display scale, room around the
  /// expression.
  final bool large;

  /// Draw the slot panel under the field (the inspector); the sheet
  /// places it in its own column (`FormulaComposer.slotPanel`).
  final bool palette;

  @override
  State<FormulaComposer> createState() => _FormulaComposerState();
}

/// A key the composer holds until the compiler's reading of the text
/// arrives: what was pressed, as it was pressed.
class _QueuedKey {
  const _QueuedKey(this.key, this.character, {required this.shift});
  final LogicalKeyboardKey key;
  final String? character;
  final bool shift;
}

enum _Handled { yes, no, deferred }

class _FormulaComposerState extends State<FormulaComposer> {
  final FocusNode _focus = FocusNode(debugLabel: 'composer');
  final FormulaGeometry _geometry = FormulaGeometry();
  final GlobalKey _fieldKey = GlobalKey(debugLabel: 'composer-field');
  final GlobalKey _rawKey = GlobalKey(debugLabel: 'composer-raw');

  /// The last projection of this mapping that parsed: what the picture
  /// is drawn from while the compiler reads a newer text.
  pb.FormulaProjection? _lastGood;

  /// Where the caret is drawn, in the field's coordinates; computed
  /// after layout from the geometry.
  Rect? _caretRect;

  /// A transient word under the field: why a key did nothing.
  String? _hint;

  // ---- the editor's own state --------------------------------------------
  //
  // The text and the caret as of the last key.  A dispatch changes the
  // store at once but the widget only at the next frame; two keys in one
  // frame would otherwise both act on the first's text.  Both follow the
  // widget at every rebuild and lead it between rebuilds.

  late String _source = widget.source;
  late CaretState? _caret = widget.composer.caret;

  /// The text on screen is the composer's own typing (or a structured
  /// action's answer): the picture stays live with the edited part as
  /// text while the compiler reads.  Text that changed any other way (the
  /// Text view, a reload) is shown as text until read.
  bool _ownEdit = false;

  /// A structured action sent and not yet answered, as of the last key.
  bool _composing = false;

  /// The keys that wait for the compiler's reading, in order.
  final List<_QueuedKey> _queue = [];

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
      _ownEdit = false;
      _queue.clear();
    }
    // a structured action's answer landed (sent by this composer, and the
    // store no longer holds it — the answer may have come within the
    // frame the request went out in): the composer's own edit.  Text
    // that changed any other way is another hand's.
    final answered = (_composing || old.composer.pendingCompose) && !widget.composer.pendingCompose;
    if (widget.source != _source) _ownEdit = answered;
    _source = widget.source;
    _caret = widget.composer.caret;
    if (!widget.composer.pendingCompose) _composing = false;
    final wasInSync = _inSyncOf(old.projection, old.source);
    _remember(widget.projection);
    if (old.composer.caret != widget.composer.caret || old.source != widget.source) {
      _hint = null;
    }
    final inSync = _inSync;
    if (_queue.isNotEmpty && !_composing) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _drain());
    } else if (!wasInSync && inSync && _caret != null && _selected == null && _focus.hasFocus) {
      // the reading arrived: the part under the caret is what the palette
      // is about, as after a click
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted || _selected != null || _caret == null) return;
        final stop = resolveStop(_stopsOf(_picture), _caret);
        if (stop != null) _select(stop.node);
      });
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

  bool _inSyncOf(pb.FormulaProjection? p, String source) {
    if (source.trim().isEmpty) return true;
    return p != null && p.source == source && p.parseOk;
  }

  /// The compiler read exactly the text the editor holds.
  bool get _inSync => _inSyncOf(widget.projection, _source);

  /// The projection the picture is drawn from, and the part shown as the
  /// text being typed into it when the text has moved on from it.
  ///
  /// Three states, one policy (app/composer.dart `composerInSync`): the
  /// compiler read this text (in sync: the picture is current); it has not
  /// read it yet (awaiting: the last picture with the edited part as text,
  /// live — keys keep working on it, by character in the edited part); it
  /// read it and could not, or the text came from another hand (stale:
  /// the text, editable as text, with the notice).
  ({
    pb.FormulaProjection? projection,
    PendingText? pending,
    ({int start, int end, int newLength})? region,
    bool stale,
  })
  get _picture {
    final p = widget.projection;
    if (_inSync) return (projection: p, pending: null, region: null, stale: false);
    final refused = p != null && p.source == _source && !p.parseOk;
    final good = _lastGood;
    if (refused || !_ownEdit) return (projection: null, pending: null, region: null, stale: true);
    // the composer's own first characters into an empty formula: text,
    // live, until read
    if (good == null || !good.hasRoot()) {
      return (projection: null, pending: null, region: null, stale: false);
    }
    final region = changedRegion(good.source, _source);
    if (region == null) return (projection: good, pending: null, region: null, stale: false);
    final node = nodeContaining(good.root, region.start, region.end);
    if (node == null) return (projection: null, pending: null, region: null, stale: true);
    final delta = region.newLength - (region.end - region.start);
    final text = excerptOf(_source, node.range.start, node.range.end + delta);
    return (
      projection: good,
      pending: PendingText(nodeId: node.id, text: text),
      // the whole part is text now: its range in the text on screen
      region: (start: node.range.start, end: node.range.end, newLength: text.length),
      stale: false,
    );
  }

  /// The stops of the picture, at the bytes of the text on screen.
  List<CaretStop> _stopsOf(
    ({
      pb.FormulaProjection? projection,
      PendingText? pending,
      ({int start, int end, int newLength})? region,
      bool stale,
    })
    picture,
  ) {
    final stops = caretStops(picture.projection);
    final region = picture.region;
    return region == null ? stops : shiftedStops(stops, region);
  }

  /// The byte range of the part being typed into, in the text on screen;
  /// `null` when the picture is current.
  ({int start, int end})? _pendingRange(
    ({
      pb.FormulaProjection? projection,
      PendingText? pending,
      ({int start, int end, int newLength})? region,
      bool stale,
    })
    picture,
  ) {
    final r = picture.region;
    if (r == null) return null;
    return (start: r.start, end: r.start + utf8.encode(picture.pending!.text).length);
  }

  String? get _selected => widget.composer.selectedNode;

  // ---- moves ---------------------------------------------------------------

  void _select(String? id) =>
      widget.dispatch(FormulaNodeSelected(mappingId: widget.mappingId, nodeId: id));

  void _setCaret(CaretState? caret) {
    _caret = caret;
    widget.dispatch(FormulaCaretMoved(mappingId: widget.mappingId, caret: caret));
  }

  /// The caret lands at a stop; the selection follows it to the part the
  /// stop belongs to, so the palette is about the same part the keys are.
  void _moveTo(CaretStop stop) {
    _setCaret(stateOf(stop));
    if (_selected != stop.node) _select(stop.node);
    _focus.requestFocus();
  }

  /// The caret lands at a byte of the text (inside the part being typed).
  void _moveToByte(int offset) {
    _setCaret(CaretState(offset));
    _focus.requestFocus();
  }

  void _clear() {
    _queue.clear();
    _setCaret(null);
    _select(null);
  }

  void _compose(pb.ComposeAction a) {
    _composing = true;
    widget.dispatch(ComposeRequested(mappingId: widget.mappingId, action: a));
    // refused without a word (the reducer's policy) nothing rebuilds: the
    // keys behind it must not wait for an answer that never comes
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_composing || widget.composer.pendingCompose) return;
      _composing = false;
      _drain();
    });
  }

  /// A text edit of the draft at byte offsets: the same path the Text
  /// view's typing takes.  The caret lands where the plan says; a word
  /// character asks for completion there.
  void _applyEdit(TextEditPlan e, {bool complete = false}) {
    final source = _source;
    final r = codeUnitRange(source, e.start, e.end);
    final next = source.replaceRange(r.start, r.end, e.text);
    final caretBytes = e.caretAfter;
    if (next != source) {
      _source = next;
      _ownEdit = true;
      widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: next));
    }
    _setCaret(CaretState(caretBytes));
    if (complete) {
      widget.dispatch(
        CompletionRequested(mappingId: widget.mappingId, source: next, offset: caretBytes),
      );
    } else if (widget.completion != null) {
      widget.dispatch(const CompletionDismissed());
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
    final source = _source;
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
    _source = next;
    _ownEdit = true;
    widget.dispatch(const CompletionDismissed());
    widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: next));
    _setCaret(CaretState(caretBytes));
    return true;
  }

  void _requestCompletion() {
    final offset = _caret?.offset;
    if (offset == null) return;
    widget.dispatch(
      CompletionRequested(mappingId: widget.mappingId, source: _source, offset: offset),
    );
  }

  // ---- keys ----------------------------------------------------------------

  static final Set<LogicalKeyboardKey> _navigationKeys = {
    LogicalKeyboardKey.arrowLeft,
    LogicalKeyboardKey.arrowRight,
    LogicalKeyboardKey.arrowUp,
    LogicalKeyboardKey.arrowDown,
    LogicalKeyboardKey.home,
    LogicalKeyboardKey.end,
    LogicalKeyboardKey.tab,
    LogicalKeyboardKey.backspace,
    LogicalKeyboardKey.delete,
  };

  /// The keys the composer acts on; the rest (⌘↩, ⌘Z, …) pass through.
  static bool _isComposerKey(LogicalKeyboardKey k, String? c) =>
      (c != null && c.isNotEmpty && !HardwareKeyboard.instance.isMetaPressed) ||
      _navigationKeys.contains(k);

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
          _handle(_QueuedKey(k, null, shift: HardwareKeyboard.instance.isShiftPressed));
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
    if (k == LogicalKeyboardKey.escape) {
      // the innermost thing first: a caret or selection; then the editor's
      // own Esc (revert) sees it
      if (_caret == null && _selected == null && _queue.isEmpty) return KeyEventResult.ignored;
      _clear();
      return KeyEventResult.handled;
    }
    final c = e.character;
    if (!_isComposerKey(k, c)) return KeyEventResult.ignored;
    final key = _QueuedKey(k, c, shift: HardwareKeyboard.instance.isShiftPressed);
    // a structured action in flight, or keys already waiting: this one
    // waits its turn
    if (_composing || _queue.isNotEmpty) {
      _queue.add(key);
      return KeyEventResult.handled;
    }
    return switch (_handle(key)) {
      _Handled.yes => KeyEventResult.handled,
      _Handled.no => KeyEventResult.ignored,
      _Handled.deferred => _defer(key),
    };
  }

  KeyEventResult _defer(_QueuedKey key) {
    _queue.add(key);
    widget.dispatch(DefinitionDraftFlushRequested(widget.mappingId));
    return KeyEventResult.handled;
  }

  /// The reading arrived (or the action was answered): the waiting keys
  /// act, in order, until one needs a reading that is not there yet.
  void _drain() {
    while (mounted && _queue.isNotEmpty && !_composing) {
      final head = _queue.first;
      final r = _handle(head);
      if (r == _Handled.deferred) {
        widget.dispatch(DefinitionDraftFlushRequested(widget.mappingId));
        return;
      }
      _queue.removeAt(0);
    }
  }

  /// One key over the editor's state.
  _Handled _handle(_QueuedKey key) {
    final k = key.key;
    final c = key.character;
    final source = _source;
    final picture = _picture;
    final p = picture.projection;
    final root = p != null && p.hasRoot() ? p.root : null;
    final stops = _stopsOf(picture);
    final caret = _caret;
    final stop = resolveStop(stops, caret);
    final selected = _selected;
    final bytes = utf8.encode(source).length;
    final wordChar = c != null && RegExp(r'^[A-Za-z_]$').hasMatch(c);
    final pending = _pendingRange(picture);
    final shift = key.shift;

    // Tab: the next slot — the next `?` of the text, from the caret or
    // the selected part; no reading needed
    if (k == LogicalKeyboardKey.tab) {
      final from = caret?.offset ?? stops.where((s) => s.node == selected).firstOrNull?.offset;
      final at = _nextSlotByte(source, from, backwards: shift);
      if (at != null) {
        final s = stops.where((s) => s.offset == at && s.side == StopSide.inSlot).firstOrNull;
        if (s != null) {
          _moveTo(s);
        } else {
          _moveToByte(at);
        }
      }
      return _Handled.yes;
    }

    // no tree drawn: text — an empty formula (one slot of Studio's: typing
    // writes it), the composer's own first characters, or text the
    // compiler could not read (mended as text)
    if (root == null) {
      final plain = picture.stale && source.trim().isNotEmpty;
      if (source.trim().isEmpty) {
        if (c == null || !RegExp(r'^[A-Za-z0-9_(!-]$').hasMatch(c)) return _Handled.no;
        final plan = textCharacterAt('?', 0, c);
        if (plan is! EditText) return _Handled.no;
        _source = '?';
        _caret = const CaretState(0);
        _applyEdit(plan.edit, complete: wordChar);
        return _Handled.yes;
      }
      final at = (caret?.offset ?? bytes).clamp(0, bytes);
      if (k == LogicalKeyboardKey.arrowLeft || k == LogicalKeyboardKey.arrowRight) {
        final to = k == LogicalKeyboardKey.arrowLeft
            ? previousCharStart(source, at)
            : nextCharEnd(source, at);
        if (to != null) _moveToByte(to);
        return _Handled.yes;
      }
      if (k == LogicalKeyboardKey.home || k == LogicalKeyboardKey.end) {
        _moveToByte(k == LogicalKeyboardKey.home ? 0 : bytes);
        return _Handled.yes;
      }
      if (k == LogicalKeyboardKey.arrowUp || k == LogicalKeyboardKey.arrowDown) {
        return _Handled.yes;
      }
      if (k == LogicalKeyboardKey.backspace) {
        return _run(textBackspaceAt(source, at, plain: plain));
      }
      if (k == LogicalKeyboardKey.delete) {
        return _run(textDeleteAt(source, at, plain: plain));
      }
      if (c == null || c.isEmpty) return _Handled.no;
      return _run(textCharacterAt(source, at, c, plain: plain), complete: wordChar);
    }

    if (stop == null && caret == null) {
      // keys on a selected part, without a caret: the operators and ⌫
      // as the palette does them
      if (selected == null || picture.stale) return _Handled.no;
      if (pending != null) return _Handled.deferred;
      final op = c == null ? null : operatorFor(c);
      if (op != null) {
        _compose(
          pb.ComposeAction(
            nodeId: selected,
            operator: pb.ComposeOperator(op: op, before: false),
          ),
        );
        return _Handled.yes;
      }
      if (k == LogicalKeyboardKey.backspace || k == LogicalKeyboardKey.delete) {
        _compose(pb.ComposeAction(nodeId: selected, remove: pb.Unit()));
        return _Handled.yes;
      }
      if (k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowLeft) {
        final own = stops.where((s) => s.node == selected).toList();
        if (own.isNotEmpty) _moveTo(k == LogicalKeyboardKey.arrowRight ? own.last : own.first);
        return _Handled.yes;
      }
      return _Handled.no;
    }
    final at = (caret?.offset ?? stop!.offset).clamp(0, bytes);
    // inside the part being typed: by character, as the text is
    final inPending = pending != null && at >= pending.start && at <= pending.end;

    // moves: always, also while the compiler reads
    if (k == LogicalKeyboardKey.arrowRight || k == LogicalKeyboardKey.arrowLeft) {
      final left = k == LogicalKeyboardKey.arrowLeft;
      if (inPending) {
        final to = left ? previousCharStart(source, at) : nextCharEnd(source, at);
        if (to != null && to >= pending.start && to <= pending.end) {
          _moveToByte(to);
          return _Handled.yes;
        }
        // out of the part: the nearest stop beyond it
        final beyond = left
            ? stops.lastWhere((s) => s.offset < pending.start, orElse: () => stops.first)
            : stops.firstWhere((s) => s.offset > pending.end, orElse: () => stops.last);
        if (beyond.offset != at) _moveTo(beyond);
        return _Handled.yes;
      }
      if (stop == null) return _Handled.yes;
      final n = left ? previousStop(stops, stop) : nextStop(stops, stop);
      if (n == null) return _Handled.yes;
      // a stop inside the part being typed is a byte of it
      if (pending != null && n.offset > pending.start && n.offset < pending.end) {
        _moveToByte(left ? pending.end : pending.start);
      } else {
        _moveTo(n);
      }
      return _Handled.yes;
    }
    if (stop == null) return _Handled.no;
    if (k == LogicalKeyboardKey.arrowUp || k == LogicalKeyboardKey.arrowDown) {
      final n = _verticalNeighbour(stops, stop, up: k == LogicalKeyboardKey.arrowUp);
      if (n != null) _moveTo(n);
      return _Handled.yes;
    }
    if (k == LogicalKeyboardKey.home) {
      _moveTo(homeStop(stops, stop));
      return _Handled.yes;
    }
    if (k == LogicalKeyboardKey.end) {
      _moveTo(endStop(stops, stop));
      return _Handled.yes;
    }
    // `)` and `,` leave a group / move to the next argument: over the
    // tree when it is current; while the compiler reads, the text tells
    // when the parenthesis or the next slot is right there, else the key
    // waits for the reading
    if (c == ')') {
      if (textCharacterAt(source, at, ')') case EditText(:final edit)) return _run(EditText(edit));
      if (!_inSync) return _Handled.deferred;
      _moveTo(exitGroup(stops, root, stop));
      return _Handled.yes;
    }
    if (c == ',') {
      final tail = source.substring(codeUnitRange(source, at, at).start);
      final m = RegExp(r'^,\s*\?').firstMatch(tail);
      if (m != null) {
        _moveToByte(at + m.end - 1);
        return _Handled.yes;
      }
      if (!_inSync) return _Handled.deferred;
      final n = nextArgument(stops, root, stop);
      if (n != null) _moveTo(n);
      return _Handled.yes;
    }
    // edits: the text rule at the caret's byte; a structural rule only
    // over a picture the compiler has read
    if (inPending) {
      if (k == LogicalKeyboardKey.backspace) return _run(textBackspaceAt(source, at));
      if (k == LogicalKeyboardKey.delete) return _run(textDeleteAt(source, at));
      if (c == null || c.isEmpty) return _Handled.no;
      return _run(textCharacterAt(source, at, c), complete: wordChar || c == ' ');
    }
    if (picture.stale) return _Handled.no;
    final KeyPlan plan;
    if (k == LogicalKeyboardKey.backspace) {
      plan = backspaceAt(p!, stops, stop, at: at, source: source);
    } else if (k == LogicalKeyboardKey.delete) {
      plan = deleteAt(p!, stops, stop, at: at, source: source);
    } else if (c != null && c.isNotEmpty) {
      plan = characterAt(p!, stop, c, at: at, source: source);
    } else {
      return _Handled.no;
    }
    // a structured action is the compiler's, over the text it read
    if (plan is Structural && !_inSync) return _Handled.deferred;
    return _run(plan, complete: wordChar || c == ' ');
  }

  /// The `?` after / before byte [from] in the text, wrapping; `null`
  /// when the text has none.
  int? _nextSlotByte(String source, int? from, {required bool backwards}) {
    final slots = <int>[];
    var bytes = 0;
    for (final rune in source.runes) {
      if (rune == 0x3F) slots.add(bytes);
      bytes += utf8.encode(String.fromCharCode(rune)).length;
    }
    if (slots.isEmpty) return null;
    if (from == null) return backwards ? slots.last : slots.first;
    if (backwards) {
      // the slot the caret stands in (`?` at from) is not the previous one
      return slots.lastWhere((s) => s < from, orElse: () => slots.last);
    }
    return slots.firstWhere((s) => s > from, orElse: () => slots.first);
  }

  _Handled _run(KeyPlan plan, {bool complete = false}) {
    switch (plan) {
      case EditText(:final edit):
        _applyEdit(edit, complete: complete);
        return _Handled.yes;
      case Structural(:final action):
        if (widget.completion != null) widget.dispatch(const CompletionDismissed());
        _compose(action);
        return _Handled.yes;
      case MoveTo(:final stop):
        _moveTo(stop);
        return _Handled.yes;
      case NeedsStructure():
        return _Handled.deferred;
      case Refused(:final reason):
        if (reason == Refused.needsOperator) {
          setState(() => _hint = context.l10n.typeAnOperatorFirst);
        }
        return _Handled.yes;
    }
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

  /// Where a byte of the part being typed stands: its text measured up
  /// to the byte with the style it is drawn in.
  Rect? _pendingRect(int at) {
    final picture = _picture;
    final pending = picture.pending;
    final range = _pendingRange(picture);
    final box = _fieldBox;
    if (pending == null || range == null || box == null) return null;
    final n = _geometry.rectOf('${pending.nodeId}/text', box);
    if (n == null) return null;
    final head = excerptOf(_source, range.start, at.clamp(range.start, range.end));
    return Rect.fromLTWH(n.left + _textWidth(head, pending.nodeId), n.top, 0, n.height);
  }

  double _textWidth(String text, String nodeId) {
    final style = _geometry.textStyles[nodeId] ?? TextStyle(fontSize: MacType.body);
    final painter = TextPainter(
      text: TextSpan(text: text, style: style),
      textDirection: TextDirection.ltr,
    )..layout();
    return painter.width;
  }

  /// The place nearest a point in the field — a stop, or a byte of the
  /// part being typed — the same row first.
  ({CaretStop? stop, int? byte}) _placeNear(Offset local, List<CaretStop> stops) {
    CaretStop? best;
    int? bestByte;
    var bestScore = double.infinity;
    double score(Rect r) {
      final dy = local.dy < r.top
          ? r.top - local.dy
          : local.dy > r.bottom
          ? local.dy - r.bottom
          : 0.0;
      return (local.dx - r.left).abs() + dy * 4;
    }

    for (final s in stops) {
      final r = _stopRect(s);
      if (r == null) continue;
      final sc = score(r);
      if (sc < bestScore) {
        bestScore = sc;
        best = s;
      }
    }
    final range = _pendingRange(_picture);
    if (range != null) {
      var at = range.start;
      while (at <= range.end) {
        final r = _pendingRect(at);
        if (r != null) {
          final sc = score(r);
          if (sc < bestScore) {
            bestScore = sc;
            best = null;
            bestByte = at;
          }
        }
        final next = nextCharEnd(_source, at);
        if (next == null || next > range.end) break;
        at = next;
      }
    }
    return (stop: best, byte: bestByte);
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

  void _goNear(Offset local, List<CaretStop> stops) {
    final near = _placeNear(local, stops);
    if (near.stop != null) {
      _moveTo(near.stop!);
    } else if (near.byte != null) {
      _moveToByte(near.byte!);
    } else {
      _focus.requestFocus();
    }
  }

  /// A tap on a part: the caret goes where the tap was — between two
  /// characters of a name or number, before or after any other part, in
  /// a slot — and the part is selected.
  void _tapNode(pb.FormulaNode n, Offset global) {
    final box = _fieldBox;
    if (box == null) return;
    final local = box.globalToLocal(global);
    final stops = _stopsOf(_picture);
    final own = stops.where((s) => s.node == n.id).toList();
    if (own.isEmpty && _picture.pending?.nodeId != n.id) {
      _select(n.id);
      _focus.requestFocus();
      return;
    }
    _goNear(local, own);
  }

  void _tapField(TapUpDetails d) {
    final box = _fieldBox;
    if (box == null) return;
    final local = box.globalToLocal(d.globalPosition);
    if (_picture.projection == null && _source.trim().isNotEmpty) {
      // raw text: the caret between its characters
      _moveToByte(_rawByteNear(local) ?? utf8.encode(_source).length);
      return;
    }
    _goNear(local, _stopsOf(_picture));
  }

  /// The byte of the raw text nearest a point.
  int? _rawByteNear(Offset local) {
    final box = _fieldBox;
    final raw = _rawKey.currentContext?.findRenderObject() as RenderBox?;
    if (box == null || raw == null || !raw.hasSize) return null;
    final origin = raw.localToGlobal(Offset.zero, ancestor: box);
    final painter = TextPainter(
      text: TextSpan(text: _source, style: _rawStyle(MacTokens.of(context))),
      textDirection: TextDirection.ltr,
    )..layout();
    final pos = painter.getPositionForOffset(local - origin);
    return byteOffsetOf(_source, pos.offset);
  }

  TextStyle _rawStyle(MacTokens t) => TextStyle(
    fontSize: widget.large ? MacType.display : MacType.code,
    fontFamily: 'Menlo',
    color: t.textPrimary,
  );

  void _placeCaret() {
    final picture = _picture;
    final caret = _caret;
    Rect? rect;
    if (caret != null) {
      final range = _pendingRange(picture);
      if (range != null && caret.offset >= range.start && caret.offset <= range.end) {
        rect = _pendingRect(caret.offset);
      }
      if (rect == null) {
        final stop = resolveStop(_stopsOf(picture), caret);
        if (stop != null) rect = _stopRect(stop);
      }
      // raw text (not read yet, or unreadable): the caret between its
      // characters
      final box = _fieldBox;
      final raw = _rawKey.currentContext?.findRenderObject() as RenderBox?;
      if (rect == null && box != null && raw != null && raw.hasSize) {
        final origin = raw.localToGlobal(Offset.zero, ancestor: box);
        final r = codeUnitRange(_source, 0, caret.offset);
        final painter = TextPainter(
          text: TextSpan(
            text: _source.substring(0, r.end),
            style: _rawStyle(MacTokens.of(context)),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        rect = Rect.fromLTWH(origin.dx + painter.width, origin.dy, 0, raw.size.height);
      }
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
    final stale = picture.stale && _source.trim().isNotEmpty;
    final enabled = !pending;
    final stop = _caret == null ? null : resolveStop(_stopsOf(picture), _caret);
    final large = widget.large;
    WidgetsBinding.instance.addPostFrameCallback((_) => _placeCaret());

    final Widget content;
    if (root == null) {
      content = _source.trim().isEmpty
          ? _EmptySlot(
              selected: selected == 'r' || _caret != null,
              large: large,
              onTap: () {
                _setCaret(const CaretState(0));
                _select('r');
                _focus.requestFocus();
              },
              hint: widget.projection?.hasResult() == true
                  ? l10n.producesDescription(widget.projection!.result.description)
                  : l10n.typeToWrite,
            )
          // text just typed (not read yet) or text the compiler could not
          // read: shown and edited as text
          : Text(
              _source,
              key: _rawKey,
              style: _rawStyle(t).copyWith(color: stale ? t.textSecondary : t.textPrimary),
            );
    } else {
      content = FormulaRender(
        projection: p!,
        concepts: widget.concepts,
        geometry: _geometry,
        selected: selected,
        pending: picture.pending,
        large: large,
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
    final padding = large
        ? const EdgeInsets.symmetric(horizontal: MacMetrics.gapGroup, vertical: MacMetrics.gap)
        : const EdgeInsets.symmetric(horizontal: 6, vertical: 5);
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
            constraints: BoxConstraints(minHeight: large ? 96 : 34),
            padding: padding,
            alignment: large ? Alignment.centerLeft : null,
            decoration: BoxDecoration(
              color: t.control,
              borderRadius: BorderRadius.circular(large ? 8 : 5),
              border: Border.all(color: _focus.hasFocus ? t.accent : t.hairline),
            ),
            child: Stack(
              children: [
                content,
                if (caret != null && _focus.hasFocus)
                  Positioned(
                    left: caret.left - padding.left - 1,
                    top: caret.top - padding.top,
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
    final slotPanel = selected != null && !stale
        ? _SlotPanel(
            key: ValueKey('slot-panel-$selected'),
            mappingId: widget.mappingId,
            nodeId: selected,
            node: selectedNode,
            slot: widget.composer.slot,
            pending: pending,
            truthValued: _truthValued(selectedNode),
            onCompose: _compose,
            onDeselect: _clear,
          )
        : null;
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
                    widget.projection == null || widget.projection!.source != _source
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
        if (slotPanel != null && widget.palette)
          Padding(
            padding: const EdgeInsets.only(top: MacMetrics.gap),
            child: slotPanel,
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
  const _EmptySlot({
    required this.selected,
    required this.onTap,
    required this.hint,
    this.large = false,
  });
  final bool selected;
  final VoidCallback? onTap;
  final String hint;
  final bool large;

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
                  style: TextStyle(
                    fontSize: large ? MacType.body : MacType.secondary,
                    color: t.textSecondary,
                  ),
                ),
              ),
            ),
          ),
        ),
        Expanded(
          child: Text(
            hint,
            style: TextStyle(
              fontSize: large ? MacType.body : MacType.secondary,
              color: t.textTertiary,
            ),
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
