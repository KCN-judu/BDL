/// The formula sheet: the definition editor of one relationship, over the
/// design, with room — the field at display size beside the name it
/// defines, the verdict and findings under it, and, in a column of its
/// own, what the selected position expects and what fits
/// (docs/architecture/studio-ui.md §4b, *The formula sheet*).
///
/// It is a view of the same draft the inspector edits (`EditorState
/// .formulaSheet` names the relationship; nothing is held here): the
/// same `DefinitionEditor` in its sheet layout, the same composer, the
/// same actions.  Esc closes it with the draft kept; ⌘↩ saves and closes.
library;

import 'package:flutter/material.dart';

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/composer.dart' show composerProjection;
import '../app/state.dart';
import 'definition_editor.dart';
import 'mac/widgets.dart';
import 'mac/tokens.dart';

class FormulaSheet extends StatelessWidget {
  const FormulaSheet({
    super.key,
    required this.state,
    required this.mappingId,
    required this.dispatch,
  });
  final AppState state;
  final int mappingId;
  final void Function(AppAction) dispatch;

  /// Two columns: the equation's, and the palette's (`_SheetBody
  /// .paletteWidth`), with the sheet's insets and one section gap.
  static const double width = 920;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final project = state.project;
    final mapping = state.mapping(mappingId);
    if (project == null || mapping == null) return const SizedBox.shrink();
    final id = mappingId;
    final concepts = {for (final c in project.concepts) c.id.toInt(): c};
    final produces = concepts[mapping.signature.output.toInt()];
    final projection = composerProjection(state, id);
    final result = projection != null && projection.hasResult()
        ? projection.result.description
        : produces?.name;
    final inputs = [for (final i in mapping.signature.inputs) ?concepts[i.toInt()]?.name];
    void close() => dispatch(const FormulaSheetDismissed());
    return Positioned.fill(
      child: GestureDetector(
        // a click on the backdrop closes the sheet, the draft kept
        behavior: HitTestBehavior.opaque,
        onTap: close,
        child: Container(
          color: Colors.black.withValues(alpha: 0.25),
          alignment: Alignment.center,
          child: GestureDetector(
            onTap: () {},
            child: Container(
              key: const ValueKey('formula-sheet'),
              width: width,
              constraints: const BoxConstraints(maxHeight: 640),
              decoration: BoxDecoration(
                color: t.window,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: t.hairline),
              ),
              padding: const EdgeInsets.fromLTRB(20, 18, 20, 16),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              context.l10n.defineRelationship(mapping.name),
                              key: const ValueKey('formula-sheet-title'),
                              style: Theme.of(context).textTheme.titleMedium,
                            ),
                            const SizedBox(height: 4),
                            Text(
                              result == null
                                  ? context.l10n.formulaSheetSubtitle
                                  : '${context.l10n.producesDescription(result)} · '
                                        '${context.l10n.formulaSheetSubtitle}',
                              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
                            ),
                          ],
                        ),
                      ),
                      // Formula | Text: the editor's preference, at the
                      // sheet's top right where a view switch lives
                      _ModeSwitch(composer: state.editor.composer, dispatch: dispatch),
                    ],
                  ),
                  const SizedBox(height: MacMetrics.gapGroup),
                  Flexible(
                    child: DefinitionEditor(
                      key: ValueKey('sheet-editor-$id'),
                      layout: DefinitionEditorLayout.sheet,
                      mappingId: id,
                      name: mapping.name,
                      produces: produces,
                      committed: mapping.hasDefinition() ? mapping.definition.formula : null,
                      draft: state.draft(id),
                      committedAnalysis: state.mappingAnalysis(id),
                      inputNames: inputs,
                      completion: state.editor.completion?.mappingId == id
                          ? state.editor.completion
                          : null,
                      hover: state.editor.hover?.mappingId == id ? state.editor.hover : null,
                      highlight: state
                          .editor
                          .highlights[HighlightState.formulaKey(id, state.editor.componentScope)],
                      component: state.editor.componentScope,
                      composer: state.editor.composer,
                      projection: projection,
                      concepts: concepts,
                      focusGeneration: state.editor.definitionFocus,
                      onDone: close,
                      dispatch: dispatch,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _ModeSwitch extends StatelessWidget {
  const _ModeSwitch({required this.composer, required this.dispatch});
  final ComposerState composer;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: 132,
    child: MacSegmented<bool>(
      key: const ValueKey('sheet-mode'),
      value: composer.formulaMode,
      options: {true: context.l10n.formula, false: context.l10n.textMode},
      onChanged: (v) => dispatch(FormulaModeChanged(v)),
    ),
  );
}
