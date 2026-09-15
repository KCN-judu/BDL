# Roadmap

Ordered as in the engineering brief; nothing beyond the first vertical
slice is started before the slice works end-to-end.

## Vertical slice (milestone 1)

| Step | | Status |
|---|---|---|
| A | architecture documents, ADRs | ✅ |
| B | Rust project model, stable ids, revisioned edits | ✅ `bdl-model` |
| C | protocol schema | ✅ `bdl-protocol` (0.1.0) |
| D | `bdld` process + Flutter connection, handshake, versions in the status bar | ✅ |
| E | crash-safe persistence, save/reopen of an unresolved mapping | ✅ |
| F | Project manager / welcome screen (hero, Start, Recent) and Concepts + Mapping editor: page shell, node canvas (drag, link, unlink, delete), inspector with full concept/mapping editing, macOS look | ✅ (docs/STUDIO_UI.md) |
| — | **architecture review gate** (brief §63) | ⏳ next |
| G | type / semantic checking: formula parser, elaboration with rep/mk and units, Core typing, Grant, dimension via typing, diagnostics, `analyze()` in bdld, Studio shows status + diagnostics | ✅ (ADR-0013) |
| H | incomplete-declaration support (declared / open / invalid / type-valid) | ✅ |
| I | reference evaluator + simulator: dependency graph, causality, clock judgment, two-phase ticks, state cells, schedules, input traces, traces; bdld Start/Step/Reset | ✅ core (Studio Simulate page planned) |
| J | `delay` / `sync` state with explicit initial values | ✅ at Core level (surface temporal phrases planned) |
| K | output binding + single-driver diagnostic: nominal `PhysicalOutput`, `DriveWF` (exact type + domain), `SingleDriver`, partial validity vs executable completeness, `output_complete` | ✅ `bdl-output` (Studio Deploy page planned) |
| L | hardware resource allocator (`bdl-hardware`): capability model, device → requirements, deterministic sound+complete solver, dead-end diagnosis, Nano golden cases, `analyze_deployment` + `AnalyzeDeployment`/`ListTargets` in bdld | ✅ (ADR-0015) |
| M | board descriptions as data: `hardware/boards/arduino_nano.toml`, `big_board.toml` generated + round-tripped | ✅ (RP2040 board file and runtime loading of `hardware/boards/` next) |
| N | Rust code generation via a backend AST + `bdl-manifest.json`: executable IR (`bdl-exec-ir`), reactive lowering (`bdl-lower`), owned Rust AST + printer (`bdl-codegen-rust`), readiness check, `compile()` | ✅ (ADR-0016) |
| O | `no_std` generated core, host execution == interpreter: `runtime/bdl-runtime-core`, `runtime/bdl-runtime-host`, corpus + golden + property differential tests, every generated crate `cargo check`ed and run | ✅ |
| P | first Embassy runtime adapter | |
| Q | `cargo check` / build orchestration in `bdld` with structured events | |
| R | flash via `probe-rs` | |
| S | telemetry back into Studio | |

## IDE service (shared language-service layer)

| Step | | Status |
|---|---|---|
| IDE-1 | `bdl-ide-db` (IdeHost, overlays, EntityRef, projections, stamped snapshots, cancellation), `bdl-ide` (diagnostics, hover/explain, completion, references, rename, actions, edit plans, invalidation preview, symbols, tokens), `bdl-lsp` MVP (hover, definition, references, rename, completion, pull diagnostics, symbols, semantic tokens, code actions); Studio drafts and LSP buffers as overlays; ADR-0017 | ✅ (docs/IDE_SERVICE_ARCHITECTURE.md) |
| IDE-2 | complete textual parser + textual project loader (persisted textual identities, outputs/clocks/devices syntax, parameter layer DI-30) | |
| IDE-3 | richer LSP: inlay hints, workspace symbols, explain/Core/Rust virtual documents, formatting, cross-surface actions applying model operations from editors | |
| IDE-4 | incremental query engine — only if profiling on real projects asks for it (baseline: ~2 ms per full analysis at 400 mappings) | |

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
