/// The Library tab of the sidebar: the concept libraries the daemon serves,
/// for browsing and discovery.  Fast insertion is the canvas's right-click
/// menu; this panel is where a designer looks for "the one for lux".
///
/// A row is a *template*, not a concept: its glyph is grey because the
/// identity (hue) is the compiler's to allocate on insertion, filled when
/// the template chooses a representation and hollow when it leaves it to be
/// decided — the same rule as the New Concept sheet's preview.  The right
/// column says what the value is measured as, in the contract's words
/// (a unit symbol, *on–off*, *count*, *decide later*).
///
/// Insertion from here — a drag onto the canvas, a double-click, or Return
/// on a focused row — dispatches exactly what the right-click menu does
/// ([InsertConceptTemplateRequested]); there is one creation path.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'mac/controls.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';

/// Product wording for a library category id.  One place, used by the
/// panel's section headers and the canvas menu alike.
String categoryLabel(String category) => switch (category) {
  'environment' => 'Environment',
  'human' => 'Human interaction',
  'motion' => 'Geometry & motion',
  'mechanical' => 'Mechanical',
  'electrical' => 'Electrical & system',
  'visual' => 'Visual & display',
  'actuation' => 'Actuation',
  'audio' => 'Audio',
  _ => category.isEmpty ? category : category[0].toUpperCase() + category.substring(1),
};

/// What a template's value is measured as, in the words the sheet and the
/// inspector use.
String representationWord(pb.ConceptTemplateView t) {
  if (!t.hasRepresentation()) return 'decide later';
  return switch (t.representation.whichKind()) {
    pb.Representation_Kind.quantity => t.unit.isEmpty ? 'no unit' : t.unit,
    pb.Representation_Kind.boolean => 'on–off',
    pb.Representation_Kind.count => 'count',
    pb.Representation_Kind.list => 'collection',
    pb.Representation_Kind.pair => 'grouped value',
    pb.Representation_Kind.optional => 'optional value',
    pb.Representation_Kind.notSet => 'decide later',
  };
}

/// The templates matching [query], in library order.  Matches the display
/// name, default name, keywords, category and unit, case-insensitively —
/// authoring convenience, never resolution.
List<pb.ConceptTemplateView> searchTemplates(Iterable<pb.ConceptTemplateView> all, String query) {
  final q = query.trim().toLowerCase();
  if (q.isEmpty) return all.toList();
  bool hit(String s) => s.toLowerCase().contains(q);
  return [
    for (final t in all)
      if (hit(t.displayName) ||
          hit(t.defaultName) ||
          hit(t.category) ||
          hit(categoryLabel(t.category)) ||
          hit(t.unit) ||
          t.keywords.any(hit))
        t,
  ];
}

/// The drag payload from a library row: the template id.  Dropped on the
/// canvas, it becomes one [InsertConceptTemplateRequested] at the drop
/// point.
class ConceptTemplateDrag {
  const ConceptTemplateDrag(this.templateId);
  final String templateId;
}

class ConceptLibraryPanel extends StatefulWidget {
  const ConceptLibraryPanel({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  State<ConceptLibraryPanel> createState() => _ConceptLibraryPanelState();
}

class _ConceptLibraryPanelState extends State<ConceptLibraryPanel> {
  late final TextEditingController _search = TextEditingController(
    text: widget.state.editor.librarySearch,
  );

  @override
  void didUpdateWidget(covariant ConceptLibraryPanel old) {
    super.didUpdateWidget(old);
    final q = widget.state.editor.librarySearch;
    if (q != _search.text) _search.text = q;
  }

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final s = widget.state;
    final library = s.library;
    final canInsert = s.project != null && s.editor.pendingInsert == null;
    final query = s.editor.librarySearch;
    final all = s.templates.toList();
    final shown = searchTemplates(all, query);
    final categories = <String>[];
    for (final x in shown) {
      if (!categories.contains(x.category)) categories.add(x.category);
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(MacMetrics.gap, MacMetrics.gap, MacMetrics.gap, 0),
          child: MacTextField(
            controller: _search,
            hint: 'Search concepts',
            onChanged: (q) => widget.dispatch(LibrarySearchChanged(q)),
          ),
        ),
        Expanded(
          child: library == null
              ? _Note(
                  s.connection is Connected
                      ? 'Loading the concept library…'
                      : 'The concept library arrives with the compiler service.',
                )
              : shown.isEmpty
              ? _Note('No concept matches “${query.trim()}”.')
              : ListView(
                  padding: const EdgeInsets.symmetric(vertical: MacMetrics.gapTight),
                  children: [
                    for (final c in categories) ...[
                      _CategoryHeader(categoryLabel(c)),
                      for (final tpl in shown)
                        if (tpl.category == c)
                          _TemplateRow(
                            template: tpl,
                            enabled: canInsert,
                            onInsert: () => widget.dispatch(InsertConceptTemplateRequested(tpl.id)),
                          ),
                    ],
                    if (s.project == null) const _Note('Open a project to add concepts from here.'),
                  ],
                ),
        ),
        if (library != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(12, MacMetrics.gapTight, 12, MacMetrics.gap),
            child: Text(
              [for (final l in library.libraries) '${l.name} ${l.version}'].join(' · '),
              style: TextStyle(fontSize: 10, color: t.textTertiary),
              overflow: TextOverflow.ellipsis,
            ),
          ),
      ],
    );
  }
}

class _CategoryHeader extends StatelessWidget {
  const _CategoryHeader(this.title);
  final String title;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(12, 10, 12, 2),
    child: Text(title, style: Theme.of(context).textTheme.titleSmall),
  );
}

class _Note extends StatelessWidget {
  const _Note(this.text);
  final String text;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 12, 12, 8),
      child: Text(text, style: TextStyle(color: t.textTertiary)),
    );
  }
}

class _TemplateRow extends StatelessWidget {
  const _TemplateRow({required this.template, required this.enabled, required this.onInsert});
  final pb.ConceptTemplateView template;
  final bool enabled;
  final VoidCallback onInsert;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final row = SizedBox(
      height: MacMetrics.rowHeight,
      child: Row(
        children: [
          TemplateGlyph(template: template),
          const SizedBox(width: MacMetrics.gap),
          Expanded(child: Text(template.displayName, overflow: TextOverflow.ellipsis)),
          Text(
            representationWord(template),
            style: TextStyle(
              fontSize: 10,
              color: t.textTertiary,
              fontFeatures: const [FontFeature.tabularFigures()],
            ),
          ),
        ],
      ),
    );
    final interactive = Tooltip(
      message: template.description,
      waitDuration: const Duration(milliseconds: 600),
      child: Shortcuts(
        shortcuts: const {
          SingleActivator(LogicalKeyboardKey.enter): ActivateIntent(),
          SingleActivator(LogicalKeyboardKey.numpadEnter): ActivateIntent(),
        },
        child: Actions(
          actions: {
            ActivateIntent: CallbackAction<ActivateIntent>(
              onInvoke: (_) {
                if (enabled) onInsert();
                return null;
              },
            ),
          },
          child: MacInteractive(
            onTap: () {},
            onDoubleTap: enabled ? onInsert : null,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: row,
          ),
        ),
      ),
    );
    if (!enabled) {
      return Padding(padding: const EdgeInsets.symmetric(horizontal: 6), child: interactive);
    }
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 6),
      child: Draggable<ConceptTemplateDrag>(
        data: ConceptTemplateDrag(template.id),
        dragAnchorStrategy: pointerDragAnchorStrategy,
        feedback: TemplateDragFeedback(template: template),
        child: interactive,
      ),
    );
  }
}

/// The template's socket-to-be: grey (identity not allocated yet), filled
/// when a representation is chosen, hollow when it is left to be decided.
class TemplateGlyph extends StatelessWidget {
  const TemplateGlyph({super.key, required this.template, this.size = 9});
  final pb.ConceptTemplateView template;
  final double size;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final grey = t.textSecondary;
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: template.hasRepresentation() ? grey : Colors.transparent,
        border: Border.all(color: grey, width: 1.5),
      ),
    );
  }
}

/// What travels under the pointer during a drag: the node's header as it
/// will look, with the grey socket.
class TemplateDragFeedback extends StatelessWidget {
  const TemplateDragFeedback({super.key, required this.template});
  final pb.ConceptTemplateView template;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    return Material(
      color: Colors.transparent,
      child: Container(
        width: 168,
        height: 26,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: BoxDecoration(
          color: t.isDark ? const Color(0xFF3A4556) : const Color(0xFFDCE3EE),
          borderRadius: BorderRadius.circular(7),
          border: Border.all(color: t.hairline),
        ),
        child: Row(
          children: [
            TemplateGlyph(template: template),
            const SizedBox(width: MacMetrics.gap),
            Expanded(
              child: Text(
                template.defaultName,
                style: TextStyle(
                  fontSize: 12.5,
                  fontWeight: FontWeight.w600,
                  color: t.textPrimary,
                  decoration: TextDecoration.none,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
