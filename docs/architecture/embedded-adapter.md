---
kind: architecture
area: runtime
status: current
---

# The embedded platform adapter

How an already-lowered raw command becomes a physical effect on a board, without
the board reaching back into the design. The first target is the Raspberry Pi
Pico (RP2040) over Embassy; the second family is Arduino over `avr-hal`, with
the Nano first; the decision is ADR-0037 (amended for the second family); the
boundary they consume is `docs/architecture/output-realization.md`; the runtime
semantics they must not alter are `docs/spec/runtime-semantics.md` and ADR-0004.

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

| Layer                         | Crate / file                                                                                          | Owns                                                                                                                                                       | Never                                          |
| ----------------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| target-independent vocabulary | `runtime/bdl-runtime-adapter` (`no_std`, depends on `bdl-runtime-core` only; every family shares it)  | the numeric policy (`duty8`), the sink traits (`PwmDuty8`, `Level`), `apply_duty8` / `apply_level`, `CommandFault`, `schedule::active`                     | a HAL type, a pin, allocation                  |
| generated glue                | `<crate>/src/adapter.rs` (feature `adapter`)                                                          | `SINKS` (bindings as data), `Applied`, `apply(tick, sink₁, …)` — one `&mut dyn` sink per machine sink in sink order                                        | a map, a name lookup, a string dispatch        |
| generated firmware            | `<crate>/src/bin/rp2040.rs` (feature `rp2040`), `memory.x`, `build.rs`, `.cargo/config.toml`          | constructing the sinks on the assigned pads, the tick loop, the schedule, the arena, the fault halt                                                        | a pin choice, a value the core did not produce |
| RP2040 binding                | `runtime/bdl-runtime-embassy-rp` (outside the workspace: the HAL tree stays out of the host lockfile) | `PwmA` / `PwmB` / `Line` over `embassy-rp`, the PWM carrier, the startup levels, `halt`, `arena`                                                           | reading a design                               |
| Arduino binding               | `runtime/bdl-runtime-arduino` (outside the workspace: a git dependency on `avr-hal`, nightly-only)    | `PwmLine` / `Line` over `avr-hal`'s pins, the initial levels, `tick_wait`, `halt`                                                                          | reading a design                               |
| target entries                | `bdl-codegen-rust::targets::Entry` — `rp2040`, `arduino` (the base) with `arduino::NANO` (the board)  | the resource → peripheral derivation, the firmware module, the files beside it, the Cargo sections, the build command, whether a family can carry an arena | choosing a resource; a second solver           |
| host counterpart              | `bdl-runtime-host::mock` (`MockPwm`, `MockLine`), `TickTrace.adapter`, `AdapterOp`                    | the same `apply` over recording sinks, so a host trace carries the firmware's operation sequence                                                           | replacing the host path                        |
| the plan                      | `bdl-compiler::target::adapter_plan` → `bdl-codegen-rust::adapter::AdapterPlan`                       | binding each machine sink to the solver-assigned resource; refusing what cannot be bound                                                                   | allocating; a second solver                    |

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
  → the target entry's derivation
      rp2040:  GPn ⇒ PIN_n; PWM ⇒ slice (n/2) % 8, channel A|B by parity
      arduino: Dn ⇒ pins.dn, An ⇒ pins.an; PWM ⇒ the board table's timer (Nano: D3/D11 → TC2, D5/D6 → TC0, D9/D10 → TC1)
  → src/bin/rp2040.rs:       `let mut command_<id> = PwmB::new(p.PWM_SLICE7, p.PIN_15, …)`
     src/bin/arduino_nano.rs: `let mut command_<id> = PwmLine::new(pins.d3.into_output().into_pwm(&timer2))`
```

`GPn` / `Dn` is the board file's resource id — identity, as the board names it —
and the target entry derives the peripheral from it; the board file's PWM unit
(the slice, the timer) is checked against that derivation and a disagreement is
an error. Every step is checked, none has a fallback:

| Situation                                                     | Where it stops           | Code                              |
| ------------------------------------------------------------- | ------------------------ | --------------------------------- |
| the placement is not feasible or complete                     | `adapter_plan`           | `adapter.deployment_not_feasible` |
| a chosen realization is not admissible                        | `adapter_plan`           | `adapter.realization_invalid`     |
| a sink's device has no assigned resource for its capability   | `adapter_plan`           | `adapter.sink_unbound`            |
| the assigned resource lacks the capability                    | `adapter_plan`           | `adapter.resource_incompatible`   |
| the profile has no sink (`i2c_level8`, `hbridge_signed`)      | `adapter_plan`           | `adapter.profile_unsupported`     |
| the design has a Source (no device provides values, ISS-0016) | `adapter_plan`           | `adapter.inputs_unbound`          |
| a collection is input-bounded or unbounded                    | `adapter_plan`           | `adapter.collections_unbounded`   |
| the base tick is zero                                         | `adapter_plan`           | `adapter.tick_invalid`            |
| the board has no target entry (the mock `big_board`)          | `adapter_plan`           | `adapter.target_unsupported`      |
| a family with no arena (Arduino) and a `bounded` design       | `adapter_plan`           | `adapter.collections_unsupported` |
| a resource is not a pad of the target, or the slice disagrees | the entry's `peripheral` | `backend.internal_lowering`       |

A device that is placed but realises nothing (no profile) gets no sink and no
peripheral: its pad is left unconfigured, and no value is invented for it.

## Numeric policy at the boundary

Production computes in `f64` (ISS-0006 stays open for the core's own
arithmetic). At the boundary one explicit policy converts a raw command to what
the peripheral takes, `bdl_runtime_adapter::duty8`:

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

## Target entries

`bdl-codegen-rust::targets::Entry` is the one place a family is known:
`Entry::for_board(family, id)` picks it from the board file's `family` and id,
and everything upstream — the plan, the glue, the host's recorded operations —
is the same for every family. An entry answers: the peripheral behind a
resource, the firmware module, the files beside it (linker inputs, the cargo
config), the Cargo sections of its feature, the build command, the toolchain it
needs beyond the pinned one, and whether the family can host a collection arena.
A second board of a family adds a board table, not an entry; a second family
adds an entry and touches nothing upstream.

| Entry             | Boards                  | HAL                     | Triple               | Toolchain            | Tick                 | Arena |
| ----------------- | ----------------------- | ----------------------- | -------------------- | -------------------- | -------------------- | ----- |
| `rp2040`          | `rp2040_pico`           | `embassy-rp` (async)    | `thumbv6m-none-eabi` | the pinned 1.89      | Embassy `Ticker`     | yes   |
| `arduino` (`avr`) | `arduino_nano` (`NANO`) | `avr-hal` (synchronous) | `avr-none`           | `nightly-2025-04-27` | blocking `tick_wait` | no    |

## Arduino

The Arduino family is a base and a board. The base (`targets::arduino`) owns
what every Arduino shares: the pin naming (`Dn` ⇒ `pins.dn`, `An` ⇒ `pins.an`
for the analog pins that are also lines), the PWM timers behind the pins
(`Timer0Pwm` / `Timer1Pwm` / `Timer2Pwm`, constructed only for the timers the
plan's sinks use), the blocking tick loop, the halt, the Cargo sections
(`arduino-hal` from its git commit `e0b0105b`, `panic-halt`,
`bdl-runtime-arduino`). A board (`arduino::Board`) says its id, its
`arduino-hal` feature and CPU, how many `D`/`A` lines it has, and which line
sits on which timer — the Nano: D3/D11 on TC2, D5/D6 on TC0, D9/D10 on TC1, the
board file's units, checked against the table. The Uno would be a second `Board`
with the same table; the Mega a table of its own.

What differs from the RP2040, as policy:

- **No Embassy.** `avr-hal` is synchronous and the AVR has no Embassy time
  driver: the firmware is `#[arduino_hal::entry] fn main() -> !` with a loop —
  step, apply in sink order, `tick_wait(TICK_MICROS)` (`arduino_hal::delay_us`).
  The wait does not subtract the step's own duration, so a tick lasts _at least_
  `TICK_MICROS`; a timer-driven tick is ISS-0017's.
- **PWM is 8-bit natively:** `avr-hal`'s `set_duty(u8)` takes the boundary's
  `duty8` value as the register value — `0` always low, `255` always high — on a
  carrier of F_CPU / (64 · 256) ≈ 977 Hz at 16 MHz (`Prescaler::Prescale64`,
  configuration). Lines start low; `PwmLine::new` sets duty 0 before enabling.
- **No collection arena.** 2 KiB of SRAM host no allocator: a `bounded` design
  is refused for the Nano (`adapter.collections_unsupported`); scalar-only
  designs are what the family runs.
- **Nightly-only build.** `avr-none` is a tier-3 target: the firmware is
  `cargo +nightly-2025-04-27 build --release --target avr-none -Zbuild-std=core --features arduino_nano`
  with `avr-gcc` as the linker; the generated crate declares
  `rust-version = "1.87"` and the resolver's fallback keeps the HAL's
  dependencies buildable on that pre-1.88 nightly (`encoding_rs` 0.8.35, not
  0.8.41). The host build of the same crate stays on the pinned stable.
- **`f64` on an 8-bit MCU** is software floating point: the core computes as
  generated (ISS-0006 unchanged); `duty8` converts at the boundary.
- **The fault halt** is `avr_device::asm::sleep()` in a loop: lines hold, the
  MCU sleeps until reset.

The generated Nano firmware for the lamp (a PWM line and a digital line) links
to an AVR ELF in CI (`the_firmware_cross_compiles_for_the_nano`,
`BDL_REQUIRE_AVR=1`); nothing has been flashed (roadmap priority 3).

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
(priority 4), a third family (the ESP32-S3, priority 5), and `bdld`
orchestrating the cargo build (priority 2).

## Building the firmware

```bash
bdld compile <project> --out target/bdl --target rp2040_pico --tick-micros 10000 --period main=1
cd target/bdl && cargo build --release --target thumbv6m-none-eabi --features rp2040
```

```bash
bdld compile <project> --out target/bdl --target arduino_nano
cd target/bdl && cargo +nightly-2025-04-27 build --release --target avr-none -Zbuild-std=core --features arduino_nano
```

The manifest's `adapter.build` carries the exact command for the board.

The crate's `.cargo/config.toml` carries the linker inputs and the
MSRV-respecting resolver (`incompatible-rust-versions = "fallback"`: the HAL's
newest dependencies may want a newer compiler than the toolchain pins);
`rust-toolchain.toml` lists the target. The host bridge is
`cargo build --features host` as before.

## Evidence

`runtime/bdl-runtime-adapter/src/lib.rs` (the policy and the schedule),
`crates/bdl-codegen-rust/src/targets/rp2040.rs` (pad → peripheral, no fallback),
`crates/bdl-compiler/tests/embedded_rp2040.rs` (sink ↔ pad ↔ peripheral chain,
the host operations against `Tick.commands` for 0 %/5 %/50 %/100 %/out of range,
the quantized profile, the core and its commands unchanged by the target, the
refusals, the arena, the schedule, determinism, the cross-build),
`crates/bdl-daemon/tests/cli.rs` (`compile --target`),
`crates/bdl-hardware/tests/solver.rs` (the Pico).
