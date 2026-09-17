/// Immutable application state.
///
/// Three categories, kept apart on purpose (docs/architecture/overview.md §Studio):
///
/// * [AppState.project]  — the *semantic projection* the compiler sent.  Studio
///   never computes semantic facts; it renders this.
/// * [AppState.editor]   — editor interaction state (selection, open panels).
/// * [AppState.render]   — ephemeral rendering state (drag, hover, zoom).
library;

import 'dart:ui' show Offset, Rect;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/foundation.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;

/// The workflow pages, in workflow order (docs/architecture/studio-ui.md §1).
enum StudioPage { design, simulate, deploy, monitor }

/// How the Design page shows the one open project (ADR-0023 §3): as the
/// graph, as its source files, or both side by side.  A view, never a
/// kind of project.
enum DesignView { design, code, split }

/// The source files of the open project as the Code view shows them: the
/// daemon's text with every committed graph edit written back, plus a
/// draft the semantic project has not accepted (ADR-0023 §5).
@immutable
class SourcesState {
  const SourcesState({
    this.revision = -1,
    this.files = const [],
    this.diagnostics = const [],
    this.openPath,
    this.buffer,
    this.sent,
    this.retry,
  });

  /// The project revision [files] describe; -1 before the first answer.
  final int revision;
  final List<pb.SourceFileView> files;

  /// Why a draft does not build (empty when every file is committed).
  final List<pb.SourceDiagnostic> diagnostics;

  /// The file in the editor.
  final String? openPath;

  /// Text typed in the editor and not yet sent: the editor's own truth
  /// while the designer types.  Null when the editor shows [files].
  final String? buffer;

  /// The text of the edit in flight, to tell its answer from a later one.
  final String? sent;

  /// An edit refused as stale, resent once the sources catch up.
  final String? retry;

  pb.SourceFileView? file(String? path) => files.where((f) => f.path == path).firstOrNull;

  pb.SourceFileView? get open => file(openPath);

  /// The text the editor shows for the open file.
  String get text => buffer ?? open?.text ?? '';

  /// Whether the graph shows an older revision than the text: some file is
  /// a draft the model did not accept.
  bool get outOfSync => files.any((f) => f.draft);

  /// Diagnostics of one file, errors before open ones, in text order.
  List<pb.SourceDiagnostic> diagnosticsOf(String? path) =>
      [...diagnostics.where((d) => d.path == path)]..sort((a, b) {
        if (a.open != b.open) return a.open ? 1 : -1;
        return a.start.compareTo(b.start);
      });

  SourcesState copyWith({
    int? revision,
    List<pb.SourceFileView>? files,
    List<pb.SourceDiagnostic>? diagnostics,
    String? openPath,
    String? buffer,
    bool clearBuffer = false,
    String? sent,
    bool clearSent = false,
    String? retry,
    bool clearRetry = false,
  }) => SourcesState(
    revision: revision ?? this.revision,
    files: files ?? this.files,
    diagnostics: diagnostics ?? this.diagnostics,
    openPath: openPath ?? this.openPath,
    buffer: clearBuffer ? null : (buffer ?? this.buffer),
    sent: clearSent ? null : (sent ?? this.sent),
    retry: clearRetry ? null : (retry ?? this.retry),
  );
}

/// Canvas node kinds.  [instance] is a component instance of a system
/// (rendered from its ports' contracts, never its body); [group] is a
/// collapsed behaviour group (a picture of its members, never a node the
/// compiler knows).
enum NodeKind { concept, mapping, output, instance, group }

/// A node on the canvas, identified by kind + stable id.
@immutable
class NodeRef {
  const NodeRef(this.kind, this.id);
  const NodeRef.concept(int id) : this(NodeKind.concept, id);
  const NodeRef.mapping(int id) : this(NodeKind.mapping, id);
  const NodeRef.output(int id) : this(NodeKind.output, id);
  const NodeRef.instance(int id) : this(NodeKind.instance, id);
  const NodeRef.group(int id) : this(NodeKind.group, id);
  final NodeKind kind;
  final int id;

  @override
  bool operator ==(Object other) => other is NodeRef && other.kind == kind && other.id == id;
  @override
  int get hashCode => Object.hash(kind, id);
  @override
  String toString() => '${kind.name}#$id';
}

/// Which design the canvas shows (docs/architecture/studio-ui.md §11).  A flat project
/// has only the system context (its own design); a system project shows
/// its top level — shared concepts, domains, sinks, top-level
/// relationships, component instances — or the *source* of one component
/// (its body, in the component's own names and identities).
@immutable
sealed class DesignContext {
  const DesignContext();
}

class SystemContext extends DesignContext {
  const SystemContext();
  @override
  bool operator ==(Object other) => other is SystemContext;
  @override
  int get hashCode => 1;
}

class ComponentContext extends DesignContext {
  const ComponentContext(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is ComponentContext && other.id == id;
  @override
  int get hashCode => Object.hash(2, id);
}

/// A behaviour group's picture: where its collapsed box stands, how big it
/// is, whether it is collapsed.  Layout only — the members are the
/// system's.
@immutable
class GroupBox {
  const GroupBox({required this.rect, this.collapsed = false});
  final Rect rect;
  final bool collapsed;

  GroupBox copyWith({Rect? rect, bool? collapsed}) =>
      GroupBox(rect: rect ?? this.rect, collapsed: collapsed ?? this.collapsed);

  @override
  bool operator ==(Object other) =>
      other is GroupBox && other.rect == rect && other.collapsed == collapsed;
  @override
  int get hashCode => Object.hash(rect, collapsed);
}

/// Where the designer left a canvas: pan and zoom.
@immutable
class CanvasViewport {
  const CanvasViewport({required this.pan, required this.zoom});
  final Offset pan;
  final double zoom;
  @override
  bool operator ==(Object other) =>
      other is CanvasViewport && other.pan == pan && other.zoom == zoom;
  @override
  int get hashCode => Object.hash(pan, zoom);
}

/// One canvas's picture: node positions, group boxes, viewport.  A
/// component's source has its own; coordinates are never shared between
/// canvases.
@immutable
class ContextLayout {
  const ContextLayout({this.nodes = const {}, this.groups = const {}, this.viewport});
  final Map<NodeRef, Offset> nodes;
  final Map<int, GroupBox> groups;
  final CanvasViewport? viewport;

  ContextLayout copyWith({
    Map<NodeRef, Offset>? nodes,
    Map<int, GroupBox>? groups,
    CanvasViewport? viewport,
  }) => ContextLayout(
    nodes: nodes ?? this.nodes,
    groups: groups ?? this.groups,
    viewport: viewport ?? this.viewport,
  );

  @override
  bool operator ==(Object other) =>
      other is ContextLayout &&
      mapEquals(other.nodes, nodes) &&
      mapEquals(other.groups, groups) &&
      other.viewport == viewport;
  @override
  int get hashCode => Object.hash(nodes.length, groups.length, viewport);
}

/// Every canvas of the project: the system (or flat) canvas and one canvas
/// per component body.  Studio authors it; the daemon stores it whole
/// (`SetLayout`).  Never semantics (ADR-0003).
@immutable
class CanvasLayout {
  const CanvasLayout({this.system = const ContextLayout(), this.components = const {}});
  final ContextLayout system;
  final Map<int, ContextLayout> components;

  ContextLayout of(DesignContext c) => switch (c) {
    SystemContext() => system,
    ComponentContext(:final id) => components[id] ?? const ContextLayout(),
  };

  /// The group boxes of the canvas on screen.
  Map<int, GroupBox> groupsOf(DesignContext c) => of(c).groups;

  CanvasLayout withContext(DesignContext c, ContextLayout layout) => switch (c) {
    SystemContext() => CanvasLayout(system: layout, components: components),
    ComponentContext(:final id) => CanvasLayout(
      system: system,
      components: {...components, id: layout},
    ),
  };

  CanvasLayout withNodes(DesignContext c, Map<NodeRef, Offset> nodes) =>
      withContext(c, of(c).copyWith(nodes: nodes));

  CanvasLayout withGroup(DesignContext c, int id, GroupBox box) =>
      withContext(c, of(c).copyWith(groups: {...of(c).groups, id: box}));

  CanvasLayout withoutGroup(DesignContext c, int id) =>
      withContext(c, of(c).copyWith(groups: {...of(c).groups}..remove(id)));

  CanvasLayout withViewport(DesignContext c, CanvasViewport v) =>
      withContext(c, of(c).copyWith(viewport: v));

  CanvasLayout withoutComponent(int id) =>
      CanvasLayout(system: system, components: {...components}..remove(id));
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
  @override
  bool operator ==(Object other) => other is NoSelection;
  @override
  int get hashCode => 0;
}

class ConceptSelected extends Selection {
  const ConceptSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is ConceptSelected && other.id == id;
  @override
  int get hashCode => Object.hash(ConceptSelected, id);
}

class MappingSelected extends Selection {
  const MappingSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is MappingSelected && other.id == id;
  @override
  int get hashCode => Object.hash(MappingSelected, id);
}

class OutputSelected extends Selection {
  const OutputSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is OutputSelected && other.id == id;
  @override
  int get hashCode => Object.hash(OutputSelected, id);
}

// ---- system entities (a system project's own objects) -------------------------

class ComponentSelected extends Selection {
  const ComponentSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is ComponentSelected && other.id == id;
  @override
  int get hashCode => Object.hash(ComponentSelected, id);
}

class InstanceSelected extends Selection {
  const InstanceSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is InstanceSelected && other.id == id;
  @override
  int get hashCode => Object.hash(InstanceSelected, id);
}

/// A port of an instance, on the system canvas.
class PortSelected extends Selection {
  const PortSelected({required this.instance, required this.port});
  final int instance;
  final int port;
  @override
  bool operator ==(Object other) =>
      other is PortSelected && other.instance == instance && other.port == port;
  @override
  int get hashCode => Object.hash(instance, port);
}

class BindingSelected extends Selection {
  const BindingSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is BindingSelected && other.id == id;
  @override
  int get hashCode => Object.hash(BindingSelected, id);
}

class GroupSelected extends Selection {
  const GroupSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is GroupSelected && other.id == id;
  @override
  int get hashCode => Object.hash(GroupSelected, id);
}

/// Several canvas nodes at once (box select, ⇧-click): the inspector offers
/// what applies to all of them — grouping the relationships among them.
class MultiSelected extends Selection {
  const MultiSelected(this.nodes);
  final Set<NodeRef> nodes;
  Iterable<int> get mappings => nodes.where((n) => n.kind == NodeKind.mapping).map((n) => n.id);
  @override
  bool operator ==(Object other) => other is MultiSelected && setEquals(other.nodes, nodes);
  @override
  int get hashCode => Object.hash(MultiSelected, nodes.length);
}

/// A link drawn between two sockets that cannot be made silently: the
/// destination is already bound (a required port takes one source), or
/// the two sides update in different timing domains (the value must be
/// carried across with a stated initial value).  The designer decides.
@immutable
class PendingBind {
  const PendingBind({
    required this.source,
    required this.destination,
    this.replaces,
    this.needsTransport = false,
    this.sourceDomain = '',
    this.destinationDomain = '',
  });
  final pb.PortRefView source;
  final pb.PortRefView destination;

  /// The binding into the destination today, if any.
  final int? replaces;
  final bool needsTransport;
  final String sourceDomain;
  final String destinationDomain;
}

/// The "Package as reusable component" sheet: the designer's choices and
/// the compiler's preview of what they mean.  The preview mutates nothing;
/// only the final edit does.
@immutable
class ExtractionState {
  const ExtractionState({
    required this.groupId,
    this.name = '',
    this.instanceName = '',
    this.keepInternal = const {},
    this.internalizeSinks = const {},
    this.preview,
    this.generation = 0,
    this.pending = true,
    this.error,
    this.memberPositions = const {},
    this.box,
  });
  final int groupId;
  final String name;
  final String instanceName;

  /// Open members the designer keeps internal (default: inputs).
  final Set<int> keepInternal;

  /// Sinks the designer moves into the component (default: external).
  final Set<int> internalizeSinks;
  final pb.ExtractionPreviewView? preview;
  final int generation;
  final bool pending;
  final String? error;

  /// Where the group's members and box stood when packaging was confirmed:
  /// the instance takes the box's place and the component's canvas starts
  /// from the members' positions, whatever arrives first afterwards.
  final Map<NodeRef, Offset> memberPositions;
  final Rect? box;

  ExtractionState copyWith({
    String? name,
    String? instanceName,
    Set<int>? keepInternal,
    Set<int>? internalizeSinks,
    pb.ExtractionPreviewView? preview,
    int? generation,
    bool? pending,
    String? error,
    bool clearError = false,
    Map<NodeRef, Offset>? memberPositions,
    Rect? box,
  }) => ExtractionState(
    groupId: groupId,
    name: name ?? this.name,
    instanceName: instanceName ?? this.instanceName,
    keepInternal: keepInternal ?? this.keepInternal,
    internalizeSinks: internalizeSinks ?? this.internalizeSinks,
    preview: preview ?? this.preview,
    generation: generation ?? this.generation,
    pending: pending ?? this.pending,
    error: clearError ? null : (error ?? this.error),
    memberPositions: memberPositions ?? this.memberPositions,
    box: box ?? this.box,
  );

  pb.ExtractionChoices get choices => pb.ExtractionChoices(
    name: name,
    instanceName: instanceName,
    keepInternal: keepInternal.map(Int64.new),
    internalizeSinks: internalizeSinks.map(Int64.new),
  );
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

/// What the left sidebar shows: the project's own objects, or the concept
/// libraries to insert from.
enum SidebarTab { project, library }

/// An insertion from a concept template that the daemon has not answered
/// yet.  When the projection with the created concept arrives, the node is
/// placed at [position] (or auto-placed), selected, and its name opened for
/// editing — the create-then-rename flow.
@immutable
class PendingInsert {
  const PendingInsert({required this.templateId, this.position});
  final String templateId;

  /// Scene position of the drop / right-click; `null` for a keyboard or
  /// panel insertion (auto-placed).
  final Offset? position;
}

/// The Simulate page: Studio's input trace and schedule (UI state), and the
/// evaluator's trace for the revision on screen.  A run belongs to one
/// revision; any commit drops it.  Nothing here evaluates anything.
@immutable
class SimulationState {
  const SimulationState({
    this.revision,
    this.nextTick = 0,
    this.samples = const [],
    this.inputs = const {},
    this.current = const {},
    this.periods = const {},
    this.pending = false,
    this.generation = 0,
    this.error,
    this.failure,
  });

  /// The revision the samples were produced for; `null` when there is no run.
  final int? revision;

  /// The next global tick to evaluate.
  final int nextTick;

  /// Every tick evaluated so far, in order, as bdld rendered them.
  final List<pb.TickSample> samples;

  /// The input trace Studio authored: per unresolved mapping, the value
  /// fed at each tick evaluated so far.
  final Map<int, Map<int, pb.Value>> inputs;

  /// The value each input takes at the ticks about to be evaluated.
  final Map<int, pb.Value> current;

  /// Activation period per timing domain (1 = every tick); domains absent
  /// here activate every tick.  A period, not a rate.
  final Map<int, int> periods;
  final bool pending;
  final int generation;

  /// The evaluator's structured failure at the last step, if any.
  final pb.Diagnostic? error;

  /// A refusal or transport failure (product language).
  final String? failure;

  bool get hasRun => revision != null && samples.isNotEmpty;

  SimulationState copyWith({
    int? revision,
    bool clearRun = false,
    int? nextTick,
    List<pb.TickSample>? samples,
    Map<int, Map<int, pb.Value>>? inputs,
    Map<int, pb.Value>? current,
    Map<int, int>? periods,
    bool? pending,
    int? generation,
    pb.Diagnostic? error,
    bool clearError = false,
    String? failure,
    bool clearFailure = false,
  }) => SimulationState(
    revision: clearRun ? null : (revision ?? this.revision),
    nextTick: clearRun ? 0 : (nextTick ?? this.nextTick),
    samples: clearRun ? const [] : (samples ?? this.samples),
    inputs: inputs ?? this.inputs,
    current: current ?? this.current,
    periods: periods ?? this.periods,
    pending: pending ?? this.pending,
    generation: generation ?? this.generation,
    error: clearError || clearRun ? null : (error ?? this.error),
    failure: clearFailure || clearRun ? null : (failure ?? this.failure),
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
    this.simulation = const SimulationState(),
    this.sidebar = SidebarTab.project,
    this.librarySearch = '',
    this.recentTemplates = const [],
    this.pendingInsert,
    this.renaming,
    this.context = const SystemContext(),
    this.layouts = const CanvasLayout(),
    this.pendingBind,
    this.extraction,
    this.pendingPlacement,
    this.pendingGroupFor,
    this.queuedSystemEdits = const [],
    this.renameNextGroup = false,
    this.view = DesignView.design,
    this.sources = const SourcesState(),
  });

  final StudioPage page;
  final Selection selection;

  /// Design, Code or Split: views of the one project (ADR-0023 §3).
  final DesignView view;

  /// The Code view's sources, fetched while it is on screen.
  final SourcesState sources;

  /// Canvas positions of the context on screen.  Studio authors these; the
  /// daemon stores them.  Never semantics (ADR-0003).
  final Map<NodeRef, Offset> layout;

  /// Which design the canvas shows: the system (or flat design) or one
  /// component's source.
  final DesignContext context;

  /// Every canvas of the project; [layout] is `layouts.of(context)`.
  final CanvasLayout layouts;

  /// A link that needs the designer's decision before it is sent.
  final PendingBind? pendingBind;

  /// The open "Package as reusable component" sheet, if any.
  final ExtractionState? extraction;

  /// Where the instance being created lands, once the daemon confirms it.
  final Offset? pendingPlacement;

  /// The group the relationship being created joins, once confirmed.
  final int? pendingGroupFor;

  /// System edits still to send, one per confirmed revision (a connect
  /// after its disconnect).
  final List<pb.SystemEditOp> queuedSystemEdits;

  /// A group is being created from the canvas: when it arrives, select it
  /// and open its name for editing (create-then-rename, no modal).
  final bool renameNextGroup;

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
  final SimulationState simulation;
  final SidebarTab sidebar;

  /// The library panel's search text.  Authoring convenience, never
  /// semantic resolution.
  final String librarySearch;

  /// Recently inserted concept templates, most recent first (at most
  /// [maxRecentTemplates]).  A Studio preference; never project state.
  final List<String> recentTemplates;

  /// A template insertion awaiting the daemon's answer.
  final PendingInsert? pendingInsert;

  /// The node whose name is being edited inline on the canvas.
  final NodeRef? renaming;

  static const int maxRecentTemplates = 6;

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
    SimulationState? simulation,
    SidebarTab? sidebar,
    String? librarySearch,
    List<String>? recentTemplates,
    PendingInsert? pendingInsert,
    bool clearPendingInsert = false,
    NodeRef? renaming,
    bool clearRenaming = false,
    DesignContext? context,
    CanvasLayout? layouts,
    PendingBind? pendingBind,
    bool clearPendingBind = false,
    ExtractionState? extraction,
    bool clearExtraction = false,
    Offset? pendingPlacement,
    bool clearPendingPlacement = false,
    int? pendingGroupFor,
    bool clearPendingGroup = false,
    List<pb.SystemEditOp>? queuedSystemEdits,
    bool? renameNextGroup,
    DesignView? view,
    SourcesState? sources,
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
      simulation: simulation ?? this.simulation,
      sidebar: sidebar ?? this.sidebar,
      librarySearch: librarySearch ?? this.librarySearch,
      recentTemplates: recentTemplates ?? this.recentTemplates,
      pendingInsert: clearPendingInsert ? null : (pendingInsert ?? this.pendingInsert),
      renaming: clearRenaming ? null : (renaming ?? this.renaming),
      context: context ?? this.context,
      layouts: layouts ?? this.layouts,
      pendingBind: clearPendingBind ? null : (pendingBind ?? this.pendingBind),
      extraction: clearExtraction ? null : (extraction ?? this.extraction),
      pendingPlacement: clearPendingPlacement ? null : (pendingPlacement ?? this.pendingPlacement),
      pendingGroupFor: clearPendingGroup ? null : (pendingGroupFor ?? this.pendingGroupFor),
      queuedSystemEdits: queuedSystemEdits ?? this.queuedSystemEdits,
      renameNextGroup: renameNextGroup ?? this.renameNextGroup,
      view: view ?? this.view,
      sources: sources ?? this.sources,
    );
  }

  /// Whether the Code view is on screen (alone or beside the graph).
  bool get showsCode => view != DesignView.design;

  /// The layout of the canvas on screen.
  ContextLayout get contextLayout => layouts.of(context);

  /// The component whose source is on screen, for scoped requests.
  int? get componentScope => switch (context) {
    SystemContext() => null,
    ComponentContext(:final id) => id,
  };
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
    this._flat,
    this.system,
    this.analysis,
    this.systemAnalysis,
    this.recent = const [],
    this.library,
    this.editor = const EditorState(),
    this.render = const RenderState(),
  });

  final DaemonConnection connection;

  /// Recently opened projects, newest first (app preference, persisted by
  /// the effect executor).
  final List<RecentProject> recent;

  /// The design on screen, owned by the compiler: the flat design of a flat
  /// project; for a system project, its top level (the base design) or the
  /// body of the component whose source is open — see
  /// [EditorState.context].  `null` when no project is open.  Session
  /// fields (revision, dirty, undo) are the project's whichever context.
  final pb.ProjectProjection? project;

  /// The projection the daemon serves for every whole-design request
  /// (analysis, simulation, deployment): the flat design, derived for a
  /// system project.  Equal to [project] for a flat project.
  pb.ProjectProjection? get flat => _flat ?? project;
  final pb.ProjectProjection? _flat;

  /// The authored system of a system project; `null` for a flat project.
  final pb.SystemView? system;

  /// The compiler's analysis of [flat] — kept only when its revision is
  /// the project's; `null` while a newer revision is still being analysed.
  final pb.ProjectAnalysis? analysis;

  /// The system-level analysis (composition, port statuses, boundaries,
  /// per-component body verdicts), at the project's revision.
  final pb.SystemAnalysisView? systemAnalysis;

  /// The concept libraries the daemon serves (the Standard Concept Library
  /// and, later, others) plus the shared quantity vocabulary.  Authoring
  /// vocabulary, independent of any project; `null` until the daemon
  /// answered.
  final pb.ConceptTemplatesResponse? library;
  final EditorState editor;
  final RenderState render;

  int get revision => project?.revision.toInt() ?? -1;

  bool get isSystem => system != null;

  /// The analysis that judges the design on screen: the flat analysis (a
  /// base relationship keeps its identity in the flat design), or the
  /// body's own analysis when a component's source is open.
  pb.ProjectAnalysis? get contextAnalysis => switch (editor.context) {
    SystemContext() => analysis,
    ComponentContext(:final id) =>
      systemAnalysis?.componentAnalyses.where((c) => c.id.toInt() == id).firstOrNull?.analysis,
  };

  pb.ComponentView? component(int id) =>
      system?.components.where((c) => c.id.toInt() == id).firstOrNull;

  pb.ComponentInstanceView? instance(int id) =>
      system?.instances.where((i) => i.id.toInt() == id).firstOrNull;

  /// The component an instance is an occurrence of.
  pb.ComponentView? componentOf(int instance) {
    final i = this.instance(instance);
    return i == null ? null : component(i.component.toInt());
  }

  pb.PortView? port(int instance, int port) =>
      componentOf(instance)?.ports.where((p) => p.id.toInt() == port).firstOrNull;

  pb.BindingView? binding(int id) => system?.bindings.where((b) => b.id.toInt() == id).firstOrNull;

  pb.BehaviorGroupView? group(int id) =>
      system?.groups.where((g) => g.id.toInt() == id).firstOrNull;

  /// The groups of the design on screen: the system's own, or the open
  /// component's.
  List<pb.BehaviorGroupView> get groupsInView => switch (editor.context) {
    SystemContext() => [...?system?.groups.where((g) => !g.hasComponent())],
    ComponentContext(:final id) => [
      ...?system?.groups.where((g) => g.hasComponent() && g.component.toInt() == id),
    ],
  };

  /// The scope a group created now belongs to.
  int? get groupScopeComponent => editor.componentScope;

  /// A group's boundary: the system view carries it at every authoring
  /// generation (read off the revision's analysis, never re-analysed).
  pb.BehaviorGroupBoundaryView? boundary(int group) =>
      system?.boundaries.where((g) => g.id.toInt() == group).firstOrNull ??
      systemAnalysis?.groups.where((g) => g.id.toInt() == group).firstOrNull;

  /// The group a relationship of the design on screen belongs to, if any.
  pb.BehaviorGroupView? groupOf(int mappingId) =>
      groupsInView.where((g) => g.members.any((m) => m.toInt() == mappingId)).firstOrNull;

  /// The component whose source is open, if any.
  pb.ComponentView? get openComponent => switch (editor.context) {
    SystemContext() => null,
    ComponentContext(:final id) => component(id),
  };

  /// Every template of every served library, in library order.
  Iterable<pb.ConceptTemplateView> get templates =>
      library?.libraries.expand((l) => l.templates) ?? const Iterable.empty();

  pb.ConceptTemplateView? template(String id) => templates.where((t) => t.id == id).firstOrNull;

  /// Analysis of one mapping of the design on screen at the current
  /// revision, if available.
  pb.MappingAnalysis? mappingAnalysis(int id) =>
      contextAnalysis?.mappings.where((m) => m.id.toInt() == id).firstOrNull;

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
      contextAnalysis?.outputs.where((o) => o.id.toInt() == id).firstOrNull;

  /// The system's authoring generation, for group edits (a stale one is
  /// refused).
  int? get authoringGeneration => system?.authoringGeneration.toInt();

  /// The selected entity, by identity, for service queries.  The IDE
  /// service answers about the flat design, so only the system context's
  /// flat entities are asked about.
  pb.EntityRef? get selectedEntity => editor.context is! SystemContext
      ? null
      : switch (editor.selection) {
          ConceptSelected(:final id) => pb.EntityRef(conceptId: Int64(id)),
          MappingSelected(:final id) => pb.EntityRef(mappingId: Int64(id)),
          OutputSelected(:final id) => pb.EntityRef(outputId: Int64(id)),
          _ => null,
        };

  AppState copyWith({
    DaemonConnection? connection,
    pb.ProjectProjection? project,
    bool clearProject = false,
    pb.ProjectProjection? flat,
    pb.SystemView? system,
    bool clearSystem = false,
    pb.ProjectAnalysis? analysis,
    bool clearAnalysis = false,
    pb.SystemAnalysisView? systemAnalysis,
    bool clearSystemAnalysis = false,
    List<RecentProject>? recent,
    pb.ConceptTemplatesResponse? library,
    EditorState? editor,
    RenderState? render,
  }) {
    return AppState(
      connection: connection ?? this.connection,
      project: clearProject ? null : (project ?? this.project),
      flat: clearProject ? null : (flat ?? _flat),
      system: (clearProject || clearSystem) ? null : (system ?? this.system),
      analysis: (clearProject || clearAnalysis) ? null : (analysis ?? this.analysis),
      systemAnalysis: (clearProject || clearAnalysis || clearSystemAnalysis)
          ? null
          : (systemAnalysis ?? this.systemAnalysis),
      recent: recent ?? this.recent,
      library: library ?? this.library,
      editor: editor ?? this.editor,
      render: render ?? this.render,
    );
  }
}
