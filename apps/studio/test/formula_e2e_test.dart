/// End to end through the real Studio stack — reducer, effect executor,
/// Dart client — against the real `bdld`:
///
/// * the Tilt / Brightness path: draft, compiler verdict, add, analysis,
///   save, close, reopen with the source and status intact;
/// * the invalid path: `Tilt + 1 s` is a local dimension diagnostic with
///   an exact span, corrected to `Tilt / 90 deg`, saved;
/// * the stale path: a draft based on revision N re-judged at N+1 when the
///   signature changes and when a representation changes, older verdicts
///   dropped;
/// * detach, and a commit refused by the daemon.
///
/// Skipped when the binary is absent (run `cargo build`).  Timings are
/// printed for docs/architecture/studio-compiler-integration.md.
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

import 'support/test_store.dart';

String? _findBdld() {
  final env = Platform.environment['BDLD_PATH'];
  if (env != null && File(env).existsSync()) return env;
  for (final rel in ['../../target/debug/bdld', '../../target/release/bdld']) {
    final f = p.normalize(p.join(Directory.current.path, rel));
    if (File(f).existsSync()) return f;
  }
  return null;
}

pb.Representation quantity(pb.Dim dim) => pb.Representation(quantity: dim);

void main() {
  final bdld = _findBdld();

  late TestStore store;
  late Directory dir;
  late String root;

  int mappingId() => store.state.project!.mappings.single.id.toInt();
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  DefinitionDraft draft() => store.state.draft(mappingId())!;

  Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
  Future<DefinitionDraft> checked() async {
    final s = await store.until((s) {
      final d = s.draft(mappingId());
      return d != null && d.check == DraftCheck.checked;
    });
    return s.draft(mappingId())!;
  }

  Future<void> project() async {
    store = TestStore(
      spawn: DaemonClient.spawn,
      executable: bdld!,
      draftDebounce: const Duration(milliseconds: 30),
    );
    store.dispatch(const AppStarted());
    await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
    expect(store.state.connection, isA<Connected>(), reason: '${store.state.connection}');
    dir = await Directory.systemTemp.createTemp('bdl-studio-e2e');
    root = p.join(dir.path, 'lamp');
    store.dispatch(NewProjectRequested(rootPath: root, name: 'lamp'));
    await store.until((s) => s.project != null);
    store.dispatch(
      CreateConceptRequested(name: 'Tilt', representation: quantity(pb.Dim(angle: 1))),
    );
    await settled();
    store.dispatch(CreateConceptRequested(name: 'Brightness', representation: quantity(pb.Dim())));
    await settled();
    store.dispatch(
      CreateMappingRequested(
        name: 'dimByTilt',
        inputs: [conceptId('Tilt')],
        output: conceptId('Brightness'),
      ),
    );
    await settled();
    store.dispatch(SelectionChanged(MappingSelected(mappingId())));
    await store.until((s) => s.analysis != null);
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  test('valid path: draft → verdict → add → analysis → save → reopen', () async {
    await project();
    try {
      final id = mappingId();
      final t0 = DateTime.now();
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt / 90 deg'));
      final d = await checked();
      final dt = DateTime.now().difference(t0);
      // ignore: avoid_print
      print(
        'draft edit → verdict rendered in state: ${dt.inMilliseconds} ms (incl. 30 ms debounce)',
      );
      expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
      expect(d.analysis!.diagnostics, isEmpty);
      expect(d.parseOk, isTrue);
      // nothing committed by checking
      expect(store.state.committedDefinition(id), isNull);
      expect(store.state.project!.canUndo, isTrue, reason: 'from the creates, not the draft');
      final revisionBefore = store.state.revision;

      store.dispatch(CommitDefinitionRequested(id));
      expect(draft().pendingCommit, 'Tilt / 90 deg');
      var s = await store.until((s) => s.committedDefinition(id) == 'Tilt / 90 deg');
      expect(s.revision, revisionBefore + 1);
      expect(s.editor.lastOutcome!.kind, pb.EditKind.EDIT_KIND_REFINEMENT);
      s = await store.until((s) => s.draft(id) == null && s.editor.pendingRequests == 0);
      // the pushed analysis for the new revision arrives
      s = await store.until((s) => s.analysis?.revision.toInt() == s.revision);
      expect(s.mappingAnalysis(id)!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);

      store.dispatch(const SaveRequested());
      s = await store.until((s) => !s.project!.dirty && s.editor.pendingRequests == 0);
      store.dispatch(const CloseProjectRequested());
      await store.until((s) => s.project == null);
      store.dispatch(OpenProjectRequested(root));
      s = await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
      expect(s.project!.mappings.single.definition.formula, 'Tilt / 90 deg');
      s = await store.until((s) => s.analysis != null);
      expect(
        s.mappingAnalysis(mappingId())!.status,
        pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
      );
      expect(s.editor.drafts, isEmpty);
    } finally {
      await teardown();
    }
  }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);

  test('invalid path: a dimension error with an exact span, then corrected and saved', () async {
    await project();
    try {
      final id = mappingId();
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt + 1 s'));
      var d = await checked();
      expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_INVALID);
      final dim = d.analysis!.diagnostics.firstWhere((x) => x.code == 'dimension.mismatch');
      expect(dim.severity, pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR);
      expect(dim.hasSpan(), isTrue);
      expect('Tilt + 1 s'.substring(dim.span.start, dim.span.end), 'Tilt + 1 s');
      expect(d.source, 'Tilt + 1 s', reason: 'an invalid draft is kept');
      expect(store.state.committedDefinition(id), isNull);
      expect(store.state.editor.lastError, isNull, reason: 'no banner for a draft verdict');

      // the model allows committing an invalid definition: it is then Invalid
      store.dispatch(CommitDefinitionRequested(id));
      var s = await store.until((s) => s.committedDefinition(id) == 'Tilt + 1 s');
      s = await store.until((s) => s.analysis?.revision.toInt() == s.revision);
      expect(s.mappingAnalysis(id)!.status, pb.MappingStatus.MAPPING_STATUS_INVALID);

      // corrected: the error goes, the valid verdict comes, save replaces
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt / 90 deg'));
      d = await checked();
      expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
      expect(d.analysis!.diagnostics, isEmpty);
      store.dispatch(CommitDefinitionRequested(id));
      s = await store.until((s) => s.committedDefinition(id) == 'Tilt / 90 deg');
      expect(s.editor.lastOutcome!.kind, pb.EditKind.EDIT_KIND_EDIT, reason: 'a replace');
      s = await store.until((s) => s.analysis?.revision.toInt() == s.revision);
      expect(s.mappingAnalysis(id)!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
      // syntax errors are local too
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt /'));
      d = await checked();
      expect(d.parseOk, isFalse);
      expect(d.analysis!.diagnostics.first.code, 'formula.parse.unexpected_token');
      expect(d.analysis!.diagnostics.first.hasSpan(), isTrue);
    } finally {
      await teardown();
    }
  }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);

  test(
    'stale path: signature and representation changes re-judge the draft at the new revision',
    () async {
      await project();
      try {
        final id = mappingId();
        store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt / 90 deg'));
        var d = await checked();
        expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
        final n = d.baseRevision;

        // the signature loses Tilt while the draft is dirty
        store.dispatch(UnlinkMappingInput(mappingId: id, conceptId: conceptId('Tilt')));
        var s = await store.until((s) => s.revision == n + 1);
        expect(s.draft(id)!.source, 'Tilt / 90 deg', reason: 'never erased');
        expect(s.draft(id)!.baseRevision, n + 1);
        d = await checked();
        expect(d.baseRevision, n + 1);
        expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_INVALID);
        expect(d.analysis!.diagnostics.any((x) => x.code.startsWith('formula.name.')), isTrue);

        // Tilt comes back; the draft is valid again
        store.dispatch(LinkConceptToMappingInput(conceptId: conceptId('Tilt'), mappingId: id));
        await store.until((s) => s.revision == n + 2);
        d = await checked();
        expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);

        // Tilt's representation changes: the same text now means something else
        store.dispatch(
          SetConceptRepresentationRequested(
            id: conceptId('Tilt'),
            representation: quantity(pb.Dim(time: 1)),
          ),
        );
        await store.until((s) => s.revision == n + 3);
        d = await checked();
        expect(d.source, 'Tilt / 90 deg');
        expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_INVALID);
        // and back
        store.dispatch(
          SetConceptRepresentationRequested(
            id: conceptId('Tilt'),
            representation: quantity(pb.Dim(angle: 1)),
          ),
        );
        await store.until((s) => s.revision == n + 4);
        d = await checked();
        expect(d.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
        // every verdict kept was for the revision held at the time
        s = store.state;
        expect(s.draft(id)!.baseRevision, s.revision);
      } finally {
        await teardown();
      }
    },
    skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false,
  );

  test('completion and hover come from bdl-ide, offsets in bytes, stale answers dropped', () async {
    await project();
    try {
      final id = mappingId();
      Future<CompletionState> complete(String source, int offset) async {
        store.dispatch(CompletionRequested(mappingId: id, source: source, offset: offset));
        final s = await store.until((s) => s.editor.completion?.pending == false);
        return s.editor.completion!;
      }

      final t0 = DateTime.now();
      var c = await complete('Ti', 2);
      // ignore: avoid_print
      print('completion round trip: ${DateTime.now().difference(t0).inMilliseconds} ms');
      final tiltItem = c.items.firstWhere((i) => i.label == 'Tilt');
      expect(tiltItem.kind, 'input');
      expect((tiltItem.replaceStart, tiltItem.replaceEnd), (0, 2));
      expect(c.items.first.label, 'Tilt', reason: 'the service ranks the input first');

      // a unit after a number
      c = await complete('Tilt / 90 d', 11);
      expect(c.items.any((i) => i.label == 'deg' && i.kind == 'unit'), isTrue);
      expect(c.items.first.kind, 'unit', reason: 'units rank first after a number');

      // an invalid partial source is fine
      c = await complete('Tilt / ', 7);
      expect(c.items, isNotEmpty);

      // a non-ASCII prefix: the service's range is in bytes and lands on `Ti`
      c = await complete('2 × Ti', 7);
      final after = c.items.firstWhere((i) => i.label == 'Tilt');
      expect((after.replaceStart, after.replaceEnd), (5, 7));

      // two requests back to back: only the newest generation is shown
      store.dispatch(CompletionRequested(mappingId: id, source: 'T', offset: 1));
      store.dispatch(CompletionRequested(mappingId: id, source: 'Ti', offset: 2));
      final g = store.state.editor.completion!.generation;
      final s = await store.until((s) => s.editor.completion?.pending == false);
      expect(s.editor.completion!.generation, g);
      expect(s.editor.completion!.offset, 2);

      // hover on the concept in the formula (the field's text is the draft)
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt / 90 deg'));
      await checked();
      final h0 = DateTime.now();
      store.dispatch(FormulaHoverRequested(mappingId: id, source: 'Tilt / 90 deg', offset: 1));
      var hs = await store.until((s) => s.editor.hover?.card != null);
      // ignore: avoid_print
      print('hover round trip: ${DateTime.now().difference(h0).inMilliseconds} ms');
      var card = hs.editor.hover!.card!;
      expect(card.found, isTrue);
      expect(card.title, 'Tilt');
      expect(card.conceptId.toInt(), conceptId('Tilt'));
      expect(card.representation, isNotEmpty);
      expect(card.span.start, 0);
      expect(card.span.end, 4);
      // nothing under a number
      store.dispatch(FormulaHoverRequested(mappingId: id, source: 'Tilt / 90 deg', offset: 8));
      hs = await store.until((s) => s.editor.hover?.card != null && s.editor.hover!.offset == 8);
      expect(hs.editor.hover!.card!.found, isFalse);
      // after the draft text changes, the hover is about the new text
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: '1 + Tilt'));
      await checked();
      store.dispatch(FormulaHoverRequested(mappingId: id, source: '1 + Tilt', offset: 5));
      hs = await store.until((s) => s.editor.hover?.card != null && s.editor.hover!.offset == 5);
      expect(hs.editor.hover!.card!.title, 'Tilt');
      expect(hs.editor.hover!.card!.span.start, 4);

      // hover on the mapping itself (a canvas node)
      store.dispatch(EntityHoverRequested(pb.EntityRef(mappingId: Int64(id))));
      hs = await store.until((s) => s.editor.hover?.card != null && s.editor.hover!.entity != null);
      card = hs.editor.hover!.card!;
      expect(card.title, 'dimByTilt');
      expect(card.signature, 'mapping dimByTilt : Tilt -> Brightness');
      // the card describes the effective world: the last hovered text,
      // `1 + Tilt`, is the mapping's draft there and does not check
      expect(card.open, isFalse);
      expect(card.status, contains('does not check'));
      // discarding the draft: declared again, an open state
      store.dispatch(DefinitionDraftReverted(id));
      await Future<void>.delayed(const Duration(milliseconds: 50));
      store.dispatch(const EntityHoverRequested(null));
      store.dispatch(EntityHoverRequested(pb.EntityRef(mappingId: Int64(id))));
      hs = await store.until((s) => s.editor.hover?.card != null && s.editor.hover!.entity != null);
      expect(hs.editor.hover!.card!.open, isTrue, reason: 'declared, no definition');
    } finally {
      await teardown();
    }
  }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);

  test('detach returns the mapping to unresolved; a refused commit keeps the draft', () async {
    await project();
    try {
      final id = mappingId();
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt / 90 deg'));
      await checked();
      store.dispatch(CommitDefinitionRequested(id));
      var s = await store.until((s) => s.committedDefinition(id) == 'Tilt / 90 deg');
      s = await store.until((s) => s.analysis?.revision.toInt() == s.revision);

      store.dispatch(DetachDefinitionRequested(id));
      s = await store.until((s) => s.committedDefinition(id) == null);
      expect(s.editor.drafts, isEmpty);
      expect(s.project!.mappings.single.state, pb.AcceptanceState.ACCEPTANCE_STATE_DECLARED);
      s = await store.until((s) => s.analysis?.revision.toInt() == s.revision);
      expect(s.mappingAnalysis(id)!.status, pb.MappingStatus.MAPPING_STATUS_DECLARED);
      expect(s.mappingAnalysis(id)!.coreExpr, isEmpty);

      // a commit against a revision the daemon has left behind is refused;
      // the draft survives with the reason
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Tilt / 2'));
      await checked();
      final behind = store.state.copyWith(project: store.state.project!.deepCopy()..revision -= 1);
      store.state = behind;
      store.dispatch(CommitDefinitionRequested(id));
      s = await store.until((s) => s.draft(id)?.commitError != null);
      expect(s.draft(id)!.source, 'Tilt / 2');
      expect(s.editor.lastError?.code, 'edit.stale_revision');
    } finally {
      await teardown();
    }
  }, skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false);
}
