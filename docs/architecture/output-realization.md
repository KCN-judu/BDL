---
kind: architecture
area: deployment
status: current
---

# Output realization

How a logical output's value reaches a board without the board reaching into the
design. The formal model is BDL_FV Phase 14
(`BDL/Surface/OutputRealization.lean`, FVD-0131 … FVD-0139, FVI-0022); the
decision is ADR-0036; the deployment layer it extends is ADR-0015 and
`docs/spec/hardware-model.md`.

## The eight principles

1. **An output is logical intent.**
   `PhysicalOutput { accepts: SemanticId, clock, required }` says what the
   product finally drives — a concept in a domain — and nothing about wires.
   `DriveWF` (exact type, exact domain), `SingleDriver` and `CompleteOutputs`
   are unchanged (FVD-0131).
2. **Realization is deployment data.** A device binding may name a **realization
   profile** (`DeviceBinding.realization: Option<OutputProfileId>`, persisted as
   `realization <id>` in the device body). A profile is not a device kind: one
   output may be realised by several profiles, several profiles may share a
   requirement template, and the profile prescribes the kind
   (`OutputProfile.kind`) — never the other way round.
3. **An encoder maps `Rep(C) → raw`.** Each profile carries an
   `Encoder { rep, raw, encode }`: a closed Core term from the representation
   the output's concept carries to a raw command type drawn from the existing
   sem-free data types (a level, a truth value, a pair). No raw type exists for
   its own sake, and no encoder ever sees a concept (FVD-0133).
4. **An encoder is pure and stateless.** It is typed in the empty declaration
   environment under `Grant::None` (`well_formed`: `Encoder.WF`) and is
   structurally free of `declRef`, `delay`, `sync` and `mk` (`purity`). It can
   construct nothing (`encoder_constructs_nothing`) and it can be evaluated only
   where its argument is: in the output's own domain. A device clock is not
   modelled; an encoder that would need one is refused, not approximated
   (FVD-0138).
5. **Hardware is checked separately.** The profile's kind feeds the existing
   requirement template (`bdl-hardware::devices::needs`) and the existing
   solver; nothing about the encoder reaches the allocator and nothing about the
   board reaches the encoder. **Admissible** is the conjunction of three
   judgments a reader can inspect one by one — the encoder is well formed, the
   representation fits (`fits`: `EFits`), the requirements place — and never
   fit-and-allocate alone: the ill-typed encoder `λn. true` claims `q0 → q0`,
   fits Brightness, allocates a PWM line, and is refused (FVD-0139, test
   `ill_typed_encoder_fits_but_is_not_well_formed`).
6. **Lowering adds downstream structure.** For every admissible realization the
   lowering adds one **machine sink** below the behavior plan
   (`bdl-exec-ir::SinkPlan`): the command `encode (rep d)` for the output's
   driver `d`, evaluated after the outputs, due exactly when the driver is, in
   the driver's domain, under no grant. No declaration, cell, clock or output is
   added or changed: `values` and `outputs` of a design lowered with or without
   sinks are identical (`lower_transparent`, `behavior_unchanged`; test
   `changing_the_realization_changes_no_behavior_and_only_the_raw_trace`). Sinks
   are not designer entities: they carry no `DeclId`, appear on no canvas, and
   the protocol does not expose them.
7. **The backend performs effects.** The machine boundary is the raw command
   trace: the generated core's `Commands` record and the host trace's
   `commands`, one entry per sink per tick (FVD-0134). Nothing in the language,
   the IR or the runtime is an effect type; what a platform adapter does with a
   command is the adapter's, and its correspondence to the raw trace is not
   proved (FVI-0022).
8. **Profile selection never changes behavior.** Choosing, changing or releasing
   a profile is one deployment edit (`SetDeviceRealization`,
   `Invalidation::Deployment`): the semantic analysis, the causality order, the
   clocks, the drive edges and every simulation sample are the same before and
   after, and the same under profile A and profile B; only the raw trace may
   differ (`two_realizations_same_behavior`).

## Where each responsibility lives

| Responsibility                                                                    | Owner                                                             | Never                                           |
| --------------------------------------------------------------------------------- | ----------------------------------------------------------------- | ----------------------------------------------- |
| the logical output, drive edges, `DriveWF`/`SingleDriver`                         | `bdl-model::surface`, `bdl-output::check_outputs`                 | changed by a profile                            |
| the profile id on a binding; the atomic profile-and-kind edit                     | `bdl-model` (`DeviceBinding.realization`, `SetDeviceRealization`) | the design graph, a mapping                     |
| the profile registry, encoders, `well_formed` / `purity` / `fits`, `encoder_body` | `bdl-output::realization`                                         | the solver; evaluation                          |
| requirement templates and placement                                               | `bdl-hardware` (unchanged)                                        | the encoder                                     |
| the three judgments composed per binding, with named diagnostics                  | `bdl-compiler::analyze_deployment` (`DeviceRealization`)          | `analyze` (target-independent, unchanged)       |
| refusing an artefact for an invalid chosen profile; building sink specs           | `bdl-compiler::backend::compile`                                  | `compile_design_ir` (a bare IR has no bindings) |
| the machine sinks                                                                 | `bdl-lower` (`Realization` → `SinkPlan`), `bdl-exec-ir::interp`   | fresh `DeclId`s, cells, clocks                  |
| `Commands` in the generated core, `commands_to_dyn`, the trace field              | `bdl-codegen-rust`, `bdl-runtime-host` (`TickTrace.commands`)     | `Values`, `Outputs`                             |
| choosing a profile, seeing the three judgments                                    | Studio Deploy page (`_RealizationRow`)                            | the Design page                                 |

## The judgments, in product words

`analyze_deployment` reports one `DeviceRealization` per device binding
(`realizations`, `DeviceId` order) and one diagnostic where a judgment fails:

| Status                            | Code                                 | Severity | Meaning                                                                                |
| --------------------------------- | ------------------------------------ | -------- | -------------------------------------------------------------------------------------- |
| `NotChosen` (bound to an output)  | `deploy.realization_unspecified`     | info     | placed by kind as before; no raw command is generated                                  |
| `UnknownProfile`                  | `deploy.realization_unknown_profile` | error    | the persisted id is not in this version's registry                                     |
| `KindMismatch`                    | `deploy.realization_kind_mismatch`   | error    | a file says a kind the profile does not prescribe (the edit layer never produces this) |
| `Incompatible` (`¬ EFits`)        | `deploy.realization_incompatible`    | error    | "the output carries `q[1]` but the profile encodes `bool`"                             |
| `EncoderInvalid` (`¬ Encoder.WF`) | `deploy.realization_encoder_invalid` | error    | a registry defect, never the designer's                                                |
| `Valid` ∧ placed                  | —                                    | —        | admissible                                                                             |

An error blocks the read model (`MissingKind::RealizationInvalid`,
`deployable = false`) and the artefact (`backend.realization_invalid`); the info
does neither. Hardware infeasibility is the deployment `status` as before;
`DeviceRealization.hardware_placed` says whether this device's requirements are
in the assignment.

## The first slice

`bdl-output::realization::profiles()` — ids are stable and persisted:

| Profile          | Kind               | `rep`  | `raw`         | Encoder                                                     |
| ---------------- | ------------------ | ------ | ------------- | ----------------------------------------------------------- |
| `pwm_duty8`      | `pwm_channel`      | `q[1]` | `q[1]`        | `n · 255 / 100` (50 → 127.5; exact, many-to-one is allowed) |
| `pwm_duty4`      | `pwm_channel`      | `q[1]` | `q[1]`        | four duties by thresholds 25/50/75 → 0, 85, 170, 255        |
| `i2c_level8`     | `i2c_sensor`       | `q[1]` | `q[1] × q[1]` | `(42, n · 255 / 100)` — register and value                  |
| `gpio_level`     | `digital_output`   | `bool` | `bool`        | identity                                                    |
| `hbridge_signed` | `h_bridge_channel` | `q[1]` | `bool × q[1]` | `(¬(n < 0), abs(n) · 255 / 100)` — direction and duty       |

Quantities are `f64` in production (kernel: `Nat`), so `pwm_duty8` carries 127.5
for 50 %: the exact value of the pure term, not a rounding the board does. There
is no rounding primitive and none is added; a quantizing profile uses comparison
and `ite`.

## Compatibility

A project written before profiles existed has bindings without `realization`:
they parse, place by kind, produce no command, and report the info diagnostic.
Its behavior trace, its analysis and its generated `Values` / `Outputs` are
byte-for-byte what they were; the generated core gains an empty `Commands {}`
record (exec IR version 3). No migration writes anything; choosing a profile on
the Deploy page is the only way a project gains one.

## Not established

FVI-0022, unchanged by this slice: stateful adapters (slew, dithering,
batching), a device clock other than the output's (would be an explicit `sync`),
atomic multi-value frames, the correspondence between the raw command trace and
what a platform adapter emits, and commitments on outputs. Production implements
none of these and refuses what would need them (`purity` rejects `delay` and
`sync`).
