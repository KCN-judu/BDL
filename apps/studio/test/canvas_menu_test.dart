/// The contextual menu as a modal input state (docs/architecture/studio-ui.md
/// §2, "Contextual menus"), under real pointer events: the right context for
/// blank canvas, a node, a link and a selected set; retargeting while open;
/// the outside pointer that dismisses and edits nothing; the wheel, the
/// hover and Esc behind an open menu; the chooser and the menu never both
/// open; a target that disappears; macOS Control-click; and the IDE
/// service's actions in the Fix submenu — ready, a choice, blocked, stale.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/platform/desktop.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'canvas_selection_test.dart' show design, layout, c1, m0, m1, m2, o0, source, rule, value;
import 'support/canvas_harness.dart' show SceneLookup;

class Harness extends StatefulWidget {
  const Harness({
    super.key,
    required this.actions,
    this.selection = const NoSelection(),
    this.semantic,
    this.chooserFor,
  });
  final List<AppAction> actions;
  final Selection selection;
  final SemanticActionsState? semantic;

  /// A second value producing Brightness, so a concept dropped on the sink
  /// has two candidates and opens the chooser.
  final bool? chooserFor;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late Map<NodeRef, Offset> nodes = {...layout};
  late Selection selection = widget.selection;
  late pb.ProjectProjection project = design();
  late SemanticActionsState? semantic = widget.semantic;
  Offset pan = Offset.zero;
  double zoom = 1;

  void remove(NodeRef node) => setState(() {
    project = project.deepCopy()..mappings.removeWhere((m) => m.id.toInt() == node.id);
    nodes = {...nodes}..remove(node);
  });

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: NodeCanvas(
        project: project,
        layout: nodes,
        selection: selection,
        actions: semantic,
        hasSources: true,
        dispatch: (a) {
          widget.actions.add(a);
          setState(() {
            if (a is SelectionChanged) selection = a.selection;
            if (a is NodeMoved) nodes = {...nodes, a.node: a.position};
            if (a is NodesMoved) nodes = {...nodes, ...a.positions};
            if (a is ViewportChanged) {
              pan = a.pan;
              zoom = a.zoom;
            }
          });
        },
      ),
    ),
  );
}

final scene = buildScene(design(), layout);
Offset header(NodeRef ref) => scene.node(ref).header.center;
Offset origin(WidgetTester t) => t.getTopLeft(find.byType(NodeCanvas));

Future<HarnessState> pumpCanvas(
  WidgetTester tester,
  List<AppAction> actions, {
  Selection selection = const NoSelection(),
  SemanticActionsState? semantic,
}) async {
  tester.view.physicalSize = const Size(1400, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final key = GlobalKey<HarnessState>();
  await tester.pumpWidget(
    Harness(key: key, actions: actions, selection: selection, semantic: semantic),
  );
  return key.currentState!;
}

Future<void> rightClick(WidgetTester t, Offset scene) async {
  await t.tapAt(origin(t) + scene, buttons: kSecondaryButton);
  await t.pump();
  await t.pump(const Duration(milliseconds: 100));
}

Finder menuItem(String label) =>
    find.descendant(of: find.byType(MenuItemButton), matching: find.text(label));

Finder submenu(String label) =>
    find.descendant(of: find.byType(SubmenuButton), matching: find.text(label));

/// The open menu surfaces: the items on show.
Iterable<String> openMenuLabels(WidgetTester t) => t
    .widgetList<Text>(
      find.descendant(
        of: find.byWidgetPredicate((w) => w is MenuItemButton || w is SubmenuButton),
        matching: find.byType(Text),
      ),
    )
    .map((w) => w.data ?? '');

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

pb.SemanticActionView action(
  String id,
  String title,
  pb.ActionApplicability applicability, {
  List<String> options = const [],
  String reason = '',
}) => pb.SemanticActionView(
  id: id,
  title: title,
  kind: 'quick_fix',
  applicability: applicability,
  reason: reason,
  options: [for (final o in options) pb.ActionChoiceView(label: o, edit: pb.EditOp())],
  edits: [pb.EditOp()],
);

void main() {
  group('context', () {
    testWidgets('A: blank canvas opens the creation menu, nothing about an object', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const MappingSelected(rule));
      await rightClick(t, const Offset(700, 450));
      final labels = openMenuLabels(t).toList();
      expect(labels, containsAll(['Add Concept', 'Add Source', 'Select All', 'Frame All']));
      expect(labels, isNot(contains('Rename')));
      expect(labels, isNot(contains('Delete dimByTilt')));
      // the selection was not touched by the blank right-click
      expect(actions.whereType<SelectionChanged>(), isEmpty);
    });

    testWidgets('B: a node opens its own commands, and no insertion', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await rightClick(t, header(m1));
      expect(actions.whereType<SelectionChanged>().single.selection, const MappingSelected(rule));
      final labels = openMenuLabels(t).toList();
      expect(
        labels,
        containsAll(['Edit Definition', 'Rename', 'Reveal in Code', 'Delete dimByTilt']),
      );
      expect(labels, isNot(contains('Add Concept')));
      // the primary command comes first
      expect(labels.first, 'Edit Definition');
      await t.tap(menuItem('Edit Definition'));
      await t.pump();
      expect(actions.whereType<EditDefinitionRequested>().single.mappingId, rule);
    });

    testWidgets('a Source has no definition to edit; a sink names its driver', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await rightClick(t, header(m0));
      var labels = openMenuLabels(t).toList();
      expect(labels, isNot(contains('Edit Definition')));
      expect(labels, containsAll(['Rename', 'Reveal in Code', 'Delete tiltSensor']));
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      await rightClick(t, header(o0));
      labels = openMenuLabels(t).toList();
      expect(labels, contains('Show Driver: brightness'));
      await t.tap(menuItem('Show Driver: brightness'));
      await t.pump();
      expect(actions.whereType<SelectionChanged>().last.selection, const MappingSelected(value));
    });

    testWidgets('C: a link opens the link menu: its ends and Disconnect, no insertion', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      final drive = scene.links.firstWhere((l) => l.to.node == o0);
      await rightClick(t, drive.midpoint);
      final labels = openMenuLabels(t).toList();
      expect(labels, containsAll(['Show brightness', 'Show light', 'Disconnect']));
      expect(labels, isNot(contains('Add Concept')));
      // the right-click selected the edge, as it selects a node
      expect(actions.whereType<SelectionChanged>().single.selection, LinkSelected(drive.id));
      await t.tap(menuItem('Disconnect'));
      await t.pump();
      // one semantic path for every disconnect: the reducer turns it into
      // the drive edit
      final off = actions.whereType<DisconnectLinkRequested>().single;
      expect(off.link, drive.id);
      expect(off.link.disconnectable, isTrue);
    });

    testWidgets('D/E: on a selected member the set is kept and the menu is about it; on an '
        'unselected object the selection becomes that object', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: MultiSelected({m0, m1, m2}, active: m1));
      await rightClick(t, header(m0));
      expect(actions.whereType<SelectionChanged>(), isEmpty, reason: 'the set is kept');
      final labels = openMenuLabels(t).toList();
      expect(labels, contains('Delete 3 objects'));
      expect(labels, isNot(contains('Rename')), reason: 'not a single-object command');
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      await rightClick(t, header(o0));
      expect(actions.whereType<SelectionChanged>().single.selection, const OutputSelected(0));
      expect(openMenuLabels(t), contains('Delete light'));
    });

    testWidgets('F: a right-click elsewhere while a menu is open retargets it', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await rightClick(t, header(m1));
      expect(openMenuLabels(t), contains('Delete dimByTilt'));
      await rightClick(t, header(m0));
      // one menu, about the new target, and nothing of the old one
      await t.pump(const Duration(milliseconds: 100));
      final labels = openMenuLabels(t).toList();
      expect(labels, contains('Delete tiltSensor'));
      expect(labels, isNot(contains('Delete dimByTilt')));
      expect(labels.where((l) => l == 'Rename'), hasLength(1));
      expect(actions.whereType<SelectionChanged>().last.selection, const MappingSelected(source));
    });
  });

  group('modal input', () {
    testWidgets('G: the outside click closes the menu and neither selects nor pans', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const MappingSelected(rule));
      await rightClick(t, header(m1));
      expect(openMenuLabels(t), isNotEmpty);
      actions.clear();
      await t.tapAt(origin(t) + header(m0));
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), isEmpty, reason: 'dismissed');
      expect(actions, isEmpty, reason: 'the dismissing click edits nothing');
      // the next click is an ordinary one
      await t.pump(kDoubleClickInterval);
      await t.tapAt(origin(t) + header(m0));
      await t.pump(const Duration(milliseconds: 40));
      expect(actions.whereType<SelectionChanged>().single.selection, const MappingSelected(source));
    });

    testWidgets('H: an outside drag closes the menu and moves nothing, pans nothing', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions);
      await rightClick(t, header(m1));
      actions.clear();
      await drag(t, header(m0), header(m0) + const Offset(120, 90));
      expect(openMenuLabels(t), isEmpty);
      expect(actions.whereType<NodeMoved>(), isEmpty);
      expect(actions.whereType<NodesMoved>(), isEmpty);
      expect(actions.whereType<SelectionChanged>(), isEmpty);
      expect(h.nodes[m0], layout[m0]);
      await rightClick(t, header(m1));
      actions.clear();
      await drag(t, const Offset(700, 450), const Offset(500, 300));
      expect(actions.whereType<ViewportChanged>(), isEmpty, reason: 'no pan');
      expect(actions.whereType<SelectionChanged>(), isEmpty, reason: 'no marquee');
      expect(h.pan, Offset.zero);
    });

    testWidgets('I: the wheel behind an open menu does not zoom the canvas', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions);
      await rightClick(t, header(m1));
      final pointer = TestPointer(1, PointerDeviceKind.mouse);
      pointer.hover(origin(t) + const Offset(700, 450));
      await t.sendEventToBinding(pointer.scroll(const Offset(0, 40)));
      await t.pump();
      expect(actions.whereType<ViewportChanged>(), isEmpty);
      expect(h.zoom, 1);
      // the menu is still up
      expect(openMenuLabels(t), isNotEmpty);
    });

    testWidgets('J: hovering behind an open menu does not change the node state', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await rightClick(t, header(m1));
      final pointer = TestPointer(1, PointerDeviceKind.mouse);
      await t.sendEventToBinding(pointer.hover(origin(t) + header(m0)));
      await t.pump();
      final paint = t.widget<CustomPaint>(
        find.descendant(of: find.byType(NodeCanvas), matching: find.byType(CustomPaint)).first,
      );
      expect((paint.painter as dynamic).hovered, isNull);
    });

    testWidgets('K: Esc closes the menu first; the selection stays; a second Esc clears it', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions, selection: const MappingSelected(rule));
      await rightClick(t, header(m1));
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), isEmpty);
      expect(actions.whereType<SelectionChanged>(), isEmpty);
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump();
      expect(actions.whereType<SelectionChanged>().single.selection, const NoSelection());
    });

    testWidgets('M: a menu whose target is deleted closes', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions);
      await rightClick(t, header(m1));
      expect(openMenuLabels(t), contains('Delete dimByTilt'));
      h.remove(m1);
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), isEmpty);
    });

    testWidgets('N: Control + primary click opens the menu where Ctrl is not the modifier', (
      t,
    ) async {
      final actions = <AppAction>[];
      final was = primaryModifierIsControl;
      primaryModifierIsControl = false;
      addTearDown(() => primaryModifierIsControl = was);
      await pumpCanvas(t, actions);
      await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await t.tapAt(origin(t) + header(m1));
      await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), contains('Delete dimByTilt'));
    });

    testWidgets('keyboard: ↓ moves through the rows, ⏎ runs the row', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(t, actions);
      await rightClick(t, header(m0));
      // tiltSensor: Rename (focused), Reveal in Code, (divider), Delete
      // tiltSensor — one step down is Reveal in Code
      await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pump(const Duration(milliseconds: 100));
      expect(actions.whereType<RevealInCodeRequested>().single.node, m0);
      expect(openMenuLabels(t), isEmpty);
    });
  });

  group('chooser', () {
    /// A second value producing Brightness: the concept dropped on the sink
    /// has two candidates.
    pb.ProjectProjection twoCandidates() => design()
      ..mappings.removeWhere((m) => m.hasDrivesOutputId())
      ..mappings.add(
        pb.MappingView(
          id: Int64(2),
          name: 'brightness',
          signature: pb.Signature(output: Int64(1)),
          definition: pb.Definition(formula: '0.5'),
          role: pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE,
        ),
      )
      ..mappings.add(
        pb.MappingView(
          id: Int64(7),
          name: 'dimmer',
          signature: pb.Signature(output: Int64(1)),
          definition: pb.Definition(formula: '0.2'),
          role: pb.RelationshipRole.RELATIONSHIP_ROLE_VALUE,
        ),
      );

    testWidgets('L: the chooser and the menu are never both open', (t) async {
      final actions = <AppAction>[];
      final h = await pumpCanvas(t, actions);
      h.project = twoCandidates();
      h.nodes = {...layout, const NodeRef.mapping(7): const Offset(600, 200)};
      // ignore: invalid_use_of_protected_member
      h.setState(() {});
      await t.pump();
      final s = buildScene(h.project, h.nodes);
      final conceptOut = s.socket(c1, side: SocketSide.output).center;
      final sinkIn = s.socket(o0, side: SocketSide.input).center;
      await drag(t, conceptOut, sinkIn);
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), containsAll(['Drive with brightness', 'Drive with dimmer']));
      // a right-click now: the chooser goes, the menu comes
      await rightClick(t, header(m1));
      final labels = openMenuLabels(t).toList();
      expect(labels, contains('Delete dimByTilt'));
      expect(labels, isNot(contains('Drive with brightness')));
      // and the other way: a drop while the menu is open first dismisses it
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump(const Duration(milliseconds: 100));
      await rightClick(t, header(m1));
      await drag(t, conceptOut, sinkIn);
      await t.pump(const Duration(milliseconds: 100));
      expect(openMenuLabels(t), isEmpty, reason: 'the drag only dismissed the menu');
      expect(actions.whereType<SetMappingDriveRequested>(), isEmpty);
    });
  });

  group('fixes', () {
    SemanticActionsState semanticFor(int revision) => SemanticActionsState(
      entity: pb.EntityRef(mappingId: Int64(rule)),
      revision: revision,
      generation: 1,
      pending: false,
      actions: [
        action('ready:1', 'Apply Tilt', pb.ActionApplicability.ACTION_APPLICABILITY_READY),
        action(
          'choice:1',
          'Read a concept',
          pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE,
          options: ['Tilt', 'Brightness'],
        ),
        action(
          'blocked:1',
          'Insert explicit sync',
          pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED,
          reason: 'the surface has no sync phrase',
        ),
      ],
    );

    testWidgets('ready runs, a choice is a submenu, blocked is disabled with its reason', (
      t,
    ) async {
      final actions = <AppAction>[];
      await pumpCanvas(
        t,
        actions,
        selection: const MappingSelected(rule),
        semantic: semanticFor(1),
      );
      await rightClick(t, header(m1));
      expect(submenu('Fix'), findsOneWidget);
      // submenus open under the pointer, as the platform's do
      final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
      await mouse.addPointer(location: t.getCenter(menuItem('Rename')));
      await t.pump();
      await mouse.moveTo(t.getCenter(submenu('Fix')));
      await t.pump(const Duration(milliseconds: 200));
      await t.pump(const Duration(milliseconds: 200));
      expect(menuItem('Apply Tilt'), findsOneWidget);
      expect(submenu('Read a concept'), findsOneWidget);
      final blocked = t.widget<MenuItemButton>(
        find.ancestor(of: find.text('Insert explicit sync'), matching: find.byType(MenuItemButton)),
      );
      expect(blocked.onPressed, isNull, reason: 'blocked is never clickable');
      expect(find.text('the surface has no sync phrase'), findsOneWidget);
      await mouse.moveTo(t.getCenter(submenu('Read a concept')));
      await t.pump(const Duration(milliseconds: 200));
      await t.pump(const Duration(milliseconds: 200));
      await t.tap(menuItem('Brightness'));
      await t.pump();
      await mouse.removePointer();
      final applied = actions.whereType<SemanticActionApplied>().single;
      expect(applied.actionId, 'choice:1');
      expect(applied.option, 1);
    });

    testWidgets('actions of another revision, or another object, are not offered', (t) async {
      final actions = <AppAction>[];
      await pumpCanvas(
        t,
        actions,
        selection: const MappingSelected(rule),
        semantic: semanticFor(0),
      );
      await rightClick(t, header(m1));
      expect(submenu('Fix'), findsNothing, reason: 'stale revision');
      await t.sendKeyEvent(LogicalKeyboardKey.escape);
      await t.pump(const Duration(milliseconds: 100));
      await rightClick(t, header(m0));
      expect(submenu('Fix'), findsNothing, reason: 'about dimByTilt, not tiltSensor');
    });
  });
}
