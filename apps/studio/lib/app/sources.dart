/// The Code view's reducers (ADR-0023 §3–§5).
///
/// The sources are the daemon's: the files with every committed graph edit
/// written back, fetched whenever the Code view is on screen and the
/// revision moves.  Typing edits the editor's own buffer; after a pause
/// the whole file is sent as one `ApplySourceEdit`.  An accepted edit is a
/// new revision (the graph follows); a refused one keeps the last revision
/// that built and the draft exactly as typed, with why.  Nothing typed is
/// discarded by Studio; only the daemon says what the text means.
library;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'effects.dart';
import 'lifecycle.dart' show afterSourceEditSettled;
import 'actions.dart' show SelectionChanged;
import 'reducer.dart' show Transition, decPending, pending, projectReceived, reduce;
import 'state.dart';

/// Design, Code or Split.  Showing text fetches it when the sources are
/// older than the project.
Transition viewChanged(AppState s, DesignView view) {
  final next = s.copyWith(editor: s.editor.copyWith(view: view));
  final showsCode = view != DesignView.design;
  final stale = s.project != null && s.editor.sources.revision != s.revision;
  return Transition(next, [if (showsCode && stale) const GetSources()]);
}

/// The designer typed: the editor's text is its own until sent.
Transition sourceTextChanged(AppState s, String path, String text) {
  final sources = s.editor.sources;
  if (sources.openPath != path) return Transition(s);
  // Typing back to what the daemon holds is not an edit.
  final same = text == sources.open?.text;
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        sources: same ? sources.copyWith(clearBuffer: true) : sources.copyWith(buffer: text),
      ),
    ),
  );
}

/// Send the editor's text against the revision Studio holds.  Nothing to
/// send when it equals the daemon's text for that file.
Transition sourceEditRequested(AppState s, String path, String text) {
  final project = s.project;
  if (project == null) return Transition(s);
  final sources = s.editor.sources;
  if (text == sources.file(path)?.text) {
    return Transition(
      s.copyWith(editor: s.editor.copyWith(sources: sources.copyWith(clearBuffer: true))),
    );
  }
  if (sources.sent != null) {
    // one edit in flight at a time: the buffer goes with the answer
    return Transition(
      s.copyWith(
        editor: s.editor.copyWith(sources: sources.copyWith(buffer: text)),
      ),
    );
  }
  return Transition(
    pending(
      s.copyWith(
        editor: s.editor.copyWith(
          sources: sources.copyWith(buffer: text, sent: text),
        ),
      ),
    ),
    [ApplySourceEdit(baseRevision: s.revision, path: path, text: text)],
  );
}

/// GetSources answered.  The editor keeps unsent typing; otherwise it
/// shows the daemon's text.  A stale edit waiting to be resent goes now.
Transition sourcesReceived(AppState s, pb.SourcesView view) {
  final project = s.project;
  if (project == null) return Transition(s);
  final held = s.editor.sources;
  var next = held.copyWith(
    revision: view.revision.toInt(),
    files: view.files,
    diagnostics: view.diagnostics,
    openPath: held.openPath ?? view.files.firstOrNull?.path,
  );
  final effects = <Effect>[];
  var state = s;
  if (held.retry != null && held.openPath != null && next.revision == s.revision) {
    effects.add(ApplySourceEdit(baseRevision: s.revision, path: held.openPath!, text: held.retry!));
    next = next.copyWith(sent: held.retry, clearRetry: true);
    state = pending(state);
  }
  state = state.copyWith(editor: state.editor.copyWith(sources: next));
  // A reveal that waited for these sources.
  if (state.editor.pendingReveal case final node? when next.revision == state.revision) {
    final r = _revealNow(state, node);
    return Transition(r.state, [...effects, ...r.effects]);
  }
  return Transition(state, effects);
}

/// ApplySourceEdit answered.  Accepted: the projection is a new revision
/// and the sources are its text.  Refused: the project is unchanged, the
/// draft is what was typed, the diagnostics say why.  Either way the
/// buffer is settled when nothing was typed since it was sent.
Transition sourceEditApplied(AppState s, pb.SourceEditApplied applied) {
  final held = s.editor.sources;
  // What was typed after the send is the next edit; otherwise the daemon's
  // text (the typed text itself, accepted or as a draft) is the editor's.
  final typedSince = held.buffer != null && held.buffer != held.sent;
  var sources = held.copyWith(clearSent: true);
  if (applied.hasSources()) {
    sources = sources.copyWith(
      revision: applied.sources.revision.toInt(),
      files: applied.sources.files,
      diagnostics: applied.sources.diagnostics,
    );
  }
  if (!typedSince) sources = sources.copyWith(clearBuffer: true);
  var state = s.copyWith(
    editor: s.editor.copyWith(pendingRequests: decPending(s), sources: sources),
  );
  final effects = <Effect>[];
  if (applied.accepted && applied.hasProject()) {
    // the projection is a commit like any other, but its text is here
    final t = projectReceived(state, applied.project, null, false);
    state = t.state.copyWith(editor: t.state.editor.copyWith(sources: sources));
    effects.addAll(t.effects.where((e) => e is! GetSources));
  }
  if (typedSince && sources.openPath != null) {
    effects.add(
      ApplySourceEdit(baseRevision: state.revision, path: sources.openPath!, text: held.buffer!),
    );
    state = pending(
      state.copyWith(
        editor: state.editor.copyWith(sources: sources.copyWith(sent: held.buffer)),
      ),
    );
    return Transition(state, effects);
  }
  // nothing left to send: a save or an unload that waited for this edit
  final settled = afterSourceEditSettled(state);
  return Transition(settled.state, [...effects, ...settled.effects]);
}

/// *Reveal in Code* for a canvas node: the node is selected, the Split view
/// is shown when the canvas alone was, and the file declaring the node is
/// opened at its declaration — through the Code view's own reveal (the one
/// definition navigation uses).  Before the sources of this revision are on
/// hand the reveal waits for them ([EditorState.pendingReveal]).
Transition revealInCode(AppState s, NodeRef node) {
  var t = reduce(s, SelectionChanged(singleSelection(node)));
  if (t.state.editor.view == DesignView.design) {
    final v = viewChanged(t.state, DesignView.split);
    t = Transition(v.state, [...t.effects, ...v.effects]);
  }
  final state = t.state;
  if (state.editor.sources.revision != state.revision) {
    return Transition(
      state.copyWith(editor: state.editor.copyWith(pendingReveal: node)),
      t.effects,
    );
  }
  final r = _revealNow(state, node);
  return Transition(r.state, [...t.effects, ...r.effects]);
}

Transition _revealNow(AppState s, NodeRef node) {
  final sel = singleSelection(node);
  for (final file in s.editor.sources.files) {
    final a = anchorOf(s, sel, file);
    if (a == null) continue;
    final generation = s.editor.toolingGeneration + 1;
    var editor = s.editor.copyWith(
      toolingGeneration: generation,
      clearPendingReveal: true,
      reveal: SourceReveal(
        generation: generation,
        location: SourceLocation(path: file.path, start: a.start, end: a.end),
      ),
    );
    if (file.path != editor.sources.openPath) {
      editor = editor.copyWith(
        sources: editor.sources.copyWith(openPath: file.path, clearBuffer: true),
      );
    }
    return Transition(s.copyWith(editor: editor));
  }
  return Transition(s.copyWith(editor: s.editor.copyWith(clearPendingReveal: true)));
}

/// The anchor of the canvas selection in the open file, for the Split
/// view's sync: where the editor scrolls when a node is selected.
pb.SourceAnchor? anchorOf(AppState s, Selection selection, pb.SourceFileView file) {
  final component = switch (s.editor.context) {
    SystemContext() => null,
    ComponentContext(:final id) => id,
  };
  bool inScope(pb.SourceAnchor a) => (a.hasComponent() ? a.component.toInt() : null) == component;
  for (final a in file.anchors) {
    final hit = switch (selection) {
      ConceptSelected(:final id) => a.hasConceptId() && a.conceptId.toInt() == id && inScope(a),
      MappingSelected(:final id) => a.hasMappingId() && a.mappingId.toInt() == id && inScope(a),
      OutputSelected(:final id) => a.hasOutputId() && a.outputId.toInt() == id && inScope(a),
      InstanceSelected(:final id) => a.hasInstanceId() && a.instanceId.toInt() == id,
      ComponentSelected(:final id) => a.hasComponentId() && a.componentId.toInt() == id,
      PortSelected(:final port) => a.hasPortId() && a.portId.toInt() == port,
      _ => false,
    };
    if (hit) return a;
  }
  return null;
}

/// The selection an anchor stands for on the canvas on screen, for the
/// other direction of the sync: the caret in an item selects its node.
Selection? selectionOf(AppState s, pb.SourceAnchor a) {
  final component = switch (s.editor.context) {
    SystemContext() => null,
    ComponentContext(:final id) => id,
  };
  final scope = a.hasComponent() ? a.component.toInt() : null;
  if (a.hasConceptId() && scope == component) return ConceptSelected(a.conceptId.toInt());
  if (a.hasMappingId() && scope == component) return MappingSelected(a.mappingId.toInt());
  if (a.hasOutputId() && scope == component) return OutputSelected(a.outputId.toInt());
  if (a.hasInstanceId() && component == null) return InstanceSelected(a.instanceId.toInt());
  if (a.hasComponentId() && component == null) return ComponentSelected(a.componentId.toInt());
  return null;
}
