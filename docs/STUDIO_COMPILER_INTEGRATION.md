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

designer presses Add definition / Save definition (⌘↩)
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
| Revert (Esc), Reload, Detach, text typed back to the committed definition | dropped on purpose, with `DiscardDefinitionDraft` so the daemon's overlay goes too |
| project close, daemon exit | dirty drafts stashed by project path; restored (rebased, conflict-checked) when that project is opened again — no modal, nothing typed is discarded |

### One overlay, two cleanup moments

`IdeHost` owns the overlay and prunes it itself when a commit makes it
redundant (committed text == draft text) or orphaned (mapping deleted).
Studio's local draft dissolves under the same rule when the confirming
projection arrives — the two agree by construction, and the Rust rule is
the one that matters for other clients. The one cleanup only Studio knows
about is the designer *abandoning* a draft (Revert, Reload, Detach, or
typing the committed text back); for that Studio sends
`DiscardDefinitionDraft`, and the executor cancels any check still
debounced for the mapping so it cannot resurrect the overlay. Without
this, a reverted draft would stay the mapping's *effective* definition for
every later hover, completion or LSP query.

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
| ⌘↩ | save the definition while dirty (*Add definition* / *Save definition*) |
| ⌘S | unchanged: *Save project*; never commits a draft — a dirty definition and an unsaved project are different states |
| Esc | revert a dirty draft |
| Return | a new line (formulas may span lines) |

### Completion and hover

`CompleteDefinitionDraft` and `HoverDefinitionDraft` (docs/PROTOCOL.md)
serve `bdl_ide::completion` and `entity_at_formula` + `hover` over the same
overlay snapshot. The field consumes both (`definition_editor.dart`):

* ⌃Space opens the pop-up; while it is open every keystroke re-asks at the
  caret's byte offset (the service filters by prefix — Studio never does);
  ↑/↓ move, Enter/Tab accept by replacing the service's byte range with
  its insert text, Esc closes (a second Esc reverts the draft). Rows are
  the service's order — relevance, then label — with kind and resulting
  type; never re-sorted. Candidates: inputs, other relationships (a call
  to complete when they have inputs), units after a number, keywords,
  `delay(…)` / `sync(…)` in a relationship without inputs.
* A 250 ms dwell over a name asks for its card: title, value form,
  status, details, the description — never a Core term (that is Explain).
  Canvas and library entities can ask `HoverEntity` for the same card.
* Both carry a generation; only the latest answer at the held revision is
  applied, and a new revision or a selection change drops them.
  Measured: ~2 ms per round trip (`formula_e2e_test.dart`).

No name, unit or keyword table exists in Dart.

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

| Compiler capability | bdl-ide | Protocol | Studio state | UI exposed | Regression tested |
|---|---|---|---|---|---|
| Formula draft (non-mutating candidate) | `draft_verdict` over a `MappingDefinitionDraft` overlay; `clear_definition_draft`; `set_committed` pruning | `AnalyzeDefinitionDraft` / `DefinitionDraftAnalysis`, `DiscardDefinitionDraft` (0.4) | `DefinitionDraft` per mapping | definition editor: status line, underlined spans, diagnostic rows, Add/Save/Revert/Detach, conflict notice | Rust: session unit, stdio e2e, `surface_equivalence`; Dart: reducer, effects (fake daemon), widget, e2e against `bdld` |
| Formula parse / elaboration / typing with spans; relationship references, application, `delay`/`sync` (DI-17) | lifted to `SemanticDiagnostic` | `MappingAnalysis.diagnostics[].span` | `analysis`, `drafts[].analysis` | editor rows + underlines; committed span-less ones in *Compiler* | stdio e2e (spans exact), Dart e2e, `surface_to_backend` (surface → Core → evaluator → generated core) |
| Dimensions, nominal types, `Grant` | same | `dimension.mismatch`, `realization.*`, `semantic.*` | same | editor / Compiler section | Dart e2e (invalid → corrected), `surface_equivalence` |
| Open ≠ error | `SemanticSeverity::Open`; `EntityStatus::Open` | `DIAGNOSTIC_SEVERITY_INFO` + `MAPPING_STATUS_OPEN` | same | orange wording *Tilt has no representation yet.*; dashed sinks | stdio e2e (open case), widget tests, `surface_equivalence` |
| `MappingStatus` ladder to `ClockConsistent` | `hover` status, `explain` | `MappingAnalysis.status` | `analysis` | canvas object state (dashed, red mark), inspector, editor verdict | shell, widget |
| Completion (inputs, relationships, units, keywords, memory forms; type-directed) | `completion(Formula { mapping, offset })` | `CompleteDefinitionDraft` / `DraftCompletionResponse` | `CompletionState` (generation, offset, items, selection) | pop-up under the field: ⌃Space, ↑/↓, Enter/Tab, Esc; re-asked per keystroke | reducer (2,1,3 → 3; service order kept), widget, e2e against bdld (input, unit after number, non-ASCII offsets) |
| Hover | `entity_at_formula` + `hover`; `hover(entity)` | `HoverDefinitionDraft`, `HoverEntity` / `DraftHoverResponse` | `HoverState` (generation, offset or entity, card) | card under the field after a 250 ms dwell; entity cards for nodes/rows through the same action | reducer, widget, e2e (concept and mapping cards; the effective world includes the draft) |
| Explain | `explain` | **none** | — | the inspector's *Explain* disclosure shows the protocol's technical fields (ids, interface, Core term, invalidation); `bdl_ide::explain` (dependencies, grant, clock, output relation) **not yet served** | — |
| Revisioned edits, stale refusal, edit classification | `preview_change` | `ApplyEdit`, `edit.stale_revision`, `EditOutcome` | `pendingRequests`, `lastError`, `lastOutcome` | banner; *Last change*; editor keeps the draft | reducer, Dart e2e (refused commit) |
| Causality (cycles, evaluation order) | lifted diagnostics; `explain` dependencies | `causal`, `cycles[]`, `evaluation_order[]` | `analysis` | status line *not causal*; per-mapping row in *Relationship*; Simulate refuses with *The design contains an instantaneous cycle.*; cycles **not drawn on the canvas** | shell, simulation widget |
| Clock domains | lifted diagnostics; `explain` clock; `clock.move_*` fixes | `ClockView`, clock edit ops, `clock.*` | `project.clocks` | library: create, rename in place, delete while unused; mapping/output *Updates in* pop-up (pure = any domain); the domain as a quiet word on the node; `clock.*` findings under *Timing*; `sync(domain, init, e)` in formulas; Simulate: period per domain; status line *reads across domains*; **no domain regions on the canvas** | reducer, widget, e2e (design_e2e), simulation e2e (sync) |
| Sync as an explicit action | `clock.cross_domain_reference` → *Insert explicit sync* is **Blocked**; its reason string in `crates/bdl-ide/src/actions.rs` still says the surface has no `sync` phrase, which is no longer true (`sync(domain, init, e)` elaborates since DI-17) — the action can now plan a draft-text edit; **not done**, and the reason text is stale | via `ListSemanticActions` | `SemanticActionsState` | listed under *Fixes* as blocked with its reason | outputs_test (blocked rendering) |
| Physical outputs | one `output.multiple_drivers` per sink; `explain`; `detach_driver`, `connect_driver`, `create_combination`, `choose_clock` fixes | `OutputView`, `OutputAnalysis`, output edit ops, `Layout.outputs` | `project.outputs`, `analysis.outputs`, `OutputSelected` | canvas sink nodes (boundary bar, dashed while open/undriven, red word when contested), drive links (drag to connect, drag away to disconnect), library, output inspector (accepts, domain, required, driver, claimants, connect/disconnect), mapping *Drives*; status line *outputs incomplete* | reducer, geometry, widget, e2e (undriven → connect via fix → complete; contested → detach via fix) |
| Simulation | — (bdld's reference evaluator) | `Start/Step/ResetSimulation` (unchanged) | `SimulationState`: authored input trace, current values, periods, samples, generation; `simulationBlockers` read off the analysis + projection | Simulate page: readiness blockers (unresolved input, input concept without a value form, invalid or missing definition, instantaneous cycle) each with a *Show* link, Step disabled while any is listed and sending nothing; input controls by value form, period per domain, Step / Step ×10 / Reset, trace table (tick, active domains, values, driven outputs; an input's cell only at ticks its domain activated); probe of the selection with Explain; failures in product words on the controls' line | reducer (trace extension, generation/revision gating, readiness, activation gating), widget (controls, wording, blockers), e2e against bdld (lamp 0/⅓/⅔/1, delay, two-domain sync, unresolved input then a changing quantity, reset, multi-clock activation, invalid design refused before any request, a revision invalidating the run, runtime division by zero), `smart_lamp_e2e` |
| Semantic actions / edit plans | `actions_for` (diagnostics concerning the entity) + `actions_at` (context) | `ListSemanticActions` / `SemanticActionsResponse` (applicability, options, edits as `EditOp`s, invalidation) | `SemanticActionsState`; `queuedEdits` for multi-step plans | *Fixes* section in the mapping and output inspectors: ready → button, needs a choice → pop-up of the service's options, blocked → reason; applied as revisioned edits one confirmed revision at a time | reducer, widget, e2e (connect-driver choice, detach-driver ready) |
| References / navigation by identity | `references`, `definition_of` | **none** (LSP only) | **none** | **none** | bdl-ide acceptance |
| Rename by identity (`plan_rename`) | `SemanticEditPlan` | **none** (LSP `rename` only; Studio renames through `Rename*` edit ops directly) | — | inline rename on the canvas and in the library rows | bdl-ide acceptance; Studio widget tests |
| Deployment analysis | — (bdld only) | `AnalyzeDeployment { target_id, revision? }` → `DeploymentAnalysis` with the read model (0.5): `status`, `design_ready`, `deployable`, `missing[]`, `rows[]`, `blocker` (docs/DEPLOYMENT_READ_MODEL.md) | `DeployState` (target, result, revision) | Deploy page (`deploy_page.dart`): target pop-up, status, per-device rows, dead end — **built on fields 4–9 and re-deriving labels client-side; migrating it to `rows`/`missing`/`blocker` is the next step** | Rust: `deploy_e2e.rs` (10-case matrix), compiler unit, protocol conversion; Dart: `deploy_test.dart` |
| Behaviour systems | bdld: `Session::apply_system`, `apply_group` (with `base_generation`, one history with semantic steps: `system_step`), `preview_extraction`, `system_analysis`, `group_boundaries` (per scope); the project host sees the derived flat design, one `IdeHost` per component body serves component-scoped drafts | `InitSystemProject`, `GetSystem` → `SystemView` (components with contracts and bodies, instances, bindings incl. base ends, exports, groups, boundaries, authoring generation, origins), `ApplySystemEdit` (27 ops), `ApplyGroupEdit`, `PreviewComponentExtraction`, `RunSystemAnalysis` (composition, port statuses, acceptance, projected diagnostics, `component_analyses`), `Layout.instances/groups/components`, draft requests with `component` (0.8) | `AppState.flat` (derived design), `system`, `systemAnalysis`, `project` = the design in view; `EditorState.context` (`SystemContext` \| `ComponentContext`), `layouts: CanvasLayout`, selections for component / instance / port / binding / group, `pendingBind`, `extraction`, `queuedSystemEdits`, drafts stashed per context (`app/system.dart`) | Design page: context bar (*Editing AdaptiveLamp · used by 3 instances*, *‹ System*); system canvas with instance nodes from contracts, binding links (transport gate), realisation sockets on open base relationships, group regions / collapsed boxes with aggregate sockets, drag-to-connect (asks before replacing, asks for an initial value across domains), drag-in/out group membership; library sections Components / Instances / Behaviors (both contexts); click / ⇧-click / ⇧-drag box selection with *Group as Behavior* and inline naming, insertion affordance while dragging over a region, aggregate sockets as proxies (a chooser when several), collapse / move / expand with hidden members travelling, semantic zoom below 0.5×, per-canvas viewports; inspectors for instance (timing, ports, findings, replace), component (promise with contract edits, versions, place instance, clock parameters, sharing, declare port), port (promise, implemented by → *Go to Source*, value, connection), binding, group (relationships, boundary, package, collapse, ungroup / delete); packaging sheet with the compiler's preview and the four choices; bound base relationships shown as *takes its value from …* with *Show Binding*; Simulate and Deploy read the flat design | Rust: `bdl-system/tests/grouping.rs` (§70–§78), `vertical_slice.rs`, `contracts.rs`, `bdl-daemon/tests/system_e2e.rs` (wire incl. group edits, preview, base ends, scoped drafts, layout); Dart: `system_reducer_test.dart` (22), `canvas_gestures_test.dart` (real pointer, no daemon), `system_e2e_test.dart` (the designer flow §79–§82 against bdld), `system_gestures_e2e_test.dart` (the thirteen-step pointer-driven flow against bdld) |
| Target list | — | `ListTargets` → `TargetView { id, display_name, description, family, resource_count, capabilities[] }` (0.5) | `DeployState.targets` | Deploy page target pop-up (shows `name`; `description`/`family`/capability summary not yet shown) | Rust e2e + conversion test |
| Hardware assignment, pins, dead ends | — | `DeviceView` (kind, output, fixed pins, requirement table), device edit ops; `AssignmentRow`/`Blocker` in the read model | `project.devices`, `DeployState` | Deploy page device rows and pin table | Rust e2e |

### Timing (local, debug build)

| Round trip | Measured |
|---|---|
| draft verdict (edit → state), incl. 30 ms test debounce | 36–37 ms (≈ 6 ms without) |
| completion | ~2 ms |
| hover | ~2 ms |
| one simulation step = restart with the trace so far + step | ~1–4 ms; four steps 4–15 ms |
| deployment analysis | < 1 ms |

### What remains, and where it goes

| Fact | Home |
|---|---|
| instantaneous cycles | canvas (the cycle's links emphasised); today the Compiler row and the Simulate refusal |
| clock domains as regions | canvas containment; today a word per node and the inspector pop-up |
| sync as a fix | `clock.cross_domain_reference` → a `DraftText` plan inserting `sync(domain, init, …)` now that the surface has the phrase; the Fixes section already renders any plan the service returns |
| Explain | a `bdl/explain`-style request serving `bdl_ide::explain`; the disclosure exists |
| deployment read model (0.5) | the Deploy page should read `rows[]` / `missing[]` / `blocker` instead of re-deriving from fields 4–9 |
| simulation values on the canvas | Monitor/Simulate: values on sockets, from the same `TickSample`s |
| hover on relationship names inside formulas | `entity_at_formula` indexes concept names only; extend the index to `Lookup::Mapping` |
| entity hover and semantic actions inside a component's source | the IDE services address flat entities; a body-scoped `EntityRef` (component + local id) would let the per-body host answer them |
| packaging a behaviour inside a component | nested components (a body that is itself a system), docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md §13 |

## 4. Non-goals of this milestone

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
