/// Definition drafts through the pure reducer: creation, generations,
/// rebasing, conflicts, commit (attach vs replace), failure, detach,
/// deletion, close/reopen stash.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

const tilt = 0;
const brightness = 1;
const dim = 0;

pb.ProjectProjection lamp({
  int revision = 1,
  String root = '/p',
  String? definition,
  List<int> inputs = const [tilt],
  bool withMapping = true,
}) {
  final p = pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: root)
    ..concepts.addAll([
      pb.ConceptView(id: Int64(tilt), name: 'Tilt'),
      pb.ConceptView(id: Int64(brightness), name: 'Brightness'),
    ]);
  if (withMapping) {
    p.mappings.add(
      pb.MappingView(
        id: Int64(dim),
        name: 'dimByTilt',
        signature: pb.Signature(inputs: inputs.map(Int64.new), output: Int64(brightness)),
        definition: definition == null ? null : pb.Definition(formula: definition),
      ),
    );
  }
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

AppState type(AppState s, String source) =>
    reduce(s, DefinitionDraftChanged(mappingId: dim, source: source)).state;

void main() {
  group('draft creation', () {
    test('typing creates a draft, asks the compiler once per change, generation climbs', () {
      final t1 = reduce(
        connected(lamp()),
        const DefinitionDraftChanged(mappingId: dim, source: 'T'),
      );
      final d1 = t1.state.draft(dim)!;
      expect(d1.source, 'T');
      expect(d1.baseRevision, 1);
      expect(d1.baseDefinition, isNull);
      expect(d1.generation, 1);
      expect(d1.check, DraftCheck.checking);
      final e1 = t1.effects.single as AnalyzeDraft;
      expect((e1.revision, e1.mappingId, e1.generation, e1.source), (1, dim, 1, 'T'));
      // not a counted request: the app is not "busy"
      expect(t1.state.editor.pendingRequests, 0);

      final t2 = reduce(t1.state, const DefinitionDraftChanged(mappingId: dim, source: 'Ti'));
      expect(t2.state.draft(dim)!.generation, 2);
      expect((t2.effects.single as AnalyzeDraft).generation, 2);
    });

    test('typing the committed text back dissolves the draft', () {
      var s = connected(lamp(definition: 'Tilt / 90 deg'));
      s = type(s, 'Tilt / 90');
      expect(s.draft(dim), isNotNull);
      s = type(s, 'Tilt / 90 deg');
      expect(s.draft(dim), isNull);
      // an unresolved mapping: emptying the field is the same
      s = type(connected(lamp()), 'x');
      s = type(s, '');
      expect(s.draft(dim), isNull);
    });

    test('whitespace-only text is a draft that is not sent for checking', () {
      final t = reduce(
        connected(lamp()),
        const DefinitionDraftChanged(mappingId: dim, source: ' '),
      );
      expect(t.effects, isEmpty);
      expect(t.state.draft(dim)!.check, DraftCheck.checked);
      expect(t.state.draft(dim)!.analysis, isNull);
    });

    test('typing for an unknown mapping or without a project does nothing', () {
      expect(
        reduce(
          const AppState(),
          const DefinitionDraftChanged(mappingId: dim, source: 'x'),
        ).state.editor.drafts,
        isEmpty,
      );
      expect(
        reduce(
          connected(lamp()),
          const DefinitionDraftChanged(mappingId: 99, source: 'x'),
        ).state.editor.drafts,
        isEmpty,
      );
    });
  });

  group('verdicts', () {
    test('only the latest generation at the held revision is accepted', () {
      var s = type(connected(lamp()), 'T');
      s = type(s, 'Ti');
      s = type(s, 'Tilt');
      expect(s.draft(dim)!.generation, 3);

      // an older verdict arrives late: ignored
      s = reduce(
        s,
        DraftAnalysisReceived(
          verdict(revision: 1, generation: 1, status: pb.MappingStatus.MAPPING_STATUS_INVALID),
        ),
      ).state;
      expect(s.draft(dim)!.check, DraftCheck.checking);
      expect(s.draft(dim)!.analysis, isNull);

      // the newest: accepted
      s = reduce(s, DraftAnalysisReceived(verdict(revision: 1, generation: 3))).state;
      expect(s.draft(dim)!.check, DraftCheck.checked);
      expect(s.draft(dim)!.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);

      // the older one again, after the newer: still ignored
      s = reduce(
        s,
        DraftAnalysisReceived(
          verdict(revision: 1, generation: 2, status: pb.MappingStatus.MAPPING_STATUS_INVALID),
        ),
      ).state;
      expect(s.draft(dim)!.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
    });

    test('a verdict for another revision is ignored', () {
      var s = type(connected(lamp(revision: 4)), 'Tilt');
      s = reduce(s, DraftAnalysisReceived(verdict(revision: 3, generation: 1))).state;
      expect(s.draft(dim)!.check, DraftCheck.checking);
      s = reduce(s, DraftAnalysisReceived(verdict(revision: 5, generation: 1))).state;
      expect(s.draft(dim)!.check, DraftCheck.checking);
    });

    test('a verdict for a mapping without a draft is ignored', () {
      final s = reduce(
        connected(lamp()),
        DraftAnalysisReceived(verdict(revision: 1, generation: 1)),
      ).state;
      expect(s.editor.drafts, isEmpty);
    });

    test('stale-revision failures wait for the projection; others mark unavailable', () {
      var s = type(connected(lamp()), 'Tilt');
      s = reduce(
        s,
        const DraftAnalysisFailed(
          mappingId: dim,
          generation: 1,
          code: 'draft.stale_revision',
          message: 'moved on',
        ),
      ).state;
      expect(s.draft(dim)!.check, DraftCheck.checking);
      s = reduce(
        s,
        const DraftAnalysisFailed(
          mappingId: dim,
          generation: 1,
          code: 'studio.transport',
          message: 'gone',
        ),
      ).state;
      expect(s.draft(dim)!.check, DraftCheck.unavailable);
      expect(s.draft(dim)!.checkError, 'gone');
      expect(s.draft(dim)!.source, 'Tilt', reason: 'source is never lost');
      // a failure of an older generation says nothing about the current text
      s = type(s, 'Tilt /');
      s = reduce(
        s,
        const DraftAnalysisFailed(mappingId: dim, generation: 1, code: 'x', message: 'old'),
      ).state;
      expect(s.draft(dim)!.check, DraftCheck.checking);
    });
  });

  group('pushes', () {
    test('a project analysis push leaves the draft untouched', () {
      var s = type(connected(lamp()), 'Tilt / 90 deg');
      s = reduce(s, DraftAnalysisReceived(verdict(revision: 1, generation: 1))).state;
      final before = s.draft(dim);
      s = reduce(s, AnalysisReceived(pb.ProjectAnalysis(revision: Int64(1)))).state;
      expect(s.draft(dim), same(before));
      expect(s.analysis, isNotNull);
    });

    test('a new revision rebases the draft and checks it again', () {
      var s = type(connected(lamp()), 'Tilt / 90 deg');
      s = reduce(s, DraftAnalysisReceived(verdict(revision: 1, generation: 1))).state;
      // the signature loses Tilt (revision 2): the same text must be re-judged
      final t = reduce(s, ProjectReceived(lamp(revision: 2, inputs: []), fromRequest: false));
      final d = t.state.draft(dim)!;
      expect(d.source, 'Tilt / 90 deg');
      expect(d.baseRevision, 2);
      expect(d.generation, 2);
      expect(d.check, DraftCheck.checking);
      expect(d.analysis, isNull, reason: 'the old verdict is about another revision');
      expect(d.conflict, isFalse);
      final e = t.effects.whereType<AnalyzeDraft>().single;
      expect((e.revision, e.generation, e.source), (2, 2, 'Tilt / 90 deg'));
      // the late verdict for revision 1 is now ignored, the new one applies
      var s2 = reduce(
        t.state,
        DraftAnalysisReceived(
          verdict(revision: 1, generation: 1, status: pb.MappingStatus.MAPPING_STATUS_INVALID),
        ),
      ).state;
      expect(s2.draft(dim)!.check, DraftCheck.checking);
      s2 = reduce(
        s2,
        DraftAnalysisReceived(
          verdict(revision: 2, generation: 2, status: pb.MappingStatus.MAPPING_STATUS_INVALID),
        ),
      ).state;
      expect(s2.draft(dim)!.analysis!.status, pb.MappingStatus.MAPPING_STATUS_INVALID);
    });

    test('a same-revision projection (a save) changes nothing', () {
      var s = type(connected(lamp()), 'Tilt');
      final before = s.draft(dim);
      final t = reduce(s, ProjectReceived(lamp(revision: 1)));
      expect(t.state.draft(dim), same(before));
      expect(t.effects, isEmpty);
    });

    test('the committed definition changing under a dirty draft is a conflict, not a loss', () {
      var s = type(connected(lamp(definition: 'Tilt / 90 deg')), 'Tilt / 45 deg');
      s = reduce(
        s,
        ProjectReceived(lamp(revision: 2, definition: 'Tilt / 180 deg'), fromRequest: false),
      ).state;
      final d = s.draft(dim)!;
      expect(d.conflict, isTrue);
      expect(d.source, 'Tilt / 45 deg');
      expect(s.committedDefinition(dim), 'Tilt / 180 deg');
      // a conflicting draft cannot be committed by accident
      expect(reduce(s, const CommitDefinitionRequested(dim)).effects, isEmpty);
      // reload: take theirs
      final reloaded = reduce(s, const DefinitionDraftReloaded(dim)).state;
      expect(reloaded.draft(dim), isNull);
      // keep: mine, rebased on what is committed now
      final kept = reduce(s, const DefinitionDraftKept(dim)).state.draft(dim)!;
      expect(kept.conflict, isFalse);
      expect(kept.baseDefinition, 'Tilt / 180 deg');
      expect(kept.source, 'Tilt / 45 deg');
      final committed = reduce(
        reduce(s, const DefinitionDraftKept(dim)).state,
        const CommitDefinitionRequested(dim),
      );
      expect(committed.effects.single, isA<ApplyEdit>());
    });

    test('a clean draft under a changed definition is simply reloaded', () {
      // the draft equals what was committed at its base: nothing to protect
      var s = connected(lamp(definition: 'a'));
      s = s.copyWith(
        editor: s.editor.copyWith(
          drafts: {
            dim: const DefinitionDraft(
              mappingId: dim,
              baseRevision: 1,
              baseDefinition: 'a',
              source: 'a',
              pendingCommit: 'zzz',
            ),
          },
        ),
      );
      s = reduce(s, ProjectReceived(lamp(revision: 2, definition: 'b'), fromRequest: false)).state;
      expect(s.draft(dim), isNull);
    });

    test('the mapping being deleted removes its draft', () {
      var s = type(connected(lamp()), 'Tilt');
      s = reduce(
        s,
        ProjectReceived(lamp(revision: 2, withMapping: false), fromRequest: false),
      ).state;
      expect(s.editor.drafts, isEmpty);
      expect(s.editor.selection, isA<NoSelection>());
    });
  });

  group('commit', () {
    test('attach when there is no definition, replace when there is', () {
      final attach = reduce(
        type(connected(lamp()), 'Tilt / 90 deg'),
        const CommitDefinitionRequested(dim),
      );
      final op1 = (attach.effects.single as ApplyEdit).op;
      expect(op1.hasAttachDefinition(), isTrue);
      expect(op1.attachDefinition.definition.formula, 'Tilt / 90 deg');
      expect((attach.effects.single as ApplyEdit).baseRevision, 1);
      expect(attach.state.draft(dim)!.pendingCommit, 'Tilt / 90 deg');
      expect(attach.state.editor.pendingRequests, 1);

      final replace = reduce(
        type(connected(lamp(definition: 'Tilt')), ' Tilt / 2 '),
        const CommitDefinitionRequested(dim),
      );
      final op2 = (replace.effects.single as ApplyEdit).op;
      expect(op2.hasReplaceDefinition(), isTrue);
      expect(op2.replaceDefinition.definition.formula, 'Tilt / 2', reason: 'trimmed');
    });

    test('a commit is sent once; a second request while pending is ignored', () {
      final first = reduce(type(connected(lamp()), 'Tilt'), const CommitDefinitionRequested(dim));
      final second = reduce(first.state, const CommitDefinitionRequested(dim));
      expect(second.effects, isEmpty);
    });

    test('empty or unchanged text commits nothing', () {
      expect(
        reduce(type(connected(lamp()), '  '), const CommitDefinitionRequested(dim)).effects,
        isEmpty,
      );
      expect(
        reduce(connected(lamp(definition: 'a')), const CommitDefinitionRequested(dim)).effects,
        isEmpty,
      );
      // whitespace around the committed text: dissolve, no edit
      final t = reduce(
        type(connected(lamp(definition: 'a')), ' a '),
        const CommitDefinitionRequested(dim),
      );
      expect(t.effects, isEmpty);
      expect(t.state.draft(dim), isNull);
    });

    test('the projection confirming the commit clears the draft', () {
      var s = reduce(
        type(connected(lamp()), 'Tilt / 90 deg'),
        const CommitDefinitionRequested(dim),
      ).state;
      s = reduce(s, ProjectReceived(lamp(revision: 2, definition: 'Tilt / 90 deg'))).state;
      expect(s.draft(dim), isNull);
      expect(s.editor.pendingRequests, 0);
      expect(s.committedDefinition(dim), 'Tilt / 90 deg');
    });

    test('typing on while the commit is in flight keeps the newer text as a draft', () {
      var s = reduce(
        type(connected(lamp()), 'Tilt / 90 deg'),
        const CommitDefinitionRequested(dim),
      ).state;
      s = type(s, 'Tilt / 90 deg + 1');
      final t = reduce(s, ProjectReceived(lamp(revision: 2, definition: 'Tilt / 90 deg')));
      final d = t.state.draft(dim)!;
      expect(d.source, 'Tilt / 90 deg + 1');
      expect(d.pendingCommit, isNull);
      expect(d.conflict, isFalse, reason: 'our own commit is not a conflict');
      expect(d.baseDefinition, 'Tilt / 90 deg');
      expect(t.effects.whereType<AnalyzeDraft>().single.source, 'Tilt / 90 deg + 1');
    });

    test('a failed commit preserves the text and reports next to it', () {
      var s = reduce(
        type(connected(lamp()), 'Tilt / 90 deg'),
        const CommitDefinitionRequested(dim),
      ).state;
      s = reduce(
        s,
        const RequestFailed(code: 'edit.stale_revision', message: 'the project moved on'),
      ).state;
      final d = s.draft(dim)!;
      expect(d.source, 'Tilt / 90 deg');
      expect(d.pendingCommit, isNull);
      expect(d.commitError, 'the project moved on');
      expect(s.editor.pendingRequests, 0);
      // it can be tried again
      expect(reduce(s, const CommitDefinitionRequested(dim)).effects.single, isA<ApplyEdit>());
      // and typing clears the stale message
      expect(type(s, 'Tilt').draft(dim)!.commitError, isNull);
    });

    test('revert drops the draft and its overlay without touching the project', () {
      final t = reduce(
        type(connected(lamp(definition: 'a')), 'b'),
        const DefinitionDraftReverted(dim),
      );
      expect(t.state.draft(dim), isNull);
      expect(t.effects.single, isA<DiscardDraft>());
      expect((t.effects.single as DiscardDraft).mappingId, dim);
      expect(t.effects.whereType<ApplyEdit>(), isEmpty);
      // reload after a conflict is the same act
      final reload = reduce(
        type(connected(lamp(definition: 'a')), 'b'),
        const DefinitionDraftReloaded(dim),
      );
      expect(reload.effects.single, isA<DiscardDraft>());
    });
  });

  group('detach', () {
    test('sends one replace-with-nothing and resets the draft and its overlay', () {
      final t = reduce(
        type(connected(lamp(definition: 'a')), 'b'),
        const DetachDefinitionRequested(dim),
      );
      expect(t.effects.whereType<DiscardDraft>(), hasLength(1));
      final op = t.effects.whereType<ApplyEdit>().single.op;
      expect(op.hasReplaceDefinition(), isTrue);
      expect(op.replaceDefinition.hasDefinition(), isFalse);
      expect(t.state.draft(dim), isNull);
      // the projection then shows the mapping unresolved; nothing lingers
      final s = reduce(t.state, ProjectReceived(lamp(revision: 2))).state;
      expect(s.committedDefinition(dim), isNull);
      expect(s.editor.drafts, isEmpty);
    });

    test('is a no-op without a definition; nothing to discard without a draft', () {
      expect(reduce(connected(lamp()), const DetachDefinitionRequested(dim)).effects, isEmpty);
      final clean = reduce(connected(lamp(definition: 'a')), const DetachDefinitionRequested(dim));
      expect(clean.effects.single, isA<ApplyEdit>());
    });
  });

  group('close and reopen', () {
    test('dirty drafts are stashed on close and restored on reopen of the same project', () {
      var s = type(connected(lamp()), 'Tilt / 90 deg');
      s = reduce(s, const CloseProjectRequested()).state;
      s = reduce(s, const ProjectClosed()).state;
      expect(s.project, isNull);
      expect(s.editor.drafts, isEmpty);
      expect(s.editor.stashedDrafts['/p']![dim]!.source, 'Tilt / 90 deg');

      // another project: nothing restored
      final other = reduce(s, ProjectReceived(lamp(root: '/q'))).state;
      expect(other.editor.drafts, isEmpty);
      expect(other.editor.stashedDrafts.containsKey('/p'), isTrue);

      // the same project: restored, rebased on its opening revision, checked
      final t = reduce(s, ProjectReceived(lamp(revision: 0)));
      final d = t.state.draft(dim)!;
      expect(d.source, 'Tilt / 90 deg');
      expect(d.baseRevision, 0);
      expect(d.check, DraftCheck.checking);
      expect(t.effects.whereType<AnalyzeDraft>().single.revision, 0);
      expect(t.state.editor.stashedDrafts.containsKey('/p'), isFalse);
    });

    test('a stashed draft whose definition was committed meanwhile is a conflict on reopen', () {
      var s = type(connected(lamp(definition: 'a')), 'b');
      s = reduce(s, const ProjectClosed()).state;
      s = reduce(s, ProjectReceived(lamp(revision: 0, definition: 'c'))).state;
      expect(s.draft(dim)!.conflict, isTrue);
      // …and one that now equals the committed text is not a draft at all
      var s2 = type(connected(lamp(definition: 'a')), 'b');
      s2 = reduce(s2, const ProjectClosed()).state;
      s2 = reduce(s2, ProjectReceived(lamp(revision: 0, definition: 'b'))).state;
      expect(s2.draft(dim), isNull);
    });

    test('the daemon exiting stashes drafts too', () {
      var s = type(connected(lamp()), 'Tilt');
      s = reduce(s, const DaemonExited(1)).state;
      expect(s.editor.drafts, isEmpty);
      expect(s.editor.stashedDrafts['/p'], isNotNull);
    });

    test('drafts are per mapping: another mapping keeps its own text', () {
      final p = lamp()
        ..mappings.add(
          pb.MappingView(
            id: Int64(5),
            name: 'other',
            signature: pb.Signature(inputs: [Int64(tilt)], output: Int64(brightness)),
          ),
        );
      var s = type(connected(p), 'one');
      s = reduce(s, const DefinitionDraftChanged(mappingId: 5, source: 'two')).state;
      s = reduce(s, const SelectionChanged(MappingSelected(5))).state;
      expect(s.draft(dim)!.source, 'one');
      expect(s.draft(5)!.source, 'two');
    });
  });
}
