/// Small macOS-flavoured building blocks: panel headers, form rows,
/// commit-on-blur text fields, segmented controls, hairline dropdowns.
/// Presentation only.
library;

import 'package:flutter/material.dart';

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
            width: 78,
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
    final field = TextField(
      controller: _c,
      focusNode: _focus,
      maxLines: widget.maxLines,
      style: TextStyle(fontSize: 13, fontFamily: widget.monospace ? 'Menlo' : null),
      decoration: InputDecoration(hintText: widget.hint),
      textInputAction: widget.maxLines == 1 ? TextInputAction.done : TextInputAction.newline,
      onSubmitted: (_) => _commit(),
    );
    if (widget.commitLabel == null) return field;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.end,
      children: [
        field,
        const SizedBox(height: 6),
        FilledButton(onPressed: _commit, child: Text(widget.commitLabel!)),
      ],
    );
  }
}

/// NSSegmentedControl look-alike.
class MacSegmented<T> extends StatelessWidget {
  const MacSegmented({
    super.key,
    required this.value,
    required this.options,
    required this.onChanged,
  });
  final T value;
  final Map<T, String> options;
  final void Function(T) onChanged;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      height: MacMetrics.controlHeight,
      decoration: BoxDecoration(
        color: t.isDark ? const Color(0x22FFFFFF) : const Color(0x11000000),
        borderRadius: BorderRadius.circular(6),
      ),
      padding: const EdgeInsets.all(2),
      child: Row(
        children: [
          for (final e in options.entries)
            Expanded(
              child: GestureDetector(
                onTap: () => onChanged(e.key),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 150),
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

/// NSPopUpButton look-alike.
class MacDropdown<T> extends StatelessWidget {
  const MacDropdown({
    super.key,
    required this.value,
    required this.items,
    required this.onChanged,
    this.labelOf,
    this.hint,
    this.compact = false,
  });
  final T? value;
  final List<T> items;
  final void Function(T) onChanged;
  final String Function(T)? labelOf;
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
            child: Text(label(i), style: TextStyle(fontSize: 13, color: t.textPrimary)),
          ),
      ],
      onSelected: onChanged,
      child: Container(
        height: MacMetrics.controlHeight,
        padding: EdgeInsets.only(left: compact ? 6 : 8, right: 4),
        decoration: BoxDecoration(
          color: t.control,
          borderRadius: BorderRadius.circular(5),
          border: Border.all(color: t.hairline),
        ),
        child: Row(
          mainAxisSize: compact ? MainAxisSize.min : MainAxisSize.max,
          children: [
            if (!compact)
              Expanded(
                child: Text(
                  value == null ? (hint ?? '') : label(value as T),
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: 13,
                    color: value == null ? t.textTertiary : t.textPrimary,
                  ),
                ),
              )
            else
              Text(hint ?? '', style: TextStyle(fontSize: 13, color: t.textSecondary)),
            Icon(Icons.unfold_more, size: 14, color: t.textSecondary),
          ],
        ),
      ),
    );
  }
}

class ConceptChip extends StatelessWidget {
  const ConceptChip({super.key, required this.label, required this.color, this.onRemove});
  final String label;
  final Color color;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      height: MacMetrics.controlHeight,
      padding: const EdgeInsets.only(left: 6, right: 2),
      decoration: BoxDecoration(
        color: t.control,
        borderRadius: BorderRadius.circular(5),
        border: Border.all(color: t.hairline),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.circle, size: 8, color: color),
          const SizedBox(width: 5),
          Text(label, style: TextStyle(fontSize: 12, color: t.textPrimary)),
          if (onRemove != null)
            IconButton(
              icon: const Icon(Icons.close, size: 12),
              onPressed: onRemove,
              constraints: const BoxConstraints.tightFor(width: 18, height: 18),
              padding: EdgeInsets.zero,
              tooltip: 'Remove',
            ),
        ],
      ),
    );
  }
}

class StatePill extends StatelessWidget {
  const StatePill({super.key, required this.word, required this.settled});
  final String word;
  final bool settled;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final color = settled ? t.settled : t.open;
    return Align(
      alignment: Alignment.centerLeft,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          color: color.withValues(alpha: 0.12),
          borderRadius: BorderRadius.circular(10),
        ),
        child: Text(
          word,
          style: TextStyle(fontSize: 11, fontWeight: FontWeight.w500, color: color),
        ),
      ),
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
    final t = MacTokens.of(context);
    final button = OutlinedButton(
      onPressed: enabled ? onPressed : null,
      style: OutlinedButton.styleFrom(foregroundColor: t.error),
      child: Text(label),
    );
    return tooltip == null ? button : Tooltip(message: tooltip!, child: button);
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
