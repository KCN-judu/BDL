/// A mapping's saved formula, expanded on its canvas node
/// (docs/architecture/studio-ui.md §2, *Expanded formula*): the committed
/// definition's projection rendered read-only (`formula_render.dart`) in
/// the node's formula region, with the first finding about it beneath and
/// one deliberate way into editing — *Edit formula*, which selects the
/// mapping and gives the inspector's editor the keyboard.  A reading
/// feature: a casual click on the picture selects the node and never
/// rewrites anything; the mapping stays expanded while the designer
/// navigates the design.
///
/// The picture is the compiler's projection of the *committed* text
/// (never a draft), fetched per revision (`FormulaPreview`); reference
/// edges into the node's formula line come from the same analysis's
/// `references`, never from what is drawn here.  A long formula is clipped
/// at the region's maximum height with a fade — the rest is read in the
/// inspector — so a node never swallows the graph (§29).
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../l10n/l10n.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'formula_render.dart';
import 'mac/tokens.dart';
import 'mac/interactive.dart';

class ExpandedFormula extends StatefulWidget {
  const ExpandedFormula({
    super.key,
    required this.mappingId,
    required this.rect,
    required this.sceneWidth,
    required this.zoom,
    required this.preview,
    required this.analysis,
    required this.concepts,
    required this.selected,
    required this.dispatch,
  });

  final int mappingId;

  /// The formula region on screen (scene rect scaled and panned).
  final Rect rect;

  /// The region's width in scene units: the picture is laid out at
  /// canvas scale and then drawn at the zoom, like the node itself.
  final double sceneWidth;
  final double zoom;
  final FormulaPreview? preview;
  final pb.MappingAnalysis? analysis;
  final Map<int, pb.ConceptView> concepts;
  final bool selected;
  final void Function(AppAction) dispatch;

  @override
  State<ExpandedFormula> createState() => _ExpandedFormulaState();
}

class _ExpandedFormulaState extends State<ExpandedFormula> {
  final FormulaGeometry _geometry = FormulaGeometry();
  final GlobalKey _contentKey = GlobalKey(debugLabel: 'expanded-formula-content');
  double? _reported;

  /// The picture's own height, once laid out: the node takes it (capped).
  void _measure() {
    final box = _contentKey.currentContext?.findRenderObject() as RenderBox?;
    if (box == null || !box.hasSize) return;
    final h = box.size.height + 8;
    if (_reported == h) return;
    _reported = h;
    widget.dispatch(FormulaExpansionMeasured(mappingId: widget.mappingId, height: h));
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    final p = widget.preview?.projection;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _measure();
    });
    // the first finding about the formula, in product language: an error
    // first, else something still open
    final findings = widget.analysis?.diagnostics ?? const <pb.Diagnostic>[];
    final finding =
        findings
            .where((d) => d.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR)
            .firstOrNull ??
        findings.firstOrNull;
    final Widget body;
    if (p == null) {
      // asked for and on its way, or refused (a definition by reference)
      body = Text(
        widget.preview != null ? l10n.formulaPreviewLoading : l10n.formulaPreviewUnavailable,
        key: ValueKey('expanded-note-${widget.mappingId}'),
        style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
      );
    } else if (!p.hasRoot()) {
      body = Text(
        l10n.formulaPreviewUnavailable,
        style: TextStyle(fontSize: MacType.secondary, color: t.textTertiary),
      );
    } else {
      body = FormulaRender(
        projection: p,
        concepts: widget.concepts,
        geometry: _geometry,
        dense: true,
      );
    }
    final content = Column(
      key: _contentKey,
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        body,
        if (finding != null)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: 4,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 3),
                  child: Container(
                    width: 6,
                    height: 6,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: finding.severity == pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR
                          ? t.error
                          : t.open,
                    ),
                  ),
                ),
                Expanded(
                  child: Text(
                    finding.message,
                    key: ValueKey('expanded-finding-${widget.mappingId}'),
                    style: TextStyle(fontSize: MacType.caption, color: t.textSecondary),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: MacLink(
            key: ValueKey('expanded-edit-${widget.mappingId}'),
            label: l10n.editFormula,
            onTap: () => widget.dispatch(EditDefinitionRequested(widget.mappingId)),
          ),
        ),
      ],
    );
    // laid out at scene scale, drawn at the zoom; clipped to the region,
    // fading at the bottom when the picture is taller than the node
    return Positioned.fromRect(
      rect: widget.rect,
      child: Semantics(
        container: true,
        label: l10n.showFormula,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => widget.dispatch(SelectionChanged(MappingSelected(widget.mappingId))),
          child: ClipRect(
            child: Transform.scale(
              scale: widget.zoom,
              alignment: Alignment.topLeft,
              child: OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: widget.sceneWidth,
                maxWidth: widget.sceneWidth,
                minHeight: 0,
                maxHeight: double.infinity,
                child: Padding(padding: const EdgeInsets.fromLTRB(10, 4, 10, 4), child: content),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
