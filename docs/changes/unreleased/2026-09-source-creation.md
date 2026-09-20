# A Source is created over a concept the designer chooses (protocol 0.23)

- Date: 2026-09-20
- Area: library, daemon, protocol, studio, l10n
- Affected: designers, protocol clients, developers
- Related: ADR-0032, ADR-0029, PRP-0001,
  [2026-09 Standard Library](2026-09-standard-library.md),
  [2026-09 relationship roles](2026-09-relationship-roles.md)

## What changed

- **A Source is never created without a concrete concept.** The Source sheet
  replaces the old _New source…_ form and the one-drag Source items: its one
  decision is the concept the Source provides — an **existing** concept of the
  design, by identity (only the Source is created; one Undo removes it alone),
  or a **new** concept created in the same transaction (one revision, one
  Undo, both or nothing) — then the Source's name. The sheet previews exactly
  what will be committed; cancelling changes nothing; the `() -> ?` of an
  unmade choice exists only on the sheet.
- **The Standard Library's Source items are presets**, not identity-owning
  fragments: each prefills the sheet (a suggested concept name, value form and
  unit; a suggested Source name) and the daemon lists the design's concepts
  with that value form first. Two concepts of one value form stay two choices;
  an open value form is a legal choice; nothing is inferred from names, units
  or dimensions and nothing is hidden. Choosing an existing concept no longer
  creates a redundant one.
- **The presets are named for what they are** — _Temperature Input_, _Tilt
  Input_, _Distance Input_, _Ambient Light Input_, _Button Input_, _Encoder
  Input_, _Analog Input_, _External Input_ (zh-Hans _温度输入_ …, ja
  _温度入力_ …) — a Source is an environment boundary, not a chosen device.
  Their ids (`std.source.temperature`, …) did not change; their suggested names
  are `Temperature` / `temperatureInput` and so on.
- **Add Source ▸** on the canvas lists _New Source…_ first, then the presets;
  every entry, and a Source row's double-click, Return or drag, opens the same
  sheet. The Library panel's Source rows say the value form a preset suggests
  and, on hover, _an input for a … concept you choose — existing, or new_ —
  never a signature over a concept nobody has chosen.

## Compatibility and migration

- Designers: a Source item no longer inserts on a click; the sheet opens. A
  project made before this change is unchanged.
- Project files: nothing — a Source is the same ordinary declaration,
  `mapping roomTemperatureInput : () -> RoomTemperature`; no `?`, no preset,
  nothing about how it was made.
- Library files: schema 2 unchanged. A `[[source]]` entry is read as a preset;
  `InstantiateLibraryItem` on it still creates both objects at once — the
  **legacy path**, kept for third-party schema-2 libraries and older clients,
  no longer taken by Studio.
- Protocol clients: protocol **0.23**, additive — `CreateSource`,
  `ListSourceCandidates` / `SourceCandidatesResponse`, `LibraryItemView.preset`.
  An older client works unchanged.
- Developers: `bdl_library::{SourcePreset, SourceCandidate, rank_concepts}`,
  `LibraryItem::preset`; `bdld` `create_source`, `list_source_candidates`;
  Studio `NewSourceRequested`, `SourceCandidatesReceived`,
  `SourceSheetDismissed`, `CreateSourceRequested`, `SourceSheetState`,
  `ui/source_sheet.dart` (`SourceSheet`, `SourceSheetForm`,
  `suggestedSourceName`), `PendingInsert.kSourceInsert`; `showNewSourceSheet`
  is gone; `UnitPreset` carries `typeName`.

## Evidence

`crates/bdl-library/src/lib.rs`
(`a_source_item_is_a_preset_that_ranks_and_suggests_and_owns_no_identity`),
`crates/bdl-daemon/tests/system_e2e.rs` (`a_source_is_created_over_a_chosen_concept`),
`crates/bdl-daemon/tests/stdio_e2e.rs` (the legacy path),
`apps/studio/test/source_role_test.dart` (the sheet), `apps/studio/test/library_test.dart`,
`apps/studio/test/library_e2e_test.dart` (against bdld).
