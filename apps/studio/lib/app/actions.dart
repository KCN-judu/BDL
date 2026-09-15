/// Everything that can change application state.
///
/// [UserAction]s originate from widgets; [ResponseAction]s originate from the
/// effect executor (daemon responses, process events).  Both go through the
/// same pure reducer.
library;

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
  const CreateConceptRequested({required this.name, this.description = ''});
  final String name;
  final String description;
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
