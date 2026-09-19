---
kind: specification
area: runtime
status: current
---

# Runtime semantics

How the kernel's tick semantics (`Ev` / `MEv`, BDL_FV `Reactive.lean`,
`Clock.lean`) become executable Rust. The **reference evaluator**
(`crates/bdl-reactive`) implements these rules today and is the executable
definition of BDL runtime behaviour; the generated core and the runtime adapters
(none exists yet — `docs/project/roadmap.md` priority 1) must agree with it tick
for tick.

## The reference evaluator (implemented)

- **Values** keep semantic identity and dimension: `Bool`, `Nat`,
  `Quantity { dim, value }`, `Semantic { id, repr }`, `None`/`Some`, `Closure`,
  partial `Prim`. A `Tilt` and a `MotorAngle` of equal magnitude are different
  values.
- **State cells** are addressed by `StateCellId { decl, path }` — the
  declaration and the expression path of the `delay`/`sync` inside its
  realization — so identity survives unrelated edits and re-elaboration.
- **A tick has two phases.** _Read_: every declaration due this tick is
  evaluated (lazily, memoised) with temporal forms yielding their cell's
  committed value, or the initial value if the cell was never written. _Write_:
  every temporal site whose writing domain is active — the owner's domain for
  `delay`, `src` for `sync` — evaluates its operand in the same read mode into
  the _next_ state. Nothing is updated in place; reads never see writes of the
  same tick.
- **Which declarations run at a tick**: those whose domain is active, plus
  domain-agnostic ones whenever anything is active (every tick if the design has
  no domains).
- **`sync src init e`** reads the cell last written by `src`'s activation
  strictly before now; two domains active at the same tick see each other's
  _previous_ activations only, so the order a host processes them in is
  unobservable (`step_in_order` exists to prove it).
- **Unresolved declarations** are inputs: a value per tick from an `InputTrace`;
  a missing one is `RuntimeError::MissingInput`, never a default.
- **Schedules** are outside the design: `Schedule { periods }` activates a
  domain at ticks divisible by its period. A `ClockId` never implies a rate.
- **Numerics (DI-15)**: IEEE `f64`. Division by zero and any non-finite result
  fail the tick with a structured error; equality is exact; traces serialise to
  JSON numbers.
- **Traces**: `TickSample { tick, active, values }` per tick; `values` holds
  every declaration due at the tick — a fed input echoed as it was fed, in the
  ticks its domain activated — and every sample keeps the `DeclId`. A value has
  one designer-facing rendering, the evaluator's (ADR-0033): in the design's
  terms, `on` / `off` for a truth value, six significant digits
  (`Brightness(0.5)`, `Held(on)`, `Tilt(0.785398 [rad])`); the JSON trace keeps
  the exact numbers.

The rest of this document states what the generated core must preserve — and,
since the backend milestone, does: `bdl-lower` + `bdl-codegen-rust` produce a
`no_std` core whose `step` is the two-phase tick above with dense slots
(`docs/architecture/executable-ir.md`, `docs/architecture/codegen-rust.md`), and
differential tests hold it to this evaluator trace for trace.

## One generated core, two hosts

```text
              generated core (no_std; runtime/bdl-runtime-core vocabulary)
                 ↑                          ↑
     host bridge (runtime/bdl-runtime-host)   firmware (Embassy adapter, planned)
```

Only the input provider, the clock activation source, the output adapter and the
telemetry transport differ. The simulator does **not** get a separate
interpretation; the reference evaluator exists for differential testing and
stays the oracle. Today the host bridge feeds a JSON run request (active clock
slots and input slots per tick) and returns every declaration's value, every
output and the first structured error.

## Declarations are not tasks

Never one async task per declaration. Per clock domain, one deterministic step
function:

```text
hardware timer / interrupt
  → activate ClockDomain c
  → step_c(prev_state, inputs, &mut next_state, &mut outputs)
  → publish snapshot of c
  → commit physical outputs owned by c
```

BDL meaning never depends on executor task order.

## Synchronous state: previous/next

`delay init e` reads the _previous_ activation's state and writes the next; it
never observes an update made earlier in the same logical tick. The generated
`step` copies the committed `Cells` into `next`, reads only `prev`, writes only
`next`, and assigns `state.cells = next` at the end — not at all on error. Every
delay carries its explicit initial value (a cell is `Option<T>`, `None` until
first written, and reads `init` evaluated now while `None`); there is no
implicit zero.

## Clock domains and `sync`: strictly before

`sync src init e` reads the _last committed snapshot of `src`_ from an
activation strictly before the current tick; `init` if there was none. It never
invokes `step_src` recursively. When two domains are ready at the same physical
instant, each observes only the other's previously committed activation — the
scheduler's order is unobservable, exactly as in the formal model. `delay` is
`sync` at the own domain. In generated code a `sync` cell's writer is the
_source_ domain's slot: it is written when `src` is active, read when the owner
is due, and a tick in which both are active sees the value committed by the
previous tick.

## Physical outputs: evaluate, then commit

```text
sample inputs → evaluate → next state → publish snapshot → final OutputId values → commit via adapter
```

Only the single final driver of each `OutputId` reaches the adapter. No mapping
block touches the HAL; supplied components never receive GPIO/PWM handles or
output writers. All combination (priority, blend, max, clamp) is ordinary
computation upstream of the one drive edge, and code generation must preserve
that: never two writers to one actuator, never a hidden first-wins / last-wins /
task-priority rule.

## Numerics

The formal model computes over `Nat`. The generated core computes over IEEE
`f64`, exactly as the reference evaluator (DI-15): division by an exact zero and
any non-finite primitive result fail the tick with a structured error;
`bdl-runtime-core::num` is the one place that policy lives on the generated
side. `f32` on device is a future, separately recorded deviation, not something
the backend does today (DI-28). Floating-point non-associativity means symbolic
normalization may not silently rewrite formulas; any rewrite records a numeric
obligation (DESIGN_ISSUES DI-1); the backend performs no rewrite.

## Collections, grouped values and the recursor

A collection (`list τ`), a grouped value (`τ × σ`) and an optional value are
data like any other: a declaration may hold one, a `delay` remembers one, a
`sync` transports one — under the rules above unchanged, so a transported
collection is the source's collection at its last activation _strictly before_
now, never the same tick's. That is what lets the Phase-9a lossless window be
written as five ordinary declarations (`log @src := cons x (delay [] log)`,
`logD @dst := sync src [] log`, `seen := length logD`, `cursor := delay 0 seen`,
`window := reverse (take (seen − cursor) logD)`; corpus case `buffer`): no
buffer primitive, no scheduler order, no same-tick visibility, no implicit
overflow. Capacity — that no window exceeds what a deployment can hold — is a
validation obligation, computed on the lowered plan and the deployment schedule
and never a change to these rules (`docs/spec/deployment-capacity.md`,
ADR-0027); the deployable form bounds the log with `take cap` and keeps the
count apart.

`fold f z [x₁, …, xₙ] = f x₁ (… (f xₙ z))` is finite iteration from the last
element: one application of `f` per element, no frame per element, and it is the
only place a function value is applied during evaluation. Every equation of the
library (`docs/spec/equation-library.md`) is a definition over it; the reference
evaluator, the executable-IR interpreter and the generated core compute it the
same way, and the corpus case `collections` holds them to each other. A list
operator's count (`take`, `drop`, `length`) is a dimensionless `f64` read as a
whole number towards zero, never below zero (the numeric deviation above).
Equality on data is structural (exact `f64` equality on quantities); no data
kind but a quantity has an order at runtime — an ordered concept compares
through its quantity representation, decided at elaboration.

## Semantic newtypes

Generated Rust keeps one newtype per concept — `pub struct Sem0(pub f64)` for
`Tilt`, named by stable id — everywhere a value is carried (inputs, declaration
values, cells, outputs), so the Rust compiler catches category mistakes at every
boundary. Internal generated code may erase wrappers where proven safe, later;
today it never does.

## Telemetry

Samples are tagged by stable ids (`DeclId`, `OutputId`, `ClockId`), activation
index and value, so Studio can show live values on the canvas (`Tilt 31.4°`,
`Brightness 0.62`).
