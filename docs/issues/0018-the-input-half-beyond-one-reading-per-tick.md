---
id: ISS-0018
state: open
area: deployment
opened: 2026-09-20
resolved-by: []
related: [ADR-0038, ADR-0037, ISS-0017, ISS-0006]
---

# ISS-0018: The input half beyond one line read once per tick: the provider's occurrence contract, stateful transducers, a Source device clock, analog and bus providers, more families

## Problem

ADR-0038 provides a Source from one digital line read once per global tick,
through a pure transducer, on the Raspberry Pi Pico. What a product also needs
is not built:

- **the provider's occurrence contract** — FV Phase 17 states and proves it
  (`BDL/Surface/Provider.lean`, FVD-0149 … FVD-0151): the raw reading of a
  transport is a bounded batch `(list raw, bool)` of the deliveries since the
  previous tick, one occurrence per fresh transport identity, arrival order,
  retransmissions erased by identity with adapter state, an observable overflow
  flag, several raw sources as several provisions. Production's reading is a
  scalar taken once per tick (the batch sampled, `Batch.latest`); no provider
  keeps identities, no reading is a list, no `cap` exists.
- **stateful transducers and a Source device clock** (FVI-0020) — debouncing,
  filtering, a sampling period slower than the tick, a bus transaction that
  answers later: `purity` refuses `delay` and `sync` in a transducer, and the
  reading happens at the global tick.
- **analog and bus providers** — an ADC channel, an I²C register, a UART frame:
  the catalogue has two GPIO polarities;
  `DeviceKind::{I2cSensor, QuadratureEncoder, Uart}` place on a board but no
  profile reads them.
- **more families** — the Arduino entry reports `backend_supported = false` for
  every input profile; `Entry::reads` is the one place a family says what it
  reads.
- **a packaged catalogue** — `Origin::Package` exists and no judgment reads it;
  who resolves, validates and signs a package is not decided.

## Why it matters

Each item is where the next real product will stop: a button that bounces, a
sensor on a bus, a command stream that retries. Deciding them inside a family's
firmware would put a deployment promise outside the records and past the four
inspectable judgments.

## Current evidence

`crates/bdl-catalogue/src/lib.rs` (`builtin_inputs`),
`crates/bdl-output/src/provision.rs` (`purity` shared with encoders),
`crates/bdl-exec-ir/src/interp.rs` (`provide`: one raw value per provider),
`crates/bdl-codegen-rust/src/targets/mod.rs` (`Entry::reads`),
`docs/architecture/embedded-adapter.md` § The input half; BDL_FV Phase 17 report
§17.1, FVI-0020, FVI-0024.

## Dependencies

For the occurrence contract: a first provider whose deliveries carry a transport
identity (a UART frame counter, a CAN sequence) — then an ADR consuming FVD-0149
… FVD-0151 with `Batch` as a shared raw reading with two channels. For stateful
transducers and a device clock: FV first (FVI-0020). Analog and bus providers
need only catalogue entries and a reader per family.

## Resolution

Open.
