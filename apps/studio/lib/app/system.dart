/// System-project transitions: the design in view, system edits, behaviour
/// groups, the packaging sheet.  Pure, like the rest of the reducer.
///
/// A system project has two truths on the wire — the authored system
/// ([AppState.system]) and the flat design derived from it
/// ([AppState.flat]) — and one design on screen ([AppState.project]): the
/// system's top level, or the body of the component whose source is open
/// ([EditorState.context]).  Everything here chooses between those; it
/// never computes a semantic fact (ADR-0001).
library;

import 'dart:ui' show Offset, Rect;

import 'package:fixnum/fixnum.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'actions.dart';
import 'drafts.dart';
import 'effects.dart';
import 'reducer.dart' show Transition, decPending, pending, projectReceived;
import 'state.dart';
import 'tooling.dart' show withoutTooling;

// ---------------------------------------------------------------------------
// The design in view
// ---------------------------------------------------------------------------

/// The projection the canvas shows for [context]: the flat design of a flat
/// project; a system's top level (its base design) or a component's body,
/// with the project's session fields.  While a system project's system has
/// not arrived yet the canvas is empty rather than showing the derived
/// flat design for a moment.
pb.ProjectProjection viewProjection(
  pb.ProjectProjection flat,
  pb.SystemView? system,
  DesignContext context,
) {
  if (flat.kind != pb.ProjectKind.PROJECT_KIND_SYSTEM) return flat;
  final design = switch (context) {
    SystemContext() => system?.base,
    ComponentContext(:final id) =>
      system?.components.where((c) => c.id.toInt() == id).firstOrNull?.body,
  };
  final view = pb.ProjectProjection()
    ..revision = flat.revision
    ..name = flat.name
    ..rootPath = flat.rootPath
    ..layout = flat.layout
    ..canUndo = flat.canUndo
    ..canRedo = flat.canRedo
    ..dirty = flat.dirty
    ..kind = flat.kind;
  if (design != null) {
    view
      ..concepts.addAll(design.concepts)
      ..mappings.addAll(design.mappings)
      ..clocks.addAll(design.clocks)
      ..outputs.addAll(design.outputs)
      ..devices.addAll(design.devices);
  }
  return view;
}

/// Re-derive the design in view after the system or the context changed.
AppState withView(AppState s, {DesignContext? context}) {
  final flat = s.flat;
  if (flat == null) return s;
  final ctx = context ?? s.editor.context;
  return s.copyWith(
    project: viewProjection(flat, s.system, ctx),
    editor: context == null ? null : s.editor.copyWith(context: ctx),
  );
}

// ---------------------------------------------------------------------------
// Sending system edits
// ---------------------------------------------------------------------------

/// One system edit against the revision Studio holds.
Transition sendSystemEdit(AppState s, pb.SystemEditOp op) {
  if (s.connection is! Connected || s.project == null || !s.isSystem) return Transition(s);
  return Transition(pending(s), [ApplySystemEdit(baseRevision: s.revision, op: op)]);
}

/// One group edit: counted (it is the designer's act), never a revision.
Transition sendGroupEdit(AppState s, pb.GroupEditOp op) {
  if (s.connection is! Connected || s.project == null || !s.isSystem) return Transition(s);
  return Transition(pending(s), [ApplyGroupEdit(op)]);
}

Transition systemAction(AppState s, UserAction a) {
  switch (a) {
    case ContextChanged(:final context):
      return contextChanged(s, context);
    case CreateComponentRequested(:final name, :final description):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          createComponent: pb.CreateComponent(name: name, description: description),
        ),
      );
    case RenameComponentRequested(:final id, :final name):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          renameComponent: pb.RenameComponent(id: Int64(id), name: name),
        ),
      );
    case SetComponentDescriptionRequested(:final id, :final description):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          setComponentDescription: pb.SetComponentDescription(
            id: Int64(id),
            description: description,
          ),
        ),
      );
    case DeleteComponentRequested(:final id):
      final t = sendSystemEdit(
        s,
        pb.SystemEditOp(deleteComponent: pb.DeleteComponent(id: Int64(id))),
      );
      // Its source cannot stay open.
      if (t.effects.isEmpty || s.editor.context != ComponentContext(id)) return t;
      final back = contextChanged(t.state, const SystemContext());
      return Transition(back.state, [...t.effects, ...back.effects]);
    case DuplicateComponentRequested(:final id, :final name):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          duplicateComponent: pb.DuplicateComponent(id: Int64(id), name: name),
        ),
      );
    case DeclarePortRequested(:final component, :final decl, :final kind, :final name):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          declarePort: pb.DeclarePort(
            component: Int64(component),
            decl: Int64(decl),
            kind: kind,
            name: name,
          ),
        ),
      );
    case RenamePortRequested(:final component, :final port, :final name):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          renamePort: pb.RenamePort(component: Int64(component), port: Int64(port), name: name),
        ),
      );
    case RetirePortRequested(:final component, :final port):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          retirePort: pb.RetirePort(component: Int64(component), port: Int64(port)),
        ),
      );
    case ChangePortContractRequested(:final component, :final port, :final contract):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          changePortContract: pb.ChangePortContract(
            component: Int64(component),
            port: Int64(port),
            contract: contract,
          ),
        ),
      );
    case RebindPortDeclarationRequested(:final component, :final port, :final decl):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          rebindPortDeclaration: pb.RebindPortDeclaration(
            component: Int64(component),
            port: Int64(port),
            decl: Int64(decl),
          ),
        ),
      );
    case SetClockParameterRequested(:final component, :final clock, :final parameter):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          setClockParameter: pb.SetClockParameter(
            component: Int64(component),
            clock: Int64(clock),
            parameter: parameter,
          ),
        ),
      );
    case ShareConceptRequested(:final component, :final local, :final system):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          shareConcept: pb.ShareConcept(
            component: Int64(component),
            local: Int64(local),
            system: system == null ? null : Int64(system),
          ),
        ),
      );
    case ExternalizeOutputRequested(:final component, :final local, :final system):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          externalizeOutput: pb.ExternalizeOutput(
            component: Int64(component),
            local: Int64(local),
            system: system == null ? null : Int64(system),
          ),
        ),
      );
    case CreateInstanceRequested(:final component, :final name, :final position):
      final t = sendSystemEdit(
        s,
        pb.SystemEditOp(
          createInstance: pb.CreateInstance(component: Int64(component), name: name),
        ),
      );
      if (t.effects.isEmpty || position == null) return t;
      return Transition(
        t.state.copyWith(editor: t.state.editor.copyWith(pendingPlacement: position)),
        t.effects,
      );
    case RenameInstanceRequested(:final id, :final name):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          renameInstance: pb.RenameInstance(id: Int64(id), name: name),
        ),
      );
    case DeleteInstanceRequested(:final id):
      return sendSystemEdit(s, pb.SystemEditOp(deleteInstance: pb.DeleteInstance(id: Int64(id))));
    case ReplaceInstanceComponentRequested(:final instance, :final component):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          replaceInstanceComponent: pb.ReplaceInstanceComponent(
            instance: Int64(instance),
            component: Int64(component),
          ),
        ),
      );
    case SetClockArgumentRequested(:final instance, :final parameter, :final clock):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          setClockArgument: pb.SetClockArgument(
            instance: Int64(instance),
            parameter: Int64(parameter),
            clock: clock == null ? null : Int64(clock),
          ),
        ),
      );
    case SetParameterArgumentRequested(:final instance, :final port, :final value):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          setParameterArgument: pb.SetParameterArgument(
            instance: Int64(instance),
            port: Int64(port),
            value: value,
          ),
        ),
      );
    case LinkEndsRequested(:final source, :final destination):
      return linkEnds(s, source, destination);
    case PendingBindConfirmed(:final transportInit):
      return pendingBindConfirmed(s, transportInit);
    case PendingBindCancelled():
      return Transition(s.copyWith(editor: s.editor.copyWith(clearPendingBind: true)));
    case BindRequested(:final source, :final destination, :final transportInit):
      return sendSystemEdit(s, bindOp(source, destination, transportInit));
    case UnbindRequested(:final binding):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(unbindPorts: pb.UnbindPorts(binding: Int64(binding))),
      );

    // ---- groups ----------------------------------------------------------
    case CreateGroupRequested(:final name, :final description, :final members):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          createGroup: pb.CreateGroup(
            name: name,
            description: description,
            members: members.map(Int64.new),
          ),
        ),
      );
    case RenameGroupRequested(:final id, :final name):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          renameGroup: pb.RenameGroup(id: Int64(id), name: name),
        ),
      );
    case SetGroupDescriptionRequested(:final id, :final description):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          setGroupDescription: pb.SetGroupDescription(id: Int64(id), description: description),
        ),
      );
    case UngroupRequested(:final id):
      return sendGroupEdit(s, pb.GroupEditOp(deleteGroup: pb.DeleteGroup(id: Int64(id))));
    case DeleteGroupWithMembersRequested(:final id):
      return sendSystemEdit(
        s,
        pb.SystemEditOp(deleteGroupWithMembers: pb.DeleteGroupWithMembers(group: Int64(id))),
      );
    case AddGroupMemberRequested(:final group, :final decl):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          addMember: pb.AddGroupMember(group: Int64(group), decl: Int64(decl)),
        ),
      );
    case RemoveGroupMemberRequested(:final group, :final decl):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          removeMember: pb.RemoveGroupMember(group: Int64(group), decl: Int64(decl)),
        ),
      );
    case MoveGroupMemberRequested(:final decl, :final to):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          moveMember: pb.MoveGroupMember(decl: Int64(decl), to: Int64(to)),
        ),
      );
    case MergeGroupsRequested(:final into, :final from):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          mergeGroups: pb.MergeGroups(into: Int64(into), from: Int64(from)),
        ),
      );
    case SplitGroupRequested(:final id, :final name, :final members):
      return sendGroupEdit(
        s,
        pb.GroupEditOp(
          splitGroup: pb.SplitGroup(id: Int64(id), name: name, members: members.map(Int64.new)),
        ),
      );
    case GroupCollapsedChanged(:final id, :final collapsed):
      final box = s.editor.layouts.groups[id] ?? const GroupBox(rect: Rect.zero);
      return _layoutChanged(s, s.editor.layouts.withGroup(id, box.copyWith(collapsed: collapsed)));
    case GroupBoxChanged(:final id, :final rect):
      final box = s.editor.layouts.groups[id] ?? const GroupBox(rect: Rect.zero);
      return _layoutChanged(s, s.editor.layouts.withGroup(id, box.copyWith(rect: rect)));

    // ---- packaging -------------------------------------------------------
    case ExtractionSheetOpened(:final group):
      final g = s.group(group);
      if (g == null) return Transition(s);
      final x = ExtractionState(groupId: group, name: g.name, generation: 1);
      return Transition(s.copyWith(editor: s.editor.copyWith(extraction: x)), [
        PreviewExtraction(group: group, choices: x.choices, generation: 1),
      ]);
    case ExtractionChoicesChanged(
      :final name,
      :final instanceName,
      :final keepInternal,
      :final internalizeSinks,
    ):
      final x = s.editor.extraction;
      if (x == null) return Transition(s);
      final next = x.copyWith(
        name: name,
        instanceName: instanceName,
        keepInternal: keepInternal,
        internalizeSinks: internalizeSinks,
        generation: x.generation + 1,
        pending: true,
        clearError: true,
      );
      return Transition(s.copyWith(editor: s.editor.copyWith(extraction: next)), [
        PreviewExtraction(group: x.groupId, choices: next.choices, generation: next.generation),
      ]);
    case ExtractionSheetClosed():
      return Transition(s.copyWith(editor: s.editor.copyWith(clearExtraction: true)));
    case ExtractionConfirmed():
      final x = s.editor.extraction;
      if (x == null || x.preview == null || x.error != null) return Transition(s);
      return sendSystemEdit(
        s,
        pb.SystemEditOp(
          extractGroupAsComponent: pb.ExtractGroupAsComponent(
            group: Int64(x.groupId),
            choices: x.choices,
          ),
        ),
      );
    default:
      return Transition(s);
  }
}

Transition _layoutChanged(AppState s, CanvasLayout layouts) => Transition(
  s.copyWith(
    editor: s.editor.copyWith(layouts: layouts, layout: layouts.of(s.editor.context)),
  ),
  [SetLayout(layoutToPb(layouts))],
);

pb.SystemEditOp bindOp(pb.PortRefView source, pb.PortRefView destination, String? transportInit) =>
    pb.SystemEditOp(
      bindPorts: pb.BindPorts(
        source: source,
        destination: destination,
        transportInit: transportInit,
      ),
    );

bool sameEnd(pb.PortRefView a, pb.PortRefView b) =>
    a.hasBaseDecl() == b.hasBaseDecl() &&
    (a.hasBaseDecl() ? a.baseDecl == b.baseDecl : a.instance == b.instance && a.port == b.port);

/// The timing domain a binding end resolves to, as a comparable key read
/// off the projection: a system domain id, a private domain of one
/// instance, or `null` when agnostic or unassigned.  Only decides whether
/// to *ask* about a transport; the compiler judges the binding.
String? endDomain(AppState s, pb.PortRefView e) {
  if (e.hasBaseDecl()) {
    final m = s.system?.base.mappings.where((m) => m.id == e.baseDecl).firstOrNull;
    return m != null && m.hasClockId() ? 'system:${m.clockId}' : null;
  }
  final p = s.port(e.instance.toInt(), e.port.toInt());
  final i = s.instance(e.instance.toInt());
  if (p == null || i == null || !p.hasContract()) return null;
  final k = p.contract;
  return switch (k.clockKind) {
    pb.ClockContractKind.CLOCK_CONTRACT_KIND_PARAMETER => () {
      final bound = i.clockBindings.where((b) => b.local == k.clockId).firstOrNull;
      return bound == null ? null : 'system:${bound.system}';
    }(),
    pb.ClockContractKind.CLOCK_CONTRACT_KIND_PRIVATE => 'private:${i.id}:${k.clockId}',
    _ => null,
  };
}

String domainName(AppState s, String? key) {
  if (key == null) return '';
  if (key.startsWith('system:')) {
    final id = int.tryParse(key.substring(7));
    return s.system?.base.clocks.where((c) => c.id.toInt() == id).firstOrNull?.name ?? '';
  }
  final parts = key.split(':');
  final inst = s.instance(int.parse(parts[1]));
  final comp = inst == null ? null : s.component(inst.component.toInt());
  final clock = comp?.body.clocks.where((c) => c.id.toInt() == int.parse(parts[2])).firstOrNull;
  return inst == null || clock == null ? '' : '${inst.name}.${clock.name}';
}

/// A link between two binding ends.  Sent as it is when the destination
/// is free and the domains agree; otherwise the designer is asked first —
/// never a silent replace, never an implicit transport.
Transition linkEnds(AppState s, pb.PortRefView source, pb.PortRefView destination) {
  final sys = s.system;
  if (sys == null) return Transition(s);
  final existing = sys.bindings.where((b) => sameEnd(b.destination, destination)).firstOrNull;
  final sd = endDomain(s, source);
  final dd = endDomain(s, destination);
  final needsTransport = sd != null && dd != null && sd != dd;
  if (existing == null && !needsTransport) {
    return sendSystemEdit(s, bindOp(source, destination, null));
  }
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        pendingBind: PendingBind(
          source: source,
          destination: destination,
          replaces: existing?.id.toInt(),
          needsTransport: needsTransport,
          sourceDomain: domainName(s, sd),
          destinationDomain: domainName(s, dd),
        ),
      ),
    ),
  );
}

Transition pendingBindConfirmed(AppState s, String? transportInit) {
  final b = s.editor.pendingBind;
  if (b == null) return Transition(s);
  final cleared = s.copyWith(editor: s.editor.copyWith(clearPendingBind: true));
  final bind = bindOp(b.source, b.destination, b.needsTransport ? transportInit : null);
  if (b.replaces == null) return sendSystemEdit(cleared, bind);
  // Disconnect first; the connection follows once the compiler confirms.
  final t = sendSystemEdit(
    cleared,
    pb.SystemEditOp(unbindPorts: pb.UnbindPorts(binding: Int64(b.replaces!))),
  );
  if (t.effects.isEmpty) return t;
  return Transition(
    t.state.copyWith(
      editor: t.state.editor.copyWith(
        queuedSystemEdits: [...t.state.editor.queuedSystemEdits, bind],
      ),
    ),
    t.effects,
  );
}

// ---------------------------------------------------------------------------
// Context switches
// ---------------------------------------------------------------------------

/// The drafts of a context are filed under its own key while another
/// context is on screen: nothing typed is lost by switching.
String draftKey(String root, DesignContext c) => switch (c) {
  SystemContext() => root,
  ComponentContext(:final id) => '$root#component:$id',
};

Transition contextChanged(AppState s, DesignContext context) {
  final flat = s.flat;
  if (flat == null || context == s.editor.context) return Transition(s);
  if (context case ComponentContext(:final id) when s.component(id) == null) return Transition(s);
  final root = flat.rootPath;
  final stashed = {...s.editor.stashedDrafts};
  final dirty = dirtyDrafts(s);
  if (dirty.isEmpty) {
    stashed.remove(draftKey(root, s.editor.context));
  } else {
    stashed[draftKey(root, s.editor.context)] = dirty;
  }
  final view = viewProjection(flat, s.system, context);
  final restored = rebaseDrafts(
    stashed.remove(draftKey(root, context)) ?? const {},
    view,
    component: switch (context) {
      SystemContext() => null,
      ComponentContext(:final id) => id,
    },
  );
  final editor = withoutTooling(s.editor).copyWith(
    context: context,
    layout: s.editor.layouts.of(context),
    selection: const NoSelection(),
    clearRenaming: true,
    clearActions: true,
    clearPendingBind: true,
    drafts: restored.drafts,
    stashedDrafts: stashed,
  );
  return Transition(s.copyWith(project: view, editor: editor), restored.effects);
}

// ---------------------------------------------------------------------------
// Responses
// ---------------------------------------------------------------------------

/// The system arrived.  Kept only at the revision Studio holds (an older
/// one is stale; a newer one means a projection is on its way and will ask
/// again).  Group edits answer with the same revision and a new authoring
/// generation: the flat design, the analysis and any simulation run are
/// untouched by construction.
Transition systemReceived(AppState s, pb.SystemView system, {required bool fromRequest}) {
  final flat = s.flat;
  final pendingCount = fromRequest ? decPending(s) : s.editor.pendingRequests;
  if (flat == null || system.revision != flat.revision) {
    return Transition(s.copyWith(editor: s.editor.copyWith(pendingRequests: pendingCount)));
  }
  // A source whose component is gone (an undo, another client) closes.
  final context = switch (s.editor.context) {
    ComponentContext(:final id) when system.components.every((c) => c.id.toInt() != id) =>
      const SystemContext(),
    final c => c,
  };
  final next = withView(s.copyWith(system: system), context: context);
  final selection = selectionStillValid(next, next.editor.selection)
      ? next.editor.selection
      : const NoSelection();
  // Boxes of groups that no longer exist go with them (layout only).
  var layouts = next.editor.layouts;
  for (final id in layouts.groups.keys.toList()) {
    if (system.groups.every((g) => g.id.toInt() != id)) layouts = layouts.withoutGroup(id);
  }
  final layoutChanged = layouts != next.editor.layouts;
  return Transition(
    next.copyWith(
      editor: next.editor.copyWith(
        pendingRequests: pendingCount,
        selection: selection,
        layouts: layouts,
        layout: layouts.of(context),
      ),
    ),
    [if (layoutChanged) SetLayout(layoutToPb(layouts))],
  );
}

/// A system edit was applied: the system and the derived flat design move
/// together.  The flat projection is handled as any commit (drafts,
/// selection, analysis, queued edits); then the system-level consequences:
/// a placed instance, a packaged group replaced by its instance, a
/// relationship created into a group.
Transition systemEditApplied(
  AppState s,
  pb.SystemView system,
  pb.ProjectProjection project,
  pb.SystemEditOutcome? outcome,
) {
  final inner = outcome != null && outcome.hasInner() ? outcome.inner : null;
  final base = projectReceived(s.copyWith(system: system), project, inner, true);
  var state = base.state;
  final effects = [...base.effects];
  var layouts = state.editor.layouts;
  var selection = state.editor.selection;
  var renaming = state.editor.renaming;
  var extraction = state.editor.extraction;
  var pendingGroup = state.editor.pendingGroupFor;
  var placement = state.editor.pendingPlacement;
  var queued = state.editor.queuedSystemEdits;

  if (outcome != null && outcome.hasCreatedInstance()) {
    final id = outcome.createdInstance.toInt();
    final node = NodeRef.instance(id);
    if (extraction != null && outcome.hasCreatedComponent()) {
      // The packaged group's instance stands where the group stood; the
      // component's own canvas opens laid out as the group was.
      final groupId = extraction.groupId;
      final members = s.group(groupId)?.members.map((m) => m.toInt()).toList() ?? const [];
      final box = layouts.groups[groupId];
      final memberPositions = {
        for (final m in members) NodeRef.mapping(m): ?layouts.system[NodeRef.mapping(m)],
      };
      final at = box != null && box.rect != Rect.zero
          ? box.rect.topLeft
          : memberPositions.isEmpty
          ? null
          : memberPositions.values.reduce((a, b) => Offset(min(a.dx, b.dx), min(a.dy, b.dy)));
      var system = {...layouts.system};
      if (at != null) system[node] = at;
      for (final m in members) {
        system.remove(NodeRef.mapping(m));
      }
      layouts = CanvasLayout(
        system: system,
        groups: {...layouts.groups}..remove(groupId),
        components: {...layouts.components, outcome.createdComponent.toInt(): memberPositions},
      );
      extraction = null;
      selection = InstanceSelected(id);
    } else if (placement != null) {
      layouts = layouts.withNodes(const SystemContext(), {...layouts.system, node: placement});
      placement = null;
      selection = InstanceSelected(id);
      renaming = node;
    } else {
      selection = InstanceSelected(id);
    }
    effects.add(SetLayout(layoutToPb(layouts)));
  } else if (outcome != null && outcome.hasCreatedComponent()) {
    selection = ComponentSelected(outcome.createdComponent.toInt());
  } else if (outcome != null && outcome.hasCreatedBinding()) {
    selection = BindingSelected(outcome.createdBinding.toInt());
  }

  // "+ Add relationship" in a group: the created relationship joins it.
  if (pendingGroup != null && inner != null && inner.hasCreatedMapping()) {
    effects.add(
      ApplyGroupEdit(
        pb.GroupEditOp(
          addMember: pb.AddGroupMember(group: Int64(pendingGroup), decl: inner.createdMapping),
        ),
      ),
    );
    state = pending(state);
    pendingGroup = null;
  }

  // A queued system edit (the connect after a disconnect) goes next.
  if (queued.isNotEmpty) {
    final next = queued.first;
    queued = queued.sublist(1);
    effects.add(ApplySystemEdit(baseRevision: project.revision.toInt(), op: next));
    state = pending(state);
  }

  return Transition(
    state.copyWith(
      editor: state.editor.copyWith(
        layouts: layouts,
        layout: layouts.of(state.editor.context),
        selection: selection,
        renaming: renaming,
        clearRenaming: renaming == null,
        extraction: extraction,
        clearExtraction: extraction == null,
        clearPendingGroup: pendingGroup == null,
        clearPendingPlacement: placement == null,
        queuedSystemEdits: queued,
      ),
    ),
    effects,
  );
}

double min(double a, double b) => a < b ? a : b;

Transition systemAnalysisReceived(AppState s, pb.SystemAnalysisView a) {
  final keep = s.flat != null && a.revision == s.flat!.revision;
  return Transition(s.copyWith(systemAnalysis: keep ? a : null, clearSystemAnalysis: !keep));
}

Transition extractionPreviewReceived(AppState s, int generation, pb.ExtractionPreviewView p) {
  final x = s.editor.extraction;
  if (x == null || x.generation != generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(
        extraction: x.copyWith(preview: p, pending: false, clearError: true),
      ),
    ),
  );
}

Transition extractionPreviewFailed(AppState s, int generation, String message) {
  final x = s.editor.extraction;
  if (x == null || x.generation != generation) return Transition(s);
  return Transition(
    s.copyWith(
      editor: s.editor.copyWith(extraction: x.copyWith(pending: false, error: message)),
    ),
  );
}

// ---------------------------------------------------------------------------
// Validity of selections and nodes across pushes
// ---------------------------------------------------------------------------

bool selectionStillValid(AppState s, Selection sel) {
  final p = s.project;
  if (p == null) return sel is NoSelection;
  return switch (sel) {
    NoSelection() => true,
    ConceptSelected(:final id) => p.concepts.any((c) => c.id.toInt() == id),
    MappingSelected(:final id) => p.mappings.any((m) => m.id.toInt() == id),
    OutputSelected(:final id) => p.outputs.any((o) => o.id.toInt() == id),
    ComponentSelected(:final id) => s.component(id) != null,
    InstanceSelected(:final id) => s.instance(id) != null,
    PortSelected(:final instance, :final port) => s.port(instance, port) != null,
    BindingSelected(:final id) => s.binding(id) != null,
    GroupSelected(:final id) => s.group(id) != null,
  };
}

bool nodeExists(AppState s, NodeRef node) {
  final p = s.project;
  if (p == null) return false;
  return switch (node.kind) {
    NodeKind.concept => p.concepts.any((c) => c.id.toInt() == node.id),
    NodeKind.mapping => p.mappings.any((m) => m.id.toInt() == node.id),
    NodeKind.output => p.outputs.any((o) => o.id.toInt() == node.id),
    NodeKind.instance => s.instance(node.id) != null,
    NodeKind.group => s.group(node.id) != null,
  };
}

// ---------------------------------------------------------------------------
// Layout ⇄ wire
// ---------------------------------------------------------------------------

CanvasLayout layoutFromPb(pb.Layout l) {
  Map<NodeRef, Offset> nodes(pb.Layout l) => {
    for (final n in l.concepts) NodeRef.concept(n.id.toInt()): Offset(n.x, n.y),
    for (final n in l.mappings) NodeRef.mapping(n.id.toInt()): Offset(n.x, n.y),
    for (final n in l.outputs) NodeRef.output(n.id.toInt()): Offset(n.x, n.y),
    for (final n in l.instances) NodeRef.instance(n.id.toInt()): Offset(n.x, n.y),
  };
  return CanvasLayout(
    system: nodes(l),
    groups: {
      for (final g in l.groups)
        g.id.toInt(): GroupBox(
          rect: Rect.fromLTWH(g.x, g.y, g.width, g.height),
          collapsed: g.collapsed,
        ),
    },
    components: {for (final c in l.components) c.id.toInt(): nodes(c.layout)},
  );
}

pb.Layout layoutToPb(CanvasLayout layouts) {
  pb.Layout nodes(Map<NodeRef, Offset> layout) {
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
      instances: [
        for (final e in entries)
          if (e.key.kind == NodeKind.instance) pos(e),
      ],
    );
  }

  final out = nodes(layouts.system);
  final groups = layouts.groups.entries.toList()..sort((a, b) => a.key.compareTo(b.key));
  out.groups.addAll([
    for (final g in groups)
      pb.GroupBox(
        id: Int64(g.key),
        x: g.value.rect.left,
        y: g.value.rect.top,
        width: g.value.rect.width,
        height: g.value.rect.height,
        collapsed: g.value.collapsed,
      ),
  ]);
  final components = layouts.components.entries.toList()..sort((a, b) => a.key.compareTo(b.key));
  out.components.addAll([
    for (final c in components) pb.ComponentLayout(id: Int64(c.key), layout: nodes(c.value)),
  ]);
  return out;
}
