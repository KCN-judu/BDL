/// A Design page driven by real pointer and keyboard events, over the real
/// reducer and effect executor, against whatever daemon the test supplies
/// (the real `bdld` for the designer-flow tests).  The store rebuilds the
/// page on every state change; scene coordinates are mapped to the
/// canvas's own pixels so a test can press on a socket it computed with
/// `buildScene`.
library;

import 'dart:async';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/ui/canvas/canvas_geometry.dart';
import 'package:bdl_studio/ui/canvas/node_canvas.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/pages/design_page.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'test_store.dart';

/// [TestStore] that tells a widget tree when the state moved.
class LiveStore extends TestStore {
  LiveStore({required super.spawn, super.executable, super.draftDebounce, super.initial});
  final ValueNotifier<int> tick = ValueNotifier(0);

  /// A reducer failure would otherwise vanish into the daemon's stream.
  Object? failure;

  @override
  void dispatch(AppAction action) {
    try {
      super.dispatch(action);
    } catch (e, st) {
      failure = '$e\n$st';
      rethrow;
    }
    tick.value++;
  }
}

/// The page, rebuilt from the store on every change.
class DesignHarness extends StatelessWidget {
  const DesignHarness({super.key, required this.store});
  final LiveStore store;

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: ValueListenableBuilder<int>(
      valueListenable: store.tick,
      builder: (context, _, _) => Scaffold(
        body: DesignPage(state: store.state, dispatch: store.dispatch),
      ),
    ),
  );
}

/// Pointer access to the canvas in scene coordinates.  Assumes the canvas
/// is at pan (0, 0) and zoom 1, which a fresh canvas is; the harness sets
/// no viewport.
class CanvasPointer {
  CanvasPointer(this.tester);
  final WidgetTester tester;

  Offset get origin => tester.getTopLeft(find.byType(NodeCanvas));

  Offset toGlobal(Offset scene) => origin + scene;

  CanvasScene scene(AppState s, {SystemSceneInput? input}) => buildScene(
    s.project!,
    s.editor.layout,
    system:
        input ??
        SystemSceneInput(
          system: s.editor.context is SystemContext ? s.system : null,
          analysis: s.systemAnalysis,
          groups: s.groupsInView,
          boundaries: [for (final g in s.groupsInView) ?s.boundary(g.id.toInt())],
          groupBoxes: s.editor.contextLayout.groups,
        ),
  );

  /// A real drag: down, several moves, up — never a synthesized action.
  Future<void> drag(Offset fromScene, Offset toScene, {int steps = 8}) async {
    final from = toGlobal(fromScene);
    final to = toGlobal(toScene);
    final g = await tester.startGesture(from, kind: PointerDeviceKind.mouse);
    await tester.pump(const Duration(milliseconds: 20));
    for (var i = 1; i <= steps; i++) {
      await g.moveTo(Offset.lerp(from, to, i / steps)!);
      await tester.pump(const Duration(milliseconds: 16));
    }
    await g.up();
    await tester.pump(const Duration(milliseconds: 20));
  }

  Future<void> shiftDrag(Offset fromScene, Offset toScene) async {
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
    await drag(fromScene, toScene);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  }

  /// A right-click: the menu's context is set on the press, the menu opens
  /// in the frame after (its items are built from the context first).
  Future<void> rightClick(Offset scene) async {
    await tester.tapAt(toGlobal(scene), buttons: kSecondaryButton);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
  }

  Future<void> doubleClick(Offset scene) async {
    await tester.tapAt(toGlobal(scene));
    await tester.pump(const Duration(milliseconds: 60));
    await tester.tapAt(toGlobal(scene));
    await tester.pump(const Duration(milliseconds: 100));
  }

  Future<void> menu(String label) async {
    final item = find.descendant(of: find.byType(MenuItemButton), matching: find.text(label));
    expect(item, findsOneWidget, reason: 'menu item "$label"');
    await tester.tap(item);
    await tester.pump(const Duration(milliseconds: 100));
  }

  /// Dispatch inside the real event loop (an effect that reaches the daemon
  /// must start there), then wait for [test].
  Future<AppState> act(LiveStore store, AppAction action, bool Function(AppState) test) async {
    await tester.runAsync(() async => store.dispatch(action));
    return settle(store, test);
  }

  /// Let the real daemon answer, then rebuild.  An effect started by a
  /// gesture runs in the test's fake zone: its continuation after the
  /// daemon's reply is a microtask there, so the two loops are drained in
  /// turn — real I/O, then a pump — until [test] holds.
  Future<AppState> settle(LiveStore store, bool Function(AppState) test) async {
    final deadline = DateTime.now().add(const Duration(seconds: 15));
    while (!test(store.state)) {
      if (DateTime.now().isAfter(deadline)) {
        fail(
          'timed out waiting for the state; last error: ${store.state.editor.lastError?.message}',
        );
      }
      await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 20)));
      await tester.pump();
    }
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));
    return store.state;
  }
}

/// Node and socket geometry helpers over a scene.
extension SceneLookup on CanvasScene {
  NodeShape node(NodeRef ref) => nodes.firstWhere((n) => n.ref == ref);
  SocketShape socket(NodeRef ref, {required SocketSide side, int? index, SocketRole? role}) =>
      node(ref).sockets.firstWhere(
        (s) =>
            s.ref.side == side &&
            (index == null || s.ref.index == index) &&
            (role == null || s.ref.role == role),
      );
}

Future<void> settleAsync(WidgetTester tester, FutureOr<void> Function() body) async {
  await tester.runAsync(() async => body());
  await tester.pump();
}
