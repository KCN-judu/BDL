/// Immutable application state.
///
/// Three categories, kept apart on purpose (docs/ARCHITECTURE.md §Studio):
///
/// * [AppState.project]  — the *semantic projection* the compiler sent.  Studio
///   never computes semantic facts; it renders this.
/// * [AppState.editor]   — editor interaction state (selection, open panels).
/// * [AppState.render]   — ephemeral rendering state (drag, hover, zoom).
library;

import 'dart:ui' show Offset;

import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// The workflow pages, in workflow order (docs/STUDIO_UI.md §1).
enum StudioPage { design, simulate, deploy, monitor }

enum NodeKind { concept, mapping }

/// A node on the canvas, identified by kind + stable id.
@immutable
class NodeRef {
  const NodeRef(this.kind, this.id);
  const NodeRef.concept(int id) : this(NodeKind.concept, id);
  const NodeRef.mapping(int id) : this(NodeKind.mapping, id);
  final NodeKind kind;
  final int id;

  @override
  bool operator ==(Object other) => other is NodeRef && other.kind == kind && other.id == id;
  @override
  int get hashCode => Object.hash(kind, id);
  @override
  String toString() => '${kind.name}#$id';
}

/// A project the user opened before.  App-level preference, not project data.
@immutable
class RecentProject {
  const RecentProject({required this.path, required this.name, required this.lastOpened});
  final String path;
  final String name;
  final DateTime lastOpened;

  Map<String, Object> toJson() => {
    'path': path,
    'name': name,
    'last_opened': lastOpened.toUtc().toIso8601String(),
  };

  static RecentProject? fromJson(Object? json) {
    if (json is! Map) return null;
    final path = json['path'];
    final name = json['name'];
    final when = DateTime.tryParse(json['last_opened']?.toString() ?? '');
    if (path is! String || name is! String || when == null) return null;
    return RecentProject(path: path, name: name, lastOpened: when);
  }
}

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
    this.page = StudioPage.design,
    this.selection = const NoSelection(),
    this.layout = const {},
    this.pendingRequests = 0,
    this.lastError,
    this.lastOutcome,
    this.pickerUnavailable = false,
  });

  final StudioPage page;
  final Selection selection;

  /// Canvas positions.  Studio authors these; the daemon stores them.  Never
  /// semantics (ADR-0003).
  final Map<NodeRef, Offset> layout;

  /// Requests sent to the daemon and not yet answered.
  final int pendingRequests;

  /// The most recent request failure, shown until dismissed.
  final UserFacingError? lastError;

  /// Refinement/edit classification of the last committed edit, shown in
  /// the inspector so the paper's distinction is visible where one acts.
  final pb.EditOutcome? lastOutcome;

  /// The OS file dialog could not be shown (e.g. Studio launched from a
  /// sandboxed host); the welcome screen then offers typing a path.
  final bool pickerUnavailable;

  EditorState copyWith({
    StudioPage? page,
    Selection? selection,
    Map<NodeRef, Offset>? layout,
    int? pendingRequests,
    UserFacingError? lastError,
    bool clearError = false,
    pb.EditOutcome? lastOutcome,
    bool clearOutcome = false,
    bool? pickerUnavailable,
  }) {
    return EditorState(
      page: page ?? this.page,
      selection: selection ?? this.selection,
      layout: layout ?? this.layout,
      pendingRequests: pendingRequests ?? this.pendingRequests,
      lastError: clearError ? null : (lastError ?? this.lastError),
      lastOutcome: clearOutcome ? null : (lastOutcome ?? this.lastOutcome),
      pickerUnavailable: pickerUnavailable ?? this.pickerUnavailable,
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
    this.recent = const [],
    this.editor = const EditorState(),
    this.render = const RenderState(),
  });

  final DaemonConnection connection;

  /// Recently opened projects, newest first (app preference, persisted by
  /// the effect executor).
  final List<RecentProject> recent;

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
    List<RecentProject>? recent,
    EditorState? editor,
    RenderState? render,
  }) {
    return AppState(
      connection: connection ?? this.connection,
      project: clearProject ? null : (project ?? this.project),
      recent: recent ?? this.recent,
      editor: editor ?? this.editor,
      render: render ?? this.render,
    );
  }
}
