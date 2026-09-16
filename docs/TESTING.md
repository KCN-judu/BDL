# Testing

Layered; determinism is a tested property, not a hope.

| Layer | What | Where | Status |
|---|---|---|---|
| Rust unit | ids, dims, edit classification, persistence, framing, conversions, session/undo | `crates/*/src` | in place |
| Rust end-to-end | spawn the real `bdld`, drive the framed protocol through steps 1–12 of the vertical slice | `crates/bdl-daemon/tests/stdio_e2e.rs` | in place |
| Dart unit | pure reducer transitions, framing codec | `apps/studio/test` | in place |
| Dart integration | the Dart client against the real `bdld` binary (skipped if not built) | `apps/studio/test/daemon_client_test.dart` | in place |
| Flutter widget | shell status line, library/canvas/inspector with an open project | `apps/studio/test/shell_test.dart` | in place |
| Dart unit | canvas geometry: scene, auto-placement, hit testing, typed drop targets | `apps/studio/test/canvas_geometry_test.dart` | in place |
| Dart unit | definition drafts through the reducer: generations, rebasing, conflicts, attach vs replace, failed commits, detach, deletion, close/reopen stash; byte-span → code-unit conversion | `apps/studio/test/draft_reducer_test.dart`, `source_span_test.dart` | in place |
| Dart effects | reducer + effect executor against a scripted daemon (`test/support/test_store.dart`): debounce, out-of-order answers, stale-revision refusal, daemon errors, transport loss | `apps/studio/test/draft_effects_test.dart` | in place |
| Flutter widget | the definition editor in the inspector over the real reducer: typing, verdicts and spans, add/save/revert/detach, conflicts, selection switches, pushes, ⌘S/Esc | `apps/studio/test/definition_editor_test.dart` | in place |
| Dart end-to-end | reducer + executor + Dart client against the real `bdld`: the Tilt/Brightness draft → verdict → add → save → reopen path, the invalid-dimension correction path, the stale (signature / representation change) path, detach and a refused commit; prints the edit → verdict timing | `apps/studio/test/formula_e2e_test.dart` | in place (skipped without the binary) |
| Rust end-to-end | definition drafts over stdio: valid / invalid / open verdicts, exact spans, generation echo, `draft.stale_revision`, `draft.unknown_mapping`, completion and hover over the overlay, discard, a deleted mapping's overlay pruned, nothing committed, save/reopen keeps the source with no phantom overlay | `crates/bdl-daemon/tests/stdio_e2e.rs` | in place |
| Dart unit + widget | completion and hover (generations, service order, byte offsets, pop-up keys, cards); timing domains and outputs (edits, plan queue, sink geometry and link rules, output inspector, fixes); Deploy page (board gating, verdict wording, placement, dead end); Simulate page (trace extension, generation/revision gating, readiness blockers with named objects and a refused step, input cells gated by domain activation, controls by value form, failure wording, blocker links) | `apps/studio/test/{tooling,outputs,deploy,simulation}_test.dart` | in place |
| Dart end-to-end | against `bdld`: completion/hover, domains → outputs → drives → fixes, deployment on two boards, simulation of the lamp / delay / sync through the surface language plus an unresolved input supplied then changed, reset, multi-clock activation, an invalid design refused before any request, a revision invalidating the run, and a runtime division by zero on the controls' line; and the Smart Lamp example across Design → Simulate → Deploy | `apps/studio/test/{formula_e2e,design_e2e,deploy,simulation,smart_lamp_e2e}_test.dart` | in place (skipped without the binary) |
| Rust cross-layer | surface formula → Core → reference evaluator → generated core for relationship application, `delay` and `sync` (DI-17) and for every textual expression form (`let`, `if`, calls, `Some`/`None`, `match`, memory in a `let` / scrutinee / branch); the Smart Lamp example authored, analysed, simulated, placed and kept byte-identical | `crates/bdl-compiler/tests/{surface_to_backend,surface_expressions,examples}.rs` | in place |
| Rust end-to-end | deployment over stdio: the target list as a chooser, the 10-case matrix of incomplete / feasible / infeasible read models by name, stale-revision refusal | `crates/bdl-daemon/tests/deploy_e2e.rs` | in place |
| Rust integration | Studio and LSP share semantics: the same formula as a `MappingDefinitionDraft` overlay and inside a `TextDocument` overlay gives the same semantic diagnostics (code, severity, entity, role) and status — valid, invalid, syntax-broken, open, after a representation change | `crates/bdl-ide/tests/surface_equivalence.rs` | in place |
| Render preview | renders the shell with the real system font to PNG (`SNAP_DIR=… flutter test --update-goldens test/snapshot_preview_test.dart`, or `just studio-snap`) — for looking, not asserting | `apps/studio/test/snapshot_preview_test.dart` | in place (opt-in) |
| Property | solver soundness and agreement with brute force; evaluator order-independence; reference == exec-IR interpreter over generated designs; a seeded batch compiled and run | `proptest` in `bdl-hardware`, `bdl-reactive`, `bdl-compiler/tests/backend_property.rs` | in place |
| Golden | corpus design → generated `Cargo.toml`, `src/lib.rs`, `src/bin/host.rs`, `bdl-manifest.json`; generating twice is byte-identical | `crates/bdl-compiler/tests/backend_golden.rs` over `tests/golden/<case>/` (`BDL_UPDATE_GOLDEN=1` to accept) | in place |
| Generated-crate build | every corpus core `cargo check`ed as a `no_std` library; host binary built | `crates/bdl-compiler/tests/backend_differential.rs` → `target/bdl-generated/` | in place |
| Behaviour systems | the TiltSource + 2×AdaptiveLamp slice through the system edit model: identity freshening and stability under renames/unrelated instances, identity-based binding, open ports, composed cycles, cross-domain transport with the strictly-before rule, single driver across instances, device multiplication, packaging, persistence, invalidation; system vs hand-written flat design compared on analysis, traces, outputs, exec-IR and generated-Rust host runs | `crates/bdl-system/tests/vertical_slice.rs`, `crates/bdl-daemon/tests/system_e2e.rs` | in place |
| Differential | reference evaluator trace == generated-Rust host trace (values, outputs, errors) per corpus case and per seeded random design | `crates/bdl-compiler/tests/backend_differential.rs`, `backend_property.rs` (policy: `docs/CODEGEN_RUST.md`) | in place; highest-value test in the project |
| IDE service | the acceptance scenarios of the shared language service: unresolved mappings stay queryable, one entity on both surfaces, one diagnostic projected to text and canvas, drafts and buffers as overlays, stale results rejected, cancellation, rename by identity, no panics on garbage | `crates/bdl-ide/tests/acceptance.rs`, `crates/bdl-ide-db/src/*` | in place |
| LSP end-to-end | a JSON-RPC client over an in-memory connection: initialize, open/change/close, hover, definition, references, prepare/rename, completion, pull diagnostics, symbols, semantic tokens, code actions, custom requests; UTF-16 positions on non-ASCII text; push fallback; project loading | `crates/bdl-lsp/tests/e2e.rs`, `crates/bdl-lsp/src/position.rs` | in place |
| Concept library | the embedded library loads and validates; two instantiations are two concepts; a project reloads without the library and a changed template does not reach it; every template is a textual completion from the same data; the daemon serves it template for template | `crates/bdl-library`, `crates/bdl-ide/tests/acceptance.rs`, `crates/bdl-daemon/tests/stdio_e2e.rs` | in place |
| Studio library | one creation path for menu and drag, create-then-rename, recents, inline rename, search; and against the real `bdld`: insert twice, rename, save, reopen | `apps/studio/test/concept_library_test.dart`, `concept_library_e2e_test.dart` | in place |
| Flutter | reducer, widget transitions, protocol integration — never screenshots only | | ongoing |

CI (`.github/workflows/ci.yml`), four jobs: **Rust** — `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`,
then the Linux `bdld` is uploaded; **Flutter** — `dart format --page-width
100 --set-exit-if-changed`, `flutter analyze`, `flutter test` with that
`bdld` on `BDLD_PATH` so the e2e suites run; **Windows** — `cargo test
--workspace`, `flutter test` against `bdld.exe`, `flutter build windows
--debug`; **Protocol** — a diff of the checked-in generated Dart protobuf
code against a fresh `protoc` run.

Locally: `just check`.

## Determinism rules under test

* `BTreeMap` everywhere in the model; projections are ordered lists.
* No random ids; ids come from the persisted allocator.
* Hardware solving iterates in documented order; codegen output is
  byte-stable for the same design + compiler version (golden files).
* Generated crates are written under `target/bdl-generated/<case>/` and
  built with one shared `CARGO_TARGET_DIR` (`target/bdl-generated/target`)
  so the runtime crates compile once; the tests spawn `cargo`, so they are
  slower than the rest (≈20 s cold) and need no network.
