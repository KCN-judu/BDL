# A second example: the Breathing Beacon, analog dynamics on the Nano

- Date: 2026-10-17
- Area: examples, ci
- Affected: designers, developers
- Related: [2026-09 Arduino adapter](2026-09-arduino-adapter.md), ADR-0036,
  ADR-0038

## What changed

- **`examples/breathing_beacon`**, a design with no Source whose behaviour is
  all state: a triangle-wave breath (`delay` cells, clamped with `min`/`max`), a
  gamma curve (`breath² / 100`), a first-order lag (`afterglow`) and a Schmitt
  trigger (on above 60, off below 20). Levels are percent, as `pwm_duty8`
  expects (0–100 ↦ 0–255). Three devices are written in the file: `pwm_duty8` on
  D9 and D10 (timer 1) and `gpio_level` on D13 (the on-board LED). The
  deployment is _Feasible_ on the Arduino Nano, so
  `bdld compile --target arduino_nano` generates firmware with nothing left to
  choose.
- It runs on its own: the Nano has no Source reader yet (ADR-0038), so the
  example makes its analog variation itself. A real ADC provider is the next
  step for an example that reacts to a sensor.

## Compatibility and migration

Nothing to do; no protocol, schema or generated-code change.

## Evidence

`crates/bdl-compiler/tests/example_breathing_beacon.rs`: the ladder; one
402-tick breath in the reference evaluator (triangle, gamma, lag, one hysteresis
pulse); `Feasible` on the pins the file names; the generated core run on the
host (duties 0 → 255 → 0, the D13 level); the AVR ELF under `BDL_REQUIRE_AVR`.
