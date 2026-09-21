/// The canvas's wire gestures (ADR-0044): a Sem block dropped on a mapping
/// block is a **text edit of the definition** (ADR-0028) — never a
/// signature edit.  A Source gets the dropped block as its definition
/// (`= sem`); a mapping block with an open position gets the block's name
/// where the compiler says the position is (`ComposeAction.fill` on the
/// slot node `MappingAnalysis.slots` names), then the draft is committed
/// as one edit.  A read edge's *Disconnect* is the same path the other
/// way: `ComposeAction.unreference` turns every occurrence of the name
/// into a slot.  Nothing here parses a formula: the slot ids are the
/// analysis's, the edited text is the compiler's answer.
library;

import 'package:fixnum/fixnum.dart';

import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'composer.dart' show composeRequested, composerInSync, formulaProjectionRequested;
import 'drafts.dart' show commitText;
import 'reducer.dart' show Transition;
import 'state.dart';

/// The open positions of a mapping block's committed definition, as the
/// analysis of this revision reports them (empty without one).
List<String> analysisSlots(AppState s, int mappingId) {
  for (final m in s.contextAnalysis?.mappings ?? const <pb.MappingAnalysis>[]) {
    if (m.id.toInt() == mappingId) return m.slots;
  }
  return const [];
}

Transition wireSemBlockRequested(AppState s, int mappingId, int semId, int? slot) {
  final m = s.mapping(mappingId);
  final sem = s.mapping(semId);
  if (m == null || sem == null || mappingId == semId) return Transition(s);
  // only a Sem block is a value a definition can name; a rule is applied
  if (sem.signature.inputs.isNotEmpty || m.signature.inputs.isNotEmpty) return Transition(s);
  if (s.draft(mappingId) != null) return Transition(s);
  // the block wired is the one the inspector shows next
  final selected = s.copyWith(
    editor: s.editor.copyWith(selection: MappingSelected(mappingId), clearPendingWire: true),
  );
  if (!m.hasDefinition()) return commitText(selected, mappingId, sem.name);
  final slots = analysisSlots(s, mappingId);
  if (slots.isEmpty) return Transition(s);
  final node = slots[(slot ?? 0).clamp(0, slots.length - 1)];
  return composeAndCommit(selected, mappingId, pb.ComposeAction(nodeId: node, fill: sem.name));
}

/// A read edge taken away (ADR-0044): every reference of [mappingId]'s
/// definition to the Sem block [semId] becomes a slot — the compiler's
/// text edit, committed as one edit.
Transition unreferenceRequested(AppState s, int mappingId, int semId) {
  final m = s.mapping(mappingId);
  if (m == null || !m.hasDefinition() || s.draft(mappingId) != null) return Transition(s);
  final selected = s.copyWith(
    editor: s.editor.copyWith(selection: MappingSelected(mappingId), clearPendingWire: true),
  );
  return composeAndCommit(selected, mappingId, pb.ComposeAction(unreference: Int64(semId)));
}

/// Apply [action] to [mappingId]'s committed definition and commit the
/// compiler's answer: through the composer when its projection of the
/// committed text is current, else after fetching that projection
/// (`pendingWire`).
Transition composeAndCommit(AppState s, int mappingId, pb.ComposeAction action) {
  if (composerInSync(s, mappingId)) {
    final t = composeRequested(s, mappingId, action);
    if (t.effects.isEmpty) return t;
    return Transition(
      t.state.copyWith(
        editor: t.state.editor.copyWith(
          composer: t.state.editor.composer.copyWith(commitOnCompose: true),
        ),
      ),
      t.effects,
    );
  }
  final t = formulaProjectionRequested(s, mappingId);
  if (t.effects.isEmpty) return t;
  return Transition(
    t.state.copyWith(
      editor: t.state.editor.copyWith(
        pendingWire: PendingWire(mappingId: mappingId, action: action),
      ),
    ),
    t.effects,
  );
}

/// The projection a wire waited for arrived: fill now, or give up when the
/// text moved meanwhile.
Transition wireAfterProjection(AppState s, int mappingId) {
  final w = s.editor.pendingWire;
  if (w == null || w.mappingId != mappingId) return Transition(s);
  final cleared = s.copyWith(editor: s.editor.copyWith(clearPendingWire: true));
  if (!composerInSync(cleared, mappingId)) return Transition(cleared);
  return composeAndCommit(cleared, mappingId, w.action);
}
