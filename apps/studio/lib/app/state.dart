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

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// The workflow pages, in workflow order (docs/STUDIO_UI.md §1).
enum StudioPage { design, simulate, deploy, monitor }

enum NodeKind { concept, mapping, output }

/// A node on the canvas, identified by kind + stable id.
@immutable
class NodeRef {
  const NodeRef(this.kind, this.id);
  const NodeRef.concept(int id) : this(NodeKind.concept, id);
  const NodeRef.mapping(int id) : this(NodeKind.mapping, id);
  const NodeRef.output(int id) : this(NodeKind.output, id);
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

class OutputSelected extends Selection {
  const OutputSelected(this.id);
  final int id;
}

/// The semantic actions the service offers for one entity at one revision:
/// the fixes for its diagnostics and its context actions.
@immutable
class SemanticActionsState {
  const SemanticActionsState({
    required this.entity,
    required this.revision,
    required this.generation,
    this.actions = const [],
    this.pending = true,
  });
  final pb.EntityRef entity;
  final int revision;
  final int generation;
  final List<pb.SemanticActionView> actions;
  final bool pending;

  SemanticActionsState copyWith({List<pb.SemanticActionView>? actions, bool? pending}) =>
      SemanticActionsState(
        entity: entity,
        revision: revision,
        generation: generation,
        actions: actions ?? this.actions,
        pending: pending ?? this.pending,
      );
}

/// How far the compiler has got with a draft's current source.
enum DraftCheck {
  /// The source changed since the last verdict; a check is debounced or in
  /// flight.  Nothing is known about this exact text yet.
  checking,

  /// [DefinitionDraft.analysis] is the compiler's verdict for exactly this
  /// source at exactly [DefinitionDraft.baseRevision].
  checked,

  /// The compiler could not be asked (daemon gone, transport failure).  The
  /// source is kept; there is simply no verdict.
  unavailable,
}

/// An uncommitted candidate definition for one mapping.
///
/// The *committed* definition lives in the projection; this is Studio's own
/// editing state and survives every projection or analysis push.  It is
/// dropped only on purpose: a confirmed commit, an explicit revert or
/// reload, the mapping's deletion, or a project close (where dirty drafts
/// are stashed by project path and restored on reopen).
@immutable
class DefinitionDraft {
  const DefinitionDraft({
    required this.mappingId,
    required this.baseRevision,
    required this.baseDefinition,
    required this.source,
    this.generation = 0,
    this.check = DraftCheck.checking,
    this.checkError,
    this.analysis,
    this.parseOk = true,
    this.conflict = false,
    this.pendingCommit,
    this.commitError,
  });

  final int mappingId;

  /// The project revision the draft was last (re)based on.  Every new
  /// revision rebases the draft and asks the compiler again.
  final int baseRevision;

  /// The committed formula the draft started from at [baseRevision]
  /// (`null` when the mapping had none).  Lets a later projection tell
  /// "the definition changed under this draft" from "something else did".
  final String? baseDefinition;

  /// What the designer has typed.  Never lost by a push.
  final String source;

  /// Monotonic per draft; each source change bumps it and only a verdict
  /// carrying the latest generation is accepted (out-of-order responses
  /// cannot overwrite newer state).
  final int generation;

  final DraftCheck check;

  /// Why the check is [DraftCheck.unavailable], in product language.
  final String? checkError;

  /// The compiler's verdict for [source] at [baseRevision], when
  /// [check] is [DraftCheck.checked].
  final pb.MappingAnalysis? analysis;

  /// False when [analysis] says the source did not parse.
  final bool parseOk;

  /// The committed definition changed while this draft was dirty.  Neither
  /// side is overwritten; the designer chooses (reload or keep).
  final bool conflict;

  /// The source sent in a commit that has not been answered yet; cleared
  /// when the projection confirms it or the request fails.
  final String? pendingCommit;

  /// The last commit's failure, shown next to the editor; cleared on the
  /// next source change or commit.
  final String? commitError;

  /// True when the source differs from the definition committed now.
  bool dirtyAgainst(String? committed) => source != (committed ?? '');

  DefinitionDraft copyWith({
    int? baseRevision,
    String? baseDefinition,
    bool clearBaseDefinition = false,
    String? source,
    int? generation,
    DraftCheck? check,
    String? checkError,
    bool clearCheckError = false,
    pb.MappingAnalysis? analysis,
    bool clearAnalysis = false,
    bool? parseOk,
    bool? conflict,
    String? pendingCommit,
    bool clearPendingCommit = false,
    String? commitError,
    bool clearCommitError = false,
  }) {
    return DefinitionDraft(
      mappingId: mappingId,
      baseRevision: baseRevision ?? this.baseRevision,
      baseDefinition: clearBaseDefinition ? null : (baseDefinition ?? this.baseDefinition),
      source: source ?? this.source,
      generation: generation ?? this.generation,
      check: check ?? this.check,
      checkError: clearCheckError ? null : (checkError ?? this.checkError),
      analysis: clearAnalysis ? null : (analysis ?? this.analysis),
      parseOk: parseOk ?? this.parseOk,
      conflict: conflict ?? this.conflict,
      pendingCommit: clearPendingCommit ? null : (pendingCommit ?? this.pendingCommit),
      commitError: clearCommitError ? null : (commitError ?? this.commitError),
    );
  }
}

/// A completion pop-up over the definition field: the service's candidates
/// for one (mapping, source, byte offset), newest request wins.
@immutable
class CompletionState {
  const CompletionState({
    required this.mappingId,
    required this.generation,
    required this.source,
    required this.offset,
    this.items = const [],
    this.selected = 0,
    this.pending = true,
  });
  final int mappingId;

  /// Request tag; a response with another generation is ignored.
  final int generation;

  /// The text and byte offset the candidates are for.
  final String source;
  final int offset;

  /// In the service's order (relevance, then label) — never re-sorted here.
  final List<pb.DraftCompletionItem> items;
  final int selected;
  final bool pending;

  CompletionState copyWith({
    int? generation,
    String? source,
    int? offset,
    List<pb.DraftCompletionItem>? items,
    int? selected,
    bool? pending,
  }) => CompletionState(
    mappingId: mappingId,
    generation: generation ?? this.generation,
    source: source ?? this.source,
    offset: offset ?? this.offset,
    items: items ?? this.items,
    selected: selected ?? this.selected,
    pending: pending ?? this.pending,
  );
}

/// A hover card over a formula name (or, from the canvas, over an entity).
@immutable
class HoverState {
  const HoverState({required this.generation, this.mappingId, this.offset, this.entity, this.card});
  final int generation;

  /// Formula hover: the mapping whose draft text is hovered, at a byte offset.
  final int? mappingId;
  final int? offset;

  /// Entity hover (canvas / library rows).
  final pb.EntityRef? entity;

  /// The service's card, once it arrived; `null` while pending.
  final pb.DraftHoverResponse? card;

  HoverState copyWith({pb.DraftHoverResponse? card}) => HoverState(
    generation: generation,
    mappingId: mappingId,
    offset: offset,
    entity: entity,
    card: card ?? this.card,
  );
}

/// The Deploy page: which board is being asked about, and the compiler's
/// target-relative answer.  Never part of the design; the board choice is
/// an editor preference for the session.
@immutable
class DeployState {
  const DeployState({
    this.targets = const [],
    this.targetsLoaded = false,
    this.targetId,
    this.analysis,
    this.pending = false,
    this.generation = 0,
    this.error,
  });

  /// The boards bdld knows, as it lists them.
  final List<pb.TargetView> targets;
  final bool targetsLoaded;

  /// The board being asked about; `null` until chosen.
  final String? targetId;

  /// The last analysis kept — only while its revision is the project's and
  /// its target is the chosen one; dropped otherwise.
  final pb.DeploymentAnalysis? analysis;
  final bool pending;

  /// Request tag; only the latest answer is applied.
  final int generation;

  /// Why the last analysis could not be made (product language).
  final String? error;

  pb.TargetView? get target => targets.where((t) => t.id == targetId).firstOrNull;

  DeployState copyWith({
    List<pb.TargetView>? targets,
    bool? targetsLoaded,
    String? targetId,
    bool clearTarget = false,
    pb.DeploymentAnalysis? analysis,
    bool clearAnalysis = false,
    bool? pending,
    int? generation,
    String? error,
    bool clearError = false,
  }) => DeployState(
    targets: targets ?? this.targets,
    targetsLoaded: targetsLoaded ?? this.targetsLoaded,
    targetId: clearTarget ? null : (targetId ?? this.targetId),
    analysis: clearAnalysis ? null : (analysis ?? this.analysis),
    pending: pending ?? this.pending,
    generation: generation ?? this.generation,
    error: clearError ? null : (error ?? this.error),
  );
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
    this.drafts = const {},
    this.stashedDrafts = const {},
    this.completion,
    this.hover,
    this.toolingGeneration = 0,
    this.actions,
    this.queuedEdits = const [],
    this.deploy = const DeployState(),
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

  /// Definition drafts of the open project, by mapping id.  A draft exists
  /// only while the designer's text differs from what is committed (or a
  /// commit is being confirmed).
  final Map<int, DefinitionDraft> drafts;

  /// Dirty drafts of projects that were closed, by project path, restored
  /// when that project is opened again.  Nothing typed is discarded by a
  /// close; no modal asks.
  final Map<String, Map<int, DefinitionDraft>> stashedDrafts;

  /// The open completion pop-up, if any (one at a time: the focused field).
  final CompletionState? completion;

  /// The hover card being shown or fetched, if any.
  final HoverState? hover;

  /// Monotonic tag for completion, hover and action requests; only the
  /// latest answer of each is applied.
  final int toolingGeneration;

  /// The service's actions for the selected entity, if asked.
  final SemanticActionsState? actions;

  /// Model edits of a multi-step plan still to send, one per confirmed
  /// revision (an edit is always sent against the revision Studio holds).
  final List<pb.EditOp> queuedEdits;

  final DeployState deploy;

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
    Map<int, DefinitionDraft>? drafts,
    Map<String, Map<int, DefinitionDraft>>? stashedDrafts,
    CompletionState? completion,
    bool clearCompletion = false,
    HoverState? hover,
    bool clearHover = false,
    int? toolingGeneration,
    SemanticActionsState? actions,
    bool clearActions = false,
    List<pb.EditOp>? queuedEdits,
    DeployState? deploy,
  }) {
    return EditorState(
      page: page ?? this.page,
      selection: selection ?? this.selection,
      layout: layout ?? this.layout,
      pendingRequests: pendingRequests ?? this.pendingRequests,
      lastError: clearError ? null : (lastError ?? this.lastError),
      lastOutcome: clearOutcome ? null : (lastOutcome ?? this.lastOutcome),
      pickerUnavailable: pickerUnavailable ?? this.pickerUnavailable,
      drafts: drafts ?? this.drafts,
      stashedDrafts: stashedDrafts ?? this.stashedDrafts,
      completion: clearCompletion ? null : (completion ?? this.completion),
      hover: clearHover ? null : (hover ?? this.hover),
      toolingGeneration: toolingGeneration ?? this.toolingGeneration,
      actions: clearActions ? null : (actions ?? this.actions),
      queuedEdits: queuedEdits ?? this.queuedEdits,
      deploy: deploy ?? this.deploy,
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
    this.analysis,
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

  /// The compiler's analysis of [project] — kept only when its revision is
  /// the project's; `null` while a newer revision is still being analysed.
  final pb.ProjectAnalysis? analysis;
  final EditorState editor;
  final RenderState render;

  int get revision => project?.revision.toInt() ?? -1;

  /// Analysis of one mapping at the current revision, if available.
  pb.MappingAnalysis? mappingAnalysis(int id) =>
      analysis?.mappings.where((m) => m.id.toInt() == id).firstOrNull;

  pb.MappingView? mapping(int id) => project?.mappings.where((m) => m.id.toInt() == id).firstOrNull;

  /// The committed formula of a mapping, `null` when it has none.
  String? committedDefinition(int id) {
    final m = mapping(id);
    return m != null && m.hasDefinition() ? m.definition.formula : null;
  }

  DefinitionDraft? draft(int id) => editor.drafts[id];

  pb.OutputView? output(int id) => project?.outputs.where((o) => o.id.toInt() == id).firstOrNull;

  pb.ClockView? clock(int id) => project?.clocks.where((c) => c.id.toInt() == id).firstOrNull;

  String? clockName(int? id) => id == null ? null : clock(id)?.name;

  /// The output pass verdict for one sink at the current revision.
  pb.OutputAnalysis? outputAnalysis(int id) =>
      analysis?.outputs.where((o) => o.id.toInt() == id).firstOrNull;

  /// The selected entity, by identity, for service queries.
  pb.EntityRef? get selectedEntity => switch (editor.selection) {
    NoSelection() => null,
    ConceptSelected(:final id) => pb.EntityRef(conceptId: Int64(id)),
    MappingSelected(:final id) => pb.EntityRef(mappingId: Int64(id)),
    OutputSelected(:final id) => pb.EntityRef(outputId: Int64(id)),
  };

  AppState copyWith({
    DaemonConnection? connection,
    pb.ProjectProjection? project,
    bool clearProject = false,
    pb.ProjectAnalysis? analysis,
    bool clearAnalysis = false,
    List<RecentProject>? recent,
    EditorState? editor,
    RenderState? render,
  }) {
    return AppState(
      connection: connection ?? this.connection,
      project: clearProject ? null : (project ?? this.project),
      analysis: (clearProject || clearAnalysis) ? null : (analysis ?? this.analysis),
      recent: recent ?? this.recent,
      editor: editor ?? this.editor,
      render: render ?? this.render,
    );
  }
}
