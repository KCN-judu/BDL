---
id: ADR-0030
status: accepted
date: 2026-09-19
area: persistence
supersedes: []
superseded-by: []
related: [ADR-0003, ADR-0009, ADR-0020, ADR-0023]
fv: []
---

# ADR-0030: Saving keeps the whole authoring state; dirty is one question

## Status

Accepted (complete-project persistence milestone). Extends ADR-0023, which made
the sources the semantic truth of every project and kept invalid text as a draft
over the last revision that built (§5) — but only in memory, so a save silently
dropped what did not build, and Studio kept formula drafts of its own that no
save reached.

## Context

A designer could see three kinds of edit that a save did not keep: text in the
Code view that did not build yet, a formula typed but not committed, and an
empty formula field. Each was "unsavable" only because it was incomplete or did
not typecheck — a distinction the designer never asked for. A project closed and
reopened lost that work, or kept it by accident (Studio's own stash, per app
session). Dirtiness was several flags in two places, and a close asked nothing.

Two questions were being confused: _what was the designer working on?_
(persistence) and _what does that work currently mean?_ (the compiler).

## Decision

1. **A save writes the complete current authoring state**, atomically per file,
   and requires nothing to build, typecheck, be attached or be deployable.
   Invalid or incomplete work is valid saved work.

2. **Where each fact lives** (`docs/spec/project-format.md`):

   | Fact                                                                                                 | Lives in                                                                  |
   | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
   | the text of a source file, exactly as typed, whether or not it builds                                | `src/<path>`                                                              |
   | the last text of that file that built, while the typed text does not                                 | `.bdl/authoring.json` `source_drafts[path].last_good`                     |
   | a definition draft — text typed for a relationship and not committed, valid, invalid or empty        | `.bdl/authoring.json` `definition_drafts[]`, by scope and relationship id |
   | identities, groups, layout, viewports, group boxes                                                   | as before (ADR-0020, ADR-0019, ADR-0003)                                  |
   | the view (Design / Code / Split), the page, the open source file, the component whose source is open | the per-user recent list (`recent.json`), never the project               |
   | selection, hover, drag previews, pending sheets, completion pop-ups                                  | nowhere                                                                   |

   The source-canonical architecture stands: `src/` is what the designer typed.
   The last-good text is a derived, persistent concern beside it so the graph
   and the identity table keep describing the project until the typed text
   builds again; a file that builds on open ends its draft.

3. **The daemon owns dirtiness, and it is one question**: the persistent state —
   the system, the layout, the text of every file (typed or accepted), every
   definition draft — differs from what was last saved or loaded. Studio keeps
   no dirty flag, no draft stash across a close, and no categories of dirtiness.

4. **Every project-unloading path runs one guard**: Close, the project manager,
   Open or New while a project is open, ⌘W, ⌘Q, the menu's Quit, the window's
   close button. Studio first sends whatever typing has not reached the project,
   then asks the project. Clean → unload at once. Dirty → _Save changes to
   “name”?_ with _Don't Save / Cancel / Save_. _Save_ unloads only after the
   save succeeded; a failed save keeps the project open and shows why. _Don't
   Save_ unloads; the next open returns to what was saved. _Cancel_ leaves
   everything as it was. The native window close and the application's exit
   request are declined until the guard has run; nothing is torn down while the
   question is open.

5. **Semantic acceptance and persistence are different concerns.** A definition
   draft is saved whether or not it checks; committing it is a semantic act.
   Text that does not build is saved as typed; what it means is the compiler's
   answer. The changed-on-disk protection (ADR-0020 §9) is unchanged: a save
   over external changes offers _Reload / Overwrite / Cancel_, never silently.

## Alternatives

- **Keep drafts in Studio and write them to a Studio-side file.** Two
  authorities for one project; a second Studio, the CLI and the language server
  would not see them; "dirty" would stay split. Lost.
- **Keep the last-good text in the source file and the typed text in a
  sidecar.** Inverts what the designer sees: an external editor would show old
  text and overwrite the typed text. Lost on the source-canonical principle.
- **Refuse to save while something does not build.** Makes the designer repair
  before keeping, the opposite of a save. Lost.
- **Autosave / crash recovery instead of a guard.** A separate feature with its
  own store and semantics; conflating it with the save model would make "saved"
  mean two things. Deferred, independently.

## Consequences

- `.bdl/authoring.json` gains two optional sections (schema unchanged, older
  files read as before); protocol 0.15 adds `SystemView.definition_drafts` so a
  client seeds its editors from the project on open.
- `bdld check` reports a saved draft's faults with the file position of the
  typed text.
- `ProjectProjection.dirty` is the only dirtiness a client needs; the projection
  answering `GetProject` after a flush is the guard's evidence.
- Status, the change fragment and the user guide record what of this is
  implemented; this record does not.
