/// The Formula Composer end to end through the real Studio stack —
/// reducer, effect executor, Dart client — against the real `bdld`: the
/// Tilt → Brightness walkthrough of the P10a brief, assembled as
/// structured actions only, saved through the ordinary definition commit.
///
/// Skipped when the binary is absent (run `cargo build`).
@Tags(['daemon', 'filesystem', 'e2e'])
library;

import 'dart:io';

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/composer.dart' show composerInSync;
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

  int mappingId() => store.state.project!.mappings.single.id.toInt();
  int conceptId(String name) =>
      store.state.project!.concepts.firstWhere((c) => c.name == name).id.toInt();
  Future<AppState> settled() => store.until((s) => s.editor.pendingRequests == 0);

  /// The draft's projection for its latest generation.
  Future<pb.FormulaProjection> projected() async {
    final id = mappingId();
    final s = await store.until((s) {
      final d = s.draft(id);
      return d != null && d.check == DraftCheck.checked && d.projection != null;
    });
    return s.draft(id)!.projection!;
  }

  Future<pb.FormulaSlotResponse> slotOf(String node) async {
    store.dispatch(FormulaNodeSelected(mappingId: mappingId(), nodeId: node));
    final s = await store.until((s) => s.editor.composer.slot?.nodeId == node);
    return s.editor.composer.slot!;
  }

  /// A structured action, sent once the projection on screen is current
  /// (the stale-projection policy: the UI offers no action before that).
  Future<String> composed(pb.ComposeAction action) async {
    final id = mappingId();
    await store.until((s) => composerInSync(s, id));
    final before = store.state.draft(id)?.source ?? '';
    store.dispatch(ComposeRequested(mappingId: id, action: action));
    final s = await store.until(
      (s) => !s.editor.composer.pendingCompose && (s.draft(id)?.source ?? '') != before,
    );
    return s.draft(id)!.source;
  }

  Future<void> project() async {
    store = TestStore(
      spawn: DaemonClient.spawn,
      executable: bdld!,
      draftDebounce: const Duration(milliseconds: 10),
    );
    store.dispatch(const AppStarted());
    await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
    expect(store.state.connection, isA<Connected>(), reason: '${store.state.connection}');
    dir = await Directory.systemTemp.createTemp('bdl-studio-composer');
    store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, 'lamp'), name: 'lamp'));
    await store.until((s) => s.project != null);
    store.dispatch(
      CreateConceptRequested(
        name: 'Tilt',
        representation: pb.Representation(quantity: pb.Dim(angle: 1)),
      ),
    );
    await settled();
    store.dispatch(
      CreateConceptRequested(
        name: 'Brightness',
        representation: pb.Representation(quantity: pb.Dim()),
      ),
    );
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

  test('clamp(Tilt / (90 deg), 0, 1) is assembled without typing and saved', () async {
    if (bdld == null) {
      markTestSkipped('bdld not built');
      return;
    }
    await project();
    try {
      final id = mappingId();
      // 1. select Tilt for the empty formula (one slot)
      var src = await composed(pb.ComposeAction(nodeId: 'r', fill: 'Tilt'));
      expect(src, 'Tilt');
      // 2. insert Divide
      src = await composed(
        pb.ComposeAction(
          nodeId: 'r',
          operator: pb.ComposeOperator(op: '/', before: false),
        ),
      );
      expect(src, 'Tilt / ?');
      var proj = await projected();
      expect(proj.slots, ['r.1']);
      // 3. the denominator slot is inferred as an angle
      var slot = await slotOf('r.1');
      expect(slot.expected.description, 'an angle');
      expect(
        slot.explanation,
        'Expected: an angle, because an angle ÷ an angle = a dimensionless quantity.',
      );
      // 5. unit candidates: deg, rad, turn — and nothing else
      expect(slot.units.map((u) => u.symbol).toList(), ['rad', 'deg', 'turn']);
      // 4, 6, 7. a numeric literal in deg, coordinate 90
      src = await composed(pb.ComposeAction(nodeId: 'r.1', fill: '90 deg'));
      expect(src, 'Tilt / 90 deg');
      // 8. the result is dimensionless: the formula checks
      proj = await projected();
      expect(proj.complete, isTrue);
      expect(proj.root.actual.description, 'a dimensionless quantity');
      // 9. wrap in clamp; 10. enter 0 and 1
      src = await composed(
        pb.ComposeAction(
          nodeId: 'r',
          call: pb.ComposeCall(name: 'clamp', arity: 3),
        ),
      );
      expect(src, 'clamp(Tilt / 90 deg, ?, ?)');
      slot = await slotOf('r.1');
      expect(slot.expected.description, 'a dimensionless quantity');
      expect(slot.units, isEmpty, reason: 'a dimensionless slot has no unit to choose');
      src = await composed(pb.ComposeAction(nodeId: 'r.1', fill: '0'));
      src = await composed(pb.ComposeAction(nodeId: 'r.2', fill: '1'));
      expect(src, 'clamp(Tilt / 90 deg, 0, 1)');
      // 11. compatible with Brightness
      proj = await projected();
      expect(proj.complete, isTrue);
      expect(
        store.state.draft(id)!.analysis!.status,
        pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
      );
      // a value-preserving unit switch on the literal
      src = await composed(
        pb.ComposeAction(
          nodeId: 'r.0.1',
          setUnit: pb.ComposeSetUnit(unitId: 'angle.rad', preserveValue: true),
        ),
      );
      expect(src, 'clamp(Tilt / 1.5707963267948966 rad, 0, 1)');
      src = await composed(
        pb.ComposeAction(
          nodeId: 'r.0.1',
          setUnit: pb.ComposeSetUnit(unitId: 'angle.deg', preserveValue: true),
        ),
      );
      expect(src, 'clamp(Tilt / 90 deg, 0, 1)');
      // 12. save through the existing definition commit path
      await projected();
      store.dispatch(CommitDefinitionRequested(id));
      final s = await store.until((s) => s.committedDefinition(id) == 'clamp(Tilt / 90 deg, 0, 1)');
      expect(s.editor.lastOutcome!.kind, pb.EditKind.EDIT_KIND_REFINEMENT);
      // the committed definition projects too, with no draft
      await store.until((s) => s.draft(id) == null);
      store.dispatch(FormulaProjectionRequested(id));
      final c = await store.until((s) => s.editor.composer.projection != null);
      expect(c.editor.composer.projection!.source, 'clamp(Tilt / 90 deg, 0, 1)');
      expect(c.editor.composer.projection!.root.children[0].children[1].coordinate, '90');
    } finally {
      await store.dispose();
      await dir.delete(recursive: true);
    }
  });

  test(
    'a natural formula round-trips through the Composer and its actions keep the form',
    () async {
      if (bdld == null) {
        markTestSkipped('bdld not built');
        return;
      }
      store = TestStore(
        spawn: DaemonClient.spawn,
        executable: bdld,
        draftDebounce: const Duration(milliseconds: 10),
      );
      store.dispatch(const AppStarted());
      await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
      dir = await Directory.systemTemp.createTemp('bdl-studio-composer');
      try {
        store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, 'rig'), name: 'rig'));
        await store.until((s) => s.project != null);
        store.dispatch(
          CreateConceptRequested(
            name: 'Angles',
            representation: pb.Representation(list: pb.Representation(quantity: pb.Dim(angle: 1))),
          ),
        );
        await settled();
        store.dispatch(
          CreateConceptRequested(
            name: 'Safe',
            representation: pb.Representation(boolean: pb.Unit()),
          ),
        );
        await settled();
        store.dispatch(
          CreateMappingRequested(
            name: 'safe',
            inputs: [conceptId('Angles')],
            output: conceptId('Safe'),
          ),
        );
        await settled();
        final id = store.state.project!.mappings.single.id.toInt();
        store.dispatch(SelectionChanged(MappingSelected(id)));
        // typed as text in the natural form: the projection is structural
        const natural = 'all angle in Angles: angle in -45 deg .. 45 deg';
        store.dispatch(DefinitionDraftChanged(mappingId: id, source: natural));
        final s = await store.until((s) => s.draft(id)?.projection?.source == natural);
        final root = s.draft(id)!.projection!.root;
        expect(root.kind, 'binder');
        expect((root.name, root.param), ('all', 'angle'));
        expect(root.paramType.description, 'an angle');
        expect(root.children[1].kind, 'compare');
        expect(root.children[1].children[0].local, isTrue);
        expect(root.children[1].children[1].kind, 'range');
        expect(s.draft(id)!.projection!.complete, isTrue);
        // the text the Composer holds is the text typed: Text → Formula → Text
        expect(s.draft(id)!.source, natural);
        // a structured action on the body keeps the natural form around it
        await store.until((s) => composerInSync(s, id));
        store.dispatch(
          ComposeRequested(
            mappingId: id,
            action: pb.ComposeAction(
              nodeId: 'r.1',
              operator: pb.ComposeOperator(op: '&&', before: false),
            ),
          ),
        );
        final s2 = await store.until((s) => (s.draft(id)?.source ?? '') != natural);
        expect(s2.draft(id)!.source, 'all angle in Angles: angle in -45 deg .. 45 deg && ?');
        // the new slot is a truth value, being the other side of `&&`
        final slot = await store.until((s) => s.editor.composer.slot?.nodeId == 'r.1.1');
        expect(slot.editor.composer.slot!.expected.description, 'true or false');
        // wrap the whole thing again: a fresh, non-capturing local
        await store.until((s) => composerInSync(s, id));
        store.dispatch(
          ComposeRequested(
            mappingId: id,
            action: pb.ComposeAction(
              nodeId: 'r.0',
              binder: pb.ComposeBinder(form: 'filter'),
            ),
          ),
        );
        final s3 = await store.until(
          (s) => (s.draft(id)?.source ?? '').startsWith('all angle in (filter'),
        );
        expect(
          s3.draft(id)!.source,
          'all angle in (filter item in Angles: ?): angle in -45 deg .. 45 deg && ?',
        );
      } finally {
        await store.dispose();
        await dir.delete(recursive: true);
      }
    },
  );

  test(
    'boolean logic and a choice are assembled without typing, and typed text comes back structured',
    () async {
      if (bdld == null) {
        markTestSkipped('bdld not built');
        return;
      }
      store = TestStore(
        spawn: DaemonClient.spawn,
        executable: bdld,
        draftDebounce: const Duration(milliseconds: 10),
      );
      store.dispatch(const AppStarted());
      await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
      dir = await Directory.systemTemp.createTemp('bdl-studio-composer');
      try {
        store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, 'ac'), name: 'ac'));
        await store.until((s) => s.project != null);
        store.dispatch(
          CreateConceptRequested(
            name: 'RoomTemp',
            representation: pb.Representation(quantity: pb.Dim(temperature: 1)),
          ),
        );
        await settled();
        for (final name in ['ButtonHeld', 'SwitchState']) {
          store.dispatch(
            CreateConceptRequested(
              name: name,
              representation: pb.Representation(boolean: pb.Unit()),
            ),
          );
          await settled();
        }
        store.dispatch(
          CreateMappingRequested(
            name: 'AirConditionerCtrl',
            inputs: [conceptId('RoomTemp'), conceptId('ButtonHeld')],
            output: conceptId('SwitchState'),
          ),
        );
        await settled();
        final id = mappingId();
        store.dispatch(SelectionChanged(MappingSelected(id)));
        await store.until((s) => s.analysis != null);
        pb.ComposeAction after(String node, String op) => pb.ComposeAction(
          nodeId: node,
          operator: pb.ComposeOperator(op: op, before: false),
        );
        // `RoomTemp > 299.15 K && ButtonHeld`, from the empty slot
        var src = await composed(pb.ComposeAction(nodeId: 'r', fill: 'RoomTemp'));
        src = await composed(after('r', '>'));
        expect(src, 'RoomTemp > ?');
        var slot = await slotOf('r.1');
        expect(slot.expected.description, 'a temperature');
        expect(slot.units.map((u) => u.symbol), contains('K'));
        expect(slot.booleans, isEmpty);
        src = await composed(pb.ComposeAction(nodeId: 'r.1', fill: '299.15 K'));
        src = await composed(after('r', '&&'));
        expect(src, 'RoomTemp > 299.15 K && ?');
        // the slot is a truth value: the two literals, ButtonHeld first
        slot = await slotOf('r.1');
        expect(slot.booleans, ['true', 'false']);
        expect(slot.references.first.label, 'ButtonHeld');
        expect(slot.references.map((r) => r.label), isNot(contains('RoomTemp')));
        src = await composed(pb.ComposeAction(nodeId: 'r.1', fill: 'ButtonHeld'));
        expect(src, 'RoomTemp > 299.15 K && ButtonHeld');
        var proj = await projected();
        expect(proj.complete, isTrue);
        expect(
          store.state.draft(id)!.analysis!.status,
          pb.MappingStatus.MAPPING_STATUS_CLOCK_CONSISTENT,
        );
        // the same as the condition of a choice, from the choice down
        store.dispatch(DefinitionDraftChanged(mappingId: id, source: ''));
        await store.until((s) => (s.draft(id)?.source ?? 'x') == '');
        src = await composed(pb.ComposeAction(nodeId: 'r', choose: pb.Unit()));
        expect(src, 'if ? then ? else ?');
        src = await composed(pb.ComposeAction(nodeId: 'r.0', fill: 'RoomTemp'));
        src = await composed(after('r.0', '>'));
        src = await composed(pb.ComposeAction(nodeId: 'r.0.1', fill: '299.15 K'));
        src = await composed(after('r.0', '&&'));
        src = await composed(pb.ComposeAction(nodeId: 'r.0.1', fill: 'ButtonHeld'));
        expect(src, 'if RoomTemp > 299.15 K && ButtonHeld then ? else ?');
        slot = await slotOf('r.1');
        expect(slot.expected.description, 'a SwitchState (true or false)');
        expect(slot.booleans, ['true', 'false']);
        src = await composed(pb.ComposeAction(nodeId: 'r.1', fill: 'true'));
        src = await composed(pb.ComposeAction(nodeId: 'r.2', fill: 'false'));
        const choice = 'if RoomTemp > 299.15 K && ButtonHeld then true else false';
        expect(src, choice);
        proj = await projected();
        expect(proj.complete, isTrue);
        expect(proj.root.kind, 'if');
        // `not` in place, on the condition's right side
        src = await composed(after('r.0.1', '!'));
        expect(src, 'if RoomTemp > 299.15 K && !ButtonHeld then true else false');
        // the same text typed in the Text view comes back fully structured
        store.dispatch(DefinitionDraftChanged(mappingId: id, source: choice));
        final s = await store.until((s) => s.draft(id)?.projection?.source == choice);
        final root = s.draft(id)!.projection!.root;
        bool opaque(pb.FormulaNode n) => n.kind == 'opaque' || n.children.any(opaque);
        expect(opaque(root), isFalse, reason: '$root');
        expect(root.kind, 'if');
        expect(root.children.map((c) => c.kind), ['compare', 'bool', 'bool']);
        expect(root.children[0].name, '&&');
        expect(root.children[0].children[0].name, '>');
        expect(root.children[0].expected.description, 'true or false');
        expect(root.children[1].expected.description, 'a SwitchState (true or false)');
        expect(s.draft(id)!.projection!.complete, isTrue);
        expect(s.draft(id)!.source, choice);
      } finally {
        await store.dispose();
        await dir.delete(recursive: true);
      }
    },
  );

  test('a torque slot infers a length and a product of two unknowns is not guessed', () async {
    if (bdld == null) {
      markTestSkipped('bdld not built');
      return;
    }
    store = TestStore(
      spawn: DaemonClient.spawn,
      executable: bdld,
      draftDebounce: const Duration(milliseconds: 10),
    );
    store.dispatch(const AppStarted());
    await store.until((s) => s.connection is Connected || s.connection is ConnectionFailed);
    dir = await Directory.systemTemp.createTemp('bdl-studio-composer');
    try {
      store.dispatch(NewProjectRequested(rootPath: p.join(dir.path, 'arm'), name: 'arm'));
      await store.until((s) => s.project != null);
      store.dispatch(
        CreateConceptRequested(
          name: 'Force',
          representation: pb.Representation(quantity: pb.Dim(mass: 1, length: 1, time: -2)),
        ),
      );
      await settled();
      store.dispatch(
        CreateConceptRequested(
          name: 'Torque',
          representation: pb.Representation(quantity: pb.Dim(mass: 1, length: 2, time: -2)),
        ),
      );
      await settled();
      store.dispatch(
        CreateConceptRequested(
          name: 'Length',
          representation: pb.Representation(quantity: pb.Dim(length: 1)),
        ),
      );
      await settled();
      store.dispatch(
        CreateMappingRequested(name: 'armLength', inputs: const [], output: conceptId('Length')),
      );
      await settled();
      store.dispatch(
        CreateMappingRequested(
          name: 'torqueOf',
          inputs: [conceptId('Force')],
          output: conceptId('Torque'),
        ),
      );
      await settled();
      final id = store.state.project!.mappings.firstWhere((m) => m.name == 'torqueOf').id.toInt();
      store.dispatch(SelectionChanged(MappingSelected(id)));
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: 'Force * ?'));
      await store.until((s) => s.draft(id)?.projection != null);
      store.dispatch(FormulaNodeSelected(mappingId: id, nodeId: 'r.1'));
      var s = await store.until((s) => s.editor.composer.slot?.nodeId == 'r.1');
      final slot = s.editor.composer.slot!;
      expect(slot.expected.description, 'a length');
      expect(slot.units.map((u) => u.symbol).toList(), ['m', 'mm', 'cm', 'km', 'inch', 'ft']);
      expect(slot.references.map((r) => r.label).toList(), ['armLength']);
      store.dispatch(DefinitionDraftChanged(mappingId: id, source: '? * ?'));
      await store.until((s) => s.draft(id)?.projection?.source == '? * ?');
      store.dispatch(FormulaNodeSelected(mappingId: id, nodeId: 'r.0'));
      s = await store.until((s) => s.editor.composer.slot?.nodeId == 'r.0');
      expect(s.editor.composer.slot!.insufficient, isTrue);
      expect(s.editor.composer.slot!.units, isEmpty);
    } finally {
      await store.dispose();
      await dir.delete(recursive: true);
    }
  });
}
