---
id: ADR-0036
status: accepted
date: 2026-09-20
area: deployment
supersedes: []
superseded-by: []
related: [ADR-0015, ADR-0032, ISS-0016, ISS-0017]
fv:
  [
    "formally proved (model): BDL_FV Phase 14
    `BDL/Surface/OutputRealization.lean` — `Encoder.WF`, `EFits`,
    `encoder_constructs_nothing`, `lower_transparent`, `behavior_unchanged`,
    `lower_correspondence`, `two_realizations_same_behavior`,
    `admissible_needs_wf` (FVD-0131 … FVD-0139, `8e65c63`)",
    "production-tested: `crates/bdl-output/src/realization.rs`,
    `crates/bdl-compiler/tests/output_realization.rs`",
  ]
---

# ADR-0036: An output is realised by a deployment-chosen profile whose pure encoder lowers the value to a raw command; admissibility is typing, fit and a solvable board

## Status

Accepted (consumption of FV Phase 14 into production, 2026-09-20). Extends
ADR-0015 (a dated amendment there records the one sentence it narrows); does not
touch ADR-0032 or Source provision (PRP-0001, ISS-0016), which sit at the other
physical boundary and stay separate.

## Context

A `PhysicalOutput` accepts a concept in a domain and a `DeviceBinding` names the
kind of hardware that carries it (ADR-0015, `docs/spec/hardware-model.md`).
Nothing said how the concept's value becomes what a PWM line, a GPIO or an I²C
register takes: the generated core's `Outputs` carried the representation and
any conversion would have been an adapter's, unchecked. FV Phase 14 modelled the
missing half as a pure **encoder** `Rep(C) → raw` and proved that adding it as a
lowering below the design changes no behavior, and — after hardening — that
"fits and allocates" is not enough to admit one (`exJ`: `λn. true`).

Constraints: the kernel is not redesigned (no `Ty::Effect`, no `R → ()`, no
`Expr::Write`, no new primitive); `Output.accepts` stays a concept, never a raw
type (Model A is refuted: `retarget_breaks_driveWF`); `OutputId`,
`OutputSpec.accepts`, `DriveEnv`, `DriveWF`, `SingleDriver`, typing, causality
and simulation are preserved; Source provision and output realization are not
merged because both are physical boundaries; the open formal items (FVI-0022)
are not implemented speculatively.

## Decision

1. **Realization is deployment data.** `DeviceBinding` gains
   `realization: Option<OutputProfileId>` — a stable string id persisted in the
   device body (`realization <id>`), never on the output, never in the design
   graph. A binding without one behaves as every binding did before this record.
2. **A profile is an encoder plus a requirement template.**
   `bdl-output::realization::OutputProfile { id, display_name, description, encoder, kind }`.
   `kind` is the existing `DeviceKind`; the profile prescribes it, and
   `SetDeviceRealization { id, profile, kind }` sets both in one edit
   (`Invalidation::Deployment`; a kind change releases the manual pins).
   `SetDeviceKind` releases the profile so the two never disagree through the
   edit layer; a file that makes them disagree is diagnosed.
3. **The encoder is a closed, pure, typed Core term.**
   `Encoder { rep, raw, encode }` with `rep` and `raw` sem-free data drawn from
   the existing types (`q`, `bool`, products); `well_formed` types `encode` as
   `rep → raw` in the empty declaration environment under `Grant::None`;
   `purity` refuses `declRef`, `delay`, `sync` and `mk` structurally.
   Quantization (many levels, one command) is valid; no injectivity or
   round-trip is asked.
4. **Admissible = well formed ∧ fits ∧ solvable**, three judgments reported
   separately (`DeviceRealization { check, hardware_placed }`) with one
   diagnostic code per failure (`deploy.realization_*`), composed in
   `analyze_deployment`; the requirements come from the profile's kind through
   the unchanged `bdl-hardware` template and solver. No parallel allocator.
5. **Lowering adds a machine sink, nothing else.** `compile` hands `bdl-lower`
   one `Realization { device, output, profile, raw, body: encode (rep d) }` per
   valid chosen profile on a validly driven output; the lowering emits a
   `SinkPlan` after the outputs, due when the driver is, in the driver's context
   under no grant; no fresh `DeclId`, cell or clock. A chosen profile that is
   not valid refuses the artefact (`backend.realization_invalid`).
6. **The machine boundary is the raw command trace.** The generated core's
   `Tick` gains `commands: Commands` beside `values` and `outputs`; the host
   bridge gains `commands_to_dyn`; `TickTrace.commands` is the additional,
   downstream view. Simulation stays behavior-level.
7. **The registry is code, versioned with the compiler.** Five profiles in this
   slice (`pwm_duty8`, `pwm_duty4`, `i2c_level8`, `gpio_level`,
   `hbridge_signed`); a persisted id the registry does not know is reported,
   never silently dropped or remapped.
8. **Studio chooses on the Deploy page only.** The device card lists every
   profile with its fit, shows the three judgments and the analysis's sentence;
   the Design page, the inspector and the canvas know nothing of profiles.

## Alternatives

- **`Output.accepts = raw`** — retargeting the output to the command type:
  refuted (`retarget_breaks_driveWF`); the drive edge's exact type would break
  and the design would learn the board.
- **An effect type or an imperative `write`** in the language: would make the
  behavior environment depend on the device; the model shows the relation
  `RawCommand` needs neither.
- **Profile = device kind** (one encoder per kind): one output could not have
  two encodings on one kind (8-bit vs 4-bit PWM) and the requirement template
  could not be shared; refused.
- **Fit + allocate as admissibility**: wrong on the same evidence (FVD-0137 →
  FVD-0139); the hardening test keeps it refused.
- **Fresh declarations for `e` and `p`** in the Design IR, as the model's
  `lowerΔ` literally does: would make sinks designer-visible entities with ids,
  canvas positions and diagnostics; the plan-level sink is the lowest layer that
  preserves `d`, `o`, `d → o` and is observably the same.
- **One profile registry per board**: a profile is semantic (representation and
  command), the board is a separate judgment; a per-board registry would fuse
  the two.

## Consequences

- Choosing a profile changes no analysis, no sample and no `Values`/`Outputs`;
  this is tested
  (`changing_the_realization_changes_no_behavior_and_only_the_raw_trace`, the
  daemon and Studio e2e).
- Exec IR version 3, protocol 0.24 (`SetDeviceRealization`,
  `DeviceView.realization`, `DeploymentAnalysis.realizations`,
  `MissingKind.REALIZATION_INVALID`), manifest `sinks`, textual syntax
  `RealizationFix`. Old projects load unchanged.
- `compile_design_ir` (a bare Design IR) lowers no sink; the corpus goldens gain
  an empty `Commands {}`.
- Not decided here, kept open in ISS-0017 / FVI-0022: stateful adapters, a
  device clock, atomic frames, the adapter's correspondence to the raw trace, a
  device catalogue beyond the five witnesses, and the Source side (ISS-0016).
- Records: `docs/architecture/output-realization.md` (the eight principles),
  `docs/spec/hardware-model.md`, `docs/spec/textual-syntax.md` §14.1,
  `docs/spec/protocol.md`, `docs/architecture/codegen-rust.md`,
  `docs/architecture/deployment-read-model.md`,
  `docs/project/formal-correspondence.md`, a change fragment.
