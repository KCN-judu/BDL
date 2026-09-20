/// The project's lifecycle: save, and the one guard every unloading path
/// goes through.
///
/// Persistence answers "what was the designer working on?"; the project
/// (in `bdld`) owns that answer and says whether it differs from what is
/// saved.  Studio never keeps a second dirty flag: before it unloads a
/// project it sends whatever typing has not reached the project yet, asks
/// for the projection, and reads `dirty` off it.  Clean → unload at once.
/// Dirty → *Save changes to “name”?* with *Don't Save / Cancel / Save*.
/// Save unloads only after the save has succeeded; Don't Save unloads and
/// the next open returns to what was saved; Cancel leaves everything as it
/// was.  Close, the project manager, Open or New while a project is open,
/// ⌘W, ⌘Q, the menu's Quit and the window's close button all arrive here.
library;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'reducer.dart' show Transition, pending;
import 'state.dart';
import 'system.dart' show draftKey;

/// Save the whole current state.  Typed text the project has not received
/// yet goes first (one edit); the save follows its answer.
Transition saveRequested(AppState s, {required bool force}) {
  if (s.project == null) return Transition(s);
  final cleared = s.copyWith(editor: s.editor.copyWith(clearError: true));
  final sources = s.editor.sources;
  final unsent = sources.buffer != null && sources.openPath != null;
  if (unsent || sources.sent != null) {
    final effects = <Effect>[];
    var next = cleared.copyWith(editor: cleared.editor.copyWith(pendingSave: force));
    if (sources.sent == null) {
      effects.add(
        ApplySourceEdit(baseRevision: s.revision, path: sources.openPath!, text: sources.buffer!),
      );
      next = pending(
        next.copyWith(
          editor: next.editor.copyWith(sources: sources.copyWith(sent: sources.buffer)),
        ),
      );
    }
    return Transition(next, effects);
  }
  return Transition(pending(cleared), [SaveProject(force: force)]);
}

/// A source edit in flight has been answered: a save that waited for it
/// goes now; an unload that waited for it asks the project.
Transition afterSourceEditSettled(AppState s) {
  var state = s;
  final effects = <Effect>[];
  if (state.editor.pendingSave != null) {
    final force = state.editor.pendingSave!;
    state = pending(state.copyWith(editor: state.editor.copyWith(clearPendingSave: true)));
    effects.add(SaveProject(force: force));
  } else if (state.editor.unloading != null) {
    state = pending(state);
    effects.add(const GetProject());
  }
  return Transition(state, effects);
}

/// Unload the open project for [intent], or run the intent at once when
/// no project is open.  With a project open: flush, ask, then decide.
Transition unloadRequested(AppState s, UnloadIntent intent) {
  if (s.project == null) return Transition(s, _run(intent));
  if (s.editor.unloadInProgress) return Transition(s);
  final sources = s.editor.sources;
  final unsent = sources.buffer != null && sources.openPath != null;
  var next = s.copyWith(editor: s.editor.copyWith(unloading: intent, clearError: true));
  if (unsent || sources.sent != null) {
    if (sources.sent != null) return Transition(next);
    return Transition(
      pending(
        next.copyWith(
          editor: next.editor.copyWith(sources: sources.copyWith(sent: sources.buffer)),
        ),
      ),
      [ApplySourceEdit(baseRevision: s.revision, path: sources.openPath!, text: sources.buffer!)],
    );
  }
  return Transition(pending(next), const [GetProject()]);
}

/// The projection that answers an unload's question.  Returns null when
/// the projection is not that answer.
Transition? unloadDecided(AppState s, pb.ProjectProjection incoming, bool fromRequest) {
  final intent = s.editor.unloading;
  if (intent == null || !fromRequest) return null;
  final base = s.copyWith(editor: s.editor.copyWith(clearUnloading: true));
  if (incoming.dirty) {
    return Transition(base.copyWith(editor: base.editor.copyWith(closeGuard: intent)));
  }
  return _close(base, intent);
}

/// The sheet's answer.
Transition closeGuardAnswered(AppState s, CloseGuardAnswer answer) {
  final intent = s.editor.closeGuard;
  if (intent == null) return Transition(s);
  final base = s.copyWith(editor: s.editor.copyWith(clearCloseGuard: true));
  switch (answer) {
    case CloseGuardAnswer.cancel:
      return Transition(base);
    case CloseGuardAnswer.dontSave:
      return _close(base, intent);
    case CloseGuardAnswer.save:
      final t = saveRequested(
        base.copyWith(editor: base.editor.copyWith(closeAfterSave: intent)),
        force: false,
      );
      return t;
  }
}

/// A save answered with a clean projection while the guard waited on it:
/// the unload goes ahead.  Returns null when there is nothing to do.
Transition? closeAfterSaved(AppState s, pb.ProjectProjection incoming, bool fromRequest) {
  final intent = s.editor.closeAfterSave;
  if (intent == null || !fromRequest || incoming.dirty) return null;
  return _close(s.copyWith(editor: s.editor.copyWith(clearCloseAfterSave: true)), intent);
}

/// A request failed while an unload waited on it (a save refused, a flush
/// refused): the project stays open, the failure shows, nothing unloads.
AppState unloadAbandoned(AppState s) => s.copyWith(
  editor: s.editor.copyWith(
    clearUnloading: true,
    clearCloseAfterSave: true,
    clearPendingSave: true,
    clearAfterClose: true,
  ),
);

/// The project has closed: the intent that asked for it runs.
Transition afterClosed(AppState s) {
  final intent = s.editor.afterClose;
  if (intent == null) return Transition(s);
  final base = s.copyWith(editor: s.editor.copyWith(clearAfterClose: true));
  return Transition(countsAsRequest(intent) ? pending(base) : base, _run(intent));
}

Transition _close(AppState s, UnloadIntent intent) {
  final recent = rememberWorkspace(s);
  return Transition(
    pending(
      s.copyWith(
        recent: recent,
        editor: s.editor.copyWith(afterClose: intent),
      ),
    ),
    [SaveRecentProjects(recent), const CloseProject()],
  );
}

List<Effect> _run(UnloadIntent intent) => switch (intent) {
  CloseOnly() => const [],
  OpenAnother(:final rootPath) => [OpenProject(rootPath)],
  CreateAnother(:final rootPath, :final name, :final template) => [
    InitProject(rootPath: rootPath, name: name, template: template),
  ],
  PickAnother() => const [PickProjectToOpen()],
  PickNew(:final template) => [PickNewProjectLocation(template: template)],
  Quit() => const [QuitApplication()],
};

/// Whether an intent that ran after a close counts as a pending request.
bool countsAsRequest(UnloadIntent intent) => switch (intent) {
  OpenAnother() || CreateAnother() => true,
  _ => false,
};

/// Where the designer is in the open project, filed under the project in
/// the recent list (a per-user preference, never project data).
List<RecentProject> rememberWorkspace(AppState s) {
  final root = s.flat?.rootPath;
  if (root == null) return s.recent;
  final ws = ProjectWorkspace(
    view: s.editor.view.name,
    openSource: s.editor.sources.openPath,
    component: switch (s.editor.context) {
      SystemContext() => null,
      ComponentContext(:final id) => id,
    },
    page: s.editor.page.name,
  );
  return [for (final r in s.recent) r.path == root ? r.withWorkspace(ws) : r];
}

/// The workspace to restore for a project being opened, if one was kept.
ProjectWorkspace? workspaceFor(AppState s, String root) =>
    s.recent.where((r) => r.path == root).firstOrNull?.workspace;

/// Seed the editors with the definition drafts the project was saved
/// with, once per open: the current context's into the editors, every
/// other scope's into the stash it is restored from on a context switch.
({Map<int, DefinitionDraft> drafts, Map<String, Map<int, DefinitionDraft>> stashed}) seedDrafts(
  AppState s,
  pb.SystemView system,
) {
  final root = s.flat?.rootPath;
  if (root == null) return (drafts: s.editor.drafts, stashed: s.editor.stashedDrafts);
  final drafts = {...s.editor.drafts};
  final stashed = {...s.editor.stashedDrafts};
  for (final d in system.definitionDrafts) {
    final context = d.hasComponent()
        ? ComponentContext(d.component.toInt())
        : const SystemContext();
    final draft = DefinitionDraft(
      mappingId: d.mappingId.toInt(),
      baseRevision: s.revision,
      baseDefinition: null,
      source: d.source,
    );
    if (context == s.editor.context) {
      drafts.putIfAbsent(draft.mappingId, () => draft);
    } else {
      final key = draftKey(root, context);
      stashed[key] = {...?stashed[key], draft.mappingId: draft};
    }
  }
  return (drafts: drafts, stashed: stashed);
}
