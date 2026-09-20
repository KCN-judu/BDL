/// The Library tab of the sidebar: the Standard Library the daemon serves —
/// reusable authoring fragments built from ordinary BDL structures — in two
/// sections, **Concepts** and **Sources**, for browsing and discovery.
/// Fast insertion is the canvas's right-click menu; this panel is where a
/// designer looks for "the one for lux" or "a temperature sensor".
///
/// A row is an *item*, not a project object: a Concept item's glyph is grey
/// because the identity (hue) is the compiler's to allocate on insertion,
/// filled when the item chooses a representation and hollow when it leaves
/// it to be decided.  A Source item is a **preset** for the Source sheet:
/// its row says it creates an input for a concept the designer chooses —
/// existing, or new — and names the value form it suggests; it shows no
/// signature, because the concept is not decided here and the item owns
/// no identity.  Names and descriptions are localized by item id
/// ([libraryItemStrings]); what gets created — identifiers, source text —
/// never changes with the locale.
///
/// Insertion from here — a drag onto the canvas, a double-click, or Return
/// on a focused row — dispatches exactly what the right-click menu does:
/// [InsertLibraryItemRequested] for a Concept item (one creation path, one
/// transaction) and [NewSourceRequested] for a Source item (the sheet,
/// prefilled by the preset).
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../l10n/l10n.dart';
import '../l10n/library_strings.dart';
import '../app/actions.dart';
import '../app/state.dart';
import '../protocol/gen/bdl/v1/bdl.pb.dart' as pb;
import 'canvas/concept_glyphs.dart' show MappingGlyph;
import 'mac/controls.dart';
import 'mac/interactive.dart';
import 'mac/tokens.dart';

/// Product wording for a library group id.  One place, used by the
/// panel's section headers and the canvas menu alike.
String categoryLabel(AppLocalizations l10n, String category) => switch (category) {
  'environment' => l10n.environment,
  'human' => l10n.humanInteraction,
  'motion' => l10n.geometryMotion,
  'mechanical' => l10n.mechanical,
  'electrical' => l10n.electricalSystem,
  'visual' => l10n.visualDisplay,
  'actuation' => l10n.actuation,
  'audio' => l10n.audio,
  'external' => l10n.libraryGroupExternal,
  _ => category.isEmpty ? category : category[0].toUpperCase() + category.substring(1),
};

/// The section an item belongs to, in the panel's words: Concepts, Sources.
String categoryTitle(AppLocalizations l10n, String category) => switch (category) {
  'source' => l10n.categorySources,
  _ => l10n.concepts,
};

/// The localized name of an item: the catalog's for a standard item, the
/// daemon's canonical English otherwise.
String itemName(AppLocalizations l10n, pb.LibraryItemView item) =>
    libraryItemStrings(l10n, item.id)?.name ?? item.displayName;

String itemDescription(AppLocalizations l10n, pb.LibraryItemView item) =>
    libraryItemStrings(l10n, item.id)?.description ?? item.description;

/// What a template's value is measured as, in the words the sheet and the
/// canvas use: the unit symbol, *on–off*, *count*, *decide later*.
String representationWord(AppLocalizations l10n, pb.ConceptTemplateView t) {
  if (!t.hasRepresentation()) return l10n.decideLaterLower;
  return switch (t.representation.whichKind()) {
    pb.Representation_Kind.quantity => t.unit.isEmpty ? l10n.noUnit : t.unit,
    pb.Representation_Kind.boolean => l10n.formOnOffShort,
    pb.Representation_Kind.count => l10n.formCountShort,
    pb.Representation_Kind.list => l10n.formCollectionShort,
    pb.Representation_Kind.pair => l10n.groupedValue,
    pb.Representation_Kind.optional => l10n.optionalValue,
    pb.Representation_Kind.notSet => l10n.decideLaterLower,
  };
}

/// The right-hand word of a row: the concept's value form for a Concept
/// item; for a Source item the value form its preset suggests for a new
/// concept — never a signature, since the concept is chosen on the sheet.
String itemWord(AppLocalizations l10n, pb.LibraryItemView item) {
  if (item.hasConcept()) return representationWord(l10n, item.concept);
  if (item.hasPreset()) {
    final p = item.preset;
    return representationWord(
      l10n,
      pb.ConceptTemplateView(
        representation: p.hasRepresentation() ? p.representation : null,
        unit: p.unit,
      ),
    );
  }
  return l10n.chooseConcept;
}

/// The hover of a Source item: what the preset does — an input for a
/// concept the designer chooses, existing or new — and the value form it
/// suggests (`Temperature`, *on / off*), in the sheet's words.  Never a
/// signature over a concept nobody has chosen.
String sourceItemHover(AppLocalizations l10n, pb.LibraryItemView item) {
  if (!item.hasPreset()) return l10n.inputForAConcept;
  final p = item.preset;
  final kind = p.typeName.isNotEmpty
      ? p.typeName
      : switch (p.hasRepresentation() ? p.representation.whichKind() : null) {
          pb.Representation_Kind.boolean => l10n.onOff,
          pb.Representation_Kind.count => l10n.count,
          _ => '',
        };
  return kind.isEmpty ? l10n.inputForAConcept : l10n.inputForConceptOfKind(kind);
}

/// The preview of what a Concept item creates, one line per object:
/// *value: Temperature (Temperature)*.  Identifiers and types are the
/// daemon's, never localized; a concept whose value form is left open says
/// so in the sheet's words (*decide later*), never a presumed scalar.  A
/// Source item previews nothing here: the objects depend on the choice
/// made on the sheet, which previews them exactly.
List<String> itemPreview(AppLocalizations l10n, pb.LibraryItemView item) => [
  if (item.category != 'source')
    for (final o in item.creates)
      if (o.kind == 'concept')
        l10n.libraryValueOf(
          '${o.name} (${o.typeName.isNotEmpty ? o.typeName : l10n.decideLaterLower})',
        )
      else ...[
        l10n.librarySourceOf(o.name),
        l10n.libraryTypeOf(o.signature),
      ],
];

/// The items matching [query], in library order.  Matches the localized
/// name and tags, the canonical English name, the default names of what
/// is created, the English keywords, the group and the unit,
/// case-insensitively — authoring convenience, never resolution; BDL
/// source is never searched as localized text.
List<pb.LibraryItemView> searchItems(
  Iterable<pb.LibraryItemView> all,
  String query, {
  AppLocalizations? l10n,
}) {
  l10n ??= kEnglish;
  final q = query.trim().toLowerCase();
  if (q.isEmpty) return all.toList();
  bool hit(String s) => s.toLowerCase().contains(q);
  return [
    for (final i in all)
      if (hit(itemName(l10n, i)) ||
          hit(libraryItemStrings(l10n, i.id)?.tags ?? '') ||
          hit(i.displayName) ||
          hit(i.group) ||
          hit(categoryLabel(l10n, i.group)) ||
          hit(categoryTitle(l10n, i.category)) ||
          i.keywords.any(hit) ||
          i.creates.any((o) => hit(o.name) || hit(o.unit)))
        i,
  ];
}

/// The concept templates matching [query] (the canvas menu's view of the
/// same search).
List<pb.ConceptTemplateView> searchTemplates(
  Iterable<pb.ConceptTemplateView> all,
  String query, {
  AppLocalizations? l10n,
}) {
  l10n ??= kEnglish;
  final q = query.trim().toLowerCase();
  if (q.isEmpty) return all.toList();
  bool hit(String s) => s.toLowerCase().contains(q);
  return [
    for (final t in all)
      if (hit(libraryItemStrings(l10n, t.id)?.name ?? t.displayName) ||
          hit(libraryItemStrings(l10n, t.id)?.tags ?? '') ||
          hit(t.displayName) ||
          hit(t.defaultName) ||
          hit(t.category) ||
          hit(categoryLabel(l10n, t.category)) ||
          hit(t.unit) ||
          t.keywords.any(hit))
        t,
  ];
}

/// The drag payload from a library row: the item id.  Dropped on the
/// canvas, it becomes one [InsertLibraryItemRequested] at the drop point.
class LibraryItemDrag {
  const LibraryItemDrag(this.itemId, {this.source = false});
  final String itemId;

  /// A Source item: the drop opens the Source sheet at the drop point
  /// instead of inserting (the concept is chosen there).
  final bool source;
}

class LibraryPanel extends StatefulWidget {
  const LibraryPanel({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  State<LibraryPanel> createState() => _LibraryPanelState();
}

class _LibraryPanelState extends State<LibraryPanel> {
  late final TextEditingController _search = TextEditingController(
    text: widget.state.editor.librarySearch,
  );

  @override
  void didUpdateWidget(covariant LibraryPanel old) {
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
    final all = s.libraryItems.toList();
    final shown = searchItems(all, query, l10n: context.l10n);
    // the two sections, Concepts then Sources, each with its groups in
    // library order
    final sections = <String, List<String>>{};
    for (final x in shown) {
      final groups = sections.putIfAbsent(x.category, () => []);
      if (!groups.contains(x.group)) groups.add(x.group);
    }
    final order = [
      'concept',
      'source',
      ...sections.keys.where((c) => c != 'concept' && c != 'source'),
    ];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(MacMetrics.gap, MacMetrics.gap, MacMetrics.gap, 0),
          child: MacTextField(
            controller: _search,
            hint: context.l10n.searchLibrary,
            onChanged: (q) => widget.dispatch(LibrarySearchChanged(q)),
          ),
        ),
        Expanded(
          child: library == null
              ? _Note(
                  s.connection is Connected
                      ? context.l10n.loadingTheLibrary
                      : context.l10n.theLibraryArrivesWithTheCompiler,
                )
              : shown.isEmpty
              ? _Note(context.l10n.noLibraryMatches(query.trim()))
              : ListView(
                  padding: const EdgeInsets.symmetric(vertical: MacMetrics.gapTight),
                  children: [
                    for (final category in order)
                      if (sections.containsKey(category)) ...[
                        _SectionHeader(
                          categoryTitle(context.l10n, category),
                          key: ValueKey('library-section-$category'),
                        ),
                        for (final g in sections[category]!) ...[
                          _CategoryHeader(categoryLabel(context.l10n, g)),
                          for (final item in shown)
                            if (item.category == category && item.group == g)
                              _ItemRow(
                                key: ValueKey('library-item-${item.id}'),
                                item: item,
                                enabled: canInsert,
                                onInsert: () => widget.dispatch(
                                  item.category == 'source'
                                      ? NewSourceRequested(presetId: item.id)
                                      : InsertLibraryItemRequested(item.id),
                                ),
                              ),
                        ],
                      ],
                    if (s.project == null) _Note(context.l10n.openAProjectToInsertFrom),
                  ],
                ),
        ),
        if (library != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(12, MacMetrics.gapTight, 12, MacMetrics.gap),
            child: Text(
              [for (final l in library.libraries) '${l.name} ${l.version}'].join(' · '),
              style: TextStyle(fontSize: MacType.caption, color: t.textTertiary),
              overflow: TextOverflow.ellipsis,
            ),
          ),
      ],
    );
  }
}

/// A section of the library: Concepts, Sources.
class _SectionHeader extends StatelessWidget {
  const _SectionHeader(this.title, {super.key});
  final String title;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 12, 12, 0),
      // A section title as the sidebar's: 11 semibold, secondary — sentence
      // case, no letter-spacing (docs/architecture/studio-ui.md §3).
      child: Text(title, style: Theme.of(context).textTheme.titleSmall),
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

class _ItemRow extends StatelessWidget {
  const _ItemRow({super.key, required this.item, required this.enabled, required this.onInsert});
  final pb.LibraryItemView item;
  final bool enabled;
  final VoidCallback onInsert;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final l10n = context.l10n;
    // A Source item opens the Source sheet, prefilled: its row wears the
    // Source silhouette (the canvas's glyph, ADR-0032); a Concept item's
    // row wears the socket-to-be.
    final row = SizedBox(
      height: MacMetrics.rowHeight,
      child: Row(
        children: [
          if (item.category == 'source')
            const MappingGlyph(declared: false, wrong: false, source: true)
          else
            ItemGlyph(item: item),
          const SizedBox(width: MacMetrics.gap),
          Expanded(child: Text(itemName(l10n, item), overflow: TextOverflow.ellipsis)),
          Text(
            itemWord(l10n, item),
            // The unit column: secondary, tabular — read to act on, so never
            // below 11 or tertiary.
            style: TextStyle(
              fontSize: MacType.secondary,
              color: t.textSecondary,
              fontFeatures: const [FontFeature.tabularFigures()],
            ),
          ),
        ],
      ),
    );
    // the tooltip: the description, then — for a Source preset — what it
    // does: an input for a concept the designer chooses on the sheet
    final message = [
      itemDescription(l10n, item),
      if (item.category == 'source') '',
      if (item.category == 'source') sourceItemHover(l10n, item),
    ].join('\n');
    final interactive = Tooltip(
      message: message,
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
      child: Draggable<LibraryItemDrag>(
        data: LibraryItemDrag(item.id, source: item.category == 'source'),
        dragAnchorStrategy: pointerDragAnchorStrategy,
        feedback: ItemDragFeedback(item: item),
        child: interactive,
      ),
    );
  }
}

/// The item's socket-to-be: grey (identity not allocated yet), filled when
/// its value has a representation, hollow when that is left to be decided.
/// A Source item's glyph is the value's: the relationship beside it has no
/// socket of its own to preview.
class ItemGlyph extends StatelessWidget {
  const ItemGlyph({super.key, required this.item, this.size = 9});
  final pb.LibraryItemView item;
  final double size;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final grey = t.textSecondary;
    final value = item.creates.where((o) => o.kind == 'concept').firstOrNull;
    final decided = value?.hasRepresentation() ?? false;
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        color: decided ? grey : Colors.transparent,
        border: Border.all(color: grey, width: 1.5),
      ),
    );
  }
}

/// What travels under the pointer during a drag: the node's header as it
/// will look, with the grey socket — the value's name, then the source's.
class ItemDragFeedback extends StatelessWidget {
  const ItemDragFeedback({super.key, required this.item});
  final pb.LibraryItemView item;

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
            ItemGlyph(item: item),
            const SizedBox(width: MacMetrics.gap),
            Expanded(
              child: Text(
                item.creates.map((o) => o.name).join(' · '),
                style: TextStyle(
                  fontSize: MacType.nodeTitle,
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
