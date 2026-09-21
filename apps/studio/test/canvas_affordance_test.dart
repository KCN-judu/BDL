/// Edges as objects and the contextual affordance
/// (docs/architecture/studio-ui.md §2, "Edges" and "Contextual
/// affordances"), under real pointer and keyboard events: hover reveals
/// (a cursor, a halo, no icons); a click selects and the selection
/// persists; the affordance appears only on the selected, hovered object
/// and stays while the pointer crosses to its icons; × is the one
/// semantic disconnect and is absent where the model has none (a read
/// edge is a name in the reading block's formula, ADR-0043); the menu icon
/// opens the right-click menu; Delete and the drag-away go the same way; a
/// selected edge that disappears is no selection; nodes get the same
/// pattern; and the hit test takes an edge at every zoom and the nearest
/// of two.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_affordance.dart';
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'canvas_menu_test.dart' show menuItem, openMenuLabels;
import 'canvas_selection_test.dart' show design, layout, refs, m0, m1, m2, d1, d2, o0, rule, value;
import 'support/canvas_harness.dart' show SceneLookup;

class Harness extends StatefulWidget {
  const Harness({
    super.key,
    required this.actions,
    this.selection = const NoSelection(),
    this.zoom,
  });
  final List<AppAction> actions;
  final Selection selection;
  final double? zoom;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late Map<NodeRef, Offset> nodes = {...layout};
  late Selection selection = widget.selection;
  late pb.ProjectProjection project = design();

  /// The sink's driver is taken away elsewhere (the inspector, the text).
  void undrive() => setState(() {
    project = project.deepCopy()
      ..mappings.firstWhere((m) => m.id.toInt() == value).clearDrivesOutputId();
  });

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: NodeCanvas(
        project: project,
        layout: nodes,
        selection: selection,
        refs: refs,
        hasSources: true,
        viewport: widget.zoom == null ? null : CanvasViewport(pan: Offset.zero, zoom: widget.zoom!),
        dispatch: (a) {
          widget.actions.add(a);
          setState(() {
            if (a is SelectionChanged) selection = a.selection;
            if (a is NodeMoved) nodes = {...nodes, a.node: a.position};
            if (a is NodesMoved) nodes = {...nodes, ...a.positions};
          });
        },
      ),
    ),
  );
}

final scene = buildScene(design(), layout, refs: refs);
LinkShape link(NodeRef from, NodeRef to) =>
    scene.links.firstWhere((l) => l.from.node == from && l.to.node == to);

/// brightness → light: a sink's driver, disconnectable.
final drive = link(m2, o0);

/// tiltSensor → dimByTilt's mapping block: a read edge — a name in
/// dimByTilt's formula, taken away by the compiler's unreference.
final read = link(m0, d1);

/// dimByTilt → brightness's mapping block: the other read edge.
final read2 = link(m1, d2);

/// dimByTilt's mapping block → dimByTilt: the definition itself, never
/// cut alone.
final produce = link(d1, m1);

Offset origin(WidgetTester t) => t.getTopLeft(find.byType(NodeCanvas));
Offset header(NodeRef ref) => scene.node(ref).header.center;

Future<HarnessState> pumpCanvas(
  WidgetTester tester,
  List<AppAction> actions, {
  Selection selection = const NoSelection(),
  double? zoom,
}) async {
  tester.view.physicalSize = const Size(1400, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final key = GlobalKey<HarnessState>();
  await tester.pumpWidget(Harness(key: key, actions: actions, selection: selection, zoom: zoom));
  return key.currentState!;
}

/// A mouse that hovers at [scene] (local pixels when [zoom] is 1).
Future<TestGesture> hover(WidgetTester t, Offset scene) async {
  final g = await t.createGesture(kind: PointerDeviceKind.mouse);
  await g.addPointer(location: origin(t) + scene);
  addTearDown(g.removePointer);
  await t.pump();
  await g.moveTo(origin(t) + scene);
  await t.pump();
  return g;
}

Future<void> click(WidgetTester t, Offset scene) async {
  await t.tapAt(origin(t) + scene);
  await t.pump();
}

Future<void> rightClick(WidgetTester t, Offset scene) async {
  await t.tapAt(origin(t) + scene, buttons: kSecondaryButton);
  await t.pump();
  await t.pump(const Duration(milliseconds: 100));
}

Finder get affordance => find.byType(CanvasAffordance);
Finder icon(String label) => find.byTooltip(label);

MouseCursor canvasCursor(WidgetTester t) => t
    .widget<MouseRegion>(
      find.descendant(of: find.byType(NodeCanvas), matching: find.byType(MouseRegion)).first,
    )
    .cursor;

Future<void> drag(WidgetTester tester, Offset from, Offset to) async {
  final o = origin(tester);
  final g = await tester.startGesture(o + from, kind: PointerDeviceKind.mouse);
  await tester.pump(const Duration(milliseconds: 20));
  for (var i = 1; i <= 6; i++) {
    await g.moveTo(o + Offset.lerp(from, to, i / 6)!);
    await tester.pump(const Duration(milliseconds: 16));
  }
  await g.up();
  await tester.pump(const Duration(milliseconds: 20));
}

void main() {
  group('hover', () {
    testWidgets('an edge under the pointer says so by the cursor alone: no icons, no selection', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      expect(canvasCursor(t), SystemMouseCursors.basic);
      await hover(t, drive.midpoint);
      expect(canvasCursor(t), SystemMouseCursors.click);
      expect(affordance, findsNothing);
      expect(actions.whereType<SelectionChanged>(), isEmpty);
    });
  });

  group('selection', () {
    testWidgets('a click selects the edge by its ends, and the selection persists', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions);
      await click(t, drive.midpoint);
      expect(actions.whereType<SelectionChanged>().single.selection, LinkSelected(drive.id));
      expect(h.selection, LinkSelected(drive.id));
      // a second click on the same edge changes nothing
      await click(t, drive.midpoint);
      expect(actions.whereType<SelectionChanged>().length, 1);
    });

    testWidgets('selected with the pointer elsewhere: no icons', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      await hover(t, const Offset(700, 600));
      expect(affordance, findsNothing);
    });

    testWidgets('a selected edge that is no longer drawn is no selection', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      h.undrive();
      await t.pump();
      await t.pump();
      expect(actions.whereType<SelectionChanged>().single.selection, const NoSelection());
    });

    testWidgets('Delete and Backspace on a selected edge ask for the selection\'s delete, '
        'which the reducer makes the one disconnect', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      await click(t, drive.midpoint);
      await t.sendKeyEvent(LogicalKeyboardKey.delete);
      await t.pump();
      expect(actions.whereType<DeleteSelectionRequested>().length, 1);
      await t.sendKeyEvent(LogicalKeyboardKey.backspace);
      await t.pump();
      expect(actions.whereType<DeleteSelectionRequested>().length, 2);
    });
  });

  group('affordance on an edge', () {
    testWidgets('selected and hovered: the menu icon and, on a disconnectable edge, ×', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      await hover(t, drive.midpoint);
      expect(affordance, findsOneWidget);
      expect(icon('Connection menu'), findsOneWidget);
      expect(icon('Disconnect'), findsOneWidget);
      // anchored above the midpoint, deterministic
      final a = t.widget<CanvasAffordance>(affordance);
      expect(a.anchor, drive.midpoint);
      expect(a.rect.bottom, drive.midpoint.dy - CanvasAffordance.lift);
      expect(a.rect.center.dx, closeTo(drive.midpoint.dx, 0.01));
      // the icon order is the same everywhere: menu first, then the object's own
      expect(a.actions.map((x) => x.label).toList(), ['Connection menu', 'Disconnect']);
      expect(a.actions.last.destructive, isTrue);
    });

    testWidgets('× is the one semantic disconnect', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      await hover(t, drive.midpoint);
      await t.tap(icon('Disconnect'));
      await t.pump();
      expect(actions.whereType<DisconnectLinkRequested>().single.link, drive.id);
      // the press on the icon did not reach the canvas as a click
      expect(actions.whereType<SelectionChanged>(), isEmpty);
    });

    testWidgets('the menu icon opens the same menu the right-click does', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      await hover(t, drive.midpoint);
      await t.tap(icon('Connection menu'));
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      final fromIcon = openMenuLabels(t).toList();
      expect(fromIcon, containsAll(['Show brightness', 'Show light', 'Disconnect']));
      // the affordance is gone while the menu is open
      expect(affordance, findsNothing);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      await rightClick(t, drive.midpoint);
      expect(openMenuLabels(t).toList(), fromIcon);
      await t.tap(menuItem('Disconnect'));
      await t.pump();
      expect(actions.whereType<DisconnectLinkRequested>().single.link, drive.id);
    });

    testWidgets('a right-click on an unselected edge selects it and opens the menu directly', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await rightClick(t, read.midpoint);
      expect(actions.whereType<SelectionChanged>().single.selection, LinkSelected(read.id));
      expect(openMenuLabels(t), containsAll(['Show tiltSensor', 'Show dimByTilt']));
    });

    testWidgets('a read edge gets the menu icon and a ×: the compiler unreferences the name', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(read.id));
      expect(read.id.disconnectable, isTrue, reason: 'ComposeAction.unreference');
      await hover(t, read.midpoint);
      expect(affordance, findsOneWidget);
      expect(icon('Connection menu'), findsOneWidget);
      expect(icon('Disconnect'), findsOneWidget);
      await rightClick(t, read.midpoint);
      expect(openMenuLabels(t), containsAll(['Show tiltSensor', 'Show dimByTilt', 'Disconnect']));
    });

    testWidgets('a produce edge gets the menu icon and no ×, and its menu has no Disconnect', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(produce.id));
      expect(produce.id.disconnectable, isFalse, reason: 'the definition goes with its editor');
      await hover(t, produce.midpoint);
      expect(affordance, findsOneWidget);
      expect(icon('Connection menu'), findsOneWidget);
      expect(icon('Disconnect'), findsNothing);
      await rightClick(t, produce.midpoint);
      expect(openMenuLabels(t), isNot(contains('Disconnect')));
    });

    testWidgets('the affordance stays while the pointer crosses from the edge to an icon, '
        'and goes when the pointer leaves both', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      final g = await hover(t, drive.midpoint);
      final rect = t.widget<CanvasAffordance>(affordance).rect;
      // half-way up: off the stroke, not yet on the row
      await g.moveTo(origin(t) + Offset(drive.midpoint.dx, rect.bottom + 3));
      await t.pump();
      expect(affordance, findsOneWidget);
      await g.moveTo(origin(t) + rect.center);
      await t.pump();
      expect(affordance, findsOneWidget);
      // away from both: gone
      await g.moveTo(origin(t) + rect.center - const Offset(0, 80));
      await t.pump();
      expect(affordance, findsNothing);
    });

    testWidgets('a selection change hides it', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions, selection: LinkSelected(drive.id));
      await hover(t, drive.midpoint);
      expect(affordance, findsOneWidget);
      h.selection = const MappingSelected(rule);
      // ignore: invalid_use_of_protected_member
      h.setState(() {});
      await t.pump();
      expect(affordance, findsNothing);
    });
  });

  group('drag-away', () {
    testWidgets('dragging a driven sink\'s input into empty space is the same disconnect; a read '
        'socket dragged away asks it too', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      final sink = scene.node(o0).sockets.single;
      await drag(t, sink.center, const Offset(500, 600));
      expect(actions.whereType<DisconnectLinkRequested>().single.link, drive.id);
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty, reason: 'the reducer decides');
      final readSocket = scene.node(d1).sockets.firstWhere((s) => s.ref.role == SocketRole.read);
      await drag(t, readSocket.center, const Offset(500, 600));
      // the canvas asks the same way; the reducer makes it the compiler's
      // unreference of the name (a text edit of the formula)
      expect(actions.whereType<DisconnectLinkRequested>().length, 2);
      expect(actions.whereType<DisconnectLinkRequested>().last.link, read.id);
    });
  });

  group('affordance on a node', () {
    testWidgets(
      'a selected, hovered node: its menu and its delete; the icons act as the menu does',
      (t) async {
        final actions = <AppAction>[];
        await pumpCanvas(t, actions);
        await click(t, header(m1));
        expect(actions.whereType<SelectionChanged>().single.selection, const MappingSelected(rule));
        final g = await hover(t, header(m1));
        expect(affordance, findsOneWidget);
        final a = t.widget<CanvasAffordance>(affordance);
        expect(a.actions.map((x) => x.label).toList(), ['Node menu', 'Delete dimByTilt']);
        expect(a.alignment, AffordanceAlignment.endAbove);
        expect(a.rect.right, closeTo(scene.node(m1).rect.right, 0.01));
        await t.tap(icon('Node menu'));
        await t.pump();
        await t.pump(const Duration(milliseconds: 100));
        expect(openMenuLabels(t), containsAll(['Edit Definition', 'Rename', 'Delete dimByTilt']));
        await t.sendKeyEvent(LogicalKeyboardKey.escape);
        await t.pump();
        await g.moveTo(origin(t) + header(m1) + const Offset(1, 0));
        await t.pump();
        await t.tap(icon('Delete dimByTilt'));
        await t.pump();
        expect(actions.whereType<DeleteSelectionRequested>().length, 1);
      },
    );

    testWidgets('an unselected hovered node shows nothing; a set shows nothing', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1}, active: m1));
      final g = await hover(t, header(m1));
      expect(affordance, findsNothing);
      await g.moveTo(origin(t) + header(m2));
      await t.pump();
      expect(affordance, findsNothing);
    });

    testWidgets('the menu key opens the selection\'s menu (keyboard parity with the icon)', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await click(t, drive.midpoint);
      await t.sendKeyEvent(LogicalKeyboardKey.contextMenu);
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), containsAll(['Show brightness', 'Show light', 'Disconnect']));
    });
  });

  group('hit testing', () {
    test('an edge is taken eight pixels from its stroke at any zoom, never from afar', () {
      for (final zoom in [0.25, 0.5, 1.0, 2.0, 3.0]) {
        final tolerance = linkTolerance(zoom);
        // seven pixels off the stroke, in scene units for this zoom
        final near = drive.midpoint + Offset(0, 7 / zoom);
        final hit = hitTest(scene, near, linkHitTolerance: tolerance);
        expect(hit, isA<HitLink>(), reason: 'zoom $zoom');
        expect((hit as HitLink).link.id, drive.id);
        // forty pixels off: nothing
        final far = drive.midpoint + Offset(0, 40 / zoom);
        expect(hitTest(scene, far, linkHitTolerance: tolerance), isA<HitNothing>());
      }
    });

    test('the stroke is not widened by the hit area', () {
      // the tolerance is a hit-test number; the painter's stroke is its
      // own (2, or 3 when selected) — see _CanvasPainter.
      expect(linkTolerance(1), 8);
      expect(linkTolerance(0.25), 32);
      expect(linkTolerance(4), 6, reason: 'never narrower than the stroke');
    });

    test('of two edges in reach, the nearer one, deterministically', () {
      // the two read edges: apart at their midpoints
      final a = read;
      final b = read2;
      final nearA = nearestLink(scene, a.midpoint, 100);
      final nearB = nearestLink(scene, b.midpoint, 100);
      expect(nearA!.id, a.id);
      expect(nearB!.id, b.id);
      // the same answer every time
      for (var i = 0; i < 5; i++) {
        expect(nearestLink(scene, a.midpoint, 100)!.id, a.id);
      }
    });

    testWidgets('at a far zoom the pointer still takes the edge', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, zoom: 0.5);
      // local pixel = scene * 0.5; six pixels below the stroke on screen
      await click(t, drive.midpoint * 0.5 + const Offset(0, 6));
      expect(actions.whereType<SelectionChanged>().single.selection, LinkSelected(drive.id));
    });
  });
}
