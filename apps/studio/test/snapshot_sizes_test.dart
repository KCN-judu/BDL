// Renders the welcome screen at several window sizes.  Opt-in via SNAP_DIR.
import 'dart:io';

import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/shell.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class _FixedStore extends AppStore {
  _FixedStore(this.fixed);
  final AppState fixed;
  @override
  AppState build() => fixed;
}

void main() {
  final dir = Platform.environment['SNAP_DIR'];
  testWidgets('welcome at several sizes', (tester) async {
    for (final f in ['/System/Library/Fonts/SFNS.ttf']) {
      if (File(f).existsSync()) {
        await (FontLoader(
          '.AppleSystemUIFont',
        )..addFont(Future.value(ByteData.sublistView(File(f).readAsBytesSync())))).load();
      }
    }
    final chakra = FontLoader('ChakraPetch');
    for (final w in ['Regular', 'Medium', 'SemiBold', 'Bold']) {
      chakra.addFont(
        Future.value(
          ByteData.sublistView(File('assets/fonts/ChakraPetch-$w.ttf').readAsBytesSync()),
        ),
      );
    }
    await chakra.load();
    final state = AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(compatible: true, compilerVersion: '0.1.0'),
      ),
    );
    for (final size in const [Size(800, 640), Size(1100, 700), Size(1800, 1000), Size(560, 760)]) {
      tester.view.physicalSize = size;
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.reset);
      await tester.pumpWidget(
        ProviderScope(
          overrides: [appStoreProvider.overrideWith(() => _FixedStore(state))],
          child: MaterialApp(theme: macTheme(Brightness.dark), home: const StudioShell()),
        ),
      );
      await tester.pumpAndSettle();
      if (dir != null) {
        await expectLater(
          find.byType(StudioShell),
          matchesGoldenFile('$dir/welcome_${size.width.toInt()}x${size.height.toInt()}.png'),
        );
      }
    }
  }, skip: dir == null);
}
