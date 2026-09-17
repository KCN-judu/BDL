/// The Code view's reducers (ADR-0023 §3–§5): the sources follow the
/// revision, typing is the editor's until sent, an accepted edit is a new
/// revision, a refused one keeps the draft and the last good project, a
/// stale one is resent, and typing during a send is the next send.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

pb.ProjectProjection project({int revision = 3}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/tmp/lamp')
      ..kind = pb.ProjectKind.PROJECT_KIND_TEXT
      ..concepts.add(pb.ConceptView(id: Int64(0), name: 'Tilt'));

pb.SourcesView sources({
  int revision = 3,
  String text = 'concept Tilt : Angle\n',
  bool draft = false,
  List<pb.SourceDiagnostic> diagnostics = const [],
}) => pb.SourcesView(
  revision: Int64(revision),
  files: [
    pb.SourceFileView(
      path: 'src/main.bdl',
      text: text,
      draft: draft,
      anchors: [pb.SourceAnchor(start: 0, end: 20, conceptId: Int64(0))],
    ),
  ],
  diagnostics: diagnostics,
);

AppState opened() {
  final connected = AppState(
    connection: Connected(executable: 'x', handshake: pb.HandshakeResponse(compatible: true)),
  );
  var s = reduce(connected, ProjectReceived(project())).state;
  s = reduce(s, SystemReceived(pb.SystemView(revision: Int64(3)))).state;
  return s;
}

void main() {
  test('showing Code fetches the sources once per revision; Design does not', () {
    var t = reduce(opened(), const DesignViewChanged(DesignView.code));
    expect(t.state.editor.view, DesignView.code);
    expect(t.effects.whereType<GetSources>(), hasLength(1));
    var s = reduce(t.state, SourcesReceived(sources())).state;
    expect(s.editor.sources.revision, 3);
    expect(s.editor.sources.openPath, 'src/main.bdl');
    expect(s.editor.sources.text, 'concept Tilt : Angle\n');
    // the same revision again: nothing to fetch
    t = reduce(s, const DesignViewChanged(DesignView.split));
    expect(t.effects, isEmpty);
    // a graph edit moves the revision: the text follows
    t = reduce(t.state, ProjectReceived(project(revision: 4)));
    expect(t.effects.whereType<GetSources>(), hasLength(1));
    // in Design, the text is not fetched
    s = reduce(opened(), const DesignViewChanged(DesignView.design)).state;
    t = reduce(s, ProjectReceived(project(revision: 4)));
    expect(t.effects.whereType<GetSources>(), isEmpty);
  });

  test('typing is the editor\'s until sent; sending is one counted request', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.code)).state;
    s = reduce(s, SourcesReceived(sources())).state;
    s = reduce(s, const SourceTextChanged('src/main.bdl', 'concept Tilt : Angle\nconcept X')).state;
    expect(s.editor.sources.buffer, 'concept Tilt : Angle\nconcept X');
    expect(s.editor.sources.text, 'concept Tilt : Angle\nconcept X');
    // typing back to the daemon's text is not an edit
    s = reduce(s, const SourceTextChanged('src/main.bdl', 'concept Tilt : Angle\n')).state;
    expect(s.editor.sources.buffer, isNull);
    s = reduce(s, const SourceTextChanged('src/main.bdl', 'concept Tilt : Angle\nconcept X')).state;
    final t = reduce(
      s,
      const SourceEditRequested('src/main.bdl', 'concept Tilt : Angle\nconcept X'),
    );
    final send = t.effects.whereType<ApplySourceEdit>().single;
    expect(send.baseRevision, 3);
    expect(send.path, 'src/main.bdl');
    expect(send.text, 'concept Tilt : Angle\nconcept X');
    expect(t.state.editor.pendingRequests, 1);
    expect(t.state.editor.sources.sent, send.text);
    // nothing to send when the text is the daemon's
    final none = reduce(
      reduce(s, SourcesReceived(sources())).state,
      const SourceEditRequested('src/main.bdl', 'concept Tilt : Angle\n'),
    );
    expect(none.effects, isEmpty);
  });

  test('an accepted edit is a new revision with its text; the buffer settles', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.code)).state;
    s = reduce(s, SourcesReceived(sources())).state;
    const typed = 'concept Tilt : Angle\nconcept Brightness : Scalar\n';
    s = reduce(s, const SourceTextChanged('src/main.bdl', typed)).state;
    s = reduce(s, const SourceEditRequested('src/main.bdl', typed)).state;
    final t = reduce(
      s,
      SourceEditApplied(
        pb.SourceEditApplied(
          accepted: true,
          project: project(revision: 4)
            ..concepts.add(pb.ConceptView(id: Int64(1), name: 'Brightness')),
          sources: sources(revision: 4, text: typed),
        ),
      ),
    );
    expect(t.state.revision, 4);
    expect(t.state.flat!.concepts.map((c) => c.name), contains('Brightness'));
    expect(t.effects.whereType<GetSystem>(), hasLength(1), reason: 'the system follows');
    expect(t.state.editor.sources.revision, 4);
    expect(t.state.editor.sources.buffer, isNull);
    expect(t.state.editor.sources.text, typed);
    expect(t.state.editor.sources.outOfSync, isFalse);
    expect(t.state.editor.pendingRequests, greaterThanOrEqualTo(0));
    // the projection's own text came with the answer: no second fetch
    expect(t.effects.whereType<GetSources>(), isEmpty);
  });

  test('a refused edit keeps the last good project and the draft, with why', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.split)).state;
    s = reduce(s, SourcesReceived(sources())).state;
    const typed = 'concept Tilt : \n';
    s = reduce(s, const SourceTextChanged('src/main.bdl', typed)).state;
    s = reduce(s, const SourceEditRequested('src/main.bdl', typed)).state;
    final fault = pb.SourceDiagnostic(
      path: 'src/main.bdl',
      code: 'syntax',
      message: 'expected a type',
      start: 15,
      end: 16,
    );
    final t = reduce(
      s,
      SourceEditApplied(
        pb.SourceEditApplied(
          accepted: false,
          project: project(),
          sources: sources(text: typed, draft: true, diagnostics: [fault]),
        ),
      ),
    );
    expect(t.state.revision, 3, reason: 'no revision');
    expect(t.state.flat!.concepts, hasLength(1), reason: 'nothing discarded');
    expect(t.state.editor.sources.outOfSync, isTrue);
    expect(t.state.editor.sources.text, typed, reason: 'the draft exactly as typed');
    expect(t.state.editor.sources.diagnosticsOf('src/main.bdl'), [fault]);
    expect(t.state.editor.pendingRequests, 0);
    expect(t.state.editor.lastError, isNull, reason: 'a draft is a state, not an error');
  });

  test('a stale edit is resent once the sources catch up; typing during a send is the next', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.code)).state;
    s = reduce(s, SourcesReceived(sources())).state;
    const typed = 'concept Tilt : Angle\nconcept A : Scalar\n';
    s = reduce(s, const SourceTextChanged('src/main.bdl', typed)).state;
    s = reduce(s, const SourceEditRequested('src/main.bdl', typed)).state;
    // the project moved under the edit (a graph edit from elsewhere)
    s = reduce(s, ProjectReceived(project(revision: 4), fromRequest: false)).state;
    var t = reduce(s, const RequestFailed(code: 'edit.stale_revision', message: 'moved'));
    expect(t.state.editor.lastError, isNull);
    expect(t.state.editor.sources.retry, typed);
    expect(t.effects.whereType<GetSources>(), hasLength(1));
    t = reduce(t.state, SourcesReceived(sources(revision: 4)));
    final resend = t.effects.whereType<ApplySourceEdit>().single;
    expect(resend.baseRevision, 4);
    expect(resend.text, typed);
    expect(t.state.editor.sources.retry, isNull);
    expect(t.state.editor.sources.text, typed, reason: 'the buffer survives the catch-up');

    // typing while an edit is in flight: sent when the answer comes
    const more = 'concept Tilt : Angle\nconcept A : Scalar\nconcept B : Scalar\n';
    s = reduce(t.state, const SourceTextChanged('src/main.bdl', more)).state;
    t = reduce(s, const SourceEditRequested('src/main.bdl', more));
    expect(t.effects, isEmpty, reason: 'one in flight at a time');
    t = reduce(
      t.state,
      SourceEditApplied(
        pb.SourceEditApplied(
          accepted: true,
          project: project(revision: 5),
          sources: sources(revision: 5, text: typed),
        ),
      ),
    );
    final next = t.effects.whereType<ApplySourceEdit>().single;
    expect(next.baseRevision, 5);
    expect(next.text, more);
    expect(t.state.editor.sources.text, more, reason: 'the editor keeps what was typed');
  });

  test('closing the project forgets the sources; reopening fetches them again', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.code)).state;
    s = reduce(s, SourcesReceived(sources())).state;
    s = reduce(s, const ProjectClosed()).state;
    expect(s.editor.sources.revision, -1);
    expect(s.editor.view, DesignView.code, reason: 'the view is a preference');
    final t = reduce(s, ProjectReceived(project()));
    expect(t.effects.whereType<GetSources>(), hasLength(1));
  });
}
