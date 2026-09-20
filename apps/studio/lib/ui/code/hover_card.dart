/// The hover card, for every BDL text field.
library;

import 'package:flutter/material.dart';

import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../mac/tokens.dart';

/// Core term here — that is Explain's job.
class HoverCard extends StatelessWidget {
  const HoverCard({super.key, required this.card});
  final pb.DraftHoverResponse card;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      key: const ValueKey('hover-card'),
      margin: const EdgeInsets.only(top: MacMetrics.gapTight),
      padding: const EdgeInsets.fromLTRB(8, 6, 8, 8),
      decoration: BoxDecoration(
        color: t.content,
        borderRadius: BorderRadius.circular(5),
        border: Border.all(color: t.hairline),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gapTight,
        children: [
          Text(
            card.title,
            style: TextStyle(
              fontSize: MacType.body,
              fontWeight: FontWeight.w600,
              color: t.textPrimary,
            ),
          ),
          if (card.signature.isNotEmpty)
            Text(
              card.signature,
              style: TextStyle(
                fontSize: MacType.secondary,
                fontFamily: 'Menlo',
                color: t.textSecondary,
              ),
            ),
          if (card.representation.isNotEmpty)
            Text(
              card.representation,
              style: TextStyle(fontSize: MacType.secondary, color: t.textPrimary),
            ),
          Text(
            card.status,
            style: TextStyle(
              fontSize: MacType.secondary,
              color: card.open ? t.open : t.textSecondary,
            ),
          ),
          for (final d in card.details)
            Row(
              spacing: MacMetrics.gap,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(
                  width: 80,
                  child: Text(
                    d.label,
                    style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
                  ),
                ),
                Expanded(
                  child: Text(
                    d.value,
                    style: TextStyle(fontSize: MacType.secondary, color: t.textPrimary),
                  ),
                ),
              ],
            ),
          if (card.explanation.isNotEmpty)
            Text(
              card.explanation,
              style: TextStyle(fontSize: MacType.secondary, color: t.textSecondary),
            ),
        ],
      ),
    );
  }
}
