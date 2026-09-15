import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

pb.HandshakeResponse handshake({bool compatible = true}) => pb.HandshakeResponse(
  compatible: compatible,
  compilerVersion: '0.1.0',
  protocolVersion: pb.Version(major: 0, minor: 1, patch: 0),
);

pb.ProjectProjection projection({int revision = 0, String root = '/p'}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: root);

AppState connected({pb.ProjectProjection? project}) => AppState(
  connection: Connected(executable: 'bdld', handshake: handshake()),
  project: project,
);

void main() {
  analysisTests();
  test('app start asks for a daemon connection exactly once', () {
    final t1 = reduce(const AppState(), const AppStarted());
    expect(t1.state.connection, isA<Connecting>());
    expect(t1.effects, [isA<ConnectDaemon>(), isA<LoadRecentProjects>()]);
    final t2 = reduce(t1.state, const ConnectRequested());
    expect(t2.effects, isEmpty);
  });

  test('incompatible daemon is reported, not used', () {
    final t = reduce(
      const AppState(connection: Connecting('bdld')),
      DaemonConnected(executable: 'bdld', handshake: handshake(compatible: false)),
    );
    expect(t.state.connection, isA<ConnectionFailed>());
  });

  test('edits are refused while disconnected and sent against the held revision', () {
    final off = reduce(const AppState(), const CreateConceptRequested(name: 'Tilt'));
    expect(off.effects, isEmpty);

    final on = reduce(
      connected(project: projection(revision: 4)),
      const CreateConceptRequested(name: 'Tilt'),
    );
    expect(on.effects.single, isA<ApplyEdit>());
    expect((on.effects.single as ApplyEdit).baseRevision, 4);
    expect(on.state.editor.pendingRequests, 1);
  });

  test('a new concept carries the kind chosen in the sheet', () {
    final t = reduce(
      connected(project: projection()),
      CreateConceptRequested(
        name: 'Light',
        representation: pb.Representation(quantity: pb.Dim(luminous: 1)),
      ),
    );
    final op = (t.effects.single as ApplyEdit).op.createConcept;
    expect(op.hasRepresentation(), isTrue);
    expect(op.representation.quantity.luminous, 1);
    final open = reduce(connected(project: projection()), const CreateConceptRequested(name: 'X'));
    expect((open.effects.single as ApplyEdit).op.createConcept.hasRepresentation(), isFalse);
  });

  test('stale projections are discarded, newer ones accepted', () {
    final s = connected(project: projection(revision: 5));
    final stale = reduce(s, ProjectReceived(projection(revision: 3)));
    expect(stale.state.project!.revision.toInt(), 5);
    final fresh = reduce(s, ProjectReceived(projection(revision: 6)));
    expect(fresh.state.project!.revision.toInt(), 6);
    expect(fresh.effects, isEmpty);
  });

  test('opening a project subscribes once', () {
    final t = reduce(connected(), ProjectReceived(projection()));
    expect(t.effects, [isA<SubscribeProject>(), isA<RunAnalysis>(), isA<SaveRecentProjects>()]);
    final again = reduce(t.state, ProjectReceived(projection(revision: 1), fromRequest: false));
    expect(again.effects, isEmpty);
    expect(again.state.editor.pendingRequests, 0);
  });

  test('selection is dropped when the entity disappears', () {
    final withConcept = projection(revision: 1)
      ..concepts.add(pb.ConceptView(id: Int64(7), name: 'Tilt'));
    var s = reduce(
      connected(project: withConcept),
      const SelectionChanged(ConceptSelected(7)),
    ).state;
    expect(s.editor.selection, isA<ConceptSelected>());
    s = reduce(s, ProjectReceived(projection(revision: 2))).state;
    expect(s.editor.selection, isA<NoSelection>());
  });

  test('request failures surface as a dismissible error', () {
    final s = reduce(
      connected(project: projection()),
      const RequestFailed(code: 'edit.duplicate_concept_name', message: 'exists'),
    ).state;
    expect(s.editor.lastError?.code, 'edit.duplicate_concept_name');
    expect(reduce(s, const ErrorDismissed()).state.editor.lastError, isNull);
  });

  test('daemon exit drops the project and pending requests', () {
    final s = reduce(connected(project: projection()), const DaemonExited(1)).state;
    expect(s.project, isNull);
    expect(s.connection, isA<ConnectionFailed>());
    expect(s.editor.pendingRequests, 0);
  });
}

pb.ProjectAnalysis analysis(
  int revision, {
  pb.MappingStatus status = pb.MappingStatus.MAPPING_STATUS_TYPE_VALID,
}) =>
    pb.ProjectAnalysis(revision: Int64(revision))
      ..mappings.add(pb.MappingAnalysis(id: Int64(0), status: status));

void analysisTests() {
  test('analysis is kept only for the revision held; stale or ahead ones are dropped', () {
    final s = connected(project: projection(revision: 5));
    final kept = reduce(s, AnalysisReceived(analysis(5))).state;
    expect(kept.analysis?.revision.toInt(), 5);
    expect(kept.mappingAnalysis(0)?.status, pb.MappingStatus.MAPPING_STATUS_TYPE_VALID);
    final stale = reduce(kept, AnalysisReceived(analysis(4))).state;
    expect(stale.analysis, isNull);
    final ahead = reduce(kept, AnalysisReceived(analysis(6))).state;
    expect(ahead.analysis, isNull);
  });

  test('a new projection drops an analysis of an older revision and opening requests one', () {
    final s = connected(project: projection(revision: 5)).copyWith(analysis: analysis(5));
    final t = reduce(s, ProjectReceived(projection(revision: 6), fromRequest: false));
    expect(t.state.analysis, isNull);
    final opened = reduce(connected(), ProjectReceived(projection()));
    expect(opened.effects.whereType<RunAnalysis>().length, 1);
  });
}
