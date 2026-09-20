---
id: ADR-0037
status: accepted
date: 2026-09-20
area: runtime
supersedes: []
superseded-by: []
related: [ADR-0004, ADR-0016, ADR-0027, ADR-0036, ISS-0006, ISS-0016, ISS-0017]
fv:
  [
    "production-tested: `crates/bdl-compiler/tests/embedded_rp2040.rs`,
    `runtime/bdl-runtime-embassy/src/lib.rs`; the raw-command → physical-effect
    correspondence is not formally proved (FVI-0022)",
  ]
---

# ADR-0037: The platform adapter consumes the generated raw commands — first on the RP2040 over Embassy — with an explicit boundary policy, generated glue, and no semantics of its own

## Status

Accepted (the first embedded platform adapter, roadmap priority 1, 2026-09-20).
Builds on ADR-0004 (one step function, adapters activate and commit), ADR-0016
(the core is an implementation of the reference evaluator), ADR-0027
(collections bounded by the design) and ADR-0036 (the raw command boundary).
Changes none of them.

## Context

After ADR-0036 a generated core ends at `Tick.commands`: per realised output,
the value a pure encoder made of the output's value. Nothing performed an
effect. The roadmap's first priority was an Embassy adapter that turns timers
into domain activations and commits outputs; with the command boundary in place,
"commit outputs" means "apply raw commands to peripherals". Constraints: no
effect, `IO`, `Action` or `R → ()` in the language; the adapter reads no
declaration; the kernel, the encoders, `SinkPlan`, the simulation and the host
runtime stay as they are; the board file is the authority on resources; `f64`
reaches the boundary and the device takes integers (ISS-0006); the design's
Sources have no device yet (ISS-0016); FVI-0022 leaves the physical
correspondence unproved.

## Decision

1. **The adapter consumes `Tick.commands` and nothing else.** Generated glue
   (`src/adapter.rs`) exposes `apply(tick, sink₁, …)` with one
   `&mut dyn PwmDuty8` / `&mut dyn Level` parameter per machine sink, in sink
   order; the firmware and the host bridge both call it. No map, no name, no
   string dispatch, no read of a value upstream of the commands.
2. **Two runtime crates.** `bdl-runtime-embassy` (`no_std`, depends on
   `bdl-runtime-core` only) holds the vocabulary — numeric policy, sink traits,
   `apply_*`, `CommandFault`, `schedule::active`. `bdl-runtime-embassy-rp` holds
   the RP2040 binding over `embassy-rp` and lives **outside the workspace** so
   the HAL's dependency tree never enters the host lockfile; it is built only
   into generated firmware.
3. **The first target is the Raspberry Pi Pico** (`rp2040_pico`, family
   `rp2040`, `thumbv6m-none-eabi`), a board file like the others. The target
   entry derives the peripheral from the pad number (`GPn` ⇒ `PIN_n`; PWM slice
   `(n/2) % 8`, channel by parity) and checks the board file's slice against it.
   No second MCU in this record.
4. **Command sink identity is the device id and the solver's resource id.**
   `DeviceBinding.id` names the `Commands` field, the manifest sink, the
   requirement and the adapter binding; the assignment's `ResourceId` names the
   pad. `adapter_plan` refuses, never substitutes: an unplaced sink, a resource
   without the capability, a profile with no sink, a design with Sources, an
   unbounded collection, an infeasible placement.
5. **The numeric policy at the boundary is explicit and reject-and-hold.**
   `duty8`: a finite raw duty in `0 ..= 255` rounds to the nearest whole duty,
   halves up; anything else (out of range, NaN, ±∞) is refused and the line
   holds its last value. No clamping, no `as` cast deciding semantics, no change
   to the encoders (`pwm_duty8` still carries `127.5`). `bool` commands are
   applied as written.
6. **PWM register mapping:** `top = 254`, `compare = duty`, so `d` is high for
   `d / 255` of the carrier period; the carrier (divider 16 ≈ 30.6 kHz) is
   peripheral configuration, never a `ClockId`.
7. **Clock activation is the compiled schedule.** One `Ticker` at
   `--tick-micros`; at global tick `t` the slots with `t % PERIODS[slot] == 0`
   are active — the simulator's rule, in `bdl-runtime-embassy`. One global
   `step`, then `apply` in sink order (the interpreter's order). No
   synchronization in the adapter.
8. **Startup and faults are adapter policy.** Every line starts low / duty 0
   before the first tick. A failed tick latches: state unchanged, lines hold,
   the firmware waits for interrupts until reset. A refused command is not a
   fault.
9. **The arena is sized from the manifest.** A `bounded` design gets
   `#[global_allocator]` over a static arena of
   `state_bytes_max + tick_bytes_max` rounded up to KiB (`embedded-alloc`,
   `static_cell`); a scalar-only design declares no allocator; an
   `input_bounded` or `unbounded` design is refused for a board.
10. **The host path records the same operations.** `HostProgram::adapter_ops`
    applies the generated `apply` to `MockPwm` / `MockLine`; `TickTrace.adapter`
    carries the operations beside `commands`. Nothing about the host path is
    replaced.
11. **`bdld compile --target <board> [--tick-micros N]`** generates the firmware
    beside the core; building it is
    `cargo build --release --target thumbv6m-none-eabi --features rp2040` in the
    generated crate. Orchestrating that build is roadmap priority 2, not this
    record.

## Alternatives

- **One task per declaration, channels for wires** — ADR-0004 already refused
  it; the adapter keeps one step and one apply.
- **A runtime dispatch table (`HashMap<String, f64>`, symbol strings)** — would
  make binding a runtime lookup and a display name an identity; refused for
  generated static parameters.
- **Clamping out-of-range duties** — hides that the design commanded what the
  profile did not promise; reject-and-hold keeps the last honest value and
  records the refusal. Recorded here so it can be revisited with evidence.
- **`f64 → u8 as` casts** — saturating and rounding toward zero by accident; the
  policy is a function with a table and tests.
- **Rounding to even, or truncating** — either is a valid policy; halves up was
  chosen for the plain reading of "nearest" and is documented as the policy, not
  a truth.
- **The HAL crate inside the workspace** — would pull ~60 crates into every host
  build and lockfile and pin the workspace's dependency resolution to the HAL's
  MSRV; kept outside, with its own resolver setting.
- **Hand-written firmware per project** — generation from the plan keeps the
  binding a compiler output and reviewable; a project may still add its own
  binary beside the generated one.
- **Modelling the PWM carrier as a clock domain** — refused (the brief's rule
  and FVD-0138): it is configuration.

## Consequences

- A design still defines only logical outputs; realization lowers them to raw
  commands; one generated target consumes those commands; the solver-assigned
  pad receives them; the core, the simulation trace, the host runtime and
  `Tick.commands` are unchanged (tested).
- New: `bdl-runtime-embassy`, `bdl-runtime-embassy-rp`,
  `hardware/boards/rp2040_pico.toml`,
  `bdl-codegen-rust::{adapter, targets::rp2040}`, `bdl-compiler::target`,
  manifest `adapter`, `TickTrace.adapter`, `HostProgram::adapter_ops`,
  `rust-toolchain.toml` `targets`, the `embedded-rp` preflight check, the
  cross-build test in CI (`BDL_REQUIRE_CROSS=1`).
- Every generated host now implements `adapter_ops` (goldens regenerated).
- Not decided here: the core's numeric representation on device (ISS-0006 stays
  deferred), Source bindings (ISS-0016), the physical correspondence and the
  FVI-0022 items (ISS-0017), flashing, telemetry, build orchestration, a second
  target.
- Records: `docs/architecture/embedded-adapter.md`,
  `docs/spec/deployment-capacity.md` § 6–7, `docs/spec/runtime-semantics.md`,
  `docs/spec/hardware-model.md`, `docs/architecture/codegen-rust.md`,
  `docs/architecture/overview.md`, `docs/project/formal-correspondence.md`,
  `docs/project/roadmap.md`, `docs/project/status.md`, a change fragment, the
  user guide's Deploy and CLI pages.

## Amendment (2026-09-20, the Arduino family)

The vocabulary crate of Decision 2 is `bdl-runtime-adapter` (renamed from
`bdl-runtime-embassy` the same day): a second family — the Arduino Nano over
`avr-hal`, a synchronous HAL with no Embassy — shares the numeric policy, the
sink traits, the glue and the recorded host operations, so the crate's name says
what it is rather than who first used it. `bdl-runtime-embassy-rp` keeps its
name; `bdl-runtime-arduino` is the Arduino binding; the target entries live in
`bdl-codegen-rust::targets` (`Entry`), one per family. Everything else above
stays as written; the Arduino family's own policies (a blocking tick, no
collection arena, nightly-only build) are
`docs/architecture/embedded-adapter.md` § Arduino.
