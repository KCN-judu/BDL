/// The *Save changes to “lamp”?* sheet through the real shell: raised by
/// every unloading path when the project says it is dirty — the toolbar's
/// Close, ⌘W, Open Project…, and the desktop's own exit request (⌘Q, the
/// menu's Quit, the window's close button) — never for a clean project;
/// Cancel keeps the project, Don't Save closes, Save closes after the save.
library;

import 'dart:ui' show AppExitResponse;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/effects/effect_executor.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/app.dart';
import 'package:bdl_studio/ui/close_guard.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/test_store.dart';

pb.ProjectProjection lamp({required bool dirty}) =>
    pb.ProjectProjection(revision: Int64(1), name: 'lamp', rootPath: '/p', dirty: dirty)
      ..kind = pb.ProjectKind.PROJECT_KIND_TEXT;

/// A daemon whose project is dirty until it is saved.
class _Daemon {
  _Daemon();
  bool dirty = true;
  bool saveFails = false;
  int closes = 0;
  int saves = 0;
  late final FakeDaemon link = FakeDaemon((m) {
    if (m.hasHandshake()) return okHandshake();
    if (m.hasGetProject())
      return pb.Response(
        project: pb.ProjectResponse(project: lamp(dirty: dirty)),
      );
    if (m.hasSaveProject()) {
      saves++;
      if (saveFails) return errorResponse('project.changed_on_disk', 'changed on disk');
      dirty = false;
      return pb.Response(project: pb.ProjectResponse(project: lamp(dirty: false)));
    }
    if (m.hasCloseProject()) {
      closes++;
      return pb.Response(ack: pb.Ack());
    }
    if (m.hasGetSystem()) {
      return pb.Response(
        system: pb.SystemResponse(system: pb.SystemView(revision: Int64(1))),
      );
    }
    if (m.hasOpenProject()) {
      return pb.Response(project: pb.ProjectResponse(project: lamp(dirty: false)));
    }
    return pb.Response(ack: pb.Ack());
  });
}

Future<({ProviderContainer container, _Daemon daemon, List<String> quits})> app(
  WidgetTester tester,
) async {
  tester.view.physicalSize = const Size(1600, 1000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final daemon = _Daemon();
  final quits = <String>[];
  final container = ProviderContainer(
    overrides: [
      appStoreProvider.overrideWith(
        () => AppStore((dispatch) {
          final e = EffectExecutor(
            dispatch,
            locate: () => 'fake-bdld',
            spawn: (_) async => daemon.link,
            draftDebounce: const Duration(milliseconds: 1),
          );
          e.quit = () async => quits.add('quit');
          return e;
        }),
      ),
    ],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(container: container, child: const StudioApp()),
  );
  await tester.pump();
  final store = container.read(appStoreProvider.notifier);
  await tester.runAsync(() async {
    while (container.read(appStoreProvider).connection is! Connected) {
      await Future<void>.delayed(const Duration(milliseconds: 5));
    }
  });
  store.dispatch(ProjectReceived(lamp(dirty: true)));
  store.dispatch(SystemReceived(pb.SystemView(revision: Int64(1))));
  await tester.pump();
  return (container: container, daemon: daemon, quits: quits);
}

Future<void> settle(WidgetTester tester, ProviderContainer c) async {
  for (var i = 0; i < 20; i++) {
    await tester.runAsync(() => Future<void>.delayed(const Duration(milliseconds: 5)));
    await tester.pump();
  }
}

final sheet = find.byType(CloseGuardSheet);

/// A button of the sheet (the toolbar has a Save too).
Finder button(String label) => find.descendant(of: sheet, matching: find.text(label));

void main() {
  testWidgets('Close on a dirty project asks; Cancel keeps it', (tester) async {
    final a = await app(tester);
    await tester.tap(find.text('Close'));
    await settle(tester, a.container);
    expect(sheet, findsOneWidget);
    expect(find.text('Save changes to “lamp”?'), findsOneWidget);
    await tester.tap(button('Cancel'));
    await settle(tester, a.container);
    expect(sheet, findsNothing);
    expect(a.container.read(appStoreProvider).project, isNotNull);
    expect(a.daemon.closes, 0);
  });

  testWidgets('Don’t Save closes without saving; Save saves then closes', (tester) async {
    final a = await app(tester);
    await tester.tap(find.text('Close'));
    await settle(tester, a.container);
    await tester.tap(button('Don’t Save'));
    await settle(tester, a.container);
    expect(a.daemon.saves, 0);
    expect(a.daemon.closes, 1);
    expect(a.container.read(appStoreProvider).project, isNull);

    // open again (the fake serves a clean project), dirty it, Save
    a.container.read(appStoreProvider.notifier).dispatch(const OpenProjectRequested('/p'));
    await settle(tester, a.container);
    a.container.read(appStoreProvider.notifier).dispatch(ProjectReceived(lamp(dirty: true)));
    a.daemon.dirty = true;
    await tester.pump();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyW);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await settle(tester, a.container);
    expect(sheet, findsOneWidget, reason: '⌘W is the same guard');
    await tester.tap(button('Save'));
    await settle(tester, a.container);
    expect(a.daemon.saves, 1);
    expect(a.daemon.closes, 2);
    expect(a.container.read(appStoreProvider).project, isNull);
  });

  testWidgets('a save that fails keeps the project open', (tester) async {
    final a = await app(tester);
    a.daemon.saveFails = true;
    await tester.tap(find.text('Close'));
    await settle(tester, a.container);
    await tester.tap(button('Save'));
    await settle(tester, a.container);
    expect(a.daemon.closes, 0);
    expect(a.container.read(appStoreProvider).project, isNotNull);
    expect(find.text('changed on disk'), findsOneWidget);
    expect(sheet, findsNothing);
  });

  testWidgets('a clean project closes at once, without the sheet', (tester) async {
    final a = await app(tester);
    a.daemon.dirty = false;
    await tester.tap(find.text('Close'));
    await settle(tester, a.container);
    expect(sheet, findsNothing);
    expect(a.daemon.closes, 1);
    expect(a.container.read(appStoreProvider).project, isNull);
  });

  testWidgets('the desktop’s exit request is declined until the guard has run', (tester) async {
    final a = await app(tester);
    final first = await tester.runAsync(() => WidgetsBinding.instance.handleRequestAppExit());
    expect(
      first,
      AppExitResponse.cancel,
      reason: 'nothing is torn down while the question is open',
    );
    await settle(tester, a.container);
    expect(sheet, findsOneWidget);
    await tester.tap(button('Cancel'));
    await settle(tester, a.container);
    expect(a.quits, isEmpty);
    expect(a.container.read(appStoreProvider).project, isNotNull);

    // again, and this time Don't Save: the project closes and the app leaves
    await tester.runAsync(() => WidgetsBinding.instance.handleRequestAppExit());
    await settle(tester, a.container);
    await tester.tap(button('Don’t Save'));
    await settle(tester, a.container);
    expect(a.daemon.closes, 1);
    expect(a.quits, ['quit']);

    // with no project open the exit is granted
    final last = await tester.runAsync(() => WidgetsBinding.instance.handleRequestAppExit());
    expect(last, AppExitResponse.exit);
  });

  testWidgets('opening another project asks first and opens it after', (tester) async {
    final a = await app(tester);
    a.container.read(appStoreProvider.notifier).dispatch(const OpenProjectRequested('/q'));
    await settle(tester, a.container);
    expect(sheet, findsOneWidget);
    await tester.tap(button('Don’t Save'));
    await settle(tester, a.container);
    expect(a.daemon.closes, 1);
    expect(
      a.daemon.link.requests.where((m) => m.hasOpenProject()).single.openProject.rootPath,
      '/q',
    );
  });
}
