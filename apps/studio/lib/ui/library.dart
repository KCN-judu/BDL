/// Left sidebar: the project's objects by kind, with "+" per section.
/// Selecting here selects on the canvas (one selection, one state).
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import 'canvas/canvas_geometry.dart' show stateWord;
import 'dialogs.dart';
import 'mac/tokens.dart';

class Library extends StatelessWidget {
  const Library({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final p = state.project;
    final sel = state.editor.selection;
    return Container(
      color: t.sidebar,
      child: p == null
          ? const SizedBox.shrink()
          : ListView(
              padding: const EdgeInsets.symmetric(vertical: 4),
              children: [
                _Section(
                  title: 'Concepts',
                  onAdd: () async {
                    final name = await showNameSheet(context, title: 'New concept', hint: 'Tilt');
                    if (name != null && name.isNotEmpty) {
                      dispatch(CreateConceptRequested(name: name));
                    }
                  },
                ),
                for (final c in p.concepts)
                  _Row(
                    color: t.conceptColor(c.id.toInt()),
                    hollow: !c.hasRepresentation(),
                    title: c.name,
                    trailing: c.hasRepresentation() ? null : 'open',
                    selected: sel is ConceptSelected && sel.id == c.id.toInt(),
                    onTap: () => dispatch(SelectionChanged(ConceptSelected(c.id.toInt()))),
                  ),
                _Section(
                  title: 'Mappings',
                  onAdd: p.concepts.isEmpty
                      ? null
                      : () async {
                          final r = await showNewMappingSheet(context, p.concepts);
                          if (r != null && r.name.isNotEmpty) {
                            dispatch(
                              CreateMappingRequested(
                                name: r.name,
                                inputs: r.inputs,
                                output: r.output,
                              ),
                            );
                          }
                        },
                ),
                for (final m in p.mappings)
                  _Row(
                    color: t.conceptColor(m.signature.output.toInt()),
                    hollow: !m.hasDefinition(),
                    title: m.name,
                    trailing: stateWord(m.state),
                    selected: sel is MappingSelected && sel.id == m.id.toInt(),
                    onTap: () => dispatch(SelectionChanged(MappingSelected(m.id.toInt()))),
                  ),
                const _Section(title: 'Contexts'),
                const _Section(title: 'Outputs'),
                const _Section(title: 'Components'),
              ],
            ),
    );
  }
}

class _Section extends StatelessWidget {
  const _Section({required this.title, this.onAdd});
  final String title;
  final VoidCallback? onAdd;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 10, 4, 2),
      child: Row(
        children: [
          Text(title, style: Theme.of(context).textTheme.titleSmall),
          const Spacer(),
          IconButton(icon: const Icon(Icons.add, size: 14), onPressed: onAdd, tooltip: 'Add'),
        ],
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row({
    required this.color,
    required this.hollow,
    required this.title,
    required this.selected,
    required this.onTap,
    this.trailing,
  });
  final Color color;
  final bool hollow;
  final String title;
  final String? trailing;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 6),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(5),
        child: Container(
          height: MacMetrics.rowHeight,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(
            color: selected ? t.selection : null,
            borderRadius: BorderRadius.circular(5),
          ),
          child: Row(
            children: [
              Container(
                width: 9,
                height: 9,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: hollow ? Colors.transparent : color,
                  border: Border.all(color: color, width: 1.5),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(child: Text(title, overflow: TextOverflow.ellipsis)),
              if (trailing != null)
                Text(trailing!, style: TextStyle(fontSize: 10, color: t.textTertiary)),
            ],
          ),
        ),
      ),
    );
  }
}
