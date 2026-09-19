/// Small macOS-flavoured building blocks: panel headers, form rows,
/// commit-on-blur text fields, segmented controls, hairline dropdowns.
/// Presentation only.
library;

import 'package:flutter/material.dart';

import '../../l10n/diagnostics.dart';
import '../../l10n/l10n.dart';
import '../../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import '../source_span.dart';
import 'controls.dart';
import 'interactive.dart';
import 'theme.dart';
import 'tokens.dart';

/// Finder-style section header: 11 pt semibold, secondary colour.
class PanelHeader extends StatelessWidget {
  const PanelHeader(this.title, {super.key, this.trailing});
  final String title;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      height: 28,
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: t.hairline)),
      ),
      child: Row(
        children: [
          Text(title, style: Theme.of(context).textTheme.titleSmall),
          const Spacer(),
          ?trailing,
        ],
      ),
    );
  }
}

class InspectorSection extends StatelessWidget {
  const InspectorSection({super.key, required this.title, required this.children, this.trailing});
  final String title;
  final List<Widget> children;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      padding: const EdgeInsets.fromLTRB(12, 10, 12, 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: t.hairline)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Text(title, style: Theme.of(context).textTheme.titleSmall),
              const Spacer(),
              ?trailing,
            ],
          ),
          const SizedBox(height: 8),
          ...children,
        ],
      ),
    );
  }
}

/// Right-aligned label, left-aligned field — the macOS form idiom.
class FormRow extends StatelessWidget {
  const FormRow({super.key, required this.label, required this.child});
  final String label;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: MacMetrics.formLabelWidth,
            child: Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                label,
                textAlign: TextAlign.right,
                style: TextStyle(fontSize: 11, color: t.textSecondary),
              ),
            ),
          ),
          const SizedBox(width: 8),
          Expanded(child: child),
        ],
      ),
    );
  }
}

/// A text field that commits on Enter or focus loss, only when changed.
/// With [commitLabel] it also shows a button (for "Attach").
class CommitTextField extends StatefulWidget {
  const CommitTextField({
    super.key,
    required this.value,
    required this.onCommit,
    this.hint,
    this.maxLines = 1,
    this.monospace = false,
    this.commitLabel,
  });
  final String value;
  final void Function(String) onCommit;
  final String? hint;
  final int maxLines;
  final bool monospace;
  final String? commitLabel;

  @override
  State<CommitTextField> createState() => _CommitTextFieldState();
}

class _CommitTextFieldState extends State<CommitTextField> {
  late final TextEditingController _c = TextEditingController(text: widget.value);
  final FocusNode _focus = FocusNode();

  @override
  void initState() {
    super.initState();
    _focus.addListener(() {
      if (!_focus.hasFocus) _commit();
    });
  }

  @override
  void didUpdateWidget(CommitTextField old) {
    super.didUpdateWidget(old);
    if (old.value != widget.value && !_focus.hasFocus) _c.text = widget.value;
  }

  void _commit() {
    if (_c.text != widget.value) {
      widget.onCommit(_c.text);
      if (widget.commitLabel != null) _c.clear();
    }
  }

  @override
  void dispose() {
    _c.dispose();
    _focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final field = MacTextField(
      controller: _c,
      focusNode: _focus,
      maxLines: widget.maxLines,
      monospace: widget.monospace,
      hint: widget.hint,
      onSubmitted: (_) => _commit(),
    );
    if (widget.commitLabel == null) return field;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        field,
        const SizedBox(height: 6),
        MacButton.primary(label: widget.commitLabel!, onPressed: _commit),
      ],
    );
  }
}

/// NSSegmentedControl look-alike.
///
/// [undecided]: no segment is chosen yet and the choice is still to be
/// made — the track is empty with a dashed outline (the same "not
/// decided" mark as a declared relationship's outline and a formula
/// slot), never a segment shown as if chosen.  One click on a segment
/// decides.
class MacSegmented<T> extends StatelessWidget {
  const MacSegmented({
    super.key,
    required this.value,
    required this.options,
    required this.onChanged,
    this.undecided = false,
  });
  final T value;
  final Map<T, String> options;
  final void Function(T) onChanged;
  final bool undecided;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      height: MacMetrics.controlHeight,
      decoration: undecided
          ? null
          : BoxDecoration(
              color: t.isDark ? const Color(0x22FFFFFF) : const Color(0x11000000),
              borderRadius: BorderRadius.circular(6),
            ),
      foregroundDecoration: undecided ? DashedOutline(color: t.textTertiary, radius: 6) : null,
      padding: const EdgeInsets.all(2),
      child: Row(
        children: [
          for (final e in options.entries)
            Expanded(
              // Segments share the width in proportion to their labels (a
              // long label never wraps), and each is a real control:
              // focusable, Space/Return selects, the standard hover and
              // focus treatments.
              flex: e.value.length + 6,
              child: MacInteractive(
                radius: 4,
                onTap: () {
                  if (e.key != value) onChanged(e.key);
                },
                child: AnimatedContainer(
                  duration: MacStates.duration,
                  curve: MacStates.curve,
                  decoration: BoxDecoration(
                    color: e.key == value ? t.control : Colors.transparent,
                    borderRadius: BorderRadius.circular(4),
                    boxShadow: e.key == value
                        ? const [
                            BoxShadow(
                              color: Color(0x22000000),
                              blurRadius: 2,
                              offset: Offset(0, 1),
                            ),
                          ]
                        : null,
                  ),
                  alignment: Alignment.center,
                  child: Text(
                    e.value,
                    style: TextStyle(
                      fontSize: 11,
                      fontWeight: e.key == value ? FontWeight.w600 : FontWeight.w400,
                      color: t.textPrimary,
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// A 1 px dashed rounded outline, as a decoration: "not decided yet" on a
/// control, the way the canvas dashes a declared relationship.
class DashedOutline extends Decoration {
  const DashedOutline({required this.color, this.radius = 6});
  final Color color;
  final double radius;

  @override
  BoxPainter createBoxPainter([VoidCallback? onChanged]) => _DashedOutlinePainter(this);
}

class _DashedOutlinePainter extends BoxPainter {
  _DashedOutlinePainter(this.decoration);
  final DashedOutline decoration;

  @override
  void paint(Canvas canvas, Offset offset, ImageConfiguration configuration) {
    final size = configuration.size ?? Size.zero;
    final rect = (offset & size).deflate(0.5);
    final path = Path()
      ..addRRect(RRect.fromRectAndRadius(rect, Radius.circular(decoration.radius)));
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..color = decoration.color;
    const dash = 4.0;
    const gap = 3.0;
    for (final metric in path.computeMetrics()) {
      var at = 0.0;
      while (at < metric.length) {
        final end = (at + dash).clamp(0.0, metric.length);
        canvas.drawPath(metric.extractPath(at, end), paint);
        at += dash + gap;
      }
    }
  }
}

/// NSPopUpButton look-alike.
class MacDropdown<T> extends StatelessWidget {
  const MacDropdown({
    super.key,
    required this.value,
    required this.items,
    required this.onChanged,
    this.labelOf,
    this.detailOf,
    this.leadingOf,
    this.hint,
    this.compact = false,
  });
  final T? value;
  final List<T> items;
  final void Function(T) onChanged;
  final String Function(T)? labelOf;

  /// A mark before the label (a concept's socket glyph), in the menu and on
  /// the closed control.
  final Widget Function(T)? leadingOf;

  /// A secondary fact shown in its own right-hand column (a unit, a count).
  final String Function(T)? detailOf;
  final String? hint;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    String label(T v) => labelOf?.call(v) ?? v.toString();
    return PopupMenuButton<T>(
      tooltip: '',
      position: PopupMenuPosition.under,
      color: t.content,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(6),
        side: BorderSide(color: t.hairline),
      ),
      itemBuilder: (_) => [
        for (final i in items)
          PopupMenuItem<T>(
            value: i,
            height: 24,
            child: Row(
              spacing: MacMetrics.gutter,
              children: [
                Expanded(
                  child: Row(
                    spacing: MacMetrics.gap,
                    children: [
                      ?leadingOf?.call(i),
                      Expanded(
                        child: Text(
                          label(i),
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(fontSize: 13, color: t.textPrimary),
                        ),
                      ),
                    ],
                  ),
                ),
                ?detailOf == null
                    ? null
                    : Text(
                        detailOf!(i),
                        style: TextStyle(
                          fontSize: 12,
                          color: t.textSecondary,
                          fontFeatures: kTabularFigures,
                        ),
                      ),
              ],
            ),
          ),
      ],
      onSelected: onChanged,
      child: _Hoverable(
        builder: (hover) => AnimatedContainer(
          duration: MacStates.duration,
          curve: MacStates.curve,
          height: MacMetrics.controlHeight,
          padding: EdgeInsets.only(left: compact ? 6 : 8, right: 4),
          decoration: BoxDecoration(
            color: hover ? t.controlHover : t.control,
            borderRadius: BorderRadius.circular(5),
            border: Border.all(color: t.hairline),
          ),
          child: Row(
            mainAxisSize: compact ? MainAxisSize.min : MainAxisSize.max,
            children: [
              if (!compact) ...[
                if (value != null && leadingOf != null)
                  Padding(
                    padding: const EdgeInsets.only(right: MacMetrics.gap),
                    child: leadingOf!(value as T),
                  ),
                Expanded(
                  child: Text(
                    value == null ? (hint ?? '') : label(value as T),
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: 13,
                      color: value == null ? t.textTertiary : t.textPrimary,
                    ),
                  ),
                ),
                if (detailOf != null && value != null)
                  Padding(
                    padding: const EdgeInsets.only(right: MacMetrics.gap),
                    child: Text(
                      detailOf!(value as T),
                      style: TextStyle(
                        fontSize: 12,
                        color: t.textSecondary,
                        fontFeatures: kTabularFigures,
                      ),
                    ),
                  ),
              ] else
                Text(hint ?? '', style: TextStyle(fontSize: 13, color: t.textSecondary)),
              Icon(Icons.unfold_more, size: 14, color: t.textSecondary),
            ],
          ),
        ),
      ),
    );
  }
}

/// Hover tracking for controls that draw their own background.
class _Hoverable extends StatefulWidget {
  const _Hoverable({required this.builder});
  final Widget Function(bool hover) builder;

  @override
  State<_Hoverable> createState() => _HoverableState();
}

class _HoverableState extends State<_Hoverable> {
  bool _hover = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hover = true),
      onExit: (_) => setState(() => _hover = false),
      child: widget.builder(_hover),
    );
  }
}

/// A small table of facts: fixed column widths, one gutter, text columns
/// left-aligned, number columns right-aligned in tabular figures.  The way
/// to show ≥ 2 rows that share fields (docs/architecture/studio-ui.md §3a).
class MacTable extends StatelessWidget {
  const MacTable({
    super.key,
    required this.columns,
    required this.rows,
    this.rowHeight = MacMetrics.rowHeight,
  });

  final List<MacColumn> columns;
  final List<List<Widget>> rows;
  final double rowHeight;

  @override
  Widget build(BuildContext context) {
    return Table(
      columnWidths: {
        for (var i = 0; i < columns.length; i++)
          i: columns[i].width == null
              ? const FlexColumnWidth()
              : FixedColumnWidth(columns[i].width!),
      },
      defaultVerticalAlignment: TableCellVerticalAlignment.middle,
      children: [
        for (final r in rows)
          TableRow(
            children: [
              for (var i = 0; i < r.length; i++)
                SizedBox(
                  height: rowHeight,
                  child: Padding(
                    padding: EdgeInsets.only(right: i == r.length - 1 ? 0 : MacMetrics.gutter),
                    child: Align(alignment: columns[i].alignment, child: r[i]),
                  ),
                ),
            ],
          ),
      ],
    );
  }
}

class MacColumn {
  const MacColumn({this.width, this.numeric = false});
  final double? width;
  final bool numeric;
  Alignment get alignment => numeric ? Alignment.centerRight : Alignment.centerLeft;
}

/// Tabular figures for numbers that sit in columns.
const List<FontFeature> kTabularFigures = [FontFeature.tabularFigures()];

/// One compiler diagnostic: severity dot, message, the offending part of the
/// formula, explanation and fixes.  Kernel detail stays in a tooltip.
class DiagnosticCard extends StatelessWidget {
  const DiagnosticCard({
    super.key,
    required this.diagnostic,
    required this.source,
    this.showMessage = true,
  });
  final pb.Diagnostic diagnostic;
  final String source;

  /// False when the message is already the line above (the editor's status
  /// line); the card then carries only the excerpt, explanation and fixes.
  final bool showMessage;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final d = diagnostic;
    final color = switch (d.severity) {
      pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_ERROR => t.error,
      pb.DiagnosticSeverity.DIAGNOSTIC_SEVERITY_WARNING => t.open,
      _ => t.accent,
    };
    // Spans are byte ranges into the formula; the excerpt must not split a
    // character (source_span.dart).
    String? excerpt;
    if (d.hasSpan() && source.isNotEmpty) {
      final e = excerptOf(source, d.span.start, d.span.end);
      if (e.isNotEmpty) excerpt = e;
    }
    final card = Padding(
      padding: const EdgeInsets.only(bottom: MacMetrics.gap),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: MacMetrics.gap,
        children: [
          // A continuation of the line above keeps its indent, not its dot.
          if (showMessage)
            Padding(
              padding: const EdgeInsets.only(top: 5),
              child: Icon(Icons.circle, size: 7, color: color),
            )
          else
            const SizedBox(width: 7),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: MacMetrics.gapTight,
              children: [
                if (showMessage)
                  Text(
                    localizedMessage(context.l10n, d.code, d.message),
                    style: TextStyle(fontSize: 12, color: t.textPrimary),
                  ),
                if (excerpt != null)
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: color.withValues(alpha: 0.10),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      excerpt,
                      style: TextStyle(fontSize: 11, fontFamily: 'Menlo', color: t.textPrimary),
                    ),
                  ),
                if (d.explanation.isNotEmpty)
                  Text(d.explanation, style: TextStyle(fontSize: 11, color: t.textSecondary)),
                for (final f in d.fixes)
                  Text(f, style: TextStyle(fontSize: 11, color: t.textSecondary)),
              ],
            ),
          ),
        ],
      ),
    );
    return d.technical.isEmpty ? card : Tooltip(message: '${d.code}\n${d.technical}', child: card);
  }
}

/// A collapsed section with a disclosure triangle (macOS inspectors).  Used
/// for the Explain layer: closed by default, nothing above it depends on it.
class MacDisclosure extends StatefulWidget {
  const MacDisclosure({
    super.key,
    required this.title,
    required this.children,
    this.initiallyOpen = false,
  });
  final String title;
  final List<Widget> children;
  final bool initiallyOpen;

  @override
  State<MacDisclosure> createState() => _MacDisclosureState();
}

class _MacDisclosureState extends State<MacDisclosure> {
  late bool _open = widget.initiallyOpen;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: t.hairline)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          MacInteractive(
            onTap: () => setState(() => _open = !_open),
            radius: 0,
            padding: const EdgeInsets.fromLTRB(8, 6, 12, 6),
            child: Row(
              spacing: MacMetrics.gapTight,
              children: [
                AnimatedRotation(
                  turns: _open ? 0.25 : 0,
                  duration: MacStates.duration,
                  curve: MacStates.curve,
                  child: Icon(Icons.arrow_right, size: 16, color: t.textSecondary),
                ),
                Text(widget.title, style: Theme.of(context).textTheme.titleSmall),
              ],
            ),
          ),
          if (_open)
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 0, 12, 12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                spacing: MacMetrics.gapTight,
                children: widget.children,
              ),
            ),
        ],
      ),
    );
  }
}

/// One line of the Explain layer: a formal fact in kernel notation.
class ExplainLine extends StatelessWidget {
  const ExplainLine(this.text, {super.key});
  final String text;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return SelectableText(
      text,
      style: TextStyle(fontSize: 11, fontFamily: 'Menlo', color: t.textSecondary, height: 1.35),
    );
  }
}

/// A named destructive action ("Delete Tilt"), never "OK".
class DestructiveButton extends StatelessWidget {
  const DestructiveButton({
    super.key,
    required this.label,
    required this.onPressed,
    this.enabled = true,
    this.tooltip,
  });
  final String label;
  final VoidCallback onPressed;
  final bool enabled;
  final String? tooltip;

  @override
  Widget build(BuildContext context) {
    return MacButton.destructive(
      label: label,
      onPressed: enabled ? onPressed : null,
      tooltip: tooltip,
    );
  }
}

/// Toolbar icon button with a tooltip; disabled state fades.
class ToolbarButton extends StatelessWidget {
  const ToolbarButton({super.key, required this.icon, required this.tooltip, this.onPressed});
  final IconData icon;
  final String tooltip;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    return IconButton(icon: Icon(icon, size: 18), tooltip: tooltip, onPressed: onPressed);
  }
}
