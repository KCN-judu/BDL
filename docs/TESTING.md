# Testing

Layered; determinism is a tested property, not a hope.

| Layer | What | Where | Status |
|---|---|---|---|
| Rust unit | ids, dims, edit classification, persistence, framing, conversions, session/undo | `crates/*/src` | in place |
| Rust end-to-end | spawn the real `bdld`, drive the framed protocol through steps 1–12 of the vertical slice | `crates/bdl-daemon/tests/stdio_e2e.rs` | in place |
| Dart unit | pure reducer transitions, framing codec | `apps/studio/test` | in place |
| Dart integration | the Dart client against the real `bdld` binary (skipped if not built) | `apps/studio/test/daemon_client_test.dart` | in place |
| Flutter widget | status bar states | `apps/studio/test/status_bar_test.dart` | in place |
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
