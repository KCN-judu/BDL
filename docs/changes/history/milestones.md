# Completed milestones

*The implementation ledger that was `docs/project/roadmap.md` until 2026-09-17,
kept as history so the roadmap can hold only what is next. Each row is a
milestone as it was recorded when it landed, with the evidence it cited.
For what exists today, read `docs/project/status.md`; for why,
the ADR named in the row. Row labels (A–O, BS, ST-n, IDE-n, LIB-n) are
the identifiers the commit history and older documents use.*

## Vertical slice (milestone 1), 2026-09-15

| Step | Landed | Evidence / record |
|---|---|---|
| A | architecture documents, ADRs | `docs/architecture/overview.md`, ADR-0001…0012 |
| B | Rust project model, stable ids, revisioned edits | `bdl-model`; ADR-0008, ADR-0009 |
| C | protocol schema | `bdl-protocol` (0.1; [protocol versions](protocol-versions.md)) |
| D | `bdld` process + Flutter connection, handshake, versions in the status bar | ADR-0002, ADR-0007 |
| E | crash-safe persistence, save/reopen of an unresolved mapping | `docs/spec/project-format.md`; `crates/bdl-daemon/tests/stdio_e2e.rs` |
| F | project manager / welcome screen and Concepts + Mapping editor: page shell, node canvas (drag, link, unlink, delete), inspector, macOS look | `docs/architecture/studio-ui.md`; ADR-0012 |
| — | the architecture review gate the engineering brief asked for (§63) was **not recorded as held**; G–O landed meanwhile | — |
| G | type / semantic checking: formula parser, elaboration with rep/mk and units, Core typing, Grant, dimension via typing, diagnostics, `analyze()` in bdld, Studio shows status + diagnostics | ADR-0013; `docs/architecture/compiler-pipeline.md` |
| H | incomplete-declaration support (declared / open / invalid / type-valid) | `docs/architecture/compiler-pipeline.md` |
| I | reference evaluator + simulator: dependency graph, causality, clock judgment, two-phase ticks, state cells, schedules, input traces, traces; bdld Start/Step/Reset; Studio Simulate page | `docs/spec/runtime-semantics.md`; ST-4 |
| J | `delay` / `sync` state with explicit initial values, in Core and as surface calls `delay(init, e)` / `sync(domain, init, e)` that elaborate, simulate and generate | DI-17; `crates/bdl-compiler/tests/surface_to_backend.rs`. Sugar phrases (`previous`, `hold`, …) are not parsed (`docs/spec/textual-syntax.md` §12); no canvas mark (`docs/architecture/studio-ui.md` §7) |
| K | output binding + single-driver diagnostic: nominal `PhysicalOutput`, `DriveWF`, `SingleDriver`, partial validity vs executable completeness, `output_complete`; outputs as canvas sink nodes and inspector objects; Studio Deploy page | `bdl-output`; ST-3, ST-5 |
| L | hardware resource allocator: capability model, device → requirements, deterministic sound+complete solver, dead-end diagnosis, Nano golden cases, `analyze_deployment` + `AnalyzeDeployment`/`ListTargets` | ADR-0015; `docs/spec/hardware-model.md` |
| M | board descriptions as data: `hardware/boards/arduino_nano.toml`, `big_board.toml` generated + round-tripped | `docs/spec/hardware-model.md` |
| N | Rust code generation via a backend AST + `bdl-manifest.json`: executable IR (`bdl-exec-ir`), reactive lowering (`bdl-lower`), owned Rust AST + printer (`bdl-codegen-rust`), readiness check, `compile()` | ADR-0016; `docs/architecture/codegen-rust.md` |
| O | `no_std` generated core, host execution == interpreter: `runtime/bdl-runtime-core`, `runtime/bdl-runtime-host`, corpus + golden + property differential tests, every generated crate `cargo check`ed and run | `docs/architecture/codegen-rust.md`; `crates/bdl-compiler/tests/backend_*` |
| BS | behaviour systems: components, interfaces (required/provided/parameter ports, clock parameters), instances with fresh identity, identity-based bindings (direct / transported), flattening with provenance, composition validation, acceptance levels, packaging, system project format, protocol 0.6 | ADR-0021; FV Phase 8a; `crates/bdl-system/tests/vertical_slice.rs` |

Not started when this ledger closed: P (first Embassy runtime adapter),
Q (`cargo check` / build orchestration in `bdld`), R (flash via
`probe-rs`), S (telemetry back into Studio) — now the roadmap's first
four priorities.

## Studio semantic surface, 2026-09-15 → 2026-09-17

Each row was verified in `docs/architecture/studio-compiler-integration.md` §3 and
`docs/architecture/studio-ui.md`.

| Step | Landed | Evidence / record |
|---|---|---|
| ST-1 | definition editor over compiler-backed drafts (`AnalyzeDefinitionDraft`, protocol 0.4): verdict, spans, add/save/revert/detach, conflicts, stash across close/reopen | [formula-editing milestone](formula-editing-milestone.md) |
| ST-2 | completion pop-up and hover cards from the IDE service (`CompleteDefinitionDraft`, `HoverDefinitionDraft`, `HoverEntity`); semantic actions as *Fixes* (`ListSemanticActions`) | `apps/studio/test/tooling_test.dart` |
| ST-3 | canvas carries semantics as geometry (hue = identity, shape = value form, dashed = declared); timing domains and physical outputs as first-class objects: library sections, inspector sections, sink nodes with drive links, the domain as a word on the node. Domain *regions* and cycle emphasis on the canvas: not built | ADR-0018; `docs/architecture/studio-ui.md` |
| ST-4 | Simulate page over `Start/Step/ResetSimulation`: inputs by value form, period per domain, replayed steps, trace table gated by domain activation, readiness blockers with *Show* links, probe panel with Explain | `apps/studio/test/simulation_test.dart` |
| ST-5 | Deploy page over `ListTargets` / `AnalyzeDeployment`: target pop-up, verdict, device rows, pin table, dead end — built on analysis fields 4–9; the 0.5 read model (`rows[]`/`missing[]`/`blocker`) is on the wire and not consumed | `docs/architecture/deployment-read-model.md`; `apps/studio/test/deploy_test.dart` |
| ST-6 | Explain disclosure — the protocol request was not added; the disclosure shows the projection's technical fields only | — |
| ST-7 | Monitor page — a placeholder (`placeholder_page.dart`); waits on telemetry (step S) | — |
| ST-8 | behaviour authoring, grouping, component authoring and system composition: system canvas with instance nodes from contracts, bindings incl. base ends, group regions and aggregate sockets, packaging sheet, component source with scoped drafts, inspectors for instances / components / ports / bindings / groups. Not built: entity hover and fixes inside a component's source, nested systems | ADR-0019; FV Phase 8b; `apps/studio/test/system_e2e_test.dart` |
| ST-9 | scoped behaviour groups and interaction hardening (ADR-0019 amendment): groups in a component's source, one undo history for semantic and authoring steps, stale authoring generations, save-dirty group edits, click/⇧-click/box selection, *Group as Behavior*, drag-in/out with an insertion affordance, aggregate sockets as proxies, collapse/move/expand policy, per-canvas viewports, semantic zoom, pointer-driven e2e. Deferred: packaging a group inside a component (ISS-0007) | `apps/studio/test/system_gestures_e2e_test.dart` |

## Examples

| | Evidence |
|---|---|
| `examples/smart_lamp` — tilt and ambient light in, one required output driven through `adaptBrightness(dimByTilt(tilt), ambient)`, one PWM device; Design → Simulate → Deploy | kept faithful by `crates/bdl-compiler/tests/examples.rs` and `apps/studio/test/smart_lamp_e2e_test.dart` |

## IDE service, 2026-09-15 → 2026-09-17

| Step | Landed | Evidence / record |
|---|---|---|
| IDE-1 | `bdl-ide-db` (IdeHost, overlays, EntityRef, projections, stamped snapshots, cancellation), `bdl-ide` (diagnostics, hover/explain, completion, references, rename, actions, edit plans, invalidation preview, symbols, tokens), `bdl-lsp` MVP (hover, definition, references, rename, completion, pull diagnostics, symbols, semantic tokens, code actions); Studio drafts and LSP buffers as overlays | ADR-0017; `docs/architecture/ide-service.md` |
| IDE-2 | complete textual parser (v0.2 project items) and textual project loader with persisted identities and a lexical parameter layer | ADR-0020; [2026-09 text projects](../unreleased/2026-09-text-projects.md) |
| IDE-3 | richer LSP: inlay hints, explain/Core/Rust virtual documents, formatting; a reference VS Code extension. Not built: workspace symbols, cross-surface actions applying model operations from editors | same |

IDE-4 (an incremental query engine, only if profiling asks for it) stays
on the roadmap as a conditional item.

## Standard Concept Library, 2026-09-15

| Step | Landed | Evidence / record |
|---|---|---|
| LIB-1 | `library/std/concepts.toml` (36 templates, schema 1, library 0.1), `bdl-library` (load, validate, search, one instantiation), shared quantity vocabulary in `bdl-model`, protocol 0.5 (`ListConceptTemplates`, `InstantiateConceptTemplate`), Studio Library tab + right-click quick insert + drag-and-drop + create-then-rename, textual completion of templates | `docs/spec/concept-library.md` |

LIB-2 (team / project / package libraries, template icons) and LIB-3 (a
separate device library) are roadmap items.
