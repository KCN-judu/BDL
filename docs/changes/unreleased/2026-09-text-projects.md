# Text projects: `.bdl` sources as the canonical design, protocol 0.9

- Date: 2026-09-17
- Area: textual, persistence, protocol, daemon, ide, studio
- Affected: designers, project authors, protocol clients, developers
- Related: ADR-0020, ADR-0014, ADR-0017; ISS-0005 (enums stay open)

## What changed

* **A third project kind.** `bdl.toml` may say `kind = "text"`: the design
  is `src/**/*.bdl` and there is no `design/*.json`. Two tool-owned
  sidecars appear beside the sources: `.bdl/identities.json` (source
  key → stable id, allocators, flat ids) and `.bdl/authoring.json`
  (behaviour groups). `ui/layout.json` is unchanged.
* **Syntax v0.2.** `clock`, `@domain`, `output … optional`, `drive`,
  `device … for … { pin n = P }`, `component { use / param clock / clock /
  requires / provides / param / mapping / enum / output / drive / device }`,
  `instance … { … }`, `bind … [init e]`, `export … as …`, and `///`
  descriptions. `context` and `require` remain reserved. Parameter names in
  a definition are lexical bindings (`MappingBlock.parameters`): a concept
  rename no longer changes what a body means.
* **Protocol 0.9** (additive): `PROJECT_KIND_TEXT`, `InitTextProject`,
  `ReloadProject`, `SaveProject.force`, `SessionInfo.textual`, and the
  error `project.changed_on_disk`.
* **bdld** rewrites the formula bodies that read a renamed concept by its
  display name (and the shared copies in component bodies) in the same
  revision as the rename; a client observing `EditApplied` sees one commit.
* **bdl-lsp** works over a whole text workspace (cross-file navigation and
  rename, scope-aware completion, formatting, inlay hints, virtual
  documents) and writes the identity sidecar after saves and watched-file
  changes. `bdld check | compile | simulate` read the same files.
* **Studio** offers *New Text Project…*, opens text projects as system
  projects, and refuses to save over sources changed on disk, offering
  *Reload from disk* or *Overwrite*.

## Compatibility and migration

* Flat and system projects are unchanged; no migration.
* Protocol clients at 0.8 keep working; a client that saves a text
  project must handle `project.changed_on_disk` (retry with `force`, or
  `ReloadProject`).
* A project written by hand gets its identities allocated and written to
  `.bdl/identities.json` on first open; commit the sidecar with the
  sources.
* Renaming two items of the same kind in one file by hand between two
  reads gives both fresh identities (`text.ambiguous_identity`); use the
  editor's rename or Studio to keep them.

## Evidence

`crates/bdl-text/tests/workspace.rs`, `crates/bdl-daemon/tests/text_e2e.rs`,
`crates/bdl-daemon/tests/cli.rs`, `crates/bdl-lsp/tests/text_workspace.rs`,
`apps/studio/test/text_project_e2e_test.dart`; user guide
`docs/user-guide/textual/`. Commits `25a46a2`…`d606405`.
