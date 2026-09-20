# From the Deploy page to a running Pico: build, flash, the Button → Lamp demo (protocol 0.26)

- Date: 2026-09-20
- Area: daemon, protocol, codegen, studio, l10n, docs
- Affected: designers, protocol clients, developers, generated crates
- Related: ADR-0039, ADR-0037, ADR-0038, ADR-0002,
  [2026-09 Source provision](2026-09-source-provision.md),
  [2026-09 Embedded adapter](2026-09-embedded-adapter.md)

## What changed

- **Build from the Deploy page.** Under the placement, a _Firmware_ section with
  the steps _Deployment · Build · Flash · Observe_ and one action for the
  current step. Before a build is possible it names the first thing in the way —
  _pressed has no device on Raspberry Pi Pico._, _lit is not fully defined._
  with a link to the Design page — from the daemon's ordered list; when
  everything holds, **Build for Raspberry Pi Pico (RP2040)**. The build reports
  its stages (_Checking the deployment_, _Generating the crate_, _Preparing the
  toolchain_, _Compiling_ with the crates done, _Writing the image_) and can be
  stopped; a failure names its stage and remedy (_The Rust target for … is not
  installed._ — _Run `rustup target add thumbv6m-none-eabi` once_) with the
  compiler's words behind **Details**.
- **Flash from the Deploy page.** With a current image the card shows the board
  a flash would reach: none — _Hold BOOTSEL while plugging the board in over
  USB; it appears as a drive named RPI-RP2_ and **Look again**; one — **Flash**;
  several — a choice, never a guess. Then _Flashed to … at …_ and _Try it: act
  on pressed; lamp should follow the design._ The Pico's own bootloader is the
  path (no tool, no probe); `probe-rs` is used when it is installed and a probe
  is wired.
- **A stale image is never flashed.** The daemon fingerprints what the image was
  built from; after any change to the design, a device, a profile, a pin or the
  board the card says _The firmware is from an earlier design or deployment._,
  offers only _Build again_, and — if the board was flashed — _The board runs an
  earlier design_. A moved node is not a change.
- **Demos on the Welcome page.** _Button → Lamp_ (the design: the Source
  `pressed`, `lit = pressed`, the output `lamp`) and _Button → Lamp, wired_
  (with `button` on GP2 as _GPIO input, active low_ and `led` on the on-board
  LED as _GPIO, on/off_), each a new project from a template.
- **`bdld build`, `bdld flash`, `bdld init --template`, `bdld templates`**: the
  same path from a terminal, stage by stage, with exit codes; `--list` shows
  what a flash could reach.
- **The generated crate carries `rust-toolchain.toml`** (the runtime's channel,
  the board's target): built outside a checkout, `rustup` installs both on first
  use.
- **Protocol 0.26** (additive): `BuildFirmware`, `GetBuildStatus`,
  `CancelBuild`, `ListFlashDevices`, `FlashFirmware`, `ListTemplates`,
  `InitProject.template`; events `BuildProgress`, `FlashProgress`;
  `DeploymentAnalysis.build_ready` / `build_blockers[]`.
- **Guide:** [Your first board](../../user-guide/getting-started/pico-demo.md) —
  wiring, the demo, Deploy, Build, Flash, the trial; the Deploy page's Firmware
  table; the CLI; troubleshooting for every build and flash failure. A manual
  smoke-test checklist: `docs/evidence/pico-smoke-test.md`. A study protocol for
  the first external UX test: `docs/project/ux-study-pico.md`.

## Compatibility and migration

- Designers: nothing to do. A project builds where it lives: `build/<target>/`
  appears inside the project folder (git ignores `build/`); delete it freely.
  Building needs Rust (`rustup`) on the machine; the board's target is installed
  on first use; the first build downloads the board's libraries.
- Protocol clients: 0.26 is additive; a 0.25 client keeps working and sees no
  firmware. `InitProject.template` is optional.
- Generated crates: a crate generated for the RP2040 now includes
  `rust-toolchain.toml`; crates generated before build as before.
- Developers: `bdld` needs the repository's `runtime/` — found above the binary
  or the working directory, or named by `BDL_RUNTIME_DIR`; tests use `CARGO`,
  `BDL_UF2_ROOTS`, `BDL_PROBE_RS`, `BDL_FLASH_RESTART_WAIT_MS` to stand in for
  the toolchain and the board. New:
  `bdl_daemon::firmware::{Plan, run, Stage, Failure, Artifact, Record, readiness, flash, uf2, protocol}`,
  `templates`;
  `bdl_codegen_rust::targets::{FlashSpec, Uf2Family, Entry::{flash, binary, toolchain_file}}`;
  `bdl_text::init_project_with_source`;
  `Session::{init_with_source, compiler_version}`; Studio `FirmwareState`,
  `firmware_section.dart`, the actions `BuildRequested`, `CancelBuildRequested`,
  `FlashRequested`, `FlashDevicesRequested`, `FlashDeviceChosen`,
  `TemplatesRequested` and their responses; `sha2` in the workspace.
- Roadmap: priorities 2 (build orchestration) and 3 (flash) landed; the first
  external UX study is placed first; the adapter's breadth moves to second until
  that study says otherwise.

## Deferred

Flashing the Arduino Nano from Studio (`avrdude`); a probe-first path; the
runtime crates packaged with an installed Studio (today a checkout or
`BDL_RUNTIME_DIR`); the runtime crates' own edits in the identity; a progress
fraction (cargo gives no total — the count of crates stands in); choosing the
board's tick and the domains' periods on the page; a build log kept across
sessions; a Monitor page reading values back; the study itself.

## Evidence

`crates/bdl-daemon/tests/firmware_e2e.rs`, `crates/bdl-daemon/src/firmware/`
unit tests, `crates/bdl-daemon/tests/cli.rs`,
`apps/studio/test/firmware_test.dart`,
`apps/studio/test/firmware_e2e_test.dart`,
`crates/bdl-compiler/tests/embedded_rp2040.rs` (the toolchain file). The
physical smoke test has **not** been run: no Pico was available to this change.
