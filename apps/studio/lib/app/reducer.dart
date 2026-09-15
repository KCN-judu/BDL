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
import 'drafts.dart';
import 'effects.dart';
import 'state.dart';
import 'tooling.dart';

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
    CreateConceptRequested(:final name, :final description, :final representation) => _edit(
      s,
      pb.EditOp(
        createConcept: pb.CreateConcept(
          name: name,
          description: description,
          representation: representation,
        ),
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
    DeleteMappingRequested(:final id) => _edit(
      s,
      pb.EditOp(deleteMapping: pb.DeleteMapping(id: Int64(id))),
    ),

    // ---- definition drafts (app/drafts.dart) --------------------------------
    DefinitionDraftChanged(:final mappingId, :final source) => _whenProject(
      s,
      () => draftChanged(s, mappingId, source),
    ),
    DefinitionDraftReverted(:final mappingId) ||
    DefinitionDraftReloaded(:final mappingId) => draftDropped(s, mappingId),
    DefinitionDraftKept(:final mappingId) => draftKept(s, mappingId),
    CommitDefinitionRequested(:final mappingId) => _whenProject(
      s,
      () => commitDefinition(s, mappingId),
    ),
    DetachDefinitionRequested(:final mappingId) => _whenProject(
      s,
      () => detachDefinition(s, mappingId),
    ),
    DraftAnalysisReceived(:final result) => draftAnalysisReceived(s, result),
    DraftAnalysisFailed(:final mappingId, :final generation, :final code, :final message) =>
      draftAnalysisFailed(s, mappingId, generation, code, message),
    DeleteSelectionRequested() => switch (s.editor.selection) {
      NoSelection() => Transition(s),
      ConceptSelected(:final id) => reduce(s, DeleteConceptRequested(id)),
      MappingSelected(:final id) => reduce(s, DeleteMappingRequested(id)),
      OutputSelected(:final id) => reduce(s, DeleteOutputRequested(id)),
    },

    // ---- timing domains ---------------------------------------------------
    CreateClockDomainRequested(:final name) => _edit(
      s,
      pb.EditOp(createClockDomain: pb.CreateClockDomain(name: name)),
    ),
    RenameClockDomainRequested(:final id, :final name) => _edit(
      s,
      pb.EditOp(
        renameClockDomain: pb.RenameClockDomain(id: Int64(id), name: name),
      ),
    ),
    DeleteClockDomainRequested(:final id) => _edit(
      s,
      pb.EditOp(deleteClockDomain: pb.DeleteClockDomain(id: Int64(id))),
    ),
    SetMappingClockRequested(:final mappingId, :final clockId) => _edit(
      s,
      pb.EditOp(
        setMappingClock: pb.SetMappingClock(
          id: Int64(mappingId),
          clockId: clockId == null ? null : Int64(clockId),
        ),
      ),
    ),

    // ---- physical outputs -------------------------------------------------
    CreateOutputRequested(
      :final name,
      :final description,
      :final accepts,
      :final clockId,
      :final required,
    ) =>
      _createOutput(s, name, description, accepts, clockId, required),
    RenameOutputRequested(:final id, :final name) => _edit(
      s,
      pb.EditOp(
        renameOutput: pb.RenameOutput(id: Int64(id), name: name),
      ),
    ),
    SetOutputAcceptsRequested(:final id, :final accepts) => _edit(
      s,
      pb.EditOp(
        setOutputAccepts: pb.SetOutputAccepts(id: Int64(id), accepts: Int64(accepts)),
      ),
    ),
    SetOutputClockRequested(:final id, :final clockId) => _edit(
      s,
      pb.EditOp(
        setOutputClock: pb.SetOutputClock(
          id: Int64(id),
          clockId: clockId == null ? null : Int64(clockId),
        ),
      ),
    ),
    SetOutputRequiredRequested(:final id, :final required) => _edit(
      s,
      pb.EditOp(
        setOutputRequired: pb.SetOutputRequired(id: Int64(id), required: required),
      ),
    ),
    DeleteOutputRequested(:final id) => _edit(
      s,
      pb.EditOp(deleteOutput: pb.DeleteOutput(id: Int64(id))),
    ),
    SetMappingDriveRequested(:final mappingId, :final outputId) => _edit(
      s,
      pb.EditOp(
        setMappingDrive: pb.SetMappingDrive(
          id: Int64(mappingId),
          outputId: outputId == null ? null : Int64(outputId),
        ),
      ),
    ),

    // ---- devices -------------------------------------------------------------
    CreateDeviceRequested(:final name, :final kind, :final outputId) => _edit(
      s,
      pb.EditOp(
        createDevice: pb.CreateDevice(
          name: name,
          kind: kind,
          outputId: outputId == null ? null : Int64(outputId),
        ),
      ),
    ),
    RenameDeviceRequested(:final id, :final name) => _edit(
      s,
      pb.EditOp(
        renameDevice: pb.RenameDevice(id: Int64(id), name: name),
      ),
    ),
    SetDeviceKindRequested(:final id, :final kind) => _edit(
      s,
      pb.EditOp(
        setDeviceKind: pb.SetDeviceKind(id: Int64(id), kind: kind),
      ),
    ),
    SetDeviceOutputRequested(:final id, :final outputId) => _edit(
      s,
      pb.EditOp(
        setDeviceOutput: pb.SetDeviceOutput(
          id: Int64(id),
          outputId: outputId == null ? null : Int64(outputId),
        ),
      ),
    ),
    SetDevicePinRequested(:final id, :final index, :final resource) => _edit(
      s,
      pb.EditOp(
        setDevicePin: pb.SetDevicePin(id: Int64(id), index: index, resource: resource),
      ),
    ),
    DeleteDeviceRequested(:final id) => _edit(
      s,
      pb.EditOp(deleteDevice: pb.DeleteDevice(id: Int64(id))),
    ),

    // ---- semantic actions (app/tooling.dart) ---------------------------------
    SemanticActionsRequested(:final entity) => semanticActionsRequested(s, entity),
    SemanticActionsReceived(:final generation, :final result) => semanticActionsReceived(
      s,
      generation,
      result,
    ),
    SemanticActionApplied(:final actionId, :final option) => semanticActionApplied(
      s,
      actionId,
      option,
    ),

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

    // ---- semantic tooling (app/tooling.dart) --------------------------------
    CompletionRequested(:final mappingId, :final source, :final offset) => completionRequested(
      s,
      mappingId,
      source,
      offset,
    ),
    CompletionDismissed() => completionDismissed(s),
    CompletionMoved(:final delta) => completionMoved(s, delta),
    FormulaHoverRequested(:final mappingId, :final source, :final offset) => formulaHoverRequested(
      s,
      mappingId,
      source,
      offset,
    ),
    EntityHoverRequested(:final entity) => entityHoverRequested(s, entity),
    CompletionReceived(:final generation, :final result) => completionReceived(
      s,
      generation,
      result,
    ),
    HoverReceived(:final generation, :final result) => hoverReceived(s, generation, result),
    ToolingFailed(:final generation) => toolingFailed(s, generation),

    // ---- editor state ------------------------------------------------------
    PageSelected(:final page) => Transition(
      s.copyWith(editor: withoutTooling(s.editor).copyWith(page: page)),
    ),
    RemoveRecentRequested(:final path) => () {
      final recent = s.recent.where((r) => r.path != path).toList();
      return Transition(s.copyWith(recent: recent), [SaveRecentProjects(recent)]);
    }(),
    RecentProjectsLoaded(:final recent) => Transition(s.copyWith(recent: recent)),
    AnalysisReceived(:final analysis, :final fromRequest) => () {
      final pending = fromRequest ? _dec(s) : s.editor.pendingRequests;
      // Keep only an analysis of the revision we hold; older ones are stale,
      // newer ones mean a projection is on its way and will bring its own.
      final keep = s.project != null && analysis.revision == s.project!.revision;
      return Transition(
        s.copyWith(
          analysis: keep ? analysis : null,
          clearAnalysis: !keep,
          editor: s.editor.copyWith(pendingRequests: pending),
        ),
      );
    }(),
    PickerUnavailable() => Transition(
      s.copyWith(
        editor: s.editor.copyWith(
          pickerUnavailable: true,
          lastError: const UserFacingError(
            code: 'studio.picker_unavailable',
            message:
                'The system file dialog could not be shown. This happens when Studio is '
                'launched from a sandboxed host (an embedded terminal, for example). '
                'Launch it from Finder or Terminal, or enter a path below.',
          ),
        ),
      ),
    ),
    SelectionChanged(:final selection) => _selected(
      s.copyWith(editor: withoutTooling(s.editor).copyWith(selection: selection)),
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
          drafts: const {},
          stashedDrafts: _stash(s),
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
          drafts: const {},
          stashedDrafts: _stash(s),
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
          drafts: draftsAfterFailedRequest(s.editor.drafts, message),
          // a failed step ends its plan; nothing after it is sent blindly
          queuedEdits: const [],
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

/// After a selection (or a new revision under one): ask the service which
/// fixes it offers for the selected object.
Transition _selected(AppState s) {
  final entity = s.selectedEntity;
  if (entity == null) {
    return Transition(s.copyWith(editor: s.editor.copyWith(clearActions: true)));
  }
  return semanticActionsRequested(s, entity);
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
/// the daemon refuses it if the project has moved on.  No effect when
/// disconnected or without a project.
Transition sendEdit(AppState s, pb.EditOp op) =>
    _whenProject(s, () => Transition(_pending(s), [ApplyEdit(baseRevision: s.revision, op: op)]));

Transition _edit(AppState s, pb.EditOp op) => sendEdit(s, op);

Transition _createOutput(
  AppState s,
  String name,
  String description,
  int accepts,
  int? clockId,
  bool required,
) {
  // Two facts the model keeps as two ops: create, then mark required.  The
  // second is queued for the confirming revision.
  final t = _edit(
    s,
    pb.EditOp(
      createOutput: pb.CreateOutput(
        name: name,
        description: description,
        accepts: Int64(accepts),
        clockId: clockId == null ? null : Int64(clockId),
      ),
    ),
  );
  if (!required || t.effects.isEmpty) return t;
  return Transition(
    t.state.copyWith(
      editor: t.state.editor.copyWith(queuedEdits: [...t.state.editor.queuedEdits, _markRequired]),
    ),
    t.effects,
  );
}

/// A queued edit that needs the id the previous step created: resolved
/// from the outcome when the confirming projection arrives.
final pb.EditOp _markRequired = pb.EditOp(
  setOutputRequired: pb.SetOutputRequired(id: Int64(-1), required: true),
);

/// Dirty drafts of the project being closed, filed under its path so a
/// reopen restores them.
Map<String, Map<int, DefinitionDraft>> _stash(AppState s) {
  final root = s.project?.rootPath;
  if (root == null) return s.editor.stashedDrafts;
  final dirty = dirtyDrafts(s);
  final next = {...s.editor.stashedDrafts};
  if (dirty.isEmpty) {
    next.remove(root);
  } else {
    next[root] = dirty;
  }
  return next;
}

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
  final analysisStillValid = s.analysis != null && s.analysis!.revision == incoming.revision;
  // Drafts: rebased on every new revision; restored from the stash when a
  // project is (re)opened.  Same revision (a save) changes nothing.
  final stashed = s.editor.stashedDrafts;
  final ({Map<int, DefinitionDraft> drafts, List<Effect> effects}) drafts = !sameProject
      ? rebaseDrafts(stashed[incoming.rootPath] ?? const {}, incoming)
      : incoming.revision == current.revision
      ? (drafts: s.editor.drafts, effects: const <Effect>[])
      : rebaseDrafts(s.editor.drafts, incoming);
  final editor = sameProject && incoming.revision == current.revision
      ? s.editor
      : withoutTooling(s.editor);
  return Transition(
        s.copyWith(
          project: incoming,
          recent: recent,
          clearAnalysis: !analysisStillValid,
          editor: editor.copyWith(
            pendingRequests: pending,
            selection: selection,
            layout: layout,
            lastOutcome: outcome,
            drafts: drafts.drafts,
            stashedDrafts: sameProject ? stashed : ({...stashed}..remove(incoming.rootPath)),
          ),
        ),
        // A freshly opened project needs a subscription for pushed changes, an
        // analysis of what was just opened, and goes to the top of Recent.
        // After an edit the daemon pushes AnalysisReady on its own.
        [
          if (!sameProject) ...[
            const SubscribeProject(),
            const RunAnalysis(),
            SaveRecentProjects(recent),
          ],
          ...drafts.effects,
        ],
      )
      .thenQueued(fromRequest && sameProject ? outcome : null)
      .thenActions(changed: !sameProject || incoming.revision != current.revision);
}

extension on Transition {
  /// The actions on screen are about a revision; a new one re-asks for
  /// the selection (drafts are unaffected: they carry their own requests).
  Transition thenActions({required bool changed}) {
    if (!changed) return this;
    final t = _selected(state);
    return Transition(t.state, [...effects, ...t.effects]);
  }

  /// A multi-step plan sends its next edit once the previous one is
  /// confirmed, against the revision just received.  A queued edit whose
  /// id is the created entity of the previous step is resolved here.
  Transition thenQueued(pb.EditOutcome? outcome) {
    final queue = state.editor.queuedEdits;
    if (queue.isEmpty || state.project == null || outcome == null) return this;
    var next = queue.first;
    if (next.hasSetOutputRequired() && next.setOutputRequired.id.toInt() < 0) {
      if (!outcome.hasCreatedOutput()) {
        return Transition(
          state.copyWith(editor: state.editor.copyWith(queuedEdits: const [])),
          effects,
        );
      }
      next = pb.EditOp(
        setOutputRequired: pb.SetOutputRequired(id: outcome.createdOutput, required: true),
      );
    }
    final rest = queue.sublist(1);
    final sent = sendEdit(state.copyWith(editor: state.editor.copyWith(queuedEdits: rest)), next);
    return Transition(sent.state, [...effects, ...sent.effects]);
  }
}

bool _selectionStillValid(Selection sel, pb.ProjectProjection p) => switch (sel) {
  NoSelection() => true,
  ConceptSelected(:final id) => p.concepts.any((c) => c.id.toInt() == id),
  MappingSelected(:final id) => p.mappings.any((m) => m.id.toInt() == id),
  OutputSelected(:final id) => p.outputs.any((o) => o.id.toInt() == id),
};

List<String> _appendLog(List<String> log, String line) {
  final next = [...log, line];
  return next.length > _maxLogLines ? next.sublist(next.length - _maxLogLines) : next;
}

Map<NodeRef, Offset> layoutFromPb(pb.Layout l) => {
  for (final n in l.concepts) NodeRef.concept(n.id.toInt()): Offset(n.x, n.y),
  for (final n in l.mappings) NodeRef.mapping(n.id.toInt()): Offset(n.x, n.y),
  for (final n in l.outputs) NodeRef.output(n.id.toInt()): Offset(n.x, n.y),
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
    outputs: [
      for (final e in entries)
        if (e.key.kind == NodeKind.output) pos(e),
    ],
  );
}
