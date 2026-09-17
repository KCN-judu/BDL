# One BDL project: sources canonical, Design · Code · Split as views, protocol 0.10

- Date: 2026-09-18
- Area: persistence, layout, protocol, daemon, lsp, studio
- Affected: designers, project authors, protocol clients, developers
- Related: ADR-0023 (supersedes ADR-0020's project *kind*; keeps its mechanisms), ADR-0003, ADR-0008, ADR-0019

## What changed

* **One kind of project.** `bdl.toml` is schema 2 and has no `kind`. Every
  project is `src/**/*.bdl` + `.bdl/identities.json` + `.bdl/authoring.json`
  + `ui/layout.json`; every open project is a behaviour system (a design
  with no components is the degenerate one). `design/project.bdl.json`
  and `design/system.bdl.json` are legacy formats read only to migrate.
  `docs/spec/project-format.md`.
* **Legacy projects migrate on open, once, in place** (`bdl-text::migrate_legacy`,
  run by bdld, the CLI and the language server before anything reads the
  project): the JSON model is rendered to `src/main.bdl`, identities are
  seeded from the ids it already has (layout, hues and references keep
  meaning; allocators carry over), groups go to `.bdl/authoring.json`, the
  sources are verified to read back as the same model, the JSON file is
  renamed `*.migrated`, the manifest is rewritten. Layout is untouched.
  There is no path back to JSON and no convert command.
* **Names are identifiers.** The text is the semantic source, so a name
  it cannot spell (a space, a leading digit, a keyword) is refused on
  every surface with `edit.invalid_name`; a legacy display name maps
  deterministically on migration (`Light Output` → `Light_Output`, a
  keyword gets a trailing `_`). `docs/spec/textual-syntax.md` §2.5.
* **The layout service** (`crates/bdl-layout`, ADR-0023 §7). Every
  entity without a position is placed by bdld on open (persisted to
  `ui/layout.json`) and on every commit: deterministically, in the column
  of its kind, beside the positioned entities it reads or produces,
  stepped down until it overlaps nothing, never moving what has a
  position; component bodies are canvases too. Studio no longer arranges
  nodes at render time.
* **Protocol 0.10** (additive): `GetSources` → `SourcesView { revision,
  files[] { path, text, draft, anchors[] }, diagnostics[] }` and
  `ApplySourceEdit { base_revision, path, text }` → `SourceEditApplied
  { accepted, project, sources }`; `ProjectProjection.kind` always
  reports `PROJECT_KIND_TEXT`; `InitSystemProject` and `InitTextProject`
  behave as `InitProject`; `Undo`/`Redo` always answer
  `SystemEditApplied`; a `Base { op }` the flat model refuses inside
  `ApplySystemEdit` answers with that edit's own `edit.<reason>`. New
  refusals: `edit.invalid_name`, `source.invalid_path`. `docs/spec/protocol.md`.
* **bdld holds the text of the project.** `GetSources` is the files as
  loaded with every committed semantic edit written back through the same
  item-level splice a save performs; `ApplySourceEdit` binds the typed
  text to its identities by reconciliation and commits a new revision, or
  keeps the last revision that built and the draft with its faults
  (ADR-0023 §5). A save writes the accepted text.
* **Studio** offers one *New Project…* and, on the Design page, *Design ·
  Code · Split* views of the open project: an editor over the daemon's
  sources with the out-of-step banner, the daemon's reasons under the
  editor, and a selection shared with the canvas in Split.
  `docs/architecture/studio-ui.md` §12.

## Compatibility and migration

* **Project authors.** Every existing project opens; the first open
  rewrites it (a `design/*.json.migrated` file appears beside new `src/`
  and `.bdl/` files, and `bdl.toml` becomes schema 2). Commit the new
  files with the project. A design whose display names were not
  identifiers gets identifier names; the identities behind them are the
  same. A project that already had `src/` files beside a JSON design is
  refused with `project.text` naming the conflict rather than merged.
* **Protocol clients at 0.9** keep working (`major` unchanged). A client
  that branched on `PROJECT_KIND_FLAT` / `PROJECT_KIND_SYSTEM` now always
  sees `PROJECT_KIND_TEXT`; a client that expected `EditApplied` from
  `Undo`/`Redo` must accept `SystemEditApplied`; a client that read
  `system_edit.base` must read the inner `edit.<reason>`.
* **Developers.** `bdl-model::persist::load_project` /
  `bdl-system::persist::load_system_project` read legacy JSON only;
  `bdl-text::load_project` is the loader. Studio's `NewProjectKind` is
  gone.

## Evidence

`crates/bdl-text/tests/migrate.rs`, `crates/bdl-compiler/tests/examples.rs`
(the checked-in example is migrated from a legacy copy and compared),
`crates/bdl-layout/tests/place.rs`, `crates/bdl-daemon/tests/text_e2e.rs`
(`a_hand_written_project_is_placed_on_open_and_new_items_on_commit`,
`code_view_edits_flow_through_the_model_and_keep_identities`),
`crates/bdl-daemon/tests/system_e2e.rs`, `crates/bdl-lsp/tests/e2e.rs`,
`apps/studio/test/sources_reducer_test.dart`,
`apps/studio/test/code_view_e2e_test.dart`,
`apps/studio/test/canvas_geometry_test.dart`.
