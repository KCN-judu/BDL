# Implementation status

What exists at repository HEAD, per area. *Implemented* means present
and exercised by the named test; it does not mean released, and it never
means formally proved (see [FORMAL_CORRESPONDENCE.md](FORMAL_CORRESPONDENCE.md)
for that). Updated when work lands or is removed; the
[roadmap](../ROADMAP.md) says what is next, the
[change records](../changes/README.md) say what changed and when.

**Snapshot:** 2026-09-17, after the textual-authoring milestone
(commits `25a46a2`…`d606405`).

| Area | State | Exists | Does not exist | Evidence |
|---|---|---|---|---|
| Language core | implemented | typing, semantic identity, dimensions, grants, causality, clock domains, `delay`/`sync`, physical outputs, single driver, completeness; formula language v0 with calls, `let`, `if`, `match` over Bool/Option/numbers | user enums (ISS-0005), affine units (ISS-0004), temporal modifiers and contexts (ISS-0010), occurrence windows (ISS-0001) | `docs/COMPILER_PIPELINE.md`; `crates/bdl-compiler/tests/surface_expressions.rs`, `surface_to_backend.rs` |
| Formal correspondence | partial | kernel Phases 1–7 and behaviour Phases 8a–8b transcribed and discharged by tests; strength recorded per claim | production is not verified; Phase 9a (buffers) not mirrored | `docs/02-kernel-spec.md`, `docs/BEHAVIOR_SYSTEMS.md`, [FORMAL_CORRESPONDENCE.md](FORMAL_CORRESPONDENCE.md) |
| Compiler and simulator | implemented | analysis with product-language diagnostics, reference evaluator, schedules, input traces, explain | — | `docs/EXECUTION_WALKTHROUGH.md`; `crates/bdl-compiler/tests/` |
| Rust backend | implemented | executable IR, reactive lowering, owned Rust AST, `no_std` core + host bridge, `bdl-manifest.json`, differential/golden/property tests | — | `docs/CODEGEN_RUST.md`; `crates/bdl-compiler/tests/backend_*.rs` |
| Hardware and deployment analysis | implemented | capability model, device → requirements, sound and complete allocator, dead-end diagnosis, board files (Nano, big board), target-relative Deploy read model | RP2040 board file; runtime loading of `hardware/boards/`; electrical/numeric constraints (DI-22, out of scope) | `docs/HARDWARE_MODEL.md`, `docs/DEPLOYMENT_READ_MODEL.md`; `crates/bdl-daemon/tests/deploy_e2e.rs` |
| Embedded runtime | planned | target-independent `no_std` core and host harness | any platform adapter (Embassy), build orchestration, flash, telemetry — roadmap priorities 1–4 | `docs/RUNTIME_SEMANTICS.md`; `runtime/` |
| Behaviour systems | implemented | components, stored port contracts, instances with fresh identity, direct and transported bindings, flattening with provenance, composition validation, packaging, versions and substitution, groups and boundaries, system project format | nested packaging inside a component (ISS-0007) | `docs/BEHAVIOR_SYSTEMS.md`, `docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md`; `crates/bdl-system/tests/`, `apps/studio/test/system_e2e_test.dart` |
| Studio | partial | Design (canvas, inspector, compiler-backed definition editor with completion/hover/fixes, domains and outputs as objects, system canvas with groups, packaging, component source), Simulate, Deploy, Library, text projects with conflict banner | Monitor page (placeholder); Explain over a protocol request; Deploy page on the 0.5 read model; domain regions and cycle emphasis on the canvas; entity hover and fixes inside a component's source; native menu bar — the detailed list is `docs/STUDIO_COMPILER_INTEGRATION.md` §3 *What remains* | `docs/STUDIO_UI.md`, `docs/STUDIO_COMPILER_INTEGRATION.md` §3; `apps/studio/test/*_e2e_test.dart` |
| Textual frontend | implemented | lossless syntax v0.1 + v0.2 project items, text projects (`kind = "text"`), stable identities with reconciliation, item-level write-back keeping comments, lexical parameter names, `bdld check/compile/simulate` | user enums are parsed and reported open (ISS-0005) | `docs/TEXTUAL_SYNTAX.md`, `docs/PROJECT_FORMAT.md`; `crates/bdl-text/tests/workspace.rs`, `crates/bdl-daemon/tests/{text_e2e,cli}.rs` |
| IDE service and LSP | implemented | overlays and text workspaces, diagnostics, hover/explain, scope-aware completion, definition/references/rename across files, code actions, symbols, semantic tokens, formatting, inlay hints, explain/Core/Rust virtual documents, cancellation; VS Code reference extension | workspace symbols; cross-surface actions applying model operations from editors; incremental query engine (only if profiling asks) | `docs/IDE_SERVICE_ARCHITECTURE.md`; `crates/bdl-lsp/tests/{e2e,text_workspace}.rs`; `editors/vscode` |
| Protocol and daemon | implemented | protobuf over framed stdio, protocol 0.9, sessions for flat/system/text projects, drafts, simulation, deployment, libraries | projection deltas and a persisted edit history (ISS-0009); `Entity::Output` (ISS-0008) | `docs/PROTOCOL.md`; `crates/bdl-daemon/tests/` |
| Standard concept library | implemented | `library/std/concepts.toml`, load/validate/search/instantiate, Studio Library tab, textual completion | team/project/package libraries (LIB-2); device library (LIB-3) | `docs/STANDARD_CONCEPT_LIBRARY.md`; `crates/bdl-library` |
| Supplied Rust components | planned | the trust boundary and obligations are designed | any implementation | `docs/COMPONENT_BOUNDARY.md`; ADR-0005 |

Known code-side drifts recorded while auditing (fix in code, not docs):
`crates/bdl-ide/src/actions.rs` *Insert explicit sync* reason string still
says the surface has no `sync` phrase; `crates/bdl-compiler/tests/examples.rs`
doc comment names the wrong crate.
