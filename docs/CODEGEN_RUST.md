# Rust code generation

`crates/bdl-codegen-rust`: an executable IR becomes a Cargo crate holding
a `no_std` semantic core, a `std` host bridge, and a versioned manifest.
Driven by `bdl_compiler::compile(snapshot, options)` (or
`compile_design_ir`), which runs analysis → readiness → lowering →
generation and returns a `CompileArtifact { analysis, exec_ir, generated,
diagnostics }`.

```
ExecIr ──emit──▶ ast::Module ──print──▶ src/lib.rs        no_std core
       ──host──▶ ast::Module ──print──▶ src/bin/host.rs   std bridge (feature "host")
       ──manifest──▶ bdl-manifest.json
       ───────────▶ Cargo.toml                            [workspace] of its own
```

The generated Rust is built as a small owned AST (`ast.rs`: the subset BDL
needs — items, structs, fns, lets, calls, `if`/`match`, literals) and
printed by one deterministic printer (`print.rs`). No `format!`-assembled
functions; no Rust parser.

## Readiness (the backend entry condition)

`bdl_compiler::readiness(&analysis, require_complete)` — code is generated
only when every relationship with a definition checks (no `Open`/`Invalid`
mapping), the design is causal, clock-consistent, and its outputs are
partially well formed (`DriveWF ∧ SingleDriver`); with
`CompileOptions::require_complete`, also `output_complete`. Otherwise one
`backend.not_ready` diagnostic lists every unmet condition and nothing is
produced. Deployment feasibility is *not* a condition here: the core is
target-independent; a platform adapter stage may require it later.

## Implementation correspondence

Not a proof — the correspondence the differential tests check.

| Reference evaluator (`bdl-reactive::eval`) | Executable IR | Generated Rust |
|---|---|---|
| `Expr::DeclRef d` → `decl_value(d)` (memoised) | `ReadDecl i` | `read_decl(decl_n, n)?` — a `let decl_n: Option<T>` bound earlier in `step` |
| unresolved declaration → `input.values[d]` or `MissingInput` | `DeclKind::Input { slot }` | `Inputs.decl_n: Option<T>`; `read_input(inputs.decl_n, n)?` when due |
| `evaluated_this_tick`: domain active, or agnostic and anything active | `Activation` + `has_domains` | `if active.is_active(CLOCK_k) { Some(…) } else { None }` / `prim::or(active.any(), !HAS_DOMAINS)` |
| `Expr::Delay/Sync` read: `prev.cells[(d,path)]` else `init` | `ReadCell { slot, init }` | `match prev.cell_k { Some(v) => v, None => init }` |
| write phase: `temporal_sites` whose writer is active → `next[(d,path)] = e` | `CellPlan { writer, operand }` in `StateCellId` order | `if active.is_active(CLOCK_w) { next.cell_k = Some(operand); }` |
| `TickOutcome.next` replaces state after the tick | — | `state.cells = next` after the write phase; on `Err` untouched |
| `Expr::Mk s e` / `Expr::Rep e` | `Wrap` / `Unwrap` | `SemN(e)` / `e.0` — `pub struct SemN(pub Repr)` per concept |
| `apply_prim` (strict, finite-checked) | `Prim { op, args }` | `num::add/sub/mul/div(a, b, decl)?`, `(a < b)`, `(a == b)`, `(!a)`, `prim::and/or/ite/get_d`, `Some(x)`, `x.is_some()`, `Option::<T>::None` |
| `Expr::Lam` / `Expr::App` | inlined: `Let` | `{ let l0 = …; body }` |
| `ClockId` (nominal) | `ClockSlot` (dense) | `pub const CLOCK_k: ClockSlot`; `ActiveDomains` bitset |
| `output_values(sample, valid_bindings)` | `OutputPlan { driver }` | `Outputs.output_n = decl_driver`, built after the write phase |
| `RuntimeError::{MissingInput, DivisionByZero, NonFinite}` with `decl`, `tick` | same | `bdl_runtime_core::RuntimeError` with `decl` (raw `DeclId`); the host adds the tick |

The runtime vocabulary — `ActiveDomains`, `ClockSlot`, `RuntimeError`,
the checked numerics and the strict primitive helpers — lives in
`runtime/bdl-runtime-core` (`no_std`, allocation-free, `unsafe`-free,
knows no device kind, board, transport or editor).

## The generated core

```rust
pub const DESIGN: &str; pub const HAS_DOMAINS: bool; pub const CLOCK_COUNT: u16;
pub const CLOCK_k: ClockSlot;
pub struct SemN(pub Repr);                       // one per concept carried
pub struct Cells { pub cell_k: Option<T>, … }    // temporal state, None until first written
pub struct State { pub cells: Cells }
pub struct Inputs { pub decl_n: Option<T>, … }   // unresolved declarations
pub struct Values { pub decl_n: Option<T>, … }   // every declaration, None when not due
pub struct Outputs { pub output_n: Option<T>, … }
pub struct Tick { pub values: Values, pub outputs: Outputs }
pub fn init() -> State;
pub fn step(state: &mut State, active: ActiveDomains, inputs: &Inputs) -> Result<Tick, RuntimeError>;
```

Everything is `Copy`, statically sized, and laid out by the compiler: no
graph, no map, no allocation, no traversal at runtime. Symbols derive from
stable ids (`decl_17`, `Sem3`, `cell_0`, `output_4`, `CLOCK_2`), never
from display names, which appear in comments and the manifest only. Types:
`q d` → `f64` (the dimension is static; it is in the manifest), `bool`,
`nat` → `u64`, `sem s` → `SemN`, `opt τ` → `Option<T>`; a function type
has no runtime representation (its declaration is inlined).

The core is `#![no_std] #![forbid(unsafe_code)]` and mentions no HAL, pin,
peripheral or board; `cargo check --lib` of every corpus crate is a test.

## The host bridge and harness

`src/bin/host.rs` (feature `host`, `std`) implements
`bdl_runtime_host::HostProgram` for the core: generated conversions
between the static types and `DynValue` (the reference evaluator's value
shape, dimensions dropped), `init`/`step` forwarding, `size_of::<State>()`.
`bdl_runtime_host::main_stdio` reads a JSON `RunRequest { ticks: [{ active:
[clock slots], inputs: [per input slot] }] }` on stdin and writes a
`RunTrace { ticks: [{ values, outputs }], error?, metrics }` on stdout.
`bdl_runtime_host::harness::Cargo` writes a generated crate, `cargo
check`s the core, builds the host binary and runs a request — what the
differential tests use (`target/bdl-generated/<design>/`, one shared
`CARGO_TARGET_DIR` so the runtime crates compile once).

Rust symbol ↔ BDL identity is `bdl-manifest.json` (`manifest_version` 1):
clocks (slot, `ClockId`, name, symbol), concepts, inputs (slot, `DeclId`,
symbol, type), decls (plan index, `DeclId`, symbol, type, activation,
input slot), cells (slot, symbol, `StateCellId` as `decl_id` + `path`,
writer slot, type), outputs (slot, `OutputId`, symbol, driver `DeclId`,
type), functions (inlined declarations). Source spans are not in it yet:
the surface elaborator's spans are per formula, and the manifest carries
the expression path so they can be joined later.

## Differential testing (`crates/bdl-compiler/tests/backend_*.rs`)

For the same design, schedule and input trace, the reference evaluator
and the generated program must agree — value by value, tick by tick:

* **corpus** (`tests/support/mod.rs`): lamp (and lamp with an output),
  pure arithmetic with dimensions and a Count, semantic `rep`/`mk` with a
  Boolean concept, booleans/comparisons/strict `if`/options, `delay`, a
  cycle broken by `delay`, `sync` across two domains in both directions
  with the slow domain on period 2 (so source and destination share some
  ticks), a domain-agnostic declaration shared by two domains, division by
  zero, non-finite result, missing input, a design with no domain. Every
  case is generated, `cargo check`ed as a `no_std` library, built with its
  bridge, run, and compared; golden files under `tests/golden/<case>/`
  pin the exact generated bytes (`BDL_UPDATE_GOLDEN=1` to accept changes)
  and generating twice must give identical output.
* **property-based**: a small subset (dimensionless quantities, constants,
  references to earlier declarations, `+ - * /`, `delay`, inputs, one or
  two domains with a period-2 slow domain). In process, reference vs the
  exec-IR interpreter over 250 generated designs per run; compiled, a
  seeded batch of 8 generated designs through the full toolchain.
* **sync / order**: the reference's own tests show its result is
  independent of the host's processing order (`step_in_order`); the
  generated program's order is fixed by the plan and its traces equal the
  reference's, and are unchanged when the request lists active domains in
  the opposite order.

### Comparison policy

* Quantities, booleans, counts, semantic wrappers and options are compared
  **exactly** (`f64` bit-for-bit in effect: `==`, and NaN never occurs).
  The generated code performs the same IEEE operations in the same order
  as the reference (no reassociation, no fusion, no CSE), so nothing
  looser is justified. The JSON transport is exact too: `serde_json` is
  built with `float_roundtrip`, without which a parsed input can be off by
  one ulp (seen, and the reason the feature is on).
* Dimensions are not compared at runtime: they are static, established by
  typing, and recorded in the manifest.
* Errors: the failing **tick** must match exactly and the failure must be
  one the reference could report at that tick — when several declarations
  fail at once the two engines may pick different ones (DI-25); the tests
  compute the reference's alternatives by starting its traversal at every
  due declaration and require the generated failure to be among them.
* The reference's per-tick `values` hold computed declarations only
  (inputs are read through, never memoised); the generated `Values` holds
  inputs too, and the tests compare those against the fed inputs (DI-27).

## Numeric semantics (current)

The generated backend follows the reference evaluator exactly: IEEE `f64`
on host and in the generated core; division by an exact zero and any
non-finite primitive result fail the tick with a structured error; equality
is exact (DI-15). `f32` on device, exact source literals, symbolic
simplification and alternate numeric lowering are separate future work
(DI-1, DI-18, DI-28) and are not conflated with backend correctness now.
Every numeric literal is printed with Rust's shortest round-tripping
representation (`{:?}`), so the generated constant is the same `f64` the
elaborator produced.

## Baseline metrics (debug build, Apple Silicon host; recorded, not optimised)

| Design | core lines | crate bytes | `size_of::<State>()` | steps | `step` ns total |
|---|---|---|---|---|---|
| lamp | 95 | 6942 | 0 | 4 | 250 |
| lamp_output | 97 | 7295 | 0 | 4 | 334 |
| arith | 91 | 6947 | 0 | 2 | 249 |
| semantic | 102 | 8096 | 0 | 3 | 290 |
| bool_opt | 99 | 8332 | 0 | 4 | 667 |
| delay | 94 | 7021 | 16 | 5 | 501 |
| delayed_cycle | 89 | 6318 | 16 | 4 | 331 |
| sync | 100 | 7427 | 32 | 7 | 1001 |
| agnostic | 91 | 6630 | 0 | 6 | 1000 |
| div_by_zero | 87 | 6233 | 0 | 3 | 542 |
| non_finite | 87 | 6230 | 0 | 2 | 417 |
| missing_input | 87 | 6243 | 0 | 2 | 375 |
| no_domains | 82 | 5782 | 0 | 2 | 334 |

(`every_corpus_case_agrees_with_the_reference -- --nocapture` prints the
current numbers.) Timings are of a debug build and include the closure
of `Instant::now()`; they are a baseline for later evaluation, nothing
more.

## What is deliberately not here

No optimisation (CSE, fusion, inlining beyond what removes binders,
reassociation). No supplied components — the architecture leaves the
`BdlPureComponent<I, O>` boundary open, and generated semantic code will
never call a HAL through it. No platform adapter: the Nano allocation and
any `DeploymentAnalysis` stay out of the semantic core by construction
(`bdl-codegen-rust` does not depend on `bdl-hardware`). The reference
evaluator is not replaced by generated code; it stays the oracle.
