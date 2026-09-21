/// The canvas's wire gesture in the reducer (ADR-0043): a Sem block
/// dropped on a Source is one attach edit of the block's name; dropped on
/// a mapping block with an open position it is the compiler's fill of the
/// first slot (`ComposeAction.fill` on `MappingAnalysis.slots`) after the
/// projection of the committed text is in hand, committed as one replace
/// edit — never a signature edit, never a draft the designer is typing; a
/// rule cannot be wired; a block with no open position takes nothing; the
/// block wired is selected.  Adding a block of a concept is the Source
/// path; a concept made from the canvas brings its block.
library;

import 'package:bdl_studio/app/actions.dart';
import 'package:bdl_studio/app/effects.dart';
import 'package:bdl_studio/app/reducer.dart';
import 'package:bdl_studio/app/state.dart';
import 'package:bdl_studio/protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'package:fixnum/fixnum.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/roles.dart';

const pressedC = 0, litC = 1;
const pressed = 10, lit = 11, litV = 12;

pb.ProjectProjection design({String? litVFormula}) =>
    pb.ProjectProjection(revision: Int64(3), name: 'lamp', rootPath: '/p')
      ..concepts.addAll([
        pb.ConceptView(id: Int64(pressedC), name: 'Pressed'),
        pb.ConceptView(id: Int64(litC), name: 'Lit'),
      ])
      ..mappings.addAll([
        mappingView(
          id: Int64(pressed),
          name: 'pressed',
          signature: pb.Signature(output: Int64(pressedC)),
        ),
        mappingView(
          id: Int64(lit),
          name: 'lit',
          signature: pb.Signature(inputs: [Int64(pressedC)], output: Int64(litC)),
          definition: pb.Definition(formula: 'Pressed'),
        ),
        mappingView(
          id: Int64(litV),
          name: 'litV',
          signature: pb.Signature(output: Int64(litC)),
          definition: litVFormula == null ? null : pb.Definition(formula: litVFormula),
        ),
      ]);

pb.ProjectAnalysis analysis({List<String> litVSlots = const []}) => pb.ProjectAnalysis(
  revision: Int64(3),
  mappings: [
    pb.MappingAnalysis(id: Int64(pressed)),
    pb.MappingAnalysis(id: Int64(lit)),
    pb.MappingAnalysis(id: Int64(litV), slots: litVSlots),
  ],
);

AppState connected(pb.ProjectProjection p, {pb.ProjectAnalysis? a}) => AppState(
  connection: Connected(executable: 'x', handshake: pb.HandshakeResponse()),
  project: p,
  analysis: a,
);

pb.EditOp opOf(Transition t) => t.effects.whereType<ApplyEdit>().single.op;

void main() {
  test('a Sem block dropped on a Source: one attach edit of its name; the block is selected', () {
    final t = reduce(
      connected(design()),
      const WireSemBlockRequested(mappingId: litV, semId: pressed),
    );
    final op = opOf(t);
    expect(op.attachDefinition.id.toInt(), litV);
    expect(op.attachDefinition.definition.formula, 'pressed');
    expect(t.state.editor.selection, const MappingSelected(litV));
    expect(t.state.editor.drafts, isEmpty, reason: 'the compiler\'s text, not a draft');
  });

  test('a rule cannot be wired, and nothing is wired into a rule', () {
    expect(
      reduce(connected(design()), const WireSemBlockRequested(mappingId: litV, semId: lit)).effects,
      isEmpty,
    );
    expect(
      reduce(
        connected(design()),
        const WireSemBlockRequested(mappingId: lit, semId: pressed),
      ).effects,
      isEmpty,
    );
    expect(
      reduce(
        connected(design()),
        const WireSemBlockRequested(mappingId: litV, semId: litV),
      ).effects,
      isEmpty,
    );
  });

  test('a defined block with no open position takes nothing', () {
    final s = connected(design(litVFormula: 'lit(pressed)'), a: analysis());
    expect(
      reduce(s, const WireSemBlockRequested(mappingId: litV, semId: pressed)).effects,
      isEmpty,
    );
  });

  test(
    'an open position: the projection first, then the compiler\'s fill, then one replace edit',
    () {
      final s = connected(
        design(litVFormula: 'lit(?)'),
        a: analysis(litVSlots: ['r.0']),
      );
      // 1. no projection of the committed text yet: it is asked for, the wire waits
      final asked = reduce(s, const WireSemBlockRequested(mappingId: litV, semId: pressed));
      expect(asked.effects.single, isA<GetFormulaProjection>());
      expect(asked.state.editor.pendingWire?.mappingId, litV);
      expect(asked.state.editor.pendingWire?.action.nodeId, 'r.0');
      expect(asked.state.editor.pendingWire?.action.fill, 'pressed');
      final generation = asked.state.editor.composer.projectionGeneration!;
      // 2. the projection arrives: the fill is asked of the compiler
      final filled = reduce(
        asked.state,
        FormulaProjectionReceived(
          generation: generation,
          result: pb.FormulaProjectionResponse(
            revision: Int64(3),
            mappingId: Int64(litV),
            projection: pb.FormulaProjection(source: 'lit(?)', parseOk: true, slots: ['r.0']),
          ),
        ),
      );
      final compose = filled.effects.whereType<ComposeFormula>().single;
      expect(compose.action.nodeId, 'r.0');
      expect(compose.action.fill, 'pressed');
      expect(compose.source, 'lit(?)');
      expect(filled.state.editor.pendingWire, isNull);
      expect(filled.state.editor.composer.commitOnCompose, isTrue);
      // 3. the compiler's text: committed as one replace edit, no draft
      final committed = reduce(
        filled.state,
        ComposeReceived(
          generation: filled.state.editor.composer.composeGeneration!,
          result: pb.ComposeFormulaResponse(
            revision: Int64(3),
            mappingId: Int64(litV),
            source: 'lit(pressed)',
          ),
        ),
      );
      final op = opOf(committed);
      expect(op.replaceDefinition.id.toInt(), litV);
      expect(op.replaceDefinition.definition.formula, 'lit(pressed)');
      expect(committed.state.editor.drafts, isEmpty);
      expect(committed.state.editor.composer.commitOnCompose, isFalse);
      expect(committed.state.editor.composer.pendingCompose, isFalse);
    },
  );

  test('a stale projection answer leaves the wire pending; a moved text drops it', () {
    final s = connected(
      design(litVFormula: 'lit(?)'),
      a: analysis(litVSlots: ['r.0']),
    );
    final asked = reduce(s, const WireSemBlockRequested(mappingId: litV, semId: pressed));
    final stale = reduce(
      asked.state,
      FormulaProjectionReceived(
        generation: 0,
        result: pb.FormulaProjectionResponse(revision: Int64(3), mappingId: Int64(litV)),
      ),
    );
    expect(stale.effects, isEmpty);
    expect(stale.state.editor.pendingWire, isNotNull);
    final moved = reduce(
      asked.state,
      FormulaProjectionReceived(
        generation: asked.state.editor.composer.projectionGeneration!,
        result: pb.FormulaProjectionResponse(
          revision: Int64(3),
          mappingId: Int64(litV),
          projection: pb.FormulaProjection(source: 'something else', parseOk: true),
        ),
      ),
    );
    expect(moved.effects, isEmpty);
    expect(moved.state.editor.pendingWire, isNull, reason: 'given up, not retried blindly');
  });

  test('Add Block: a Source of the concept at the point, named after it, the Source path', () {
    final t = reduce(
      connected(design()),
      const AddBlockRequested(conceptId: litC, position: Offset(300, 200)),
    );
    final create = t.effects.single as CreateSource;
    expect(create.existingConcept, litC);
    expect(create.sourceName, 'lit2', reason: '`lit` is taken');
    expect(t.state.editor.pendingInsert?.position, const Offset(300, 200));
    expect(t.state.editor.pendingInsert?.templateId, PendingInsert.kSourceInsert);
    expect(t.state.editor.pendingInsert?.named, isTrue);
  });
}
