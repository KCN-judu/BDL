/// The Formula Composer's transitions: mode, selection, slot queries and
/// structured actions over the one definition draft.
///
/// Studio owns which projection is on screen (Formula or Text), which node
/// is selected and whether a pop-up is open.  The compiler owns the tree
/// (`FormulaProjection`, delivered with every draft verdict), every
/// expected type, every candidate (`GetFormulaSlot`) and the text a
/// structured action makes (`ComposeFormula`).  A composed answer is put
/// into the draft through the ordinary `DefinitionDraftChanged` path —
/// there is no second commit path and no second formula store.
library;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'drafts.dart' show draftChanged;
import 'effects.dart';
import 'reducer.dart' show Transition;
import 'state.dart';

/// The source the Composer is looking at: the draft's, else the committed.
String composerSource(AppState s, int mappingId) =>
    s.draft(mappingId)?.source ?? s.committedDefinition(mappingId) ?? '';

/// The projection on screen: the draft's own, else the committed one
/// fetched for this mapping.  `null` while none has arrived.
pb.FormulaProjection? composerProjection(AppState s, int mappingId) {
  final d = s.draft(mappingId);
  if (d != null) return d.projection;
  final c = s.editor.composer;
  return c.mappingId == mappingId ? c.projection : null;
}

/// The stale-projection policy: a projection is *current* only when it is
/// of exactly the text on screen and that text parsed.  Anything else —
/// no answer yet, an older answer, text that cannot be read — is shown
/// dimmed as context and never edited structurally: a node id of one text
/// means nothing in another.  An empty draft is one slot and needs no
/// projection.
bool composerInSync(AppState s, int mappingId) {
  final source = composerSource(s, mappingId);
  if (source.trim().isEmpty) return true;
  final p = composerProjection(s, mappingId);
  return p != null && p.source == source && p.parseOk;
}

Transition formulaModeChanged(AppState s, bool formulaMode) {
  final c = s.editor.composer;
  if (c.formulaMode == formulaMode) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        composer: c.copyWith(formulaMode: formulaMode, clearSelection: true),
      ),
    ),
  );
}

/// The committed definition's projection, when there is no draft to carry
/// one.  Skipped while a draft exists (its verdict carries the projection).
Transition formulaProjectionRequested(AppState s, int mappingId) {
  if (s.project == null || s.mapping(mappingId) == null) return Transition(s);
  if (s.draft(mappingId) != null) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  final c = s.editor.composer;
  final keep = c.mappingId == mappingId
      ? c
      : const ComposerState().copyWith(formulaMode: c.formulaMode);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        composer: keep.copyWith(mappingId: mappingId, projectionGeneration: generation),
        toolingGeneration: generation,
      ),
    ),
    [
      GetFormulaProjection(
        revision: s.revision,
        mappingId: mappingId,
        generation: generation,
        component: s.editor.componentScope,
      ),
    ],
  );
}

Transition formulaProjectionReceived(AppState s, int generation, pb.FormulaProjectionResponse r) {
  final c = s.editor.composer;
  if (c.projectionGeneration != generation ||
      c.mappingId != r.mappingId.toInt() ||
      r.revision.toInt() != s.revision) {
    return Transition(s);
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        composer: c.copyWith(projection: r.hasProjection() ? r.projection : null),
      ),
    ),
  );
}

/// Select a node: the service is asked what the position expects and what
/// fits.  Selecting the node already selected re-asks (the draft may have
/// moved); `null` clears.
Transition formulaNodeSelected(AppState s, int mappingId, String? nodeId) {
  final c = s.editor.composer;
  if (nodeId == null) {
    return Transition(
      s.copyWith(editor: s.editor.copyWith(composer: c.copyWith(clearSelection: true))),
    );
  }
  if (s.project == null || s.mapping(mappingId) == null) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  final base = c.mappingId == mappingId ? c : c.copyWith(clearProjection: true);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        composer: base.copyWith(
          mappingId: mappingId,
          selectedNode: nodeId,
          clearSlot: true,
          slotGeneration: generation,
        ),
        toolingGeneration: generation,
      ),
    ),
    [
      GetFormulaSlot(
        revision: s.revision,
        mappingId: mappingId,
        source: composerSource(s, mappingId),
        nodeId: nodeId,
        generation: generation,
        component: s.editor.componentScope,
      ),
    ],
  );
}

Transition formulaSlotReceived(AppState s, int generation, pb.FormulaSlotResponse r) {
  final c = s.editor.composer;
  if (c.slotGeneration != generation ||
      c.mappingId != r.mappingId.toInt() ||
      c.selectedNode != r.nodeId ||
      r.revision.toInt() != s.revision) {
    return Transition(s);
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(composer: c.copyWith(slot: r)),
    ),
  );
}

/// A structured action: sent with the source it acts on; the answer
/// becomes the draft's text through the ordinary change path.  Refused
/// (nothing sent) while the projection on screen is not current — a node
/// id from stale structure never edits the current text — while another
/// action is in flight, and while the draft is in conflict or saving.
Transition composeRequested(AppState s, int mappingId, pb.ComposeAction action) {
  if (s.project == null || s.mapping(mappingId) == null) return Transition(s);
  final d = s.draft(mappingId);
  if (d != null && (d.conflict || d.pendingCommit != null)) return Transition(s);
  if (!composerInSync(s, mappingId)) return Transition(s);
  final c = s.editor.composer;
  if (c.pendingCompose) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  final source = composerSource(s, mappingId);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        composer: c.copyWith(
          mappingId: mappingId,
          pendingCompose: true,
          composeGeneration: generation,
          composeSource: source,
        ),
        toolingGeneration: generation,
      ),
    ),
    [
      ComposeFormula(
        revision: s.revision,
        mappingId: mappingId,
        source: source,
        action: action,
        generation: generation,
        component: s.editor.componentScope,
      ),
    ],
  );
}

Transition composeReceived(AppState s, int generation, pb.ComposeFormulaResponse r) {
  final c = s.editor.composer;
  final mappingId = r.mappingId.toInt();
  // the answer is applied only to the text it was computed against, for
  // the request still awaited, at the revision held: anything else is an
  // answer to a formula that no longer exists and is discarded
  if (c.mappingId != mappingId ||
      c.composeGeneration != generation ||
      r.revision.toInt() != s.revision ||
      c.composeSource != composerSource(s, mappingId)) {
    return Transition(
      s.copyWith(
        editor: s.editor.copyWith(
          composer: c.composeGeneration == generation ? c.copyWith(clearCompose: true) : c,
        ),
      ),
    );
  }
  // the answer is a draft change like any typing; then select what the
  // service says comes next (the new slot, else the edited node)
  final t = draftChanged(s, mappingId, r.source);
  final next = r.select.isEmpty ? null : r.select;
  final settled = t.state.editor.composer.copyWith(clearCompose: true, clearSelection: true);
  final withSelection = Transition(
    t.state.copyWith(editor: t.state.editor.copyWith(composer: settled)),
    t.effects,
  );
  if (next == null) return withSelection;
  final sel = formulaNodeSelected(withSelection.state, mappingId, next);
  return Transition(sel.state, [...withSelection.effects, ...sel.effects]);
}

/// A tooling failure for one of the Composer's requests: the pop-up just
/// does not fill; a pending action is released.
EditorState composerAfterFailure(EditorState e, int generation) {
  final c = e.composer;
  var next = c;
  if (c.slotGeneration == generation) next = next.copyWith(clearSlot: true);
  if (c.projectionGeneration == generation) next = next.copyWith(clearProjection: true);
  if (c.composeGeneration == generation) next = next.copyWith(clearCompose: true);
  return identical(next, c) ? e : e.copyWith(composer: next);
}

/// A new selection or projection: the Composer's selection is about the
/// old text.
EditorState withoutComposerSelection(EditorState e) => e.composer.selectedNode == null
    ? e
    : e.copyWith(composer: e.composer.copyWith(clearSelection: true));
