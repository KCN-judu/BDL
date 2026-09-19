/// Timing domains, physical outputs, drives and semantic actions through
/// the real Studio stack against the real `bdld`: authored as EditOps,
/// judged by the output pass, fixed through bdl-ide's actions, persisted.
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/daemon/daemon_client.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
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

void main() {
  final bdld = _findBdld();

  late TestStore store;
  late Directory dir;
  late String root;

  Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);
  Future<AppState> analysed() =>
      store.until((s) => s.analysis != null && s.analysis!.revision == s.project!.revision);
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  int mappingId(String name) =>
      store.state.project!.mappings.firstWhere((m) => m.name == name).id.toInt();

  Future<void> project() async {
    store = TestStore(spawn: DaemonClient.spawn, executable: bdld!);
    store.dispatch(const AppStarted());
    await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
    expect(store.state.connection, isA<Connected>());
    dir = await Directory.systemTemp.createTemp('bdl-studio-design-e2e');
    root = p.join(dir.path, 'rover');
    store.dispatch(NewProjectRequested(rootPath: root, name: 'rover'));
    await store.until((s) => s.project != null);
    store.dispatch(
      CreateConceptRequested(
        name: 'Speed',
        representation: pb.Representation(quantity: pb.Dim()),
      ),
    );
    await settled();
    for (final name in ['cruise', 'boost']) {
      store.dispatch(
        CreateMappingRequested(name: name, inputs: const [], output: conceptId('Speed')),
      );
      await settled();
      store.dispatch(DefinitionDraftChanged(mappingId: mappingId(name), source: '0.5'));
      await store.until((s) => s.draft(mappingId(name))?.check == DraftCheck.checked);
      store.dispatch(CommitDefinitionRequested(mappingId(name)));
      await store.until((s) => s.committedDefinition(mappingId(name)) == '0.5');
    }
    await analysed();
  }

  Future<void> teardown() async {
    await store.dispose();
    await dir.delete(recursive: true);
  }

  test(
    'domain → output → drive → complete; a second driver is a local conflict with a fix',
    () async {
      await project();
      try {
        store.dispatch(const CreateClockDomainRequested('main'));
        var s = await settled();
        final main_ = s.project!.clocks.single.id.toInt();
        expect(s.project!.clocks.single.name, 'main');

        // a required output in that domain: two steps, one gesture
        store.dispatch(
          CreateOutputRequested(
            name: 'motor',
            accepts: conceptId('Speed'),
            clockId: main_,
            required: true,
          ),
        );
        s = await store.until(
          (s) => s.project!.outputs.length == 1 && s.project!.outputs.single.required,
        );
        final motor = s.project!.outputs.single.id.toInt();
        s = await analysed();
        expect(s.outputAnalysis(motor)!.state, pb.OutputState.OUTPUT_STATE_UNDRIVEN);
        expect(s.analysis!.outputComplete, isFalse);
        expect(
          s.analysis!.diagnostics.any(
            (d) =>
                d.code == 'output.missing_driver' &&
                d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_INFO,
          ),
          isTrue,
          reason: 'incomplete, not wrong',
        );

        // the service offers a fix for the undriven sink: connect a driver (a choice)
        store.dispatch(SelectionChanged(OutputSelected(motor)));
        s = await store.until((s) => s.editor.actions?.pending == false);
        final connect = s.editor.actions!.actions.firstWhere(
          (a) => a.addresses.contains('output.missing_driver'),
        );
        expect(connect.applicability, pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE);
        final cruiseOption = connect.options.indexWhere((o) => o.label.contains('cruise'));
        expect(cruiseOption, isNonNegative);
        // cruise must update in the sink's domain to be a well-formed driver
        store.dispatch(SetMappingClockRequested(mappingId: mappingId('cruise'), clockId: main_));
        await settled();
        s = await store.until((s) => s.editor.actions?.pending == false);
        final connect2 = s.editor.actions!.actions.firstWhere(
          (a) => a.addresses.contains('output.missing_driver'),
        );
        store.dispatch(
          SemanticActionApplied(
            actionId: connect2.id,
            option: connect2.options.indexWhere((o) => o.label.contains('cruise')),
          ),
        );
        s = await store.until((s) => s.mapping(mappingId('cruise'))!.hasDrivesOutputId());
        s = await analysed();
        expect(s.outputAnalysis(motor)!.state, pb.OutputState.OUTPUT_STATE_DRIVEN);
        expect(s.outputAnalysis(motor)!.driver.toInt(), mappingId('cruise'));
        expect(s.analysis!.outputComplete, isTrue);

        // a second driver: the compiler says contested; nothing arbitrates
        store.dispatch(SetMappingClockRequested(mappingId: mappingId('boost'), clockId: main_));
        await settled();
        store.dispatch(SetMappingDriveRequested(mappingId: mappingId('boost'), outputId: motor));
        await settled();
        s = await analysed();
        expect(s.outputAnalysis(motor)!.state, pb.OutputState.OUTPUT_STATE_CONFLICT);
        expect(
          s.outputAnalysis(motor)!.claimants.map((c) => c.toInt()),
          containsAll([mappingId('cruise'), mappingId('boost')]),
        );
        expect(s.analysis!.outputComplete, isFalse);
        expect(
          s.mappingAnalysis(mappingId('boost'))!.status,
          pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
          reason: 'a contested drive edge does not move the mapping down the ladder',
        );
        // the fix: detach one of them — a ready action, the designer's pick
        s = await store.until(
          (s) => s.editor.actions?.pending == false && s.editor.actions!.revision == s.revision,
        );
        final detachBoost = s.editor.actions!.actions.firstWhere(
          (a) =>
              a.applicability == pb.ActionApplicability.ACTION_APPLICABILITY_READY &&
              a.edits.any(
                (e) => e.hasSetMappingDrive() && e.setMappingDrive.id.toInt() == mappingId('boost'),
              ),
        );
        expect(detachBoost.addresses, contains('output.multiple_drivers'));
        store.dispatch(SemanticActionApplied(actionId: detachBoost.id));
        s = await store.until((s) => !s.mapping(mappingId('boost'))!.hasDrivesOutputId());
        s = await analysed();
        expect(s.outputAnalysis(motor)!.state, pb.OutputState.OUTPUT_STATE_DRIVEN);

        // a domain in use cannot be deleted; the refusal is a banner, the design untouched
        store.dispatch(DeleteClockDomainRequested(main_));
        s = await store.until((s) => s.editor.lastError != null);
        expect(s.editor.lastError!.code, 'edit.clock_in_use');
        expect(s.project!.clocks, hasLength(1));

        // persistence: save, close, reopen keeps domains, outputs, drives
        store.dispatch(const SaveRequested());
        await store.until((s) => !s.project!.dirty && s.editor.pendingRequests == 0);
        store.dispatch(const CloseProjectRequested());
        await store.until((s) => s.project == null);
        store.dispatch(OpenProjectRequested(root));
        s = await store.until((s) => s.project != null && s.editor.pendingRequests == 0);
        expect(s.project!.clocks.single.name, 'main');
        expect(s.project!.outputs.single.name, 'motor');
        expect(s.project!.outputs.single.required, isTrue);
        expect(s.mapping(mappingId('cruise'))!.drivesOutputId.toInt(), motor);
        s = await analysed();
        expect(s.analysis!.outputComplete, isTrue);
      } finally {
        await teardown();
      }
    },
    skip: bdld == null ? 'bdld binary not built (run `cargo build`)' : false,
  );
}
