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
/// Keyboard (documented in docs/STUDIO_UI.md §4a): ⌘↩ saves the definition
/// while it is dirty; ⌘S keeps its meaning (*Save project*) and never
/// commits a draft; Esc reverts a dirty draft.  Ordinary text-editing
/// shortcuts are untouched; Return inserts a line.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
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
  }) {
    final text = draft?.source ?? committed ?? '';
    final dirty = draft != null && draft.dirtyAgainst(committed);
    final conflict = draft?.conflict ?? false;
    final saving = draft?.pendingCommit != null;
    final hint = inputNames.isEmpty
        ? 'expression with no inputs'
        : 'expression over ${inputNames.join(', ')}';

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
        statusText =
            'No definition yet. A legal state: other relationships may already depend '
            'on the signature.';
        tone = VerdictTone.none;
      } else if (committedAnalysis == null) {
        diagnostics = const [];
        statusText = 'Checking…';
        tone = VerdictTone.checking;
      } else {
        diagnostics = committedAnalysis.diagnostics.where((d) => d.hasSpan()).toList();
        (statusText, tone) = _summarise(committedAnalysis, parseOk: true);
      }
    } else if (draft.source.trim().isEmpty) {
      diagnostics = const [];
      statusText = committed == null
          ? 'Nothing to add yet.'
          : 'Empty. Detach to remove the definition.';
      tone = VerdictTone.none;
    } else if (saving) {
      diagnostics = draft.analysis?.diagnostics ?? const [];
      statusText = 'Saving…';
      tone = VerdictTone.checking;
    } else if (draft.commitError case final e?) {
      diagnostics = draft.analysis?.diagnostics ?? const [];
      statusText = 'Not saved: $e';
      tone = VerdictTone.error;
    } else {
      switch (draft.check) {
        case DraftCheck.checking:
          diagnostics = const [];
          statusText = 'Checking…';
          tone = VerdictTone.checking;
        case DraftCheck.unavailable:
          diagnostics = const [];
          statusText = 'Not checked: ${draft.checkError ?? 'the compiler service is unavailable'}';
          tone = VerdictTone.open;
        case DraftCheck.checked:
          final a = draft.analysis;
          diagnostics = a?.diagnostics ?? const [];
          (statusText, tone) = a == null
              ? ('Checking…', VerdictTone.checking)
              : _summarise(a, parseOk: draft.parseOk);
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
      commitLabel: committed == null ? 'Add definition' : 'Save definition',
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

  /// One line for the ladder verdict.  Open is not an error: it says what is
  /// still to be decided.  Invalid says the first thing that is wrong.
  static (String, VerdictTone) _summarise(pb.MappingAnalysis a, {required bool parseOk}) {
    final errors = a.diagnostics.where(
      (d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
    );
    final others = a.diagnostics.where(
      (d) => d.severity != pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
    );
    return switch (a.status) {
      pb.MappingStatus.MAPPING_STATUS_INVALID => (
        errors.firstOrNull?.message ?? (parseOk ? 'Invalid definition.' : 'Cannot be read.'),
        VerdictTone.error,
      ),
      pb.MappingStatus.MAPPING_STATUS_OPEN => (
        others.firstOrNull?.message ?? 'Open: something it needs is not decided yet.',
        VerdictTone.open,
      ),
      pb.MappingStatus.MAPPING_STATUS_TYPE_VALID ||
      pb.MappingStatus.MAPPING_STATUS_TEMPORALLY_VALID ||
      pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT => ('Valid definition', VerdictTone.settled),
      _ => ('No definition.', VerdictTone.none),
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
  });

  final int mappingId;

  /// The committed formula, `null` when the mapping has none.
  final String? committed;
  final DefinitionDraft? draft;
  final pb.MappingAnalysis? committedAnalysis;
  final List<String> inputNames;
  final void Function(AppAction) dispatch;

  @override
  State<DefinitionEditor> createState() => _DefinitionEditorState();
}

class _DefinitionEditorState extends State<DefinitionEditor> {
  late final _MarkedController _controller = _MarkedController(text: _desiredText);
  final FocusNode _focus = FocusNode(debugLabel: 'definition');

  String get _desiredText => widget.draft?.source ?? widget.committed ?? '';

  @override
  void didUpdateWidget(DefinitionEditor old) {
    super.didUpdateWidget(old);
    _syncText();
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
  }

  @override
  void dispose() {
    _controller.dispose();
    _focus.dispose();
    super.dispose();
  }

  void _commit() => widget.dispatch(CommitDefinitionRequested(widget.mappingId));
  void _revert() => widget.dispatch(DefinitionDraftReverted(widget.mappingId));

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final m = DefinitionEditorModel(
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
    final toneColor = switch (m.tone) {
      VerdictTone.none || VerdictTone.checking => t.textTertiary,
      VerdictTone.settled => t.settled,
      VerdictTone.open => t.open,
      VerdictTone.error => t.error,
    };

    return CallbackShortcuts(
      bindings: {
        if (m.canCommit) const SingleActivator(LogicalKeyboardKey.enter, meta: true): _commit,
        if (m.canRevert) const SingleActivator(LogicalKeyboardKey.escape): _revert,
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          MacTextField(
            key: const ValueKey('definition-field'),
            controller: _controller,
            focusNode: _focus,
            maxLines: 4,
            monospace: true,
            hint: m.hint,
            onChanged: (v) =>
                widget.dispatch(DefinitionDraftChanged(mappingId: widget.mappingId, source: v)),
          ),
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
                      fontSize: 11,
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
                if (m.canRevert) MacButton(label: 'Revert', onPressed: _revert),
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
              'Adding a definition is a refinement: nothing established elsewhere is reopened.',
              style: TextStyle(fontSize: 11, color: t.textSecondary),
            )
          else ...[
            Align(
              alignment: Alignment.centerLeft,
              child: MacButton(
                label: 'Detach definition',
                onPressed: m.canDetach
                    ? () => widget.dispatch(DetachDefinitionRequested(widget.mappingId))
                    : null,
              ),
            ),
            const SizedBox(height: MacMetrics.gapTight),
            Text(
              'Replacing or detaching is an edit: this definition and the simulation are '
              're-checked.',
              style: TextStyle(fontSize: 11, color: t.textSecondary),
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
                'This relationship\'s definition changed while you were editing it.',
                key: const ValueKey('definition-status'),
                style: TextStyle(fontSize: 11, color: t.open),
              ),
            ),
          ],
        ),
        Wrap(
          spacing: MacMetrics.gap,
          runSpacing: MacMetrics.gap,
          children: [
            MacButton(
              label: 'Reload',
              onPressed: onReload,
              tooltip: 'Show the definition committed meanwhile; drop what you typed',
            ),
            MacButton(
              label: 'Keep mine',
              onPressed: onKeep,
              tooltip: 'Keep what you typed; save will replace the committed definition',
            ),
          ],
        ),
      ],
    );
  }
}

/// A controller that underlines byte-span diagnostics in place.  The marks
/// are recomputed from state on every build; the text itself is never
/// touched here.
class _MarkedController extends TextEditingController {
  _MarkedController({super.text});

  List<(TextRange, Color)> marks = const [];

  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    final text = this.text;
    final valid = marks.where((m) => !m.$1.isCollapsed && m.$1.end <= text.length).toList()
      ..sort((a, b) => a.$1.start.compareTo(b.$1.start));
    if (valid.isEmpty) {
      return super.buildTextSpan(context: context, style: style, withComposing: withComposing);
    }
    final children = <TextSpan>[];
    var at = 0;
    for (final (range, color) in valid) {
      final start = range.start.clamp(at, text.length);
      if (start > at) children.add(TextSpan(text: text.substring(at, start)));
      final end = range.end.clamp(start, text.length);
      if (end > start) {
        children.add(
          TextSpan(
            text: text.substring(start, end),
            style: TextStyle(
              decoration: TextDecoration.underline,
              decorationStyle: TextDecorationStyle.wavy,
              decorationColor: color,
              decorationThickness: 1.5,
            ),
          ),
        );
      }
      at = end;
    }
    if (at < text.length) children.add(TextSpan(text: text.substring(at)));
    return TextSpan(style: style, children: children);
  }
}
