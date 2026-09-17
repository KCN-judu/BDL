/// The pure reducer: `State × Action → State × Effects`.
///
/// No I/O, no Flutter widgets, no daemon.  Everything about *how the
/// application behaves* is readable here and testable without a widget tree.
library;

import 'dart:ui' show Rect;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'actions.dart';
import 'deploy.dart';
import 'drafts.dart';
import 'effects.dart';
import 'simulation.dart';
import 'state.dart';
import 'system.dart';
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
    NewProjectPickRequested(:final kind) => _whenConnected(
      s,
      () => s.project == null ? Transition(s, [PickNewProjectLocation(kind: kind)]) : Transition(s),
    ),
    OpenProjectRequested(:final rootPath) => _whenConnected(
      s,
      () => Transition(pending(s), [OpenProject(rootPath)]),
    ),
    NewProjectRequested(:final rootPath, :final name, :final kind) => _whenConnected(
      s,
      () => Transition(pending(s), [InitProject(rootPath: rootPath, name: name, kind: kind)]),
    ),
    SaveRequested(:final force) => _whenProject(
      s,
      () => Transition(pending(s.copyWith(editor: s.editor.copyWith(clearError: true))), [
        SaveProject(force: force),
      ]),
    ),
    ReloadProjectRequested() => _whenProject(
      s,
      () => Transition(pending(s.copyWith(editor: s.editor.copyWith(clearError: true))), const [
        ReloadProject(),
      ]),
    ),
    CloseProjectRequested() => _whenProject(
      s,
      () => Transition(pending(s), const [CloseProject()]),
    ),
    UndoRequested() => _whenProject(
      s,
      () => s.project!.canUndo ? Transition(pending(s), const [Undo()]) : Transition(s),
    ),
    RedoRequested() => _whenProject(
      s,
      () => s.project!.canRedo ? Transition(pending(s), const [Redo()]) : Transition(s),
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
    CreateMappingRequested(:final name, :final inputs, :final output, :final group) => () {
      final t = _edit(
        s,
        pb.EditOp(
          createMapping: pb.CreateMapping(
            name: name,
            signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(output)),
          ),
        ),
      );
      if (group == null || t.effects.isEmpty) return t;
      return Transition(
        t.state.copyWith(editor: t.state.editor.copyWith(pendingGroupFor: group)),
        t.effects,
      );
    }(),
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
      NoSelection() || PortSelected() || MultiSelected() => Transition(s),
      ConceptSelected(:final id) => reduce(s, DeleteConceptRequested(id)),
      MappingSelected(:final id) => reduce(s, DeleteMappingRequested(id)),
      OutputSelected(:final id) => reduce(s, DeleteOutputRequested(id)),
      ComponentSelected(:final id) => reduce(s, DeleteComponentRequested(id)),
      InstanceSelected(:final id) => reduce(s, DeleteInstanceRequested(id)),
      BindingSelected(:final id) => reduce(s, UnbindRequested(id)),
      // Deleting a group keeps its relationships; the destructive delete
      // is a named action in the group's inspector.
      GroupSelected(:final id) => reduce(s, UngroupRequested(id)),
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

    // ---- simulation (app/simulation.dart) ---------------------------------------
    SimulationInputChanged(:final mappingId, :final value) => simulationInputChanged(
      s,
      mappingId,
      value,
    ),
    SimulationPeriodChanged(:final clockId, :final period) => simulationPeriodChanged(
      s,
      clockId,
      period,
    ),
    SimulationStepRequested(:final ticks) => _whenProject(
      s,
      () => simulationStepRequested(s, ticks),
    ),
    SimulationResetRequested() => simulationResetRequested(s),
    SimulationReceived(:final generation, :final response) => simulationReceived(
      s,
      generation,
      response,
    ),
    SimulationFailed(:final generation, :final message) => simulationFailed(s, generation, message),

    // ---- deployment (app/deploy.dart) -----------------------------------------
    TargetsRequested() => targetsRequested(s),
    TargetsReceived(:final targets) => targetsReceived(s, targets),
    TargetSelected(:final targetId) => targetSelected(s, targetId),
    DeploymentRequested() => deploymentRequested(s),
    DeploymentReceived(:final generation, :final analysis) => deploymentReceived(
      s,
      generation,
      analysis,
    ),
    DeploymentFailed(:final generation, :final message) => deploymentFailed(s, generation, message),

    // ---- editor state ------------------------------------------------------
    PageSelected(:final page) => () {
      final t = Transition(s.copyWith(editor: withoutTooling(s.editor).copyWith(page: page)));
      // Deploy needs the board list once; ask on the first visit.
      if (page != StudioPage.deploy || s.connection is! Connected) return t;
      final targets = targetsRequested(t.state);
      return Transition(targets.state, [...t.effects, ...targets.effects]);
    }(),
    RemoveRecentRequested(:final path) => () {
      final recent = s.recent.where((r) => r.path != path).toList();
      return Transition(s.copyWith(recent: recent), [SaveRecentProjects(recent)]);
    }(),
    RecentProjectsLoaded(:final recent) => Transition(s.copyWith(recent: recent)),
    AnalysisReceived(:final analysis, :final fromRequest) => () {
      final pending = fromRequest ? decPending(s) : s.editor.pendingRequests;
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
      s.copyWith(
        editor: withoutTooling(s.editor).copyWith(selection: selection, clearRenaming: true),
      ),
    ),

    // ---- concept library ---------------------------------------------------
    InsertConceptTemplateRequested(:final templateId, :final position) => _whenProject(s, () {
      if (s.editor.pendingInsert != null) return Transition(s);
      final busy = pending(s);
      return Transition(
        busy.copyWith(
          editor: busy.editor.copyWith(
            pendingInsert: PendingInsert(templateId: templateId, position: position),
            recentTemplates: rememberTemplate(s.editor.recentTemplates, templateId),
            clearRenaming: true,
          ),
        ),
        [
          InstantiateConceptTemplate(
            baseRevision: s.revision,
            templateId: templateId,
            component: s.editor.componentScope,
          ),
        ],
      );
    }),
    SidebarTabSelected(:final tab) => Transition(
      s.copyWith(editor: s.editor.copyWith(sidebar: tab)),
    ),
    LibrarySearchChanged(:final query) => Transition(
      s.copyWith(editor: s.editor.copyWith(librarySearch: query)),
    ),
    InlineRenameStarted(:final node) => _whenProject(
      s,
      () => Transition(
        s.copyWith(
          editor: s.editor.copyWith(
            selection: switch (node.kind) {
              NodeKind.concept => ConceptSelected(node.id),
              NodeKind.mapping => MappingSelected(node.id),
              NodeKind.output => OutputSelected(node.id),
              NodeKind.instance => InstanceSelected(node.id),
              NodeKind.group => GroupSelected(node.id),
            },
            renaming: node,
          ),
        ),
      ),
    ),
    InlineRenameFinished(:final node, :final name) => _inlineRenameFinished(s, node, name),
    ConceptTemplatesReceived(:final library) => Transition(s.copyWith(library: library)),
    // A collapsed group's box is layout of its own kind.
    NodeMoved(:final node, :final position) when node.kind == NodeKind.group => systemAction(
      s,
      GroupBoxChanged(
        id: node.id,
        rect: (s.editor.contextLayout.groups[node.id]?.rect ?? Rect.zero).let(
          (r) => Rect.fromLTWH(position.dx, position.dy, r.width, r.height),
        ),
      ),
    ),
    NodeMoved(:final node, :final position) => _whenProject(s, () {
      final layout = {...s.editor.layout, node: position};
      final layouts = s.editor.layouts.withNodes(s.editor.context, layout);
      return Transition(
        s.copyWith(
          editor: s.editor.copyWith(layout: layout, layouts: layouts),
        ),
        [SetLayout(layoutToPb(layouts))],
      );
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
      // The concept libraries are the daemon's; ask once per connection.
      [if (handshake.compatible) const ListConceptTemplates()],
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
          layouts: const CanvasLayout(),
          context: const SystemContext(),
          clearExtraction: true,
          clearPendingBind: true,
          queuedSystemEdits: const [],
          drafts: const {},
          stashedDrafts: _stash(s),
          deploy: deployWithoutProject(s.editor.deploy).copyWith(targetsLoaded: false),
          simulation: const SimulationState(),
        ),
      ),
    ),
    DaemonLogged(:final line) => Transition(
      s.copyWith(render: s.render.copyWith(daemonLog: _appendLog(s.render.daemonLog, line))),
    ),
    ProjectReceived(:final project, :final outcome, :final fromRequest) => projectReceived(
      s,
      project,
      outcome,
      fromRequest,
    ),
    SystemReceived(:final system, :final fromRequest) => systemReceived(
      s,
      system,
      fromRequest: fromRequest,
    ),
    SystemEditApplied(:final system, :final project, :final outcome) => systemEditApplied(
      s,
      system,
      project,
      outcome,
    ),
    SystemAnalysisReceived(:final analysis) => systemAnalysisReceived(s, analysis),
    ExtractionPreviewReceived(:final generation, :final preview) => extractionPreviewReceived(
      s,
      generation,
      preview,
    ),
    ExtractionPreviewFailed(:final generation, :final message) => extractionPreviewFailed(
      s,
      generation,
      message,
    ),
    ProjectClosed() => Transition(
      s.copyWith(
        clearProject: true,
        editor: s.editor.copyWith(
          pendingRequests: decPending(s),
          selection: const NoSelection(),
          layout: const {},
          layouts: const CanvasLayout(),
          context: const SystemContext(),
          clearExtraction: true,
          clearPendingBind: true,
          queuedSystemEdits: const [],
          clearOutcome: true,
          drafts: const {},
          stashedDrafts: _stash(s),
          deploy: deployWithoutProject(s.editor.deploy),
          simulation: const SimulationState(),
        ),
      ),
    ),
    RequestSucceeded() => Transition(
      s.copyWith(editor: s.editor.copyWith(pendingRequests: decPending(s))),
    ),
    RequestFailed(:final code, :final message, :final details) => () {
      final refetch = code == 'group_edit.stale_generation' && s.isSystem;
      return Transition(
        s.copyWith(
          editor: s.editor.copyWith(
            // the failed request settles; a counted refetch takes its place
            pendingRequests: decPending(s) + (refetch ? 1 : 0),
            clearPendingInsert: true,
            clearPendingPlacement: true,
            clearPendingGroup: true,
            renameNextGroup: false,
            lastError: UserFacingError(code: code, message: message, details: details),
            drafts: draftsAfterFailedRequest(s.editor.drafts, message),
            // a failed step ends its plan; nothing after it is sent blindly
            queuedEdits: const [],
            queuedSystemEdits: const [],
          ),
        ),
        // The group table moved under us: show the current one (counted).
        [if (refetch) const GetSystem()],
      );
    }(),
    // ---- system projects (app/system.dart) --------------------------------
    UserAction() => systemAction(s, action),
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

/// The inline editor closed.  A changed name is an ordinary rename edit;
/// an unchanged or empty one changes nothing (the default name stays).
Transition _inlineRenameFinished(AppState s, NodeRef node, String? name) {
  final cleared = s.copyWith(editor: s.editor.copyWith(clearRenaming: true));
  final wanted = name?.trim();
  if (wanted == null || wanted.isEmpty) return Transition(cleared);
  final current = switch (node.kind) {
    NodeKind.concept => s.project?.concepts.where((c) => c.id.toInt() == node.id).firstOrNull?.name,
    NodeKind.mapping => s.mapping(node.id)?.name,
    NodeKind.output => s.project?.outputs.where((o) => o.id.toInt() == node.id).firstOrNull?.name,
    NodeKind.instance => s.instance(node.id)?.name,
    NodeKind.group => s.group(node.id)?.name,
  };
  if (current == null || current == wanted) return Transition(cleared);
  return switch (node.kind) {
    NodeKind.instance => systemAction(cleared, RenameInstanceRequested(id: node.id, name: wanted)),
    NodeKind.group => systemAction(cleared, RenameGroupRequested(id: node.id, name: wanted)),
    NodeKind.concept => sendEdit(
      cleared,
      pb.EditOp(
        renameConcept: pb.RenameConcept(id: Int64(node.id), name: wanted),
      ),
    ),
    NodeKind.mapping => sendEdit(
      cleared,
      pb.EditOp(
        renameMapping: pb.RenameMapping(id: Int64(node.id), name: wanted),
      ),
    ),
    NodeKind.output => sendEdit(
      cleared,
      pb.EditOp(
        renameOutput: pb.RenameOutput(id: Int64(node.id), name: wanted),
      ),
    ),
  };
}

/// Move [templateId] to the front of the recent list, capped.
List<String> rememberTemplate(List<String> recent, String templateId) => [
  templateId,
  ...recent.where((t) => t != templateId).take(EditorState.maxRecentTemplates - 1),
];

/// Every semantic edit is sent against the revision Studio currently holds;
/// the daemon refuses it if the project has moved on.  No effect when
/// disconnected or without a project.
Transition sendEdit(AppState s, pb.EditOp op) => _whenProject(s, () {
  if (!s.isSystem) return Transition(pending(s), [ApplyEdit(baseRevision: s.revision, op: op)]);
  // On a system project a flat edit is an edit of the design in view: the
  // system's own design, or the body of the open component.
  final sop = switch (s.editor.context) {
    SystemContext() => pb.SystemEditOp(base: op),
    ComponentContext(:final id) => pb.SystemEditOp(
      editComponentBody: pb.EditComponentBody(component: Int64(id), op: op),
    ),
  };
  return Transition(pending(s), [ApplySystemEdit(baseRevision: s.revision, op: sop)]);
});

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
  final key = draftKey(root, s.editor.context);
  final dirty = dirtyDrafts(s);
  final next = {...s.editor.stashedDrafts};
  if (dirty.isEmpty) {
    next.remove(key);
  } else {
    next[key] = dirty;
  }
  return next;
}

AppState pending(AppState s) =>
    s.copyWith(editor: s.editor.copyWith(pendingRequests: s.editor.pendingRequests + 1));

int decPending(AppState s) => s.editor.pendingRequests > 0 ? s.editor.pendingRequests - 1 : 0;

/// Accept a projection only if it is at least as new as what we hold for the
/// same project.  Responses for an older revision are discarded — this is
/// the stale-result rule of the protocol.
Transition projectReceived(
  AppState s,
  pb.ProjectProjection incoming,
  pb.EditOutcome? outcome,
  bool fromRequest,
) {
  final current = s.flat;
  final sameProject = current != null && current.rootPath == incoming.rootPath;
  final pendingCount = fromRequest ? decPending(s) : s.editor.pendingRequests;
  if (sameProject && incoming.revision < current.revision) {
    return Transition(s.copyWith(editor: s.editor.copyWith(pendingRequests: pendingCount)));
  }
  final isSystem =
      incoming.kind == pb.ProjectKind.PROJECT_KIND_SYSTEM ||
      incoming.kind == pb.ProjectKind.PROJECT_KIND_TEXT;
  // A system project's system is fetched with the project and again
  // whenever the flat design moved without it (undo, redo, another
  // client); until it arrives the canvas waits rather than showing the
  // derived flat design.
  var system = sameProject && s.system != null && s.system!.revision == incoming.revision
      ? s.system
      : null;
  // A save answers with the same revision and `dirty = false`; the system
  // view's own dirty flag follows.
  if (system != null && !incoming.dirty && system.dirty) {
    system = system.deepCopy()..dirty = false;
  }
  final context = sameProject ? s.editor.context : const SystemContext();
  final view = viewProjection(incoming, system, context);
  final needsSystem = isSystem && system == null;
  // Layout: the daemon's copy is authoritative on open; afterwards Studio is
  // the author and only merges in positions it does not know yet.
  final stored = layoutFromPb(incoming.layout);
  ContextLayout merged(ContextLayout stored, ContextLayout own) => ContextLayout(
    nodes: {...stored.nodes, ...own.nodes},
    groups: {...stored.groups, ...own.groups},
    viewport: own.viewport ?? stored.viewport,
  );
  final layouts = sameProject
      ? CanvasLayout(
          system: merged(stored.system, s.editor.layouts.system),
          components: {
            ...stored.components,
            for (final e in s.editor.layouts.components.entries)
              e.key: merged(stored.components[e.key] ?? const ContextLayout(), e.value),
          },
        )
      : stored;
  var layout = layouts.of(context).nodes;
  // Create-then-rename: the concept a template insertion created lands
  // where the designer pointed, is selected, and opens for naming.
  final insert = s.editor.pendingInsert;
  final created =
      fromRequest && sameProject && insert != null && outcome != null && outcome.hasCreatedConcept()
      ? NodeRef.concept(outcome.createdConcept.toInt())
      : null;
  final dropped = created == null ? null : insert?.position;
  final placed = dropped != null;
  var layoutsOut = layouts;
  if (created != null && dropped != null) {
    layout = {...layout, created: dropped};
    layoutsOut = layouts.withNodes(context, layout);
  }
  final next = s.copyWith(
    project: view,
    flat: incoming,
    system: system,
    clearSystem: system == null,
  );
  final selection = created != null
      ? ConceptSelected(created.id)
      : selectionStillValid(next, s.editor.selection)
      ? s.editor.selection
      : const NoSelection();
  // An inline rename survives pushed projections (the daemon echoes every
  // commit) as long as its node still exists.
  final renaming = created ?? s.editor.renaming;
  final renamingValid = renaming != null && nodeExists(next, renaming);
  final recent = sameProject ? s.recent : _remember(s.recent, incoming);
  final analysisStillValid = s.analysis != null && s.analysis!.revision == incoming.revision;
  // Drafts: rebased on every new revision; restored from the stash when a
  // project is (re)opened.  Same revision (a save) changes nothing.
  final stashed = s.editor.stashedDrafts;
  final key = draftKey(incoming.rootPath, context);
  final ({Map<int, DefinitionDraft> drafts, List<Effect> effects}) drafts = !sameProject
      ? rebaseDrafts(stashed[key] ?? const {}, view, component: s.editor.componentScope)
      : incoming.revision == current.revision
      ? (drafts: s.editor.drafts, effects: const <Effect>[])
      : rebaseDrafts(s.editor.drafts, view, component: s.editor.componentScope);
  final editor = sameProject && incoming.revision == current.revision
      ? s.editor
      : withoutTooling(s.editor)
            .copyWith(simulation: simulationAfterRevision(s.editor.simulation, incoming));
  return Transition(
        next.copyWith(
          recent: recent,
          clearAnalysis: !analysisStillValid,
          editor: editor.copyWith(
            // the system is part of the project: its fetch is pending too
            pendingRequests: pendingCount + (needsSystem ? 1 : 0),
            selection: selection,
            layout: layout,
            layouts: layoutsOut,
            context: context,
            lastOutcome: outcome,
            drafts: drafts.drafts,
            stashedDrafts: sameProject ? stashed : ({...stashed}..remove(key)),
            clearPendingInsert: fromRequest,
            renaming: renamingValid ? renaming : null,
            clearRenaming: !renamingValid,
            clearExtraction: !sameProject,
            clearPendingBind: !sameProject,
          ),
        ),
        // A freshly opened project needs a subscription for pushed changes, an
        // analysis of what was just opened, and goes to the top of Recent.
        // After an edit the daemon pushes AnalysisReady on its own.  A
        // system project also needs its system and its system analysis at
        // every new revision.
        [
          if (!sameProject) ...[
            const SubscribeProject(),
            const RunAnalysis(),
            SaveRecentProjects(recent),
          ],
          if (needsSystem) const GetSystem(),
          if (isSystem && (!sameProject || incoming.revision != current.revision))
            const RunSystemAnalysis(),
          if (placed) SetLayout(layoutToPb(layoutsOut)),
          ...drafts.effects,
        ],
      )
      .thenQueued(fromRequest && sameProject ? outcome : null)
      .thenActions(changed: !sameProject || incoming.revision != current.revision)
      .thenDeployment(changed: !sameProject || incoming.revision != current.revision);
}

extension on Transition {
  /// A deployment answer is about a revision; a new one drops it and asks
  /// again for the chosen board.
  Transition thenDeployment({required bool changed}) {
    if (!changed) return this;
    final t = deploymentAfterRevision(state);
    return Transition(t.state, [...effects, ...t.effects]);
  }

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

List<String> _appendLog(List<String> log, String line) {
  final next = [...log, line];
  return next.length > _maxLogLines ? next.sublist(next.length - _maxLogLines) : next;
}

extension<T> on T {
  R let<R>(R Function(T) f) => f(this);
}
