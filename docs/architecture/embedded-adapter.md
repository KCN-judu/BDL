---
kind: architecture
area: runtime
status: current
---

# The embedded platform adapter

How an already-lowered raw command becomes a physical effect on a board, without
the board reaching back into the design. The first target is the Raspberry Pi
Pico (RP2040) over Embassy; the decision is ADR-0037; the boundary it consumes
is `docs/architecture/output-realization.md`; the runtime semantics it must not
alter are `docs/spec/runtime-semantics.md` and ADR-0004.

## The machine command boundary

```text
logical Output ─▶ realization profile ─▶ pure encoder ─▶ SinkPlan ─▶ Tick.commands
                                                                        │
                        (behavior ends here; nothing below is a term)   │
                                                                        ▼
                                 adapter::apply(tick, sink, …) ─▶ PwmDuty8 / Level sink
                                                                        │
                                                        bdl-runtime-embassy-rp ─▶ PWM slice / GPIO pad
```

The generated core's `Tick { values, outputs, commands }` is the last thing
behavior produces: `Commands.command_<device id>` is the profile's encoder
applied to the driver's value, `None` when the driver was not due. The adapter
reads `Tick.commands` and nothing else — no declaration, no output, no concept —
and turns each command into one peripheral operation. It performs the effect; it
interprets nothing.

## Ownership

| Layer                         | Crate / file                                                                                          | Owns                                                                                                                                   | Never                                          |
| ----------------------------- | ----------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| target-independent vocabulary | `runtime/bdl-runtime-embassy` (`no_std`, depends on `bdl-runtime-core` only)                          | the numeric policy (`duty8`), the sink traits (`PwmDuty8`, `Level`), `apply_duty8` / `apply_level`, `CommandFault`, `schedule::active` | a HAL type, a pin, allocation                  |
| generated glue                | `<crate>/src/adapter.rs` (feature `adapter`)                                                          | `SINKS` (bindings as data), `Applied`, `apply(tick, sink₁, …)` — one `&mut dyn` sink per machine sink in sink order                    | a map, a name lookup, a string dispatch        |
| generated firmware            | `<crate>/src/bin/rp2040.rs` (feature `rp2040`), `memory.x`, `build.rs`, `.cargo/config.toml`          | constructing the sinks on the assigned pads, the tick loop, the schedule, the arena, the fault halt                                    | a pin choice, a value the core did not produce |
| RP2040 binding                | `runtime/bdl-runtime-embassy-rp` (outside the workspace: the HAL tree stays out of the host lockfile) | `PwmA` / `PwmB` / `Line` over `embassy-rp`, the PWM carrier, the startup levels, `halt`, `arena`                                       | reading a design                               |
| host counterpart              | `bdl-runtime-host::mock` (`MockPwm`, `MockLine`), `TickTrace.adapter`, `AdapterOp`                    | the same `apply` over recording sinks, so a host trace carries the firmware's operation sequence                                       | replacing the host path                        |
| the plan                      | `bdl-compiler::target::adapter_plan` → `bdl-codegen-rust::adapter::AdapterPlan`                       | binding each machine sink to the solver-assigned resource; refusing what cannot be bound                                               | allocating; a second solver                    |

The core crate (`src/lib.rs`) is unchanged by a target except for one
`#[cfg(feature = "adapter")] pub mod adapter;` line; `cargo check --lib` of the
core still compiles with no feature and no HAL. The `host` feature enables
`adapter` so the host bridge records what the firmware would do.

## Command sink identity

One stable route, no display name anywhere on it:

```text
DeviceBinding.id (DeviceId)
  = SinkPlan.device = Commands.command_<id> = manifest sinks[].device_id
  → deployment.requirements[{ device: id, capability }] (the kind's template)
  → deployment.assignment[that requirement] = ResourceId ("GP15")   (the solver's)
  → AdapterPlan.sinks[].resource = manifest adapter.bindings[].resource = SINKS[].resource
  → targets::rp2040::peripheral: GPn ⇒ PIN_n; PWM ⇒ slice (n/2) % 8, channel A|B by parity
  → src/bin/rp2040.rs: `let mut command_<id> = PwmB::new(p.PWM_SLICE7, p.PIN_15, …)`
```

`GPn` is the board file's resource id — identity, as the board names it — and
the target entry derives the peripheral from the pad number; the board file's
PWM unit (the slice) is checked against that derivation and a disagreement is an
error. Every step is checked, none has a fallback:

| Situation                                                     | Where it stops                | Code                              |
| ------------------------------------------------------------- | ----------------------------- | --------------------------------- |
| the placement is not feasible or complete                     | `adapter_plan`                | `adapter.deployment_not_feasible` |
| a chosen realization is not admissible                        | `adapter_plan`                | `adapter.realization_invalid`     |
| a sink's device has no assigned resource for its capability   | `adapter_plan`                | `adapter.sink_unbound`            |
| the assigned resource lacks the capability                    | `adapter_plan`                | `adapter.resource_incompatible`   |
| the profile has no sink (`i2c_level8`, `hbridge_signed`)      | `adapter_plan`                | `adapter.profile_unsupported`     |
| the design has a Source (no device provides values, ISS-0016) | `adapter_plan`                | `adapter.inputs_unbound`          |
| a collection is input-bounded or unbounded                    | `adapter_plan`                | `adapter.collections_unbounded`   |
| the base tick is zero                                         | `adapter_plan`                | `adapter.tick_invalid`            |
| a resource is not a pad of the target, or the slice disagrees | `targets::rp2040::peripheral` | `backend.internal_lowering`       |

A device that is placed but realises nothing (no profile) gets no sink and no
peripheral: its pad is left unconfigured, and no value is invented for it.

## Numeric policy at the boundary

Production computes in `f64` (ISS-0006 stays open for the core's own
arithmetic). At the boundary one explicit policy converts a raw command to what
the peripheral takes, `bdl_runtime_embassy::duty8`:

| Raw duty command (`q[1]`, from `pwm_duty8` / `pwm_duty4`) | Result                                                                     |
| --------------------------------------------------------- | -------------------------------------------------------------------------- |
| finite, in `0 ..= 255`                                    | nearest whole duty, halves up: `127.5 → 128`, `31.875 → 32`, `254.5 → 255` |
| `0`, `255`                                                | `0`, `255` exactly                                                         |
| finite, below `0` or above `255`                          | refused: `CommandFault::OutOfRange { 0, 255 }`                             |
| NaN, ±∞                                                   | refused: `CommandFault::NotFinite`                                         |

A refused command **writes nothing**: the line holds its last applied value, and
the refusal is recorded (`Applied`, `AdapterOp::Refused`). No clamping — a
design that commands 105 % has said something the profile did not promise to
carry, and the peripheral holds rather than guesses. A `bool` command
(`gpio_level`) is applied as written; nothing can be refused. The policy is a
function of the value alone, deterministic, and tested
(`duty8_rounds_halves_up_and_keeps_the_ends`,
`duty8_refuses_out_of_range_and_non_finite`).

The encoder stays what it is: `pwm_duty8` still carries `127.5` for 50 % in the
raw trace, on the host and on the device; the adapter, not the encoder, decides
the register value.

## Raw value → peripheral value (RP2040)

`PwmA` / `PwmB` configure their slice with `top = 254`, so `compare = d` is high
for exactly `d / 255` of the period: `0` is always low, `255` always high. The
carrier is the 125 MHz system clock divided by `DEFAULT_PWM_DIVIDER = 16` over
255 steps ≈ 30.6 kHz — peripheral configuration, never a BDL clock domain.
`Line` is `Output::new(pin, Low)` and `set_level` writes the truth value.

## Clocks and the tick loop

The firmware invents no clock semantics. It fires one Embassy `Ticker` at
`TICK_MICROS` (`bdld compile --tick-micros`, default 10 000 µs) and, at global
tick `t`, activates the clock slots the compiled schedule says are due:
`PERIODS[slot]` in ticks (`bdld compile --period domain=N`, default

1. and `schedule::active(t, &PERIODS)` — `tick % period == 0`, the simulator's
   `Schedule::active_at` verbatim, tested against it. Then:

```text
tick:  ticker.next().await
       active = schedule::active(tick, &PERIODS)
       match design::step(&mut state, active, &inputs)   // one global step, ADR-0004
         Ok(t)  => adapter::apply(&t, &mut command_a, &mut command_b, …)  // sink order
         Err(_) => halt()
```

Commands are applied **after** the step returns, in sink order, from the same
`Tick` the interpreter and the host see — the same order as
`TickResult.commands`. A domain that is not due leaves its sinks untouched
(`Applied.command_x = None`, `AdapterOp::Held`). No cross-domain synchronization
happens in the adapter: the compiler resolved or refused it upstream.

## Startup, faults, reset

- **Startup:** every PWM line is configured at duty 0 and every digital line low
  before the first tick (`PwmA::new` / `PwmB::new` / `Line::new`).
  Peripheral-library defaults never decide a line's first state.
- **Fault:** a tick that fails (`RuntimeError` — a due input without a value, a
  division by zero, a non-finite result) does not advance the state, and the
  firmware **halts**: every line holds its last applied value and the core waits
  for interrupts forever (`halt`). A refused command is not a fault: it holds
  one line and ticking continues.
- **Reset:** a hardware reset restarts at startup (every line low). There is no
  shutdown path and no safety system beyond this.
- **Panics** (`handle_alloc_error`, a HAL invariant) go to `panic-halt`: the
  same hold-and-wait, by a different door.

## The collection arena

A `bounded` design (docs/spec/deployment-capacity.md) needs an allocator. The
plan sizes a static arena from the manifest's bounds —
`state_bytes_max + tick_bytes_max` rounded up to whole KiB — and the firmware
declares `#[global_allocator] static HEAP: Heap` (`embedded-alloc`'s linked-list
heap) over `static ARENA: StaticCell<[u8; N]>`, handed over once before the
first tick. A scalar-only design declares no allocator at all (the core stays
`Copy`, allocation-free). An `input_bounded` or `unbounded` design is refused
for a board. Exhaustion is impossible for a `bounded` design by the report's
bound; were it to happen it would be an allocation error → panic → halt, never a
dropped value.

## Physical effect boundary

Phase 14 (FV) proves the model up to the raw command relation
(`lower_correspondence`); the host tests prove the generated `Commands` against
the interpreter; this page's tests prove the generated `adapter::apply` over
recording sinks against those commands and the policy above, and that the
firmware cross-compiles. What a PWM slice or a pad does with the register write
is `embassy-rp` and silicon: the correspondence between the raw command trace
and the physical effect is **not formally proved** (FVI-0022, ISS-0017) and this
page claims none — the adapter is _production-tested_.

## Out of scope, by design

Stateful adapters (slew, dithering, hysteresis), a device clock other than the
output's, atomic multi-value frames, buffered or queued peripherals,
backpressure, I²C and H-bridge sinks, flashing (roadmap priority 3), telemetry
(priority 4), a second MCU (priority 5), and `bdld` orchestrating the cargo
build (priority 2).

## Building the firmware

```bash
bdld compile <project> --out target/bdl --target rp2040_pico --tick-micros 10000 --period main=1
cd target/bdl && cargo build --release --target thumbv6m-none-eabi --features rp2040
```

The crate's `.cargo/config.toml` carries the linker inputs and the
MSRV-respecting resolver (`incompatible-rust-versions = "fallback"`: the HAL's
newest dependencies may want a newer compiler than the toolchain pins);
`rust-toolchain.toml` lists the target. The host bridge is
`cargo build --features host` as before.

## Evidence

`runtime/bdl-runtime-embassy/src/lib.rs` (the policy and the schedule),
`crates/bdl-codegen-rust/src/targets/rp2040.rs` (pad → peripheral, no fallback),
`crates/bdl-compiler/tests/embedded_rp2040.rs` (sink ↔ pad ↔ peripheral chain,
the host operations against `Tick.commands` for 0 %/5 %/50 %/100 %/out of range,
the quantized profile, the core and its commands unchanged by the target, the
refusals, the arena, the schedule, determinism, the cross-build),
`crates/bdl-daemon/tests/cli.rs` (`compile --target`),
`crates/bdl-hardware/tests/solver.rs` (the Pico).
