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
import 'preferences.dart';

export 'preferences.dart';

/// A relationship's domain is the empty product `()`: it reads nothing, so
/// it is read as a value — the candidate for a physical output, a
/// parameter, a transport, and, unresolved, a simulation input.  The
/// canonical type is `() -> B` (`crates/bdl-ir/src/ty.rs`); on the canvas
/// the unit domain is drawn as no input socket at all.
extension SignatureDomain on pb.Signature {
  bool get isUnitDomain => inputs.isEmpty;
}

/// What a relationship is, as a designer reads it — one of three, stated
/// by the daemon (`MappingView.role`, protocol 0.20) from the authored
/// shape and realization state at the revision, never stored and never
/// re-derived here (ADR-0032; FV Phase 12 `Source`, `resolved_not_source`).
/// Everything else Studio shows about a relationship — declared, invalid,
/// applied by nothing, driven, bound, backing a port — is a *state* that
/// varies within a role.
enum RelationshipRole {
  /// Unit domain, no realization: a value provided from outside this
  /// design — the environment, or, in a component body, the port it backs.
  /// Observed once per activation; a simulation input.
  source,

  /// A domain with inputs: a function from what it reads to what it
  /// produces, with or without a definition.  It has no value of its own;
  /// a value's formula applies it.
  rule,

  /// Unit domain with a realization — a formula, a binding's reference,
  /// memory, a constant: a value at every activation of its domain.
  value,
}

/// The relationship that drives [output] today, if one does (the model
/// keeps `drives` on the relationship; a second claimant is a conflict the
/// output pass reports).
pb.MappingView? currentDriver(pb.ProjectProjection p, pb.OutputView output) =>
    p.mappings.where((m) => m.hasDrivesOutputId() && m.drivesOutputId == output.id).firstOrNull;

/// The relationships that could drive [output] under the output pass's own
/// rule (DriveWF): a value or a Source — never a rule, whose type is an
/// arrow — producing exactly the concept the sink accepts (nominal
/// identity, the same `ConceptId`), updating in the sink's domain when the
/// sink has one, and not the final target of another sink.  The current
/// driver is left out: it is the sink's state, not a candidate.  Read off
/// the projection the daemon sent (role, signature, domain, drives); the
/// pass keeps the last word on a text-authored edge.
List<pb.MappingView> driveCandidates(pb.ProjectProjection p, pb.OutputView output) => [
  for (final m in p.mappings)
    if (relationshipRole(m) != RelationshipRole.rule &&
        m.signature.output == output.accepts &&
        (!output.hasClockId() || (m.hasClockId() && m.clockId == output.clockId)) &&
        !(m.hasDrivesOutputId() && m.drivesOutputId != output.id) &&
        !(m.hasDrivesOutputId() && m.drivesOutputId == output.id))
      m,
];

/// The role of [m], as the daemon stated it.  Every projection carries it;
/// a view without one is not a projection of this daemon.
RelationshipRole relationshipRole(pb.MappingView m) => switch (m.role) {
  pb.RelationshipRole.RELATIONSHIP_ROLE_SOURCE => RelationshipRole.source,
  pb.RelationshipRole.RELATIONSHIP_ROLE_RULE => RelationshipRole.rule,
  pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE => RelationshipRole.value,
  _ => throw StateError(
    'relationship ${m.id} carries no role: the daemon states it (protocol 0.20)',
  ),
};

/// The role facts Studio's surfaces read, all of them off the projection
/// and the system view — never off a formula, a name or the shape alone.
extension RoleFacts on AppState {
  /// [m] is a Source where the designer is — provided by the environment
  /// of the design on screen.  Inside an open component, a Source that
  /// backs a port is provided through that port ([backsPort]) and wears
  /// the port's word instead.
  bool isSource(pb.MappingView m) =>
      relationshipRole(m) == RelationshipRole.source && !backsPort(m);

  /// [m] is *declared*: a rule with no formula yet — the one hole a
  /// designer must fill.  A Source is complete (its provider is outside),
  /// and a value has its realization.  This is what the canvas draws
  /// dashed and what the status line counts as *not yet defined*.
  bool isDeclared(pb.MappingView m) =>
      relationshipRole(m) == RelationshipRole.rule && !m.hasDefinition();

  /// Whether [m] backs a port of the component whose source is open.
  bool backsPort(pb.MappingView m) {
    final id = m.id.toInt();
    return openComponent?.ports.any((p) => p.decl.toInt() == id) ?? false;
  }

  /// Whether a binding of the system realises [m] (its definition is a
  /// reference the system made): a state of a value, said as *bound to*.
  bool realisedByBinding(pb.MappingView m) {
    final id = m.id.toInt();
    return system?.bindings.any(
          (b) => b.destination.hasBaseDecl() && b.destination.baseDecl.toInt() == id,
        ) ??
        false;
  }
}

/// The workflow pages, in workflow order (docs/architecture/studio-ui.md §1).
enum StudioPage { design, simulate, deploy, monitor }

/// What the designer wanted to do that needs the open project unloaded
/// first.  Every project-unloading path — Close, the project manager, Open
/// or New while a project is open, ⌘W, ⌘Q, the window's close button —
/// goes through one guard: a clean project unloads at once, a dirty one
/// asks *Save / Don't Save / Cancel* and the intent runs after the close.
@immutable
sealed class UnloadIntent {
  const UnloadIntent();
}

class CloseOnly extends UnloadIntent {
  const CloseOnly();
  @override
  bool operator ==(Object other) => other is CloseOnly;
  @override
  int get hashCode => 1;
}

class OpenAnother extends UnloadIntent {
  const OpenAnother(this.rootPath);
  final String rootPath;
  @override
  bool operator ==(Object other) => other is OpenAnother && other.rootPath == rootPath;
  @override
  int get hashCode => Object.hash(OpenAnother, rootPath);
}

class CreateAnother extends UnloadIntent {
  const CreateAnother({required this.rootPath, required this.name, this.template});
  final String rootPath;
  final String name;

  /// A template id the project starts from (a demo), or empty.
  final String? template;
  @override
  bool operator ==(Object other) =>
      other is CreateAnother &&
      other.rootPath == rootPath &&
      other.name == name &&
      other.template == template;
  @override
  int get hashCode => Object.hash(CreateAnother, rootPath, name, template);
}

/// Then show the OS picker to open a project.
class PickAnother extends UnloadIntent {
  const PickAnother();
  @override
  bool operator ==(Object other) => other is PickAnother;
  @override
  int get hashCode => 2;
}

/// Then show the OS save dialog to create a project.
class PickNew extends UnloadIntent {
  const PickNew({this.template});

  /// A template id the new project starts from, or empty.
  final String? template;
  @override
  bool operator ==(Object other) => other is PickNew && other.template == template;
  @override
  int get hashCode => 3;
}

class Quit extends UnloadIntent {
  const Quit();
  @override
  bool operator ==(Object other) => other is Quit;
  @override
  int get hashCode => 4;
}

/// The one question a dirty project asks before it is unloaded.
enum CloseGuardAnswer { save, dontSave, cancel }

/// Where the designer was in a project, kept per user (never in the
/// project): the view, the source file in the editor, the component whose
/// source was open, the page.
@immutable
class ProjectWorkspace {
  const ProjectWorkspace({this.view, this.openSource, this.component, this.page});
  final String? view;
  final String? openSource;
  final int? component;
  final String? page;

  Map<String, Object> toJson() => {
    'view': ?view,
    'open_source': ?openSource,
    'component': ?component,
    'page': ?page,
  };

  static ProjectWorkspace? fromJson(Object? json) {
    if (json is! Map) return null;
    final component = json['component'];
    return ProjectWorkspace(
      view: json['view'] is String ? json['view'] as String : null,
      openSource: json['open_source'] is String ? json['open_source'] as String : null,
      component: component is num ? component.toInt() : null,
      page: json['page'] is String ? json['page'] as String : null,
    );
  }

  @override
  bool operator ==(Object other) =>
      other is ProjectWorkspace &&
      other.view == view &&
      other.openSource == openSource &&
      other.component == component &&
      other.page == page;
  @override
  int get hashCode => Object.hash(view, openSource, component, page);
}

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
    this.replaced = 0,
    this.formatGeneration = 0,
    this.formatting,
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

  /// Bumped when the editor's text was replaced by the daemon's — a
  /// formatted file — so the editor shows it (keeping the caret where it
  /// can) rather than treating it as the designer's own typing.
  final int replaced;

  /// The format request in flight: its generation and the text it was
  /// asked over.  An answer to another generation, or to a text the
  /// designer has since changed, changes nothing.
  final int formatGeneration;
  final String? formatting;

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
    int? replaced,
    int? formatGeneration,
    String? formatting,
    bool clearFormatting = false,
  }) => SourcesState(
    revision: revision ?? this.revision,
    files: files ?? this.files,
    diagnostics: diagnostics ?? this.diagnostics,
    openPath: openPath ?? this.openPath,
    buffer: clearBuffer ? null : (buffer ?? this.buffer),
    sent: clearSent ? null : (sent ?? this.sent),
    retry: clearRetry ? null : (retry ?? this.retry),
    replaced: replaced ?? this.replaced,
    formatGeneration: formatGeneration ?? this.formatGeneration,
    formatting: clearFormatting ? null : (formatting ?? this.formatting),
  );
}

/// A place in a source file, as the daemon states it: the path and a
/// byte range into that file's text as the daemon holds it.
@immutable
class SourceLocation {
  const SourceLocation({required this.path, required this.start, required this.end});
  final String path;
  final int start;
  final int end;

  @override
  bool operator ==(Object other) =>
      other is SourceLocation && other.path == path && other.start == start && other.end == end;

  @override
  int get hashCode => Object.hash(path, start, end);

  @override
  String toString() => 'SourceLocation($path, $start, $end)';
}

/// Where the Code view should go: the daemon's definition site for the
/// name the designer asked about (Cmd-click, F12).  The pane opens the
/// file and selects the range once per [generation].
@immutable
class SourceReveal {
  const SourceReveal({required this.generation, required this.location});
  final int generation;
  final SourceLocation location;
}

/// The references list under the Code view: every site naming one entity,
/// as the daemon states them, for the name the designer asked about
/// (Shift-F12).
@immutable
class SourceReferencesState {
  const SourceReferencesState({
    required this.generation,
    required this.path,
    required this.offset,
    this.title = '',
    this.locations = const [],
    this.pending = true,
  });

  /// The request tag; an older answer is dropped.
  final int generation;

  /// Where it was asked.
  final String path;
  final int offset;

  /// The entity's name, once answered.
  final String title;
  final List<SourceLocation> locations;
  final bool pending;

  SourceReferencesState copyWith({String? title, List<SourceLocation>? locations, bool? pending}) =>
      SourceReferencesState(
        generation: generation,
        path: path,
        offset: offset,
        title: title ?? this.title,
        locations: locations ?? this.locations,
        pending: pending ?? this.pending,
      );
}

/// One classified span of a text, in UTF-16 code units of the text it was
/// classified over, by the legend's names: Studio colours by name and
/// never by index, and leaves a name it does not know plain.
@immutable
class HighlightSpan {
  const HighlightSpan(this.start, this.end, this.type, this.modifiers);
  final int start;
  final int end;

  /// An LSP token type name (`type`, `function`, `variable`, `keyword`,
  /// `number`, `operator`, `comment`, …) or a BDL one (`unit`, `slot`).
  final String type;

  /// LSP modifier names (`declaration`, `defaultLibrary`) and BDL's
  /// (`source`, `output`, `device`, `instance`, `unresolved`).
  final Set<String> modifiers;

  HighlightSpan shifted(int by) => HighlightSpan(start + by, end + by, type, modifiers);

  @override
  bool operator ==(Object other) =>
      other is HighlightSpan &&
      other.start == start &&
      other.end == end &&
      other.type == type &&
      setEquals(other.modifiers, modifiers);

  @override
  int get hashCode => Object.hash(start, end, type, modifiers.length);

  @override
  String toString() => 'HighlightSpan($start, $end, $type, $modifiers)';
}

/// The tokens the IDE service gave for one document (a source file or a
/// formula draft), with the text they are over.  Only the latest
/// request's answer is applied; an older answer is dropped.  While the
/// editor's text differs from [text], the editor shifts the spans of the
/// unchanged prefix and suffix and shows the changed middle plain until
/// the next answer — so nothing flickers and nothing stale is drawn over
/// new text.
@immutable
class HighlightState {
  const HighlightState({
    required this.key,
    required this.generation,
    required this.requested,
    this.text = '',
    this.spans = const [],
    this.legendVersion,
    this.pending = false,
  });

  /// `file:<path>` or `formula:<component or ->:<mapping id>`.
  final String key;

  /// The generation of the latest request for this document.
  final int generation;

  /// The text the latest request carried.
  final String requested;

  /// The text [spans] are over.
  final String text;
  final List<HighlightSpan> spans;

  /// The legend version the spans were named by, once one answered.
  final int? legendVersion;

  /// A request is in flight; the spans on show are the previous answer's.
  final bool pending;

  static String fileKey(String path) => 'file:$path';
  static String formulaKey(int mappingId, int? component) =>
      'formula:${component ?? '-'}:$mappingId';

  HighlightState copyWith({
    int? generation,
    String? requested,
    String? text,
    List<HighlightSpan>? spans,
    int? legendVersion,
    bool? pending,
  }) => HighlightState(
    key: key,
    generation: generation ?? this.generation,
    requested: requested ?? this.requested,
    text: text ?? this.text,
    spans: spans ?? this.spans,
    legendVersion: legendVersion ?? this.legendVersion,
    pending: pending ?? this.pending,
  );
}

/// Canvas node kinds — the concept ladder's (ADR-0043, ADR-0044).
/// [mapping] is a declaration: on the canvas a **Sem block** (a
/// unit-domain declaration, one value per tick; a Source when nothing
/// defines it), in the Project list any relationship, a rule too.
/// [definition] is a Sem block's **mapping block** — its definition drawn
/// as a node of its own, keyed by the Sem block's id (one declaration, two
/// nodes, two positions).  [concept] is a template and never a canvas
/// node: the kind names a Project-list row and the concept sheet's
/// preview.  A rule is a template too, named in a mapping block's header,
/// never a node.  [instance] is a component instance of a system (rendered
/// from its ports' contracts, never its body); [group] is a collapsed
/// behaviour group (a picture of its members, never a node the compiler
/// knows).
enum NodeKind { concept, mapping, definition, output, instance, group }

/// A node on the canvas, identified by kind + stable id.
@immutable
class NodeRef {
  const NodeRef(this.kind, this.id);
  const NodeRef.concept(int id) : this(NodeKind.concept, id);
  const NodeRef.mapping(int id) : this(NodeKind.mapping, id);
  const NodeRef.definition(int id) : this(NodeKind.definition, id);
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
  const RecentProject({
    required this.path,
    required this.name,
    required this.lastOpened,
    this.workspace,
  });
  final String path;
  final String name;
  final DateTime lastOpened;

  /// Where the designer was when the project was last closed.
  final ProjectWorkspace? workspace;

  Map<String, Object> toJson() => {
    'path': path,
    'name': name,
    'last_opened': lastOpened.toUtc().toIso8601String(),
    if (workspace != null) 'workspace': workspace!.toJson(),
  };

  static RecentProject? fromJson(Object? json) {
    if (json is! Map) return null;
    final path = json['path'];
    final name = json['name'];
    final when = DateTime.tryParse(json['last_opened']?.toString() ?? '');
    if (path is! String || name is! String || when == null) return null;
    return RecentProject(
      path: path,
      name: name,
      lastOpened: when,
      workspace: ProjectWorkspace.fromJson(json['workspace']),
    );
  }

  RecentProject withWorkspace(ProjectWorkspace? workspace) =>
      RecentProject(path: path, name: name, lastOpened: lastOpened, workspace: workspace);
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

/// What an edge of the canvas is (ADR-0044): a Sem block **read** by a
/// mapping block (a name in its definition — `Reads`), a mapping block
/// **producing** its Sem block (the definition itself — `ProducedBy`), a
/// Sem block **driving** a sink (the drive edge), or a picture (a
/// collapsed group's aggregate edge).
enum LinkKind { read, produce, drive, picture }

/// One read, produce or drive edge of the canvas, by the ends it joins:
/// stable across revisions (the ids are), gone when either end or the
/// edge itself goes.  A binding is its own selection ([BindingSelected]).
@immutable
class LinkId {
  const LinkId({required this.from, required this.to, required this.concept, this.index = 0});

  /// The node whose output socket the edge leaves.
  final NodeRef from;

  /// The node whose input socket it enters, and — for a relationship's
  /// inputs — which one.
  final NodeRef to;
  final int concept;
  final int index;

  LinkKind get kind {
    if (from.kind == NodeKind.mapping && to.kind == NodeKind.definition) return LinkKind.read;
    if (from.kind == NodeKind.definition && to.kind == NodeKind.mapping) return LinkKind.produce;
    if (from.kind == NodeKind.mapping && to.kind == NodeKind.output) return LinkKind.drive;
    return LinkKind.picture;
  }

  /// Whether the edge can be taken away on its own (ADR-0044): a drive
  /// edge by `SetMappingDrive` to none; a read edge by a text edit of the
  /// definition the compiler makes (`ComposeAction.unreference`: every
  /// occurrence of the name becomes a slot).  A produce edge is the
  /// definition itself, write-once — it goes with the definition (the
  /// inspector's *Detach definition*), never alone; a collapsed group's
  /// aggregate edges stand for members and are a picture.
  bool get disconnectable => kind == LinkKind.drive || kind == LinkKind.read;

  @override
  bool operator ==(Object other) =>
      other is LinkId &&
      other.from == from &&
      other.to == to &&
      other.concept == concept &&
      other.index == index;
  @override
  int get hashCode => Object.hash(from, to, concept, index);
  @override
  String toString() => '$from→$to:$concept/$index';
}

class LinkSelected extends Selection {
  const LinkSelected(this.link);
  final LinkId link;
  @override
  bool operator ==(Object other) => other is LinkSelected && other.link == link;
  @override
  int get hashCode => Object.hash(LinkSelected, link);
}

class GroupSelected extends Selection {
  const GroupSelected(this.id);
  final int id;
  @override
  bool operator ==(Object other) => other is GroupSelected && other.id == id;
  @override
  int get hashCode => Object.hash(GroupSelected, id);
}

/// Several canvas nodes at once (a marquee, ⌘-click): the *selected set*,
/// and among them the *active* object — the one the last plain or ⌘-click
/// named, the anchor of a range or chain selection, the object a keyboard
/// action starts from (docs/architecture/studio-ui.md §2, selection).  The
/// inspector offers what applies to all of them.  A single selection is its
/// own active object; the active one is never inferred from set order.
class MultiSelected extends Selection {
  const MultiSelected(this.nodes, {this.active});
  final Set<NodeRef> nodes;

  /// The active object of the set, if one was named; always a member.
  final NodeRef? active;
  Iterable<int> get mappings => nodes.where((n) => n.kind == NodeKind.mapping).map((n) => n.id);
  @override
  bool operator ==(Object other) =>
      other is MultiSelected && setEquals(other.nodes, nodes) && other.active == active;
  @override
  int get hashCode => Object.hash(MultiSelected, nodes.length, active);
}

/// The canvas nodes a selection stands for (a component, a port or a
/// binding is not a canvas node: an empty set).
Set<NodeRef> selectedNodes(Selection sel) => switch (sel) {
  MultiSelected(:final nodes) => nodes,
  ConceptSelected(:final id) => {NodeRef.concept(id)},
  MappingSelected(:final id) => {NodeRef.mapping(id)},
  OutputSelected(:final id) => {NodeRef.output(id)},
  InstanceSelected(:final id) => {NodeRef.instance(id)},
  GroupSelected(:final id) => {NodeRef.group(id)},
  _ => const <NodeRef>{},
};

/// The active object of a selection: a single selection is its own; a set's
/// is the one it names; a component, port or binding has none on the canvas.
NodeRef? activeNode(Selection sel) => switch (sel) {
  MultiSelected(:final active) => active,
  _ => selectedNodes(sel).firstOrNull,
};

/// The selection that stands for [nodes] with [active] named: none, one
/// (its own active object) or several.  [active] must be a member; when
/// it is not (it left the set), the set has no active object.
Selection selectionOfNodes(Set<NodeRef> nodes, {NodeRef? active}) {
  // a mapping block stands for its Sem block: the declaration is selected
  nodes = {for (final n in nodes) asDeclaration(n)};
  active = active == null ? null : asDeclaration(active);
  if (nodes.isEmpty) return const NoSelection();
  if (nodes.length == 1) return singleSelection(nodes.single);
  return MultiSelected(nodes, active: active != null && nodes.contains(active) ? active : null);
}

/// The node a selection names: a mapping block's is its Sem block (one
/// declaration, two nodes — ADR-0044); every other node is its own.
NodeRef asDeclaration(NodeRef n) => n.kind == NodeKind.definition ? NodeRef.mapping(n.id) : n;

/// Where a mapping block sits beside its Sem block when nothing has placed
/// it yet: directly to the left, as the layout service attaches it
/// (`bdl_layout::metrics`: the block's width and two gaps).  The canvas
/// draws an unplaced block there; a Sem block moved to a point takes its
/// block along at this offset.
Offset attachedBlockPosition(Offset sem) => Offset(sem.dx - 200 - 2 * 16, sem.dy);

/// `roomTemperature`, `roomTemperature2`: the name of a new Sem block of a
/// concept — the concept's name in lower camel case, kept unique among
/// [taken] (the design's names).
String blockNameFor(String conceptName, Iterable<String> taken) {
  final base = conceptName.isEmpty
      ? 'block'
      : conceptName[0].toLowerCase() + conceptName.substring(1);
  final names = taken.toSet();
  if (!names.contains(base)) return base;
  var i = 2;
  while (names.contains('$base$i')) {
    i++;
  }
  return '$base$i';
}

/// The single selection of one canvas node.
Selection singleSelection(NodeRef ref) => switch (ref.kind) {
  NodeKind.concept => ConceptSelected(ref.id),
  // a mapping block and its Sem block are one declaration: one selection
  NodeKind.mapping || NodeKind.definition => MappingSelected(ref.id),
  NodeKind.output => OutputSelected(ref.id),
  NodeKind.instance => InstanceSelected(ref.id),
  NodeKind.group => GroupSelected(ref.id),
};

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
    this.applyOnArrival,
  });
  final pb.EntityRef entity;
  final int revision;
  final int generation;
  final List<pb.SemanticActionView> actions;
  final bool pending;

  /// The kind of action the designer already chose by its title, before
  /// the list arrived (a Fix offered where the finding is shown, away from
  /// the inspector): applied the moment it arrives ready.  One that needs
  /// a choice or is blocked is shown instead — never guessed.
  final String? applyOnArrival;

  /// The action of [kind] among these (an id is `<kind>:<entity>…`).
  pb.SemanticActionView? ofKind(String kind) =>
      actions.where((x) => x.id == kind || x.id.startsWith('$kind:')).firstOrNull;

  /// The actions on show are about this object at this revision.
  bool isFor(pb.EntityRef e, int rev) => entity == e && revision == rev;

  SemanticActionsState copyWith({
    List<pb.SemanticActionView>? actions,
    bool? pending,
    bool clearApplyOnArrival = false,
  }) => SemanticActionsState(
    entity: entity,
    revision: revision,
    generation: generation,
    actions: actions ?? this.actions,
    pending: pending ?? this.pending,
    applyOnArrival: clearApplyOnArrival ? null : applyOnArrival,
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
    this.projection,
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

  /// The Formula Composer's view of [source], from the same verdict
  /// (protocol 0.12).  A projection of the text, never a second store:
  /// it is dropped with [analysis] on every change and never edited here.
  final pb.FormulaProjection? projection;

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
    pb.FormulaProjection? projection,
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
      projection: clearAnalysis ? null : (projection ?? this.projection),
    );
  }
}

/// The Formula Composer: which editing projection of the definition is on
/// screen and what is selected in the structured one.  Studio owns the
/// selection, the mode and the open pop-up; the compiler owns the tree,
/// every expected type and every candidate (protocol 0.12).
@immutable
class ComposerState {
  const ComposerState({
    this.formulaMode = true,
    this.mappingId,
    this.selectedNode,
    this.slot,
    this.slotGeneration,
    this.projection,
    this.projectionGeneration,
    this.pendingCompose = false,
    this.composeGeneration,
    this.composeSource,
    this.caret,
    this.commitOnCompose = false,
  });

  /// The compose answer awaited is a canvas wire (ADR-0044): commit the
  /// draft it makes as one edit, instead of leaving it to the designer.
  final bool commitOnCompose;

  /// Formula (structured) or Text — a preference of the editor, not of
  /// any mapping.
  final bool formulaMode;

  /// The structural caret (`app/caret.dart`): where typing lands, as a
  /// byte offset into the draft source and, when the projection on screen
  /// has it, the compiler's caret it stands at — two carets may share a
  /// byte (after the denominator, after the fraction) and be two places.
  /// `null`: no caret.
  final CaretState? caret;

  /// The mapping the selection, slot and committed projection belong to.
  final int? mappingId;

  /// The selected node of the projection, by its path; `null` when none.
  final String? selectedNode;

  /// What the selected position expects and what fits, once answered.
  final pb.FormulaSlotResponse? slot;
  final int? slotGeneration;

  /// The projection of the *committed* definition, fetched while there is
  /// no draft (a draft carries its own).
  final pb.FormulaProjection? projection;
  final int? projectionGeneration;

  /// A structured action is being answered: its request tag and the draft
  /// text it was computed against.  An answer is applied only while both
  /// still hold — a newer keystroke or action makes it stale.
  final bool pendingCompose;
  final int? composeGeneration;
  final String? composeSource;

  ComposerState copyWith({
    bool? formulaMode,
    int? mappingId,
    bool clearMapping = false,
    String? selectedNode,
    bool clearSelection = false,
    pb.FormulaSlotResponse? slot,
    bool clearSlot = false,
    int? slotGeneration,
    pb.FormulaProjection? projection,
    bool clearProjection = false,
    int? projectionGeneration,
    bool? pendingCompose,
    int? composeGeneration,
    String? composeSource,
    bool clearCompose = false,
    CaretState? caret,
    bool clearCaret = false,
    bool? commitOnCompose,
  }) => ComposerState(
    formulaMode: formulaMode ?? this.formulaMode,
    mappingId: clearMapping ? null : (mappingId ?? this.mappingId),
    selectedNode: clearSelection ? null : (selectedNode ?? this.selectedNode),
    slot: clearSlot || clearSelection ? null : (slot ?? this.slot),
    slotGeneration: clearSelection
        ? null
        : (slotGeneration ?? (clearSlot ? null : this.slotGeneration)),
    projection: clearProjection ? null : (projection ?? this.projection),
    projectionGeneration: clearProjection
        ? null
        : (projectionGeneration ?? this.projectionGeneration),
    pendingCompose: clearCompose ? false : (pendingCompose ?? this.pendingCompose),
    composeGeneration: clearCompose ? null : (composeGeneration ?? this.composeGeneration),
    composeSource: clearCompose ? null : (composeSource ?? this.composeSource),
    caret: clearCaret ? null : (caret ?? this.caret),
    commitOnCompose: clearCompose ? false : (commitOnCompose ?? this.commitOnCompose),
  );
}

/// A canvas wire waiting for the projection of the definition it edits
/// (ADR-0044): which block, and the compiler action — a fill of a slot
/// with a Sem block's name, or an unreference of one.
@immutable
class PendingWire {
  const PendingWire({required this.mappingId, required this.action});
  final int mappingId;
  final pb.ComposeAction action;
}

/// The structural caret of the Formula view: a byte [offset] into the
/// draft on screen — what a typed character is inserted at — and the
/// compiler caret [id] it was placed at (`r.0:after`, or Studio's own
/// `r.0.0@2` inside a name), when known.  The id decides between carets
/// that share a byte; the offset survives a text edit the id does not.
@immutable
class CaretState {
  const CaretState(this.offset, {this.id});
  final int offset;
  final String? id;

  @override
  bool operator ==(Object other) => other is CaretState && other.offset == offset && other.id == id;

  @override
  int get hashCode => Object.hash(offset, id);

  @override
  String toString() => 'CaretState($offset, $id)';
}

/// A completion pop-up over the definition field: the service's candidates
/// for one (mapping, source, byte offset), newest request wins.
@immutable
class CompletionState {
  const CompletionState({
    this.mappingId,
    this.path,
    required this.generation,
    required this.source,
    required this.offset,
    this.items = const [],
    this.selected = 0,
    this.pending = true,
  });

  /// The formula field's relationship, or …
  final int? mappingId;

  /// … the Code view's source file the pop-up belongs to.
  final String? path;

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
    path: path,
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
  const HoverState({
    required this.generation,
    this.mappingId,
    this.path,
    this.offset,
    this.entity,
    this.card,
  });
  final int generation;

  /// Formula hover: the mapping whose draft text is hovered, at a byte offset.
  final int? mappingId;

  /// Source hover: the Code view's file whose text is hovered, at a byte offset.
  final String? path;
  final int? offset;

  /// Entity hover (canvas / library rows).
  final pb.EntityRef? entity;

  /// The service's card, once it arrived; `null` while pending.
  final pb.DraftHoverResponse? card;

  HoverState copyWith({pb.DraftHoverResponse? card}) => HoverState(
    generation: generation,
    mappingId: mappingId,
    path: path,
    offset: offset,
    entity: entity,
    card: card ?? this.card,
  );
}

/// The firmware of the chosen board, as the daemon reports it: the last
/// build's status (its artifact and whether it is still the design on
/// screen), the build or flash running now, the devices a flash could
/// reach, the last flash.  Every fact here is the daemon's; Studio asks
/// and shows (docs/architecture/firmware-build.md).
@immutable
class FirmwareState {
  const FirmwareState({
    this.status,
    this.statusGeneration = 0,
    this.building,
    this.output = const [],
    this.devices = const [],
    this.methods = const [],
    this.devicesLoaded = false,
    this.chosenDevice,
    this.flashing,
    this.flashed,
    this.error,
  });

  /// `GetBuildStatus`'s answer, or the final `BuildProgress`'s.
  final pb.BuildStatus? status;

  /// Request tag; only the latest answer is applied.
  final int statusGeneration;

  /// The build running now, as the last event described it.
  final pb.BuildProgress? building;

  /// The compiler's lines of the running build (advanced view).
  final List<String> output;

  /// What a flash could reach, as last asked.
  final List<pb.FlashDevice> devices;
  final List<pb.FlashMethodView> methods;
  final bool devicesLoaded;

  /// The device chosen when several are reachable.
  final String? chosenDevice;

  /// The flash running now, as the last event described it.
  final pb.FlashProgress? flashing;

  /// The last flash's outcome (COMPLETED or FAILED) for this target.
  final pb.FlashProgress? flashed;

  /// A refused build or flash request (product language).
  final String? error;

  bool get isBuilding => building != null;
  bool get isFlashing => flashing != null;
  pb.BuildArtifact? get artifact => status?.hasArtifact() == true ? status!.artifact : null;
  bool get artifactFresh => status?.artifactFresh ?? false;

  /// The one device a flash may take without a choice: exactly one
  /// reachable, or the chosen one when several are.
  pb.FlashDevice? get flashTarget {
    if (devices.length == 1) return devices.single;
    return devices.where((d) => d.id == chosenDevice).firstOrNull;
  }

  FirmwareState copyWith({
    pb.BuildStatus? status,
    bool clearStatus = false,
    int? statusGeneration,
    pb.BuildProgress? building,
    bool clearBuilding = false,
    List<String>? output,
    List<pb.FlashDevice>? devices,
    List<pb.FlashMethodView>? methods,
    bool? devicesLoaded,
    String? chosenDevice,
    bool clearChosenDevice = false,
    pb.FlashProgress? flashing,
    bool clearFlashing = false,
    pb.FlashProgress? flashed,
    bool clearFlashed = false,
    String? error,
    bool clearError = false,
  }) => FirmwareState(
    status: clearStatus ? null : (status ?? this.status),
    statusGeneration: statusGeneration ?? this.statusGeneration,
    building: clearBuilding ? null : (building ?? this.building),
    output: output ?? this.output,
    devices: devices ?? this.devices,
    methods: methods ?? this.methods,
    devicesLoaded: devicesLoaded ?? this.devicesLoaded,
    chosenDevice: clearChosenDevice ? null : (chosenDevice ?? this.chosenDevice),
    flashing: clearFlashing ? null : (flashing ?? this.flashing),
    flashed: clearFlashed ? null : (flashed ?? this.flashed),
    error: clearError ? null : (error ?? this.error),
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
    this.firmware = const FirmwareState(),
    this.templates = const [],
    this.templatesLoaded = false,
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

  /// The chosen board's firmware; reset when the board changes.
  final FirmwareState firmware;

  /// The templates a new project can start from, as bdld lists them.
  final List<pb.TemplateView> templates;
  final bool templatesLoaded;

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
    FirmwareState? firmware,
    List<pb.TemplateView>? templates,
    bool? templatesLoaded,
  }) => DeployState(
    targets: targets ?? this.targets,
    targetsLoaded: targetsLoaded ?? this.targetsLoaded,
    targetId: clearTarget ? null : (targetId ?? this.targetId),
    analysis: clearAnalysis ? null : (analysis ?? this.analysis),
    pending: pending ?? this.pending,
    generation: generation ?? this.generation,
    error: clearError ? null : (error ?? this.error),
    firmware: firmware ?? this.firmware,
    templates: templates ?? this.templates,
    templatesLoaded: templatesLoaded ?? this.templatesLoaded,
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
  const PendingInsert({required this.templateId, this.position, this.named = false});

  /// The library item, or [kSourceInsert] for a Source created through the
  /// Source sheet (the created objects are placed the same way), or
  /// [kConceptInsert] for a concept created through the concept sheet.
  final String templateId;

  /// Scene position of the drop / right-click; `null` for a keyboard or
  /// panel insertion (auto-placed).
  final Offset? position;

  /// The designer named the object before it was created (the concept
  /// sheet): it lands selected and its name does not open for editing.
  /// The legacy create-then-rename path (an item instantiated with its
  /// default name) opens the name.
  final bool named;

  /// The [templateId] of a Source created over a chosen concept.
  static const String kSourceInsert = 'source';

  /// The [templateId] of a concept created on the concept sheet.
  static const String kConceptInsert = 'concept';
}

/// The concept sheet (ADR-0041): a concept is created from a *value
/// category* — a Standard Library item — and a name the designer gives
/// before anything is created.  `null` when the sheet is closed.
@immutable
class ConceptSheetState {
  const ConceptSheetState({this.presetId = '', this.position});

  /// The library item (a value category) prefilling the sheet; empty for
  /// the Project tab's *New concept*, where the category is chosen on the
  /// sheet.
  final String presetId;

  /// Where the concept lands (scene coordinates); `null` auto-places.
  final Offset? position;
}

/// The Source sheet, open over the design: a Source is created over a
/// concept the designer chooses — an existing one of the design in view,
/// or a new one created in the same transaction — so nothing is committed
/// until the choice is complete, and cancelling changes nothing
/// (docs/spec/concept-library.md).  [candidates] is the daemon's ranked
/// answer for the design at [revision]; `null` while it is on its way.
@immutable
class SourceSheetState {
  const SourceSheetState({
    required this.presetId,
    required this.revision,
    this.position,
    this.candidates,
    this.conceptId,
  });

  /// The Source item whose preset prefills the sheet; empty for the
  /// generic *New Source…*.
  final String presetId;

  /// A block sheet (ADR-0044): the concept is this one and not chosen on
  /// the sheet — the designer names the block, the thing it is.
  final int? conceptId;

  /// The revision the candidates were asked for.
  final int revision;

  /// Where the objects land (scene coordinates); `null` auto-places.
  final Offset? position;
  final pb.SourceCandidatesResponse? candidates;

  bool get ready => candidates != null;

  SourceSheetState withCandidates(pb.SourceCandidatesResponse c) => SourceSheetState(
    presetId: presetId,
    revision: revision,
    position: position,
    candidates: c,
    conceptId: conceptId,
  );
}

/// A committed definition's projection for an expanded node on the canvas:
/// requested at [revision] (the answer is applied only for that revision
/// and [generation]); [projection] is `null` while the answer is on its
/// way.
@immutable
class FormulaPreview {
  const FormulaPreview({required this.revision, required this.generation, this.projection});
  final int revision;
  final int generation;
  final pb.FormulaProjection? projection;
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
    this.pendingReveal,
    this.definitionFocus = 0,
    this.actions,
    this.queuedEdits = const [],
    this.deploy = const DeployState(),
    this.simulation = const SimulationState(),
    this.sidebar = SidebarTab.project,
    this.librarySearch = '',
    this.recentTemplates = const [],
    this.pendingInsert,
    this.pendingWire,
    this.sourceSheet,
    this.conceptSheet,
    this.formulaSheet,
    this.expandedFormulas = const {},
    this.formulaPreviews = const {},
    this.renaming,
    this.context = const SystemContext(),
    this.layouts = const CanvasLayout(),
    this.layoutBefore,
    this.frameRequest = 0,
    this.pendingBind,
    this.extraction,
    this.pendingPlacement,
    this.pendingGroupFor,
    this.queuedSystemEdits = const [],
    this.renameNextGroup = false,
    this.view = DesignView.design,
    this.sources = const SourcesState(),
    this.composer = const ComposerState(),
    this.closeGuard,
    this.unloading,
    this.afterClose,
    this.closeAfterSave,
    this.pendingSave,
    this.draftsSeeded = false,
    this.highlights = const {},
    this.reveal,
    this.references,
  });

  final StudioPage page;

  /// The Code view's pending navigation and its references list
  /// (`app/code_tooling.dart`).
  final SourceReveal? reveal;

  /// *Reveal in Code* asked for a node before its sources were on hand:
  /// revealed when they arrive.
  final NodeRef? pendingReveal;

  /// *Edit Definition*: bumped so the inspector's definition editor takes
  /// focus once, for the selected relationship.
  final int definitionFocus;
  final SourceReferencesState? references;

  /// The semantic tokens of the texts on screen, by document key (a source
  /// file, a formula draft): what the IDE service last said the spans of
  /// a text are.  Kept across revisions and pushed projections — they are
  /// about a text, not a revision — and shifted, never recomputed, while
  /// the designer types (`app/highlighting.dart`).
  final Map<String, HighlightState> highlights;

  /// The *Save changes?* sheet is up for this intent.
  final UnloadIntent? closeGuard;

  /// An unload was asked for and the project is being asked whether it
  /// differs from what is saved (its edits flushed first); the answer
  /// closes it or raises [closeGuard].
  final UnloadIntent? unloading;

  /// What to do once the project has closed.
  final UnloadIntent? afterClose;

  /// The guard's *Save*: unload once the save has succeeded, never before.
  final UnloadIntent? closeAfterSave;

  /// A save asked for while typed text was still unsent: sent after that
  /// text has reached the project.
  final bool? pendingSave;

  /// The definition drafts the project was saved with have been taken
  /// into the editors once for this open.
  final bool draftsSeeded;

  /// The Formula Composer's editor state (mode, selection, open slot).
  final ComposerState composer;
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

  /// The layout as it was before the last _Arrange Automatically_, kept
  /// until any other layout change: _Undo Arrange_ restores it.  Layout
  /// only, apart from the semantic history (ADR-0003).
  final CanvasLayout? layoutBefore;

  /// Bumped when the canvas should frame the whole design once (after an
  /// arrangement); never on the designer's own moves.
  final int frameRequest;

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

  /// Recently inserted library items, most recent first (at most
  /// [maxRecentTemplates]).  A Studio preference; never project state.
  final List<String> recentTemplates;

  /// A template insertion awaiting the daemon's answer.
  final PendingInsert? pendingInsert;

  /// A canvas wire awaiting the projection it fills.
  final PendingWire? pendingWire;

  /// The Source sheet, while open (`null` otherwise).
  final SourceSheetState? sourceSheet;

  /// The concept sheet, while open (`null` otherwise).
  final ConceptSheetState? conceptSheet;

  /// The formula sheet — the definition editor of one relationship, over
  /// the design, at display size (docs/architecture/studio-ui.md §4b,
  /// *The formula sheet*) — while open: the relationship's id.  It is a
  /// view of the same draft the inspector edits; nothing is held in it.
  final int? formulaSheet;

  /// The mappings whose saved formula is shown expanded on the canvas
  /// (docs/architecture/studio-ui.md §2, *Expanded formula*), each with
  /// the height of its picture — the initial height until the picture is
  /// drawn and measured, then its own, never above the maximum: a reading
  /// state of the editor, never project data, kept across selections and
  /// revisions while the mapping exists.
  final Map<int, double> expandedFormulas;

  /// The committed definitions' projections fetched for the expanded
  /// nodes, by mapping id, each stamped with the revision it is of: a
  /// preview of another revision is stale and is replaced, never drawn as
  /// current.
  final Map<int, FormulaPreview> formulaPreviews;

  /// The node whose name is being edited inline on the canvas.
  final NodeRef? renaming;

  static const int maxRecentTemplates = 6;

  /// An expanded formula region's height before its picture is measured,
  /// and the most it may grow to (`docs/architecture/studio-ui.md` §2,
  /// *Expanded formula*): a long formula is clipped on the node, the rest
  /// read in the inspector — never a node that swallows the graph.
  static const double formulaInitialHeight = 64;
  static const double formulaMaxHeight = 220;

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
    PendingWire? pendingWire,
    bool clearPendingWire = false,
    SourceSheetState? sourceSheet,
    bool clearSourceSheet = false,
    ConceptSheetState? conceptSheet,
    bool clearConceptSheet = false,
    int? formulaSheet,
    bool clearFormulaSheet = false,
    Map<int, double>? expandedFormulas,
    Map<int, FormulaPreview>? formulaPreviews,
    NodeRef? renaming,
    bool clearRenaming = false,
    DesignContext? context,
    CanvasLayout? layouts,
    CanvasLayout? layoutBefore,
    bool clearLayoutBefore = false,
    int? frameRequest,
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
    ComposerState? composer,
    UnloadIntent? closeGuard,
    bool clearCloseGuard = false,
    UnloadIntent? unloading,
    bool clearUnloading = false,
    UnloadIntent? afterClose,
    bool clearAfterClose = false,
    UnloadIntent? closeAfterSave,
    bool clearCloseAfterSave = false,
    bool? pendingSave,
    bool clearPendingSave = false,
    bool? draftsSeeded,
    Map<String, HighlightState>? highlights,
    SourceReveal? reveal,
    NodeRef? pendingReveal,
    bool clearPendingReveal = false,
    int? definitionFocus,
    SourceReferencesState? references,
    bool clearReferences = false,
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
      pendingWire: clearPendingWire ? null : (pendingWire ?? this.pendingWire),
      sourceSheet: clearSourceSheet ? null : (sourceSheet ?? this.sourceSheet),
      conceptSheet: clearConceptSheet ? null : (conceptSheet ?? this.conceptSheet),
      formulaSheet: clearFormulaSheet ? null : (formulaSheet ?? this.formulaSheet),
      expandedFormulas: expandedFormulas ?? this.expandedFormulas,
      formulaPreviews: formulaPreviews ?? this.formulaPreviews,
      renaming: clearRenaming ? null : (renaming ?? this.renaming),
      context: context ?? this.context,
      layouts: layouts ?? this.layouts,
      layoutBefore: clearLayoutBefore ? null : (layoutBefore ?? this.layoutBefore),
      frameRequest: frameRequest ?? this.frameRequest,
      pendingBind: clearPendingBind ? null : (pendingBind ?? this.pendingBind),
      extraction: clearExtraction ? null : (extraction ?? this.extraction),
      pendingPlacement: clearPendingPlacement ? null : (pendingPlacement ?? this.pendingPlacement),
      pendingGroupFor: clearPendingGroup ? null : (pendingGroupFor ?? this.pendingGroupFor),
      queuedSystemEdits: queuedSystemEdits ?? this.queuedSystemEdits,
      renameNextGroup: renameNextGroup ?? this.renameNextGroup,
      view: view ?? this.view,
      sources: sources ?? this.sources,
      composer: composer ?? this.composer,
      closeGuard: clearCloseGuard ? null : (closeGuard ?? this.closeGuard),
      unloading: clearUnloading ? null : (unloading ?? this.unloading),
      afterClose: clearAfterClose ? null : (afterClose ?? this.afterClose),
      closeAfterSave: clearCloseAfterSave ? null : (closeAfterSave ?? this.closeAfterSave),
      pendingSave: clearPendingSave ? null : (pendingSave ?? this.pendingSave),
      draftsSeeded: draftsSeeded ?? this.draftsSeeded,
      highlights: highlights ?? this.highlights,
      reveal: reveal ?? this.reveal,
      pendingReveal: clearPendingReveal ? null : (pendingReveal ?? this.pendingReveal),
      definitionFocus: definitionFocus ?? this.definitionFocus,
      references: clearReferences ? null : (references ?? this.references),
    );
  }

  /// Whether an unload is in progress in any phase — asking, sheet up,
  /// saving first, or closing — so a second request does not start another.
  bool get unloadInProgress =>
      closeGuard != null || unloading != null || afterClose != null || closeAfterSave != null;

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
    this.preferences = const AppPreferences(),
    this.library,
    this.valueCategories = const [],
    this.editor = const EditorState(),
    this.render = const RenderState(),
  });

  /// Application preferences (language): the user's environment, persisted
  /// by the effect executor; never part of a project.
  final AppPreferences preferences;

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

  /// The concept libraries the daemon serves (the Standard Library's Concept items
  /// and, later, others) plus the shared quantity vocabulary.  Authoring
  /// vocabulary, independent of any project; `null` until the daemon
  /// answered.
  final pb.LibraryItemsResponse? library;

  /// The value categories the compiler serves (`ListValueCategories`,
  /// protocol 0.27): every category a concept may be represented by —
  /// the named quantities, the truth value, the count — with its
  /// preferred unit and the units a designer is offered, as the
  /// compiler's descriptors.  Studio composes no unit string.  Empty until
  /// the daemon answered.
  final List<pb.ValueCategoryView> valueCategories;
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

  /// The relationships [id]'s definition references — the kernel's
  /// `dependsOn`, as the analysis on screen reports it (`references`); empty
  /// while no analysis of this revision exists.  The canvas's reference
  /// edges and the inspector's *Depends on* row (ADR-0034).
  List<int> refsOf(int id) => [
    for (final m in contextAnalysis?.mappings ?? const <pb.MappingAnalysis>[])
      if (m.id.toInt() == id)
        for (final d in m.references) d.toInt(),
  ];

  /// The relationships whose definitions reference [id] — the compiler's
  /// `applied_by`, the direct reverse edges it states (protocol 0.20),
  /// never an inversion done here; the inspector's *Named in* row and,
  /// for a rule, who applies it.  [id] itself is left out when its own
  /// definition names it.
  List<int> referrersOf(int id) => [
    for (final m in contextAnalysis?.mappings ?? const <pb.MappingAnalysis>[])
      if (m.id.toInt() == id)
        for (final d in m.appliedBy)
          if (d.toInt() != id) d.toInt(),
  ];

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

  /// Every item of every served library, in library order.
  Iterable<pb.LibraryItemView> get libraryItems =>
      library?.libraries.expand((l) => l.items) ?? const Iterable.empty();

  pb.LibraryItemView? libraryItem(String id) => libraryItems.where((i) => i.id == id).firstOrNull;

  /// The Concept items as templates (what the canvas menu's concept
  /// sub-menus list), in library order.
  Iterable<pb.ConceptTemplateView> get templates =>
      libraryItems.where((i) => i.hasConcept()).map((i) => i.concept);

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
    AppPreferences? preferences,
    pb.LibraryItemsResponse? library,
    List<pb.ValueCategoryView>? valueCategories,
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
      preferences: preferences ?? this.preferences,
      library: library ?? this.library,
      valueCategories: valueCategories ?? this.valueCategories,
      editor: editor ?? this.editor,
      render: render ?? this.render,
    );
  }
}
