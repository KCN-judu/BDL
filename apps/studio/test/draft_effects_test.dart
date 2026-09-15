/// Draft analysis through the real reducer and effect executor against a
/// scripted daemon: debounce, request shape, generation echo, out-of-order
/// answers, daemon errors and transport loss.
library;

import 'dart:async';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/test_store.dart';

const dim = 0;

pb.ProjectProjection lamp({int revision = 1}) =>
    pb.ProjectProjection(revision: Int64(revision), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(0), name: 'Tilt'),
        pb.ConceptView(id: Int64(1), name: 'Brightness'),
      ])
      ..mappings.add(
        pb.MappingView(
          id: Int64(dim),
          name: 'dimByTilt',
          signature: pb.Signature(inputs: [Int64(0)], output: Int64(1)),
        ),
      );

pb.Response draftOk(pb.AnalyzeDefinitionDraftRequest r, {pb.MappingStatus? status}) => pb.Response(
  definitionDraft: pb.DefinitionDraftAnalysis(
    revision: r.revision,
    mappingId: r.mappingId,
    generation: r.generation,
    parseOk: true,
    analysis: pb.MappingAnalysis(
      id: r.mappingId,
      status: status ?? pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
    ),
  ),
);

/// A connected store holding `lamp()` at revision 1, talking to [daemon].
Future<TestStore> connectedStore(FakeDaemon daemon) async {
  final store = TestStore(spawn: (_) async => daemon);
  store.dispatch(const AppStarted());
  await store.until((s) => s.connection is Connected);
  store.dispatch(ProjectReceived(lamp()));
  await store.until((s) => s.project != null);
  daemon.requests.clear();
  return store;
}

void main() {
  test('a burst of keystrokes is one debounced request carrying the last generation', () async {
    final daemon = FakeDaemon((m) {
      if (m.hasHandshake()) return okHandshake();
      if (m.hasAnalyzeDefinitionDraft()) return draftOk(m.analyzeDefinitionDraft);
      return pb.Response(ack: pb.Ack());
    });
    final store = await connectedStore(daemon);
    for (final text in ['T', 'Ti', 'Til', 'Tilt']) {
      store.dispatch(DefinitionDraftChanged(mappingId: dim, source: text));
    }
    expect(store.state.draft(dim)!.generation, 4);
    final s = await store.until((s) => s.draft(dim)?.check == DraftCheck.checked);
    final sent = daemon.requests.where((m) => m.hasAnalyzeDefinitionDraft()).toList();
    expect(sent, hasLength(1), reason: 'one request for four keystrokes');
    final r = sent.single.analyzeDefinitionDraft;
    expect(r.source, 'Tilt');
    expect(r.generation.toInt(), 4);
    expect(r.revision.toInt(), 1);
    expect(r.mappingId.toInt(), dim);
    expect(s.draft(dim)!.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
    // draft checks are not counted: nothing was "pending"
    expect(s.editor.pendingRequests, 0);
    await store.dispose();
  });

  test('an older answer arriving after a newer one cannot overwrite it', () async {
    // The daemon answers generation 1 slowly and marks it invalid; generation
    // 2 fast and valid.  The slow answer lands last and must be ignored.
    final gate = Completer<void>();
    final daemon = FakeDaemon((m) async {
      if (m.hasHandshake()) return okHandshake();
      if (m.hasAnalyzeDefinitionDraft()) {
        final r = m.analyzeDefinitionDraft;
        if (r.generation.toInt() == 1) {
          await gate.future;
          return draftOk(r, status: pb.MappingStatus.MAPPING_STATUS_INVALID);
        }
        return draftOk(r);
      }
      return pb.Response(ack: pb.Ack());
    });
    final store = await connectedStore(daemon);
    store.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'Tilt +'));
    // let the debounce fire so generation 1 is actually sent
    await Future<void>.delayed(const Duration(milliseconds: 40));
    expect(daemon.requests.where((m) => m.hasAnalyzeDefinitionDraft()), hasLength(1));
    store.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'Tilt + 1'));
    final s = await store.until((s) => s.draft(dim)?.check == DraftCheck.checked);
    expect(s.draft(dim)!.generation, 2);
    expect(s.draft(dim)!.analysis!.status, pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT);
    gate.complete();
    await Future<void>.delayed(const Duration(milliseconds: 20));
    expect(
      store.state.draft(dim)!.analysis!.status,
      pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
      reason: 'the late verdict for generation 1 was dropped',
    );
    await store.dispose();
  });

  test('a stale-revision refusal keeps the draft checking until the projection re-asks', () async {
    const daemonRevision = 2;
    final daemon = FakeDaemon((m) {
      if (m.hasHandshake()) return okHandshake();
      if (m.hasAnalyzeDefinitionDraft()) {
        final r = m.analyzeDefinitionDraft;
        if (r.revision.toInt() != daemonRevision) {
          return errorResponse('draft.stale_revision', 'moved on');
        }
        return draftOk(r);
      }
      return pb.Response(ack: pb.Ack());
    });
    final store = await connectedStore(daemon);
    store.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'Tilt'));
    await Future<void>.delayed(const Duration(milliseconds: 40));
    expect(store.state.draft(dim)!.check, DraftCheck.checking);
    expect(store.state.draft(dim)!.source, 'Tilt');
    // the projection for revision 2 arrives (pushed), the draft is re-asked and answered
    store.dispatch(ProjectReceived(lamp(revision: 2), fromRequest: false));
    final s = await store.until((s) => s.draft(dim)?.check == DraftCheck.checked);
    expect(s.draft(dim)!.baseRevision, 2);
    await store.dispose();
  });

  test('a daemon error or transport loss leaves the draft unchecked, never discarded', () async {
    var mode = 'error';
    final daemon = FakeDaemon((m) {
      if (m.hasHandshake()) return okHandshake();
      if (m.hasAnalyzeDefinitionDraft()) {
        if (mode == 'error') return errorResponse('session.no_project', 'no project');
        throw StateError('pipe closed');
      }
      return pb.Response(ack: pb.Ack());
    });
    final store = await connectedStore(daemon);
    store.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'Tilt'));
    var s = await store.until((s) => s.draft(dim)?.check == DraftCheck.unavailable);
    expect(s.draft(dim)!.source, 'Tilt');
    expect(s.draft(dim)!.checkError, 'no project');
    expect(s.editor.lastError, isNull, reason: 'a draft check failure is local, not a banner');

    mode = 'throw';
    store.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'Tilt /'));
    s = await store.until((s) => s.draft(dim)?.check == DraftCheck.unavailable);
    expect(s.draft(dim)!.source, 'Tilt /');
    expect(s.draft(dim)!.checkError, contains('could not be reached'));
    await store.dispose();
  });

  test('the daemon exiting stashes the draft; the source survives', () async {
    final daemon = FakeDaemon((m) {
      if (m.hasHandshake()) return okHandshake();
      return pb.Response(ack: pb.Ack());
    });
    final store = await connectedStore(daemon);
    store.dispatch(const DefinitionDraftChanged(mappingId: dim, source: 'Tilt'));
    store.dispatch(const DaemonExited(9));
    expect(store.state.project, isNull);
    expect(store.state.editor.stashedDrafts['/p']![dim]!.source, 'Tilt');
    await store.dispose();
  });
}
