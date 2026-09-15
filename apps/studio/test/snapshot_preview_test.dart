// Renders the shell with real fonts to a PNG so the look can be inspected
// without screen-recording permission.  Not a golden: opt-in via SNAP_DIR.
import 'dart:io';

import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
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
}

void main() {
  final dir = Platform.environment['SNAP_DIR'];
  testWidgets('render preview', (tester) async {
    final font = File('/System/Library/Fonts/SFNS.ttf');
    if (font.existsSync()) {
      final loader = FontLoader('.AppleSystemUIFont')
        ..addFont(Future.value(ByteData.sublistView(font.readAsBytesSync())));
      await loader.load();
    }
    tester.view.physicalSize = const Size(1440, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final project = pb.ProjectProjection(revision: Int64(7), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(
          id: Int64(0),
          name: 'Tilt',
          representation: pb.Representation(quantity: pb.Dim(angle: 1)),
        ),
        pb.ConceptView(
          id: Int64(1),
          name: 'Brightness',
          representation: pb.Representation(quantity: pb.Dim()),
        ),
        pb.ConceptView(id: Int64(2), name: 'Temperature'),
        pb.ConceptView(
          id: Int64(3),
          name: 'Held',
          representation: pb.Representation(boolean: pb.Unit()),
        ),
      ])
      ..mappings.addAll([
        pb.MappingView(
          id: Int64(0),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(0), Int64(3)], output: Int64(1)),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED,
        ),
        pb.MappingView(
          id: Int64(1),
          name: 'warmPulse',
          signature: pb.Signature(inputs: [Int64(2)], output: Int64(1)),
          definition: pb.Definition(formula: 'ite(Temperature > 40K, 0.3, 1)'),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED,
        ),
      ])
      ..layout = pb.Layout(
        concepts: [
          pb.NodePosition(id: Int64(0), x: 60, y: 80),
          pb.NodePosition(id: Int64(3), x: 60, y: 200),
          pb.NodePosition(id: Int64(2), x: 60, y: 340),
          pb.NodePosition(id: Int64(1), x: 720, y: 160),
        ],
        mappings: [
          pb.NodePosition(id: Int64(0), x: 380, y: 100),
          pb.NodePosition(id: Int64(1), x: 380, y: 300),
        ],
      );
    for (final brightness in [Brightness.light, Brightness.dark]) {
      final state = AppState(
        connection: Connected(
          executable: 'bdld',
          handshake: pb.HandshakeResponse(
            compatible: true,
            compilerVersion: '0.1.0',
            protocolVersion: pb.Version(major: 0, minor: 1, patch: 0),
          ),
        ),
        project: project,
        editor: EditorState(
          selection: const MappingSelected(0),
          layout: {
            const NodeRef.concept(0): const Offset(60, 80),
            const NodeRef.concept(3): const Offset(60, 200),
            const NodeRef.concept(2): const Offset(60, 340),
            const NodeRef.concept(1): const Offset(720, 160),
            const NodeRef.mapping(0): const Offset(380, 100),
            const NodeRef.mapping(1): const Offset(380, 300),
          },
          lastOutcome: pb.EditOutcome(kind: pb.EditKind.EDIT_KIND_REFINEMENT),
        ),
      );
      await tester.pumpWidget(
        ProviderScope(
          overrides: [appStoreProvider.overrideWith(() => _FixedStore(state))],
          child: MaterialApp(theme: macTheme(brightness), home: const StudioShell()),
        ),
      );
      await tester.pumpAndSettle();
      if (dir != null) {
        await expectLater(
          find.byType(StudioShell),
          matchesGoldenFile('$dir/shell_${brightness.name}.png'),
        );
      }
    }
  }, skip: dir == null);
}
