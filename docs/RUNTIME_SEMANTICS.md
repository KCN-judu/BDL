# Runtime semantics

How the kernel's tick semantics (`Ev` / `MEv`, BDL_FV `Reactive.lean`,
`Clock.lean`) become executable Rust. Nothing in this document is
implemented yet; it fixes the rules the generated core and the runtime
adapters must obey.

## One generated core, two hosts

```
              bdl-generated-core (no_std)
                 ↑              ↑
     host simulation        firmware (Embassy adapter first)
```

Only the input provider, the clock activation source, the output adapter and
the telemetry transport differ. The simulator does **not** get a separate
interpretation; a reference interpreter exists for differential testing.

## Declarations are not tasks

Never one async task per declaration. Per clock domain, one deterministic
step function:

```
hardware timer / interrupt
  → activate ClockDomain c
  → step_c(prev_state, inputs, &mut next_state, &mut outputs)
  → publish snapshot of c
  → commit physical outputs owned by c
```

BDL meaning never depends on executor task order.

## Synchronous state: previous/next

`delay init e` reads the *previous* activation's state and writes the next;
it never observes an update made earlier in the same logical tick. A
double buffer (`prev` / `next`) is the initial implementation. Every delay
carries its explicit initial value; there is no implicit zero.

## Clock domains and `sync`: strictly before

`sync src init e` reads the *last committed snapshot of `src`* from an
activation strictly before the current tick; `init` if there was none. It
never invokes `step_src` recursively. When two domains are ready at the same
physical instant, each observes only the other's previously committed
activation — the scheduler's order is unobservable, exactly as in the formal
model. `delay` is `sync` at the own domain.

## Physical outputs: evaluate, then commit

```
sample inputs → evaluate → next state → publish snapshot → final OutputId values → commit via adapter
```

Only the single final driver of each `OutputId` reaches the adapter. No
mapping block touches the HAL; supplied components never receive GPIO/PWM
handles or output writers. All combination (priority, blend, max, clamp) is
ordinary computation upstream of the one drive edge, and code generation
must preserve that: never two writers to one actuator, never a hidden
first-wins / last-wins / task-priority rule.

## Numerics

The formal model computes over `Nat`. The generated core computes over IEEE
floats (`f32` on device, `f64` on host unless configured). Floating-point
non-associativity means symbolic normalization may not silently rewrite
formulas; any rewrite records a numeric obligation (DESIGN_ISSUES DI-1).

## Semantic newtypes

Generated Rust keeps `struct Tilt(f32)`, `struct Brightness(f32)` at
boundaries — supplied components, public APIs, device bindings, host
integration — so the Rust compiler catches category mistakes there. Internal
generated code may erase wrappers where proven safe.

## Telemetry

Samples are tagged by stable ids (`DeclId`, `OutputId`, `ClockId`),
activation index and value, so Studio can show live values on the canvas
(`Tilt 31.4°`, `Brightness 0.62`).
