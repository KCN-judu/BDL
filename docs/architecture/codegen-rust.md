---
kind: architecture
area: codegen
status: current
---

# Rust code generation

`crates/bdl-codegen-rust`: an executable IR becomes a Cargo crate holding a
`no_std` semantic core, a `std` host bridge, and a versioned manifest. Driven by
`bdl_compiler::compile(snapshot, options)` (or `compile_design_ir`), which runs
analysis → readiness → lowering → generation and returns a
`CompileArtifact { analysis, exec_ir, generated, diagnostics }`.

```text
ExecIr ──emit──▶ ast::Module ──print──▶ src/lib.rs        no_std core
       ──host──▶ ast::Module ──print──▶ src/bin/host.rs   std bridge (feature "host")
       ──manifest──▶ bdl-manifest.json
       ───────────▶ Cargo.toml                            [workspace] of its own
```

The generated Rust is built as a small owned AST (`ast.rs`: the subset BDL needs
— items, structs, fns, lets, calls, `if`/`match`, literals) and printed by one
deterministic printer (`print.rs`). No `format!`-assembled functions; no Rust
parser.

## Readiness (the backend entry condition)

`bdl_compiler::readiness(&analysis, require_complete)` — code is generated only
when every relationship with a definition checks (no `Open`/`Invalid` mapping),
the design is causal, clock-consistent, and its outputs are partially well
formed (`DriveWF ∧ SingleDriver`); with `CompileOptions::require_complete`, also
`output_complete`. Otherwise one `backend.not_ready` diagnostic lists every
unmet condition and nothing is produced. Deployment feasibility is _not_ a
condition here: the core is target-independent; a platform adapter stage may
require it later.

## Implementation correspondence

Not a proof — the correspondence the differential tests check.

| Reference evaluator (`bdl-reactive::eval`)                                    | Executable IR                                         | Generated Rust                                                                                                                                                                             |
| ----------------------------------------------------------------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Expr::DeclRef d` → `decl_value(d)` (memoised)                                | `ReadDecl i`                                          | `read_decl(&decl_n, n)?` — a `let decl_n: Option<T>` bound earlier in `step`, cloned out; `read_decl_ref(&decl_n, n)?` where a borrow suffices                                             |
| unresolved declaration → `input.values[d]` or `MissingInput`                  | `DeclKind::Input { slot }`                            | `Inputs.decl_n: Option<T>`; `read_input(&inputs.decl_n, n)?` when due                                                                                                                      |
| `evaluated_this_tick`: domain active, or agnostic and anything active         | `Activation` + `has_domains`                          | `if active.is_active(CLOCK_k) { Some(…) } else { None }` / `prim::or(active.any(), !HAS_DOMAINS)`                                                                                          |
| `Expr::Delay/Sync` read: `prev.cells[(d,path)]` else `init`                   | `ReadCell { slot, init }`                             | `match &prev.cell_k { Some(v) => v.clone(), None => init }`                                                                                                                                |
| write phase: `temporal_sites` whose writer is active → `next[(d,path)] = e`   | `CellPlan { writer, operand }` in `StateCellId` order | `let write_k: Option<T> = if active.is_active(CLOCK_w) { Some(operand) } else { None };` — every operand before any commit                                                                 |
| `TickOutcome.next` replaces state after the tick                              | —                                                     | commit: `if write_k.is_some() { state.cells.cell_k = write_k; }` per cell; unchanged cells are never copied; on `Err` untouched                                                            |
| `Expr::Mk s e` / `Expr::Rep e`                                                | `Wrap` / `Unwrap`                                     | `SemN(e)` / `e.0` — `pub struct SemN(pub Repr)` per concept                                                                                                                                |
| `apply_prim` (strict, finite-checked)                                         | `Prim { op, args }`                                   | `num::add/sub/mul/div(a, b, decl)?`, `(&a < &b)`, `(&a == &b)`, `(!a)`, `prim::and/or/ite/get_d`, `Some(x)`, `x.is_some()`, `Option::<T>::None`                                            |
| `ite c x y` with `x`, `y` total (nothing in them can fail)                    | `Prim { Ite }` (still strict)                         | `if c { x } else { y }` — observationally the strict `ite`, without building both branches                                                                                                 |
| `Expr::Lam` / `Expr::App`                                                     | inlined: `Let`                                        | `{ let l0 = …; body }`; a local referenced once in its own scope is moved, any other reference is `.clone()`                                                                               |
| `Expr::Fold f z l` (finite iteration from the last element)                   | `Fold { elem, acc, step, init, list }`                | `list::fold(xs, init, \|l1, l2\| Ok(step))?` — one closure per recursor, applied by the runtime, never a closure value                                                                     |
| `nil`/`cons`/`length`/`take`/`drop`/`reverse`/`head`/`toList`                 | `PrimOp::{Nil, Cons, …}`                              | `list::nil::<T>()`, `list::cons(x, xs)`, `list::length_of(&xs)`, `list::head_of(&xs)`, `list::take_of(k, &xs)`, `list::drop(k, xs)`, … (`runtime/bdl-runtime-core`, feature `collections`) |
| `pair`/`fst`/`snd`; `Value::Pair`                                             | `PrimOp::{Pair, Fst, Snd}`                            | `(a, b)`, `p.0`, `p.1`                                                                                                                                                                     |
| `Prim::Eq { ty }` (`Value::structurally_equal`)                               | `PrimOp::Eq`                                          | `(a == b)` — `PartialEq` on `f64`, `bool`, `u64`, `SemN`, `Option`, `Vec`, tuples is the same elementwise equality                                                                         |
| `ClockId` (nominal)                                                           | `ClockSlot` (dense)                                   | `pub const CLOCK_k: ClockSlot`; `ActiveDomains` bitset                                                                                                                                     |
| `output_values(sample, valid_bindings)`                                       | `OutputPlan { driver }`                               | `Outputs.output_n = decl_driver.clone()`, built after the write phase                                                                                                                      |
| the encoder body `encode (rep d)` of a realization (no reference form)        | `SinkPlan { driver, command }`                        | `let command_d = if decl_driver.is_some() { Some(…) } else { None }` after the commit; `Commands.command_d`; `commands_to_dyn` in the host (docs/architecture/output-realization.md)       |
| `RuntimeError::{MissingInput, DivisionByZero, NonFinite}` with `decl`, `tick` | same                                                  | `bdl_runtime_core::RuntimeError` with `decl` (raw `DeclId`); the host adds the tick                                                                                                        |

The runtime vocabulary — `ActiveDomains`, `ClockSlot`, `RuntimeError`, the
checked numerics and the strict primitive helpers — lives in
`runtime/bdl-runtime-core` (`no_std`, `unsafe`-free, knows no device kind,
board, transport or editor). It is allocation-free unless its `collections`
feature is on: then `list` provides the list operators and the recursor over
`alloc::vec::Vec`, and a target needs a global allocator (ADR-0024). The
generator turns the feature on exactly when the plan carries a list
(`ExecIr::uses_lists`) and records it as `requires_allocator` in the manifest.

**Lists in the core are stored last element first.** `cons` pushes, the recursor
consumes from the front, so the library's `map`, `filter` and `append` — folds
that `cons` onto the accumulator — stay linear; `==` is elementwise as in the
list's order; `take`/`drop`/`head`/`reverse` translate accordingly. The host
bridge reverses at the boundary (`list::from_ordered`, `into_ordered`); nothing
inside the core observes the storage order — the audit
`runtime/bdl-runtime-core/tests/lists.rs` compares every operator, the library's
folds and equality against a list-order model on generated lists.

### Cost discipline

What the emitter guarantees about copies, so a bounded design has a bounded,
predictable cost per tick (docs/spec/deployment-capacity.md §6):

- **Moves, not clones.** A use analysis per expression tree counts the
  references to each local, the two branches of a lazy `if` counting once as the
  larger; a local referenced once in the scope that binds it is moved, any other
  reference clones, and a local captured by a fold's closure is always cloned
  (the closure runs per element). A fold step that `cons`es onto its accumulator
  therefore moves it: `map`, `filter`, `append`, `sum`, `any`, `all`, `contains`
  clone nothing per element (tested with a clone-counting element in
  `tests/lists.rs`; measured on the generated core at 2 000 → 32 000 elements:
  ×15, docs/evidence/testing.md).
- **A total conditional is lazy.** A strict `ite` both of whose branches cannot
  fail (no arithmetic, no declaration read, transitively) is emitted as a Rust
  `if`: the reference evaluates both branches, but with nothing that can fail in
  either, choosing first is not observable; a branch that can fail keeps
  `prim::ite` and its strictness (corpus `ite_strictness`).
- **Borrowed reads.** `length`, `head`, `take`, `==` and `<` take their operands
  by reference — a local as `&l`, a declaration through `read_decl_ref` — so a
  list read for its size, first element, prefix or comparison is never copied.
- **Commit-only writes.** A tick clones no cell it does not read: the write
  phase evaluates every active writer's operand into a local, the commit moves
  those into the state, and unchanged cells are untouched. Reading a cell
  (`ReadCell`) still clones it into the declaration's value — the value is
  observable in `Tick.values` — as does `read_decl` where an owned value is
  needed.

What remains: `zip` clones its accumulator pair per element, because the pair's
`rest` is used twice in one step (ISS-0013); `read_decl` and `ReadCell` clone
where an owned value is consumed; no fusion of `map → filter` chains — each is
linear and measured, and a fused emission was not justified by the numbers.

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
pub struct Commands { pub command_d: Option<Raw>, … }   // one per realised output (device id d), after the outputs
pub struct Tick { pub values: Values, pub outputs: Outputs, pub commands: Commands }
pub fn init() -> State;
pub fn step(state: &mut State, active: ActiveDomains, inputs: &Inputs) -> Result<Tick, RuntimeError>;
```

A program without lists is `Copy`, statically sized, and laid out by the
compiler: no graph, no map, no allocation, no traversal at runtime. A program
with lists derives `Clone` instead of `Copy` on its records and concepts, starts
with `extern crate alloc;`, and reads declarations and cells by clone where an
owned value is consumed (the same generated code either way). Its manifest
carries a `collections` entry — per cell the bound of its outermost list
(`finite`/`input`/`unbounded`), the list-typed input slots, `state_bytes_max`,
`tick_bytes_max`, `unbounded` — computed by `bdl-exec-ir::bounds`
(docs/spec/deployment-capacity.md). Symbols derive from stable ids (`decl_17`,
`Sem3`, `cell_0`, `output_4`, `CLOCK_2`), never from display names, which appear
in comments and the manifest only. Types: `q d` → `f64` (the dimension is
static; it is in the manifest), `bool`, `nat` → `u64`, `sem s` → `SemN`, `opt τ`
→ `Option<T>`, `list τ` → `Vec<T>` (last element first), `τ × σ` → `(T, S)`; a
function type has no runtime representation (its declaration is inlined; a rule
given to an equation is inlined into the fold's closure).

The core is `#![no_std] #![forbid(unsafe_code)]` and mentions no HAL, pin,
peripheral or board; `cargo check --lib` of every corpus crate is a test.

## The host bridge and harness

`src/bin/host.rs` (feature `host`, `std`) implements
`bdl_runtime_host::HostProgram` for the core: generated conversions between the
static types and `DynValue` (the reference evaluator's value shape, dimensions
dropped), `init`/`step` forwarding, `size_of::<State>()`.
`bdl_runtime_host::main_stdio` reads a JSON
`RunRequest { ticks: [{ active: [clock slots], inputs: [per input slot] }] }` on
stdin and writes a `RunTrace { ticks: [{ values, outputs }], error?, metrics }`
on stdout. `bdl_runtime_host::harness::Cargo` writes a generated crate,
`cargo check`s the core, builds the host binary and runs a request — what the
differential tests use (`target/bdl-generated/<design>/`, one shared
`CARGO_TARGET_DIR` so the runtime crates compile once).

Rust symbol ↔ BDL identity is `bdl-manifest.json` (`manifest_version` 1): clocks
(slot, `ClockId`, name, symbol), concepts, inputs (slot, `DeclId`, symbol,
type), decls (plan index, `DeclId`, symbol, type, activation, input slot), cells
(slot, symbol, `StateCellId` as `decl_id` + `path`, writer slot, type), outputs
(slot, `OutputId`, symbol, driver `DeclId`, type), functions (inlined
declarations). Source spans are not in it yet: the surface elaborator's spans
are per formula, and the manifest carries the expression path so they can be
joined later.

## Differential testing (`crates/bdl-compiler/tests/backend_*.rs`)

For the same design, schedule and input trace, the reference evaluator and the
generated program must agree — value by value, tick by tick:

- **corpus** (`tests/support/mod.rs`): lamp (and lamp with an output), pure
  arithmetic with dimensions and a Count, `collections` (the library's `any`,
  `sum`, `map`, `filter`, `zip`, `contains`, `clamp`, `getOrElse`/`head`,
  structural equality, a list in a state cell) and `buffer` (the Phase-9a
  lossless window as five declarations over `delay`/`sync`), semantic `rep`/`mk`
  with a Boolean concept, booleans/comparisons/strict `if`/options, `delay`, a
  cycle broken by `delay`, `sync` across two domains in both directions with the
  slow domain on period 2 (so source and destination share some ticks), a
  domain-agnostic declaration shared by two domains, division by zero,
  non-finite result, missing input, a design with no domain. Every case is
  generated, `cargo check`ed as a `no_std` library, built with its bridge, run,
  and compared; golden files under `tests/golden/<case>/` pin the exact
  generated bytes (`BDL_UPDATE_GOLDEN=1` to accept changes) and generating twice
  must give identical output.
- **property-based**: a small subset (dimensionless quantities, constants,
  references to earlier declarations, `+ - * /`, `delay`, inputs, one or two
  domains with a period-2 slow domain). In process, reference vs the exec-IR
  interpreter over 250 generated designs per run; compiled, a seeded batch of 8
  generated designs through the full toolchain.
- **sync / order**: the reference's own tests show its result is independent of
  the host's processing order (`step_in_order`); the generated program's order
  is fixed by the plan and its traces equal the reference's, and are unchanged
  when the request lists active domains in the opposite order.

### Comparison policy

- Quantities, booleans, counts, semantic wrappers and options are compared
  **exactly** (`f64` bit-for-bit in effect: `==`, and NaN never occurs). The
  generated code performs the same IEEE operations in the same order as the
  reference (no reassociation, no fusion, no CSE), so nothing looser is
  justified. The JSON transport is exact too: `serde_json` is built with
  `float_roundtrip`, without which a parsed input can be off by one ulp (seen,
  and the reason the feature is on).
- Dimensions are not compared at runtime: they are static, established by
  typing, and recorded in the manifest.
- Errors: the failing **tick** must match exactly and the failure must be one
  the reference could report at that tick — when several declarations fail at
  once the two engines may pick different ones (DI-25); the tests compute the
  reference's alternatives by starting its traversal at every due declaration
  and require the generated failure to be among them.
- The reference's per-tick `values` hold computed declarations only (inputs are
  read through, never memoised); the generated `Values` holds inputs too, and
  the tests compare those against the fed inputs (DI-27).

## Numeric semantics (current)

The generated backend follows the reference evaluator exactly: IEEE `f64` on
host and in the generated core; division by an exact zero and any non-finite
primitive result fail the tick with a structured error; equality is exact
(DI-15). `f32` on device, exact source literals, symbolic simplification and
alternate numeric lowering are separate future work (DI-1, DI-18, DI-28) and are
not conflated with backend correctness now. Every numeric literal is printed
with Rust's shortest round-tripping representation (`{:?}`), so the generated
constant is the same `f64` the elaborator produced.

## Baseline metrics (debug build, Apple Silicon host; recorded, not optimised)

| Design        | core lines | crate bytes | `size_of::<State>()` | steps | `step` ns total |
| ------------- | ---------- | ----------- | -------------------- | ----- | --------------- |
| lamp          | 95         | 6942        | 0                    | 4     | 250             |
| lamp_output   | 97         | 7295        | 0                    | 4     | 334             |
| arith         | 91         | 6947        | 0                    | 2     | 249             |
| semantic      | 102        | 8096        | 0                    | 3     | 290             |
| bool_opt      | 99         | 8332        | 0                    | 4     | 667             |
| delay         | 94         | 7021        | 16                   | 5     | 501             |
| delayed_cycle | 89         | 6318        | 16                   | 4     | 331             |
| sync          | 100        | 7427        | 32                   | 7     | 1001            |
| agnostic      | 91         | 6630        | 0                    | 6     | 1000            |
| div_by_zero   | 87         | 6233        | 0                    | 3     | 542             |
| non_finite    | 87         | 6230        | 0                    | 2     | 417             |
| missing_input | 87         | 6243        | 0                    | 2     | 375             |
| no_domains    | 82         | 5782        | 0                    | 2     | 334             |
| collections   | 124        | 14340       | 24                   | 4     | 104042          |
| buffer        | 115        | 10061       | 64                   | 7     | 57458           |

(`collections` and `buffer` — the list, pair and fold cases — allocate; their
timings are of a debug build, recorded before the moved-local emission.) The
per-operation scaling of the generated core is in docs/evidence/testing.md
§Collections cost (`collections_cost_measurement`, ignored by default).

(`every_corpus_case_agrees_with_the_reference -- --nocapture` prints the current
numbers.) Timings are of a debug build and include the closure of
`Instant::now()`; they are a baseline for later evaluation, nothing more.

## What is deliberately not here

No optimisation beyond the cost discipline above (no CSE, fusion, inlining
beyond what removes binders, reassociation). No supplied components — the
architecture leaves the `BdlPureComponent<I, O>` boundary open, and generated
semantic code will never call a HAL through it. No platform adapter: the Nano
allocation and any `DeploymentAnalysis` stay out of the semantic core by
construction (`bdl-codegen-rust` does not depend on `bdl-hardware`). The
reference evaluator is not replaced by generated code; it stays the oracle.
