# Studio ↔ compiler integration

How Studio exposes the compiler's semantic state, what it does not expose
yet, and the record of the formula-editing milestone: the bugs found, their
root causes, the fixes and the regression tests.

Ownership never moves: Flutter owns the draft text, focus, dirtiness,
request generations and selection; Rust owns parsing, name resolution,
elaboration, types, dimensions, causality, clocks, outputs and deployment
(ADR-0001). Studio computes no semantic judgment — not even "does it parse".

## 1. The definition editor

```
designer types
   │  DefinitionDraftChanged            (reducer: app/drafts.dart)
   ▼
EditorState.drafts[mapping] = DefinitionDraft { source, baseRevision,
                               baseDefinition, generation, check, … }
   │  AnalyzeDraft effect               (executor: 150 ms debounce per mapping)
   ▼
bdld  AnalyzeDefinitionDraft { revision, mapping_id, generation, source }
   │  read-only: the draft becomes an overlay on the session's IdeHost, the
   │  committed snapshot + overlays are composed and the whole pipeline runs
   │  (bdl_ide::draft_verdict → bdl_compiler::analyze); no revision
   ▼
DefinitionDraftAnalysis { revision, mapping_id, generation, parse_ok, MappingAnalysis }
   │  DraftAnalysisReceived  — kept only if generation == draft.generation
   │                           and revision == held revision == draft.baseRevision
   ▼
status line + underlined spans + diagnostic rows under the field

designer presses Add definition / Save definition (⌘S, ⌘↩)
   │  CommitDefinitionRequested — exactly one ApplyEdit; Attach when the
   │  committed projection has no definition, Replace when it has one;
   │  draft kept with pendingCommit until the projection confirms
   ▼
ProjectChanged / EditApplied → AnalysisReady   (the usual revision path)
```

### Draft state (`apps/studio/lib/app/state.dart`)

| Field | Meaning |
|---|---|
| `source` | what the designer typed; never lost by a push |
| `baseRevision` | the project revision the draft was last rebased on |
| `baseDefinition` | the committed formula at that base (`null` = none); tells "the definition moved under me" from "something else moved" |
| `generation` | monotonic per draft; bumped by every source change and every rebase; only the latest generation's verdict is accepted |
| `check` | `checking` / `checked` / `unavailable` |
| `analysis`, `parseOk` | the compiler's `MappingAnalysis` for exactly (source, baseRevision) |
| `conflict` | the committed definition changed while the draft was dirty |
| `pendingCommit` | the trimmed source sent in a commit not yet confirmed |
| `commitError`, `checkError` | the last failure, shown next to the editor |

Invariant: a draft exists only while its text differs from the committed
definition (typing the committed text back dissolves it) or while a commit
awaits confirmation.

### Lifetime rules (`apps/studio/lib/app/drafts.dart`)

| Event | Draft |
|---|---|
| widget rebuild, `AnalysisReady`, same-revision projection (a save) | untouched |
| new revision of the open project | rebased: `baseRevision` moves, generation bumps, re-checked — any change (signature, representation, an input's *name*) can change what the same text means |
| new revision where the committed definition ≠ `baseDefinition` and the draft is dirty | `conflict = true`; nothing overwritten; the editor offers *Reload* (take theirs) / *Keep mine* (rebase on theirs) |
| new revision confirming `pendingCommit` | dropped if the text equals it; rebased (not a conflict) if the designer kept typing |
| the mapping is gone | dropped |
| `RequestFailed` while `pendingCommit` is set | `pendingCommit` cleared, `commitError` set, text kept |
| `DraftAnalysisFailed` with `draft.stale_revision` | ignored: the projection that follows re-asks |
| any other `DraftAnalysisFailed` | `check = unavailable`, text kept |
| Revert (Esc), Reload, Detach | dropped on purpose |
| project close, daemon exit | dirty drafts stashed by project path; restored (rebased, conflict-checked) when that project is opened again — no modal, nothing typed is discarded |

### Attach vs replace

The designer sees *Add definition* (no committed definition) or *Save
definition* (there is one). The `EditOp` is chosen in the reducer from the
committed projection at commit time, never by the widget branch, so a
detach or undo that lands between typing and saving cannot send the wrong
op. `Detach definition` is `ReplaceDefinition(None)`.

### Committing an invalid definition

The model allows it (`apply_edit` does not consult the elaborator): the
mapping is then `Invalid` with the same diagnostics the draft showed. This
is the existing policy — a definition may be recorded before it is right,
as a concept may exist before its representation — and this milestone does
not change it. The editor makes the consequence visible before the commit
and again after it (the committed verdict shows in the same place once the
draft dissolves).

### Debounce and generations

`kDraftDebounce = 150 ms` in `effect_executor.dart`, per mapping: a burst
of keystrokes costs one request, the last one. The generation is assigned
by the reducer at the keystroke, so a superseded request is simply never
sent and a late answer is dropped by generation. Commit never waits on the
debounce (it is a separate, counted `ApplyEdit`).

### Timing (local, debug build, `formula_e2e_test.dart`)

| Leg | Measured |
|---|---|
| edit → verdict in state, including a 30 ms test debounce | 36–37 ms |
| ⇒ request → response → reducer, `Tilt / 90 deg` over a two-concept design | ≈ 6 ms |
| `bdl-daemon` e2e, whole draft round trip (Rust client) | sub-millisecond per request in `definition_drafts_over_stdio` |

Draft analysis re-runs the whole pipeline over the effective design
(committed + overlay). That is the point (one semantic truth) and it is
cheap at this size (~2 ms at 400 mappings, docs/IDE_SERVICE_ARCHITECTURE.md);
when designs grow, the compiler's incremental invalidation (`EditOutcome`)
is the lever, not a Dart-side shortcut.

### Keyboard

| Key | In the definition field |
|---|---|
| ⌘S, ⌘↩ | save the definition while dirty; ⌘S falls through to *Save project* otherwise |
| Esc | revert a dirty draft |
| Return | a new line (formulas may span lines) |

### Source spans

Compiler spans are UTF-8 byte ranges into the draft source; Flutter text
is UTF-16. `ui/source_span.dart` converts (never splitting a character;
spans past the end collapse at the end). The editor underlines every
diagnostic span in place (wavy, coloured by severity) and the diagnostic
row repeats the excerpt, so the location survives without colour. Spans of
a draft verdict index the draft text; spans of the committed verdict index
the committed text — the editor shows exactly one of the two.

## 2. Bugs found and fixed

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

| Compiler capability | Protocol | Studio state | UI surfaced | Tested |
|---|---|---|---|---|
| Revisioned edits, stale-revision refusal | `ApplyEdit`, `edit.stale_revision` | `pendingRequests`, `lastError` | banner; the definition editor keeps the draft | reducer, e2e |
| Formula parse + elaboration + typing, spans | `MappingAnalysis.diagnostics[].span` | `analysis`, `drafts[].analysis` | editor status line, underlines, diagnostic rows | e2e (spans exact) |
| Draft analysis (non-mutating) | `AnalyzeDefinitionDraft` / `DefinitionDraftAnalysis` (0.4) | `DefinitionDraft` | editor | Rust session unit + stdio e2e, Dart reducer/effects/widget/e2e |
| `MappingStatus` ladder to `ClockConsistent` | `MappingAnalysis.status` | `analysis` | inspector *State* pill; canvas node status; editor verdict | shell, editor |
| Nominal semantic types, `Grant` | diagnostics `semantic.*`, `realization.*` | `analysis` | editor / Compiler section rows | e2e (Open case) |
| Dimensions | `dimension.mismatch` with span | `analysis` | editor | e2e |
| Causality (cycles, evaluation order) | `causal`, `cycles[]`, `evaluation_order[]`, `reactive.instantaneous_cycle` | `analysis` | status line *not causal*; per-mapping diagnostic in the Compiler section; cycles and order **not drawn on the canvas** | shell (status line) |
| Clock domains | `clock_consistent`, `clock.*` diagnostics; `ClockView`, `MappingView.clock_id`; clock edit ops | `analysis`, `project.clocks` | status line *reads across domains*; per-mapping diagnostic; **no domain editing, no domain regions on the canvas** | shell |
| Physical outputs | `OutputView`, `OutputAnalysis`, `open_outputs`, `output_complete`; output edit ops | `project.outputs`, `analysis.outputs` | status line *outputs incomplete*; driver faults as mapping diagnostics; **no output nodes, no drive links, no output editing** | shell |
| Deployment analysis, targets | `ListTargets`, `AnalyzeDeployment` | **none** | **none** (Deploy page is a placeholder) | — |
| Device bindings, pins, conflicts | `DeviceView`, device edit ops, `DeploymentAnalysis.dead_end` | **none** | **none** | — |
| Simulation | `Start/Step/ResetSimulation` | **none** | **none** (Simulate page is a placeholder) | — |
| Edit classification (refinement / edit, invalidations) | `EditOutcome` | `lastOutcome` | inspector *Last change* | — |

### Placement decided for the gaps (not implemented here)

| Fact | Home |
|---|---|
| instantaneous cycles, evaluation order | canvas (the cycle's links emphasised); order in the Simulate page |
| clock domains and cross-domain reads | canvas regions (containment) + inspector *Domain* pop-up per mapping/output |
| outputs and drive links | canvas: a distinct silhouette for a sink, links from drivers; inspector for accepts/domain/required |
| deployment status, target, assignment, dead end | Deploy page: target pop-up, per-output → device → requirement → resource table, the blocker named |
| simulation values | Simulate page + values on sockets |

## 4. Non-goals of this milestone

* No draft indication on the canvas: the canvas draws committed state only
  (a dirty draft must not make an unresolved mapping look defined). The
  status line's *N unsaved definitions* is the whole-app cue.
* Rich highlighting beyond wavy underlines (no gutter, no hover
  explanations in the field). The controller-based approach can grow into
  that without changing the state model.
* Per-request attribution of `RequestFailed`: the protocol answers by
  request id inside the client, but the reducer sees one stream. A draft
  awaiting confirmation takes the message; with two commits in flight for
  two mappings (impossible from the UI: each mapping has one field) the
  attribution could be wrong.
