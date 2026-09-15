/// Left sidebar: the project's objects by kind, with "+" per section.
/// Selecting here selects on the canvas (one selection, one state).
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart';
import 'dialogs.dart';
import 'mac/interactive.dart';
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
                    final r = await showNewConceptSheet(context);
                    if (r != null) {
                      dispatch(
                        CreateConceptRequested(
                          name: r.name,
                          description: r.description,
                          representation: r.representation,
                        ),
                      );
                    }
                  },
                ),
                for (final c in p.concepts)
                  _Row(
                    glyph: SocketGlyph.of(c, t),
                    title: c.name,
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
                    glyph: MappingGlyph(
                      declared: !m.hasDefinition(),
                      wrong:
                          state.mappingAnalysis(m.id.toInt())?.status ==
                          pb.MappingStatus.MAPPING_STATUS_INVALID,
                    ),
                    title: m.name,
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

/// One object in the library: its glyph (the same mark as on the canvas)
/// and its name.  State is in the glyph — hollow, dashed, red mark — not in
/// a trailing word.
class _Row extends StatelessWidget {
  const _Row({
    required this.glyph,
    required this.title,
    required this.selected,
    required this.onTap,
  });
  final Widget glyph;
  final String title;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 6),
      child: MacInteractive(
        onTap: onTap,
        selected: selected,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: SizedBox(
          height: MacMetrics.rowHeight,
          child: Row(
            spacing: MacMetrics.gap,
            children: [
              SizedBox(width: 14, child: Center(child: glyph)),
              Expanded(child: Text(title, overflow: TextOverflow.ellipsis)),
            ],
          ),
        ),
      ),
    );
  }
}
