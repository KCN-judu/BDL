# The Arduino family: raw commands reach an Arduino Nano over avr-hal

- Date: 2026-09-20
- Area: runtime, codegen, compiler, daemon, ci
- Affected: developers, generated crates
- Related: ADR-0037 (amended), ISS-0006, ISS-0016, ISS-0017,
  [2026-09 embedded adapter](2026-09-embedded-adapter.md)

## What changed

- **A second family, one entry per family.** `bdl-codegen-rust::targets::Entry`
  is where a board family is known: the peripheral behind a resource, the
  firmware module, the files beside it, the Cargo sections, the build command,
  the toolchain, whether the family can host a collection arena. The RP2040
  entry moved into it unchanged; the plan, the glue and the host's recorded
  operations are family-independent.
- **The Arduino base and the Nano.** `targets::arduino` owns what every Arduino
  shares (`Dn` ⇒ `pins.dn`, the timers behind the PWM pins, a blocking tick loop
  over `avr-hal`, the halt); `arduino::NANO` is the board table (D3/D11 on timer
  2, D5/D6 on timer 0, D9/D10 on timer 1 — the board file's units, checked).
  `bdld compile --target arduino_nano` generates `src/bin/arduino_nano.rs`;
  `cargo +nightly-2025-04-27 build --release --target avr-none -Zbuild-std=core --features arduino_nano`
  builds it (the manifest's `adapter.build` carries the command).
- **The vocabulary crate is `bdl-runtime-adapter`** (was `bdl-runtime-embassy`):
  the numeric policy, the sink traits, the glue and the recorded operations are
  every family's. `bdl-runtime-arduino` is the Arduino binding (`PwmLine`,
  `Line`, `tick_wait`, `halt`), outside the workspace and nightly-only.
- **Arduino policies:** no Embassy (a blocking `tick_wait` after each step, so a
  tick lasts at least `TICK_MICROS`); PWM is 8-bit natively at ≈977 Hz; lines
  start low; no collection arena (`adapter.collections_unsupported` for a
  `bounded` design); `f64` is software floating point on the AVR; the fault halt
  sleeps until reset. A board without an entry (the mock `big_board`) is
  `adapter.target_unsupported`.

## Compatibility and migration

- Designers: nothing to do; `--target rp2040_pico` is as before.
- Developers: `bdl_runtime_embassy` → `bdl_runtime_adapter` in imports and paths
  (`CodegenOptions::runtime_adapter_path`, `runtime_arduino_path`);
  `bdl_codegen_rust::targets::{Entry, Peripheral, arduino::{Board, NANO}}`;
  manifest `adapter.{feature, toolchain, build}`;
  `harness::Cargo::build_firmware_with`; `bdld compile --target arduino_nano`.
- Toolchain and CI: the Linux Rust job installs `nightly-2025-04-27` (with
  `rust-src`, `clippy`) and `gcc-avr`, runs the `embedded-avr` check and
  requires the Nano cross-build (`BDL_REQUIRE_AVR=1`); locally both skip,
  visibly, where the toolchain is absent
  (`rustup toolchain install nightly-2025-04-27 --component rust-src`, `avr-gcc`
  on PATH — the Arduino IDE's copy works).

## Evidence

`crates/bdl-compiler/tests/embedded_arduino_nano.rs` (sink ↔ line ↔ timer by
device id and resource id; the same core and glue as the Pico's; the host's
operations; collections and a foreign family refused; a line without PWM
refused; the AVR ELF), `crates/bdl-codegen-rust/src/targets/arduino.rs` (the
board table, pin fields, no fallback), `crates/bdl-daemon/tests/cli.rs`
(`compile --target arduino_nano`), `runtime/bdl-runtime-arduino` (clippy-clean
for `avr-none`).
