import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/app/store.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/shell.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

class _FixedStore extends AppStore {
  _FixedStore(this.fixed);
  final AppState fixed;
  @override
  AppState build() => fixed;
}

Widget _app(AppState state) => ProviderScope(
  key: UniqueKey(),
  overrides: [appStoreProvider.overrideWith(() => _FixedStore(state))],
  child: MaterialApp(theme: macTheme(Brightness.light), home: const StudioShell()),
);

void main() {
  testWidgets('status line shows the compiler version, not the protocol, when compatible', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
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
    await tester.pumpWidget(_app(state));
    expect(find.textContaining('Compiler 9.9.9'), findsOneWidget);
    expect(find.textContaining('protocol 0.1.0'), findsNothing);
    // no project: the project manager (welcome) is shown, not the workspace
    expect(find.text('Behavior\nDesigner'), findsOneWidget);
    expect(find.text('Open Project…'), findsOneWidget);
    expect(find.text('Deploy'), findsNothing);
  });

  testWidgets('open project renders library rows and the canvas', (tester) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final project = pb.ProjectProjection(revision: Int64(3), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(0), name: 'Tilt'),
        pb.ConceptView(id: Int64(1), name: 'Brightness'),
      ])
      ..mappings.add(
        mappingView(
          id: Int64(0),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(0)], output: Int64(1)),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED,
        ),
      );
    final state = AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
      ),
      project: project,
      editor: const EditorState(selection: MappingSelected(0)),
    );
    await tester.pumpWidget(_app(state));
    expect(find.text('Tilt'), findsWidgets);
    expect(find.text('dimByTilt'), findsWidgets);
    expect(find.text('1 not yet defined'), findsOneWidget);
    // inspector shows the mapping's editable name and the delete action
    expect(find.text('Delete dimByTilt'), findsOneWidget);
    expect(find.text('Deploy'), findsOneWidget);
  });

  testWidgets('status line counts only declared relationships as not yet defined', (tester) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    // Two Sources (read nothing, no formula: the environment provides them)
    // and one defined rule — nothing is missing, so nothing is *not yet
    // defined*; the Sources are a plain count.
    final project = pb.ProjectProjection(revision: Int64(3), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(0), name: 'Tilt'),
        pb.ConceptView(id: Int64(1), name: 'Brightness'),
      ])
      ..mappings.addAll([
        mappingView(
          id: Int64(0),
          name: 'tilt',
          signature: pb.Signature(output: Int64(0)),
        ),
        mappingView(
          id: Int64(1),
          name: 'level',
          signature: pb.Signature(output: Int64(1)),
        ),
        mappingView(
          id: Int64(2),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(0)], output: Int64(1)),
          definition: pb.Definition(formula: 'Tilt / 90 deg'),
        ),
      ]);
    final state = AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
      ),
      project: project,
    );
    await tester.pumpWidget(_app(state));
    expect(find.text('2 sources'), findsOneWidget);
    expect(find.textContaining('not yet defined'), findsNothing);

    // one declared rule (reads Tilt, no formula yet) is the one open item
    final declared = project.deepCopy()
      ..mappings.add(
        mappingView(
          id: Int64(3),
          name: 'brighten',
          signature: pb.Signature(inputs: [Int64(0)], output: Int64(1)),
        ),
      );
    await tester.pumpWidget(_app(state.copyWith(project: declared)));
    expect(find.text('2 sources'), findsOneWidget);
    expect(find.text('1 not yet defined'), findsOneWidget);
  });

  testWidgets('status line names unsaved drafts and whole-design verdicts', (tester) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final project = pb.ProjectProjection(revision: Int64(3), name: 'lamp', rootPath: '/p')
      ..concepts.add(pb.ConceptView(id: Int64(0), name: 'Tilt'))
      ..mappings.add(
        mappingView(
          id: Int64(0),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(0)], output: Int64(0)),
        ),
      );
    final state = AppState(
      connection: Connected(
        executable: 'bdld',
        handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
      ),
      project: project,
      analysis: pb.ProjectAnalysis(
        revision: Int64(3),
        causal: false,
        clockConsistent: true,
        outputComplete: false,
      ),
      editor: const EditorState(
        drafts: {
          0: DefinitionDraft(mappingId: 0, baseRevision: 3, baseDefinition: null, source: 'Tilt'),
        },
      ),
    );
    await tester.pumpWidget(_app(state));
    expect(find.text('1 definition not added'), findsOneWidget);
    expect(find.text('not causal'), findsOneWidget);
    expect(find.text('outputs incomplete'), findsOneWidget);
    expect(find.text('reads across domains'), findsNothing);

    // a deployment verdict is named with its board and only for this revision
    final deployed = state.copyWith(
      editor: state.editor.copyWith(
        deploy: DeployState(
          targets: [pb.TargetView(id: 'big_board', name: 'Big Board')],
          targetsLoaded: true,
          targetId: 'big_board',
          analysis: pb.DeploymentAnalysis(
            revision: Int64(3),
            target: 'big_board',
            status: pb.DeploymentStatus.DEPLOYMENT_STATUS_INFEASIBLE,
          ),
        ),
      ),
    );
    await tester.pumpWidget(_app(deployed));
    expect(find.text('not feasible on Big Board'), findsOneWidget);
    expect(find.textContaining('invalid'), findsNothing);
  });
}
