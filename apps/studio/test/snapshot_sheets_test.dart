// Renders the creation sheets over the workspace.  Opt-in via SNAP_DIR.
@Tags(['filesystem'])
library;

import 'dart:async';
import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/dialogs.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/shell.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class _FixedStore extends AppStore {
  _FixedStore(this.fixed);
  final AppState fixed;
  @override
  AppState build() => fixed;

  // The state never moves: a request the shell makes on its own (the
  // tokens of a text on screen) has no executor to go to.
  @override
  void dispatch(AppAction action) {}
}

void main() {
  final dir = Platform.environment['SNAP_DIR'];
  testWidgets('sheets', (tester) async {
    final f = File('/System/Library/Fonts/SFNS.ttf');
    if (f.existsSync()) {
      await (FontLoader(
        '.AppleSystemUIFont',
      )..addFont(Future.value(ByteData.sublistView(f.readAsBytesSync())))).load();
    }
    tester.view.physicalSize = const Size(1200, 800);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final project = pb.ProjectProjection(revision: Int64(1), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(
          id: Int64(0),
          name: 'Tilt',
          representation: pb.Representation(quantity: pb.Dim(angle: 1)),
        ),
        pb.ConceptView(id: Int64(1), name: 'Brightness'),
        pb.ConceptView(
          id: Int64(3),
          name: 'Held',
          representation: pb.Representation(boolean: pb.Unit()),
        ),
      ]);
    final state = AppState(
      connection: Connected(executable: 'bdld', handshake: pb.HandshakeResponse(compatible: true)),
      project: project,
    );
    late BuildContext ctx;
    await tester.pumpWidget(
      ProviderScope(
        overrides: [appStoreProvider.overrideWith(() => _FixedStore(state))],
        child: MaterialApp(
          theme: macTheme(Brightness.dark),
          home: Builder(
            builder: (c) {
              ctx = c;
              return const StudioShell();
            },
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    unawaited(showNewConceptSheet(ctx));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).first, 'Temperature');
    await tester.tap(find.text('Quantity'));
    await tester.pumpAndSettle();
    if (dir != null) {
      await expectLater(find.byType(MaterialApp), matchesGoldenFile('$dir/sheet_concept.png'));
    }
    Navigator.of(ctx).pop();
    await tester.pumpAndSettle();

    unawaited(showNewMappingSheet(ctx, project.concepts));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).first, 'dimByTilt');
    // the 12 pt texts are the concept toggles (the pop-up shows 13 pt)
    Finder toggle(String name) => find.descendant(
      of: find.byType(Dialog),
      matching: find.byWidgetPredicate(
        (w) => w is Text && w.data == name && w.style?.fontSize == 12,
      ),
    );
    await tester.tap(toggle('Tilt'));
    await tester.pumpAndSettle();
    await tester.tap(toggle('Held'));
    await tester.pumpAndSettle();
    if (dir != null) {
      await expectLater(find.byType(MaterialApp), matchesGoldenFile('$dir/sheet_mapping.png'));
    }
  }, skip: dir == null);
}
