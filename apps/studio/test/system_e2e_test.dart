/// Behaviour authoring, grouping, packaging and composition through the
/// real Studio stack against the real `bdld` (brief §79–§82): the designer
/// flow from a top-level design to a component with two instances, with
/// every fact read off what the daemon sent.
library;

import 'dart:io';
import 'dart:ui';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/test_store.dart';

String? _findBdld() {
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && File(env).existsSync()) return env;
  for (final rel in ['../../target/debug/bdld', '../../target/release/bdld']) {
    final f = p.normalize(p.join(Directory.current.path, rel));
    if (File(f).existsSync()) return f;
  }
  return null;
}

void main() {
  final bdld = _findBdld();

  late TestStore store;
  late Directory dir;
  late String root;

  Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
  Future<AppState> analysed() => store.until(
    (s) =>
        s.analysis != null &&
        s.analysis!.revision == s.flat!.revision &&
        s.systemAnalysis != null &&
        s.systemAnalysis!.revision == s.flat!.revision &&
        s.system != null &&
        s.system!.revision == s.flat!.revision,
  );
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  int clockId(String name) =>
      store.state.project!.clocks.firstWhere((c) => c.name == name).id.toInt();

  pb.Value angle(double deg) => pb.Value(
    semantic: pb.SemanticValue(
      conceptId: Int64(store.state.flat!.concepts.firstWhere((c) => c.name == 'Tilt').id.toInt()),
      repr: pb.Value(
        quantity: pb.Quantity(dim: pb.Dim(angle: 1), value: deg * 3.141592653589793 / 180),
      ),
    ),
  );

  Future<void> defineFormula(String name, String source) async {
    final id = mappingId(name);
    store.dispatch(DefinitionDraftChanged(mappingId: id, source: source));
    await store.until((s) => s.draft(id)?.check == DraftCheck.checked);
    store.dispatch(CommitDefinitionRequested(id));
    await store.until((s) => s.committedDefinition(id) == source);
  }

  Future<void> project() async {
    store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    store.dispatch(const AppStarted());
    await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
    expect(store.state.connection, isA<Connected>());
    dir = await Directory.systemTemp.createTemp('bdl-studio-system-e2e');
    root = p.join(dir.path, 'lamp');
    store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp', kind: NewProjectKind.system));
    await store.until((s) => s.project != null && s.system != null);
    expect(store.state.flat!.kind, pb.ProjectKind.PROJECT_KIND_SYSTEM);
    expect(store.state.isSystem, isTrue);
    expect(store.state.editor.context, const SystemContext());

    // the top-level design: Tilt, Brightness, main; raw → tiltValue →
    // [dimByTilt, brightness] → indicator; light ← brightness
    store.dispatch(
      CreateConceptRequested(
        name: 'Tilt',
        representation: pb.Representation(quantity: pb.Dim(angle: 1)),
      ),
    );
    await settled();
    store.dispatch(
      CreateConceptRequested(
        name: 'Brightness',
        representation: pb.Representation(quantity: pb.Dim()),
      ),
    );
    await settled();
    store.dispatch(const CreateClockDomainRequested('main'));
    await settled();
    final tilt = conceptId('Tilt');
    final level = conceptId('Brightness');
    store.dispatch(CreateMappingRequested(name: 'raw', inputs: const [], output: tilt));
    await settled();
    store.dispatch(CreateMappingRequested(name: 'tiltValue', inputs: const [], output: tilt));
    await settled();
    store.dispatch(CreateMappingRequested(name: 'dimByTilt', inputs: [tilt], output: level));
    await settled();
    store.dispatch(CreateMappingRequested(name: 'brightness', inputs: const [], output: level));
    await settled();
    store.dispatch(CreateMappingRequested(name: 'indicator', inputs: const [], output: level));
    await settled();
    for (final m in ['raw', 'tiltValue', 'brightness', 'indicator']) {
      store.dispatch(SetMappingClockRequested(mappingId: mappingId(m), clockId: clockId('main')));
      await settled();
    }
    await defineFormula('tiltValue', 'raw');
    await defineFormula('dimByTilt', 'Tilt / 90 deg');
    await defineFormula('brightness', 'dimByTilt(tiltValue)');
    await defineFormula('indicator', 'brightness');
    store.dispatch(
      CreateOutputRequested(
        name: 'light',
        accepts: level,
        clockId: clockId('main'),
        required: true,
      ),
    );
    await store.until(
      (s) =>
          s.project!.outputs.length == 1 &&
          s.editor.pendingRequests == 0 &&
          s.editor.queuedEdits.isEmpty,
    );
    store.dispatch(
      SetMappingDriveRequested(
        mappingId: mappingId('brightness'),
        outputId: store.state.project!.outputs.single.id.toInt(),
      ),
    );
    await settled();
    await analysed();
  }

  // Simulation is of the whole (flat) design, whichever context is open.
  int flatId(String name) =>
      store.state.flat!.mappings.firstWhere((m) => m.name == name).id.toInt();

  Future<List<String>> simulateIndicator() async {
    store.dispatch(const SimulationResetRequested());
    store.dispatch(SimulationInputChanged(mappingId: flatId('raw'), value: angle(45)));
    store.dispatch(const SimulationStepRequested(3));
    final s = await store.until((s) => s.editor.simulation.samples.length == 3);
    final id = flatId('indicator');
    return [
      for (final t in s.editor.simulation.samples)
        t.values.firstWhere((v) => v.mappingId.toInt() == id).rendered,
    ];
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  test(
    'group → package → compose: the designer flow, every fact from the daemon',
    () async {
      await project();
      try {
        var s = store.state;
        expect(s.editor.lastError?.message, isNull);
        final level = conceptId('Brightness');
        expect(s.analysis!.causal, isTrue);
        final before = await simulateIndicator();
        expect(before, hasLength(3));
        final revision = s.revision;

        // ---- §70/§79 grouping: authoring metadata, no revision, nothing reset ----
        store.dispatch(
          CreateGroupRequested(
            name: 'Lamp',
            members: [mappingId('dimByTilt'), mappingId('brightness')],
          ),
        );
        s = await store.until((s) => s.system!.groups.isNotEmpty && s.editor.pendingRequests == 0);
        final group = s.system!.groups.single;
        expect(group.name, 'Lamp');
        expect(group.members.map((m) => m.toInt()), [
          mappingId('dimByTilt'),
          mappingId('brightness'),
        ]);
        expect(s.revision, revision, reason: 'a group edit is not a revision');
        expect(s.analysis, isNotNull, reason: 'the analysis is not invalidated');
        expect(s.editor.simulation.samples, hasLength(3), reason: 'the run is kept');
        expect(s.system!.authoringGeneration.toInt(), 1);
        store.dispatch(RenameGroupRequested(id: group.id.toInt(), name: 'Adaptive lamp'));
        await settled();
        store.dispatch(
          AddGroupMemberRequested(group: group.id.toInt(), decl: mappingId('indicator')),
        );
        await settled();
        store.dispatch(
          RemoveGroupMemberRequested(group: group.id.toInt(), decl: mappingId('indicator')),
        );
        s = await settled();
        expect(s.system!.groups.single.name, 'Adaptive lamp');
        expect(s.system!.groups.single.members, hasLength(2));
        expect(s.revision, revision);
        // a relationship is in at most one group: the refusal is a banner
        store.dispatch(const CreateGroupRequested(name: 'Other'));
        s = await settled();
        final other = s.system!.groups.firstWhere((g) => g.name == 'Other').id.toInt();
        store.dispatch(AddGroupMemberRequested(group: other, decl: mappingId('dimByTilt')));
        s = await store.until((s) => s.editor.lastError != null);
        expect(s.editor.lastError!.code, 'group_edit.already_grouped');
        store.dispatch(const ErrorDismissed());
        store.dispatch(UngroupRequested(other));
        s = await settled();
        expect(s.system!.groups, hasLength(1));

        // ---- the boundary is the compiler's (§71–§74), served with the analysis ----
        store.dispatch(const SelectionChanged(NoSelection()));
        s = await store.until((s) => s.boundary(group.id.toInt()) != null);
        final b = s.boundary(group.id.toInt())!;
        expect(b.crossingIn.map((d) => d.toInt()), [mappingId('tiltValue')]);
        expect(b.externalOutputs.map((d) => d.toInt()), [mappingId('brightness')]);
        expect(b.drivenMembers.map((d) => d.toInt()), [mappingId('brightness')]);
        expect(b.privateCandidates.map((d) => d.toInt()), [mappingId('dimByTilt')]);
        expect(b.openMembers, isEmpty);

        // ---- collapsed: a box with aggregate sockets, members hidden, links re-routed ----
        store.dispatch(GroupCollapsedChanged(id: group.id.toInt(), collapsed: true));
        s = store.state;
        expect(s.editor.contextLayout.groups[group.id.toInt()]!.collapsed, isTrue);
        var scene = buildScene(
          s.project!,
          s.editor.layout,
          system: SystemSceneInput(
            system: s.system,
            analysis: s.systemAnalysis,
            groups: s.groupsInView,
            boundaries: [for (final g in s.groupsInView) ?s.boundary(g.id.toInt())],
            groupBoxes: s.editor.contextLayout.groups,
          ),
        );
        final box = scene.nodes.firstWhere((n) => n.ref == NodeRef.group(group.id.toInt()));
        expect(scene.nodes.any((n) => n.ref == NodeRef.mapping(mappingId('dimByTilt'))), isFalse);
        expect(
          box.sockets
              .where((x) => x.ref.side == SocketSide.input)
              .map((x) => box.socketLabels[x.ref]),
          ['tiltValue'],
        );
        expect(
          box.sockets
              .where((x) => x.ref.side == SocketSide.output)
              .map((x) => box.socketLabels[x.ref]),
          ['brightness'],
        );
        // the aggregate socket accepts no link: it is a picture, not a port
        final aggregate = box.sockets.first.ref;
        final tiltOut = scene.nodes
            .firstWhere((n) => n.ref == NodeRef.mapping(mappingId('tiltValue')))
            .sockets
            .firstWhere((x) => x.ref.side == SocketSide.output)
            .ref;
        expect(canLink(tiltOut, aggregate), isFalse);
        // and the dependency edge into the box is drawn from tiltValue
        expect(scene.links.any((l) => l.from == tiltOut && l.to == aggregate), isTrue);
        store.dispatch(GroupCollapsedChanged(id: group.id.toInt(), collapsed: false));

        // ---- §75 packaging: preview, then one atomic edit ----
        store.dispatch(ExtractionSheetOpened(group.id.toInt()));
        s = await store.until((s) => s.editor.extraction?.preview != null);
        var preview = s.editor.extraction!.preview!;
        expect(preview.required.map((x) => x.name), ['tiltValue']);
        expect(preview.provided.map((x) => x.name), ['brightness']);
        expect(preview.sinks.single.name, 'light');
        expect(preview.sinks.single.internal, isFalse);
        expect(preview.clocks.map((c) => c.toInt()), [clockId('main')]);
        expect(s.revision, revision, reason: 'a preview changes nothing');
        store.dispatch(const ExtractionChoicesChanged(name: 'AdaptiveLamp'));
        s = await store.until(
          (s) =>
              s.editor.extraction?.preview?.name == 'AdaptiveLamp' && !s.editor.extraction!.pending,
        );
        preview = s.editor.extraction!.preview!;
        expect(preview.instanceName, 'adaptiveLamp');
        final groupPos = const Offset(300, 200);
        // the members' positions seed the component's own canvas
        store.dispatch(NodeMoved(NodeRef.mapping(mappingId('dimByTilt')), const Offset(310, 240)));
        final dimLocal = mappingId('dimByTilt');
        store.dispatch(
          GroupBoxChanged(id: group.id.toInt(), rect: Rect.fromLTWH(300, 200, 200, 100)),
        );
        store.dispatch(const ExtractionConfirmed());
        s = await store.until(
          (s) => s.system!.instances.isNotEmpty && s.editor.pendingRequests == 0,
        );
        expect(s.editor.extraction, isNull);
        expect(s.system!.groups, isEmpty);
        final comp = s.system!.components.single;
        final inst = s.system!.instances.single;
        expect(comp.name, 'AdaptiveLamp');
        expect(inst.name, 'adaptiveLamp');
        expect(s.editor.selection, isA<InstanceSelected>());
        expect(s.editor.layout[NodeRef.instance(inst.id.toInt())], groupPos);
        expect(
          s.editor.layouts.components[comp.id.toInt()]!.nodes[NodeRef.mapping(dimLocal)],
          const Offset(310, 240),
          reason: 'a member keeps its identity and its place inside the component',
        );
        expect(s.system!.bindings, hasLength(2));
        // the base: tiltValue kept, dimByTilt gone, brightness an open copy realised by a binding
        expect(s.project!.mappings.any((m) => m.name == 'dimByTilt'), isFalse);
        final copy = s.project!.mappings.firstWhere((m) => m.name == 'brightness');
        expect(copy.hasDefinition(), isFalse, reason: 'the authored base has an open copy');
        final realised = s.flat!.mappings.firstWhere((m) => m.id == copy.id);
        expect(realised.definition.whichKind(), pb.Definition_Kind.reference);
        s = await analysed();
        expect(s.systemAnalysis!.acceptance, pb.SystemAcceptance.SYSTEM_ACCEPTANCE_EXECUTABLE);
        expect(s.analysis!.causal, isTrue);
        // Theorem R witnessed on the wire: the same values, tick for tick
        final after = await simulateIndicator();
        expect(after, before);
        // the canvas: an instance node from the contract, a base→port link and a port→base link
        s = store.state;
        scene = buildScene(
          s.project!,
          s.editor.layout,
          system: SystemSceneInput(system: s.system, analysis: s.systemAnalysis),
        );
        final node = scene.nodes.firstWhere((n) => n.ref == NodeRef.instance(inst.id.toInt()));
        expect(node.subtitle, 'AdaptiveLamp');
        expect(
          node.sockets.map((x) => node.socketLabels[x.ref]),
          containsAll(['tiltValue', 'brightness']),
        );
        expect(scene.links.where((l) => l.binding != null), hasLength(2));
        expect(node.sockets.every((x) => !x.open), isTrue, reason: 'both ports are bound');

        // ---- §81 derived artifact safety: the bound copy is shown, not edited ----
        store.dispatch(SelectionChanged(MappingSelected(copy.id.toInt())));
        expect(store.state.committedDefinition(copy.id.toInt()), isNull);
        final into = s.system!.bindings.firstWhere((b) => b.destination.hasBaseDecl());
        expect(into.destination.baseDecl, copy.id);
        expect(into.source.instance, inst.id);

        // ---- component source: the body in its own scope, drafts scoped to it ----
        store.dispatch(ContextChanged(ComponentContext(comp.id.toInt())));
        s = store.state;
        expect(s.editor.context, ComponentContext(comp.id.toInt()));
        expect(
          s.project!.mappings.map((m) => m.name),
          containsAll(['tiltValue', 'dimByTilt', 'brightness']),
        );
        expect(s.project!.mappings, hasLength(3));
        expect(s.project!.revision, s.flat!.revision);
        final bodyBrightness = s.project!.mappings
            .firstWhere((m) => m.name == 'brightness')
            .id
            .toInt();
        expect(s.contextAnalysis, isNotNull, reason: 'the body has its own analysis');
        store.dispatch(
          DefinitionDraftChanged(mappingId: bodyBrightness, source: 'dimByTilt(tiltValue) * 2'),
        );
        s = await store.until((s) => s.draft(bodyBrightness)?.check == DraftCheck.checked);
        expect(s.draft(bodyBrightness)!.parseOk, isTrue);
        expect(
          s.draft(bodyBrightness)!.analysis!.status,
          pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
        );
        // completion in the body's scope offers the body's names, never flat ones
        store.dispatch(CompletionRequested(mappingId: bodyBrightness, source: 'dim', offset: 3));
        s = await store.until((s) => s.editor.completion?.pending == false);
        expect(s.editor.completion!.items.any((i) => i.label.startsWith('dimByTilt')), isTrue);
        expect(s.editor.completion!.items.any((i) => i.label.contains('.')), isFalse);
        store.dispatch(const CompletionDismissed());
        store.dispatch(CommitDefinitionRequested(bodyBrightness));
        s = await store.until(
          (s) => s.committedDefinition(bodyBrightness) == 'dimByTilt(tiltValue) * 2',
        );
        expect(
          s.editor.context,
          ComponentContext(comp.id.toInt()),
          reason: 'a body edit keeps the source open',
        );
        s = await analysed();
        final doubled = await simulateIndicator();
        expect(doubled, isNot(before), reason: 'the body edit reached the instance');
        // back to the system: the body's draft state does not leak
        store.dispatch(const ContextChanged(SystemContext()));
        s = store.state;
        expect(s.editor.context, const SystemContext());
        expect(s.project!.mappings.any((m) => m.name == 'indicator'), isTrue);

        // ---- §78/§80 a second instance, base→port fan-out, no silent replace, transport ----
        store.dispatch(
          CreateInstanceRequested(
            component: comp.id.toInt(),
            name: 'second',
            position: const Offset(700, 200),
          ),
        );
        s = await store.until(
          (s) => s.system!.instances.length == 2 && s.editor.pendingRequests == 0,
        );
        final second = s.system!.instances.firstWhere((i) => i.name == 'second');
        expect(s.editor.layout[NodeRef.instance(second.id.toInt())], const Offset(700, 200));
        expect(s.editor.selection, InstanceSelected(second.id.toInt()));
        store.dispatch(
          SetClockArgumentRequested(
            instance: second.id.toInt(),
            parameter: clockId('main'),
            clock: clockId('main'),
          ),
        );
        await settled();
        final req = comp.ports.firstWhere((x) => x.kind == pb.PortKind.PORT_KIND_REQUIRED);
        final prov = comp.ports.firstWhere((x) => x.kind == pb.PortKind.PORT_KIND_PROVIDED);
        // base tiltValue feeds the second instance too: an ordinary binding, sent at once
        store.dispatch(
          LinkEndsRequested(
            source: pb.PortRefView(baseDecl: Int64(mappingId('tiltValue'))),
            destination: pb.PortRefView(instance: second.id, port: req.id),
          ),
        );
        s = await store.until(
          (s) => s.system!.bindings.length == 3 && s.editor.pendingRequests == 0,
        );
        expect(s.editor.pendingBind, isNull);
        expect(s.editor.selection, isA<BindingSelected>());
        // an open base relationship takes a provided port; a second source asks first
        store.dispatch(CreateMappingRequested(name: 'mirror', inputs: const [], output: level));
        await settled();
        store.dispatch(
          SetMappingClockRequested(mappingId: mappingId('mirror'), clockId: clockId('main')),
        );
        await settled();
        store.dispatch(
          LinkEndsRequested(
            source: pb.PortRefView(instance: second.id, port: prov.id),
            destination: pb.PortRefView(baseDecl: Int64(mappingId('mirror'))),
          ),
        );
        s = await store.until(
          (s) => s.system!.bindings.length == 4 && s.editor.pendingRequests == 0,
        );
        store.dispatch(
          LinkEndsRequested(
            source: pb.PortRefView(instance: inst.id, port: prov.id),
            destination: pb.PortRefView(baseDecl: Int64(mappingId('mirror'))),
          ),
        );
        s = store.state;
        expect(s.editor.pendingBind, isNotNull, reason: 'never a silent replace');
        expect(s.editor.pendingBind!.replaces, isNotNull);
        expect(s.editor.pendingBind!.needsTransport, isFalse);
        store.dispatch(const PendingBindConfirmed());
        s = await store.until(
          (s) =>
              s.editor.pendingRequests == 0 &&
              s.system!.bindings.any(
                (b) =>
                    b.destination.hasBaseDecl() &&
                    b.destination.baseDecl.toInt() == mappingId('mirror') &&
                    b.source.instance == inst.id,
              ),
        );
        expect(s.system!.bindings, hasLength(4));
        // across timing domains: the link asks for an initial value, the binding carries it
        store.dispatch(const CreateClockDomainRequested('aux'));
        await settled();
        store.dispatch(CreateMappingRequested(name: 'slow', inputs: const [], output: level));
        await settled();
        store.dispatch(
          SetMappingClockRequested(mappingId: mappingId('slow'), clockId: clockId('aux')),
        );
        await settled();
        store.dispatch(
          LinkEndsRequested(
            source: pb.PortRefView(instance: inst.id, port: prov.id),
            destination: pb.PortRefView(baseDecl: Int64(mappingId('slow'))),
          ),
        );
        s = store.state;
        expect(s.editor.pendingBind!.needsTransport, isTrue);
        expect(s.editor.pendingBind!.sourceDomain, 'main');
        expect(s.editor.pendingBind!.destinationDomain, 'aux');
        store.dispatch(const PendingBindConfirmed(transportInit: '0'));
        s = await store.until(
          (s) => s.system!.bindings.length == 5 && s.editor.pendingRequests == 0,
        );
        final carried = s.system!.bindings.firstWhere((b) => b.hasTransportInit());
        expect(carried.transportInit, '0');
        s = await analysed();
        expect(
          s.systemAnalysis!.composition.where((d) => d.code.startsWith('system.transport')),
          isEmpty,
        );
        scene = buildScene(
          s.project!,
          s.editor.layout,
          system: SystemSceneInput(system: s.system, analysis: s.systemAnalysis),
        );
        expect(scene.links.where((l) => l.transport == '0'), hasLength(1));

        // ---- §80 versions: substitution accepted on interfaces, refused when a used port is gone ----
        store.dispatch(DuplicateComponentRequested(id: comp.id.toInt(), name: 'AdaptiveLamp v2'));
        s = await store.until(
          (s) => s.system!.components.length == 2 && s.editor.pendingRequests == 0,
        );
        final v2 = s.system!.components.firstWhere((c) => c.name == 'AdaptiveLamp v2');
        expect(s.editor.selection, ComponentSelected(v2.id.toInt()));
        store.dispatch(
          ReplaceInstanceComponentRequested(instance: second.id.toInt(), component: v2.id.toInt()),
        );
        s = await store.until(
          (s) => s.instance(second.id.toInt())!.component == v2.id && s.editor.pendingRequests == 0,
        );
        expect(s.system!.bindings, hasLength(5), reason: 'bindings survive a substitution');
        // a port in use (second.tiltValue is bound) cannot be retired
        store.dispatch(RetirePortRequested(component: v2.id.toInt(), port: req.id.toInt()));
        s = await store.until((s) => s.editor.lastError != null);
        expect(
          s.editor.lastError!.code,
          'system_edit.port_in_use',
          reason: 'a bound port cannot be retired',
        );
        store.dispatch(const ErrorDismissed());
        // second no longer provides anything: v2 may drop that port …
        store.dispatch(RetirePortRequested(component: v2.id.toInt(), port: prov.id.toInt()));
        s = await store.until(
          (s) => s.editor.pendingRequests == 0 && s.component(v2.id.toInt())!.ports.length == 1,
        );
        // … but then cannot stand in for adaptiveLamp, whose brightness is used
        store.dispatch(
          ReplaceInstanceComponentRequested(instance: inst.id.toInt(), component: v2.id.toInt()),
        );
        s = await store.until((s) => s.editor.lastError != null);
        expect(s.editor.lastError!.code, 'system_edit.not_substitutable');
        expect(s.instance(inst.id.toInt())!.component, comp.id);
        store.dispatch(const ErrorDismissed());
        store.dispatch(
          ReplaceInstanceComponentRequested(
            instance: second.id.toInt(),
            component: comp.id.toInt(),
          ),
        );
        await store.until((s) => s.instance(second.id.toInt())!.component == comp.id);
        store.dispatch(DeleteInstanceRequested(second.id.toInt()));
        s = await store.until((s) => s.editor.lastError != null);
        expect(s.editor.lastError!.code, 'system_edit.instance_in_use');
        store.dispatch(const ErrorDismissed());

        // ---- §79 an explicit contract change is classified; the body then fails its promise ----
        final k = prov.contract.deepCopy()
          ..clockKind = pb.ClockContractKind.CLOCK_CONTRACT_KIND_AGNOSTIC
          ..clearClockId();
        store.dispatch(
          ChangePortContractRequested(
            component: comp.id.toInt(),
            port: prov.id.toInt(),
            contract: k,
          ),
        );
        s = await store.until(
          (s) =>
              s
                  .component(comp.id.toInt())!
                  .ports
                  .firstWhere((x) => x.id == prov.id)
                  .contract
                  .clockKind ==
              pb.ClockContractKind.CLOCK_CONTRACT_KIND_AGNOSTIC,
        );
        s = await analysed();
        expect(s.systemAnalysis!.components.firstWhere((c) => c.id == comp.id).realizes, isFalse);
        expect(
          s.systemAnalysis!.composition.any((d) => d.code == 'component.port_clock_mismatch'),
          isTrue,
        );
        scene = buildScene(
          s.project!,
          s.editor.layout,
          system: SystemSceneInput(system: s.system, analysis: s.systemAnalysis),
        );
        expect(
          scene.nodes.firstWhere((n) => n.ref == NodeRef.instance(inst.id.toInt())).unrealized,
          isTrue,
        );
        store.dispatch(const UndoRequested());
        s = await store.until(
          (s) =>
              s.system != null &&
              s.system!.revision == s.flat!.revision &&
              s
                      .component(comp.id.toInt())!
                      .ports
                      .firstWhere((x) => x.id == prov.id)
                      .contract
                      .clockKind ==
                  pb.ClockContractKind.CLOCK_CONTRACT_KIND_PARAMETER,
        );
        s = await analysed();
        expect(s.systemAnalysis!.components.firstWhere((c) => c.id == comp.id).realizes, isTrue);

        // ---- persistence: system, layout, groups survive save and reopen ----
        store.dispatch(CreateGroupRequested(name: 'Rest', members: [mappingId('indicator')]));
        await settled();
        store.dispatch(const SaveRequested());
        await store.until((s) => !s.project!.dirty && s.editor.pendingRequests == 0);
        store.dispatch(const CloseProjectRequested());
        await store.until((s) => s.project == null);
        store.dispatch(OpenProjectRequested(root));
        s = await store.until((s) => s.project != null && s.system != null);
        expect(s.system!.components, hasLength(2));
        expect(s.system!.instances, hasLength(2));
        expect(s.system!.bindings, hasLength(5));
        expect(s.system!.groups.single.name, 'Rest');
        expect(s.editor.layout[NodeRef.instance(inst.id.toInt())], groupPos);
        expect(
          s.editor.layouts.components[comp.id.toInt()]!.nodes[NodeRef.mapping(dimLocal)],
          const Offset(310, 240),
        );
        expect(s.editor.context, const SystemContext());
        s = await analysed();
        // two lamps drive the system's one light: a conflict the compiler
        // names, never an arbitration
        expect(s.systemAnalysis!.acceptance, pb.SystemAcceptance.SYSTEM_ACCEPTANCE_INVALID);
        expect(s.analysis!.diagnostics.any((d) => d.code == 'output.multiple_drivers'), isTrue);
      } finally {
        await teardown();
      }
    },
    skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false,
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
