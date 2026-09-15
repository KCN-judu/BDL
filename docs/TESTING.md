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
| Render preview | renders the shell with the real system font to PNG (`SNAP_DIR=… flutter test --update-goldens test/snapshot_preview_test.dart`, or `just studio-snap`) — for looking, not asserting | `apps/studio/test/snapshot_preview_test.dart` | in place (opt-in) |
| Property | solver output satisfies constraints; evaluator stability; codegen stability; refinement preserves public projections | `proptest` | planned with each subsystem |
| Golden | project → diagnostics / IR / generated Rust / trace | `tests/golden/` | planned |
| Differential | reference interpreter trace == generated-Rust host trace == Lean fixtures where practical | `tests/differential/` | planned; highest-value test in the project |
| Flutter | reducer, widget transitions, protocol integration — never screenshots only | | ongoing |

CI (`.github/workflows/ci.yml`): `cargo fmt --check`, `clippy -D warnings`,
`cargo test`; `dart format --set-exit-if-changed`, `flutter analyze`,
`flutter test` (with the built `bdld` so the integration test runs); a diff
of the checked-in generated Dart protobuf code against a fresh `protoc` run.

Locally: `just check`.

## Determinism rules under test

* `BTreeMap` everywhere in the model; projections are ordered lists.
* No random ids; ids come from the persisted allocator.
* Hardware solving (when it lands) iterates in documented order; codegen
  output is byte-stable for the same project + compiler version.
