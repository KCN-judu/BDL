/// Left sidebar: the project's objects by kind, with "+" per section.
/// Selecting here selects on the canvas (one selection, one state).
library;

import 'package:flutter/material.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart';
import 'concept_library_panel.dart';
import 'dialogs.dart';
import 'mac/widgets.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';
import 'units.dart';

/// Left sidebar, two tabs: **Project** — the project's objects by kind —
/// and **Library** — the concept libraries to insert from
/// (`concept_library_panel.dart`).
class Library extends StatelessWidget {
  const Library({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Container(
      color: t.sidebar,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(MacMetrics.gap, MacMetrics.gap, MacMetrics.gap, 0),
            child: MacSegmented<SidebarTab>(
              value: state.editor.sidebar,
              options: const {SidebarTab.project: 'Project', SidebarTab.library: 'Library'},
              onChanged: (tab) => dispatch(SidebarTabSelected(tab)),
            ),
          ),
          Expanded(
            child: switch (state.editor.sidebar) {
              SidebarTab.project => _ProjectObjects(state: state, dispatch: dispatch),
              SidebarTab.library => ConceptLibraryPanel(state: state, dispatch: dispatch),
            },
          ),
        ],
      ),
    );
  }
}

class _ProjectObjects extends StatelessWidget {
  const _ProjectObjects({required this.state, required this.dispatch});
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
                    final r = await showNewConceptSheet(
                      context,
                      presets: unitPresetsFrom(state.library?.quantities ?? const []),
                    );
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
                _Section(
                  title: 'Timing domains',
                  onAdd: () async {
                    final name = await showNewClockSheet(context, p.clocks);
                    if (name != null) dispatch(CreateClockDomainRequested(name));
                  },
                ),
                for (final c in p.clocks)
                  _ClockRow(
                    clock: c,
                    inUse:
                        p.mappings.any((m) => m.hasClockId() && m.clockId == c.id) ||
                        p.outputs.any((o) => o.hasClockId() && o.clockId == c.id),
                    dispatch: dispatch,
                  ),
                _Section(
                  title: 'Outputs',
                  onAdd: p.concepts.isEmpty
                      ? null
                      : () async {
                          final r = await showNewOutputSheet(context, p.concepts, p.clocks);
                          if (r != null) {
                            dispatch(
                              CreateOutputRequested(
                                name: r.name,
                                accepts: r.accepts,
                                clockId: r.clockId,
                                required: r.required,
                              ),
                            );
                          }
                        },
                ),
                for (final o in p.outputs)
                  _Row(
                    glyph: OutputGlyph(
                      state: state.outputAnalysis(o.id.toInt())?.state,
                      open: !o.hasClockId(),
                    ),
                    title: o.name,
                    selected: sel is OutputSelected && sel.id == o.id.toInt(),
                    onTap: () => dispatch(SelectionChanged(OutputSelected(o.id.toInt()))),
                  ),
                const _Section(title: 'Contexts'),
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

/// A timing domain: a name, renamed in place, deleted only while nothing
/// updates in it.  A domain is when things happen, never how often.
class _ClockRow extends StatelessWidget {
  const _ClockRow({required this.clock, required this.inUse, required this.dispatch});
  final pb.ClockView clock;
  final bool inUse;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final id = clock.id.toInt();
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 0, 6, 2),
      child: Row(
        spacing: MacMetrics.gapTight,
        children: [
          Text('↻', style: TextStyle(fontSize: 12, color: t.textSecondary)),
          Expanded(
            child: CommitTextField(
              value: clock.name,
              onCommit: (v) => dispatch(RenameClockDomainRequested(id: id, name: v)),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.close, size: 12),
            onPressed: inUse ? null : () => dispatch(DeleteClockDomainRequested(id)),
            tooltip: inUse ? 'Still in use' : 'Delete ${clock.name}',
            constraints: const BoxConstraints.tightFor(width: 20, height: 20),
            padding: EdgeInsets.zero,
          ),
        ],
      ),
    );
  }
}
