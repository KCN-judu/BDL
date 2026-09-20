---
id: ISS-0016
state: open
area: deployment
opened: 2026-09-19
resolved-by: []
related: [ADR-0032, ADR-0015, ADR-0037, PRP-0001]
---

# ISS-0016: A device binding for a Source

## Problem

A Source (an unresolved `() -> A`, ADR-0032) is a value the environment
provides; what realizes it on a target — an I2C temperature sensor, an ADC
channel, a GPIO input — is deployment's, never the design's. Deployment today
binds a _device_ to a physical **output**
(`device pwmLight : pwm_channel for light`, ADR-0015) and to nothing else: there
is no `for <source>`, no device kind that provides a value, and the Deploy page
lists outputs only. A deployed product therefore has no place to say which
peripheral feeds `TempSensor`; the generated core reads the Source from its
input vector, and the platform adapter is expected to fill it.

## Why it matters

The five-layer distinction the milestone drew (concept, representation,
relationship, role, device binding) has its last layer empty for Sources. The
inspector says _Provided by the environment; no device is bound yet_ and nothing
lets the designer bind one. Until the first embedded platform adapter (roadmap
priority 1) exists, it is unclear which device kinds provide values and what a
Source binding must carry (a pin, a bus address, a sampling period), so the
design would be guessed.

## Current evidence

Since ADR-0037 the RP2040 firmware refuses a design with a Source
(`adapter.inputs_unbound`): its `Inputs` are all `None`, and a due input would
fault the tick. `crates/bdl-model` `Device { kind, output, pins }` — an output,
never a declaration; `docs/spec/hardware-model.md`;
`apps/studio/lib/ui/pages/deploy_page.md` lists outputs;
`apps/studio/lib/ui/inspector.dart` (`realizationEnvironment`); the generated
core's input vector (`docs/architecture/codegen-rust.md`).

## Dependencies

The first embedded platform adapter: its device catalog decides which kinds
_provide_ and what a binding needs. Then an ADR extending ADR-0015's device
model with a `for <source>` end, the Deploy page's Sources list, and the
inspector's _Realization_ row naming the bound device. The design graph is
unchanged by any of it (ADR-0032 §1).

## Resolution

Open. PRP-0001 proposes the shape of the binding's missing half — the transducer
from the device's raw reading to the concept — as a formal construction over
designs; the device catalog question above stays.
