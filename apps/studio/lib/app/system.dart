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
import 'package:flutter/foundation.dart' show setEquals;

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'actions.dart';
import 'drafts.dart';
import 'effects.dart';
import 'lifecycle.dart' show seedDrafts, workspaceFor;
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
  if (flat.kind != pb.ProjectKind.PROJECT_KIND_SYSTEM &&
      flat.kind != pb.ProjectKind.PROJECT_KIND_TEXT) {
    return flat;
  }
  final design = switch (context) {
    SystemContext() => system?.base,
    ComponentContext(:final id) =>
      system?.components.where((c) => c.id.toInt() == id).firstOrNull?.body,
  };
  // A group edit dirties the project without moving the revision: the
  // system view knows, the flat projection may be older.
  final dirty = flat.dirty || (system != null && system.revision == flat.revision && system.dirty);
  final view = pb.ProjectProjection()
    ..revision = flat.revision
    ..name = flat.name
    ..rootPath = flat.rootPath
    ..layout = flat.layout
    ..canUndo = flat.canUndo
    ..canRedo = flat.canRedo
    ..dirty = dirty
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
/// It names the authoring generation Studio holds, so a table another
/// client moved refuses it instead of being overwritten.
Transition sendGroupEdit(AppState s, pb.GroupEditOp op) {
  if (s.connection is! Connected || s.project == null || !s.isSystem) return Transition(s);
  return Transition(pending(s), [ApplyGroupEdit(op, baseGeneration: s.authoringGeneration)]);
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
    case CreateGroupRequested(:final name, :final description, :final members, :final renameAfter):
      final scope = s.groupScopeComponent;
      final t = sendGroupEdit(
        s,
        pb.GroupEditOp(
          createGroup: pb.CreateGroup(
            name: name,
            description: description,
            members: members.map(Int64.new),
            component: scope == null ? null : Int64(scope),
          ),
        ),
      );
      if (t.effects.isEmpty || !renameAfter) return t;
      return Transition(
        t.state.copyWith(editor: t.state.editor.copyWith(renameNextGroup: true)),
        t.effects,
      );
    case GroupSelectionRequested():
      final sel = s.editor.selection;
      final members = switch (sel) {
        MultiSelected() => sel.mappings.where((m) => s.groupOf(m) == null).toList(),
        MappingSelected(:final id) when s.groupOf(id) == null => [id],
        _ => const <int>[],
      };
      if (members.isEmpty) return Transition(s);
      return systemAction(
        s,
        CreateGroupRequested(name: freshGroupName(s), members: members, renameAfter: true),
      );
    case ViewportChanged(:final pan, :final zoom):
      final layouts = s.editor.layouts.withViewport(
        s.editor.context,
        CanvasViewport(pan: pan, zoom: zoom),
      );
      return Transition(s.copyWith(editor: s.editor.copyWith(layouts: layouts)), [
        SetLayout(layoutToPb(layouts)),
      ]);
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
      final ctx = s.editor.context;
      var box = s.editor.contextLayout.groups[id] ?? const GroupBox(rect: Rect.zero);
      // Collapsing without a box yet: it starts where the members are, so
      // nothing jumps to the origin.
      if (collapsed && box.rect == Rect.zero) {
        final at = memberOrigin(s, id);
        if (at != null) box = box.copyWith(rect: Rect.fromLTWH(at.dx, at.dy, 208, 96));
      }
      return _layoutChanged(
        s,
        s.editor.layouts.withGroup(ctx, id, box.copyWith(collapsed: collapsed)),
      );
    case GroupBoxChanged(:final id, :final rect):
      final ctx = s.editor.context;
      final box = s.editor.contextLayout.groups[id] ?? const GroupBox(rect: Rect.zero);
      // Moving a collapsed box carries its hidden members along, so that
      // expanding later shows them where the group now is (the stored
      // internal layout translated by the box's delta).
      var layouts = s.editor.layouts.withGroup(ctx, id, box.copyWith(rect: rect));
      if (box.collapsed && box.rect != Rect.zero && rect.topLeft != box.rect.topLeft) {
        final delta = rect.topLeft - box.rect.topLeft;
        final nodes = {...layouts.of(ctx).nodes};
        for (final m in s.group(id)?.members ?? const <Int64>[]) {
          final ref = NodeRef.mapping(m.toInt());
          if (nodes[ref] case final p?) nodes[ref] = p + delta;
        }
        layouts = layouts.withNodes(ctx, nodes);
      }
      return _layoutChanged(s, layouts);

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
      // Remember where the group stands now: the answer may arrive after a
      // pushed projection has already retired it.
      final members = s.group(x.groupId)?.members.map((m) => m.toInt()).toList() ?? const [];
      final nodes = s.editor.layouts.system.nodes;
      final captured = x.copyWith(
        memberPositions: {for (final m in members) NodeRef.mapping(m): ?nodes[NodeRef.mapping(m)]},
        box: s.editor.layouts.system.groups[x.groupId]?.rect,
      );
      return sendSystemEdit(
        s.copyWith(editor: s.editor.copyWith(extraction: captured)),
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
    editor: s.editor.copyWith(
      layouts: layouts,
      layout: layouts.of(s.editor.context).nodes,
      clearLayoutBefore: true,
    ),
  ),
  [SetLayout(layoutToPb(layouts))],
);

/// The top-left of a group's members on the canvas on screen, if any has
/// a stored position.
Offset? memberOrigin(AppState s, int group) {
  final nodes = s.editor.layout;
  Offset? at;
  for (final m in s.group(group)?.members ?? const <Int64>[]) {
    final p = nodes[NodeRef.mapping(m.toInt())];
    if (p == null) continue;
    at = at == null ? p : Offset(min(at.dx, p.dx), min(at.dy, p.dy));
  }
  return at == null ? null : at - const Offset(16, 38);
}

/// `Behavior`, `Behavior 2`, …: a default name the designer renames inline.
String freshGroupName(AppState s) {
  final taken = s.groupsInView.map((g) => g.name).toSet();
  if (!taken.contains('Behavior')) return 'Behavior';
  var i = 2;
  while (taken.contains('Behavior $i')) {
    i++;
  }
  return 'Behavior $i';
}

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
    layout: s.editor.layouts.of(context).nodes,
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
  var next = withView(s.copyWith(system: system), context: context);
  // The definition drafts the project was saved with, once per open: into
  // the editors of the context on screen, into the stash for the others;
  // then the context the designer left the project in.
  final seedEffects = <Effect>[];
  if (!next.editor.draftsSeeded) {
    final seeded = seedDrafts(next, system);
    final rebased = rebaseDrafts(
      seeded.drafts,
      next.project!,
      component: next.editor.componentScope,
    );
    next = next.copyWith(
      editor: next.editor.copyWith(
        drafts: rebased.drafts,
        stashedDrafts: seeded.stashed,
        draftsSeeded: true,
      ),
    );
    seedEffects.addAll(rebased.effects);
    final remembered = workspaceFor(next, flat.rootPath)?.component;
    if (remembered != null && next.component(remembered) != null) {
      final t = contextChanged(next, ComponentContext(remembered));
      return Transition(
        t.state.copyWith(editor: t.state.editor.copyWith(pendingRequests: pendingCount)),
        [...seedEffects, ...t.effects],
      );
    }
  }
  var selection = surviving(next, next.editor.selection);
  // Boxes of groups that no longer exist go with them, and the canvas of
  // a component that is gone (layout only).
  var layouts = next.editor.layouts;
  final live = system.groups.map((g) => g.id.toInt()).toSet();
  for (final id in layouts.system.groups.keys.toList()) {
    if (!live.contains(id)) layouts = layouts.withoutGroup(const SystemContext(), id);
  }
  for (final c in layouts.components.keys.toList()) {
    if (system.components.every((x) => x.id.toInt() != c)) {
      layouts = layouts.withoutComponent(c);
      continue;
    }
    for (final id in layouts.components[c]!.groups.keys.toList()) {
      if (!live.contains(id)) layouts = layouts.withoutGroup(ComponentContext(c), id);
    }
  }
  final layoutChanged = layouts != next.editor.layouts;
  // Create-then-rename: a group made from the canvas opens for naming.
  NodeRef? renaming = next.editor.renaming;
  var renameNext = next.editor.renameNextGroup;
  if (renameNext && fromRequest) {
    final before = s.system?.groups.map((g) => g.id.toInt()).toSet() ?? const <int>{};
    final created = system.groups.map((g) => g.id.toInt()).where((id) => !before.contains(id));
    if (created.isNotEmpty) {
      selection = GroupSelected(created.first);
      renaming = NodeRef.group(created.first);
    }
    renameNext = false;
  }
  return Transition(
    next.copyWith(
      editor: next.editor.copyWith(
        pendingRequests: pendingCount,
        selection: selection,
        layouts: layouts,
        layout: layouts.of(context).nodes,
        renaming: renaming,
        clearRenaming: renaming == null,
        renameNextGroup: renameNext,
      ),
    ),
    [...seedEffects, if (layoutChanged) SetLayout(layoutToPb(layouts))],
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
      final memberPositions = extraction.memberPositions;
      final box = extraction.box;
      final at = box != null && box != Rect.zero
          ? box.topLeft
          : memberPositions.isEmpty
          ? null
          : memberPositions.values.reduce((a, b) => Offset(min(a.dx, b.dx), min(a.dy, b.dy)));
      var system = {...layouts.system.nodes};
      if (at != null) system[node] = at;
      for (final m in memberPositions.keys) {
        system.remove(m);
      }
      layouts = CanvasLayout(
        system: ContextLayout(
          nodes: system,
          groups: {...layouts.system.groups}..remove(groupId),
          viewport: layouts.system.viewport,
        ),
        components: {
          ...layouts.components,
          outcome.createdComponent.toInt(): ContextLayout(nodes: memberPositions),
        },
      );
      extraction = null;
      selection = InstanceSelected(id);
    } else if (placement != null) {
      layouts = layouts.withNodes(const SystemContext(), {
        ...layouts.system.nodes,
        node: placement,
      });
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
        layout: layouts.of(state.editor.context).nodes,
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

/// The selection after a new projection: what still exists.  A selected set
/// keeps its surviving members (a multi-selection is not lost because one
/// member was deleted or an analysis arrived); a single selection whose
/// object is gone clears.
Selection surviving(AppState s, Selection sel) {
  if (sel is MultiSelected) {
    final kept = {
      for (final n in sel.nodes)
        if (nodeExists(s, n)) n,
    };
    return setEquals(kept, sel.nodes) ? sel : selectionOfNodes(kept, active: sel.active);
  }
  return selectionStillValid(s, sel) ? sel : const NoSelection();
}

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
    // Whether the edge itself is still drawn is the canvas's to judge (it
    // builds the edges); here the ends must still exist.
    LinkSelected(:final link) => nodeExists(s, link.from) && nodeExists(s, link.to),
    GroupSelected(:final id) => s.group(id) != null,
    MultiSelected(:final nodes) => nodes.every((n) => nodeExists(s, n)),
  };
}

/// The designer's name for a node of the canvas, `?` when it is gone.
String nodeName(AppState s, NodeRef n) {
  final p = s.project;
  if (p == null) return '?';
  return switch (n.kind) {
    NodeKind.concept =>
      p.concepts.where((c) => c.id.toInt() == n.id).map((c) => c.name).firstOrNull ?? '?',
    NodeKind.mapping =>
      p.mappings.where((m) => m.id.toInt() == n.id).map((m) => m.name).firstOrNull ?? '?',
    NodeKind.output =>
      p.outputs.where((o) => o.id.toInt() == n.id).map((o) => o.name).firstOrNull ?? '?',
    NodeKind.instance => s.instance(n.id)?.name ?? '?',
    NodeKind.group => s.group(n.id)?.name ?? '?',
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
  ContextLayout context(pb.Layout l) => ContextLayout(
    nodes: {
      for (final n in l.concepts) NodeRef.concept(n.id.toInt()): Offset(n.x, n.y),
      for (final n in l.mappings) NodeRef.mapping(n.id.toInt()): Offset(n.x, n.y),
      for (final n in l.outputs) NodeRef.output(n.id.toInt()): Offset(n.x, n.y),
      for (final n in l.instances) NodeRef.instance(n.id.toInt()): Offset(n.x, n.y),
    },
    groups: {
      for (final g in l.groups)
        g.id.toInt(): GroupBox(
          rect: Rect.fromLTWH(g.x, g.y, g.width, g.height),
          collapsed: g.collapsed,
        ),
    },
    viewport: l.hasViewport()
        ? CanvasViewport(pan: Offset(l.viewport.x, l.viewport.y), zoom: l.viewport.zoom)
        : null,
  );
  return CanvasLayout(
    system: context(l),
    components: {for (final c in l.components) c.id.toInt(): context(c.layout)},
  );
}

pb.Layout layoutToPb(CanvasLayout layouts) {
  pb.Layout context(ContextLayout c) {
    final entries = c.nodes.entries.toList()..sort((a, b) => a.key.id.compareTo(b.key.id));
    pb.NodePosition pos(MapEntry<NodeRef, Offset> e) =>
        pb.NodePosition(id: Int64(e.key.id), x: e.value.dx, y: e.value.dy);
    final groups = c.groups.entries.toList()..sort((a, b) => a.key.compareTo(b.key));
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
      groups: [
        for (final g in groups)
          pb.GroupBox(
            id: Int64(g.key),
            x: g.value.rect.left,
            y: g.value.rect.top,
            width: g.value.rect.width,
            height: g.value.rect.height,
            collapsed: g.value.collapsed,
          ),
      ],
      viewport: c.viewport == null
          ? null
          : pb.Viewport(x: c.viewport!.pan.dx, y: c.viewport!.pan.dy, zoom: c.viewport!.zoom),
    );
  }

  final out = context(layouts.system);
  final components = layouts.components.entries.toList()..sort((a, b) => a.key.compareTo(b.key));
  out.components.addAll([
    for (final c in components) pb.ComponentLayout(id: Int64(c.key), layout: context(c.value)),
  ]);
  return out;
}
