/// Inspectors for a system project's own objects — a component instance,
/// a component, a port, a binding, a behaviour group — in the designer's
/// words (docs/architecture/studio-ui.md §11).  Each control commits one system or group
/// edit; every verdict shown is the compiler's, read off
/// [AppState.systemAnalysis].  Nothing here decides substitutability,
/// compatibility or a boundary.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/canvas_geometry.dart' show socketKind;
import 'canvas/concept_glyphs.dart';
import 'dialogs.dart';
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

// ---------------------------------------------------------------------------
// Shared readers of the projection (labels only, never judgments)
// ---------------------------------------------------------------------------

String portKindWord(pb.PortKind k) => switch (k) {
  pb.PortKind.PORT_KIND_REQUIRED => 'requires',
  pb.PortKind.PORT_KIND_PROVIDED => 'provides',
  pb.PortKind.PORT_KIND_PARAMETER => 'parameter',
  _ => '',
};

/// "Requires Tilt", "Provides (Tilt) → Brightness": the promise, in the
/// component's own concept names.
String contractText(pb.PortView p) {
  final k = p.contract;
  final ins = k.inputNames.join(', ');
  final shape = ins.isEmpty ? k.outputName : '($ins) → ${k.outputName}';
  return '${portKindWord(p.kind)[0].toUpperCase()}${portKindWord(p.kind).substring(1)} $shape';
}

/// "Updates in tick (parameter)", "Updates in its own tick", "Any domain".
String contractTiming(pb.PortView p) => switch (p.contract.clockKind) {
  pb.ClockContractKind.CLOCK_CONTRACT_KIND_PARAMETER =>
    'Updates in ${p.contract.clockName} (a parameter of the component)',
  pb.ClockContractKind.CLOCK_CONTRACT_KIND_PRIVATE => 'Updates in its own ${p.contract.clockName}',
  _ => 'Any timing domain',
};

String endLabel(AppState s, pb.PortRefView e) {
  if (e.hasBaseDecl()) {
    return s.system?.base.mappings.where((m) => m.id == e.baseDecl).firstOrNull?.name ?? '?';
  }
  final inst = s.instance(e.instance.toInt());
  final port = s.port(e.instance.toInt(), e.port.toInt());
  return '${inst?.name ?? '?'}.${port?.name ?? '?'}';
}

/// The status word of a port at an instance, from the system analysis.
String portStatusWord(AppState s, int instance, int port) {
  final st = s.systemAnalysis?.ports
      .where((p) => p.port.instance.toInt() == instance && p.port.port.toInt() == port)
      .firstOrNull;
  if (st == null) return '';
  return switch (st.status) {
    pb.PortStatusKind.PORT_STATUS_KIND_PROVIDED => 'provided',
    pb.PortStatusKind.PORT_STATUS_KIND_BOUND => () {
      final b = s.binding(st.binding.toInt());
      return b == null ? 'bound' : 'bound to ${endLabel(s, b.source)}';
    }(),
    pb.PortStatusKind.PORT_STATUS_KIND_EXPORTED => 'system input',
    pb.PortStatusKind.PORT_STATUS_KIND_VALUED => 'given a value',
    pb.PortStatusKind.PORT_STATUS_KIND_OPEN => 'open',
    _ => '',
  };
}

/// Diagnostics the system analysis attributes to one instance.
List<pb.Diagnostic> instanceDiagnostics(AppState s, int instance) => [
  for (final p in s.systemAnalysis?.projected ?? const <pb.ProjectedDiagnostic>[])
    if (p.hasOrigin() && p.origin.instance.toInt() == instance) p.diagnostic,
];

/// Composition diagnostics about one component's promise (`component.*`).
List<pb.Diagnostic> componentDiagnostics(AppState s, pb.ComponentView c) => [
  for (final d in s.systemAnalysis?.composition ?? const <pb.Diagnostic>[])
    if (d.code.startsWith('component.') && d.message.contains(c.name)) d,
];

// ---------------------------------------------------------------------------
// Instance
// ---------------------------------------------------------------------------

class InstanceInspector extends StatelessWidget {
  const InstanceInspector({
    super.key,
    required this.state,
    required this.id,
    required this.dispatch,
  });
  final AppState state;
  final int id;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final inst = state.instance(id);
    final comp = inst == null ? null : state.component(inst.component.toInt());
    if (inst == null || comp == null) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final sys = state.system!;
    final others = sys.components.where((c) => c.id != comp.id).toList();
    final realizes = state.systemAnalysis?.components
        .where((c) => c.id == comp.id)
        .firstOrNull
        ?.realizes;
    final ports = [...comp.ports]..sort((a, b) => a.id.compareTo(b.id));
    final used = sys.instances.where((i) => i.component == comp.id).length;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Instance',
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: inst.name,
                onCommit: (v) => dispatch(RenameInstanceRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Of',
              child: Row(
                children: [
                  Expanded(
                    child: Text(comp.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
                  ),
                  MacButton(
                    label: 'Edit Source',
                    onPressed: () => dispatch(ContextChanged(ComponentContext(comp.id.toInt()))),
                  ),
                ],
              ),
            ),
            Text(
              'One of $used instance${used == 1 ? '' : 's'} of ${comp.name}. What you see here is '
              'its promise; its source is shared by all of them.',
              style: small,
            ),
            if (realizes == false)
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: Text(
                  "${comp.name}'s source no longer keeps its promise; open it to see why.",
                  style: TextStyle(fontSize: 11, color: t.error),
                ),
              ),
            if (others.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: FormRow(
                  label: 'Replace with',
                  child: MacDropdown<int>(
                    value: null,
                    hint: 'another component…',
                    items: [for (final c in others) c.id.toInt()],
                    labelOf: (c) => state.component(c)?.name ?? '?',
                    onChanged: (c) =>
                        dispatch(ReplaceInstanceComponentRequested(instance: id, component: c)),
                  ),
                ),
              ),
          ],
        ),
        if (comp.clockParams.isNotEmpty)
          InspectorSection(
            title: 'Timing',
            children: [
              for (final c in comp.clockParams)
                FormRow(
                  label: comp.body.clocks.where((k) => k.id == c).firstOrNull?.name ?? '?',
                  child: MacDropdown<int>(
                    value:
                        inst.clockBindings.where((b) => b.local == c).firstOrNull?.system.toInt() ??
                        -1,
                    items: [-1, for (final k in sys.base.clocks) k.id.toInt()],
                    labelOf: (k) => k < 0
                        ? 'not assigned'
                        : sys.base.clocks.where((x) => x.id.toInt() == k).firstOrNull?.name ?? '?',
                    onChanged: (k) => dispatch(
                      SetClockArgumentRequested(
                        instance: id,
                        parameter: c.toInt(),
                        clock: k < 0 ? null : k,
                      ),
                    ),
                  ),
                ),
              Text(
                "Each timing parameter of the component is one of the system's domains here.",
                style: small,
              ),
            ],
          ),
        InspectorSection(
          title: 'Ports',
          children: [
            if (ports.isEmpty) Text('No ports yet.', style: small),
            for (final p in ports)
              _PortRow(state: state, instance: inst, port: p, dispatch: dispatch),
          ],
        ),
        if (instanceDiagnostics(state, id) case final ds when ds.isNotEmpty)
          InspectorSection(
            title: 'Findings',
            children: [
              for (final d in ds)
                Padding(
                  padding: const EdgeInsets.only(bottom: MacMetrics.gap),
                  child: DiagnosticCard(diagnostic: d, source: ''),
                ),
            ],
          ),
        InspectorSection(
          title: 'Remove',
          children: [
            DestructiveButton(
              label: 'Delete ${inst.name}',
              onPressed: () => dispatch(DeleteInstanceRequested(id)),
            ),
            Text('Disconnect its ports first; the component stays.', style: small),
          ],
        ),
      ],
    );
  }
}

/// One port at an instance: the promise, the status, the value (for a
/// parameter).  Click: the port's inspector.
class _PortRow extends StatelessWidget {
  const _PortRow({
    required this.state,
    required this.instance,
    required this.port,
    required this.dispatch,
  });
  final AppState state;
  final pb.ComponentInstanceView instance;
  final pb.PortView port;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final status = portStatusWord(state, instance.id.toInt(), port.id.toInt());
    final open = status == 'open';
    final comp = state.component(instance.component.toInt());
    final concept = comp?.body.concepts
        .where((c) => c.id == port.contract.signature.output)
        .firstOrNull;
    return InkWell(
      onTap: () => dispatch(
        SelectionChanged(PortSelected(instance: instance.id.toInt(), port: port.id.toInt())),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 3),
        child: Row(
          children: [
            if (concept != null)
              SocketGlyph(
                kind: socketKind(concept),
                color: t.conceptColor(port.contract.signature.output.toInt()),
                size: 10,
              )
            else
              const SizedBox(width: 10),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                '${port.name} · ${portKindWord(port.kind)}',
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
            ),
            Text(
              status,
              style: TextStyle(fontSize: 11, color: open ? t.textTertiary : t.textSecondary),
            ),
          ],
        ),
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Port (at an instance)
// ---------------------------------------------------------------------------

class PortInspector extends StatelessWidget {
  const PortInspector({
    super.key,
    required this.state,
    required this.instance,
    required this.port,
    required this.dispatch,
  });
  final AppState state;
  final int instance;
  final int port;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final inst = state.instance(instance);
    final comp = inst == null ? null : state.component(inst.component.toInt());
    final p = state.port(instance, port);
    if (inst == null || comp == null || p == null) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final status = portStatusWord(state, instance, port);
    final binding = state.system!.bindings
        .where(
          (b) =>
              !b.destination.hasBaseDecl() &&
              b.destination.instance.toInt() == instance &&
              b.destination.port.toInt() == port,
        )
        .firstOrNull;
    final impl = comp.body.mappings.where((m) => m.id == p.decl).firstOrNull;
    final value = inst.parameterBindings.where((b) => b.port == p.id).firstOrNull;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Port',
          trailing: Text(status, style: small),
          children: [
            FormRow(
              label: 'Name',
              child: Text(
                '${inst.name}.${p.name}',
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
            ),
            FormRow(
              label: 'Promise',
              child: Text(contractText(p), style: TextStyle(fontSize: 12, color: t.textPrimary)),
            ),
            FormRow(
              label: 'Timing',
              child: Text(contractTiming(p), style: TextStyle(fontSize: 12, color: t.textPrimary)),
            ),
            FormRow(
              label: 'Implemented by',
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      impl?.name ?? 'nothing (the promise has no backing)',
                      style: TextStyle(fontSize: 12, color: impl == null ? t.error : t.textPrimary),
                    ),
                  ),
                  MacButton(
                    label: 'Go to Source',
                    onPressed: () {
                      dispatch(ContextChanged(ComponentContext(comp.id.toInt())));
                      if (impl != null) {
                        dispatch(SelectionChanged(MappingSelected(impl.id.toInt())));
                      }
                    },
                  ),
                ],
              ),
            ),
            if (p.description.isNotEmpty) Text(p.description, style: small),
          ],
        ),
        if (p.kind == pb.PortKind.PORT_KIND_PARAMETER)
          InspectorSection(
            title: 'Value',
            children: [
              FormRow(
                label: 'Value',
                child: CommitTextField(
                  value: value?.source ?? '',
                  hint: 'a constant, e.g. 0.5',
                  monospace: true,
                  onCommit: (v) => dispatch(
                    SetParameterArgumentRequested(
                      instance: instance,
                      port: port,
                      value: v.trim().isEmpty ? null : v.trim(),
                    ),
                  ),
                ),
              ),
              Text(
                'A closed constant in the port\'s units, or bind the port instead.',
                style: small,
              ),
            ],
          ),
        if (p.kind != pb.PortKind.PORT_KIND_PROVIDED)
          InspectorSection(
            title: 'Connection',
            children: [
              if (binding != null) ...[
                Text(
                  'Bound to ${endLabel(state, binding.source)}'
                  '${binding.hasTransportInit() ? ', carried across timing domains starting at ${binding.transportInit}' : ''}.',
                  style: TextStyle(fontSize: 12, color: t.textPrimary),
                ),
                const SizedBox(height: 6),
                Row(
                  children: [
                    MacButton(
                      label: 'Show Binding',
                      onPressed: () =>
                          dispatch(SelectionChanged(BindingSelected(binding.id.toInt()))),
                    ),
                    const SizedBox(width: 8),
                    MacButton(
                      label: 'Disconnect',
                      onPressed: () => dispatch(UnbindRequested(binding.id.toInt())),
                    ),
                  ],
                ),
              ] else
                Text(
                  'Open: nothing supplies it yet. Draw a link from a provided port or a top-level '
                  'relationship of the same concept. A design with open ports still simulates, '
                  'with the port as an input.',
                  style: small,
                ),
            ],
          ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Binding
// ---------------------------------------------------------------------------

class BindingInspector extends StatelessWidget {
  const BindingInspector({
    super.key,
    required this.state,
    required this.id,
    required this.dispatch,
  });
  final AppState state;
  final int id;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final b = state.binding(id);
    if (b == null) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final issues = [
      for (final d in state.systemAnalysis?.composition ?? const <pb.Diagnostic>[])
        if (d.technical.contains('binding bind#$id')) d,
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Binding',
          children: [
            FormRow(
              label: 'From',
              child: Text(
                endLabel(state, b.source),
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
            ),
            FormRow(
              label: 'To',
              child: Text(
                endLabel(state, b.destination),
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
            ),
            FormRow(
              label: 'Timing',
              child: Text(
                b.hasTransportInit()
                    ? 'Carried across timing domains: the destination sees the last value '
                          'committed strictly before its own activation, starting at ${b.transportInit}.'
                    : 'Direct: the destination reads the value as it is.',
                style: TextStyle(fontSize: 12, color: t.textPrimary),
              ),
            ),
            Text(
              'A binding converts nothing: both ends carry the same concept, by identity.',
              style: small,
            ),
            for (final d in issues)
              Padding(
                padding: const EdgeInsets.only(top: MacMetrics.gap),
                child: DiagnosticCard(diagnostic: d, source: ''),
              ),
            const SizedBox(height: 8),
            MacButton(label: 'Disconnect', onPressed: () => dispatch(UnbindRequested(id))),
          ],
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Component (its promise; the body is edited as source)
// ---------------------------------------------------------------------------

class ComponentInspector extends StatelessWidget {
  const ComponentInspector({
    super.key,
    required this.state,
    required this.id,
    required this.dispatch,
  });
  final AppState state;
  final int id;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final comp = state.component(id);
    final sys = state.system;
    if (comp == null || sys == null) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final instances = sys.instances.where((i) => i.component == comp.id).toList();
    final realizes = state.systemAnalysis?.components
        .where((c) => c.id == comp.id)
        .firstOrNull
        ?.realizes;
    final ports = [...comp.ports]..sort((a, b) => a.id.compareTo(b.id));
    final editing = state.editor.context == ComponentContext(id);
    final bodyMappings = [...comp.body.mappings]..sort((a, b) => a.id.compareTo(b.id));
    final bodyClocks = [...comp.body.clocks]..sort((a, b) => a.id.compareTo(b.id));
    final bodyConcepts = [...comp.body.concepts]..sort((a, b) => a.id.compareTo(b.id));
    final bodyOutputs = [...comp.body.outputs]..sort((a, b) => a.id.compareTo(b.id));
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Component',
          trailing: realizes == null
              ? null
              : Text(
                  realizes ? 'keeps its promise' : 'promise broken',
                  style: TextStyle(fontSize: 11, color: realizes ? t.textSecondary : t.error),
                ),
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: comp.name,
                onCommit: (v) => dispatch(RenameComponentRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Meaning',
              child: CommitTextField(
                value: comp.description,
                maxLines: 3,
                onCommit: (v) => dispatch(SetComponentDescriptionRequested(id: id, description: v)),
              ),
            ),
            Text(
              instances.isEmpty
                  ? 'Not placed yet.'
                  : 'Used by ${instances.length} instance${instances.length == 1 ? '' : 's'}: '
                        '${instances.map((i) => i.name).join(', ')}.',
              style: small,
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              runSpacing: 6,
              children: [
                MacButton(
                  label: editing ? 'Back to System' : 'Edit Source',
                  onPressed: () => dispatch(
                    ContextChanged(editing ? const SystemContext() : ComponentContext(id)),
                  ),
                ),
                MacButton(
                  label: 'Duplicate as Version',
                  onPressed: () => dispatch(
                    DuplicateComponentRequested(id: id, name: _versionName(sys, comp.name)),
                  ),
                ),
                if (state.editor.context is SystemContext)
                  MacButton(
                    label: 'Place Instance',
                    onPressed: () => dispatch(
                      CreateInstanceRequested(component: id, name: _instanceName(sys, comp.name)),
                    ),
                  ),
              ],
            ),
          ],
        ),
        InspectorSection(
          title: 'Promise',
          children: [
            if (ports.isEmpty)
              Text(
                editing
                    ? 'No ports yet. Select a relationship of the source and declare it a port.'
                    : 'No ports yet.',
                style: small,
              ),
            for (final p in ports)
              _ContractEditor(state: state, component: comp, port: p, dispatch: dispatch),
          ],
        ),
        if (editing) ...[
          if (bodyClocks.isNotEmpty)
            InspectorSection(
              title: 'Timing parameters',
              children: [
                for (final c in bodyClocks)
                  FormRow(
                    label: c.name,
                    child: MacDropdown<bool>(
                      value: comp.clockParams.contains(c.id),
                      items: const [true, false],
                      labelOf: (v) =>
                          v ? 'a parameter (the system assigns it)' : 'private to each instance',
                      onChanged: (v) => dispatch(
                        SetClockParameterRequested(
                          component: id,
                          clock: c.id.toInt(),
                          parameter: v,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          if (bodyConcepts.isNotEmpty)
            InspectorSection(
              title: 'Shared concepts',
              children: [
                for (final c in bodyConcepts)
                  FormRow(
                    label: c.name,
                    child: MacDropdown<int>(
                      value:
                          comp.sharedConcepts
                              .where((p) => p.local == c.id)
                              .firstOrNull
                              ?.system
                              .toInt() ??
                          -1,
                      items: [-1, for (final s in sys.base.concepts) s.id.toInt()],
                      labelOf: (s) => s < 0
                          ? 'private (fresh per instance)'
                          : 'stands for ${sys.base.concepts.where((x) => x.id.toInt() == s).firstOrNull?.name ?? '?'}',
                      onChanged: (s) => dispatch(
                        ShareConceptRequested(
                          component: id,
                          local: c.id.toInt(),
                          system: s < 0 ? null : s,
                        ),
                      ),
                    ),
                  ),
                Text(
                  'A shared concept is the same identity everywhere; a private one is this component\'s own, fresh in every instance.',
                  style: small,
                ),
              ],
            ),
          if (bodyOutputs.isNotEmpty)
            InspectorSection(
              title: 'Physical outputs',
              children: [
                for (final o in bodyOutputs)
                  FormRow(
                    label: o.name,
                    child: MacDropdown<int>(
                      value:
                          comp.externalOutputs
                              .where((p) => p.local == o.id)
                              .firstOrNull
                              ?.system
                              .toInt() ??
                          -1,
                      items: [-1, for (final s in sys.base.outputs) s.id.toInt()],
                      labelOf: (s) => s < 0
                          ? 'private (one per instance)'
                          : 'the system\'s ${sys.base.outputs.where((x) => x.id.toInt() == s).firstOrNull?.name ?? '?'}',
                      onChanged: (s) => dispatch(
                        ExternalizeOutputRequested(
                          component: id,
                          local: o.id.toInt(),
                          system: s < 0 ? null : s,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          if (bodyMappings.any((m) => ports.every((p) => p.decl != m.id)))
            InspectorSection(
              title: 'Declare a port',
              children: [
                for (final m in bodyMappings.where((m) => ports.every((p) => p.decl != m.id)))
                  Padding(
                    padding: const EdgeInsets.only(bottom: 6),
                    child: Row(
                      children: [
                        Expanded(
                          child: Text(m.name, style: TextStyle(fontSize: 12, color: t.textPrimary)),
                        ),
                        MacDropdown<pb.PortKind>(
                          value: null,
                          hint: 'expose as…',
                          compact: true,
                          items: const [
                            pb.PortKind.PORT_KIND_REQUIRED,
                            pb.PortKind.PORT_KIND_PROVIDED,
                            pb.PortKind.PORT_KIND_PARAMETER,
                          ],
                          labelOf: portKindWord,
                          onChanged: (k) => dispatch(
                            DeclarePortRequested(
                              component: id,
                              decl: m.id.toInt(),
                              kind: k,
                              name: m.name,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                Text(
                  'The promise is taken from the relationship as it is now, and kept until you change it here.',
                  style: small,
                ),
              ],
            ),
        ],
        if (componentDiagnostics(state, comp) case final ds when ds.isNotEmpty)
          InspectorSection(
            title: 'Findings',
            children: [
              for (final d in ds)
                Padding(
                  padding: const EdgeInsets.only(bottom: MacMetrics.gap),
                  child: DiagnosticCard(diagnostic: d, source: ''),
                ),
            ],
          ),
        InspectorSection(
          title: 'Remove',
          children: [
            DestructiveButton(
              label: 'Delete ${comp.name}',
              enabled: instances.isEmpty,
              onPressed: () => dispatch(DeleteComponentRequested(id)),
            ),
            if (instances.isNotEmpty) Text('Delete its instances first.', style: small),
          ],
        ),
      ],
    );
  }

  static String _versionName(pb.SystemView sys, String name) {
    final taken = sys.components.map((c) => c.name).toSet();
    var i = 2;
    while (taken.contains('$name v$i')) {
      i++;
    }
    return '$name v$i';
  }

  static String _instanceName(pb.SystemView sys, String name) {
    final base = name.isEmpty ? 'instance' : name[0].toLowerCase() + name.substring(1);
    final taken = sys.instances.map((i) => i.name).toSet();
    if (!taken.contains(base)) return base;
    var i = 2;
    while (taken.contains('$base$i')) {
      i++;
    }
    return '$base$i';
  }
}

/// One port of a component: its promise, and — since the promise is a
/// stored fact, not a view of the body — the explicit ways to change it:
/// point it at another relationship (the promise stays), change its timing
/// (the promise changes; the compiler classifies the consequences), rename,
/// retire.
class _ContractEditor extends StatelessWidget {
  const _ContractEditor({
    required this.state,
    required this.component,
    required this.port,
    required this.dispatch,
  });
  final AppState state;
  final pb.ComponentView component;
  final pb.PortView port;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final id = component.id.toInt();
    final body = component.body;
    final impl = body.mappings.where((m) => m.id == port.decl).firstOrNull;
    final concept = body.concepts.where((c) => c.id == port.contract.signature.output).firstOrNull;
    final editing = state.editor.context == ComponentContext(id);
    // Timing choices of the contract: any domain, or one of the body's
    // domains (a parameter when declared so, private otherwise).
    final timingItems = [-1, for (final c in body.clocks) c.id.toInt()];
    final timingValue = port.contract.hasClockId() ? port.contract.clockId.toInt() : -1;
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              if (concept != null)
                SocketGlyph(
                  kind: socketKind(concept),
                  color: t.conceptColor(port.contract.signature.output.toInt()),
                  size: 10,
                ),
              const SizedBox(width: 8),
              Expanded(
                child: editing
                    ? CommitTextField(
                        value: port.name,
                        onCommit: (v) => dispatch(
                          RenamePortRequested(component: id, port: port.id.toInt(), name: v),
                        ),
                      )
                    : Text(
                        port.name,
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: t.textPrimary,
                        ),
                      ),
              ),
              const SizedBox(width: 8),
              Text(portKindWord(port.kind), style: small),
            ],
          ),
          Padding(
            padding: const EdgeInsets.only(left: 18, top: 2),
            child: Text(contractText(port), style: TextStyle(fontSize: 12, color: t.textPrimary)),
          ),
          if (!editing)
            Padding(
              padding: const EdgeInsets.only(left: 18, top: 2),
              child: Text(contractTiming(port), style: small),
            ),
          if (editing) ...[
            const SizedBox(height: 6),
            FormRow(
              label: 'Timing',
              child: MacDropdown<int>(
                value: timingValue,
                items: timingItems,
                labelOf: (c) => c < 0
                    ? 'any domain'
                    : '${body.clocks.where((x) => x.id.toInt() == c).firstOrNull?.name ?? '?'}'
                          '${component.clockParams.any((p) => p.toInt() == c) ? ' (parameter)' : ' (private)'}',
                onChanged: (c) {
                  final k = port.contract.deepCopy();
                  if (c < 0) {
                    k.clockKind = pb.ClockContractKind.CLOCK_CONTRACT_KIND_AGNOSTIC;
                    k.clearClockId();
                  } else {
                    k.clockKind = component.clockParams.any((p) => p.toInt() == c)
                        ? pb.ClockContractKind.CLOCK_CONTRACT_KIND_PARAMETER
                        : pb.ClockContractKind.CLOCK_CONTRACT_KIND_PRIVATE;
                    k.clockId = Int64(c);
                  }
                  dispatch(
                    ChangePortContractRequested(component: id, port: port.id.toInt(), contract: k),
                  );
                },
              ),
            ),
            FormRow(
              label: 'Implemented by',
              child: MacDropdown<int>(
                value: impl?.id.toInt(),
                hint: 'a relationship of the source',
                items: [for (final m in body.mappings) m.id.toInt()],
                labelOf: (m) =>
                    body.mappings.where((x) => x.id.toInt() == m).firstOrNull?.name ?? '?',
                onChanged: (m) => dispatch(
                  RebindPortDeclarationRequested(component: id, port: port.id.toInt(), decl: m),
                ),
              ),
            ),
            Row(
              children: [
                MacButton(
                  label: 'Retire port',
                  onPressed: () =>
                      dispatch(RetirePortRequested(component: id, port: port.id.toInt())),
                ),
              ],
            ),
          ],
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// Behaviour group
// ---------------------------------------------------------------------------

class GroupInspector extends StatelessWidget {
  const GroupInspector({super.key, required this.state, required this.id, required this.dispatch});
  final AppState state;
  final int id;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final g = state.group(id);
    final p = state.project;
    if (g == null || p == null) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final b = state.boundary(id);
    String name(int d) => p.mappings.where((m) => m.id.toInt() == d).firstOrNull?.name ?? '?';
    Widget names(Iterable<int> ids, {String empty = 'none'}) => ids.isEmpty
        ? Text(empty, style: small)
        : Wrap(
            spacing: 6,
            runSpacing: 4,
            children: [
              for (final d in ids)
                InkWell(
                  onTap: () => dispatch(SelectionChanged(MappingSelected(d))),
                  child: Text(name(d), style: TextStyle(fontSize: 12, color: t.accent)),
                ),
            ],
          );
    final box = state.editor.contextLayout.groups[id];
    final members = g.members.map((m) => m.toInt()).toList();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: 'Group',
          trailing: Text(
            '${members.length} relationship${members.length == 1 ? '' : 's'}',
            style: small,
          ),
          children: [
            FormRow(
              label: 'Name',
              child: CommitTextField(
                value: g.name,
                onCommit: (v) => dispatch(RenameGroupRequested(id: id, name: v)),
              ),
            ),
            FormRow(
              label: 'Meaning',
              child: CommitTextField(
                value: g.description,
                maxLines: 3,
                onCommit: (v) => dispatch(SetGroupDescriptionRequested(id: id, description: v)),
              ),
            ),
            Text(
              g.hasComponent()
                  ? "A behavior is a way of seeing this component's source: grouping, moving, "
                        'splitting or dissolving it changes nothing about what the component '
                        'promises or does.'
                  : 'A behavior is a way of seeing the design: grouping, moving, splitting or '
                        'dissolving it changes nothing about what the design means.',
              style: small,
            ),
          ],
        ),
        InspectorSection(
          title: 'Relationships',
          children: [
            Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: MacButton(
                label: '+ Add relationship',
                onPressed: p.concepts.isEmpty
                    ? null
                    : () async {
                        final r = await showNewMappingSheet(context, p.concepts);
                        if (r != null && r.name.isNotEmpty) {
                          dispatch(
                            CreateMappingRequested(
                              name: r.name,
                              inputs: r.inputs,
                              output: r.output,
                              group: id,
                            ),
                          );
                        }
                      },
              ),
            ),
            if (members.isEmpty) Text('Empty. Drag relationships in, or add one.', style: small),
            for (final m in members)
              Padding(
                padding: const EdgeInsets.only(bottom: 4),
                child: Row(
                  children: [
                    MappingGlyph(
                      declared:
                          !(p.mappings
                                  .where((x) => x.id.toInt() == m)
                                  .firstOrNull
                                  ?.hasDefinition() ??
                              false),
                      wrong:
                          state.mappingAnalysis(m)?.status ==
                          pb.MappingStatus.MAPPING_STATUS_INVALID,
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: InkWell(
                        onTap: () => dispatch(SelectionChanged(MappingSelected(m))),
                        child: Text(name(m), style: TextStyle(fontSize: 12, color: t.textPrimary)),
                      ),
                    ),
                    IconButton(
                      icon: const Icon(Icons.close, size: 14),
                      tooltip: 'Remove from group',
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                      onPressed: () => dispatch(RemoveGroupMemberRequested(group: id, decl: m)),
                    ),
                  ],
                ),
              ),
          ],
        ),
        InspectorSection(
          title: 'Boundary',
          children: [
            if (b == null)
              Text('Computed once the analysis arrives.', style: small)
            else ...[
              FormRow(label: 'Inputs', child: names(b.externalInputs.map((d) => d.toInt()))),
              FormRow(label: 'Outputs', child: names(b.externalOutputs.map((d) => d.toInt()))),
              FormRow(label: 'Open', child: names(b.openMembers.map((d) => d.toInt()))),
              FormRow(
                label: 'Physical outputs',
                child: names(b.drivenMembers.map((d) => d.toInt())),
              ),
              FormRow(label: 'Internal', child: names(b.privateCandidates.map((d) => d.toInt()))),
              Text(
                'Read off the dependency graph: what the members read from outside, what outside '
                'reads from them. A picture of the cut, not a connection of its own.',
                style: small,
              ),
            ],
          ],
        ),
        if (!g.hasComponent())
          InspectorSection(
            title: 'Package',
            children: [
              MacButton.primary(
                label: 'Package as Reusable Component…',
                onPressed: members.isEmpty ? null : () => dispatch(ExtractionSheetOpened(id)),
              ),
              const SizedBox(height: 6),
              Text(
                'Turns the behavior into a component and one instance in its place; the design '
                'computes the same values.',
                style: small,
              ),
            ],
          )
        else
          InspectorSection(
            title: 'Package',
            children: [
              Text(
                'A behavior inside a component stays a way of seeing its source. Packaging it '
                'as a component of its own comes with nested components; until then, edit and '
                'move it freely.',
                style: small,
              ),
            ],
          ),
        InspectorSection(
          title: 'Canvas',
          children: [
            MacButton(
              label: (box?.collapsed ?? false) ? 'Expand' : 'Collapse',
              onPressed: () =>
                  dispatch(GroupCollapsedChanged(id: id, collapsed: !(box?.collapsed ?? false))),
            ),
          ],
        ),
        InspectorSection(
          title: 'Remove',
          children: [
            Wrap(
              spacing: 8,
              runSpacing: 6,
              children: [
                MacButton(label: 'Ungroup', onPressed: () => dispatch(UngroupRequested(id))),
                DestructiveButton(
                  label: 'Delete group and relationships',
                  onPressed: () => dispatch(DeleteGroupWithMembersRequested(id)),
                ),
              ],
            ),
            Text('Ungroup keeps every relationship where it is.', style: small),
          ],
        ),
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Several nodes at once
// ---------------------------------------------------------------------------

/// A multi-selection: what the nodes are, and — for the relationships among
/// them — *Group as Behavior*.  A quiet suggestion appears when they read
/// each other or share a timing domain; nothing is ever grouped on its own.
class MultiInspector extends StatelessWidget {
  const MultiInspector({
    super.key,
    required this.state,
    required this.nodes,
    required this.dispatch,
  });
  final AppState state;
  final Set<NodeRef> nodes;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project;
    if (p == null) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    final mappings = [
      for (final n in nodes)
        if (n.kind == NodeKind.mapping) ?p.mappings.where((m) => m.id.toInt() == n.id).firstOrNull,
    ];
    final free = mappings.where((m) => state.groupOf(m.id.toInt()) == null).toList();
    final others = nodes.length - mappings.length;
    // A suggestion, from the projection only: one produces what another
    // reads, or they update in one domain.
    final produced = mappings.map((m) => m.signature.output).toSet();
    final related = mappings.any((m) => m.signature.inputs.any(produced.contains));
    final domains = mappings.where((m) => m.hasClockId()).map((m) => m.clockId).toSet();
    final sameDomain =
        free.length > 1 && domains.length == 1 && mappings.every((m) => m.hasClockId());
    final suggest = free.length > 1 && (related || sameDomain);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InspectorSection(
          title: '${nodes.length} selected',
          children: [
            Text(
              '${mappings.length} relationship${mappings.length == 1 ? '' : 's'}'
              '${others > 0 ? ', $others other${others == 1 ? '' : 's'}' : ''}.',
              style: TextStyle(fontSize: 12, color: t.textPrimary),
            ),
            for (final m in mappings)
              Padding(
                padding: const EdgeInsets.only(top: 3),
                child: InkWell(
                  onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
                  child: Text(m.name, style: TextStyle(fontSize: 12, color: t.accent)),
                ),
              ),
          ],
        ),
        if (state.isSystem && free.isNotEmpty)
          InspectorSection(
            title: 'Behavior',
            children: [
              if (suggest)
                Padding(
                  padding: const EdgeInsets.only(bottom: 6),
                  child: Text(
                    related
                        ? 'These relationships read each other.'
                        : 'These relationships update in one timing domain.',
                    style: small,
                  ),
                ),
              MacButton(
                label: free.length == 1
                    ? 'Group as Behavior'
                    : 'Group ${free.length} relationships as Behavior',
                onPressed: () => dispatch(const GroupSelectionRequested()),
              ),
              if (free.length < mappings.length)
                Padding(
                  padding: const EdgeInsets.only(top: 6),
                  child: Text(
                    '${mappings.length - free.length} already in a behavior; move them from there.',
                    style: small,
                  ),
                ),
            ],
          ),
      ],
    );
  }
}
