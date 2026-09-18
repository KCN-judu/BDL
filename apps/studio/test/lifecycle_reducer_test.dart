/// The one guard every project-unloading path goes through
/// (app/lifecycle.dart): a clean project closes at once; a dirty one asks
/// *Save / Don't Save / Cancel*; Save unloads only after the save succeeded;
/// a failed save keeps the project open; Open / New / Quit while a project
/// is open run their intent after the close; unsent typing is flushed first.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

pb.ProjectProjection lamp({bool dirty = false, int revision = 1}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p', dirty: dirty)
      ..kind = pb.ProjectKind.PROJECT_KIND_TEXT;

AppState connected() => AppState(
  connection: Connected(executable: 'x', handshake: pb.HandshakeResponse(compatible: true)),
);

/// A project as Studio holds it after open.
AppState opened({bool dirty = false}) {
  var s = reduce(connected(), ProjectReceived(lamp(dirty: dirty))).state;
  s = reduce(s, SystemReceived(pb.SystemView(revision: Int64(1)))).state;
  return s;
}

/// The project answers the guard's question.
Transition answered(AppState s, {required bool dirty}) =>
    reduce(s, ProjectReceived(lamp(dirty: dirty)));

void main() {
  test('closing a clean project asks the project once and closes at once', () {
    var t = reduce(opened(), const CloseProjectRequested());
    expect(t.effects.whereType<GetProject>(), hasLength(1));
    expect(t.state.editor.unloading, const CloseOnly());
    expect(t.state.editor.closeGuard, isNull);
    t = answered(t.state, dirty: false);
    expect(t.state.editor.closeGuard, isNull, reason: 'no question for a clean project');
    expect(t.effects.whereType<CloseProject>(), hasLength(1));
    expect(t.state.editor.afterClose, const CloseOnly());
    final closed = reduce(t.state, const ProjectClosed());
    expect(closed.state.project, isNull);
    expect(closed.effects, isEmpty);
    expect(closed.state.editor.afterClose, isNull);
  });

  test('a dirty project raises the sheet; Cancel leaves everything as it was', () {
    var t = reduce(opened(), const CloseProjectRequested());
    t = answered(t.state, dirty: true);
    expect(t.state.editor.closeGuard, const CloseOnly());
    expect(t.effects.whereType<CloseProject>(), isEmpty);
    t = reduce(t.state, const CloseGuardAnswered(CloseGuardAnswer.cancel));
    expect(t.state.editor.closeGuard, isNull);
    expect(t.state.editor.unloadInProgress, isFalse);
    expect(t.state.project, isNotNull);
    expect(t.effects, isEmpty);
  });

  test('Don’t Save closes without saving', () {
    var t = reduce(opened(), const CloseProjectRequested());
    t = answered(t.state, dirty: true);
    t = reduce(t.state, const CloseGuardAnswered(CloseGuardAnswer.dontSave));
    expect(t.effects.whereType<SaveProject>(), isEmpty);
    expect(t.effects.whereType<CloseProject>(), hasLength(1));
    expect(t.state.editor.afterClose, const CloseOnly());
  });

  test('Save closes only after the save succeeded', () {
    var t = reduce(opened(), const CloseProjectRequested());
    t = answered(t.state, dirty: true);
    t = reduce(t.state, const CloseGuardAnswered(CloseGuardAnswer.save));
    expect(t.effects.whereType<SaveProject>(), hasLength(1));
    expect(t.effects.whereType<CloseProject>(), isEmpty, reason: 'not before the save answers');
    expect(t.state.editor.closeAfterSave, const CloseOnly());
    // the save's answer: clean
    t = reduce(t.state, ProjectReceived(lamp(dirty: false)));
    expect(t.effects.whereType<CloseProject>(), hasLength(1));
    expect(t.state.editor.closeAfterSave, isNull);
    expect(t.state.editor.afterClose, const CloseOnly());
  });

  test('a failed save keeps the project open and shows why', () {
    var t = reduce(opened(), const CloseProjectRequested());
    t = answered(t.state, dirty: true);
    t = reduce(t.state, const CloseGuardAnswered(CloseGuardAnswer.save));
    t = reduce(
      t.state,
      const RequestFailed(code: 'project.changed_on_disk', message: 'changed on disk'),
    );
    expect(t.state.project, isNotNull);
    expect(t.state.editor.lastError?.code, 'project.changed_on_disk');
    expect(t.state.editor.closeAfterSave, isNull);
    expect(t.state.editor.afterClose, isNull);
    expect(t.state.editor.unloadInProgress, isFalse);
    expect(t.effects.whereType<CloseProject>(), isEmpty);
    // the designer may try again
    expect(
      reduce(t.state, const CloseProjectRequested()).effects.whereType<GetProject>(),
      hasLength(1),
    );
  });

  test('opening another project runs the guard, then opens it after the close', () {
    var t = reduce(opened(), const OpenProjectRequested('/q'));
    expect(t.effects.whereType<OpenProject>(), isEmpty, reason: 'the guard first');
    expect(t.state.editor.unloading, const OpenAnother('/q'));
    t = answered(t.state, dirty: true);
    expect(t.state.editor.closeGuard, const OpenAnother('/q'));
    t = reduce(t.state, const CloseGuardAnswered(CloseGuardAnswer.dontSave));
    expect(t.effects.whereType<CloseProject>(), hasLength(1));
    final closed = reduce(t.state, const ProjectClosed());
    expect(closed.effects.whereType<OpenProject>().single.rootPath, '/q');
    expect(closed.state.editor.pendingRequests, 1, reason: 'the open is counted');
  });

  test('New Project and the pickers go through the guard too', () {
    var t = reduce(opened(), const NewProjectRequested(rootPath: '/n', name: 'n'));
    expect(t.state.editor.unloading, const CreateAnother(rootPath: '/n', name: 'n'));
    t = reduce(opened(), const OpenProjectPickRequested());
    expect(t.state.editor.unloading, const PickAnother());
    t = reduce(opened(), const NewProjectPickRequested());
    expect(t.state.editor.unloading, const PickNew());
    // with nothing open they run at once
    expect(
      reduce(connected(), const OpenProjectPickRequested()).effects.whereType<PickProjectToOpen>(),
      hasLength(1),
    );
  });

  test('quit runs the guard and leaves once the project is closed', () {
    var t = reduce(opened(), const QuitRequested());
    expect(t.effects.whereType<QuitApplication>(), isEmpty);
    t = answered(t.state, dirty: false);
    expect(t.effects.whereType<CloseProject>(), hasLength(1));
    final closed = reduce(t.state, const ProjectClosed());
    expect(closed.effects.whereType<QuitApplication>(), hasLength(1));
    // with nothing open: at once
    expect(
      reduce(connected(), const QuitRequested()).effects.whereType<QuitApplication>(),
      hasLength(1),
    );
  });

  test('a second unload request while one is in progress is ignored', () {
    var t = reduce(opened(), const CloseProjectRequested());
    final again = reduce(t.state, const QuitRequested());
    expect(again.effects, isEmpty);
    expect(again.state.editor.unloading, const CloseOnly());
  });

  test('unsent typing reaches the project before it is asked or saved', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.code)).state;
    s = reduce(
      s,
      SourcesReceived(
        pb.SourcesView(
          revision: Int64(1),
          files: [pb.SourceFileView(path: 'src/main.bdl', text: 'a\n')],
        ),
      ),
    ).state;
    s = reduce(s, const SourceTextChanged('src/main.bdl', 'a\nb\n')).state;
    // close: the typed text goes first, the question after its answer
    var t = reduce(s, const CloseProjectRequested());
    expect(t.effects.whereType<ApplySourceEdit>().single.text, 'a\nb\n');
    expect(t.effects.whereType<GetProject>(), isEmpty);
    t = reduce(
      t.state,
      SourceEditApplied(
        pb.SourceEditApplied(
          accepted: false,
          project: lamp(dirty: true),
          sources: pb.SourcesView(
            revision: Int64(1),
            files: [pb.SourceFileView(path: 'src/main.bdl', text: 'a\nb\n', draft: true)],
          ),
        ),
      ),
    );
    expect(t.effects.whereType<GetProject>(), hasLength(1));
    // save: the same, then the save
    t = reduce(s, const SaveRequested());
    expect(t.effects.whereType<ApplySourceEdit>(), hasLength(1));
    expect(t.effects.whereType<SaveProject>(), isEmpty);
    expect(t.state.editor.pendingSave, isFalse);
    t = reduce(
      t.state,
      SourceEditApplied(
        pb.SourceEditApplied(
          accepted: true,
          project: lamp(revision: 2, dirty: true),
          sources: pb.SourcesView(
            revision: Int64(2),
            files: [pb.SourceFileView(path: 'src/main.bdl', text: 'a\nb\n')],
          ),
        ),
      ),
    );
    expect(t.effects.whereType<SaveProject>(), hasLength(1));
    expect(t.state.editor.pendingSave, isNull);
  });

  test('closing remembers where the designer was, and reopening returns there', () {
    var s = reduce(opened(), const DesignViewChanged(DesignView.split)).state;
    s = reduce(s, const PageSelected(StudioPage.simulate)).state;
    var t = reduce(s, const CloseProjectRequested());
    t = answered(t.state, dirty: false);
    final saved = t.effects.whereType<SaveRecentProjects>().single.recent;
    final ws = saved.firstWhere((r) => r.path == '/p').workspace!;
    expect(ws.view, 'split');
    expect(ws.page, 'simulate');
    final closed = reduce(t.state, const ProjectClosed()).state;
    final reopened = reduce(closed, ProjectReceived(lamp())).state;
    expect(reopened.editor.view, DesignView.split);
    expect(reopened.editor.page, StudioPage.simulate);
  });
}
