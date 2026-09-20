---
id: ADR-0038
status: accepted
date: 2026-09-20
area: deployment
supersedes: []
superseded-by: []
related:
  [
    ADR-0015,
    ADR-0032,
    ADR-0036,
    ADR-0037,
    ISS-0016,
    ISS-0017,
    ISS-0018,
    PRP-0001,
  ]
fv:
  [
    "formally proved (model): FV Phase 13 `Provision.one`, `Channel.WF`, `Fits`;
    FV Phase 16 `Catalogue`, `assignSource`, `assign_indistinguishable`,
    `assignSource_origin_irrelevant`, `two_providers_same_behavior`,
    `provisionable_mono`",
    "production-tested: `crates/bdl-compiler/tests/source_provision.rs`,
    `crates/bdl-output/src/provision.rs`, `crates/bdl-text/tests/workspace.rs`;
    the reading → raw-value correspondence on the board is not proved
    (FVI-0022), and the provider occurrence contract of FV Phase 17 is not built
    (ISS-0018)",
  ]
---

# ADR-0038: A Source is provided by a device through a catalogue profile; the catalogue is the package boundary, and no judgment reads a profile's origin

## Status

Accepted (the Source half of the platform adapter, roadmap priority 1,
2026-09-20). Extends ADR-0015 (a device binding is deployment data) and mirrors
ADR-0036 (a pure encoder realises an output) on the input side; changes neither,
nor ADR-0032 (a Source is a derived role) nor ADR-0037 (the adapter consumes
commands and has no semantics of its own).

## Context

Since ADR-0037 a design with a Source could not run on a board: the generated
`Inputs` were all `None`, and `adapter_plan` refused the design
(`adapter.inputs_unbound`, ISS-0016). FV Phase 13 had already given the shape of
the missing half — a **transducer** `raw → rep(C)` under no grant, and a
provision that constructs the Source's concept from the transduced reading under
the Source's own grant — and FV Phase 16 the shape of what a deployment chooses
from: a **catalogue** of input and output profiles, each with an **origin**
(builtin or a package) that no judgment reads
(`assignSource_origin_irrelevant`), and the theorem that two providers with the
same semantic trace give the same behaviour (`two_providers_same_behavior`).
Constraints: no new BDL primitive, no `Event`/`Message` type, no package
manager, no pin or bus on the Design canvas, Dart computes no judgment, old
projects keep loading.

## Decision

1. **A device consumes or provides, never both.** `DeviceBinding` gains
   `source: Option<DeclId>` (the Source it provides, by stable identity) and
   `provider: Option<InputProfileId>`; `output`/`realization` and
   `source`/`provider` are exclusive, and `apply_edit` keeps them so
   (`SetDeviceSource` releases the output, `SetDeviceOutput` releases the
   Source; `SetDeviceKind` releases both profiles; deleting the Source releases
   the device). Only a relationship whose role is _Source_ may be provided
   (`EditError::NotASource`).
2. **The catalogue is one crate, `bdl-catalogue`**:
   `Origin {Builtin, Package(id)}`,
   `InputProfile {id, transducer: Transducer {raw, rep, transduce}, kind}`,
   `OutputProfile` (moved from `bdl-output`, ids and encoders unchanged),
   `InputEntry`/`OutputEntry {origin, profile}`, `Catalogue {inputs, outputs}`
   with `builtin()` and `add_*` that refuse an id already present. Every
   judgment takes the entry's _profile_; origin is display data
   (`InputProfileView.origin`). A package manager, when one exists, adds entries
   and changes nothing here.
3. **The provision judgment mirrors the realization judgment**
   (`bdl-output::provision`): `well_formed` (the transducer is a typed pure
   `raw → rep` under `Grant::None`), `fits` (the Source's concept carries
   `rep`), then the placement's `hardware_placed` and the target entry's
   `backend_supported`. The first two are the semantic contract
   (`InputContract`); the last two are feasibility; they are never folded into
   one verdict. Statuses: `NotChosen`, `UnknownProfile`, `KindMismatch`,
   `TransducerInvalid`, `Incompatible`, `Valid`.
4. **The deployment analysis judges every Source** (`SourceProvision`, one per
   Source, in `DeclId` order): an unprovided Source makes the deployment
   _Incomplete_ (`deploy.source_unprovided`, info), a device without a profile
   is `deploy.provider_unspecified` (info), a chosen profile that is not valid
   is an error that blocks the artefact (`deploy.provider_unknown_profile`,
   `deploy.provider_kind_mismatch`, `deploy.provider_incompatible`,
   `deploy.provider_transducer_invalid`), two devices on one Source is
   `deploy.source_contested`, and a profile this board's firmware cannot read is
   `deploy.provider_unsupported` (warning: the design and the placement are
   fine). The report's missing items gain `SourceNoDevice` and
   `ProviderInvalid`.
5. **The provider is lowered below the inputs.** `ExecIr.providers` carries one
   `ProviderPlan` per admissible provision: the provision body
   `mk c (transduce r)` lowered in the Source's own context with the raw reading
   bound to a fresh local; `interp::provide(ir, raw)` evaluates it before
   `step`. The declarations, cells, outputs and sinks of the program are
   unchanged by a provider (tested); a Source without one stays a plain input
   slot the simulation supplies.
6. **The adapter's input half is symmetric to its output half.** Generated
   `src/adapter.rs` gains `SOURCES`,
   `provide(reading_<device>: Option<Raw>, …) -> Result<Inputs, RuntimeError>`
   and `read(&mut dyn LevelSource, …)`; the firmware reads every source once per
   tick, in input-slot order, then steps, then applies. `bdl-runtime-adapter`
   gains `LevelSource`/`read_level`/ `SourceBinding`; `bdl-runtime-embassy-rp`
   gains `Sense::{pull_down, pull_up}`. The host bridge implements
   `inputs_from_readings` (`TickRequest.readings`, additive) through the same
   `provide`, so a host trace and the firmware make the same inputs from the
   same readings.
7. **The first providers are two GPIO line profiles**, `gpio_level_in` (line
   high is `true`, pulled down) and `gpio_level_in_low` (line low is `true`,
   pulled up; the transducer is `not`), over the new requirement template
   `DeviceKind::DigitalInput` (`digital_in`). The pull is peripheral
   configuration; the polarity is the transducer's. The Raspberry Pi Pico reads
   them; the Arduino family reports `backend_supported = false` and refuses the
   firmware by name.
8. **Text and protocol.**
   `device b : digital_input for pressed { provider gpio_level_in, pin 0 = GP2 }`
   — `for` resolves an output first, then a Source (`text.not_a_source` for a
   Value or a Rule); `provider` is a contextual word in the device body.
   Protocol 0.25 adds `DeviceView.source_id` / `provider`, `SetDeviceSource`,
   `SetDeviceProvider`, `DeploymentAnalysis.provisions[]` (`ProvisionView`,
   `ProvisionStatus`, `InputProfileView` with `origin`),
   `DeviceKind.DIGITAL_INPUT`,
   `MissingKind.{SOURCE_NO_DEVICE, PROVIDER_INVALID}`.
9. **`adapter_plan` refuses only what it cannot bind**: a Source without a
   device (`adapter.source_unprovided`), a device without a profile
   (`adapter.provider_unspecified`), a profile the family cannot read
   (`adapter.provider_unsupported`), an invalid provider
   (`adapter.provider_invalid`), a provider without a placed digital-in line
   (`adapter.source_unbound`). A placement that is incomplete only because of an
   unprovided Source is named by the Source, not by the placement.
10. **One reading per tick.** A provider observes its line once per global tick,
    before the core steps; a scalar Source is the sample. The batch,
    deduplication, merge and per-tick bound of FV Phase 17's provider contract
    are not built (ISS-0018); nothing here preempts them, since the raw reading
    stays the profile's and the reading's timing the adapter's.

## Alternatives

- **A device kind that provides, without a profile** (the ISS-0016 sketch) —
  would put the transducer nowhere; refused for the same reason ADR-0036 refused
  an encoder-less realization.
- **The transducer as a formula in the design** (`pressed = !line`) — moves a
  deployment fact into the behaviour and makes the design depend on the board;
  refused (ADR-0032 §1, FV `assign_indistinguishable`).
- **A Source-side device clock** (sampling period as a `ClockId`) — FVI-0020
  stays open; the reading happens at the global tick, and a slower device is
  Phase 15's `lowerSync` question, not decided here.
- **Origin as a field on the profile the judgments see** — would let a package
  profile be judged differently; the theorem says it must not
  (`assignSource_origin_irrelevant`), so origin lives on the entry only.
- **A separate Studio Sources list** — the boundary view keeps one Devices
  section; a device is _for_ an output or a Source in one pop-up, and an
  unprovided Source is a missing item and an info line, exactly like an
  unrealised output.
- **A registry keyed by device kind** — one Source may be provided by several
  profiles of one kind (both GPIO polarities); the profile prescribes the kind,
  never the reverse.

## Consequences

- A design with a Source now runs on the Pico: `Button → rule → Lamp` is one
  firmware with a GPIO input and a GPIO output, cross-built in CI; the host
  bridge and the interpreter give the same behaviour for the same readings; two
  providers of one Source with the same semantic trace give the same values,
  outputs and commands (tested, the production image of
  `two_providers_same_behavior`).
- Old projects: a Source with no device is an _Incomplete_ deployment with the
  reason, never an error and never a migration; `DeviceBinding` files without
  `source`/`provider` load unchanged. Fixtures and tests that asserted
  _Feasible_ for such projects now assert _Incomplete_.
- New: `crates/bdl-catalogue`, `bdl-output::provision`, `ExecIr.providers` (exec
  IR version 4), `interp::provide`, `ProviderBinding`/`SourceKind`/
  `AdapterPlan.providers`, `Entry::{reads, source_peripheral}`, manifest
  `adapter.sources[]`, `HostProgram::inputs_from_readings`,
  `TickRequest.readings`, `ReadingError`, `LevelSource`, `Sense`,
  `DeviceKind::DigitalInput`, `InputProfileId`, `SetDeviceSource`/
  `SetDeviceProvider`, protocol 0.25, Studio's provision row and the inspector's
  projected _Realization_ row for a Source.
- Not decided here: the provider occurrence contract (ISS-0018), stateful
  transducers and a Source device clock (FVI-0020), analog and bus providers, an
  Arduino reader, a package manager.
- Records: `docs/architecture/embedded-adapter.md` § The input half and § The
  package boundary, `docs/architecture/output-realization.md` § The catalogue,
  `docs/spec/{textual-syntax,protocol,hardware-model,project-format}.md`,
  `docs/project/{status,roadmap,formal-correspondence}.md`, ISS-0016 (resolved),
  ISS-0017 (amended), ISS-0018 (new), a change fragment, the user guide's Deploy
  and CLI pages.
