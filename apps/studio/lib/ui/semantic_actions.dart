/// Fixes offered by the IDE service for the selected object: a ready plan
/// is one button; a plan that needs a choice is a pop-up of the options the
/// service enumerated; a blocked one says why, and stays visible so the
/// designer knows the language cannot say it yet.  Nothing here decides
/// what a fix means — the service does; Studio applies the edits it names.
library;

import 'package:fixnum/fixnum.dart';
import 'package:flutter/material.dart';

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'mac/widgets.dart';

class FixList extends StatelessWidget {
  const FixList({
    super.key,
    required this.actions,
    required this.dispatch,
    this.quickFixesOnly = true,
    this.excludeKinds = const {},
  });
  final SemanticActionsState? actions;
  final void Function(AppAction) dispatch;

  /// Context actions (refactors) usually duplicate a button the inspector
  /// already has; by default only the fixes for diagnostics are listed.
  final bool quickFixesOnly;

  /// Kinds of action shown beside their finding instead (one home each).
  final Set<String> excludeKinds;

  @override
  Widget build(BuildContext context) {
    final a = actions;
    if (a == null) return const SizedBox.shrink();
    final items = a.actions
        .where(
          (x) =>
              (!quickFixesOnly || x.kind == 'quick_fix') &&
              !excludeKinds.any((k) => x.id == k || x.id.startsWith('$k:')),
        )
        .toList();
    if (items.isEmpty) return const SizedBox.shrink();
    return InspectorSection(
      title: context.l10n.fixes,
      children: [
        for (final x in items)
          Padding(
            padding: const EdgeInsets.only(bottom: MacMetrics.gap),
            child: FixItem(action: x, dispatch: dispatch),
          ),
      ],
    );
  }
}

/// One fix as the service described it: the control that applies it (a
/// button, or a pop-up of the choices), then its explanation — or, when
/// blocked, the reason — and what applying it would reopen.  The same
/// drawing wherever a fix is offered: the inspector's Fixes section, the
/// finding it addresses, the Simulate page's readiness area and probe.
class FixItem extends StatelessWidget {
  const FixItem({super.key, required this.action, required this.dispatch});
  final pb.SemanticActionView action;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final x = action;
    final small = TextStyle(fontSize: MacType.secondary, color: t.textSecondary);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: MacMetrics.gapTight,
      children: [
        switch (x.applicability) {
          pb.ActionApplicability.ACTION_APPLICABILITY_READY => Align(
            alignment: Alignment.centerLeft,
            child: MacButton(
              label: x.title,
              onPressed: () => dispatch(SemanticActionApplied(actionId: x.id)),
            ),
          ),
          pb.ActionApplicability.ACTION_APPLICABILITY_NEEDS_CHOICE => MacDropdown<int>(
            value: null,
            hint: x.title,
            items: [for (var i = 0; i < x.options.length; i++) i],
            labelOf: (i) => x.options[i].label,
            onChanged: (i) => dispatch(SemanticActionApplied(actionId: x.id, option: i)),
          ),
          _ => Text(
            x.title,
            style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
          ),
        },
        if (x.applicability == pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED)
          Text(context.l10n.notPossibleYet(x.reason), style: small)
        else if (x.explanation.isNotEmpty)
          Text(x.explanation, style: small),
        if (x.invalidation.isNotEmpty &&
            x.applicability != pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED)
          Text(
            x.invalidation[0].toUpperCase() + x.invalidation.substring(1),
            style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
          ),
      ],
    );
  }
}

/// A fix offered where its finding is shown, for an object that may not
/// be the selection: once the service's list for that object is on show,
/// the fix as the service described it ([FixItem]); until then, one
/// button bearing the fix's title that selects the object, asks, and
/// applies the fix when it arrives ready — or leaves a choice or a reason
/// on show.  One click when the tool can do it alone; never a guess.
class OfferedFix extends StatelessWidget {
  const OfferedFix({
    super.key,
    required this.state,
    required this.selection,
    required this.actionKind,
    required this.title,
    required this.dispatch,
  });
  final AppState state;

  /// The object the fix belongs to, and the kind of action (an id is
  /// `<kind>:<entity>…`; among one object's actions the kind is enough).
  final Selection selection;
  final String actionKind;

  /// The fix's title, as the service words it, for the button before the
  /// list arrives.
  final String title;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final a = state.editor.actions;
    final entity = switch (selection) {
      MappingSelected(:final id) => pb.EntityRef(mappingId: Int64(id)),
      ConceptSelected(:final id) => pb.EntityRef(conceptId: Int64(id)),
      OutputSelected(:final id) => pb.EntityRef(outputId: Int64(id)),
      _ => null,
    };
    if (entity != null && a != null && a.isFor(entity, state.revision) && !a.pending) {
      final x = a.ofKind(actionKind);
      if (x != null) return FixItem(action: x, dispatch: dispatch);
    }
    return Align(
      alignment: Alignment.centerLeft,
      child: MacButton(
        label: title,
        onPressed: () =>
            dispatch(SemanticActionChosen(selection: selection, actionKind: actionKind)),
      ),
    );
  }
}
