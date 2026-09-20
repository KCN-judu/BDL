---
id: ISS-0017
state: open
area: deployment
opened: 2026-09-20
resolved-by: []
related: [ADR-0036, ADR-0037, ADR-0038, ISS-0016, ISS-0018]
---

# ISS-0017: Output realization beyond a pure encoder: stateful adapters, a device clock, atomic frames, the adapter's correspondence, a device catalogue

## Problem

ADR-0036 realises a logical output by a pure, stateless encoder in the output's
domain and stops at the raw command trace. What a product also needs is not
modelled and not implemented:

- **stateful output adapters** — slew-rate limiting, PWM dithering, protocol
  batching, hysteresis — are not pure encoders; whether each is behavior (an
  ordinary declaration with `delay`), deployment adaptation, or the machine's
  own protocol state is undecided;
- **a device clock** different from the output's — a carrier frequency, a bus
  schedule — would be an explicit `sync` in the model; production refuses an
  encoder that mentions `sync` and has no other spelling for it;
- **atomic multi-value frames** — a display frame, a multi-register write — are
  not one output realised once; the singleton realization is primitive by
  decision (FVD-0136) and shared devices batch or combine upstream;
- **the adapter's correspondence** — that a platform adapter emits what the raw
  command trace says, tick by tick — is the codegen half FV does not prove;
  production's differential test stops at the generated `Commands`;
- **a device catalogue** — the five profiles are witnesses (`pwm_duty8`,
  `pwm_duty4`, `i2c_level8`, `gpio_level`, `hbridge_signed`); a real board's
  catalogue, its parameters (register addresses, resolutions) and who owns it
  are open, as is quantities being `f64` where a command register is a count.

## Why it matters

The first embedded platform adapter (roadmap priority 1) will meet every one of
these; deciding them ad hoc in the adapter would move deployment facts out of
the records and past the three-judgment admissibility.

## Current evidence

`crates/bdl-output/src/realization.rs` (`purity` refuses `delay`/`sync`;
`profiles()`), `crates/bdl-lower` (`SinkPlan` in the driver's domain),
`docs/architecture/output-realization.md` § Not established, BDL_FV FVI-0022.
Since ADR-0037 the first platform adapter consumes the raw commands on the
RP2040 (`docs/architecture/embedded-adapter.md`): the adapter's correspondence
is tested through recording sinks and a cross-build, never past the register
write; `i2c_level8` and `hbridge_signed` are refused for the board
(`adapter.profile_unsupported`), and the adapter applies each command
independently — the stateful, device-clock and atomic-frame questions are
untouched.

## Dependencies

The first embedded platform adapter; for a device clock or stateful adapters an
FV phase first (FVI-0022), then an ADR each.

## Audit against FV Phases 15 – 17 (2026-09-20)

The formal side has moved on three of the five items; production has consumed
one:

- **a device catalogue** — consumed. FV Phase 16 (`Catalogue`, `Origin`,
  `assign_indistinguishable`, `realizable_mono`) is ADR-0038's `bdl-catalogue`:
  the five output profiles moved there unchanged, an origin sits on the entry
  and no judgment reads it, and a package adds entries. Parameters (register
  addresses, resolutions) and `f64` where a register is a count stay open
  (ISS-0006).
- **a device clock** — proved, not built. FV Phase 15 lowers a realization into
  an explicit device domain through `sync` (`lowerSync`, FVD-0140 … FVD-0142)
  and Phase 17 gives the occurrence-preserving crossing (`lowerWindow`,
  FVD-0152: the sink's type — `raw` or `list raw` — selects the lowering).
  Production still refuses `sync` in an encoder and has no spelling for a device
  domain; FVI-0024 is narrowed to the acknowledging device and `initRep`.
- **stateful output adapters** — Phase 15 gives the adapter's policy, operation
  and line (`Policy`, `Op`, `AdapterOp`, `Line`) and the rule that no stateful
  realization primitive exists without a witness; Phase 17 gives the batch as a
  list of operations and a fold (FVD-0153). Production's `duty8` policy and
  `Applied` are that shape for one operation per tick; slew, dither and
  hysteresis are still undecided between behaviour and adaptation.
- **atomic frames** — unchanged (FVD-0136 stands; FVD-0148 for the paired axis
  under batching).
- **the adapter's correspondence** — unchanged (FVI-0022): tested through
  recording sinks and a cross-build, never past the register write. The input
  half now has the same evidence shape (ADR-0038, ISS-0018).

## Resolution

Open.
