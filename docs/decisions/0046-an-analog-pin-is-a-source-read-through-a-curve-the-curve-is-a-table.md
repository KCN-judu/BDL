---
id: ADR-0046
status: accepted
date: 2026-10-17
area: deployment
supersedes: []
superseded-by: []
related: [ADR-0036, ADR-0037, ADR-0038]
fv:
  [
    "formally proved (model): FV Phase 13 `Provision.one`, `Channel.WF`, `Fits`
    — a curve is one more transducer",
    "production-tested: `crates/bdl-compiler/tests/example_sensor_station.rs`,
    `crates/bdl-compiler/tests/source_provision.rs`; the count →
    physical-quantity correspondence on the board is not proved (FVI-0022)",
  ]
---

# ADR-0046: An analog pin is a Source read through a curve, and a curve is a table

## Status

Accepted (the Arduino reader that ADR-0038 left open, and the first analog
providers). Extends ADR-0038 and mirrors ADR-0036; changes neither.

## Context

ADR-0038 gave a Source a provider profile and a pure transducer, and the first
two providers read a digital line on the Pico; the Arduino family reported every
provider unsupported. A design that reacts to a sensor — a thermistor, a
photoresistor, a ranger — therefore could not run on the board the examples
target. The transducer shape already fits: a reading `raw → rep(C)` under no
grant. What was missing was a device kind and a reader for the ADC, and a way to
write a sensor's curve in a language that has no `ln`, `exp` or `pow`.

## Decision

1. **`DeviceKind::AnalogInput`** (`analog_input`; requirement `analog_in`, one
   pin). Protocol 0.31, additive.
2. **The raw reading is the converter's count**, a dimensionless quantity
   `0 … 1023` (`SourceKind::Adc10`, the `AdcSource` trait). The reference is the
   supply; a wiring that makes the count meaningful (a divider's resistor and
   side) is part of the profile's description, as a pull is part of a line
   profile's.
3. **A curve is a table**, not a primitive: `bdl-catalogue::curves::piecewise`
   lowers sampled points to nested `if … then … else` over `<`, `-`, `*`, `+` —
   terms the kernel, the interpreter and the generated core already have. A
   profile's tail either holds its end value (a sensor saturates) or continues
   with a slope (an encoder's out-of-range command stays out of range, so the
   sink's own refusal applies). No new primitive, no change to the kernel.
4. **Profiles** (stable ids): `adc_percent` (a level 0–100), `adc_ntc10k_b3950`
   (kelvin), `adc_ldr_gl5528` (lux), `adc_sharp_gp2y0a21` (metres); and the
   output profile `pwm_gamma8` (gamma 2.2 into a duty). Each table names its
   part and says how it was sampled; replacing one with a datasheet's own points
   is a one-constant change that does not touch the judgments.
5. **The Arduino entry reads them**: `Entry::reads(Adc10)`; `main` builds one
   `arduino_hal::Adc`, constructs a channel per provider on its assigned pin
   (`A0 … A5`), and calls `adapter::provide` with the converted counts at the
   top of every tick, in input-slot order. The RP2040 entry does not read them
   yet (`adapter.provider_unsupported`).

## Alternatives

- **A `ln`/`pow` primitive** — would let the β equation be written as such, at
  the price of a kernel change and a floating-point library on an 8-bit part;
  refused. A table is what a datasheet gives and is exact at its points.
- **Fixed-point or integer counts in the raw type** — `Nat` would be more
  faithful to the converter, but the existing host bridge and every encoder use
  `q 0`; one raw type for readings and commands keeps the numeric policy in one
  place.
- **The curve in the design** (`celsius = f(counts)`) — moves a deployment fact
  into the behaviour; refused for ADR-0038's reason.

## Consequences

- A design can now read a thermistor, a photoresistor, a ranger and a
  potentiometer on a Nano and be built and flashed as it stands:
  `examples/sensor_station`.
- Tables are a code-size cost on an 8-bit part (the thermistor's is 25 points);
  measured, not assumed, by the AVR build.
- Not decided here: an ADC reference other than the supply, oversampling, the
  Pico's ADC, I²C sensors, calibration points supplied by the designer.
- Records: `docs/architecture/embedded-adapter.md` § The input half,
  `docs/architecture/output-realization.md`,
  `docs/spec/{protocol,project-format, textual-syntax}.md`, protocol versions, a
  change fragment.
