# Analog inputs on the Arduino: ADC readers, sensor curves, a gamma output (protocol 0.31)

- Date: 2026-10-17
- Area: catalogue, codegen, compiler, protocol, studio, examples
- Affected: designers, developers, generated crates
- Related: ADR-0046, ADR-0038,
  [2026-09 Arduino adapter](2026-09-arduino-adapter.md),
  [2026-10 Breathing Beacon](2026-10-breathing-beacon.md)

## What changed

- **`analog_input` device kind** (`DeviceKind::AnalogInput`, `analog_in`, one
  pin); protocol 0.31 (`DEVICE_KIND_ANALOG_INPUT`), Studio's Deploy page and its
  en/zh/ja strings.
- **Four input profiles**: `adc_percent`, `adc_ntc10k_b3950` (kelvin),
  `adc_ldr_gl5528` (lux), `adc_sharp_gp2y0a21` (metres); **one output profile**,
  `pwm_gamma8`. Curves are tables (`bdl-catalogue::curves`), lowered to nested
  conditionals: the kernel gains nothing.
- **The Arduino reads them**: `SourceKind::Adc10`, `AdcSource`/`read_adc` in
  `bdl-runtime-adapter`, one shared `arduino_hal::Adc` in the generated `main`,
  `A0 … A5`; the manifest records `kind: "adc"`.
- **`examples/sensor_station`**: a thermistor, a photoresistor and a trim pot
  in; a gamma-corrected night light, a fan and an overheat lamp out; Feasible on
  the Nano as shipped.

## Compatibility and migration

Additive: old projects, the Pico's firmware and every golden are unchanged. A
design that gives an analog profile to a Pico is refused by name
(`adapter.provider_unsupported`).

## Evidence

`crates/bdl-compiler/tests/example_sensor_station.rs` (the deployment and the
manifest, the generated core over counts computed from the parts' own physics
through four conditions, the extremes of the range, the AVR ELF under
`BDL_REQUIRE_AVR`), `source_provision.rs` (the candidates), the Nano firmware
type-checks and compiles up to the linker.
