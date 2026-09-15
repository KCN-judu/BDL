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

class AttachFormulaRequested extends UserAction {
  const AttachFormulaRequested({required this.mappingId, required this.source});
  final int mappingId;
  final String source;
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

/// `source == null` detaches the definition.
class ReplaceDefinitionRequested extends UserAction {
  const ReplaceDefinitionRequested({required this.mappingId, required this.source});
  final int mappingId;
  final String? source;
}

class DeleteMappingRequested extends UserAction {
  const DeleteMappingRequested(this.id);
  final int id;
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
