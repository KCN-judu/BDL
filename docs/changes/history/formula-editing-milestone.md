# The formula-editing milestone: bugs found and fixed, and its non-goals

*Moved from `docs/STUDIO_COMPILER_INTEGRATION.md` §2 and §4 on
2026-09-17 so that page states the current integration only. This is the
record of the Studio definition-editor milestone (protocol 0.4,
`AnalyzeDefinitionDraft`); the regression tests named here still run.*

## Bugs found and fixed

| # | Bug | Root cause | Fix | Regression test |
|---|---|---|---|---|
| 1 | Pressing *Attach* cleared the field at once; a refused attach (stale revision, `already_defined`, daemon down) lost the formula | `CommitTextField` cleared its controller on dispatch, before any answer | drafts live in `EditorState`; `pendingCommit` keeps the text until the projection confirms; `RequestFailed` keeps it with the reason | `draft_reducer_test` "a failed commit preserves the text"; `definition_editor_test` "a failed save keeps the text"; `formula_e2e_test` "a refused commit keeps the draft" |
| 2 | Switching selection and back lost an in-progress formula | the text lived only in the inspector widget's controller, keyed by mapping id | drafts keyed by mapping id in app state; the field follows the state | `definition_editor_test` "switching selection and back restores the draft" |
| 3 | Clicking anywhere (a canvas node, another field) committed the formula as a side effect; attach on blur | `CommitTextField` committed on focus loss | edits happen only on *Add/Save definition*, *Detach*, or ⌘S/⌘↩ | `definition_editor_test` "editing an existing definition … can be saved or reverted" (typing alone does not change the project) |
| 4 | Attach vs replace chosen by which widget branch was mounted; an undo/detach landing meanwhile made the next save fail with `edit.not_defined`/`already_defined` and the text vanished with the branch switch | UI-derived op | op chosen from the committed projection at commit time | `draft_reducer_test` "attach when there is no definition, replace when there is" |
| 5 | Rapid attach twice sent two `AttachDefinition`s (second refused, text lost) | no in-flight state | a commit is refused while `pendingCommit` is set | `draft_reducer_test` "a commit is sent once"; `definition_editor_test` "add definition sends one attach" |
| 6 | Diagnostic excerpts wrong after any non-ASCII character (`×`, `°`) | byte offsets used as UTF-16 indices | `source_span.dart` | `source_span_test` |
| 7 | A signature or representation change while typing left the old verdict on screen | no draft, no re-analysis | every new revision rebases and re-checks the draft | `draft_reducer_test` "a new revision rebases the draft"; `formula_e2e_test` "stale path" |
| 8 | The committed definition changing under an edit (another client, undo) silently replaced the text when the field lost focus | `didUpdateWidget` overwrote unfocused fields | conflict state with *Reload* / *Keep mine* | `draft_reducer_test` "…is a conflict, not a loss"; `definition_editor_test` "a conflict is a notice" |
| 9 | Closing the project (or the daemon exiting) discarded an in-progress formula | project state cleared wholesale | dirty drafts stashed by project path, restored on reopen | `draft_reducer_test` "close and reopen"; `draft_effects_test` "the daemon exiting stashes the draft" |
| 10 | A concept created from the *New concept* sheet lost its chosen kind and unit: every concept was created *open*, so every formula over it was *Open* instead of typed | the reducer built `CreateConcept` from name and description only | `representation` forwarded | `reducer_test` "a new concept carries the kind chosen in the sheet"; the e2e suite depends on it |
| 11 | Formula-local diagnostics only appeared in the project-wide *Compiler* section, after the commit | no draft analysis | `AnalyzeDefinitionDraft`; diagnostics under the field | `formula_e2e_test` "invalid path" |
| 12 | Every draft check would have counted as a pending request and shown the app as busy | `_call(counted: true)` default | draft checks are uncounted and answered by their own actions | `draft_effects_test` "a burst of keystrokes …" (`pendingRequests == 0`) |

Not reproduced (verified correct by reading and by test): stale project
projections are dropped (`reducer_test`); an analysis for another revision
is dropped and the inspector shows *checking* until the right one arrives
(`reducer_test` analysis tests); a mapping's deletion clears selection and
draft (`draft_reducer_test`); reopen preserves the source exactly
(`stdio_e2e` `definition_drafts_over_stdio`, `formula_e2e_test` valid
path).

## 3. Integration matrix

## Non-goals of the milestone (as recorded at the time)

* No draft indication on the canvas: the canvas draws committed state only
  (a dirty draft must not make an unresolved mapping look defined). The
  status line's *N unsaved definitions* is the whole-app cue.
* Rich highlighting beyond wavy underlines (no gutter). The
  controller-based approach can grow into that without changing the state
  model.
* Per-request attribution of `RequestFailed`: the protocol answers by
  request id inside the client, but the reducer sees one stream. A draft
  awaiting confirmation takes the message; with two commits in flight for
  two mappings (impossible from the UI: each mapping has one field) the
  attribution could be wrong.
* The simulation restarts the run for every step rather than extending the
  daemon's input trace: no protocol change, and the trace is Studio's own.
  An append-only `inputs` on `StepSimulation` would save the restart if
  runs grow long.
* The board choice is a session preference, not project data.
