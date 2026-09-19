# The Standard Library: items, one transaction, presentation-owned text

- Date: 2026-09-19
- Area: library, daemon, protocol, studio, l10n
- Affected: designers, protocol clients, developers
- Related: ADR-0032 (amended), ADR-0031, ADR-0029,
  [2026-09 Source role](2026-09-source-role.md)

## What changed

- **The Standard Concept Library is the Standard Library**: a catalogue of
  _items_, each a fragment of ordinary BDL objects, in two sections — _Concepts_
  (36, one concept each) and _Sources_ (8, a concept and its `() -> concept`
  relationship). The Library tab shows the two sections with their groups; the
  Source rows wear the Source silhouette and say what they create. _External
  Value_ joins the Sources (a host, a network, a simulation — a Source is a
  boundary, not a sensor); _Button / Switch State_ is _Button State_; _Analog
  Input_ and _External Value_ leave the value form to the designer.
- **Instantiating an item is one transaction**: every object of the fragment or
  none. A Source item's concept and relationship are exactly one revision and
  one history entry (one Undo removes both, a Redo recreates them with the same
  identities); when any step is refused nothing is applied — no object, no
  layout placement, no history entry, no dirtiness, no identity consumed. A name
  chosen for an object is used as given (a taken name is refused, never silently
  numbered); a key the item does not create is refused (`library.invalid_plan`).
- **Item names, descriptions and search tags follow the language** — all 44
  items in English, 简体中文 and 日本語 — through Studio's ordinary
  localization; the library, the compiler service and the protocol carry English
  only, and what an item creates (`RoomTemp`, `TempSensor`) never changes with
  the locale. Search matches the localized words: `温度`, `传感器`, `センサー`
  find the Temperature Sensor.
- `library/std/concepts.toml` is **schema 2**: `[[template]]` (a Concept item)
  and `[[source]]` with `[source.value]` / `[source.relationship]`; schema 1
  files still load.

## Compatibility and migration

- Designers: nothing to do; every project means what it did. A Source made from
  the library before this change is the same two objects.
- Project files: nothing — a project never records an item.
- Protocol clients: protocol **0.17**, additive — `ListLibraryItems` /
  `LibraryItemsResponse` (`LibraryItemView`, `LibraryObjectView`) and
  `InstantiateLibraryItem { item_id, names{key → name}, component? }`.
  `ListConceptTemplates` / `InstantiateConceptTemplate` stay and serve the
  Concept items; the 0.16 fields `ConceptTemplateView.source_default_name`,
  `display_names`, `descriptions` and
  `InstantiateConceptTemplateRequest.source_name` are deprecated and never set —
  a client that used them lists Sources through `ListLibraryItems` and localizes
  by item id. Refusals: `library.unknown_item`, `library.invalid_plan`.
- Library files: a `[[template]]` with a `source` or `i18n` field is refused
  (unknown keys); write a `[[source]]` and put translations in
  `locale/library/std.json`.
- Developers:
  `bdl_library::{LibraryItem, ItemCategory, Fragment, FragmentObject, ConceptSpec, MappingSpec, CreatedObject}`,
  `bdl_library::{PlannedStep, PlanError, plan, free_name}` (`plan` refuses a key
  the item does not create; a chosen name is used as given),
  `Library::{items, item, search_items}`, `LibrarySet::{items, item}` (the
  template-only `instantiate`, `search` and `categories` are gone);
  `ConceptTemplate` is the projection `LibraryItem::as_concept_template`;
  `Session::{transaction, apply_library_item}` replace `apply_system_then`;
  Studio `InsertLibraryItemRequested`, `LibraryItemsReceived`,
  `AppState.libraryItems`, `ui/library_panel.dart` (`LibraryPanel`, `itemName`,
  `searchItems`), `l10n/library_strings.dart` (generated). `just library-l10n`
  regenerates the presentation strings; preflight `l10n` checks them.

## Evidence

`crates/bdl-library/src/lib.rs` (unit tests),
`crates/bdl-daemon/tests/stdio_e2e.rs` (`library_items_over_stdio`),
`crates/bdl-daemon/tests/text_e2e.rs`
(`a_source_item_is_two_ordinary_edits_in_one_commit_and_writes_the_unit_domain`),
`apps/studio/test/library_test.dart`, `apps/studio/test/library_e2e_test.dart`,
`apps/studio/test/source_role_test.dart`, `scripts/gen_library_l10n.py --check`.
