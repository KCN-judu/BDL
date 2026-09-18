/// Fixes offered by the IDE service for the selected object: a ready plan
/// is one button; a plan that needs a choice is a pop-up of the options the
/// service enumerated; a blocked one says why, and stays visible so the
/// designer knows the language cannot say it yet.  Nothing here decides
/// what a fix means — the service does; Studio applies the edits it names.
library;

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
  });
  final SemanticActionsState? actions;
  final void Function(AppAction) dispatch;

  /// Context actions (refactors) usually duplicate a button the inspector
  /// already has; by default only the fixes for diagnostics are listed.
  final bool quickFixesOnly;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final a = actions;
    if (a == null) return const SizedBox.shrink();
    final items = a.actions.where((x) => !quickFixesOnly || x.kind == 'quick_fix').toList();
    if (items.isEmpty) return const SizedBox.shrink();
    final small = TextStyle(fontSize: 11, color: t.textSecondary);
    return InspectorSection(
      title: context.l10n.fixes,
      children: [
        for (final x in items)
          Padding(
            padding: const EdgeInsets.only(bottom: MacMetrics.gap),
            child: Column(
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
                  _ => Text(x.title, style: TextStyle(fontSize: 12, color: t.textTertiary)),
                },
                if (x.applicability == pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED)
                  Text('Not possible yet: ${x.reason}', style: small)
                else if (x.explanation.isNotEmpty)
                  Text(x.explanation, style: small),
                if (x.invalidation.isNotEmpty &&
                    x.applicability != pb.ActionApplicability.ACTION_APPLICABILITY_BLOCKED)
                  Text(
                    x.invalidation[0].toUpperCase() + x.invalidation.substring(1),
                    style: TextStyle(fontSize: 11, color: t.textTertiary),
                  ),
              ],
            ),
          ),
      ],
    );
  }
}
