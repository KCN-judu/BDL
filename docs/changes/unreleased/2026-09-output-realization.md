# A physical output is realised by a deployment-chosen profile with a pure encoder (protocol 0.24)

- Date: 2026-09-20
- Area: compiler, lowering, codegen, runtime, textual, protocol, daemon, studio,
  l10n
- Affected: designers, project files, protocol clients, developers
- Related: ADR-0036, ADR-0015, ISS-0017, FV Phase 14 (FVD-0131 … FVD-0139,
  FVI-0022), [2026-09 Source creation](2026-09-source-creation.md)

## What changed

- **A device binding may choose a realization profile.** On the Deploy page a
  device card lists every profile with whether it fits the output's concept
  (_PWM, 8-bit duty_, _PWM, 4 levels_, _I2C register, 8-bit_, _GPIO, on/off_,
  _H-bridge, signed level_), and choosing one sets the device kind the profile
  needs in the same edit. The card shows the three judgments behind
  admissibility — _encoder_, _fits_, _placed_ — and the analysis's sentence when
  one fails. The Design page is untouched: a profile is deployment data and
  never changes what the design means.
- **The encoder is a typed pure term.** Each profile's encoder maps the
  concept's representation to a raw command drawn from the existing value forms
  (a level, a truth value, a pair); it is typed under no grant, refers to no
  relationship, remembers nothing and reads no other domain. Choosing the 8-bit
  and the 4-level PWM profile for the same lamp gives the same behavior trace
  and different raw traces; 40 % and 41 % may share a duty.
- **Admissible is three judgments, never fit-and-allocate.** An encoder that
  fits the concept and allocates a PWM line but is ill typed is refused
  (`deploy.realization_encoder_invalid`); a profile that encodes another
  representation is `deploy.realization_incompatible`; an id this version does
  not know is `deploy.realization_unknown_profile`; a binding without a profile
  is placed by kind as before and told so (`deploy.realization_unspecified`,
  info). The board's feasibility is the deployment status, as before.
- **The generated core carries the raw commands.** `Tick` gains
  `commands: Commands` beside `values` and `outputs`, one field per realised
  output (`command_<device id>`), computed after the outputs from the driver's
  value and nothing else; the host trace gains `commands`; the manifest gains
  `sinks`. `Values` and `Outputs` are what they were.

## Compatibility and migration

- Designers: nothing to do. A project without profiles behaves exactly as
  before; the Deploy page adds an informational line per device until a profile
  is chosen.
- Project files: the device body may carry `realization <id>`
  (`device pwmLight : pwm_channel for light { realization pwm_duty8, pin 0 = D3 }`);
  a file without it loads unchanged and is written back without it. A file
  naming an unknown id loads and is diagnosed; nothing is dropped or rewritten.
- Protocol clients: protocol **0.24**, additive — `SetDeviceRealization`,
  `DeviceView.realization`, `DeploymentAnalysis.realizations[]`,
  `RealizationStatus`, `OutputProfileView`, `MissingKind.REALIZATION_INVALID`.
  An older client works unchanged; `deployable` is now also false while a chosen
  profile is not valid.
- Generated crates: exec IR version **3**; every generated core has a `Commands`
  record (empty when nothing is realised) and its host implements
  `commands_to_dyn`; a crate generated before this change does not build against
  the new `bdl-runtime-host` until regenerated. Trace JSON from older hosts
  still parses (`commands` defaults to empty).
- Developers: `bdl_model::OutputProfileId`, `DeviceBinding.realization`,
  `EditOp::SetDeviceRealization`; `bdl_output::realization` (`Encoder`,
  `OutputProfile`, `profiles`, `profile`, `well_formed`, `purity`, `fits`,
  `encoder_body`, `check_binding`); `bdl_compiler::DeviceRealization`,
  `DeploymentAnalysis::realizations` / `realization_blocked`,
  `MissingKind::RealizationInvalid`; `bdl_lower::lower` takes `&Realizations`
  (`Realization`); `bdl_exec_ir::{SinkPlan, SinkSlot}`, `ExecIr::sinks`,
  `TickResult::commands`; `bdl_codegen_rust::names::command`,
  `manifest::SinkEntry`; `bdl_runtime_host::HostProgram::commands_to_dyn`,
  `TickTrace::commands`; `bdl_syntax` `RealizationFix`,
  `DeviceItem::realization`; Studio `SetDeviceRealizationRequested`.

## Evidence

`crates/bdl-output/src/realization.rs` (the judgments; the hardening negative
`ill_typed_encoder_fits_but_is_not_well_formed`; impure and constructing
encoders refused), `crates/bdl-compiler/tests/output_realization.rs` (same
behavior trace under profile A and B, exact PWM / quantized / I²C / GPIO /
H-bridge encodings, the named diagnostics, hardware judged apart, a legacy
binding unchanged, the generated `Commands` against the interpreter),
`crates/bdl-syntax/tests/acceptance.rs`
(`a_device_body_carries_its_realization_profile`),
`crates/bdl-text/tests/workspace.rs` (round trip),
`crates/bdl-daemon/tests/stdio_e2e.rs` (`outputs_and_deployment_over_stdio`),
`apps/studio/test/deploy_test.dart` (the card; the bdld e2e).
