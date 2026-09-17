# Roadmap

Ordered as in the engineering brief; nothing beyond the first vertical
slice is started before the slice works end-to-end.

## Vertical slice (milestone 1)

| Step | | Status |
|---|---|---|
| A | architecture documents, ADRs | ✅ |
| B | Rust project model, stable ids, revisioned edits | ✅ `bdl-model` |
| C | protocol schema | ✅ `bdl-protocol` (now 0.5.0; history in docs/PROTOCOL.md) |
| D | `bdld` process + Flutter connection, handshake, versions in the status bar | ✅ |
| E | crash-safe persistence, save/reopen of an unresolved mapping | ✅ |
| F | Project manager / welcome screen (hero, Start, Recent) and Concepts + Mapping editor: page shell, node canvas (drag, link, unlink, delete), inspector with full concept/mapping editing, macOS look | ✅ (docs/STUDIO_UI.md) |
| — | **architecture review gate** (brief §63) | ⏳ not recorded as held; G–O landed meanwhile |
| G | type / semantic checking: formula parser, elaboration with rep/mk and units, Core typing, Grant, dimension via typing, diagnostics, `analyze()` in bdld, Studio shows status + diagnostics | ✅ (ADR-0013) |
| H | incomplete-declaration support (declared / open / invalid / type-valid) | ✅ |
| I | reference evaluator + simulator: dependency graph, causality, clock judgment, two-phase ticks, state cells, schedules, input traces, traces; bdld Start/Step/Reset | ✅ core; Studio Simulate page ✅ (STUDIO_UI.md §10; ST-4 below) |
| J | `delay` / `sync` state with explicit initial values | ✅ Core; ✅ surface calls `delay(init, e)` / `sync(domain, init, e)` elaborate, simulate and generate (DI-17, `surface_to_backend.rs`); sugar phrases (`previous`, `hold`, …) not parsed (TEXTUAL_SYNTAX.md §12); no canvas mark (STUDIO_UI.md §7) |
| K | output binding + single-driver diagnostic: nominal `PhysicalOutput`, `DriveWF` (exact type + domain), `SingleDriver`, partial validity vs executable completeness, `output_complete` | ✅ `bdl-output`; outputs as canvas sink nodes and inspector objects ✅ (ST-3); Studio Deploy page ✅ (ST-5) |
| L | hardware resource allocator (`bdl-hardware`): capability model, device → requirements, deterministic sound+complete solver, dead-end diagnosis, Nano golden cases, `analyze_deployment` + `AnalyzeDeployment`/`ListTargets` in bdld | ✅ (ADR-0015) |
| M | board descriptions as data: `hardware/boards/arduino_nano.toml`, `big_board.toml` generated + round-tripped | ✅ (RP2040 board file and runtime loading of `hardware/boards/` next) |
| N | Rust code generation via a backend AST + `bdl-manifest.json`: executable IR (`bdl-exec-ir`), reactive lowering (`bdl-lower`), owned Rust AST + printer (`bdl-codegen-rust`), readiness check, `compile()` | ✅ (ADR-0016) |
| O | `no_std` generated core, host execution == interpreter: `runtime/bdl-runtime-core`, `runtime/bdl-runtime-host`, corpus + golden + property differential tests, every generated crate `cargo check`ed and run | ✅ |
| P | first Embassy runtime adapter | |
| BS | behaviour systems: components, interfaces (required/provided/parameter ports, clock parameters), instances with fresh identity, identity-based bindings (direct / transported), flattening with provenance, composition validation, acceptance levels, packaging, system project format, protocol 0.6 | ✅ semantics and protocol (ADR-0017); Studio system canvas next |
| Q | `cargo check` / build orchestration in `bdld` with structured events | |
| R | flash via `probe-rs` | |
| S | telemetry back into Studio | |

## Studio semantic surface

The Studio work after step F, in landing order. Each row is verified in
docs/STUDIO_COMPILER_INTEGRATION.md §3 (the audit table) and
docs/STUDIO_UI.md.

| Step | | Status |
|---|---|---|
| ST-1 | definition editor over compiler-backed drafts (`AnalyzeDefinitionDraft`, protocol 0.4): verdict, spans, add/save/revert/detach, conflicts, stash across close/reopen | ✅ |
| ST-2 | completion pop-up and hover cards from the IDE service (`CompleteDefinitionDraft`, `HoverDefinitionDraft`, `HoverEntity`); semantic actions as *Fixes* (`ListSemanticActions`) | ✅ |
| ST-3 | canvas carries semantics as geometry (hue = identity, shape = value form, dashed = declared); timing domains and physical outputs as first-class objects: library sections, inspector sections, sink nodes with drive links, the domain as a word on the node | ✅ (domain *regions* and cycle emphasis on the canvas: not built) |
| ST-4 | Simulate page over `Start/Step/ResetSimulation`: inputs by value form, period per domain, replayed steps, trace table gated by domain activation, readiness blockers with *Show* links, probe panel with Explain | ✅ |
| ST-5 | Deploy page over `ListTargets` / `AnalyzeDeployment`: target pop-up, verdict, device rows, pin table, dead end | ✅ built on analysis fields 4–9; the 0.5 read model (`rows[]`/`missing[]`/`blocker`) is on the wire and not yet consumed |
| ST-6 | Explain disclosure served by `bdl_ide::explain` (dependencies, grant, clock, output relation) | protocol request not added; the disclosure shows the projection's technical fields only |
| ST-7 | Monitor page (telemetry) | placeholder page (`placeholder_page.dart`); waits on step S |
| ST-8 | Behaviour authoring, grouping, component authoring and system composition (ADR-0019, FV Phase 8b): system canvas with instance nodes from contracts, bindings incl. base ends, group regions and aggregate sockets, packaging sheet, component source with scoped drafts, inspectors for instances / components / ports / bindings / groups | ✅ (not built: entity hover and fixes inside a component's source, nested systems) |
| ST-9 | Scoped behaviour groups and interaction hardening (ADR-0019 amendment): groups in a component's source, one undo history for semantic and authoring steps, stale authoring generations, save-dirty group edits, click/⇧-click/box selection, *Group as Behavior*, drag-in/out with an insertion affordance, aggregate sockets as proxies, collapse/move/expand policy, per-canvas viewports, semantic zoom, pointer-driven e2e | ✅ (deferred: packaging a group inside a component — nested components, docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md §13) |

## Examples

| | Status |
|---|---|
| `examples/smart_lamp` — tilt and ambient light in, one required output driven through `adaptBrightness(dimByTilt(tilt), ambient)`, one PWM device; Design → Simulate → Deploy | ✅ kept faithful by `crates/bdl-compiler/tests/examples.rs` and `apps/studio/test/smart_lamp_e2e_test.dart` |

## IDE service (shared language-service layer)

| Step | | Status |
|---|---|---|
| IDE-1 | `bdl-ide-db` (IdeHost, overlays, EntityRef, projections, stamped snapshots, cancellation), `bdl-ide` (diagnostics, hover/explain, completion, references, rename, actions, edit plans, invalidation preview, symbols, tokens), `bdl-lsp` MVP (hover, definition, references, rename, completion, pull diagnostics, symbols, semantic tokens, code actions); Studio drafts and LSP buffers as overlays; ADR-0017 | ✅ (docs/IDE_SERVICE_ARCHITECTURE.md) |
| IDE-2 | complete textual parser + textual project loader (persisted textual identities, outputs/clocks/devices syntax, parameter layer DI-30) | |
| IDE-3 | richer LSP: inlay hints, workspace symbols, explain/Core/Rust virtual documents, formatting, cross-surface actions applying model operations from editors | |
| IDE-4 | incremental query engine — only if profiling on real projects asks for it (baseline: ~2 ms per full analysis at 400 mappings) | |

## Standard Concept Library

| Step | | Status |
|---|---|---|
| LIB-1 | `library/std/concepts.toml` (36 templates, schema 1, library 0.1), `bdl-library` (load, validate, search, one instantiation), shared quantity vocabulary in `bdl-model`, protocol 0.5 (`ListConceptTemplates`, `InstantiateConceptTemplate`), Studio Library tab + right-click quick insert + drag-and-drop + create-then-rename, textual completion of templates | ✅ (docs/STANDARD_CONCEPT_LIBRARY.md) |
| LIB-2 | team / project / package libraries (discovery rule over `Library::from_toml`), template icons | |
| LIB-3 | device library, separate: device packages that *provide* concepts and generate requirements | |

## Explicitly deferred (brief §52)

AI assistant · plugin marketplace · animation polish · cloud sync ·
collaboration · full StateHandler editor · arbitrary firmware backends ·
SMT · full numeric physical validation · code sandbox · component
marketplace · custom debugger.

## Known v0.1 simplifications to revisit

* `ProjectChanged` sends the full projection; deltas planned (PROTOCOL.md).
* Revision/undo history is session-only; a persisted edit log is planned.
* Second embedded target (ESP32-S3) after RP2040 to prove HAL independence.
* Native menu bar (`PlatformMenuBar` on macOS, in-window on Windows).
