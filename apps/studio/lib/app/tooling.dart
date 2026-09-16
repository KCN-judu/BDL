/// Semantic tooling transitions: completion pop-up and hover cards.
///
/// Studio owns the pop-up (which row is selected, whether it is open) and
/// the hover's position; the IDE service owns the candidates and the
/// card's content.  Every request carries a generation and only the latest
/// answer is applied, so a slow answer to an old keystroke can never
/// overwrite a newer one.  No name, unit or keyword table exists here.
library;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition, sendEdit;
import 'state.dart';

Transition completionRequested(AppState s, int mappingId, String source, int offset) {
  if (s.project == null || s.mapping(mappingId) == null) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  final previous = s.editor.completion;
  // Re-requesting for the same field keeps the rows on screen (no flicker)
  // until the new candidates arrive; a new field starts empty.
  final completion = previous != null && previous.mappingId == mappingId
      ? previous.copyWith(generation: generation, source: source, offset: offset, pending: true)
      : CompletionState(
          mappingId: mappingId,
          generation: generation,
          source: source,
          offset: offset,
        );
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(completion: completion, toolingGeneration: generation),
    ),
    [
      CompleteDraft(
        revision: s.revision,
        mappingId: mappingId,
        source: source,
        offset: offset,
        generation: generation,
        component: s.editor.componentScope,
      ),
    ],
  );
}

Transition completionReceived(AppState s, int generation, pb.DraftCompletionResponse r) {
  final c = s.editor.completion;
  if (c == null ||
      generation != c.generation ||
      r.mappingId.toInt() != c.mappingId ||
      r.revision.toInt() != s.revision) {
    return Transition(s);
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        completion: c.copyWith(
          items: r.items,
          pending: false,
          selected: r.items.isEmpty ? 0 : c.selected.clamp(0, r.items.length - 1),
        ),
      ),
    ),
  );
}

Transition completionMoved(AppState s, int delta) {
  final c = s.editor.completion;
  if (c == null || c.items.isEmpty) return Transition(s);
  final n = c.items.length;
  final next = ((c.selected + delta) % n + n) % n;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(completion: c.copyWith(selected: next)),
    ),
  );
}

Transition completionDismissed(AppState s) =>
    Transition(s.copyWith(editor: s.editor.copyWith(clearCompletion: true)));

Transition formulaHoverRequested(AppState s, int mappingId, String source, int? offset) {
  if (offset == null) return Transition(s.copyWith(editor: s.editor.copyWith(clearHover: true)));
  if (s.project == null || s.mapping(mappingId) == null) return Transition(s);
  final h = s.editor.hover;
  if (h != null && h.mappingId == mappingId && h.offset == offset && h.entity == null) {
    return Transition(s);
  }
  final generation = s.editor.toolingGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        hover: HoverState(generation: generation, mappingId: mappingId, offset: offset),
        toolingGeneration: generation,
      ),
    ),
    [
      HoverDraft(
        revision: s.revision,
        mappingId: mappingId,
        source: source,
        offset: offset,
        generation: generation,
        component: s.editor.componentScope,
      ),
    ],
  );
}

Transition entityHoverRequested(AppState s, pb.EntityRef? entity) {
  if (entity == null) return Transition(s.copyWith(editor: s.editor.copyWith(clearHover: true)));
  // The IDE service explains flat entities; a component body's entities
  // are reached through their formulas only.
  if (s.project == null || s.editor.context is! SystemContext) return Transition(s);
  final h = s.editor.hover;
  if (h != null && h.entity == entity) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        hover: HoverState(generation: generation, entity: entity),
        toolingGeneration: generation,
      ),
    ),
    [HoverEntity(revision: s.revision, entity: entity, generation: generation)],
  );
}

Transition hoverReceived(AppState s, int generation, pb.DraftHoverResponse r) {
  final h = s.editor.hover;
  if (h == null || generation != h.generation || r.revision.toInt() != s.revision) {
    return Transition(s);
  }
  // Nothing under the pointer: no card (the hover state stays so the same
  // spot is not asked again).
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(hover: h.copyWith(card: r)),
    ),
  );
}

Transition toolingFailed(AppState s, int generation) {
  final e = s.editor;
  var next = e;
  if (e.completion?.generation == generation) next = next.copyWith(clearCompletion: true);
  if (e.hover?.generation == generation) next = next.copyWith(clearHover: true);
  return Transition(identical(next, e) ? s : s.copyWith(editor: next));
}

/// Tooling is about the current text of the current revision: a new
/// projection or a change of selection drops whatever is on screen.
EditorState withoutTooling(EditorState e) => e.completion == null && e.hover == null
    ? e
    : e.copyWith(clearCompletion: true, clearHover: true);

// ---- semantic actions --------------------------------------------------------

Transition semanticActionsRequested(AppState s, pb.EntityRef entity) {
  if (s.project == null) return Transition(s);
  final a = s.editor.actions;
  if (a != null && a.entity == entity && a.revision == s.revision) return Transition(s);
  final generation = s.editor.toolingGeneration + 1;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        actions: SemanticActionsState(entity: entity, revision: s.revision, generation: generation),
        toolingGeneration: generation,
      ),
    ),
    [ListSemanticActions(revision: s.revision, entity: entity, generation: generation)],
  );
}

Transition semanticActionsReceived(AppState s, int generation, pb.SemanticActionsResponse r) {
  final a = s.editor.actions;
  if (a == null || generation != a.generation || r.revision.toInt() != s.revision) {
    return Transition(s);
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(actions: a.copyWith(actions: r.actions, pending: false)),
    ),
  );
}

/// Apply an action: the first model edit now, the rest queued for the
/// confirming revisions.  A blocked action, or a choice not made, applies
/// nothing.
Transition semanticActionApplied(AppState s, String actionId, int? option) {
  final a = s.editor.actions;
  final action = a?.actions.where((x) => x.id == actionId).firstOrNull;
  if (a == null || action == null || a.revision != s.revision) return Transition(s);
  final List<pb.EditOp> edits = switch (action.applicability) {
    pb.ActionApplicability.ACTION_APPLICABILITY_READY => action.edits,
    pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE =>
      option != null && option >= 0 && option < action.options.length
          ? [action.options[option].edit]
          : const [],
    _ => const [],
  };
  if (edits.isEmpty) return Transition(s);
  final t = sendEdit(s, edits.first);
  if (t.effects.isEmpty) return t;
  return Transition(
    t.state.copyWith(
      editor: t.state.editor.copyWith(queuedEdits: edits.sublist(1), clearActions: true),
    ),
    t.effects,
  );
}
