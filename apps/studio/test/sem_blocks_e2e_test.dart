/// The Sem-block canvas against the real `bdld` (ADR-0044; BDL_FV Phase 21,
/// `lamp_picture`, `rule_template`, `sensors_natural`): `pressed` a Source
/// Sem block, `lit` a rule template (not a node), `litV` a Sem block whose
/// mapping block applies the rule; the canvas draws Sem blocks and sinks
/// only, with read edges from the analysis; dropping a Sem block on a
/// Source gives it a definition, dropping it on an open position fills the
/// compiler's slot — text edits, committed as one edit each; a rule
/// applied twice gives two Sem blocks of one concept, and two Sources of
/// one concept beside them, and nothing complains.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
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
  late CanvasPointer canvas;

  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();

  /// Let the real daemon answer and the fake clock advance (a draft a
  /// gesture made is checked on a timer of the test zone), then rebuild —
  /// until [test] holds.
  Future<AppState> settle(WidgetTester tester, bool Function(AppState) test) async {
    for (var i = 0; i < 600 && !test(store.state); i++) {
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await tester.pump(const Duration(milliseconds: 20));
    }
    if (!test(store.state)) {
      fail('timed out; last error: ${store.state.editor.lastError?.message}');
    }
    await tester.pump(const Duration(milliseconds: 20));
    return store.state;
  }

  Future<AppState> act(AppAction a, [bool Function(AppState)? test]) =>
      canvas.act(store, a, test ?? (s) => s.editor.pendingRequests == 0);
  Future<AppState> analysed(WidgetTester tester) => settle(
    tester,
    (s) =>
        s.editor.pendingRequests == 0 &&
        s.analysis != null &&
        s.analysis!.revision == s.project!.revision &&
        s.system != null &&
        s.system!.revision == s.flat!.revision,
  );
  pb.MappingAnalysis analysisOf(AppState s, String name) =>
      s.analysis!.mappings.firstWhere((m) => m.id.toInt() == mappingId(name));

  Future<void> block(String name, int output, {String? formula, required Offset at}) async {
    await act(CreateMappingRequested(name: name, inputs: const [], output: output));
    final id = mappingId(name);
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
    store = LiveStore(
      spawn: DaemonClient.spawn,
      executable: bdld!,
      draftDebounce: const Duration(milliseconds: 10),
    );
    canvas = CanvasPointer(tester);
    await tester.pumpWidget(DesignHarness(store: store));
    await act(
      const AppStarted(),
      (s) => s.connection is Connected || s.connection is ConnectionFailed,
    );
    expect(store.state.connection, isA<Connected>());
    dir =
        await tester.runAsync(() => Directory.systemTemp.createTemp('bdl-studio-sem')) ??
        Directory.systemTemp;
    final root = p.join(dir.path, 'lamp');
    await act(
      NewProjectRequested(rootPath: root, name: 'lamp'),
      (s) => s.project != null && s.system != null,
    );
    await act(
      CreateConceptRequested(
        name: 'Pressed',
        representation: pb.Representation(boolean: pb.Unit()),
      ),
    );
    await act(
      CreateConceptRequested(
        name: 'Lit',
        representation: pb.Representation(boolean: pb.Unit()),
      ),
    );
    final pressed = conceptId('Pressed');
    final lit = conceptId('Lit');
    // the Source, the rule template, the Sem block to wire
    await block('pressed', pressed, at: const Offset(40, 40));
    await act(CreateMappingRequested(name: 'lit', inputs: [pressed], output: lit));
    await act(
      DefinitionDraftChanged(mappingId: mappingId('lit'), source: 'Pressed'),
      (s) => s.draft(mappingId('lit'))?.check == DraftCheck.checked,
    );
    await act(
      CommitDefinitionRequested(mappingId('lit')),
      (s) => s.committedDefinition(mappingId('lit')) == 'Pressed',
    );
    await block('litV', lit, at: const Offset(420, 40));
    await analysed(tester);
  }

  Future<void> teardown(WidgetTester tester) async {
    // the daemon's shutdown handshake needs the real event loop; a failure
    // above must not turn into a two-minute hang here
    await tester.runAsync(
      () => store.dispose().timeout(const Duration(seconds: 5), onTimeout: () {}),
    );
    await tester.runAsync(() => dir.delete(recursive: true));
  }

  testWidgets(
    'the picture: Sem blocks and read edges, the rule a template; the wire is a text edit',
    (tester) async {
      await project(tester);
      try {
        var s = store.state;
        // the canvas: two Sem blocks, no concept node, no rule node
        var scene = canvas.scene(s);
        expect(scene.nodes.map((n) => n.ref).toSet(), {
          NodeRef.mapping(mappingId('pressed')),
          NodeRef.mapping(mappingId('litV')),
        });
        expect(scene.nodes.where((n) => n.ref.kind == NodeKind.concept), isEmpty);
        expect(scene.links, isEmpty, reason: 'litV names nothing yet');
        final litV = scene.node(NodeRef.mapping(mappingId('litV')));
        expect(litV.source, isTrue, reason: 'a Sem block with no definition is a Source');

        // 1. pressed dropped on litV: the definition `pressed` — one edit
        final pressedOut = scene
            .socket(NodeRef.mapping(mappingId('pressed')), side: SocketSide.output)
            .center;
        await canvas.drag(pressedOut, litV.rect.center);
        s = await settle(
          tester,
          (s) =>
              s.editor.pendingRequests == 0 &&
              s.committedDefinition(mappingId('litV')) == 'pressed' &&
              s.draft(mappingId('litV')) == null,
        );
        expect(s.editor.selection, MappingSelected(mappingId('litV')));
        expect(
          s.mapping(mappingId('litV'))!.signature.inputs,
          isEmpty,
          reason: 'never a signature edit',
        );
        s = await analysed(tester);
        expect(analysisOf(s, 'litV').references.map((d) => d.toInt()), [mappingId('pressed')]);
        scene = canvas.scene(s);
        // litV has a mapping block now, joined to it; pressed is read into it
        expect(scene.links.map((l) => l.id.kind).toSet(), {LinkKind.read, LinkKind.produce});
        final read = scene.links.singleWhere((l) => l.id.kind == LinkKind.read);
        expect(read.from.node, NodeRef.mapping(mappingId('pressed')));
        expect(read.to.node, NodeRef.definition(mappingId('litV')));
        expect(read.to.role, SocketRole.read);
        final produce = scene.links.singleWhere((l) => l.id.kind == LinkKind.produce);
        expect(produce.from.node, NodeRef.definition(mappingId('litV')));
        expect(produce.to.node, NodeRef.mapping(mappingId('litV')));

        // 2. the rule applied with an open position: the slot is the
        // compiler's; pressed dropped into it fills it, and the block
        // applies the rule
        await act(
          DefinitionDraftChanged(mappingId: mappingId('litV'), source: 'lit(?)'),
          (s) => s.draft(mappingId('litV'))?.check == DraftCheck.checked,
        );
        await act(
          CommitDefinitionRequested(mappingId('litV')),
          (s) => s.committedDefinition(mappingId('litV')) == 'lit(?)',
        );
        s = await analysed(tester);
        expect(analysisOf(s, 'litV').slots, hasLength(1));
        scene = canvas.scene(s);
        final slot = scene
            .node(NodeRef.definition(mappingId('litV')))
            .sockets
            .singleWhere((x) => x.ref.role == SocketRole.slot);
        expect(slot.open, isTrue);
        // the drop on the slot socket is the same request the reducer
        // takes here (a compose round trip from the test's fake zone would
        // not settle); the drop itself is covered by the widget tests
        await act(
          WireSemBlockRequested(
            mappingId: mappingId('litV'),
            semId: mappingId('pressed'),
            slot: slot.ref.index,
          ),
          (s) => s.editor.pendingRequests == 0,
        );
        s = await settle(
          tester,
          (s) =>
              s.editor.pendingRequests == 0 &&
              s.committedDefinition(mappingId('litV')) == 'lit(pressed)' &&
              s.draft(mappingId('litV')) == null,
        );
        s = await analysed(tester);
        expect(analysisOf(s, 'litV').references.map((d) => d.toInt()).toSet(), {
          mappingId('pressed'),
          mappingId('lit'),
        }, reason: 'reads the Sem block and applies the rule (FV Reads = DependsOn)');
        scene = canvas.scene(s);
        final node = scene.node(NodeRef.definition(mappingId('litV')));
        expect(node.dependsOn, ['pressed']);
        expect(node.applies, ['lit']);
        expect(node.title, 'lit', reason: 'the mapping block names the rule it applies');
        expect(
          scene.links.where((l) => l.id.kind == LinkKind.read).length,
          1,
          reason: 'the rule makes no edge',
        );
        expect(scene.nodes.where((n) => n.ref.id == mappingId('lit')), isEmpty);

        // 3. the rule applied twice: two Sem blocks of Lit over two Sources
        // of Pressed — ordinary, and nothing complains
        await block('pressedB', conceptId('Pressed'), at: const Offset(40, 300));
        await block('litB', conceptId('Lit'), formula: 'lit(pressedB)', at: const Offset(420, 300));
        s = await analysed(tester);
        final litBlocks = s.project!.mappings
            .where(
              (m) => m.signature.inputs.isEmpty && m.signature.output.toInt() == conceptId('Lit'),
            )
            .map((m) => m.name)
            .toSet();
        expect(litBlocks, {'litV', 'litB'});
        for (final name in ['pressed', 'pressedB', 'lit', 'litV', 'litB']) {
          expect(analysisOf(s, name).diagnostics, isEmpty, reason: name);
        }
        expect(analysisOf(s, 'lit').appliedBy.map((d) => d.toInt()).toSet(), {
          mappingId('litV'),
          mappingId('litB'),
        });
        scene = canvas.scene(s);
        expect(scene.links.where((l) => l.id.kind == LinkKind.read).length, 2);
        expect(scene.links.where((l) => l.id.kind == LinkKind.produce).length, 2);
        expect(scene.nodes.length, 6, reason: 'four Sem blocks, two mapping blocks');
      } finally {
        await teardown(tester);
      }
    },
    skip: bdld == null,
    timeout: const Timeout(Duration(minutes: 2)),
  );
}
