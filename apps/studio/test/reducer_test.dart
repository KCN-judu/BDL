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
  test('app start asks for a daemon connection exactly once', () {
    final t1 = reduce(const AppState(), const AppStarted());
    expect(t1.state.connection, isA<Connecting>());
    expect(t1.effects, [isA<ConnectDaemon>()]);
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
    expect(t.effects, [isA<SubscribeProject>()]);
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
