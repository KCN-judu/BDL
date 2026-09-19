// Renders the shell with real fonts to a PNG so the look can be inspected
// without screen-recording permission.  Not a golden: opt-in via SNAP_DIR.
@Tags(['filesystem'])
library;

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
    final chakra = FontLoader('ChakraPetch');
    for (final w in ['Regular', 'Medium', 'SemiBold', 'Bold']) {
      chakra.addFont(
        Future.value(
          ByteData.sublistView(File('assets/fonts/ChakraPetch-$w.ttf').readAsBytesSync()),
        ),
      );
    }
    await chakra.load();
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
          definition: pb.Definition(formula: 'Tilt + 1 s'),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED,
        ),
        pb.MappingView(
          id: Int64(1),
          name: 'warmPulse',
          signature: pb.Signature(inputs: [Int64(2)], output: Int64(1)),
          definition: pb.Definition(formula: 'ite(Temperature > 40K, 0.3, 1)'),
          state: pb.AcceptanceState.ACCEPTANCE_STATE_DEFINED,
          clockId: Int64(0),
          drivesOutputId: Int64(0),
        ),
      ])
      ..clocks.add(pb.ClockView(id: Int64(0), name: 'interaction'))
      ..outputs.add(
        pb.OutputView(
          id: Int64(0),
          name: 'Light Output',
          accepts: Int64(1),
          clockId: Int64(0),
          required: true,
        ),
      )
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
        outputs: [pb.NodePosition(id: Int64(0), x: 720, y: 320)],
      );
    for (final brightness in [Brightness.light, Brightness.dark]) {
      final welcome = AppState(
        connection: Connected(
          executable: 'bdld',
          handshake: pb.HandshakeResponse(
            compatible: true,
            compilerVersion: '0.1.0',
            protocolVersion: pb.Version(major: 0, minor: 1, patch: 0),
          ),
        ),
        recent: [
          RecentProject(
            path: '/Users/kcn/Projects/lamp',
            name: 'lamp',
            lastOpened: DateTime.now().subtract(const Duration(hours: 2)),
          ),
          RecentProject(
            path: '/Users/kcn/Projects/rover',
            name: 'rover',
            lastOpened: DateTime.now().subtract(const Duration(days: 1)),
          ),
          RecentProject(path: '/Volumes/old/cup', name: 'cup', lastOpened: DateTime(2026, 7, 2)),
        ],
      );
      await tester.pumpWidget(
        ProviderScope(
          key: UniqueKey(),
          overrides: [appStoreProvider.overrideWith(() => _FixedStore(welcome))],
          child: MaterialApp(theme: macTheme(brightness), home: const StudioShell()),
        ),
      );
      await tester.pumpAndSettle();
      if (dir != null) {
        await expectLater(
          find.byType(StudioShell),
          matchesGoldenFile('$dir/welcome_${brightness.name}.png'),
        );
      }

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
        analysis: pb.ProjectAnalysis(revision: Int64(7))
          ..mappings.addAll([
            pb.MappingAnalysis(
              id: Int64(0),
              status: pb.MappingStatus.MAPPING_STATUS_INVALID,
              interface: 'sem#0 → sem#3 → sem#1',
              diagnostics: [
                pb.Diagnostic(
                  code: 'dimension.mismatch',
                  severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
                  mappingId: Int64(0),
                  span: pb.SourceSpan(start: 0, end: 10),
                  message: 'This expression adds values with different physical dimensions: an angle and a time.',
                  explanation: 'Only quantities of the same dimension can be added, subtracted or compared. Multiplying or dividing combines dimensions.',
                  technical: '+ : q[rad] → q[rad] → …, found q[s]',
                ),
              ],
            ),
            pb.MappingAnalysis(
              id: Int64(1),
              status: pb.MappingStatus.MAPPING_STATUS_TYPE_VALID,
              interface: 'sem#2 → sem#1',
              inferredType: 'sem#2 → sem#1',
              coreExpr: 'λ(sem#2). (mk sem#1 (ite[q[1]] (lt[K] 313.15[K] (rep #0)) 0.3[1] 1[1]))',
            ),
          ]),
        editor: EditorState(
          selection: const MappingSelected(0),
          layout: {
            const NodeRef.concept(0): const Offset(60, 80),
            const NodeRef.concept(3): const Offset(60, 200),
            const NodeRef.concept(2): const Offset(60, 340),
            const NodeRef.concept(1): const Offset(720, 160),
            const NodeRef.mapping(0): const Offset(380, 100),
            const NodeRef.mapping(1): const Offset(380, 300),
            const NodeRef.output(0): const Offset(720, 320),
          },
          lastOutcome: pb.EditOutcome(kind: pb.EditKind.EDIT_KIND_REFINEMENT),
        ),
      );
      await tester.pumpWidget(
        ProviderScope(
          key: UniqueKey(),
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

      // The Library tab with a search, and a concept just inserted from it:
      // selected, its name open for editing on the canvas.
      pb.LibraryItemView conceptItem(pb.ConceptTemplateView t) => pb.LibraryItemView(
        id: t.id,
        category: 'concept',
        displayName: t.displayName,
        description: t.description,
        group: t.category,
        creates: [
          pb.LibraryObjectView(
            kind: 'concept',
            key: 'concept',
            name: t.defaultName,
            typeName: t.typeName,
            representation: t.hasRepresentation() ? t.representation : null,
            unit: t.unit,
          ),
        ],
        concept: t,
      );
      final library = state.copyWith(
        library: pb.LibraryItemsResponse(
          libraries: [
            pb.LibraryView(
              id: 'std',
              name: 'BDL Standard Library',
              schemaVersion: 2,
              version: '0.2',
              items: [
                for (final (cat, name, rep, unit, role) in [
                  ('environment', 'Temperature', pb.Dim(temperature: 1), 'K', 1),
                  (
                    'environment',
                    'Ambient Light',
                    pb.Dim(luminous: 1, angle: 2, length: -2),
                    'lx',
                    1,
                  ),
                  ('environment', 'Humidity', pb.Dim(), '', 1),
                  ('motion', 'Tilt', pb.Dim(angle: 1), 'deg', 1),
                  ('motion', 'Distance', pb.Dim(length: 1), 'm', 1),
                  ('actuation', 'Motor Speed', pb.Dim(), '', 2),
                  ('actuation', 'Motor Angle', pb.Dim(angle: 1), 'deg', 2),
                  ('actuation', 'Heater Power', pb.Dim(), '', 2),
                ])
                  conceptItem(
                    pb.ConceptTemplateView(
                      id: 'std.$cat.${name.toLowerCase().replaceAll(' ', '_')}',
                      displayName: name,
                      defaultName: name.replaceAll(' ', ''),
                      description: 'How $name is measured.',
                      category: cat,
                      roleHint: pb.RoleHint.valueOf(role)!,
                      representation: pb.Representation(quantity: rep),
                      unit: unit,
                    ),
                  ),
                conceptItem(
                  pb.ConceptTemplateView(
                    id: 'std.human.button_pressed',
                    displayName: 'Button Pressed',
                    defaultName: 'ButtonPressed',
                    description: 'Whether a button is held down.',
                    category: 'human',
                    roleHint: pb.RoleHint.ROLE_HINT_INPUT,
                    representation: pb.Representation(boolean: pb.Unit()),
                  ),
                ),
                pb.LibraryItemView(
                  id: 'std.source.temperature',
                  category: 'source',
                  displayName: 'Temperature Sensor',
                  description: 'A temperature the product measures.',
                  group: 'environment',
                  creates: [
                    pb.LibraryObjectView(
                      kind: 'concept',
                      key: 'value',
                      name: 'RoomTemp',
                      typeName: 'Temperature',
                      representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
                      unit: 'K',
                    ),
                    pb.LibraryObjectView(
                      kind: 'mapping',
                      key: 'source',
                      name: 'TempSensor',
                      signature: '() -> RoomTemp',
                    ),
                  ],
                ),
              ],
            ),
          ],
        ),
        editor: state.editor.copyWith(
          sidebar: SidebarTab.library,
          librarySearch: 'mo',
          selection: const ConceptSelected(2),
          renaming: const NodeRef.concept(2),
        ),
      );
      await tester.pumpWidget(
        ProviderScope(
          key: UniqueKey(),
          overrides: [appStoreProvider.overrideWith(() => _FixedStore(library))],
          child: MaterialApp(theme: macTheme(brightness), home: const StudioShell()),
        ),
      );
      await tester.pumpAndSettle();
      if (dir != null) {
        await expectLater(
          find.byType(StudioShell),
          matchesGoldenFile('$dir/library_${brightness.name}.png'),
        );
      }

      // The definition editor with a dirty, checked draft over the invalid
      // committed definition: verdict line, unsaved marker, Revert / Save.
      final drafting = state.copyWith(
        editor: state.editor.copyWith(
          drafts: {
            0: DefinitionDraft(
              mappingId: 0,
              baseRevision: 7,
              baseDefinition: 'Tilt + 1 s',
              source: 'Tilt / 90 deg',
              generation: 3,
              check: DraftCheck.checked,
              analysis: pb.MappingAnalysis(
                id: Int64(0),
                status: pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
              ),
            ),
          },
        ),
      );
      await tester.pumpWidget(
        ProviderScope(
          key: UniqueKey(),
          overrides: [appStoreProvider.overrideWith(() => _FixedStore(drafting))],
          child: MaterialApp(theme: macTheme(brightness), home: const StudioShell()),
        ),
      );
      await tester.pumpAndSettle();
      if (dir != null) {
        await expectLater(
          find.byType(StudioShell),
          matchesGoldenFile('$dir/shell_draft_${brightness.name}.png'),
        );
      }
    }
  }, skip: dir == null);
}
