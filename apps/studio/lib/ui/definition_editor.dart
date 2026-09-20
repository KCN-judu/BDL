/// The definition editor: an authoring surface over the compiler's verdict.
///
/// Studio owns the draft (`DefinitionDraft` in the editor state); the
/// compiler owns what it means.  The field edits the draft, the compiler is
/// asked as the designer types (debounced, read-only), and the project
/// changes only on *Add definition* / *Save definition* / *Detach*.
///
/// What the designer sees, ranked by how often the task needs it:
///
/// 1. the verdict on the text as it stands — one quiet status line under
///    the field (dot + words; the words carry the meaning, the dot the tone);
/// 2. where the problem is — the span underlined in the field, and the
///    diagnostic rows beneath with the excerpt, explanation and fixes;
/// 3. the names in scope — the field's hint, from the signature;
/// 4. draft vs committed — the buttons (*Save definition* / *Revert*) exist
///    only while the two differ;
/// 5. a conflict — the committed definition moved under a dirty draft —
///    as a notice with the two ways out, never a silent overwrite.
///
/// Keyboard (documented in docs/architecture/studio-ui.md §4a): ⌘↩ saves the definition
/// while it is dirty; ⌘S keeps its meaning (*Save project*) and never
/// commits a draft; Esc reverts a dirty draft.  Ordinary text-editing
/// shortcuts are untouched; Return inserts a line.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';

import '../l10n/diagnostics.dart' show isPlacementNote;
import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'code/completion_popup.dart';
import 'code/highlighting_controller.dart';
import 'code/hover_card.dart';
import 'code/syntax_theme.dart';
import 'formula_composer.dart';
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';
import 'source_span.dart';

/// Colour-independent tone of the status line; the text always says it too.
enum VerdictTone { none, checking, settled, open, error }

/// Everything the editor shows, derived once per build from state.  Pure,
/// so the wording and the rules are unit-testable without widgets.
@immutable
class DefinitionEditorModel {
  const DefinitionEditorModel._({
    required this.text,
    required this.committed,
    required this.dirty,
    required this.conflict,
    required this.saving,
    required this.statusText,
    required this.tone,
    required this.diagnostics,
    required this.canCommit,
    required this.commitLabel,
    required this.hint,
  });

  factory DefinitionEditorModel({
    required String? committed,
    required DefinitionDraft? draft,
    required pb.MappingAnalysis? committedAnalysis,
    required List<String> inputNames,
    AppLocalizations? l10n,
  }) {
    l10n ??= kEnglish;
    final text = draft?.source ?? committed ?? '';
    final dirty = draft != null && draft.dirtyAgainst(committed);
    final conflict = draft?.conflict ?? false;
    final saving = draft?.pendingCommit != null;
    final hint = inputNames.isEmpty
        ? l10n.expressionWithNoInputs
        : l10n.expressionOver(inputNames.join(', '));

    // Which verdict is on show: the draft's when there is a draft, else the
    // committed one.  Committed diagnostics without a span are about the
    // mapping's place in the design, not its text; they stay in the
    // inspector's Compiler section.
    final List<pb.Diagnostic> diagnostics;
    final String statusText;
    final VerdictTone tone;
    if (draft == null) {
      if (committed == null) {
        diagnostics = const [];
        statusText = l10n.noDefinitionYetALegalStateOther;
        tone = VerdictTone.none;
      } else if (committedAnalysis == null) {
        diagnostics = const [];
        statusText = l10n.checking;
        tone = VerdictTone.checking;
      } else {
        diagnostics = committedAnalysis.diagnostics.where((d) => d.hasSpan()).toList();
        (statusText, tone) = _summarise(l10n, committedAnalysis, parseOk: true);
      }
    } else if (draft.source.trim().isEmpty) {
      diagnostics = const [];
      statusText = committed == null ? l10n.nothingToAddYet : l10n.emptyDetachToRemoveTheDefinition;
      tone = VerdictTone.none;
    } else if (saving) {
      diagnostics = _ofTheText(draft.analysis);
      statusText = l10n.saving;
      tone = VerdictTone.checking;
    } else if (draft.commitError case final e?) {
      diagnostics = _ofTheText(draft.analysis);
      statusText = l10n.notSaved(e);
      tone = VerdictTone.error;
    } else {
      switch (draft.check) {
        case DraftCheck.checking:
          diagnostics = const [];
          statusText = l10n.checking;
          tone = VerdictTone.checking;
        case DraftCheck.unavailable:
          diagnostics = const [];
          statusText = l10n.notChecked(draft.checkError ?? l10n.compilerServiceUnavailable);
          tone = VerdictTone.open;
        case DraftCheck.checked:
          final a = draft.analysis;
          diagnostics = _ofTheText(a);
          (statusText, tone) = a == null
              ? (l10n.checking, VerdictTone.checking)
              : _summarise(l10n, a, parseOk: draft.parseOk);
      }
    }
    return DefinitionEditorModel._(
      text: text,
      committed: committed,
      dirty: dirty,
      conflict: conflict,
      saving: saving,
      statusText: statusText,
      tone: tone,
      diagnostics: diagnostics,
      canCommit: dirty && !conflict && !saving && text.trim().isNotEmpty,
      commitLabel: committed == null ? l10n.addDefinition : l10n.saveDefinition,
      hint: hint,
    );
  }

  final String text;
  final String? committed;
  final bool dirty;
  final bool conflict;
  final bool saving;
  final String statusText;
  final VerdictTone tone;

  /// The diagnostics on show, in the compiler's order.
  final List<pb.Diagnostic> diagnostics;
  final bool canCommit;
  final String commitLabel;
  final String hint;

  bool get canRevert => dirty && !saving;
  bool get canDetach => committed != null && !saving;

  /// A draft verdict's findings about the text: the mapping's own list
  /// less the notes on its place in the design (the inspector's).
  static List<pb.Diagnostic> _ofTheText(pb.MappingAnalysis? a) =>
      a?.diagnostics.where((d) => !isPlacementNote(d.code)).toList() ?? const [];

  /// One line for the ladder verdict.  Open is not an error: it says what is
  /// still to be decided.  Invalid says the first thing that is wrong.
  static (String, VerdictTone) _summarise(
    AppLocalizations l10n,
    pb.MappingAnalysis a, {
    required bool parseOk,
  }) {
    final errors = a.diagnostics.where(
      (d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
    );
    final others = a.diagnostics.where(
      (d) =>
          d.severity != pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR && !isPlacementNote(d.code),
    );
    return switch (a.status) {
      pb.MappingStatus.MAPPING_STATUS_INVALID => (
        errors.firstOrNull?.message ?? (parseOk ? l10n.invalidDefinition : l10n.cannotBeRead),
        VerdictTone.error,
      ),
      pb.MappingStatus.MAPPING_STATUS_OPEN => (
        others.firstOrNull?.message ?? l10n.openSomethingItNeedsIsNotDecided,
        VerdictTone.open,
      ),
      pb.MappingStatus.MAPPING_STATUS_TYPE_VALID ||
      pb.MappingStatus.MAPPING_STATUS_TEMPORALLY_VALID ||
      pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT => (
        l10n.validDefinition,
        VerdictTone.settled,
      ),
      _ => (l10n.noDefinition, VerdictTone.none),
    };
  }
}

class DefinitionEditor extends StatefulWidget {
  const DefinitionEditor({
    super.key,
    required this.mappingId,
    required this.committed,
    required this.draft,
    required this.committedAnalysis,
    required this.inputNames,
    required this.dispatch,
    this.completion,
    this.hover,
    this.highlight,
    this.component,
    this.composer = const ComposerState(),
    this.projection,
    this.concepts = const {},
    this.focusGeneration = 0,
  });

  final int mappingId;

  /// Bumped by *Edit Definition*: the field takes focus once per bump.
  final int focusGeneration;

  /// The IDE service's tokens over the text on screen, and the component
  /// whose body this relationship belongs to (the request's scope).
  final HighlightState? highlight;
  final int? component;

  /// The Formula Composer's editor state, and the projection on screen
  /// (the draft's own, else the committed definition's).
  final ComposerState composer;
  final pb.FormulaProjection? projection;
  final Map<int, pb.ConceptView> concepts;

  /// The committed formula, `null` when the mapping has none.
  final String? committed;
  final DefinitionDraft? draft;
  final pb.MappingAnalysis? committedAnalysis;
  final List<String> inputNames;
  final void Function(AppAction) dispatch;

  /// The open completion pop-up, when it belongs to this field.
  final CompletionState? completion;

  /// The hover card, when it belongs to this field.
  final HoverState? hover;

  @override
  State<DefinitionEditor> createState() => _DefinitionEditorState();
}

class _DefinitionEditorState extends State<DefinitionEditor> {
  late final HighlightingController _controller = HighlightingController(text: _desiredText);
  final FocusNode _focus = FocusNode(debugLabel: 'definition');
  Timer? _highlightPause;

  String get _desiredText => widget.draft?.source ?? widget.committed ?? '';

  @override
  void initState() {
    super.initState();
    _needProjection();
    _needHighlight();
  }

  @override
  void didUpdateWidget(DefinitionEditor old) {
    super.didUpdateWidget(old);
    _syncText();
    _needProjection();
    if (old.mappingId != widget.mappingId || old.component != widget.component) _needHighlight();
    if (widget.focusGeneration != old.focusGeneration) _takeFocus();
  }

  void _takeFocus() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _focus.requestFocus();
    });
  }

  /// The text on screen wants its spans: when the field first shows a
  /// relationship, and after a pause in typing.  The reducer drops a
  /// repeat of a text already asked about.
  void _needHighlight() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _highlight();
    });
  }

  void _highlight() {
    _highlightPause?.cancel();
    widget.dispatch(
      SemanticTokensRequested.formula(
        mappingId: widget.mappingId,
        text: _controller.text,
        component: widget.component,
      ),
    );
  }

  /// Formula mode over a committed definition with no draft: the
  /// projection is fetched once per mapping (a draft carries its own).
  void _needProjection() {
    final c = widget.composer;
    if (!c.formulaMode || widget.draft != null || widget.committed == null) return;
    if (c.mappingId == widget.mappingId && c.projectionGeneration != null) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) widget.dispatch(FormulaProjectionRequested(widget.mappingId));
    });
  }

  /// The field follows the state (revert, reload, a confirmed commit, a
  /// rebase); while the designer types, state and field already agree.
  void _syncText() {
    final desired = _desiredText;
    if (_controller.text == desired) return;
    final offset = _controller.selection.baseOffset.clamp(0, desired.length);
    _controller.value = TextEditingValue(
      text: desired,
      selection: TextSelection.collapsed(offset: offset),
    );
    _needHighlight();
  }

  @override
  void dispose() {
    _hoverTimer?.cancel();
    _highlightPause?.cancel();
    _controller.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _commit() => widget.dispatch(CommitDefinitionRequested(widget.mappingId));
  void _revert() => widget.dispatch(DefinitionDraftReverted(widget.mappingId));

  // ---- completion --------------------------------------------------------

  bool get _completionOpen => widget.completion != null;

  int get _caretBytes => byteOffsetOf(_controller.text, _controller.selection.baseOffset);

  void _requestCompletion() => widget.dispatch(
    CompletionRequested(mappingId: widget.mappingId, source: _controller.text, offset: _caretBytes),
  );

  void _onChanged(String v) {
    widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: v));
    _highlightPause?.cancel();
    _highlightPause = Timer(kHighlightPause, _highlight);
    // With the pop-up open, every keystroke re-asks at the new caret: the
    // service filters by prefix, Studio never does.
    if (_completionOpen) _requestCompletion();
  }

  /// Replace the service's byte range with its insert text; the caret
  /// lands after it.  The candidate's semantics are not consulted.
  void _accept() {
    final c = widget.completion;
    if (c == null || c.items.isEmpty) return;
    final item = c.items[c.selected.clamp(0, c.items.length - 1)];
    final text = _controller.text;
    final range = codeUnitRange(text, item.replaceStart, item.replaceEnd);
    final next = text.replaceRange(range.start, range.end, item.insert);
    _controller.value = TextEditingValue(
      text: next,
      selection: TextSelection.collapsed(offset: range.start + item.insert.length),
    );
    widget.dispatch(const CompletionDismissed());
    widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: next));
  }

  // ---- hover -------------------------------------------------------------

  Timer? _hoverTimer;
  int? _hoverBytes;

  /// The byte offset under the pointer, through the field's render object.
  int? _offsetAt(Offset global) {
    RenderEditable? editable;
    void visit(RenderObject r) {
      if (editable != null) return;
      if (r is RenderEditable) {
        editable = r;
      } else {
        r.visitChildren(visit);
      }
    }

    final root = context.findRenderObject();
    if (root == null) return null;
    visit(root);
    final e = editable;
    if (e == null || !e.hasSize) return null;
    final local = e.globalToLocal(global);
    if (!(Offset.zero & e.size).contains(local)) return null;
    final position = e.getPositionForPoint(global);
    return byteOffsetOf(_controller.text, position.offset);
  }

  void _onHover(PointerHoverEvent e) {
    final bytes = _offsetAt(e.position);
    if (bytes == _hoverBytes) return;
    _hoverBytes = bytes;
    _hoverTimer?.cancel();
    if (bytes == null) {
      _endHover();
      return;
    }
    // A short dwell, so sweeping the pointer across the text asks once.
    _hoverTimer = Timer(const Duration(milliseconds: 250), () {
      widget.dispatch(
        FormulaHoverRequested(mappingId: widget.mappingId, source: _controller.text, offset: bytes),
      );
    });
  }

  void _endHover() {
    _hoverTimer?.cancel();
    _hoverBytes = null;
    if (widget.hover != null) {
      widget.dispatch(
        FormulaHoverRequested(mappingId: widget.mappingId, source: _controller.text, offset: null),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final m = DefinitionEditorModel(
      l10n: context.l10n,
      committed: widget.committed,
      draft: widget.draft,
      committedAnalysis: widget.committedAnalysis,
      inputNames: widget.inputNames,
    );
    _controller.marks = [
      for (final d in m.diagnostics)
        if (d.hasSpan())
          (codeUnitRange(m.text, d.span.start, d.span.end), _severityColor(t, d.severity)),
    ];
    _controller.theme = SyntaxTheme.of(t);
    _controller.setHighlight(widget.highlight);
    final toneColor = switch (m.tone) {
      VerdictTone.none || VerdictTone.checking => t.textTertiary,
      VerdictTone.settled => t.settled,
      VerdictTone.open => t.open,
      VerdictTone.error => t.error,
    };

    final completion = widget.completion;
    final card = widget.hover?.card;

    final formulaMode = widget.composer.formulaMode;
    final projection = widget.projection;
    // the stale-projection policy (app/composer.dart `composerInSync`): the
    // projection is current only when it is of exactly the text on screen
    // and that text parsed; otherwise it is shown dimmed as context and no
    // structured action is offered.  An empty draft is one slot.
    final emptyFormula = m.text.trim().isEmpty;
    final inSync =
        emptyFormula || (projection != null && projection.source == m.text && projection.parseOk);
    return CallbackShortcuts(
      bindings: {
        // The pop-up takes the navigation keys only while it is open; Esc
        // then closes it rather than reverting the draft.
        if (_completionOpen) ...{
          const SingleActivator(LogicalKeyboardKey.arrowDown): () =>
              widget.dispatch(const CompletionMoved(1)),
          const SingleActivator(LogicalKeyboardKey.arrowUp): () =>
              widget.dispatch(const CompletionMoved(-1)),
          const SingleActivator(LogicalKeyboardKey.enter): _accept,
          const SingleActivator(LogicalKeyboardKey.tab): _accept,
          const SingleActivator(LogicalKeyboardKey.escape): () =>
              widget.dispatch(const CompletionDismissed()),
        } else ...{
          if (m.canRevert) const SingleActivator(LogicalKeyboardKey.escape): _revert,
        },
        const SingleActivator(LogicalKeyboardKey.space, control: true): _requestCompletion,
        if (m.canCommit) const SingleActivator(LogicalKeyboardKey.enter, meta: true): _commit,
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Formula | Text: two projections of the one draft (§4b).  Local
          // to this editor; never a project-level view.
          Row(
            children: [
              SizedBox(
                width: 132,
                child: MacSegmented<bool>(
                  key: const ValueKey('definition-mode'),
                  value: formulaMode,
                  options: {true: context.l10n.formula, false: context.l10n.textMode},
                  onChanged: (v) => widget.dispatch(FormulaModeChanged(v)),
                ),
              ),
            ],
          ),
          const SizedBox(height: MacMetrics.gapTight),
          if (formulaMode)
            FormulaComposer(
              key: ValueKey('composer-${widget.mappingId}'),
              mappingId: widget.mappingId,
              source: m.text,
              projection: projection,
              composer: widget.composer,
              concepts: widget.concepts,
              dispatch: widget.dispatch,
              outOfSync: !inSync,
              onEditAsText: () => widget.dispatch(const FormulaModeChanged(false)),
            )
          else
            Listener(
              onPointerHover: _onHover,
              child: MouseRegion(
                onExit: (_) => _endHover(),
                child: MacTextField(
                  key: const ValueKey('definition-field'),
                  controller: _controller,
                  focusNode: _focus,
                  maxLines: 4,
                  monospace: true,
                  hint: m.hint,
                  onChanged: _onChanged,
                ),
              ),
            ),
          if (!formulaMode &&
              completion != null &&
              (completion.items.isNotEmpty || completion.pending))
            CompletionPopup(
              completion: completion,
              onPick: (i) {
                widget.dispatch(CompletionMoved(i - completion.selected));
                _accept();
              },
            ),
          if (!formulaMode && card != null && card.found) HoverCard(card: card),
          const SizedBox(height: MacMetrics.gapTight),
          if (m.conflict)
            _ConflictNotice(
              onReload: () => widget.dispatch(DefinitionDraftReloaded(widget.mappingId)),
              onKeep: () => widget.dispatch(DefinitionDraftKept(widget.mappingId)),
            )
          else
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: MacMetrics.gap,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 5),
                  child: Icon(
                    m.tone == VerdictTone.none ? Icons.circle_outlined : Icons.circle,
                    size: 7,
                    color: toneColor,
                  ),
                ),
                Expanded(
                  child: Text(
                    m.statusText,
                    key: const ValueKey('definition-status'),
                    style: TextStyle(
                      fontSize: MacType.secondary,
                      color: m.tone == VerdictTone.none ? t.textSecondary : toneColor,
                    ),
                  ),
                ),
              ],
            ),
          if (m.diagnostics.isNotEmpty) ...[
            const SizedBox(height: MacMetrics.gap),
            // The status line already says the first message; its row adds
            // the excerpt, explanation and fixes without repeating it.
            for (final d in m.diagnostics)
              DiagnosticCard(diagnostic: d, source: m.text, showMessage: d.message != m.statusText),
          ],
          if ((m.dirty || m.committed == null) && !m.conflict) ...[
            const SizedBox(height: MacMetrics.gap),
            // Primary action right, its alternative to its left (macOS);
            // wraps rather than clips when the inspector is narrow.
            Wrap(
              alignment: WrapAlignment.end,
              spacing: MacMetrics.gap,
              runSpacing: MacMetrics.gap,
              children: [
                if (m.canRevert) MacButton(label: context.l10n.revert, onPressed: _revert),
                MacButton.primary(
                  label: m.commitLabel,
                  onPressed: m.canCommit ? _commit : null,
                  tooltip: m.canCommit ? '⌘↩' : null,
                ),
              ],
            ),
          ],
          const SizedBox(height: MacMetrics.gap),
          if (m.committed == null)
            Text(
              context.l10n.addingADefinitionIsARefinementNothing,
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
            )
          else ...[
            Align(
              alignment: Alignment.centerLeft,
              child: MacButton(
                label: context.l10n.detachDefinition,
                onPressed: m.canDetach
                    ? () => widget.dispatch(DetachDefinitionRequested(widget.mappingId))
                    : null,
              ),
            ),
            const SizedBox(height: MacMetrics.gapTight),
            Text(
              context.l10n.replacingOrDetachingIsAnEditThis,
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
            ),
          ],
        ],
      ),
    );
  }
}

Color _severityColor(MacTokens t, pb.DiagnosticSeverity s) => switch (s) {
  pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR => t.error,
  pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_WARNING => t.open,
  _ => t.open,
};

/// The committed definition changed while the draft was dirty.  Neither is
/// overwritten until the designer says which one stands.
class _ConflictNotice extends StatelessWidget {
  const _ConflictNotice({required this.onReload, required this.onKeep});
  final VoidCallback onReload;
  final VoidCallback onKeep;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: MacMetrics.gap,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: MacMetrics.gap,
          children: [
            Padding(
              padding: const EdgeInsets.only(top: 5),
              child: Icon(Icons.circle, size: 7, color: t.open),
            ),
            Expanded(
              child: Text(
                context.l10n.thisRelationshipSDefinitionChangedWhileYou,
                key: const ValueKey('definition-status'),
                style: TextStyle(fontSize: MacType.secondary, color: t.open),
              ),
            ),
          ],
        ),
        Wrap(
          spacing: MacMetrics.gap,
          runSpacing: MacMetrics.gap,
          children: [
            MacButton(
              label: context.l10n.reload,
              onPressed: onReload,
              tooltip: context.l10n.showTheDefinitionCommittedMeanwhileDropWhat,
            ),
            MacButton(
              label: context.l10n.keepMine,
              onPressed: onKeep,
              tooltip: context.l10n.keepWhatYouTypedSaveWillReplace,
            ),
          ],
        ),
      ],
    );
  }
}

/// The service's candidates, in the service's order.  Rows: the label
/// (monospace, what will be inserted), its kind, the resulting type in a
/// secondary column.  Selection is the accent tint; the selected row also
/// carries a ▸ so it survives without colour.
