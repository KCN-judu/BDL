/// Immutable application state.
///
/// Three categories, kept apart on purpose (docs/ARCHITECTURE.md §Studio):
///
/// * [AppState.project]  — the *semantic projection* the compiler sent.  Studio
///   never computes semantic facts; it renders this.
/// * [AppState.editor]   — editor interaction state (selection, open panels).
/// * [AppState.render]   — ephemeral rendering state (drag, hover, zoom).
library;

import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

@immutable
sealed class DaemonConnection {
  const DaemonConnection();
}

class Disconnected extends DaemonConnection {
  const Disconnected();
}

class Connecting extends DaemonConnection {
  const Connecting(this.executable);
  final String executable;
}

class Connected extends DaemonConnection {
  const Connected({required this.executable, required this.handshake});
  final String executable;
  final pb.HandshakeResponse handshake;
}

class ConnectionFailed extends DaemonConnection {
  const ConnectionFailed(this.reason);
  final String reason;
}

/// What is selected on the canvas / in the lists.
@immutable
sealed class Selection {
  const Selection();
}

class NoSelection extends Selection {
  const NoSelection();
}

class ConceptSelected extends Selection {
  const ConceptSelected(this.id);
  final int id;
}

class MappingSelected extends Selection {
  const MappingSelected(this.id);
  final int id;
}

@immutable
class EditorState {
  const EditorState({
    this.selection = const NoSelection(),
    this.pendingRequests = 0,
    this.lastError,
  });

  final Selection selection;

  /// Requests sent to the daemon and not yet answered.
  final int pendingRequests;

  /// The most recent request failure, shown until dismissed.
  final UserFacingError? lastError;

  EditorState copyWith({
    Selection? selection,
    int? pendingRequests,
    UserFacingError? lastError,
    bool clearError = false,
  }) {
    return EditorState(
      selection: selection ?? this.selection,
      pendingRequests: pendingRequests ?? this.pendingRequests,
      lastError: clearError ? null : (lastError ?? this.lastError),
    );
  }
}

@immutable
class UserFacingError {
  const UserFacingError({required this.code, required this.message, this.details = ''});
  final String code;
  final String message;
  final String details;
}

/// Ephemeral rendering state.  Nothing here is persisted or sent anywhere.
@immutable
class RenderState {
  const RenderState({this.daemonLog = const []});

  /// Last lines of the daemon's stderr, for the expert view.
  final List<String> daemonLog;

  RenderState copyWith({List<String>? daemonLog}) =>
      RenderState(daemonLog: daemonLog ?? this.daemonLog);
}

@immutable
class AppState {
  const AppState({
    this.connection = const Disconnected(),
    this.project,
    this.editor = const EditorState(),
    this.render = const RenderState(),
  });

  final DaemonConnection connection;

  /// Semantic projection of the open project, owned by the compiler.  `null`
  /// when no project is open.
  final pb.ProjectProjection? project;
  final EditorState editor;
  final RenderState render;

  int get revision => project?.revision.toInt() ?? -1;

  AppState copyWith({
    DaemonConnection? connection,
    pb.ProjectProjection? project,
    bool clearProject = false,
    EditorState? editor,
    RenderState? render,
  }) {
    return AppState(
      connection: connection ?? this.connection,
      project: clearProject ? null : (project ?? this.project),
      editor: editor ?? this.editor,
      render: render ?? this.render,
    );
  }
}
