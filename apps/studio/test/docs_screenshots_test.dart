/// The user guide's screenshots, taken from the real Studio against the
/// real `bdld` on the documentation fixtures (docs/fixtures/*), as
/// `docs/user-guide/screenshots/manifest.json` describes them.
///
/// Every scene is reached through the store — the same actions a click or
/// a keystroke dispatches — never through pixel coordinates: entities are
/// named, pages selected, drafts typed, ticks stepped, sheets opened by
/// their state.  The whole `StudioShell` is rendered with the app's own
/// theme and fonts at a fixed window size, and the manifest's crop is cut
/// from that frame.
///
/// The test always runs when `bdld` is built and fails loudly when a scene
/// cannot be reached (fixture does not open, a named node is missing, an
/// expected text is absent): it is the guide's UI regression probe.  It
/// writes PNGs only when `DOCS_SHOTS=1` (`just docs-shots`), together with
/// `docs/user-guide/screenshots/captured.json`, which records the commit,
/// the fixture and manifest hashes each image came from.
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/definition_editor.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/library.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/deploy_page.dart';
import 'package:bdl_studio/ui/pages/design_page.dart';
import 'package:bdl_studio/ui/pages/simulate_page.dart';
import 'package:bdl_studio/ui/shell.dart';
import 'package:bdl_studio/ui/system_inspector.dart';
import 'package:bdl_studio/ui/welcome/welcome_page.dart';
import 'package:crypto/crypto.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/canvas_harness.dart';

// ---- where things are --------------------------------------------------------

final String repoRoot = p.normalize(p.join(Directory.current.path, '..', '..'));
final String manifestPath = p.join(repoRoot, 'docs', 'user-guide', 'screenshots', 'manifest.json');
final String ledgerPath = p.join(repoRoot, 'docs', 'user-guide', 'screenshots', 'captured.json');

String? _findBdld() {
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && File(env).existsSync()) return env;
  for (final rel in ['target/debug/bdld', 'target/release/bdld']) {
    final f = p.join(repoRoot, rel);
    if (File(f).existsSync()) return f;
  }
  return null;
}

bool get writing => Platform.environment['DOCS_SHOTS'] == '1';

// ---- the store behind the shell ------------------------------------------------

/// The app's Riverpod store, driven by a [LiveStore] (real reducer, real
/// effect executor, real daemon) so the shell renders exactly what the app
/// would while the test dispatches what a designer's gestures would.
class _DocsStore extends AppStore {
  _DocsStore(this.live);
  final LiveStore live;

  @override
  AppState build() {
    void follow() => state = live.state;
    live.tick.addListener(follow);
    ref.onDispose(() => live.tick.removeListener(follow));
    return live.state;
  }

  @override
  void dispatch(AppAction action) => live.dispatch(action);
}

// ---- fonts ---------------------------------------------------------------------

/// The app's fonts: the system UI font and Menlo as macOS resolves them,
/// the Material icon font from the Flutter SDK, and the bundled wordmark
/// face.  Without them the test renderer draws boxes.
Future<void> _loadFonts() async {
  Future<void> load(String family, List<String> paths) async {
    final loader = FontLoader(family);
    var any = false;
    for (final path in paths) {
      final f = File(path);
      if (!f.existsSync()) continue;
      loader.addFont(Future.value(ByteData.sublistView(f.readAsBytesSync())));
      any = true;
    }
    // Without the real font the renderer falls back: fine for the regression
    // run on another platform, never for an image that goes into the guide.
    if (!any) {
      if (writing) fail('font $family: none of $paths exists on this machine');
      return;
    }
    await loader.load();
  }

  final flutterRoot = Platform.environment['FLUTTER_ROOT'];
  await load('.AppleSystemUIFont', ['/System/Library/Fonts/SFNS.ttf']);
  await load('Menlo', ['/System/Library/Fonts/Menlo.ttc', '/System/Library/Fonts/SFNSMono.ttf']);
  await load('MaterialIcons', [
    if (flutterRoot != null)
      p.join(
        flutterRoot,
        'bin',
        'cache',
        'artifacts',
        'material_fonts',
        'MaterialIcons-Regular.otf',
      ),
  ]);
  await load('ChakraPetch', [
    for (final w in ['Regular', 'Medium', 'SemiBold', 'Bold']) 'assets/fonts/ChakraPetch-$w.ttf',
  ]);
}

// ---- the manifest ---------------------------------------------------------------

class Manifest {
  Manifest(this.json);
  final Map<String, dynamic> json;
  Size get window => Size(
    (json['window']['width'] as num).toDouble(),
    (json['window']['height'] as num).toDouble(),
  );
  double get scale => (json['window']['scale'] as num).toDouble();
  Brightness get theme => json['theme'] == 'dark' ? Brightness.dark : Brightness.light;
  String get fixturesDir => p.join(repoRoot, json['fixtures'] as String);
  String get assetsDir => p.join(repoRoot, json['assets'] as String);
  List<Map<String, dynamic>> get screenshots =>
      (json['screenshots'] as List).cast<Map<String, dynamic>>();
}

// ---- one scene -------------------------------------------------------------------

class Scene {
  Scene(this.tester, this.store, this.entry);
  final WidgetTester tester;
  final LiveStore store;
  final Map<String, dynamic> entry;

  String get id => entry['id'] as String;
  AppState get s => store.state;
  pb.ProjectProjection get project => s.project!;

  Never wrong(String what) => fail('[$id] $what');

  // -- names → ids (the manifest names entities as the designer sees them)

  int concept(String name) =>
      project.concepts.where((c) => c.name == name).firstOrNull?.id.toInt() ??
      wrong('no concept named "$name" in ${project.name}');
  int mapping(String name) =>
      project.mappings.where((m) => m.name == name).firstOrNull?.id.toInt() ??
      wrong('no relationship named "$name" in ${project.name}');
  int output(String name) =>
      project.outputs.where((o) => o.name == name).firstOrNull?.id.toInt() ??
      wrong('no physical output named "$name" in ${project.name}');
  int device(String name) =>
      project.devices.where((d) => d.name == name).firstOrNull?.id.toInt() ??
      wrong('no device named "$name" in ${project.name}');
  int instance(String name) =>
      s.system?.instances.where((i) => i.name == name).firstOrNull?.id.toInt() ??
      wrong('no instance named "$name"');
  int component(String name) =>
      s.system?.components.where((c) => c.name == name).firstOrNull?.id.toInt() ??
      wrong('no component named "$name"');
  int group(String name) =>
      s.groupsInView.where((g) => g.name == name).firstOrNull?.id.toInt() ??
      wrong('no behavior named "$name" on this canvas');

  /// `instance.port` or a top-level relationship, as a binding end.
  pb.PortRefView end(String text) {
    final parts = text.split('.');
    if (parts.length == 1) return pb.PortRefView(baseDecl: Int64(mapping(text)));
    final inst = instance(parts[0]);
    final comp = s.system!.instances.firstWhere((i) => i.id.toInt() == inst).component.toInt();
    final port = s.component(comp)?.ports.where((x) => x.name == parts[1]).firstOrNull;
    if (port == null) wrong('${parts[0]} has no port ${parts[1]}');
    return pb.PortRefView(instance: Int64(inst), port: port.id);
  }

  /// A node by the name on its header, whatever its kind.
  NodeRef node(String name) {
    final scene = this.scene();
    final hits = [
      for (final n in scene.nodes)
        if (n.title == name) n.ref,
      for (final g in scene.groups)
        if (g.title == name) NodeRef.group(g.id),
    ];
    if (hits.isEmpty) {
      final names = scene.nodes.map((n) => n.title).join(', ');
      wrong('no node named "$name" on the canvas (nodes: $names)');
    }
    if (hits.length > 1) wrong('"$name" names ${hits.length} nodes; name the kind');
    return hits.single;
  }

  Selection selection(Map<String, dynamic> by) {
    if (by['concept'] case final String n) return ConceptSelected(concept(n));
    if (by['mapping'] case final String n) return MappingSelected(mapping(n));
    if (by['output'] case final String n) return OutputSelected(output(n));
    if (by['instance'] case final String n) return InstanceSelected(instance(n));
    if (by['component'] case final String n) return ComponentSelected(component(n));
    if (by['group'] case final String n) return GroupSelected(group(n));
    wrong('select: name a concept, mapping, output, instance, component or group');
  }

  // -- the scene the canvas draws (its geometry, in scene coordinates)

  SystemSceneInput sceneInput() {
    final sys = s.system;
    if (sys == null) return const SystemSceneInput();
    final groups = s.groupsInView;
    final boundaries = [for (final g in groups) ?s.boundary(g.id.toInt())];
    return switch (s.editor.context) {
      SystemContext() => SystemSceneInput(
        system: sys,
        analysis: s.systemAnalysis,
        groups: groups,
        boundaries: boundaries,
        groupBoxes: s.editor.contextLayout.groups,
      ),
      ComponentContext(:final id) => SystemSceneInput(
        groups: groups,
        boundaries: boundaries,
        groupBoxes: s.editor.contextLayout.groups,
        portWords: {
          for (final p in s.component(id)?.ports ?? const <pb.PortView>[])
            p.decl.toInt(): portKindWord(p.kind),
        },
      ),
    };
  }

  CanvasScene scene() => buildScene(
    project,
    s.editor.layout,
    statuses: {
      for (final m in s.contextAnalysis?.mappings ?? const <pb.MappingAnalysis>[])
        m.id.toInt(): m.status,
    },
    outputStates: {
      for (final o in s.contextAnalysis?.outputs ?? const <pb.OutputAnalysis>[])
        o.id.toInt(): o.state,
    },
    system: sceneInput(),
  );

  /// Scene → window coordinates.  The fixture pins the canvas viewport in
  /// `ui/layout.json`; a fixture without one is refused rather than guessed.
  Rect toWindow(Rect scene) {
    final vp = s.editor.contextLayout.viewport;
    if (vp == null) wrong('the fixture has no viewport in ui/layout.json for this canvas');
    final origin = tester.getTopLeft(find.byType(NodeCanvas));
    return Rect.fromLTWH(
      origin.dx + vp.pan.dx + scene.left * vp.zoom,
      origin.dy + vp.pan.dy + scene.top * vp.zoom,
      scene.width * vp.zoom,
      scene.height * vp.zoom,
    );
  }

  Rect nodeRect(String name) {
    final ref = node(name);
    final sc = scene();
    for (final n in sc.nodes) {
      if (n.ref == ref) return toWindow(n.rect);
    }
    for (final g in sc.groups) {
      if (NodeRef.group(g.id) == ref) return toWindow(g.rect);
    }
    wrong('node "$name" has no shape');
  }

  // -- waiting

  Future<AppState> settle(bool Function(AppState) test, {String? why}) async {
    final deadline = DateTime.now().add(const Duration(seconds: 20));
    while (!test(store.state)) {
      if (store.failure != null) wrong('reducer failed: ${store.failure}');
      if (DateTime.now().isAfter(deadline)) {
        wrong(
          'timed out waiting for ${why ?? 'the state'}; last error: '
          '${store.state.editor.lastError?.message}',
        );
      }
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await tester.pump();
    }
    await tester.pump();
    return store.state;
  }

  Future<AppState> act(AppAction action, bool Function(AppState) test, {String? why}) async {
    await tester.runAsync(() async => store.dispatch(action));
    return settle(test, why: why);
  }

  bool quiet(AppState s) => s.editor.pendingRequests == 0 && s.editor.queuedEdits.isEmpty;
  bool analysed(AppState s) =>
      quiet(s) &&
      s.project != null &&
      s.analysis != null &&
      s.analysis!.revision == s.project!.revision &&
      (s.system == null ||
          (s.system!.revision == s.project!.revision &&
              s.systemAnalysis != null &&
              s.systemAnalysis!.revision == s.project!.revision));

  // -- steps

  Future<void> run(Map<String, dynamic> step) async {
    if (step['page'] case final String page) {
      await act(PageSelected(_page(page)), (s) => s.editor.page == _page(page));
      return;
    }
    if (step['select'] case final Map<String, dynamic> by) {
      final sel = selection(by);
      await act(SelectionChanged(sel), (s) => s.editor.selection == sel);
      return;
    }
    if (step['context'] case final Map<String, dynamic> by) {
      final ctx = ComponentContext(component(by['component'] as String));
      await act(ContextChanged(ctx), (s) => s.editor.context == ctx && analysed(s));
      return;
    }
    if (step['draft'] case final Map<String, dynamic> d) {
      final id = mapping(d['mapping'] as String);
      await act(
        DefinitionDraftChanged(mappingId: id, source: d['source'] as String),
        (s) => s.draft(id)?.check == DraftCheck.checked,
        why: 'the draft to be checked',
      );
      return;
    }
    if (step['input'] case final Map<String, dynamic> i) {
      final id = mapping(i['mapping'] as String);
      final m = project.mappings.firstWhere((m) => m.id.toInt() == id);
      final c = project.concepts.firstWhere((c) => c.id == m.signature.output);
      final pb.Value repr;
      if (i['quantity'] case final num v) {
        repr = pb.Value(
          quantity: pb.Quantity(dim: c.representation.quantity, value: v.toDouble()),
        );
      } else if (i['boolean'] case final bool b) {
        repr = pb.Value(boolean: b);
      } else {
        wrong('input: give "quantity" or "boolean"');
      }
      final value = pb.Value(
        semantic: pb.SemanticValue(conceptId: c.id, repr: repr),
      );
      await act(
        SimulationInputChanged(mappingId: id, value: value),
        (s) => s.editor.simulation.current[id] == value,
      );
      return;
    }
    if (step['step'] case final int n) {
      final before = s.editor.simulation.nextTick;
      await act(
        SimulationStepRequested(n),
        (s) => !s.editor.simulation.pending && s.editor.simulation.nextTick == before + n,
        why: 'tick ${before + n}',
      );
      if (s.editor.simulation.error case final e?) wrong('simulation failed: $e');
      return;
    }
    if (step['target'] case final String target) {
      await settle((s) => s.editor.deploy.targetsLoaded, why: 'the board list');
      await act(
        TargetSelected(target),
        (s) => s.editor.deploy.targetId == target && s.editor.deploy.analysis != null,
        why: 'the placement on $target',
      );
      return;
    }
    if (step['pin'] case final Map<String, dynamic> pin) {
      final id = device(pin['device'] as String);
      final index = pin['index'] as int;
      final resource = pin['resource'] as String?;
      final gen = s.editor.deploy.generation;
      await act(
        SetDevicePinRequested(id: id, index: index, resource: resource),
        (s) =>
            analysed(s) &&
            s.editor.deploy.analysis != null &&
            !s.editor.deploy.pending &&
            s.editor.deploy.generation > gen,
        why: 'the placement after fixing pin $index',
      );
      return;
    }
    if (step['collapse'] case final Map<String, dynamic> g) {
      final id = group(g['group'] as String);
      await act(
        GroupCollapsedChanged(id: id, collapsed: true),
        (s) => s.editor.contextLayout.groups[id]?.collapsed == true,
      );
      return;
    }
    if (step['package'] case final Map<String, dynamic> g) {
      final id = group(g['group'] as String);
      await act(
        ExtractionSheetOpened(id),
        (s) => s.editor.extraction?.preview != null && !s.editor.extraction!.pending,
        why: 'the packaging preview',
      );
      return;
    }
    if (step['link'] case final Map<String, dynamic> l) {
      final from = end(l['from'] as String);
      final to = end(l['to'] as String);
      await act(
        LinkEndsRequested(source: from, destination: to),
        (s) => s.editor.pendingBind != null || s.system!.bindings.any((b) => b.destination == to),
        why: 'the binding or its question',
      );
      return;
    }
    if (step['sidebar'] case final String tab) {
      final t = tab == 'library' ? SidebarTab.library : SidebarTab.project;
      await act(SidebarTabSelected(t), (s) => s.editor.sidebar == t);
      return;
    }
    wrong('unknown step $step');
  }

  StudioPage _page(String name) => switch (name) {
    'design' => StudioPage.design,
    'simulate' => StudioPage.simulate,
    'deploy' => StudioPage.deploy,
    _ => wrong('unknown page "$name"'),
  };

  // -- what must be on screen

  void check() {
    final exp = (entry['expect'] as Map<String, dynamic>?) ?? const {};
    for (final n in (exp['nodes'] as List?)?.cast<String>() ?? const <String>[]) {
      node(n);
    }
    for (final g in (exp['groups'] as List?)?.cast<String>() ?? const <String>[]) {
      if (!scene().groups.any((x) => x.title == g)) {
        wrong('no expanded behavior "$g" on the canvas');
      }
    }
    for (final t in (exp['texts'] as List?)?.cast<String>() ?? const <String>[]) {
      if (find.text(t, findRichText: true).evaluate().isEmpty) wrong('expected the text "$t"');
    }
    for (final t in (exp['textsContaining'] as List?)?.cast<String>() ?? const <String>[]) {
      if (find.textContaining(t, findRichText: true).evaluate().isEmpty) {
        wrong('expected a text containing "$t"');
      }
    }
    if (exp['outputComplete'] == true && s.analysis?.outputComplete != true) {
      wrong('expected every required output to be driven');
    }
    if (exp['ticks'] case final int n) {
      if (s.editor.simulation.samples.length != n) {
        wrong('expected $n ticks in the trace, found ${s.editor.simulation.samples.length}');
      }
    }
  }

  // -- the crop

  Rect crop(Map<String, dynamic> c) {
    Rect r;
    if (c['region'] case final String region) {
      r = switch (region) {
        'window' => Offset.zero & tester.view.physicalSize / tester.view.devicePixelRatio,
        'canvas' => tester.getRect(find.byType(NodeCanvas)),
        'inspector' => tester.getRect(find.byType(Inspector)),
        'sidebar' => tester.getRect(find.byType(Library)),
        'page' => tester.getRect(switch (s.editor.page) {
          StudioPage.design => find.byType(DesignPage),
          StudioPage.simulate => find.byType(SimulatePage),
          StudioPage.deploy => find.byType(DeployPage),
          StudioPage.monitor => wrong('the Monitor page is a placeholder'),
        }),
        'welcome' => tester.getRect(find.byType(WelcomePage)),
        _ => wrong('unknown region "$region"'),
      };
    } else if (c['key'] case final String key) {
      final f = find.byKey(ValueKey(key));
      if (f.evaluate().isEmpty) wrong('no widget keyed "$key" on screen');
      r = tester.getRect(f.first);
    } else if (c['widget'] case final String type) {
      final f = switch (type) {
        'DefinitionEditor' => find.byType(DefinitionEditor),
        'Inspector' => find.byType(Inspector),
        'NodeCanvas' => find.byType(NodeCanvas),
        _ => wrong('unknown widget "$type"'),
      };
      if (f.evaluate().isEmpty) wrong('no $type on screen');
      r = tester.getRect(f.first);
    } else if (c['text'] case final String text) {
      final f = find.text(text, findRichText: true);
      if (f.evaluate().isEmpty) {
        wrong('no text "$text" on screen');
      }
      r = tester.getRect(f.first);
    } else if ((c['nodes'] ?? c['groups']) case final List names) {
      // Nodes must lie on the canvas — a node under the inspector or off
      // the edge is a layout to fix in the fixture, not to crop around.
      final canvas = tester.getRect(find.byType(NodeCanvas));
      r = names.cast<String>().map(nodeRect).reduce((a, b) => a.expandToInclude(b));
      if (!(canvas.contains(r.topLeft) && canvas.contains(r.bottomRight))) {
        wrong('the nodes $names ($r) do not fit the canvas $canvas; fix the fixture layout');
      }
    } else if (c['union'] case final List parts) {
      r = parts.cast<Map<String, dynamic>>().map(crop).reduce((a, b) => a.expandToInclude(b));
    } else {
      wrong('crop: give region, key, widget, nodes, groups or union');
    }
    final pad = (c['pad'] as num?)?.toDouble() ?? 0;
    final window = Offset.zero & tester.view.physicalSize / tester.view.devicePixelRatio;
    var out = r.inflate(pad).intersect(window);
    if (c['maxHeight'] case final num h) {
      out = Rect.fromLTWH(out.left, out.top, out.width, out.height.clamp(0, h.toDouble()));
    }
    if (c['maxWidth'] case final num w) {
      out = Rect.fromLTWH(out.left, out.top, out.width.clamp(0, w.toDouble()), out.height);
    }
    if (out.isEmpty) wrong('the crop $c is off screen');
    return out;
  }
}

// ---- the test ----------------------------------------------------------------------

void main() {
  final bdld = _findBdld();
  final manifest = Manifest(
    jsonDecode(File(manifestPath).readAsStringSync()) as Map<String, dynamic>,
  );
  if (writing && bdld == null) {
    throw StateError('DOCS_SHOTS=1 but bdld is not built (cargo build -p bdl-daemon)');
  }

  setUpAll(_loadFonts);

  final ledger = <String, dynamic>{};
  if (File(ledgerPath).existsSync()) {
    ledger.addAll(jsonDecode(File(ledgerPath).readAsStringSync()) as Map<String, dynamic>);
  }

  for (final entry in manifest.screenshots) {
    final id = entry['id'] as String;
    testWidgets('screenshot $id', skip: bdld == null, (tester) async {
      final shotKey = GlobalKey();
      final logical = manifest.window;
      tester.view.physicalSize = logical * manifest.scale;
      tester.view.devicePixelRatio = manifest.scale;
      addTearDown(tester.view.reset);

      final store = LiveStore(spawn: DaemonClient.spawn, executable: bdld!);
      addTearDown(() => tester.runAsync(store.dispose));
      final scene = Scene(tester, store, entry);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [appStoreProvider.overrideWith(() => _DocsStore(store))],
          child: MaterialApp(
            debugShowCheckedModeBanner: false,
            theme: macTheme(manifest.theme),
            home: RepaintBoundary(key: shotKey, child: const StudioShell()),
          ),
        ),
      );

      // 1. start bdld
      await scene.act(const AppStarted(), (s) => s.connection is Connected, why: 'the compiler');

      // 2–3. open the fixture
      if (entry['fixture'] case final String fixture) {
        final root = p.join(manifest.fixturesDir, fixture);
        if (!Directory(root).existsSync()) scene.wrong('fixture $root does not exist');
        await scene.act(
          OpenProjectRequested(root),
          scene.analysed,
          why: 'the fixture to open and be analysed',
        );
        if (store.state.editor.lastError case final e?) {
          scene.wrong('opening refused: ${e.message}');
        }
      } else if (entry['view'] != 'welcome') {
        scene.wrong('a fixture is required for every view but "welcome"');
      }

      // 4. navigate
      if (entry['view'] case final String view when view != 'welcome') {
        await scene.run({'page': view});
        if (view == 'deploy') {
          await scene.settle((s) => s.editor.deploy.targetsLoaded, why: 'the board list');
        }
      }
      for (final step in (entry['steps'] as List).cast<Map<String, dynamic>>()) {
        await scene.run(step);
      }

      // 7. stable rendering, then the checks
      await tester.pumpAndSettle(const Duration(milliseconds: 100));
      scene.check();

      // 8. capture: the whole frame at the manifest's scale, then the crop
      final rect = scene.crop(entry['crop'] as Map<String, dynamic>);
      final boundary = shotKey.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      expect(
        boundary.localToGlobal(Offset.zero),
        Offset.zero,
        reason: 'the shell fills the window',
      );
      final image = await tester.runAsync(() => _capture(boundary, rect, manifest.scale));
      final bytes = await tester.runAsync(() => image!.toByteData(format: ui.ImageByteFormat.png));
      expect(bytes, isNotNull);

      // 9. write
      if (!writing) return;
      final out = File(p.join(manifest.assetsDir, entry['output'] as String));
      out.parent.createSync(recursive: true);
      out.writeAsBytesSync(bytes!.buffer.asUint8List());
      ledger[id] = {
        'output': entry['output'],
        'commit': await tester.runAsync(() => _git(['rev-parse', '--short=12', 'HEAD'])),
        'dirty': await tester.runAsync(
          () async => (await _git([
            'status',
            '--porcelain',
            '--',
            'apps/studio/lib',
            'crates',
            'docs/fixtures',
            'docs/user-guide/screenshots/manifest.json',
          ])).isNotEmpty,
        ),
        'captured_at': '${DateTime.now().toUtc().toIso8601String().substring(0, 19)}Z',
        'fixture_sha256': entry['fixture'] == null
            ? null
            : hashTree(p.join(manifest.fixturesDir, entry['fixture'] as String)),
        'entry_sha256': hashEntry(entry),
        'width': image!.width,
        'height': image.height,
      };
      final sorted = {for (final k in ledger.keys.toList()..sort()) k: ledger[k]};
      File(ledgerPath).writeAsStringSync('${const JsonEncoder.withIndent('  ').convert(sorted)}\n');
    });
  }
}

/// The frame at [scale] device pixels per logical pixel, cut to [rect]
/// (logical coordinates of the boundary).
Future<ui.Image> _capture(RenderRepaintBoundary boundary, Rect rect, double scale) async {
  final frame = await boundary.toImage(pixelRatio: scale);
  final recorder = ui.PictureRecorder();
  final canvas = Canvas(recorder);
  final src = Rect.fromLTWH(
    rect.left * scale,
    rect.top * scale,
    rect.width * scale,
    rect.height * scale,
  );
  canvas.drawImageRect(frame, src, Offset.zero & src.size, Paint());
  final picture = recorder.endRecording();
  final out = await picture.toImage(src.width.round(), src.height.round());
  picture.dispose();
  frame.dispose();
  return out;
}

Future<String> _git(List<String> args) async {
  final r = await Process.run('git', args, workingDirectory: repoRoot);
  if (r.exitCode != 0) fail('git ${args.join(' ')}: ${r.stderr}');
  return (r.stdout as String).trim();
}

/// A hash over a fixture's files (path and content), independent of the
/// machine — what `scripts/check_screenshots.py` recomputes.
String hashTree(String root) {
  final files = Directory(root).listSync(recursive: true).whereType<File>().toList()
    ..sort((a, b) => a.path.compareTo(b.path));
  final bytes = BytesBuilder(copy: false);
  for (final f in files) {
    bytes.add(utf8.encode('${p.relative(f.path, from: root).replaceAll(r'\', '/')}\n'));
    bytes.add(f.readAsBytesSync());
    bytes.add(utf8.encode('\n'));
  }
  return sha256.convert(bytes.takeBytes()).toString();
}

/// A hash over the manifest entry's scene (everything but its prose).
String hashEntry(Map<String, dynamic> entry) {
  final scene = {
    for (final k in ['fixture', 'view', 'steps', 'expect', 'crop', 'output']) k: entry[k],
  };
  return sha256.convert(utf8.encode(_canonical(scene))).toString();
}

String _canonical(Object? v) {
  if (v is Map) {
    final keys = v.keys.map((k) => k.toString()).toList()..sort();
    return '{${keys.map((k) => '${jsonEncode(k)}:${_canonical(v[k])}').join(',')}}';
  }
  if (v is List) return '[${v.map(_canonical).join(',')}]';
  return jsonEncode(v);
}
