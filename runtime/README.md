# runtime

Crates linked into, or driving, generated programs — never into the compiler.

- `bdl-runtime-core` — `no_std`, allocation-free, `unsafe`-free: the vocabulary
  every generated semantic core is written against (`ActiveDomains`,
  `ClockSlot`, `RuntimeError`, checked numerics, strict primitive helpers).
  Knows no board, device kind, transport or editor.
- `bdl-runtime-host` — `std`: `DynValue`, JSON run requests and traces over
  stdio (`main_stdio`), and a cargo driver (`harness`) that writes, checks,
  builds and runs a generated crate. Used by the differential tests and by
  tooling.

- `bdl-runtime-adapter` — `no_std`, depends on `bdl-runtime-core` only: the
  platform adapter's vocabulary — the numeric policy at the raw command boundary
  (`duty8`), the sink traits (`PwmDuty8`, `Level`), `apply_*`, `CommandFault`,
  the compiled schedule as `ActiveDomains`. Knows no HAL.
- `bdl-runtime-embassy-rp` — the RP2040 binding over `embassy-rp` (PWM slices,
  GPIO pads, the arena, the fault halt). **Outside the workspace** so the HAL's
  dependency tree never enters the host lockfile; built only into generated
  firmware (`cargo` in its directory; the `embedded-rp` preflight check).
- `bdl-runtime-arduino` — the Arduino binding over `avr-hal` (PWM pins on the
  ATmega timers, output lines, the blocking tick wait, the fault halt).
  **Outside the workspace** and **nightly-only** (`avr-hal` is a git dependency
  on `nightly-2025-04-27`, `avr-none` a tier-3 target, `avr-gcc` the linker);
  the `embedded-avr` preflight check skips where that toolchain is absent.

See docs/spec/runtime-semantics.md, docs/architecture/codegen-rust.md and
docs/architecture/embedded-adapter.md.
