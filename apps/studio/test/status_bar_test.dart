import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/status_bar.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class _FixedStore extends AppStore {
  _FixedStore(this.fixed);
  final AppState fixed;
  @override
  AppState build() => fixed;
}

void main() {
  testWidgets('status bar shows compiler and protocol versions when connected', (tester) async {
    final state = AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(
          compatible: true,
          compilerVersion: '9.9.9',
          protocolVersion: pb.Version(major: 0, minor: 1, patch: 0),
        ),
      ),
    );
    await tester.pumpWidget(
      ProviderScope(
        overrides: [appStoreProvider.overrideWith(() => _FixedStore(state))],
        child: const MaterialApp(home: Scaffold(body: StatusBar())),
      ),
    );
    expect(find.textContaining('compiler 9.9.9'), findsOneWidget);
    expect(find.textContaining('protocol 0.1.0'), findsOneWidget);
    expect(find.textContaining('connected'), findsOneWidget);
  });

  testWidgets('status bar offers reconnect when disconnected', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [appStoreProvider.overrideWith(() => _FixedStore(const AppState()))],
        child: const MaterialApp(home: Scaffold(body: StatusBar())),
      ),
    );
    expect(find.text('Reconnect'), findsOneWidget);
  });
}
