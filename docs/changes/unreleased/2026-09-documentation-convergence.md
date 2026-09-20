# Documentation convergence: the records match HEAD at protocol 0.23

- Date: 2026-09-20
- Area: process
- Affected: developers, coding agents
- Related: ADR-0032, ADR-0035, ISS-0015, ISS-0016, PRP-0001

## What changed

A repository-wide audit of the current pages against the code and tests at
`3fba224`, after the relationship-role unification (0.20), semantic highlighting
(0.21), the Code view IDE (0.22) and Source creation (0.23). Only current pages
moved; ADRs and change records are untouched.

- **Front door** (`docs/README.md`): the protocol row said 0.9 — it is 0.23; the
  Standard Library row said "concept template libraries" — it names Concept
  items, Source presets and Source creation; the snapshot counts thirteen open
  issues, names FV Phase 14 as an open direction, and its history paragraph is a
  pointer to `changes/` instead of the whole ledger.
- **Status** (`docs/project/status.md`): hover on an equation's name moved from
  _does not exist_ to _exists_ (formula editor, Code view and language server,
  `bdl_ide::navigation`); Code-pane completion and hover left the text-sync gap
  (inline fault ranges remain); the Studio row describes the Source sheet
  instead of "eight items that create the concept"; the protocol row reaches
  0.23; the snapshot intro is a short current statement; the component-scope gap
  is stated exactly (`HoverEntity` / `ListSemanticActions` take no scope).
- **Roadmap**: equation hover, Code-pane completion/hover and _Format_ deleted
  (landed); the Composer bullet lists only the forms still opaque (`match`,
  blocks, rules, literals, `delay`/`sync` — `if` is structured); the Code-view
  slice lists what is still missing; the library bullet separates LIB-2/LIB-3
  from output-side realization, which has no production record.
- **Architecture**: `overview.md` gains _Behaviour semantics stop at the output;
  realization is deployment's_ — the current output model
  (`PhysicalOutput { accepts, clock, required }`, exact `DriveWF`, device kind
  as deployment data, no encoder) and BDL's architectural position stated
  without usability claims; its crate table names `bdl-library`'s items and
  presets and `bdl-ide`'s navigation; `studio-ui.md` drops the Code-view "not
  built" items that landed, reads _Applied in_ from
  `MappingAnalysis.applied_by`, and describes a dropped Source row as opening
  the Source sheet; `ide-service.md` loses "low-code / high-code".
- **Spec**: `protocol.md` gains the 0.23 rows (`CreateSource`,
  `ListSourceCandidates`, `LibraryItemView.preset`); `concept-library.md` no
  longer says objects live in `design/project.bdl.json`.
- **Formal correspondence**: the FV repository's records were renamed on
  2026-09-20 (`D-NN` → `FVD-00NN`, `OI-NN` → `FVI-00NN`, notes under
  `docs/notes/`); every current page cites the new ids and paths (ADRs keep the
  old ones as history); the Source row names the model's derived role stated by
  the compiler; a Phase 14 row records output realization by device encoders as
  _formally proved, not implemented_.
- **User guide**: "template" → "item" / "Source preset" throughout; the front
  page no longer says the language need not be known; the workspace page names
  the Library's contents; the zh-Hans and ja catalogs cover every passage of the
  core pages again, including the Code view IDE and the Composer's logic. The
  guide's extractor treats a wrapped image line as an image (a `]` inside alt
  text no longer makes it a message).
- Studio's translator descriptions that said "template" say "library item".

## Compatibility and migration

Nothing to migrate: documentation only. A reader who cited an FV decision as
`D-NN` finds it as `FVD-00NN` in `../BDL_FV/docs/decisions/`; the map is
`../BDL_FV/docs/project/decision-id-migration.md`.

## Evidence

`just docs-check` (format, lint, the record validator, the screenshot ledger,
the localization checks), `python3 scripts/preflight.py fast`, and the tests the
corrected statements name: `crates/bdl-ide/tests/navigation.rs`
(`equations_hover_with_the_library_words_and_have_no_definition`),
`crates/bdl-daemon/tests/text_e2e.rs` (equation hover over the wire),
`apps/studio/test/code_ide_e2e_test.dart`,
`apps/studio/test/library_e2e_test.dart` and
`crates/bdl-daemon/tests/system_e2e.rs`
(`a_source_is_created_over_a_chosen_concept`).
