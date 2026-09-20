/// The completion pop-up, for every BDL text field: the IDE service's
/// candidates in its order, one row each — label in monospace, the kind
/// word, the resulting type — with the selected row marked.  Studio
/// picks and accepts; it composes no candidate and re-sorts nothing
/// (`docs/architecture/studio-compiler-integration.md`).  Keys are the
/// field's (`definition_editor.dart`, `pages/code_pane.dart`): ↑/↓ move,
/// Return/Tab accept, Esc closes, ⌃Space opens.
library;

import 'package:flutter/material.dart';

import '../../app/state.dart';
import '../../l10n/l10n.dart';
import '../mac/tokens.dart';

class CompletionPopup extends StatelessWidget {
  const CompletionPopup({super.key, required this.completion, required this.onPick});
  final CompletionState completion;
  final void Function(int index) onPick;

  static const double rowHeight = 22;
  static const int visibleRows = 6;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final items = completion.items;
    return Container(
      key: const ValueKey('completion-popup'),
      margin: const EdgeInsets.only(top: MacMetrics.gapTight),
      constraints: const BoxConstraints(maxHeight: rowHeight * visibleRows + 2),
      decoration: BoxDecoration(
        color: t.content,
        borderRadius: BorderRadius.circular(5),
        border: Border.all(color: t.hairline),
      ),
      child: items.isEmpty
          ? Padding(
              padding: const EdgeInsets.all(6),
              child: Text(
                context.l10n.looking,
                style: TextStyle(fontSize: 11, color: t.textTertiary),
              ),
            )
          : ListView.builder(
              shrinkWrap: true,
              padding: EdgeInsets.zero,
              itemExtent: rowHeight,
              itemCount: items.length,
              itemBuilder: (context, i) {
                final item = items[i];
                final selected = i == completion.selected;
                return MouseRegion(
                  cursor: SystemMouseCursors.basic,
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTap: () => onPick(i),
                    child: Container(
                      color: selected ? t.selection : null,
                      padding: const EdgeInsets.symmetric(horizontal: 6),
                      child: Row(
                        spacing: MacMetrics.gap,
                        children: [
                          SizedBox(
                            width: 8,
                            child: Text(
                              selected ? '▸' : '',
                              style: TextStyle(fontSize: 10, color: t.textSecondary),
                            ),
                          ),
                          Expanded(
                            child: Text(
                              item.label,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(
                                fontSize: 12,
                                fontFamily: 'Menlo',
                                color: t.textPrimary,
                              ),
                            ),
                          ),
                          Text(item.kind, style: TextStyle(fontSize: 10, color: t.textTertiary)),
                          if (item.resultingType.isNotEmpty)
                            SizedBox(
                              width: 72,
                              child: Text(
                                item.resultingType,
                                textAlign: TextAlign.right,
                                overflow: TextOverflow.ellipsis,
                                style: TextStyle(fontSize: 10, color: t.textSecondary),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
    );
  }
}

/// The everyday meaning of the name under the pointer, as the service
/// states it: title, what kind of value, its status, a few details.  No
