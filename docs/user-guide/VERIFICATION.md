# Verification matrix

Every page of the user guide was checked against the product at one
commit. A row names the page, the behaviour it claims, and the evidence —
a source file whose labels or logic were read, or a test that exercises
the workflow. When the product moves, re-verify the rows that cite the
changed files and update the commit column.

**Verified at:** `36f966f` (main, 2026-09-17).

Evidence paths are relative to the repository root. *Studio* means
`apps/studio/lib`, *test* means `apps/studio/test`.

## Getting started

| Page | Claim | Evidence |
|---|---|---|
| `install-and-launch.md` | toolchain versions; `just studio`; dialog fallback | `README.md`, `justfile`, `docs/GETTING_STARTED.md`; `Studio/ui/welcome/welcome_page.dart` (*Open by path…*, *New at path…*) |
| | project manager actions: *New Project…*, *New System…*, *Open Project…*, Recent | `Studio/ui/welcome/welcome_page.dart` |
| | status line connection words | `Studio/ui/shell.dart` (*Compiler …*, *Connecting to the compiler*, *Compiler not connected*) |
| | no flat → system conversion | no such request in `crates/bdl-protocol/proto/bdl/v1/bdl.proto`; `NewProjectRequested(system:)` only |
| `first-behavior.md` | concept sheet fields and value forms; unit column | `Studio/ui/dialogs.dart` (*New concept*, *Quantity*, *On / off*, *Count*, *Decide later*, *Unit*); `Studio/ui/units.dart` |
| | mapping sheet: *Reads*, *Produces*, Create disabled without Produces | `Studio/ui/dialogs.dart`; `docs/STUDIO_UI.md` §5 |
| | dashed *declared* node; red mark at formula line | `Studio/ui/canvas/node_canvas.dart`; `docs/STUDIO_UI.md` §2 |
| | draft verdict as you type; *Add definition* / ⌘↩; type-mismatch message | `Studio/ui/definition_editor.dart`; `crates/bdl-elab/src/formula.rs` (`realization.type_mismatch` wording); `test/definition_editor_test.dart`, `test/formula_e2e_test.dart` |
| | completion ⌃Space, hover card | `Studio/ui/definition_editor.dart`; `test/tooling_test.dart` |
| | nullary undefined mapping = simulation input; `dimByTilt(tilt)` application | `crates/bdl-compiler/tests/surface_to_backend.rs`; `examples/smart_lamp/design/project.bdl.json`; `docs/DESIGN_ISSUES.md` DI-17 |
| | timing domain creation and *Updates in*; *Any timing domain* | `Studio/ui/library.dart` (*Timing domains*), `Studio/ui/inspector.dart` (*Updates in*), `Studio/ui/system_inspector.dart` (*Any timing domain*); `test/outputs_test.dart` |
| | output sheet (*Accepts*, *Updates in*, *Required*); sink node; drag to connect; Connect pop-up marks *has inputs* | `Studio/ui/dialogs.dart`, `Studio/ui/inspector.dart` (Connect dropdown), `Studio/ui/canvas/canvas_geometry.dart` (`canLink`); `test/outputs_test.dart`, `test/design_e2e_test.dart` |
| | only nullary same-domain driver fits; otherwise reported | `crates/bdl-output/src/lib.rs` (`output.type_mismatch`, `output.clock_mismatch`) |
| `first-simulation.md` | page regions; input in base units; period *every N*; Step / Step ×10 / Reset | `Studio/ui/pages/simulate_page.dart`; `Studio/ui/canvas/canvas_geometry.dart` (`dimLabel`) |
| | readiness sentences with *Show*; Step disabled | `Studio/app/simulation.dart` (`simulationBlockers`); `test/simulation_test.dart` ("readiness names the object…") |
| | values rendered `Brightness(0.5)`; lamp 0/⅓/⅔/1 | `test/simulation_test.dart` ("lamp: Tilt 0, 30, 60, 90 deg…") |
| | replay semantics; reset keeps inputs; new revision drops samples | `Studio/app/simulation.dart`; `test/simulation_test.dart` (reducer group) |
| | column header selects; probe with Explain | `Studio/ui/pages/simulate_page.dart` (`_Probe`, header `onTap`) |
| | inputs shown only on activation ticks | `test/simulation_test.dart` ("an input is shown only at ticks where its domain activated", "multi-clock…") |
| `first-deployment.md` | Target pop-up, board names, verdict wording, *Add device*, kinds, pin field, dead end wording | `Studio/ui/pages/deploy_page.dart`; `hardware/boards/*.toml`; `crates/bdl-hardware/src/devices.rs`; `test/deploy_test.dart`, `crates/bdl-daemon/tests/deploy_e2e.rs` |
| | board is a session preference; deployment never changes semantic analysis | `docs/STUDIO_COMPILER_INTEGRATION.md` §4; `crates/bdl-daemon/tests/deploy_e2e.rs` (case 10) |
| | *feasible* excludes electrical constraints; first dead end only | `docs/DESIGN_ISSUES.md` DI-21, DI-22 |

## Concepts

| Page | Claim | Evidence |
|---|---|---|
| `concepts.md` | nominal identity; hue per concept; link only between same concept | `Studio/ui/canvas/canvas_geometry.dart` (`canLink`); `crates/bdl-elab/src/formula.rs` (`formula.call.argument_type`) |
| | value forms and quantity kinds | `crates/bdl-model/src/quantity.rs`; `Studio/ui/units.dart` |
| | *Decide later*; *Checked once … is decided.*; refinement vs edit on representation change | `Studio/ui/inspector.dart`; `crates/bdl-model/src/edit.rs` (classification); `docs/STUDIO_UI.md` §4 |
| | library templates are not identities | `docs/STANDARD_CONCEPT_LIBRARY.md`; `crates/bdl-library` tests |
| `relationships.md` | signature-first; three shapes; only nullary drives | `crates/bdl-model/src/surface.rs`; `docs/DESIGN_ISSUES.md` DI-20 |
| | link drag edits the signature; drag away disconnects | `Studio/ui/canvas/node_canvas.dart`; `docs/STUDIO_UI.md` §2 |
| `incomplete-designs.md` | states and their marks; status-line phrases | `Studio/ui/shell.dart` (*not yet defined*, *outputs incomplete*, *not causal*, *reads across domains*); `docs/COMPILER_PIPELINE.md` (ladder) |
| | Simulate blockers | `Studio/app/simulation.dart` |
| | required vs optional output; contested is an error | `crates/bdl-output/src/lib.rs` |
| `timing.md` | domain is a name, not a rate; period is a schedule | `docs/RUNTIME_SEMANTICS.md`; `docs/02-kernel-spec.md` |
| | agnostic declarations evaluated whenever anything is active | `docs/RUNTIME_SEMANTICS.md`; `crates/bdl-reactive/src/eval.rs` |
| | `delay` / `sync` semantics and placement rules | `docs/TEXTUAL_SYNTAX.md` §11.1; `crates/bdl-elab/src/formula.rs` (`formula.temporal.*`); `crates/bdl-compiler/tests/surface_to_backend.rs`; `test/simulation_test.dart` (delay, sync) |
| | cross-domain read finding wording | `crates/bdl-reactive/src/clocks.rs` (`clock.cross_domain_reference`) |
| `physical-outputs.md` | rules, states, inspector lines | `crates/bdl-output/src/lib.rs`; `Studio/ui/inspector.dart` (*Undriven — …*, *Driven by …*, *Still driven by …*); `Studio/ui/canvas/canvas_geometry.dart` (`SinkState`) |
| | fixes offered | `crates/bdl-ide/src/actions.rs` (detach, combination, connect driver) |
| `behavior-groups.md` | system projects only; membership rules; not a revision; undo covers; dirty | `Studio/ui/pages/design_page.dart` (`groupsEnabled: state.isSystem`); `test/system_e2e_test.dart` (grouping section); `test/system_reducer_test.dart` ("a group edit dirties the project without a revision…"); `docs/adr/0019-*.md` |
| | boundary is computed; aggregate sockets accept no link | `crates/bdl-system/tests/grouping.rs` (`boundary_is_a_projection_and_sockets_add_no_dependency`); `test/system_e2e_test.dart` ("the aggregate socket accepts no link") |
| | semantic transparency | `crates/bdl-system/tests/grouping.rs` (`grouping_is_semantically_transparent`) |
| | menu and inspector labels | `Studio/ui/canvas/node_canvas.dart`, `Studio/ui/system_inspector.dart` |
| `components.md` | port kinds, stored contracts, *Realizes* once per component | `crates/bdl-system/tests/contracts.rs`; `docs/adr/0018-a-component-interface-is-a-stored-promise.md` |
| | shared vs private concepts | `crates/bdl-system/tests/contracts.rs` (`equal_representations_are_not_equal_concepts`); `docs/BEHAVIOR_SYSTEMS.md` |
| | *Declare a port* in the component inspector; *Edit Source*; context bar text | `Studio/ui/system_inspector.dart`; `Studio/ui/pages/design_page.dart` |
| | source edit reaches every instance | `test/system_e2e_test.dart` ("the body edit reached the instance") |
| | not portable, not nestable | `docs/BEHAVIOR_SYSTEMS.md` (Limitations); `docs/DESIGN_ISSUES.md` DI-44 |
| `behavior-systems.md` | bindings by identity; fan-out; no silent replace; transport sheet | `test/system_e2e_test.dart` (§78/§80 section); `test/system_reducer_test.dart` ("a taken destination asks…", "different timing domains ask for an initial value…") |
| | open port = open, simulated as input | `crates/bdl-system/tests/vertical_slice.rs` (`an_unbound_required_port_is_open_not_invalid`); `docs/BEHAVIOR_SYSTEMS.md` |
| | one design to simulator/deploy; `lampA.brightness` naming | `test/system_e2e_test.dart` (`simulateIndicator` over `flatId`); `docs/BEHAVIOR_SYSTEMS.md` |
| | two instances driving one output → `output.multiple_drivers` | `test/system_e2e_test.dart` (end of test) |
| | restricted formal claims | `docs/BEHAVIOR_SYSTEMS.md` (Theorems J, R marked restricted) |

## Studio

| Page | Claim | Evidence |
|---|---|---|
| `workspace.md` | regions; *Edited* beside the title; Save enabled only when dirty; page shortcuts ⌘1–⌘4; banner with *Dismiss*; settings gear reconnects only | `Studio/ui/shell.dart` |
| | drafts survive selection/page/close | `Studio/app/drafts.dart`; `test/draft_reducer_test.dart` ("close and reopen") |
| `canvas.md` | encodings; gestures; context menu items | `Studio/ui/canvas/node_canvas.dart` (`_onDoubleTapDown`, `_menuItems`, key handling), `Studio/ui/canvas/canvas_geometry.dart`; `test/canvas_geometry_test.dart`, `test/canvas_gestures_test.dart` |
| | Library drag-and-drop | `Studio/ui/concept_library_panel.dart` (`Draggable`), `Studio/ui/canvas/node_canvas.dart` (`DragTarget`); `test/concept_library_test.dart` |
| `library.md` | Project tab sections and sheets; system-project sections; component glyph | `Studio/ui/library.dart` |
| | Library tab rows, double-click / drag insert, create-then-rename, recents | `Studio/ui/concept_library_panel.dart`; `test/concept_library_test.dart`, `test/concept_library_e2e_test.dart` |
| `inspector.md` | sections and labels per object | `Studio/ui/inspector.dart` (string table); `Studio/ui/semantic_actions.dart` (*Fixes* in mapping and output inspectors) |
| `formula-editor.md` | verdict line words, buttons, keys, conflict notice | `Studio/ui/definition_editor.dart`; `test/definition_editor_test.dart` |
| | invalid formula may be saved | `docs/STUDIO_COMPILER_INTEGRATION.md` §1 ("Committing an invalid definition") |
| | where committed findings appear | `Studio/ui/inspector.dart` (`timingIssues`, `driveIssues`, `broader`) |
| `simulate.md` | as `first-simulation.md`; failure wording | `Studio/ui/pages/simulate_page.dart` (`simulation.*` → wording) |
| `deploy.md` | as `first-deployment.md`; page never restates design validity | `Studio/ui/pages/deploy_page.dart` (`_Verdict` doc comment) |
| | codegen exists but has no Studio surface | `docs/ROADMAP.md` (N, O done; Q, R open); no compile request in `bdl.proto` |
| `system-projects.md` | contexts, bar text, instance node anatomy, gestures, inspectors, packaging sheet | `Studio/ui/pages/design_page.dart` (`_ContextBar`), `Studio/ui/system_inspector.dart`, `Studio/ui/system_sheets.dart`, `Studio/ui/canvas/node_canvas.dart`; `test/system_e2e_test.dart`, `test/system_gestures_e2e_test.dart`; `docs/STUDIO_UI.md` §11 |
| | not built list | `docs/STUDIO_UI.md` §11 "Not built"; `docs/DESIGN_ISSUES.md` DI-40, DI-44 |

## Workflows

| Page | Claim | Evidence |
|---|---|---|
| `sensor-to-output.md` | the Smart Lamp design and values | `examples/smart_lamp/design/project.bdl.json`; `crates/bdl-compiler/tests/examples.rs`; `test/smart_lamp_e2e_test.dart` |
| | `if` over a comparison with a unit; `chooseBrightness` pattern | `docs/TEXTUAL_SYNTAX.md` §1, §11.1; `crates/bdl-compiler/tests/surface_expressions.rs` |
| `multi-output-behavior.md` | contested output, fixes, required vs optional | `crates/bdl-output/src/lib.rs`; `crates/bdl-ide/src/actions.rs`; `test/design_e2e_test.dart` (contested → detach via fix) |
| `grouping-behavior.md` | every step | `test/system_e2e_test.dart` (grouping, boundary, collapsed sections); `test/system_reducer_test.dart` ("a multi-selection groups only the free relationships…", "a group created from the canvas is selected and opens for naming…") |
| `package-as-component.md` | preview, choices, one edit, same trace, member identity kept, source edit reaches instance | `test/system_e2e_test.dart` (§75 packaging section); `crates/bdl-system/tests/grouping.rs` (`extraction_is_a_differential_witness_of_theorem_r`, `an_open_member_is_an_input_by_default_or_stays_open_inside`, `a_driven_member_keeps_its_drive_and_the_sink_stays_external`) |
| `composing-components.md` | second instance, clock argument, fan-out, replace sheet, contested light | `test/system_e2e_test.dart` (§78/§80 section, end of test); `crates/bdl-system/tests/grouping.rs` (`fan_out_delete_and_transport_coexist`) |
| `cross-domain-transport.md` | `sync` in a formula; trace values; binding sheet *Starts at*; gate on link | `test/simulation_test.dart` (sync case); `test/system_e2e_test.dart` (transport section); `test/system_reducer_test.dart` ("a bound copy shows what it takes and a transport is a gate") |
| `component-versioning.md` | *Duplicate as Version*, *Replace with*, port retirement refusals, `not_substitutable`, `instance_in_use`, contract change classified, undo | `test/system_e2e_test.dart` (§80 versions, §79 contract sections); `crates/bdl-system/tests/contracts.rs` (`a_refining_version_substitutes_an_incompatible_one_is_refused`, `a_contract_change_is_explicit_and_classified`) |

## Textual

| Page | Claim | Evidence |
|---|---|---|
| `overview.md` | parsing scope; overlay semantics; no textual project; no syntax for domains/outputs/devices; parameter-name limitation; enums open | `docs/TEXTUAL_SYNTAX.md` §11–12; `docs/IDE_SERVICE_ARCHITECTURE.md` ("Textual surface today", "Overlays"); `crates/bdl-ide-db/src/textual.rs`; `docs/DESIGN_ISSUES.md` DI-19, DI-30 |
| | same verdict on both surfaces | `crates/bdl-ide/tests/surface_equivalence.rs` |
| `syntax-basics.md` | grammar and the checked/unchecked matrix; type names | `docs/TEXTUAL_SYNTAX.md` §4–6, §11.1; `crates/bdl-ide-db/src/textual.rs` (`representation_named`); `crates/bdl-syntax/test_data/valid/*.bdl` |
| `editor-and-lsp.md` | capabilities, root resolution, full sync, encodings, pull diagnostics with push fallback, custom requests, disabled model-only actions | `crates/bdl-lsp/src/server.rs` (`ServerCapabilities`, `InitializationOptions`, `open_host`), `crates/bdl-lsp/src/convert.rs` (`code_action`, `workspace_edit`, token legend); `crates/bdl-lsp/tests/e2e.rs` |

## Troubleshooting and reference

| Page | Claim | Evidence |
|---|---|---|
| `troubleshooting/*.md` | every quoted message | `crates/bdl-elab/src/formula.rs`, `crates/bdl-check/src/pretty.rs`, `crates/bdl-reactive/src/{causality,clocks}.rs`, `crates/bdl-output/src/lib.rs`, `crates/bdl-compiler/src/*.rs` (deploy), `crates/bdl-system/src/*.rs` (system, component), `crates/bdl-daemon/src/server.rs` (error codes), `crates/bdl-model/src/edit.rs` (edit errors), `Studio/app/simulation.dart`, `Studio/ui/pages/{simulate,deploy}_page.dart` |
| `timing-errors.md` | *Insert explicit sync* is blocked | `crates/bdl-ide/src/actions.rs` (`Applicability::Blocked`) |
| `reference/terminology.md` | Studio labels | the string tables of `Studio/ui/*.dart` |
| `reference/keyboard-and-mouse.md` | bindings; Windows bindings inactive; ⌘N/⌘O labels only | `Studio/ui/shell.dart` (`CallbackShortcuts`, `meta: true`), `Studio/ui/definition_editor.dart`, `Studio/ui/canvas/node_canvas.dart`, `Studio/ui/welcome/welcome_page.dart`, `Studio/platform/desktop.dart` |
| `reference/status-meanings.md` | every phrase | `Studio/ui/shell.dart`, `Studio/ui/inspector.dart`, `Studio/ui/definition_editor.dart`, `Studio/ui/system_inspector.dart`, `Studio/ui/pages/*.dart` |
| `reference/formula-language.md` | operators, units, forms, placement | `docs/TEXTUAL_SYNTAX.md` §5–6, §11.1; `crates/bdl-elab/src/units.rs`; `crates/bdl-ir/src/expr.rs` (`Prim`) |
| `reference/project-files.md` | file layout, contents, what is not saved, atomic writes, migration | `docs/PROJECT_FORMAT.md`; `crates/bdl-model/src/persist.rs`; `crates/bdl-system/src/persist.rs` |

## Known gaps recorded while verifying

Facts the guide states as limitations, so that a later pass can lift
them when the product changes:

* Windows shortcut bindings are declared with the Command modifier
  (`meta: true`) although labelled Ctrl — documented as "not yet active"
  in `reference/keyboard-and-mouse.md`.
* *New Project…* / *Open Project…* show ⌘N / ⌘O but have no binding.
* The *Insert explicit sync* fix is offered as blocked with a reason that
  predates `sync` in formulas.
* The Deploy page does not consume the deployment read model
  (`rows`/`missing`/`blocker`) and does not restate design readiness.
* Monitor is a placeholder; no firmware build, flash or telemetry.
* A plain project cannot become a system project.
