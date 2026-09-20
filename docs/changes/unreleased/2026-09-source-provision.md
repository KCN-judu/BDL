# A Source is provided by a device through a catalogue profile; the firmware reads the line (protocol 0.25)

- Date: 2026-09-20
- Area: compiler, lowering, codegen, runtime, textual, protocol, daemon, studio,
  l10n, hardware
- Affected: designers, project files, protocol clients, developers, generated
  crates
- Related: ADR-0038, ADR-0036, ADR-0037, ISS-0016 (resolved), ISS-0017,
  ISS-0018, FV Phases 13, 16 and 17,
  [2026-09 Output realization](2026-09-output-realization.md),
  [2026-09 Embedded adapter](2026-09-embedded-adapter.md)

## What changed

- **A device can be for a Source.** On the Deploy page a device's pop-up lists
  the design's outputs and its Sources (_tilt — Source_); a device bound to a
  Source shows a _Provider_ pop-up with the catalogue's input profiles
  (compatible ones first), and four judgments — _transducer_, _fits_, _placed_,
  _readable_ — with the analysis's sentence when one fails. The inspector's
  _Realization_ row for a Source now says what the deployment says on the chosen
  board: _Provided by the environment; no device on Raspberry Pi Pico yet_,
  _Provided by button as GPIO input, active low on Raspberry Pi Pico_.
- **A design with a Source runs on a board.**
  `bdld compile --target rp2040_pico` generates the input half beside the output
  half: `src/adapter.rs` gains `provide` and `read`, the firmware reads every
  provided line once per tick before the core steps, the manifest lists
  `adapter.sources`. `Button → rule → Lamp` is one firmware with a GPIO input
  and a GPIO output, cross-built in CI.
- **Two providers.** `gpio_level_in` (line high is _true_, pulled down) and
  `gpio_level_in_low` (line low is _true_, pulled up), both over the new device
  kind _Digital input_ (`digital_input`, one `digital_in` line). The Raspberry
  Pi Pico reads them; the Arduino Nano reports _cannot read yet_ and refuses the
  firmware by name.
- **The catalogue.** The five output profiles moved, unchanged, into the new
  `bdl-catalogue` crate beside the two input profiles; every entry carries an
  origin (_builtin_ today) that no judgment reads.
- **A Source with no device is an incomplete deployment.** The verdict is _Fits
  … so far — the binding is not finished_, the Sources are listed in the missing
  items (_tilt has no device on Arduino Nano_) and as info lines; the design's
  own analysis and simulation are untouched. Before this change the same project
  said _Feasible_ and the firmware refused it (`adapter.inputs_unbound`); now
  the deployment says why and the firmware refuses it by the Source's name
  (`adapter.source_unprovided`).
- **Text.**
  `device button : digital_input for pressed { provider gpio_level_in, pin 0 = GP2 }`
  — `for` names an output or a Source; `provider` sits in the body like
  `realization`; a device for a Value or a Rule is `text.not_a_source`.
- **Diagnostics.** `deploy.source_unprovided` (info),
  `deploy.provider_unspecified` (info), `deploy.provider_unknown_profile`,
  `deploy.provider_kind_mismatch`, `deploy.provider_incompatible`,
  `deploy.provider_transducer_invalid`, `deploy.source_contested` (errors),
  `deploy.provider_unsupported` (warning); `backend.provider_invalid`;
  `adapter.source_unprovided`, `adapter.provider_unspecified`,
  `adapter.provider_invalid`, `adapter.provider_unsupported`,
  `adapter.source_unbound`. `adapter.inputs_unbound` is gone.

## Compatibility and migration

- Designers: nothing to do. A project with a Source keeps loading and simulating
  as before; its Deploy page now reads _incomplete_ until a device provides the
  Source, which is the truth it always was.
- Project files: `DeviceBinding` gains optional `source` and `provider`; files
  without them are unchanged, and nothing writes them until a device is bound to
  a Source. The `.bdl` device body gains the optional `provider` entry. No
  migration.
- Protocol clients: 0.25 is additive (`DeviceView.source_id` / `provider`,
  `SetDeviceSource`, `SetDeviceProvider`, `DeploymentAnalysis.provisions[]`,
  `ProvisionView`, `ProvisionStatus`, `InputProfileView`,
  `DeviceKind.DIGITAL_INPUT`, `MissingKind.SOURCE_NO_DEVICE` /
  `PROVIDER_INVALID`); a 0.24 client keeps working and sees a Source-bearing
  project as `INCOMPLETE` with an unfamiliar `MissingKind` it should render by
  its `message`.
- Generated crates: exec IR version 4 (`ExecIr.providers`); every host bridge
  implements `inputs_from_readings` and every `adapter.rs` gains `provide` /
  `read` — regenerate crates generated before this change. `TickRequest` gains
  optional `readings`. `bdl-manifest.json` gains `adapter.sources[]` (additive).
- Developers:
  `bdl_catalogue::{Catalogue, Origin, PackageId, InputProfile, Transducer, InputEntry, OutputEntry, builtin_inputs, builtin_outputs}`
  (`bdl_output::realization` re-exports `Encoder`, `OutputProfile`, `Catalogue`;
  `profiles()` / `profile()` stay), `bdl_output::provision`,
  `bdl_model::InputProfileId`, `DeviceBinding::{source, provider}`,
  `DeviceKind::DigitalInput`, `EditOp::{SetDeviceSource, SetDeviceProvider}`,
  `EditError::NotASource`,
  `bdl_lower::{Provision, Provisions, lower_with_provisions}`,
  `bdl_exec_ir::{ProviderPlan, ExecIr::providers, interp::provide}`,
  `bdl_compiler::{SourceProvision, MissingKind::{SourceNoDevice, ProviderInvalid}, target::source_kind}`,
  `DeploymentAnalysis::{provisions, unprovided_sources, provision_blocked}`,
  `bdl_codegen_rust::adapter::{SourceKind, ProviderBinding, AdapterPlan::providers, host_inputs_from_readings}`,
  `targets::Entry::{reads, source_peripheral}`, `manifest::AdapterSourceEntry`,
  `bdl_runtime_adapter::{LevelSource, read_level, SourceBinding}`,
  `bdl_runtime_embassy_rp::Sense`,
  `bdl_runtime_host::{ReadingError, TickRequest::readings, HostProgram::inputs_from_readings}`;
  `bdl_syntax` `ProviderFix`; `bdl_text::print::device` prints `for <source>`
  and `provider`.

## Evidence

`crates/bdl-compiler/tests/source_provision.rs` (the chain end to end, the host
and the interpreter from the same readings, two providers with one semantic
trace, the simulation untouched, the old project as an incomplete deployment,
every refusal, the cross-build), `crates/bdl-output/src/provision.rs` (the
judgments, origin never reaching one), `crates/bdl-text/tests/workspace.rs`
(`for <source>` / `provider` round trip),
`crates/bdl-daemon/tests/system_e2e.rs`, `apps/studio/test/deploy_test.dart`
(the provider row, the e2e provision on the Pico),
`apps/studio/test/smart_lamp_e2e_test.dart`.
