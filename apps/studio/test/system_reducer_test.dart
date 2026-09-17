/// System-project state without a daemon: the design in view, contexts,
/// group edits that invalidate nothing, links that ask before replacing or
/// transporting, packaging's layout hand-over, and the wire layout.
library;

import 'dart:ui';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/system.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

const tilt = 0, level = 1;
const main_ = 0, aux = 1;
const tiltValue = 0, brightness = 1, mirror = 2;
const comp = 0;
const lampA = 0, lampB = 1;
const reqPort = 0, provPort = 1;

pb.ConceptView concept(int id, String name, {int? angle}) => pb.ConceptView(
  id: Int64(id),
  name: name,
  representation: pb.Representation(quantity: pb.Dim(angle: angle ?? 0)),
);

pb.MappingView mapping(int id, String name, int output, {String? formula, int? clock}) =>
    pb.MappingView(
      id: Int64(id),
      name: name,
      signature: pb.Signature(output: Int64(output)),
      definition: formula == null ? null : pb.Definition(formula: formula),
      clockId: clock == null ? null : Int64(clock),
    );

pb.ProjectProjection base() => pb.ProjectProjection(
  name: 'lamp',
  concepts: [concept(tilt, 'Tilt', angle: 1), concept(level, 'Brightness')],
  mappings: [
    mapping(tiltValue, 'tiltValue', tilt, formula: 'raw', clock: main_),
    mapping(brightness, 'brightness', level, clock: main_),
    mapping(mirror, 'mirror', level, clock: aux),
  ],
  clocks: [
    pb.ClockView(id: Int64(main_), name: 'main'),
    pb.ClockView(id: Int64(aux), name: 'aux'),
  ],
);

pb.ComponentView component() => pb.ComponentView(
  id: Int64(comp),
  name: 'AdaptiveLamp',
  clockParams: [Int64(0)],
  sharedConcepts: [pb.IdPair(local: Int64(0), system: Int64(tilt))],
  body: pb.ProjectProjection(
    concepts: [concept(0, 'Tilt', angle: 1), concept(1, 'Brightness')],
    mappings: [
      mapping(0, 'tiltValue', 0, clock: 0),
      mapping(1, 'brightness', 1, formula: '1', clock: 0),
    ],
    clocks: [pb.ClockView(id: Int64(0), name: 'tick')],
  ),
  ports: [
    pb.PortView(
      id: Int64(reqPort),
      name: 'tiltValue',
      kind: pb.PortKind.PORT_KIND_REQUIRED,
      decl: Int64(0),
      contract: pb.PortContractView(
        signature: pb.Signature(output: Int64(0)),
        outputName: 'Tilt',
        clockKind: pb.ClockContractKind.CLOCK_CONTRACT_KIND_PARAMETER,
        clockId: Int64(0),
        clockName: 'tick',
      ),
    ),
    pb.PortView(
      id: Int64(provPort),
      name: 'brightness',
      kind: pb.PortKind.PORT_KIND_PROVIDED,
      decl: Int64(1),
      contract: pb.PortContractView(
        signature: pb.Signature(output: Int64(1)),
        outputName: 'Brightness',
        clockKind: pb.ClockContractKind.CLOCK_CONTRACT_KIND_PARAMETER,
        clockId: Int64(0),
        clockName: 'tick',
      ),
    ),
  ],
);

pb.ComponentInstanceView instance(int id, String name) => pb.ComponentInstanceView(
  id: Int64(id),
  component: Int64(comp),
  name: name,
  clockBindings: [pb.IdPair(local: Int64(0), system: Int64(main_))],
);

pb.PortRefView port(int inst, int p) => pb.PortRefView(instance: Int64(inst), port: Int64(p));
pb.PortRefView baseEnd(int decl) => pb.PortRefView(baseDecl: Int64(decl));

pb.SystemView system({
  int revision = 3,
  List<pb.BindingView> bindings = const [],
  List<pb.BehaviorGroupView> groups = const [],
  int generation = 0,
}) => pb.SystemView(
  revision: Int64(revision),
  name: 'lamp',
  base: base(),
  components: [component()],
  instances: [instance(lampA, 'lampA'), instance(lampB, 'lampB')],
  bindings: bindings,
  groups: groups,
  authoringGeneration: Int64(generation),
  // lampB's private Brightness has flat identity 40
  origins: [
    pb.OriginView(
      flat: Int64(40),
      sort: pb.LocalSort.LOCAL_SORT_SEM,
      instance: Int64(lampB),
      component: Int64(comp),
      local: Int64(1),
    ),
    pb.OriginView(
      flat: Int64(41),
      sort: pb.LocalSort.LOCAL_SORT_SEM,
      instance: Int64(lampA),
      component: Int64(comp),
      local: Int64(1),
    ),
  ],
);

pb.ProjectProjection flat({int revision = 3}) => base()
  ..revision = Int64(revision)
  ..rootPath = '/tmp/lamp'
  ..kind = pb.ProjectKind.PROJECT_KIND_SYSTEM;

AppState connected() => AppState(
  connection: Connected(executable: 'x', handshake: pb.HandshakeResponse(compatible: true)),
);

/// A system project as Studio holds it after open, system and analysis.
AppState opened({
  List<pb.BindingView> bindings = const [],
  List<pb.BehaviorGroupView> groups = const [],
}) {
  var s = reduce(connected(), ProjectReceived(flat())).state;
  s = reduce(
    s,
    SystemReceived(system(bindings: bindings, groups: groups), fromRequest: false),
  ).state;
  s = reduce(s, AnalysisReceived(pb.ProjectAnalysis(revision: Int64(3), causal: true))).state;
  s = reduce(s, SystemAnalysisReceived(pb.SystemAnalysisView(revision: Int64(3)))).state;
  return s;
}

void main() {
  group('the design in view', () {
    test('a system project opens on its top level and asks for the system', () {
      final t = reduce(connected(), ProjectReceived(flat()));
      expect(t.state.flat!.kind, pb.ProjectKind.PROJECT_KIND_SYSTEM);
      expect(t.state.project!.mappings, isEmpty, reason: 'nothing until the system arrives');
      expect(t.effects.whereType<GetSystem>(), hasLength(1));
      expect(t.effects.whereType<RunSystemAnalysis>(), hasLength(1));
      final s = reduce(t.state, SystemReceived(system(), fromRequest: false)).state;
      expect(s.isSystem, isTrue);
      expect(s.project!.mappings.map((m) => m.name), ['tiltValue', 'brightness', 'mirror']);
      expect(s.project!.revision, s.flat!.revision);
    });

    test('a stale system is dropped; a newer flat design asks again', () {
      var s = opened();
      s = reduce(s, SystemReceived(system(revision: 2), fromRequest: false)).state;
      expect(s.system!.revision.toInt(), 3);
      final t = reduce(s, ProjectReceived(flat(revision: 4)));
      expect(t.state.system, isNull);
      expect(t.state.project!.mappings, isEmpty);
      expect(t.effects.whereType<GetSystem>(), hasLength(1));
    });

    test('the component context shows the body, in its own ids, and scopes drafts', () {
      var s = opened();
      s = reduce(s, DefinitionDraftChanged(mappingId: brightness, source: 'tiltValue')).state;
      expect(s.draft(brightness), isNotNull);
      final t = reduce(s, const ContextChanged(ComponentContext(comp)));
      s = t.state;
      expect(s.editor.context, const ComponentContext(comp));
      expect(s.project!.mappings.map((m) => m.name), ['tiltValue', 'brightness']);
      expect(s.project!.clocks.single.name, 'tick');
      expect(s.draft(brightness), isNull, reason: 'the system context\'s draft is stashed');
      expect(s.editor.stashedDrafts.keys, contains('/tmp/lamp'));
      expect(s.editor.componentScope, comp);
      final d = reduce(s, DefinitionDraftChanged(mappingId: 1, source: 'tiltValue * 2'));
      final check = d.effects.whereType<AnalyzeDraft>().single;
      expect(check.component, comp, reason: 'judged in the body\'s scope');
      s = reduce(d.state, const ContextChanged(SystemContext())).state;
      expect(s.draft(brightness)?.source, 'tiltValue', reason: 'restored on the way back');
      expect(s.editor.stashedDrafts.keys, contains('/tmp/lamp#component:$comp'));
    });

    test('a flat edit on a system project is the design in view\'s edit', () {
      final s = opened();
      final sys = reduce(s, const RenameMappingRequested(id: brightness, name: 'b'));
      final op = sys.effects.whereType<ApplySystemEdit>().single.op;
      expect(op.hasBase(), isTrue);
      final body = reduce(
        reduce(s, const ContextChanged(ComponentContext(comp))).state,
        const RenameMappingRequested(id: 1, name: 'b'),
      );
      final op2 = body.effects.whereType<ApplySystemEdit>().single.op;
      expect(op2.hasEditComponentBody(), isTrue);
      expect(op2.editComponentBody.component.toInt(), comp);
    });
  });

  group('groups', () {
    test('a group edit is sent without a revision and its answer keeps everything', () {
      var s = opened();
      s = s.copyWith(
        editor: s.editor.copyWith(
          simulation: s.editor.simulation.copyWith(
            revision: 3,
            samples: [pb.TickSample(tick: Int64(0))],
          ),
        ),
      );
      final t = reduce(s, const CreateGroupRequested(name: 'Lamp', members: [brightness]));
      expect(t.effects.whereType<ApplyGroupEdit>(), hasLength(1));
      expect(t.effects.whereType<ApplySystemEdit>(), isEmpty);
      final answered = reduce(
        t.state,
        SystemReceived(
          system(
            generation: 1,
            groups: [
              pb.BehaviorGroupView(id: Int64(7), name: 'Lamp', members: [Int64(brightness)]),
            ],
          ),
        ),
      );
      expect(answered.state.analysis, isNotNull);
      expect(answered.state.systemAnalysis, isNotNull);
      expect(answered.state.editor.simulation.samples, hasLength(1));
      expect(answered.state.editor.pendingRequests, 0);
      expect(answered.state.group(7)!.name, 'Lamp');
      expect(answered.state.groupOf(brightness)!.id.toInt(), 7);
      expect(answered.effects, isEmpty);
    });

    test('collapse and box are layout, sent as layout; a gone group loses its box', () {
      var s = opened(
        groups: [pb.BehaviorGroupView(id: Int64(7), name: 'Lamp')],
      );
      final t = reduce(s, const GroupCollapsedChanged(id: 7, collapsed: true));
      expect(t.effects.whereType<SetLayout>().single.layout.groups.single.collapsed, isTrue);
      expect(t.effects.whereType<ApplySystemEdit>(), isEmpty);
      s = reduce(t.state, const GroupBoxChanged(id: 7, rect: Rect.fromLTWH(1, 2, 300, 200))).state;
      expect(s.editor.contextLayout.groups[7]!.rect, const Rect.fromLTWH(1, 2, 300, 200));
      expect(s.editor.contextLayout.groups[7]!.collapsed, isTrue);
      final gone = reduce(s, SystemReceived(system(generation: 2)));
      expect(gone.state.editor.layouts.system.groups, isEmpty);
      expect(gone.effects.whereType<SetLayout>(), hasLength(1));
    });

    test('"+ Add relationship" joins the group once the compiler confirms', () {
      final s = opened(
        groups: [pb.BehaviorGroupView(id: Int64(7), name: 'Lamp')],
      );
      final t = reduce(
        s,
        const CreateMappingRequested(name: 'gain', inputs: [], output: level, group: 7),
      );
      expect(t.state.editor.pendingGroupFor, 7);
      final applied = reduce(
        t.state,
        SystemEditApplied(
          system: system(
            revision: 4,
            groups: [pb.BehaviorGroupView(id: Int64(7), name: 'Lamp')],
          ),
          project: flat(revision: 4),
          outcome: pb.SystemEditOutcome(inner: pb.EditOutcome(createdMapping: Int64(9))),
        ),
      );
      final add = applied.effects.whereType<ApplyGroupEdit>().single.op.addMember;
      expect(add.group.toInt(), 7);
      expect(add.decl.toInt(), 9);
      expect(applied.state.editor.pendingGroupFor, isNull);
    });
  });

  group('links between binding ends', () {
    test('a free destination in the same domain is bound at once', () {
      final s = opened();
      final t = reduce(
        s,
        LinkEndsRequested(source: baseEnd(tiltValue), destination: port(lampA, reqPort)),
      );
      final op = t.effects.whereType<ApplySystemEdit>().single.op.bindPorts;
      expect(op.source.baseDecl.toInt(), tiltValue);
      expect(op.destination.instance.toInt(), lampA);
      expect(op.hasTransportInit(), isFalse);
      expect(t.state.editor.pendingBind, isNull);
    });

    test('a taken destination asks; confirming disconnects, then connects', () {
      final s = opened(
        bindings: [
          pb.BindingView(
            id: Int64(5),
            source: port(lampA, provPort),
            destination: baseEnd(brightness),
          ),
        ],
      );
      final t = reduce(
        s,
        LinkEndsRequested(source: port(lampB, provPort), destination: baseEnd(brightness)),
      );
      expect(t.effects, isEmpty, reason: 'never a silent replace');
      expect(t.state.editor.pendingBind!.replaces, 5);
      expect(t.state.editor.pendingBind!.needsTransport, isFalse);
      final confirmed = reduce(t.state, const PendingBindConfirmed());
      final unbind = confirmed.effects.whereType<ApplySystemEdit>().single.op;
      expect(unbind.unbindPorts.binding.toInt(), 5);
      expect(confirmed.state.editor.queuedSystemEdits, hasLength(1));
      final after = reduce(
        confirmed.state,
        SystemEditApplied(
          system: system(revision: 4),
          project: flat(revision: 4),
          outcome: pb.SystemEditOutcome(),
        ),
      );
      final bind = after.effects.whereType<ApplySystemEdit>().single;
      expect(bind.baseRevision, 4);
      expect(bind.op.bindPorts.source.instance.toInt(), lampB);
      expect(after.state.editor.queuedSystemEdits, isEmpty);
      final cancelled = reduce(t.state, const PendingBindCancelled());
      expect(cancelled.state.editor.pendingBind, isNull);
    });

    test('different timing domains ask for an initial value; the binding carries it', () {
      final s = opened();
      // lampA.brightness updates in main (parameter tick → main); mirror in aux
      final t = reduce(
        s,
        LinkEndsRequested(source: port(lampA, provPort), destination: baseEnd(mirror)),
      );
      final b = t.state.editor.pendingBind!;
      expect(b.needsTransport, isTrue);
      expect(b.sourceDomain, 'main');
      expect(b.destinationDomain, 'aux');
      final confirmed = reduce(t.state, const PendingBindConfirmed(transportInit: '0'));
      expect(confirmed.effects.whereType<ApplySystemEdit>().single.op.bindPorts.transportInit, '0');
    });
  });

  group('packaging', () {
    test('the sheet previews with its choices and the instance takes the group\'s place', () {
      var s = opened(
        groups: [
          pb.BehaviorGroupView(id: Int64(7), name: 'Lamp', members: [Int64(brightness)]),
        ],
      );
      s = reduce(s, const NodeMoved(NodeRef.mapping(brightness), Offset(50, 60))).state;
      s = reduce(s, const GroupBoxChanged(id: 7, rect: Rect.fromLTWH(30, 40, 200, 100))).state;
      var t = reduce(s, const ExtractionSheetOpened(7));
      expect(t.state.editor.extraction!.name, 'Lamp');
      expect(t.effects.whereType<PreviewExtraction>().single.generation, 1);
      t = reduce(
        t.state,
        const ExtractionChoicesChanged(name: 'AdaptiveLamp', keepInternal: {brightness}),
      );
      final preview = t.effects.whereType<PreviewExtraction>().single;
      expect(preview.generation, 2);
      expect(preview.choices.name, 'AdaptiveLamp');
      expect(preview.choices.keepInternal.map((d) => d.toInt()), [brightness]);
      // an older preview is ignored; the current one lands
      s = reduce(
        t.state,
        ExtractionPreviewReceived(generation: 1, preview: pb.ExtractionPreviewView(name: 'old')),
      ).state;
      expect(s.editor.extraction!.preview, isNull);
      s = reduce(
        s,
        ExtractionPreviewReceived(
          generation: 2,
          preview: pb.ExtractionPreviewView(name: 'AdaptiveLamp'),
        ),
      ).state;
      expect(s.editor.extraction!.preview!.name, 'AdaptiveLamp');
      expect(s.editor.extraction!.pending, isFalse);
      t = reduce(s, const ExtractionConfirmed());
      final op = t.effects.whereType<ApplySystemEdit>().single.op.extractGroupAsComponent;
      expect(op.group.toInt(), 7);
      expect(op.choices.name, 'AdaptiveLamp');
      final applied = reduce(
        t.state,
        SystemEditApplied(
          system: system(revision: 4),
          project: flat(revision: 4),
          outcome: pb.SystemEditOutcome(createdComponent: Int64(3), createdInstance: Int64(9)),
        ),
      );
      final e = applied.state.editor;
      expect(e.extraction, isNull);
      expect(e.selection, const InstanceSelected(9));
      expect(e.layout[const NodeRef.instance(9)], const Offset(30, 40));
      expect(e.layouts.system.groups.containsKey(7), isFalse);
      expect(e.layouts.components[3]!.nodes[const NodeRef.mapping(brightness)], const Offset(50, 60));
      expect(e.layout.containsKey(const NodeRef.mapping(brightness)), isFalse);
      expect(applied.effects.whereType<SetLayout>(), hasLength(1));
    });

    test('a placed instance lands where the designer pointed and opens for naming', () {
      final s = opened();
      final t = reduce(
        s,
        const CreateInstanceRequested(component: comp, name: 'lampC', position: Offset(9, 9)),
      );
      expect(t.state.editor.pendingPlacement, const Offset(9, 9));
      final applied = reduce(
        t.state,
        SystemEditApplied(
          system: system(revision: 4),
          project: flat(revision: 4),
          outcome: pb.SystemEditOutcome(createdInstance: Int64(2)),
        ),
      );
      expect(applied.state.editor.layout[const NodeRef.instance(2)], const Offset(9, 9));
      expect(applied.state.editor.renaming, const NodeRef.instance(2));
      expect(applied.state.editor.selection, const InstanceSelected(2));
    });
  });

  group('the system canvas', () {
    test('instance nodes come from contracts; identity is the system\'s', () {
      final s = opened(
        bindings: [
          pb.BindingView(
            id: Int64(5),
            source: baseEnd(tiltValue),
            destination: port(lampA, reqPort),
          ),
        ],
      );
      final scene = buildScene(
        s.project!,
        s.editor.layout,
        system: SystemSceneInput(system: s.system, analysis: s.systemAnalysis),
      );
      final a = scene.nodes.firstWhere((n) => n.ref == const NodeRef.instance(lampA));
      final b = scene.nodes.firstWhere((n) => n.ref == const NodeRef.instance(lampB));
      expect(a.subtitle, 'AdaptiveLamp');
      expect(a.timing, 'main');
      final aReq = a.sockets.firstWhere((x) => x.ref.index == reqPort);
      final aProv = a.sockets.firstWhere((x) => x.ref.index == provPort);
      final bProv = b.sockets.firstWhere((x) => x.ref.index == provPort);
      expect(aReq.ref.concept, tilt, reason: 'a shared concept is the system concept');
      expect(aReq.open, isFalse, reason: 'bound');
      expect(
        b.sockets.firstWhere((x) => x.ref.index == reqPort).open,
        isTrue,
        reason: 'nobody binds it',
      );
      expect(aProv.ref.concept, 41);
      expect(bProv.ref.concept, 40, reason: 'a private concept is its own identity per instance');
      expect(aProv.ref.concept != bProv.ref.concept, isTrue);
      // the binding is a link, from the base relationship's output to the port
      final link = scene.links.firstWhere((l) => l.binding == 5);
      expect(link.from.node, const NodeRef.mapping(tiltValue));
      expect(link.to, aReq.ref);
      // an open base relationship has a realisation socket; a defined one does not
      final open = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(brightness));
      final defined = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(tiltValue));
      expect(open.sockets.any((x) => x.ref.role == SocketRole.realise), isTrue);
      expect(defined.sockets.any((x) => x.ref.role == SocketRole.realise), isFalse);
      // typing at the pointer: same identity, binding ends only
      final realise = open.sockets.firstWhere((x) => x.ref.role == SocketRole.realise).ref;
      final levelConceptOut = scene.nodes
          .firstWhere((n) => n.ref == const NodeRef.concept(level))
          .sockets
          .firstWhere((x) => x.ref.side == SocketSide.output)
          .ref;
      expect(
        canLink(bProv.ref, realise),
        isFalse,
        reason: 'lampB\'s private Brightness is not Brightness',
      );
      expect(canLink(levelConceptOut, realise), isFalse, reason: 'a concept is not a binding end');
      final tiltOut = defined.sockets.firstWhere((x) => x.ref.side == SocketSide.output).ref;
      expect(canLink(tiltOut, b.sockets.firstWhere((x) => x.ref.index == reqPort).ref), isTrue);
      expect(realise.bindingEnd!.baseDecl.toInt(), brightness);
      expect(tiltOut.bindingEnd!.baseDecl.toInt(), tiltValue);
      expect(aReq.ref.bindingEnd!.port.toInt(), reqPort);
    });

    test('a bound copy shows what it takes and a transport is a gate', () {
      final s = opened(
        bindings: [
          pb.BindingView(
            id: Int64(5),
            source: port(lampA, provPort),
            destination: baseEnd(mirror),
            transportInit: '0',
          ),
        ],
      );
      final scene = buildScene(
        s.project!,
        s.editor.layout,
        system: SystemSceneInput(system: s.system),
      );
      final m = scene.nodes.firstWhere((n) => n.ref == const NodeRef.mapping(mirror));
      expect(m.definition, '= lampA.brightness');
      expect(m.declared, isFalse);
      expect(scene.links.firstWhere((l) => l.binding == 5).transport, '0');
    });
  });

  test('the wire layout round-trips instances, groups, viewports and component canvases', () {
    final layouts = CanvasLayout(
      system: ContextLayout(
        nodes: {
          const NodeRef.mapping(1): const Offset(1, 2),
          const NodeRef.instance(3): const Offset(4, 5),
        },
        groups: {7: const GroupBox(rect: Rect.fromLTWH(1, 2, 3, 4), collapsed: true)},
        viewport: const CanvasViewport(pan: Offset(9, 8), zoom: 0.5),
      ),
      components: {
        2: ContextLayout(
          nodes: {const NodeRef.concept(0): const Offset(6, 7)},
          groups: {9: const GroupBox(rect: Rect.fromLTWH(5, 6, 7, 8))},
        ),
      },
    );
    final back = layoutFromPb(layoutToPb(layouts));
    expect(back.system, layouts.system);
    expect(back.components, layouts.components);
    expect(back.components[2]!.groups[9]!.rect, const Rect.fromLTWH(5, 6, 7, 8));
  });
}
