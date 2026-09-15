/// The pure reducer: `State × Action → State × Effects`.
///
/// No I/O, no Flutter widgets, no daemon.  Everything about *how the
/// application behaves* is readable here and testable without a widget tree.
library;

import 'dart:ui' show Offset;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'actions.dart';
import 'effects.dart';
import 'state.dart';

@immutable
class Transition {
  const Transition(this.state, [this.effects = const []]);
  final AppState state;
  final List<Effect> effects;
}

const int _maxLogLines = 200;

Transition reduce(AppState s, AppAction action) {
  return switch (action) {
    AppStarted() || ConnectRequested() => _connect(s),

    // ---- project lifecycle -------------------------------------------------
    OpenProjectPickRequested() => _whenConnected(
      s,
      () => s.project == null ? Transition(s, const [PickProjectToOpen()]) : Transition(s),
    ),
    NewProjectPickRequested() => _whenConnected(
      s,
      () => s.project == null ? Transition(s, const [PickNewProjectLocation()]) : Transition(s),
    ),
    OpenProjectRequested(:final rootPath) => _whenConnected(
      s,
      () => Transition(_pending(s), [OpenProject(rootPath)]),
    ),
    NewProjectRequested(:final rootPath, :final name) => _whenConnected(
      s,
      () => Transition(_pending(s), [InitProject(rootPath: rootPath, name: name)]),
    ),
    SaveRequested() => _whenProject(s, () => Transition(_pending(s), const [SaveProject()])),
    CloseProjectRequested() => _whenProject(
      s,
      () => Transition(_pending(s), const [CloseProject()]),
    ),
    UndoRequested() => _whenProject(
      s,
      () => s.project!.canUndo ? Transition(_pending(s), const [Undo()]) : Transition(s),
    ),
    RedoRequested() => _whenProject(
      s,
      () => s.project!.canRedo ? Transition(_pending(s), const [Redo()]) : Transition(s),
    ),

    // ---- concepts ----------------------------------------------------------
    CreateConceptRequested(:final name, :final description) => _edit(
      s,
      pb.EditOp(
        createConcept: pb.CreateConcept(name: name, description: description),
      ),
    ),
    RenameConceptRequested(:final id, :final name) => _edit(
      s,
      pb.EditOp(
        renameConcept: pb.RenameConcept(id: Int64(id), name: name),
      ),
    ),
    SetConceptDescriptionRequested(:final id, :final description) => _edit(
      s,
      pb.EditOp(
        setConceptDescription: pb.SetConceptDescription(id: Int64(id), description: description),
      ),
    ),
    SetConceptRepresentationRequested(:final id, :final representation) => _edit(
      s,
      pb.EditOp(
        setConceptRepresentation: pb.SetConceptRepresentation(
          id: Int64(id),
          representation: representation,
        ),
      ),
    ),
    DeleteConceptRequested(:final id) => _edit(
      s,
      pb.EditOp(deleteConcept: pb.DeleteConcept(id: Int64(id))),
    ),

    // ---- mappings ----------------------------------------------------------
    CreateMappingRequested(:final name, :final inputs, :final output) => _edit(
      s,
      pb.EditOp(
        createMapping: pb.CreateMapping(
          name: name,
          signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
        ),
      ),
    ),
    RenameMappingRequested(:final id, :final name) => _edit(
      s,
      pb.EditOp(
        renameMapping: pb.RenameMapping(id: Int64(id), name: name),
      ),
    ),
    SetMappingDescriptionRequested(:final id, :final description) => _edit(
      s,
      pb.EditOp(
        setMappingDescription: pb.SetMappingDescription(id: Int64(id), description: description),
      ),
    ),
    SetMappingSignatureRequested(:final id, :final inputs, :final output) => _edit(
      s,
      _setSignature(id, inputs, output),
    ),
    AttachFormulaRequested(:final mappingId, :final source) => _edit(
      s,
      pb.EditOp(
        attachDefinition: pb.AttachDefinition(
          id: Int64(mappingId),
          definition: pb.Definition(formula: source),
        ),
      ),
    ),
    ReplaceDefinitionRequested(:final mappingId, :final source) => _edit(
      s,
      pb.EditOp(
        replaceDefinition: pb.ReplaceDefinition(
          id: Int64(mappingId),
          definition: source == null ? null : pb.Definition(formula: source),
        ),
      ),
    ),
    DeleteMappingRequested(:final id) => _edit(
      s,
      pb.EditOp(deleteMapping: pb.DeleteMapping(id: Int64(id))),
    ),
    DeleteSelectionRequested() => switch (s.editor.selection) {
      NoSelection() => Transition(s),
      ConceptSelected(:final id) => reduce(s, DeleteConceptRequested(id)),
      MappingSelected(:final id) => reduce(s, DeleteMappingRequested(id)),
    },

    // ---- canvas links (typed by concept identity) --------------------------
    LinkConceptToMappingInput(:final conceptId, :final mappingId) => _withMapping(s, mappingId, (
      m,
    ) {
      final inputs = m.signature.inputs.map((i) => i.toInt()).toList();
      if (inputs.contains(conceptId)) return Transition(s);
      return _edit(s, _setSignature(mappingId, [...inputs, conceptId], m.signature.output.toInt()));
    }),
    LinkMappingOutputToConcept(:final mappingId, :final conceptId) => _withMapping(s, mappingId, (
      m,
    ) {
      if (m.signature.output.toInt() == conceptId) return Transition(s);
      final inputs = m.signature.inputs.map((i) => i.toInt()).toList();
      return _edit(s, _setSignature(mappingId, inputs, conceptId));
    }),
    UnlinkMappingInput(:final mappingId, :final conceptId) => _withMapping(s, mappingId, (m) {
      final inputs = m.signature.inputs.map((i) => i.toInt()).where((i) => i != conceptId).toList();
      return _edit(s, _setSignature(mappingId, inputs, m.signature.output.toInt()));
    }),

    // ---- editor state ------------------------------------------------------
    PageSelected(:final page) => Transition(s.copyWith(editor: s.editor.copyWith(page: page))),
    RemoveRecentRequested(:final path) => () {
      final recent = s.recent.where((r) => r.path != path).toList();
      return Transition(s.copyWith(recent: recent), [SaveRecentProjects(recent)]);
    }(),
    RecentProjectsLoaded(:final recent) => Transition(s.copyWith(recent: recent)),
    SelectionChanged(:final selection) => Transition(
      s.copyWith(editor: s.editor.copyWith(selection: selection)),
    ),
    NodeMoved(:final node, :final position) => _whenProject(s, () {
      final layout = {...s.editor.layout, node: position};
      return Transition(s.copyWith(editor: s.editor.copyWith(layout: layout)), [
        SetLayout(layoutToPb(layout)),
      ]);
    }),
    ErrorDismissed() => Transition(s.copyWith(editor: s.editor.copyWith(clearError: true))),

    // ---- responses ---------------------------------------------------------
    DaemonConnected(:final executable, :final handshake) => Transition(
      s.copyWith(
        connection: handshake.compatible
            ? Connected(executable: executable, handshake: handshake)
            : ConnectionFailed(
                'protocol mismatch: daemon speaks '
                '${handshake.protocolVersion.major}.x, Studio does not',
              ),
      ),
    ),
    DaemonConnectionFailed(:final reason) => Transition(
      s.copyWith(connection: ConnectionFailed(reason)),
    ),
    DaemonExited(:final exitCode) => Transition(
      s.copyWith(
        connection: ConnectionFailed('compiler service exited (code $exitCode)'),
        clearProject: true,
        editor: s.editor.copyWith(
          pendingRequests: 0,
          selection: const NoSelection(),
          layout: const {},
        ),
      ),
    ),
    DaemonLogged(:final line) => Transition(
      s.copyWith(render: s.render.copyWith(daemonLog: _appendLog(s.render.daemonLog, line))),
    ),
    ProjectReceived(:final project, :final outcome, :final fromRequest) => _projectReceived(
      s,
      project,
      outcome,
      fromRequest,
    ),
    ProjectClosed() => Transition(
      s.copyWith(
        clearProject: true,
        editor: s.editor.copyWith(
          pendingRequests: _dec(s),
          selection: const NoSelection(),
          layout: const {},
          clearOutcome: true,
        ),
      ),
    ),
    RequestSucceeded() => Transition(
      s.copyWith(editor: s.editor.copyWith(pendingRequests: _dec(s))),
    ),
    RequestFailed(:final code, :final message, :final details) => Transition(
      s.copyWith(
        editor: s.editor.copyWith(
          pendingRequests: _dec(s),
          lastError: UserFacingError(code: code, message: message, details: details),
        ),
      ),
    ),
  };
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

Transition _connect(AppState s) {
  if (s.connection is Connecting) return Transition(s);
  return Transition(s.copyWith(connection: const Connecting('')), const [
    ConnectDaemon(),
    LoadRecentProjects(),
  ]);
}

const int _maxRecent = 12;

/// Newest first, de-duplicated by path, capped.
List<RecentProject> _remember(List<RecentProject> recent, pb.ProjectProjection p) {
  final entry = RecentProject(path: p.rootPath, name: p.name, lastOpened: DateTime.now());
  return [entry, ...recent.where((r) => r.path != p.rootPath)].take(_maxRecent).toList();
}

Transition _whenConnected(AppState s, Transition Function() then) =>
    s.connection is Connected ? then() : Transition(s);

Transition _whenProject(AppState s, Transition Function() then) =>
    s.connection is Connected && s.project != null ? then() : Transition(s);

Transition _withMapping(AppState s, int id, Transition Function(pb.MappingView m) then) {
  final m = s.project?.mappings.where((m) => m.id.toInt() == id).firstOrNull;
  return m == null ? Transition(s) : then(m);
}

pb.EditOp _setSignature(int id, List<int> inputs, int output) => pb.EditOp(
  setMappingSignature: pb.SetMappingSignature(
    id: Int64(id),
    signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
  ),
);

/// Every semantic edit is sent against the revision Studio currently holds;
/// the daemon refuses it if the project has moved on.
Transition _edit(AppState s, pb.EditOp op) =>
    _whenProject(s, () => Transition(_pending(s), [ApplyEdit(baseRevision: s.revision, op: op)]));

AppState _pending(AppState s) =>
    s.copyWith(editor: s.editor.copyWith(pendingRequests: s.editor.pendingRequests + 1));

int _dec(AppState s) => s.editor.pendingRequests > 0 ? s.editor.pendingRequests - 1 : 0;

/// Accept a projection only if it is at least as new as what we hold for the
/// same project.  Responses for an older revision are discarded — this is
/// the stale-result rule of the protocol.
Transition _projectReceived(
  AppState s,
  pb.ProjectProjection incoming,
  pb.EditOutcome? outcome,
  bool fromRequest,
) {
  final current = s.project;
  final sameProject = current != null && current.rootPath == incoming.rootPath;
  final pending = fromRequest ? _dec(s) : s.editor.pendingRequests;
  if (sameProject && incoming.revision < current.revision) {
    return Transition(s.copyWith(editor: s.editor.copyWith(pendingRequests: pending)));
  }
  final selection = _selectionStillValid(s.editor.selection, incoming)
      ? s.editor.selection
      : const NoSelection();
  // Layout: the daemon's copy is authoritative on open; afterwards Studio is
  // the author and only merges in positions it does not know yet.
  final stored = layoutFromPb(incoming.layout);
  final layout = sameProject ? {...stored, ...s.editor.layout} : stored;
  final recent = sameProject ? s.recent : _remember(s.recent, incoming);
  return Transition(
    s.copyWith(
      project: incoming,
      recent: recent,
      editor: s.editor.copyWith(
        pendingRequests: pending,
        selection: selection,
        layout: layout,
        lastOutcome: outcome,
      ),
    ),
    // A freshly opened project needs a subscription for pushed changes and
    // goes to the top of the recent list.
    sameProject ? const [] : [const SubscribeProject(), SaveRecentProjects(recent)],
  );
}

bool _selectionStillValid(Selection sel, pb.ProjectProjection p) => switch (sel) {
  NoSelection() => true,
  ConceptSelected(:final id) => p.concepts.any((c) => c.id.toInt() == id),
  MappingSelected(:final id) => p.mappings.any((m) => m.id.toInt() == id),
};

List<String> _appendLog(List<String> log, String line) {
  final next = [...log, line];
  return next.length > _maxLogLines ? next.sublist(next.length - _maxLogLines) : next;
}

Map<NodeRef, Offset> layoutFromPb(pb.Layout l) => {
  for (final n in l.concepts) NodeRef.concept(n.id.toInt()): Offset(n.x, n.y),
  for (final n in l.mappings) NodeRef.mapping(n.id.toInt()): Offset(n.x, n.y),
};

pb.Layout layoutToPb(Map<NodeRef, Offset> layout) {
  final entries = layout.entries.toList()..sort((a, b) => a.key.id.compareTo(b.key.id));
  pb.NodePosition pos(MapEntry<NodeRef, Offset> e) =>
      pb.NodePosition(id: Int64(e.key.id), x: e.value.dx, y: e.value.dy);
  return pb.Layout(
    concepts: [
      for (final e in entries)
        if (e.key.kind == NodeKind.concept) pos(e),
    ],
    mappings: [
      for (final e in entries)
        if (e.key.kind == NodeKind.mapping) pos(e),
    ],
  );
}
