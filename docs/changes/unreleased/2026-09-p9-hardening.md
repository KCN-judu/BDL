# P9 hardening: linear folds, collection bounds, window capacity, the order invariant

- Date: 2026-09-18
- Area: codegen, runtime, compiler, deployment, daemon, language
- Affected: designers, developers, platform-adapter authors
- Related: ADR-0026, ADR-0027, ISS-0011, ISS-0012, ISS-0013

## What changed

- **The generated core copies less.** A local used once is moved, a strict
  conditional whose branches cannot fail is a Rust `if`, `length`/`head`/
  `take`/`==`/`<` borrow their operands, and a tick writes only the cells its
  active writers produce — no whole-state clone. `map`, `filter`, `append`,
  `sum` are linear in the core (measured ×15 for ×16 elements); `zip` stays
  quadratic (ISS-0013, narrowed). Runtime core: `list::length_of`, `head_of`,
  `take_of`, `read_decl_ref`.
- **Collection bounds and window capacity are computed** (ADR-0027,
  `docs/spec/deployment-capacity.md`): every list a program carries gets a sound
  static bound (finite / as large as an input / unbounded); every list-carrying
  crossing gets the capacity the schedule requires (the production
  `Capacity.lean`); `CompileArtifact.collections` reports them and the manifest
  carries `collections` (per-cell bound, list inputs, `state_bytes_max`,
  `tick_bytes_max`, `unbounded`).
- **New diagnostics**: `deployment.unbounded_list_state` (warning on a host,
  **error** under `MemoryPolicy::Bounded` — no artefact),
  `deployment.list_input_unbounded`, `deployment.window_capacity`.
- **`bdld compile`** takes `--bounded-memory` and `--period domain=N`
  (repeatable) and prints the collections summary; `--json` includes the report.
- **The order invariant** is kept by the model: declaring a concept ordered
  needs a quantity value form (`edit.order_needs_quantity`), and an ordered
  concept's value form may only change to another quantity — clear the order
  first. Text: `ordered concept Mode : Count` is `text.order_needs_quantity` and
  loads unordered.
- **The mixed comparison rule** is decided (ADR-0026; ISS-0012 resolved): a
  concept beside a plain value of its representation is observed; the order
  declaration governs comparisons between concept values only.
- **`delay`/`sync`** elaborate the remembered value before the initial value, so
  `cons(x, delay([], log))` with `x` a concept over the list's element form
  elaborates (before: `type.temporal_mismatch`).

## Compatibility and migration

- Designers: a design whose remembered collection grows without bound now sees a
  warning; nothing is refused unless a bounded-memory target is asked for. An
  `ordered concept` over a count or a collection in existing sources loads with
  a fault and without the order (it never had an effect).
- Project files: nothing changes on disk. Protocol: nothing changes on the wire;
  the new edit refusal code `edit.order_needs_quantity` joins the `edit.*`
  family (0.11 unchanged).
- Developers: `CompileOptions` gains `memory` and `schedule` (`Default` keeps
  the old behaviour); `CompileArtifact` gains `collections`; every generated
  core changed shape (goldens regenerated) with the same trace.
- Platform-adapter authors: a list-carrying core needs a global allocator sized
  from the manifest's `collections` entry, and a stated maximum per list-typed
  input; `docs/spec/deployment-capacity.md` §6 has the readiness matrix for
  RP2040/RP2350 and ESP32-S3.

## Evidence

`runtime/bdl-runtime-core/tests/lists.rs` (the storage-order audit, the clone
counts), `crates/bdl-compiler/tests/backend_differential.rs` (the three-engine
corpus, `bounded_buffer`/`overflowing_buffer`, `ite_strictness`, the 3 000-tick
run, `collections_cost_measurement`), `crates/bdl-compiler/tests/collections.rs`
(bounds sound on every tick, the classification, the refusal, the window
capacity), `crates/bdl-reactive/src/capacity.rs` (the `Capacity.lean` theorems
as property tests), `crates/bdl-daemon/tests/cli.rs`
(`compile --period --bounded-memory`), `crates/bdl-elab/tests/equations.rs` (the
mixed-comparison matrix, product language), `crates/bdl-model/src/edit.rs` and
`crates/bdl-text/tests/workspace.rs` (the order invariant); commits `496f1f2`,
`6403cdd`, `faa8d9b`, `2de1b06`, `fabf936`.
