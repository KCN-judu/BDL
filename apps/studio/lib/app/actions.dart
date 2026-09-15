/// Everything that can change application state.
///
/// [UserAction]s originate from widgets; [ResponseAction]s originate from the
/// effect executor (daemon responses, process events).  Both go through the
/// same pure reducer.
library;

import 'dart:ui' show Offset;

import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'state.dart';

@immutable
sealed class AppAction {
  const AppAction();
}

// ---------------------------------------------------------------------------
// User actions
// ---------------------------------------------------------------------------

sealed class UserAction extends AppAction {
  const UserAction();
}

class AppStarted extends UserAction {
  const AppStarted();
}

class ConnectRequested extends UserAction {
  const ConnectRequested();
}

/// User chose "Open Project…": ask the OS for a folder.
class OpenProjectPickRequested extends UserAction {
  const OpenProjectPickRequested();
}

/// User chose "New Project…": ask the OS where to create it.
class NewProjectPickRequested extends UserAction {
  const NewProjectPickRequested();
}

class OpenProjectRequested extends UserAction {
  const OpenProjectRequested(this.rootPath);
  final String rootPath;
}

class NewProjectRequested extends UserAction {
  const NewProjectRequested({required this.rootPath, required this.name});
  final String rootPath;
  final String name;
}

class SaveRequested extends UserAction {
  const SaveRequested();
}

class CloseProjectRequested extends UserAction {
  const CloseProjectRequested();
}

class UndoRequested extends UserAction {
  const UndoRequested();
}

class RedoRequested extends UserAction {
  const RedoRequested();
}

class CreateConceptRequested extends UserAction {
  const CreateConceptRequested({required this.name, this.description = '', this.representation});
  final String name;
  final String description;
  final pb.Representation? representation;
}

class CreateMappingRequested extends UserAction {
  const CreateMappingRequested({required this.name, required this.inputs, required this.output});
  final String name;
  final List<int> inputs;
  final int output;
}

// ---- definition drafts -----------------------------------------------------
//
// The formula editor edits a draft owned by Studio; the project changes only
// on [CommitDefinitionRequested] / [DetachDefinitionRequested].

/// The designer typed in the definition editor.
class DefinitionDraftChanged extends UserAction {
  const DefinitionDraftChanged({required this.mappingId, required this.source});
  final int mappingId;
  final String source;
}

/// Discard the draft; the editor shows the committed definition again.
class DefinitionDraftReverted extends UserAction {
  const DefinitionDraftReverted(this.mappingId);
  final int mappingId;
}

/// After a conflict: take the definition committed meanwhile, drop the draft.
class DefinitionDraftReloaded extends UserAction {
  const DefinitionDraftReloaded(this.mappingId);
  final int mappingId;
}

/// After a conflict: keep the draft, rebase it on the current revision.
class DefinitionDraftKept extends UserAction {
  const DefinitionDraftKept(this.mappingId);
  final int mappingId;
}

/// Commit the draft: attach when the mapping has no definition, replace
/// when it has one — chosen from the committed projection, never by the
/// widget.
class CommitDefinitionRequested extends UserAction {
  const CommitDefinitionRequested(this.mappingId);
  final int mappingId;
}

/// Remove the committed definition; the mapping becomes unresolved again.
class DetachDefinitionRequested extends UserAction {
  const DetachDefinitionRequested(this.mappingId);
  final int mappingId;
}

// ---- semantic tooling in the definition field -------------------------------

/// Ask for completion at a byte [offset] into [source] (⌃Space, or typing
/// while the pop-up is open).
class CompletionRequested extends UserAction {
  const CompletionRequested({required this.mappingId, required this.source, required this.offset});
  final int mappingId;
  final String source;
  final int offset;
}

class CompletionDismissed extends UserAction {
  const CompletionDismissed();
}

/// Move the selection by [delta] rows (wraps).
class CompletionMoved extends UserAction {
  const CompletionMoved(this.delta);
  final int delta;
}

/// Hover a name in the definition field of [mappingId] at a byte [offset]
/// into [source]; `null` offset ends the hover.
class FormulaHoverRequested extends UserAction {
  const FormulaHoverRequested({
    required this.mappingId,
    required this.source,
    required this.offset,
  });
  final int mappingId;
  final String source;
  final int? offset;
}

/// Hover an entity (a canvas node, a library row); `null` ends the hover.
class EntityHoverRequested extends UserAction {
  const EntityHoverRequested(this.entity);
  final pb.EntityRef? entity;
}

class PageSelected extends UserAction {
  const PageSelected(this.page);
  final StudioPage page;
}

class RenameConceptRequested extends UserAction {
  const RenameConceptRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetConceptDescriptionRequested extends UserAction {
  const SetConceptDescriptionRequested({required this.id, required this.description});
  final int id;
  final String description;
}

class SetConceptRepresentationRequested extends UserAction {
  const SetConceptRepresentationRequested({required this.id, required this.representation});
  final int id;
  final pb.Representation? representation;
}

class DeleteConceptRequested extends UserAction {
  const DeleteConceptRequested(this.id);
  final int id;
}

class RenameMappingRequested extends UserAction {
  const RenameMappingRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetMappingDescriptionRequested extends UserAction {
  const SetMappingDescriptionRequested({required this.id, required this.description});
  final int id;
  final String description;
}

class SetMappingSignatureRequested extends UserAction {
  const SetMappingSignatureRequested({
    required this.id,
    required this.inputs,
    required this.output,
  });
  final int id;
  final List<int> inputs;
  final int output;
}

class DeleteMappingRequested extends UserAction {
  const DeleteMappingRequested(this.id);
  final int id;
}

// ---- timing domains, outputs, devices ---------------------------------------

class CreateClockDomainRequested extends UserAction {
  const CreateClockDomainRequested(this.name);
  final String name;
}

class RenameClockDomainRequested extends UserAction {
  const RenameClockDomainRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class DeleteClockDomainRequested extends UserAction {
  const DeleteClockDomainRequested(this.id);
  final int id;
}

/// `clockId == null` makes the mapping domain-agnostic (pure).
class SetMappingClockRequested extends UserAction {
  const SetMappingClockRequested({required this.mappingId, required this.clockId});
  final int mappingId;
  final int? clockId;
}

class CreateOutputRequested extends UserAction {
  const CreateOutputRequested({
    required this.name,
    required this.accepts,
    this.description = '',
    this.clockId,
    this.required = false,
  });
  final String name;
  final String description;
  final int accepts;
  final int? clockId;
  final bool required;
}

class RenameOutputRequested extends UserAction {
  const RenameOutputRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetOutputAcceptsRequested extends UserAction {
  const SetOutputAcceptsRequested({required this.id, required this.accepts});
  final int id;
  final int accepts;
}

class SetOutputClockRequested extends UserAction {
  const SetOutputClockRequested({required this.id, required this.clockId});
  final int id;
  final int? clockId;
}

class SetOutputRequiredRequested extends UserAction {
  const SetOutputRequiredRequested({required this.id, required this.required});
  final int id;
  final bool required;
}

class DeleteOutputRequested extends UserAction {
  const DeleteOutputRequested(this.id);
  final int id;
}

/// Connect a mapping as the driver of an output, or (`outputId == null`)
/// disconnect it from whatever it drives.
class SetMappingDriveRequested extends UserAction {
  const SetMappingDriveRequested({required this.mappingId, required this.outputId});
  final int mappingId;
  final int? outputId;
}

class CreateDeviceRequested extends UserAction {
  const CreateDeviceRequested({required this.name, required this.kind, this.outputId});
  final String name;
  final pb.DeviceKind kind;
  final int? outputId;
}

class RenameDeviceRequested extends UserAction {
  const RenameDeviceRequested({required this.id, required this.name});
  final int id;
  final String name;
}

class SetDeviceKindRequested extends UserAction {
  const SetDeviceKindRequested({required this.id, required this.kind});
  final int id;
  final pb.DeviceKind kind;
}

class SetDeviceOutputRequested extends UserAction {
  const SetDeviceOutputRequested({required this.id, required this.outputId});
  final int id;
  final int? outputId;
}

class SetDevicePinRequested extends UserAction {
  const SetDevicePinRequested({required this.id, required this.index, required this.resource});
  final int id;
  final int index;
  final String? resource;
}

class DeleteDeviceRequested extends UserAction {
  const DeleteDeviceRequested(this.id);
  final int id;
}

// ---- semantic actions ---------------------------------------------------------

/// Ask the service which fixes and context actions it offers for the
/// selected entity.
class SemanticActionsRequested extends UserAction {
  const SemanticActionsRequested(this.entity);
  final pb.EntityRef entity;
}

/// Apply a ready action's model edits (one revisioned edit each, in order),
/// or the chosen option of a needs-choice action.
class SemanticActionApplied extends UserAction {
  const SemanticActionApplied({required this.actionId, this.option});
  final String actionId;
  final int? option;
}

/// Delete whatever is selected.
class DeleteSelectionRequested extends UserAction {
  const DeleteSelectionRequested();
}

/// A node finished being dragged; commit its position (layout, not semantics).
class NodeMoved extends UserAction {
  const NodeMoved(this.node, this.position);
  final NodeRef node;
  final Offset position;
}

/// A link was drawn from a concept's value socket into a mapping's inputs.
class LinkConceptToMappingInput extends UserAction {
  const LinkConceptToMappingInput({required this.conceptId, required this.mappingId});
  final int conceptId;
  final int mappingId;
}

/// A link was drawn from a mapping's output socket to a concept.
class LinkMappingOutputToConcept extends UserAction {
  const LinkMappingOutputToConcept({required this.mappingId, required this.conceptId});
  final int mappingId;
  final int conceptId;
}

/// A concept was disconnected from a mapping's inputs.
class UnlinkMappingInput extends UserAction {
  const UnlinkMappingInput({required this.mappingId, required this.conceptId});
  final int mappingId;
  final int conceptId;
}

class RemoveRecentRequested extends UserAction {
  const RemoveRecentRequested(this.path);
  final String path;
}

class SelectionChanged extends UserAction {
  const SelectionChanged(this.selection);
  final Selection selection;
}

class ErrorDismissed extends UserAction {
  const ErrorDismissed();
}

// ---------------------------------------------------------------------------
// Response actions (from effects)
// ---------------------------------------------------------------------------

sealed class ResponseAction extends AppAction {
  const ResponseAction();
}

class DaemonConnected extends ResponseAction {
  const DaemonConnected({required this.executable, required this.handshake});
  final String executable;
  final pb.HandshakeResponse handshake;
}

class DaemonConnectionFailed extends ResponseAction {
  const DaemonConnectionFailed(this.reason);
  final String reason;
}

class DaemonExited extends ResponseAction {
  const DaemonExited(this.exitCode);
  final int exitCode;
}

class DaemonLogged extends ResponseAction {
  const DaemonLogged(this.line);
  final String line;
}

/// A project projection arrived (from open/init/save/get/edit responses or a
/// ProjectChanged event).  Carries the revision; stale ones are dropped.
class ProjectReceived extends ResponseAction {
  const ProjectReceived(this.project, {this.outcome, this.fromRequest = true});
  final pb.ProjectProjection project;
  final pb.EditOutcome? outcome;

  /// True when this answers a request Studio sent (and counted as pending);
  /// false for unsolicited `ProjectChanged` events.
  final bool fromRequest;
}

/// The OS picker returned instantly with nothing — it was refused, not
/// cancelled (view-bridge failure under a sandboxed host).
class PickerUnavailable extends ResponseAction {
  const PickerUnavailable();
}

/// A compiler analysis arrived (response or pushed event).  Revision-tagged;
/// the reducer keeps it only if it matches the project it holds.
class AnalysisReceived extends ResponseAction {
  const AnalysisReceived(this.analysis, {this.fromRequest = false});
  final pb.ProjectAnalysis analysis;
  final bool fromRequest;
}

/// The compiler's verdict on a draft.  Tagged with revision, mapping and
/// generation; the reducer keeps it only if all three still match.
class DraftAnalysisReceived extends ResponseAction {
  const DraftAnalysisReceived(this.result);
  final pb.DefinitionDraftAnalysis result;
}

/// A draft check could not be completed.  `draft.stale_revision` means a
/// newer projection is on its way and will re-ask; anything else leaves the
/// draft unchecked (never discarded).
class DraftAnalysisFailed extends ResponseAction {
  const DraftAnalysisFailed({
    required this.mappingId,
    required this.generation,
    required this.code,
    required this.message,
  });
  final int mappingId;
  final int generation;
  final String code;
  final String message;
}

class CompletionReceived extends ResponseAction {
  const CompletionReceived({required this.generation, required this.result});
  final int generation;
  final pb.DraftCompletionResponse result;
}

class HoverReceived extends ResponseAction {
  const HoverReceived({required this.generation, required this.result});
  final int generation;
  final pb.DraftHoverResponse result;
}

/// A tooling request (completion, hover) failed; the pop-up or card just
/// does not appear.  Never a banner.
class ToolingFailed extends ResponseAction {
  const ToolingFailed({required this.generation, required this.code, required this.message});
  final int generation;
  final String code;
  final String message;
}

class SemanticActionsReceived extends ResponseAction {
  const SemanticActionsReceived({required this.generation, required this.result});
  final int generation;
  final pb.SemanticActionsResponse result;
}

class RecentProjectsLoaded extends ResponseAction {
  const RecentProjectsLoaded(this.recent);
  final List<RecentProject> recent;
}

class ProjectClosed extends ResponseAction {
  const ProjectClosed();
}

class RequestSucceeded extends ResponseAction {
  const RequestSucceeded();
}

class RequestFailed extends ResponseAction {
  const RequestFailed({required this.code, required this.message, this.details = ''});
  final String code;
  final String message;
  final String details;
}
