# The first embedded platform adapter: raw commands reach a Raspberry Pi Pico over Embassy

- Date: 2026-09-20
- Area: runtime, codegen, compiler, hardware, daemon, ci
- Affected: designers, developers, generated crates
- Related: ADR-0037, ADR-0036, ADR-0004, ADR-0027, ISS-0006, ISS-0016, ISS-0017,
  [2026-09 Output realization](2026-09-output-realization.md)

## What changed

- **A design can be compiled for a board.**
  `bdld compile <project> --target rp2040_pico` generates, beside the unchanged
  `no_std` core, the platform adapter: `src/adapter.rs` (each machine sink's raw
  command applied to one sink through the numeric policy), `src/bin/rp2040.rs`
  (one Embassy task that ticks the core at `--tick-micros`, activates the
  domains the compiled `--period` schedule says are due, and applies the
  commands to the pads the placement assigned), `memory.x`, `build.rs` and
  `.cargo/config.toml`.
  `cargo build --release --target thumbv6m-none-eabi --features rp2040` in the
  generated crate is the firmware.
- **The Raspberry Pi Pico is a target** (`rp2040_pico`): 27 pads, PWM on every
  pad but the LED over eight slices, ADC, two I²C, two UART, two SPI — a board
  file like the Nano's; the Deploy page lists it.
- **A PWM sink drives a PWM slice, a GPIO sink drives a pad.** `pwm_duty8` and
  `pwm_duty4` commands become an 8-bit duty on the slice and channel the
  datasheet gives the assigned pad (`top = 254`, so 255 is fully on);
  `gpio_level` commands become the pad's level. `i2c_level8` and
  `hbridge_signed` have no sink yet and are refused for a board.
- **The numeric policy at the boundary is explicit.** A raw duty in `0 ..= 255`
  rounds to the nearest whole duty (halves up: `127.5 → 128`); anything outside,
  or not finite, is refused and the line holds its last value — no clamping, no
  accidental cast. The encoders are unchanged.
- **Startup, faults, schedule, arena are adapter policy.** Every line starts
  low; a failed tick halts the firmware with every line held; the schedule is
  the simulator's (`tick % period == 0`); a design with bounded collections gets
  a static arena sized from the manifest's bounds; a design with a Source, or
  with an unbounded collection, cannot be compiled for a board yet.
- **The host records what the firmware would do.** A generated host's trace
  carries `adapter` beside `commands`: the same generated `apply` over recording
  sinks (`Pwm { duty }`, `Level { high }`, `Refused`, `Held`).

## Compatibility and migration

- Designers: nothing to do; without `--target` `bdld compile` writes what it
  wrote before. A project without a device on every output, or with a Source, is
  refused for a board with the reason.
- Generated crates: every host bridge implements `adapter_ops` (empty when no
  target); regenerate crates generated before this change to build against the
  new `bdl-runtime-host`. `bdl-manifest.json` gains an optional `adapter` entry
  (manifest version unchanged, additive).
- Developers:
  `bdl_runtime_adapter::{duty8, CommandFault, PwmDuty8, Level, apply_duty8, apply_level, SinkBinding, schedule::active}`;
  `bdl_runtime_embassy_rp::{PwmA, PwmB, Line, halt, arena, DUTY8_TOP, DEFAULT_PWM_DIVIDER}`
  (outside the workspace; `just`-less: `cargo` in its directory);
  `bdl_runtime_host::{AdapterOp, mock, TickTrace::adapter, HostProgram::adapter_ops, harness::Cargo::build_firmware}`;
  `bdl_codegen_rust::{adapter::{AdapterPlan, SinkBinding, SinkKind}, targets::rp2040, generate_with_adapter, manifest::AdapterEntry}`,
  `CodegenOptions::{runtime_embassy_path, runtime_embassy_rp_path}`, AST
  `Item::{Mod, Static}`, `Expr::{Array, Loop, Await}`, `Function::is_async`;
  `bdl_compiler::{compile_for_target, adapter_plan, TargetOptions}`;
  `bdl_hardware::boards::rp2040_pico`; `bdld compile --target --tick-micros`.
- Toolchain and CI: `rust-toolchain.toml` lists `thumbv6m-none-eabi` (rustup
  installs it); the workspace tests cross-compile the generated firmware
  (skipped locally without the target unless `BDL_REQUIRE_CROSS=1`, which CI
  sets); the preflight check `embedded-rp` formats and lints the RP2040 crate
  for the embedded triple. The HAL's crates are fetched from crates.io by the
  generated crate's own lockfile.

## Evidence

`runtime/bdl-runtime-adapter/src/lib.rs` (the policy's matrix, hold on refusal,
the schedule against the simulator's rule),
`crates/bdl-codegen-rust/src/targets/rp2040.rs` (pad → peripheral; a foreign pad
or a disagreeing slice is an error),
`crates/bdl-compiler/tests/embedded_rp2040.rs` (the sink ↔ pad ↔ peripheral
chain; the host's operations against `Tick.commands` for 0 %, 5 %, 50 %, 100 %
and out of range; the quantized profile; the core and its commands unchanged by
the target; unplaced sink, incompatible resource, infeasible placement, Source,
unsupported profile, unbounded collections refused; the arena; the schedule;
determinism; the cross-build to an ELF), `crates/bdl-daemon/tests/cli.rs`
(`compile --target`), `crates/bdl-hardware/tests/solver.rs` (the Pico places a
PWM line and a digital output).
