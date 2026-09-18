# A save keeps the whole authoring state; one guard on every way out, protocol 0.15

- Date: 2026-09-19
- Area: persistence, daemon, protocol, studio
- Affected: designers, project authors, protocol clients, developers
- Related: ADR-0030 (extends ADR-0023 §5, ADR-0020 §9)

## What changed

- **Saving requires nothing to build.** Text typed in the Code view that does
  not build is saved as typed under `src/`; the last text of that file that
  built goes to `.bdl/authoring.json` (`source_drafts`), so the graph and the
  identities keep describing the project until the typed text builds again. A
  file repaired in an editor ends its draft on open.
- **Definition drafts are project state.** Text typed for a relationship and not
  committed — valid, invalid or empty — is saved by scope and relationship
  (`.bdl/authoring.json` `definition_drafts`), restored on open and reported in
  `SystemView.definition_drafts` (protocol 0.15). Studio keeps no stash of its
  own across a close any more.
- **Dirty is one question.** `ProjectProjection.dirty` says whether the
  persistent state — the system, the layout, the text of every file (typed or
  accepted), every definition draft — differs from what was saved or loaded.
  Nothing else counts as dirty anywhere.
- **One guard on every way out of a project.** Close, the project manager, Open
  / New while a project is open, ⌘W, ⌘Q, the menu's Quit and the window's close
  button all send unsent typing to the project, ask it, and either unload at
  once (clean) or ask _Save changes to “name”?_ — _Don't Save / Cancel / Save_.
  Save unloads only after the save succeeded; a refused save keeps the project
  open with its reason; Don't Save returns to what was saved on the next open.
  The macOS runner turns the window's close button into a terminate request the
  app may decline, so nothing is torn down while the question is open.
- **Where the designer was** — the view (Design / Code / Split), the page, the
  open source file, the component whose source was open — is kept per user in
  the recent list and restored on the next open.
- `bdld check` reports a saved draft's faults with the position in the typed
  text.

## Compatibility and migration

- **Project authors.** Nothing to do: `.bdl/authoring.json` reads as before
  without the new sections, and a project saved with them opens in older tools
  as the typed sources (an older tool sees the file that does not build as
  faults, and does not know the drafts).
- **Protocol clients at 0.14** keep working (`major` unchanged);
  `SystemView.definition_drafts` is new and may be ignored. A client with
  formula editors should seed them from it on open and stop keeping drafts
  across a close.
- **Developers.** `bdl_text::save_project_with` / `Drafts` /
  `DefinitionDraftFile`; `Session::definition_drafts`, `dirty` now covers
  drafts; Studio's `NewProjectKind`-era stash (`_stash`) is gone, replaced by
  `app/lifecycle.dart`.

## Evidence

`crates/bdl-text/tests/drafts.rs`, `crates/bdl-daemon/tests/text_e2e.rs`
(`unfinished_edits_survive_save_close_and_reopen`),
`apps/studio/test/lifecycle_reducer_test.dart`,
`apps/studio/test/close_guard_test.dart` (the real shell and the desktop's exit
request), `apps/studio/test/persistence_e2e_test.dart` (against `bdld`: valid
and invalid formula drafts, text that does not build, a moved node — Save,
close, reopen, exact text; Don't Save reverts),
`apps/studio/test/draft_reducer_test.dart`.
