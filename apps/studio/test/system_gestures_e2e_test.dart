/// The human interaction surface of behaviour authoring, driven by real
/// pointer and keyboard events over the real reducer, executor and `bdld`
/// (brief §27–§31): box select, Group as Behavior, drag in and out,
/// collapse / move / expand, package, fan-out and disconnect, no silent
/// replace, a transport's initial value, aggregate sockets as proxies, and
/// a group inside a component's source.  No step dispatches its final
/// reducer action directly; project *setup* does (it is not what is under
/// test).
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/system_sheets.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/canvas_harness.dart';

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

  late LiveStore store;
  late Directory dir;
  late String root;
  late CanvasPointer canvas;

  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();
  int clockId(String name) =>
      store.state.project!.clocks.firstWhere((c) => c.name == name).id.toInt();
  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);
  Future<AppState> analysed(WidgetTester tester) => canvas.settle(
    store,
    (s) =>
        s.editor.pendingRequests == 0 &&
        s.systemAnalysis != null &&
        s.systemAnalysis!.revision == s.flat!.revision &&
        s.system != null &&
        s.system!.revision == s.flat!.revision,
  );

  /// A mapping node's header, in scene coordinates.
  Offset header(AppState s, String name) =>
      canvas.scene(s).node(NodeRef.mapping(mappingId(name))).header.center;

  Future<void> mapping(
    WidgetTester tester,
    String name,
    String? formula,
    int output, {
    List<int> inputs = const [],
    String clock = 'main',
    required Offset at,
  }) async {
    await act(CreateMappingRequested(name: name, inputs: inputs, output: output));
    final id = mappingId(name);
    await act(SetMappingClockRequested(mappingId: id, clockId: clockId(clock)));
    if (formula != null) {
      await act(
        DefinitionDraftChanged(mappingId: id, source: formula),
        (s) => s.draft(id)?.check == DraftCheck.checked,
      );
      await act(CommitDefinitionRequested(id), (s) => s.committedDefinition(id) == formula);
    }
    // the Sem block, with its mapping block beside it (ADR-0044)
    await act(
      NodesMoved({
        NodeRef.mapping(id): at,
        if (formula != null) NodeRef.definition(id): attachedBlockPosition(at),
      }),
    );
  }

  Future<void> project(WidgetTester tester) async {
    tester.view.physicalSize = const Size(2400, 1400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    store = LiveStore(spawn: DaemonClient.spawn, executable: bdld!);
    canvas = CanvasPointer(tester);
    await tester.pumpWidget(DesignHarness(store: store));
    await act(
      const AppStarted(),
      (s) => s.connection is Connected || s.connection is ConnectionFailed,
    );
    expect(store.state.connection, isA<Connected>());
    dir =
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-gestures')) ??
        Directory.systemTemp;
    root = p.join(dir.path, 'lamp');
    await act(
      NewProjectRequested(rootPath: root, name: 'lamp'),
      (s) => s.project != null && s.system != null,
    );
    await act(
      CreateConceptRequested(
        name: 'Tilt',
        representation: pb.Representation(quantity: pb.Dim(angle: 1)),
      ),
    );
    await act(
      CreateConceptRequested(
        name: 'Brightness',
        representation: pb.Representation(quantity: pb.Dim()),
      ),
    );
    await act(const CreateClockDomainRequested('main'));
    await act(const CreateClockDomainRequested('aux'));
    final tilt = conceptId('Tilt');
    final level = conceptId('Brightness');
    await mapping(tester, 'raw', null, tilt, at: const Offset(400, 40));
    await mapping(tester, 'tiltValue', 'raw', tilt, at: const Offset(400, 400));
    // Sem blocks (ADR-0043): dimByTilt reads tiltValue by name; brightness
    // reads dimByTilt.  A rule template would not be a node to box-select.
    await mapping(tester, 'dimByTilt', 'tiltValue / 90 deg', level, at: const Offset(700, 40));
    await mapping(tester, 'brightness', 'dimByTilt', level, at: const Offset(700, 200));
    await mapping(tester, 'indicator', 'brightness', level, at: const Offset(1000, 40));
    await mapping(tester, 'extra', 'brightness', level, at: const Offset(1000, 300));
    await mapping(tester, 'mirror', null, level, at: const Offset(1300, 40));
    await mapping(tester, 'mirror2', null, level, at: const Offset(1300, 200));
    await mapping(tester, 'slow', null, level, clock: 'aux', at: const Offset(1300, 360));
    await analysed(tester);
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  testWidgets(
    'the designer flow, by pointer and keyboard',
    (tester) async {
      await project(tester);
      try {
        var s = store.state;
        final revision = s.revision;

        // 1. box select raw, dimByTilt, brightness (⇧-drag on empty canvas)
        await canvas.drag(const Offset(380, 20), const Offset(920, 290));
        await tester.pump();
        s = store.state;
        expect(s.editor.selection, isA<MultiSelected>());
        final picked = (s.editor.selection as MultiSelected).mappings.toSet();
        expect(picked, {mappingId('raw'), mappingId('dimByTilt'), mappingId('brightness')});

        // 2. Group as Behavior from the contextual menu; the name opens inline
        await canvas.rightClick(header(s, 'brightness'));
        await canvas.menu('Group as Behavior (3 relationships)');
        s = await canvas.settle(
          store,
          (s) => s.groupsInView.isNotEmpty && s.editor.pendingRequests == 0,
        );
        final group = s.groupsInView.single;
        expect(group.name, 'Behavior');
        expect(group.members.map((m) => m.toInt()).toSet(), picked);
        expect(group.hasComponent(), isFalse, reason: 'the system\'s own design');
        expect(s.revision, revision, reason: 'grouping is not a revision');
        expect(s.editor.renaming, NodeRef.group(group.id.toInt()));
        final inline = find.descendant(
          of: find.byType(NodeCanvas),
          matching: find.byType(TextField),
        );
        expect(inline, findsOneWidget);
        await tester.enterText(inline, 'Lamp');
        await tester.testTextInput.receiveAction(TextInputAction.done);
        s = await canvas.settle(store, (s) => s.group(group.id.toInt())?.name == 'Lamp');
        expect(s.project!.dirty, isTrue, reason: 'a group edit needs saving');

        // 3. drag a fourth relationship into the region: it joins
        await canvas.drag(header(s, 'extra'), const Offset(600, 130));
        s = await canvas.settle(
          store,
          (s) => s.group(group.id.toInt())!.members.any((m) => m.toInt() == mappingId('extra')),
        );
        expect(s.editor.layout[NodeRef.mapping(mappingId('extra'))]!.dy, lessThan(200));

        // 4. drag one out: it leaves; nothing semantic moved
        final flatBefore = s.flat!.mappings.map((m) => m.writeToBuffer()).toList();
        await canvas.drag(header(s, 'extra'), const Offset(1010, 600));
        s = await canvas.settle(
          store,
          (s) => s.group(group.id.toInt())!.members.every((m) => m.toInt() != mappingId('extra')),
        );
        expect(s.flat!.mappings.map((m) => m.writeToBuffer()).toList(), flatBefore);
        expect(s.revision, revision);

        // 5. collapse from the region's own menu
        var scene = canvas.scene(s);
        final region = scene.groups.single;
        await canvas.rightClick(region.titleBand.center);
        await canvas.menu('Collapse');
        await tester.pump();
        s = store.state;
        final box = s.editor.contextLayout.groups[group.id.toInt()]!;
        expect(box.collapsed, isTrue);
        expect(
          box.rect.topLeft,
          region.rect.topLeft,
          reason: 'the box starts where the region was',
        );
        scene = canvas.scene(s);
        final boxNode = scene.node(NodeRef.group(group.id.toInt()));
        expect(
          boxNode.sockets
              .where((x) => x.ref.side == SocketSide.input)
              .map((x) => boxNode.socketLabels[x.ref]),
          containsAll(['tiltValue', 'raw']),
        );

        // 6. drag the collapsed group: hidden members travel with it
        final dimBefore = s.editor.layout[NodeRef.mapping(mappingId('dimByTilt'))]!;
        await canvas.drag(boxNode.header.center, boxNode.header.center + const Offset(200, 400));
        await tester.pump();
        s = store.state;
        final moved = s.editor.contextLayout.groups[group.id.toInt()]!;
        final delta = moved.rect.topLeft - box.rect.topLeft;
        expect(delta.distance, greaterThan(300), reason: 'the box moved (minus the drag slop)');
        expect(
          s.editor.layout[NodeRef.mapping(mappingId('dimByTilt'))],
          dimBefore + delta,
          reason: 'hidden members travel with the box',
        );

        // 7. expand: the members show where the group now is
        scene = canvas.scene(s);
        await canvas.rightClick(scene.node(NodeRef.group(group.id.toInt())).header.center);
        await canvas.menu('Expand');
        await tester.pump();
        s = store.state;
        expect(s.editor.contextLayout.groups[group.id.toInt()]!.collapsed, isFalse);
        scene = canvas.scene(s);
        expect(scene.groups.single.rect.contains(dimBefore + delta + const Offset(20, 20)), isTrue);
        expect(s.revision, revision, reason: 'layout is never a revision');

        // 11–13. package from the region's menu: an open member kept internal, then Package
        await canvas.rightClick(scene.groups.single.titleBand.center);
        await canvas.menu('Package as Reusable Component…');
        s = await canvas.settle(store, (s) => s.editor.extraction?.preview != null);
        expect(
          s.editor.extraction!.preview!.required.map((x) => x.name),
          containsAll(['tiltValue', 'raw']),
        );
        await tester.tap(find.text('Keep internal'));
        await tester.pump();
        s = await canvas.settle(
          store,
          (s) =>
              s.editor.extraction?.preview != null &&
              !s.editor.extraction!.pending &&
              s.editor.extraction!.keepInternal.isNotEmpty,
        );
        expect(s.editor.extraction!.preview!.required.map((x) => x.name), ['tiltValue']);
        await tester.tap(
          find.descendant(of: find.byType(ExtractionSheet), matching: find.text('Package')),
        );
        s = await canvas.settle(
          store,
          (s) => s.system!.instances.isNotEmpty && s.editor.pendingRequests == 0,
        );
        s = await analysed(tester);
        expect(s.editor.extraction, isNull);
        expect(s.groupsInView, isEmpty);
        final inst = s.system!.instances.single;
        final comp = s.system!.components.single;
        expect(s.editor.selection, InstanceSelected(inst.id.toInt()));
        final prov = comp.ports.firstWhere((x) => x.name == 'brightness');
        expect(prov.kind, pb.PortKind.PORT_KIND_PROVIDED);
        expect(
          comp.ports.firstWhere((x) => x.name == 'raw').kind,
          pb.PortKind.PORT_KIND_PROVIDED,
          reason: 'tiltValue outside reads raw',
        );

        // 8. fan-out by pointer: the provided port to two open relationships
        scene = canvas.scene(s);
        SocketShape provSocket() => canvas
            .scene(store.state)
            .socket(
              NodeRef.instance(inst.id.toInt()),
              side: SocketSide.output,
              index: prov.id.toInt(),
            );
        SocketShape realise(String name) => canvas
            .scene(store.state)
            .socket(
              NodeRef.mapping(mappingId(name)),
              side: SocketSide.input,
              role: SocketRole.realise,
            );
        await canvas.drag(provSocket().center, realise('mirror').center);
        s = await canvas.settle(
          store,
          (s) => s.system!.bindings.length == 4 && s.editor.pendingRequests == 0,
        );
        await canvas.drag(provSocket().center, realise('mirror2').center);
        s = await canvas.settle(
          store,
          (s) => s.system!.bindings.length == 5 && s.editor.pendingRequests == 0,
        );
        final intoMirror = s.system!.bindings.where(
          (b) =>
              b.destination.hasBaseDecl() && b.destination.baseDecl.toInt() == mappingId('mirror'),
        );
        final intoMirror2 = s.system!.bindings.where(
          (b) =>
              b.destination.hasBaseDecl() && b.destination.baseDecl.toInt() == mappingId('mirror2'),
        );
        expect(intoMirror, hasLength(1));
        expect(intoMirror2, hasLength(1));
        expect(intoMirror.single.source.instance, inst.id);
        // disconnect mirror by dragging its bound socket away; mirror2 stays
        await canvas.drag(realise('mirror').center, const Offset(1500, 700));
        s = await canvas.settle(
          store,
          (s) => s.system!.bindings.length == 4 && s.editor.pendingRequests == 0,
        );
        expect(
          s.system!.bindings.any(
            (b) =>
                b.destination.hasBaseDecl() &&
                b.destination.baseDecl.toInt() == mappingId('mirror2'),
          ),
          isTrue,
        );
        expect(
          s.system!.bindings.any(
            (b) =>
                b.destination.hasBaseDecl() &&
                b.destination.baseDecl.toInt() == mappingId('mirror'),
          ),
          isFalse,
        );

        // 9. a second source for the taken destination asks first
        SocketShape outOf(String name) => canvas
            .scene(store.state)
            .socket(NodeRef.mapping(mappingId(name)), side: SocketSide.output);
        await canvas.drag(outOf('indicator').center, realise('mirror2').center);
        await tester.pump();
        s = store.state;
        expect(s.editor.pendingBind, isNotNull, reason: 'never a silent replace');
        expect(find.text('Replace the connection?'), findsOneWidget);
        await tester.tap(find.text('Disconnect and Connect'));
        s = await canvas.settle(
          store,
          (s) =>
              s.editor.pendingRequests == 0 &&
              s.editor.queuedSystemEdits.isEmpty &&
              s.system!.bindings.any(
                (b) =>
                    b.source.hasBaseDecl() && b.source.baseDecl.toInt() == mappingId('indicator'),
              ),
        );
        expect(s.system!.bindings, hasLength(4));

        // 10. across timing domains: the initial value is asked for, then carried
        await canvas.drag(provSocket().center, realise('slow').center);
        await tester.pump();
        expect(find.text('Carry across timing domains'), findsOneWidget);
        await tester.enterText(
          find.descendant(of: find.byType(PendingBindSheet), matching: find.byType(TextField)),
          '0',
        );
        await tester.pump();
        await tester.tap(find.text('Connect'));
        s = await canvas.settle(
          store,
          (s) => s.system!.bindings.length == 5 && s.editor.pendingRequests == 0,
        );
        expect(s.system!.bindings.where((b) => b.hasTransportInit()).single.transportInit, '0');

        // §30. aggregate sockets are proxies: a collapsed group of readers
        final tilt = conceptId('Tilt');
        await mapping(tester, 'consumerA', 'tiltValue', tilt, at: const Offset(400, 700));
        await mapping(tester, 'consumerB', '30 deg', tilt, at: const Offset(400, 820));
        await mapping(tester, 'follower', 'consumerA', tilt, at: const Offset(800, 700));
        await mapping(tester, 'openTilt', null, tilt, at: const Offset(800, 860));
        s = await analysed(tester);
        await canvas.drag(const Offset(380, 680), const Offset(640, 910));
        await tester.pump();
        s = store.state;
        expect((s.editor.selection as MultiSelected).mappings.toSet(), {
          mappingId('consumerA'),
          mappingId('consumerB'),
        });
        await canvas.rightClick(header(s, 'consumerB'));
        await canvas.menu('Group as Behavior (2 relationships)');
        s = await canvas.settle(
          store,
          (s) => s.groupsInView.isNotEmpty && s.editor.pendingRequests == 0,
        );
        final readers = s.groupsInView.single;
        final inlineReaders = find.descendant(
          of: find.byType(NodeCanvas),
          matching: find.byType(TextField),
        );
        await tester.enterText(inlineReaders, 'Readers');
        await tester.testTextInput.receiveAction(TextInputAction.done);
        s = await canvas.settle(store, (s) => s.group(readers.id.toInt())?.name == 'Readers');
        scene = canvas.scene(store.state);
        await canvas.rightClick(scene.groups.single.titleBand.center);
        await canvas.menu('Collapse');
        await tester.pump();
        s = store.state;
        scene = canvas.scene(s);
        final readersBox = scene.node(NodeRef.group(readers.id.toInt()));
        final aggIn = readersBox.sockets.firstWhere((x) => x.ref.side == SocketSide.input);
        final aggOut = readersBox.sockets.firstWhere((x) => x.ref.side == SocketSide.output);
        expect(readersBox.socketLabels[aggIn.ref], 'tiltValue');
        expect(readersBox.socketLabels[aggOut.ref], 'consumerA');
        // output proxy: the drop binds the concrete relationship, never the group
        await canvas.drag(aggOut.center, realise('openTilt').center);
        s = await canvas.settle(
          store,
          (s) => s.system!.bindings.length == 6 && s.editor.pendingRequests == 0,
        );
        final viaProxy = s.system!.bindings.firstWhere(
          (b) =>
              b.destination.hasBaseDecl() &&
              b.destination.baseDecl.toInt() == mappingId('openTilt'),
        );
        expect(viaProxy.source.hasBaseDecl(), isTrue);
        expect(viaProxy.source.baseDecl.toInt(), mappingId('consumerA'));
        // input proxy: the aggregate input stands for consumerA's read of
        // tiltValue — a name in its formula (ADR-0043): a Sem block dropped
        // on it reaches no socket that takes a link, and nothing changes
        final aBefore = s.project!.mappings
            .firstWhere((m) => m.name == 'consumerA')
            .writeToBuffer();
        final bBefore = s.project!.mappings
            .firstWhere((m) => m.name == 'consumerB')
            .writeToBuffer();
        final rawOut = canvas
            .scene(s)
            .socket(NodeRef.mapping(mappingId('raw')), side: SocketSide.output);
        final aggInNow = canvas
            .scene(s)
            .node(NodeRef.group(readers.id.toInt()))
            .sockets
            .firstWhere((x) => x.ref.side == SocketSide.input);
        await canvas.drag(rawOut.center, aggInNow.center);
        await tester.pump(const Duration(milliseconds: 100));
        s = store.state;
        expect(s.editor.pendingRequests, 0);
        expect(
          s.project!.mappings.firstWhere((m) => m.name == 'consumerA').writeToBuffer(),
          aBefore,
        );
        expect(
          s.project!.mappings.firstWhere((m) => m.name == 'consumerB').writeToBuffer(),
          bBefore,
          reason: 'no dependency for the other member',
        );
        s = await analysed(tester);
        final rb = s.boundary(readers.id.toInt())!;
        expect(rb.crossingEdges.map((e) => e.from.toInt()).toSet(), {mappingId('consumerA')});

        // §31. a behaviour inside the component's source
        final revBefore = s.revision;
        final compBefore = comp.writeToBuffer();
        final draftId = mappingId('extra');
        s = await act(
          DefinitionDraftChanged(mappingId: draftId, source: 'brightness * 2'),
          (s) => s.draft(draftId)?.check == DraftCheck.checked,
        );
        scene = canvas.scene(s);
        await canvas.doubleClick(scene.node(NodeRef.instance(inst.id.toInt())).header.center);
        await tester.pump();
        s = store.state;
        expect(s.editor.context, ComponentContext(comp.id.toInt()));
        final bodyDim = s.project!.mappings.firstWhere((m) => m.name == 'dimByTilt').id.toInt();
        final bodyBright = s.project!.mappings.firstWhere((m) => m.name == 'brightness').id.toInt();
        scene = canvas.scene(s);
        final r1 = scene.node(NodeRef.mapping(bodyDim)).rect;
        final r2 = scene.node(NodeRef.mapping(bodyBright)).rect;
        final union = r1.expandToInclude(r2).inflate(20);
        await canvas.drag(union.topLeft, union.bottomRight);
        await tester.pump();
        s = store.state;
        expect(
          (s.editor.selection as MultiSelected).mappings.toSet(),
          containsAll({bodyDim, bodyBright}),
        );
        await canvas.rightClick(scene.node(NodeRef.mapping(bodyBright)).header.center);
        await canvas.menu('Group as Behavior (2 relationships)');
        s = await canvas.settle(
          store,
          (s) => s.groupsInView.isNotEmpty && s.editor.pendingRequests == 0,
        );
        final local = s.groupsInView.single;
        expect(local.hasComponent(), isTrue);
        expect(local.component.toInt(), comp.id.toInt());
        expect(local.members.map((m) => m.toInt()).toSet(), {bodyDim, bodyBright});
        await tester.enterText(
          find.descendant(of: find.byType(NodeCanvas), matching: find.byType(TextField)),
          'Dimming',
        );
        await tester.testTextInput.receiveAction(TextInputAction.done);
        s = await canvas.settle(store, (s) => s.group(local.id.toInt())?.name == 'Dimming');
        expect(s.revision, revBefore, reason: 'no semantic revision');
        final compAfter = s.component(comp.id.toInt())!;
        expect(compAfter.writeToBuffer(), compBefore, reason: 'body, stamps and promise unchanged');
        expect(compAfter.stamp, comp.stamp);
        expect(compAfter.interfaceStamp, comp.interfaceStamp);
        expect(
          s.systemAnalysis!.revision,
          s.flat!.revision,
          reason: 'the analysis is still current',
        );
        expect(s.systemAnalysis!.components.single.realizes, isTrue);
        expect(s.system!.origins, isNotEmpty);
        // the boundary of a component group is in the body's own ids
        final lb = s.boundary(local.id.toInt())!;
        expect(lb.members.map((m) => m.toInt()).toSet(), {bodyDim, bodyBright});
        expect(lb.crossingIn.map((m) => m.toInt()), [
          s.project!.mappings.firstWhere((m) => m.name == 'tiltValue').id.toInt(),
        ]);
        // the dirty draft of the system context survived the trip
        s = await act(const ContextChanged(SystemContext()));
        expect(s.draft(draftId)?.source, 'brightness * 2');

        // save, reopen: both scopes' groups survive
        s = await act(
          const SaveRequested(),
          (s) => !s.project!.dirty && s.editor.pendingRequests == 0,
        );
        await act(const CloseProjectRequested(), (s) => s.project == null);
        s = await act(OpenProjectRequested(root), (s) => s.project != null && s.system != null);
        expect(s.system!.groups.where((g) => g.hasComponent()).single.name, 'Dimming');
        expect(s.system!.groups.where((g) => !g.hasComponent()).single.name, 'Readers');
        expect(s.editor.layouts.components[comp.id.toInt()]!.nodes, isNotEmpty);
        expect(s.editor.layouts.system.groups.keys, contains(readers.id.toInt()));
      } finally {
        await tester.runAsync(teardown);
      }
    },
    skip: bdld == null,
    timeout: const Timeout(Duration(minutes: 3)),
  );
}
