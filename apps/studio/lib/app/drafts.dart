/// Definition-draft transitions: the pure half of the formula editor.
///
/// A draft is Studio's candidate text for one mapping's definition.  The
/// compiler (through `bdld`) is the only judge of what it means; Studio
/// owns the text, its generation counter, its base revision and whether the
/// designer has been told that the committed definition moved underneath.
/// Nothing here mutates the project: the only edits are the one commit and
/// the one detach, and both are ordinary revisioned `EditOp`s.
library;

import 'package:fixnum/fixnum.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition, sendEdit;
import 'state.dart';

/// The designer typed.  The draft exists while the text differs from the
/// committed definition; typing the committed text back dissolves it (and
/// its overlay).
Transition draftChanged(AppState s, int id, String source) {
  if (s.mapping(id) == null) return Transition(s);
  final committed = s.committedDefinition(id);
  final existing = s.draft(id);
  if (source == (committed ?? '')) return draftDropped(s, id);
  final generation = (existing?.generation ?? 0) + 1;
  final base =
      existing ??
      DefinitionDraft(
        mappingId: id,
        baseRevision: s.revision,
        baseDefinition: committed,
        source: source,
      );
  final draft = base.copyWith(
    source: source,
    generation: generation,
    check: source.trim().isEmpty ? DraftCheck.checked : DraftCheck.checking,
    clearAnalysis: true,
    clearCheckError: true,
    clearCommitError: true,
  );
  return Transition(
    _withDraft(s, draft),
    _checkEffects(s.revision, draft, s.editor.componentScope),
  );
}

/// Drop the draft on purpose (revert, reload); the daemon's overlay is
/// discarded with it so the two never disagree about what is being judged.
Transition draftDropped(AppState s, int id) => s.draft(id) == null
    ? Transition(s)
    : Transition(_withoutDraft(s, id), [DiscardDraft(id, component: s.editor.componentScope)]);

/// After a conflict, keep the draft: it is now based on what is committed.
Transition draftKept(AppState s, int id) {
  final d = s.draft(id);
  if (d == null) return Transition(s);
  final committed = s.committedDefinition(id);
  return Transition(
    _withDraft(
      s,
      d.copyWith(
        conflict: false,
        baseDefinition: committed,
        clearBaseDefinition: committed == null,
      ),
    ),
  );
}

/// Commit the draft as exactly one edit, chosen from the committed state:
/// attach when there is no definition, replace when there is.  The draft
/// stays until the projection confirms the new definition.
Transition commitDefinition(AppState s, int id) {
  final d = s.draft(id);
  final m = s.mapping(id);
  if (d == null || m == null || d.conflict || d.pendingCommit != null) return Transition(s);
  final source = d.source.trim();
  if (source.isEmpty) return Transition(s);
  if (source == s.committedDefinition(id)) return Transition(_withoutDraft(s, id));
  final definition = pb.Definition(formula: source);
  final op = m.hasDefinition()
      ? pb.EditOp(
          replaceDefinition: pb.ReplaceDefinition(id: Int64(id), definition: definition),
        )
      : pb.EditOp(
          attachDefinition: pb.AttachDefinition(id: Int64(id), definition: definition),
        );
  final t = sendEdit(s, op);
  if (t.effects.isEmpty) return t;
  return Transition(
    _withDraft(t.state, d.copyWith(pendingCommit: source, clearCommitError: true)),
    t.effects,
  );
}

/// Detach the committed definition.  An explicit act on the definition, so
/// any draft for the mapping goes with it.
Transition detachDefinition(AppState s, int id) {
  final m = s.mapping(id);
  if (m == null || !m.hasDefinition()) return Transition(s);
  final t = sendEdit(s, pb.EditOp(replaceDefinition: pb.ReplaceDefinition(id: Int64(id))));
  if (t.effects.isEmpty) return t;
  return Transition(_withoutDraft(t.state, id), [
    if (s.draft(id) != null) DiscardDraft(id, component: s.editor.componentScope),
    ...t.effects,
  ]);
}

/// A verdict is kept only for the draft's latest generation at the held
/// revision; anything else is a response to text that no longer exists.
Transition draftAnalysisReceived(AppState s, pb.DefinitionDraftAnalysis r) {
  final id = r.mappingId.toInt();
  final d = s.draft(id);
  if (d == null ||
      r.generation.toInt() != d.generation ||
      r.revision.toInt() != s.revision ||
      r.revision.toInt() != d.baseRevision) {
    return Transition(s);
  }
  if (d.source.trim().isEmpty) {
    return Transition(
      _withDraft(s, d.copyWith(check: DraftCheck.checked, clearAnalysis: true, parseOk: true)),
    );
  }
  return Transition(
    _withDraft(
      s,
      d.copyWith(
        check: DraftCheck.checked,
        analysis: r.analysis,
        parseOk: r.parseOk,
        clearCheckError: true,
        projection: r.hasProjection() ? r.projection : null,
      ),
    ),
  );
}

Transition draftAnalysisFailed(AppState s, int id, int generation, String code, String message) {
  final d = s.draft(id);
  if (d == null || generation != d.generation) return Transition(s);
  // The project moved on; the projection that follows re-asks.
  if (code == 'draft.stale_revision') return Transition(s);
  return Transition(_withDraft(s, d.copyWith(check: DraftCheck.unavailable, checkError: message)));
}

/// A commit was answered with an error: the draft is kept, the reason
/// travels with it.  (Failures are not attributed per request by the
/// protocol; a draft awaiting confirmation takes the message.)
Map<int, DefinitionDraft> draftsAfterFailedRequest(
  Map<int, DefinitionDraft> drafts,
  String message,
) => {
  for (final e in drafts.entries)
    e.key: e.value.pendingCommit == null
        ? e.value
        : e.value.copyWith(clearPendingCommit: true, commitError: message),
};

/// Rebase drafts on a projection that supersedes the one they were based
/// on.  Used for every new revision of the open project and for restoring
/// stashed drafts when a project is reopened.
///
/// Per draft: the mapping gone → dropped; our own commit confirmed → done
/// (or rebased if the designer kept typing); the committed definition
/// changed under a dirty draft → flagged as a conflict, nothing
/// overwritten; otherwise rebased.  Every surviving non-empty draft is
/// checked again, because any change (a signature, a representation, even
/// an input's name) can change what the same text means.
({Map<int, DefinitionDraft> drafts, List<Effect> effects}) rebaseDrafts(
  Map<int, DefinitionDraft> drafts,
  pb.ProjectProjection incoming, {
  int? component,
}) {
  final revision = incoming.revision.toInt();
  final next = <int, DefinitionDraft>{};
  final effects = <Effect>[];
  for (final d in drafts.values) {
    final m = incoming.mappings.where((m) => m.id.toInt() == d.mappingId).firstOrNull;
    if (m == null) continue;
    final committed = m.hasDefinition() ? m.definition.formula : null;
    var draft = d;
    if (d.pendingCommit != null && committed == d.pendingCommit) {
      if (d.source == committed) continue;
      draft = d.copyWith(
        baseDefinition: committed,
        clearBaseDefinition: committed == null,
        clearPendingCommit: true,
        conflict: false,
      );
    } else if (committed != d.baseDefinition) {
      if (d.source == (committed ?? '')) continue;
      if (!d.dirtyAgainst(d.baseDefinition)) continue;
      draft = d.copyWith(conflict: true);
    } else {
      draft = d.copyWith(conflict: false);
    }
    final empty = draft.source.trim().isEmpty;
    draft = draft.copyWith(
      baseRevision: revision,
      generation: draft.generation + 1,
      check: empty ? DraftCheck.checked : DraftCheck.checking,
      clearAnalysis: true,
      clearCheckError: true,
    );
    next[draft.mappingId] = draft;
    effects.addAll(_checkEffects(revision, draft, component));
  }
  return (drafts: next, effects: effects);
}

/// The drafts worth keeping across a close: those whose text differs from
/// what is committed.
Map<int, DefinitionDraft> dirtyDrafts(AppState s) => {
  for (final e in s.editor.drafts.entries)
    if (e.value.dirtyAgainst(s.committedDefinition(e.key))) e.key: e.value,
};

/// Every draft reaches the project — an empty one too, since the project
/// saves it — and comes back with a verdict; an empty draft's verdict is
/// not kept (an empty field has nothing to judge).
List<Effect> _checkEffects(int revision, DefinitionDraft d, int? component) => [
  AnalyzeDraft(
    revision: revision,
    mappingId: d.mappingId,
    generation: d.generation,
    source: d.source,
    component: component,
  ),
];

AppState _withDraft(AppState s, DefinitionDraft d) =>
    s.copyWith(editor: s.editor.copyWith(drafts: {...s.editor.drafts, d.mappingId: d}));

AppState _withoutDraft(AppState s, int id) {
  if (!s.editor.drafts.containsKey(id)) return s;
  return s.copyWith(editor: s.editor.copyWith(drafts: {...s.editor.drafts}..remove(id)));
}
