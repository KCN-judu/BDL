/// The definition editor in the inspector, driven through the real reducer:
/// unresolved and defined mappings, typing, verdicts and spans, save,
/// replace, detach, conflicts, selection switches, pushes, keyboard.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:bdl_studio/ui/definition_editor.dart';
import 'package:bdl_studio/ui/inspector.dart';
import 'package:bdl_studio/ui/mac/theme.dart';
import 'package:bdl_studio/ui/mac/tokens.dart';
import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

const tilt = 0;
const brightness = 1;
const dim = 0;
const other = 1;

pb.ProjectProjection lamp({int revision = 1, String? definition, String? otherDefinition}) {
  final p = pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
    ..concepts.addAll([
      pb.ConceptView(id: Int64(tilt), name: 'Tilt'),
      pb.ConceptView(id: Int64(brightness), name: 'Brightness'),
    ]);
  p.mappings.add(
    pb.MappingView(
      id: Int64(dim),
      name: 'dimByTilt',
      signature: pb.Signature(inputs: [Int64(tilt)], output: Int64(brightness)),
      definition: definition == null ? null : pb.Definition(formula: definition),
    ),
  );
  p.mappings.add(
    pb.MappingView(
      id: Int64(other),
      name: 'other',
      signature: pb.Signature(inputs: [], output: Int64(brightness)),
      definition: otherDefinition == null ? null : pb.Definition(formula: otherDefinition),
    ),
  );
  return p;
}

AppState connected(pb.ProjectProjection project) => AppState(
  connection: Connected(
    executable: 'bdld',
    handshake: pb.HandshakeResponse(compatible: true, protocolVersion: pb.Version()),
  ),
  project: project,
  editor: const EditorState(selection: MappingSelected(dim)),
);

pb.Diagnostic diag(
  String code,
  String message, {
  int? start,
  int? end,
  pb.DiagnosticSeverity severity = pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR,
  String explanation = '',
}) => pb.Diagnostic(
  code: code,
  severity: severity,
  mappingId: Int64(dim),
  message: message,
  explanation: explanation,
  span: start == null ? null : pb.SourceSpan(start: start, end: end),
);

pb.DefinitionDraftAnalysis verdict({
  required int revision,
  required int generation,
  pb.MappingStatus status = pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
  bool parseOk = true,
  List<pb.Diagnostic> diagnostics = const [],
}) => pb.DefinitionDraftAnalysis(
  revision: Int64(revision),
  mappingId: Int64(dim),
  generation: Int64(generation),
  parseOk: parseOk,
  analysis: pb.MappingAnalysis(id: Int64(dim), status: status, diagnostics: diagnostics),
);

/// The inspector over a live state: every dispatch goes through `reduce`,
/// effects are recorded instead of executed.
class Harness extends StatefulWidget {
  const Harness({super.key, required this.initial});
  final AppState initial;
  @override
  State<Harness> createState() => HarnessState();
}

class HarnessState extends State<Harness> {
  late AppState state = widget.initial;
  final List<Effect> effects = [];

  void dispatch(AppAction a) {
    setState(() {
      final t = reduce(state, a);
      state = t.state;
      effects.addAll(t.effects);
    });
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    theme: macTheme(Brightness.light),
    home: Scaffold(
      body: Align(
        alignment: Alignment.topLeft,
        child: SizedBox(
          width: MacMetrics.inspectorWidth,
          height: 800,
          child: Inspector(state: state, dispatch: dispatch),
        ),
      ),
    ),
  );
}

Future<HarnessState> pump(WidgetTester tester, AppState initial) async {
  final key = GlobalKey<HarnessState>();
  await tester.pumpWidget(Harness(key: key, initial: initial));
  return key.currentState!;
}

Finder get field => find.descendant(
  of: find.byKey(const ValueKey('definition-field')),
  matching: find.byType(TextField),
);
String statusText(WidgetTester t) =>
    t.widget<Text>(find.byKey(const ValueKey('definition-status'))).data!;

/// The inspector scrolls; bring the control into view before tapping it.
Future<void> tapText(WidgetTester t, String label) async {
  await t.ensureVisible(find.text(label));
  await t.tap(find.text(label));
  await t.pump();
}

void main() {
  testWidgets('an unresolved mapping shows an empty editor with the names in scope', (t) async {
    final h = await pump(t, connected(lamp()));
    expect(find.text('expression over Tilt'), findsOneWidget);
    expect(statusText(t), startsWith('No definition yet.'));
    expect(find.text('Add definition'), findsOneWidget);
    expect(find.text('Detach definition'), findsNothing);
    expect(find.text('Revert'), findsNothing);
    expect(h.state.editor.drafts, isEmpty);
  });

  testWidgets('typing makes a draft, asks the compiler and shows checking', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    expect(h.state.draft(dim)!.source, 'Tilt / 90 deg');
    expect(h.effects.whereType<AnalyzeDraft>().single.source, 'Tilt / 90 deg');
    expect(statusText(t), 'Checking…');
    expect(find.text('unsaved'), findsOneWidget);
    expect(find.text('Revert'), findsOneWidget);
  });

  testWidgets('a compiler diagnostic is shown under the text with its excerpt', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt + 1 s');
    await t.pump();
    h.dispatch(
      DraftAnalysisReceived(
        verdict(
          revision: 1,
          generation: 1,
          status: pb.MappingStatus.MAPPING_STATUS_INVALID,
          diagnostics: [
            diag(
              'dimension.mismatch',
              'These cannot be added: an angle and a time.',
              start: 0,
              end: 10,
              explanation: 'Both sides of + must have the same dimension.',
            ),
          ],
        ),
      ),
    );
    await t.pump();
    expect(statusText(t), 'These cannot be added: an angle and a time.');
    expect(find.text('Tilt + 1 s'), findsWidgets, reason: 'the excerpt echoes the span');
    expect(find.text('Both sides of + must have the same dimension.'), findsOneWidget);
    expect(find.text('Add definition'), findsOneWidget);
    // correcting it: the old verdict goes, the new one comes
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    expect(statusText(t), 'Checking…');
    expect(find.text('Both sides of + must have the same dimension.'), findsNothing);
    h.dispatch(DraftAnalysisReceived(verdict(revision: 1, generation: 2)));
    await t.pump();
    expect(statusText(t), 'Valid definition');
  });

  testWidgets('open is shown as open, not as an error', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt / 2');
    await t.pump();
    h.dispatch(
      DraftAnalysisReceived(
        verdict(
          revision: 1,
          generation: 1,
          status: pb.MappingStatus.MAPPING_STATUS_OPEN,
          diagnostics: [
            diag(
              'semantic.unbound_representation',
              'Tilt has no representation yet.',
              severity: pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_INFO,
            ),
          ],
        ),
      ),
    );
    await t.pump();
    expect(statusText(t), 'Tilt has no representation yet.');
    final tokens = MacTokens.of(t.element(find.byKey(const ValueKey('definition-status'))));
    final style = t.widget<Text>(find.byKey(const ValueKey('definition-status'))).style!;
    expect(style.color, tokens.open);
    expect(style.color, isNot(tokens.error));
  });

  testWidgets('add definition sends one attach and the field keeps the text until confirmed', (
    t,
  ) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    h.dispatch(DraftAnalysisReceived(verdict(revision: 1, generation: 1)));
    await t.pump();
    await tapText(t, 'Add definition');
    final edit = h.effects.whereType<ApplyEdit>().single;
    expect(edit.op.hasAttachDefinition(), isTrue);
    expect(statusText(t), 'Saving…');
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    // a second tap while saving does nothing
    await tapText(t, 'Add definition');
    expect(h.effects.whereType<ApplyEdit>(), hasLength(1));
    // the projection confirms: committed, no draft, no Revert
    h.dispatch(ProjectReceived(lamp(revision: 2, definition: 'Tilt / 90 deg')));
    await t.pump();
    expect(h.state.draft(dim), isNull);
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    expect(find.text('Revert'), findsNothing);
    expect(find.text('Detach definition'), findsOneWidget);
    expect(find.text('unsaved'), findsNothing);
  });

  testWidgets('a failed save keeps the text and says so next to it', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    await tapText(t, 'Add definition');
    h.dispatch(const RequestFailed(code: 'edit.stale_revision', message: 'the project moved on'));
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    expect(statusText(t), 'Not saved: the project moved on');
    expect(find.text('Add definition'), findsOneWidget);
  });

  testWidgets('editing an existing definition starts from it and can be saved or reverted', (
    t,
  ) async {
    final h = await pump(t, connected(lamp(definition: 'Tilt / 90 deg')));
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    expect(find.text('Save definition'), findsNothing, reason: 'nothing to save yet');
    await t.enterText(field, 'Tilt / 45 deg');
    await t.pump();
    expect(find.text('Save definition'), findsOneWidget);
    expect(h.state.committedDefinition(dim), 'Tilt / 90 deg', reason: 'not changed by typing');
    await tapText(t, 'Revert');
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    expect(h.state.draft(dim), isNull);

    await t.enterText(field, 'Tilt / 45 deg');
    await t.pump();
    await tapText(t, 'Save definition');
    expect(h.effects.whereType<ApplyEdit>().single.op.hasReplaceDefinition(), isTrue);
  });

  testWidgets('detach sends one replace-with-nothing and the editor returns to unresolved', (
    t,
  ) async {
    final h = await pump(t, connected(lamp(definition: 'Tilt / 90 deg')));
    await tapText(t, 'Detach definition');
    final op = h.effects.whereType<ApplyEdit>().single.op;
    expect(op.hasReplaceDefinition() && !op.replaceDefinition.hasDefinition(), isTrue);
    h.dispatch(ProjectReceived(lamp(revision: 2)));
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, '');
    expect(statusText(t), startsWith('No definition yet.'));
    expect(find.text('Detach definition'), findsNothing);
  });

  testWidgets('switching selection and back restores the draft', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    h.dispatch(const SelectionChanged(MappingSelected(other)));
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, '');
    expect(find.text('expression with no inputs'), findsOneWidget);
    h.dispatch(const SelectionChanged(MappingSelected(dim)));
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
  });

  testWidgets('the source survives analysis and projection pushes', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    h.dispatch(AnalysisReceived(pb.ProjectAnalysis(revision: Int64(1))));
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    // an unrelated change elsewhere: new revision, same mapping
    h.dispatch(ProjectReceived(lamp(revision: 2, otherDefinition: '1'), fromRequest: false));
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 90 deg');
    expect(statusText(t), 'Checking…');
    expect(h.effects.whereType<AnalyzeDraft>().last.revision, 2);
  });

  testWidgets('a conflict is a notice with two ways out, never a silent overwrite', (t) async {
    final h = await pump(t, connected(lamp(definition: 'Tilt / 90 deg')));
    await t.enterText(field, 'Tilt / 45 deg');
    await t.pump();
    h.dispatch(
      ProjectReceived(lamp(revision: 2, definition: 'Tilt / 180 deg'), fromRequest: false),
    );
    await t.pump();
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 45 deg');
    expect(statusText(t), contains('changed while you were editing'));
    expect(find.text('Save definition'), findsNothing);
    await tapText(t, 'Keep mine');
    expect(find.text('Save definition'), findsOneWidget);
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 45 deg');

    // and Reload takes theirs
    h.dispatch(ProjectReceived(lamp(revision: 3, definition: 'Tilt / 10 deg'), fromRequest: false));
    await t.pump();
    await tapText(t, 'Reload');
    expect(t.widget<TextField>(field).controller!.text, 'Tilt / 10 deg');
    expect(h.state.draft(dim), isNull);
  });

  testWidgets('⌘↩ saves a dirty definition, Esc reverts it, ⌘S is left to the project', (t) async {
    final h = await pump(t, connected(lamp()));
    await t.tap(field);
    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.escape);
    await t.pump();
    expect(h.state.draft(dim), isNull);
    expect(t.widget<TextField>(field).controller!.text, '');

    await t.enterText(field, 'Tilt / 90 deg');
    await t.pump();
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.keyS);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await t.pump();
    expect(h.effects.whereType<ApplyEdit>(), isEmpty, reason: '⌘S never commits a draft');
    expect(h.state.draft(dim)!.source, 'Tilt / 90 deg');

    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.enter);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    await t.pump();
    expect(h.effects.whereType<ApplyEdit>().single.op.hasAttachDefinition(), isTrue);
  });

  test('the model words every state in product language', () {
    const inputs = ['Tilt'];
    DefinitionEditorModel model({
      String? committed,
      DefinitionDraft? draft,
      pb.MappingAnalysis? a,
    }) => DefinitionEditorModel(
      committed: committed,
      draft: draft,
      committedAnalysis: a,
      inputNames: inputs,
    );
    expect(model().commitLabel, 'Add definition');
    expect(model(committed: 'x').commitLabel, 'Save definition');
    expect(model(committed: 'x').statusText, 'Checking…');
    expect(
      model(
        committed: 'x',
        a: pb.MappingAnalysis(status: pb.MappingStatus.MAPPING_STATUS_TYPE_VALID),
      ).statusText,
      'Valid definition',
    );
    const d = DefinitionDraft(mappingId: dim, baseRevision: 1, baseDefinition: null, source: ' ');
    expect(model(draft: d).statusText, 'Nothing to add yet.');
    expect(model(draft: d).canCommit, isFalse);
    final unavailable = d.copyWith(
      source: 'Tilt',
      check: DraftCheck.unavailable,
      checkError: 'gone',
    );
    expect(model(draft: unavailable).statusText, 'Not checked: gone');
    expect(model(draft: unavailable).canCommit, isTrue, reason: 'the model may still commit it');
    final syntax = d.copyWith(
      source: 'Tilt /',
      check: DraftCheck.checked,
      parseOk: false,
      analysis: pb.MappingAnalysis(
        status: pb.MappingStatus.MAPPING_STATUS_INVALID,
        diagnostics: [
          diag(
            'formula.parse.unexpected_token',
            'Expected an expression after `/`.',
            start: 6,
            end: 6,
          ),
        ],
      ),
    );
    expect(model(draft: syntax).statusText, 'Expected an expression after `/`.');
    expect(model(draft: syntax).tone, VerdictTone.error);
  });
}
