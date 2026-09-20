---
kind: architecture
area: daemon
status: current
---

# Firmware: build, artifact, flash

How a project becomes an image on a board, and how the board is told. The daemon
owns the whole path; Studio asks and watches
([ADR-0039](../decisions/0039-the-daemon-owns-the-build-and-the-flash.md)). The
compiler's judgments are upstream and untouched:
[embedded-adapter.md](embedded-adapter.md) says what firmware is generated and
when it is refused; this page says how it is built, packaged, kept honest and
written to a board.

## Where it sits

```text
Studio (Deploy page)         bdld                                     the host
  BuildFirmware ───────────▶ firmware::run(plan, snapshot)
                              Checking    compile_for_target        (in memory)
                              Generating  <project>/build/<target>/ (files)
                              Preparing   cargo, the target, runtime/
                              Compiling   cargo build --message-format=json ──▶ rustc, the HAL
                              Packaging   ELF ─▶ UF2                (pure)
                              Completed   bdl-build.json            (the record)
  ◀── BuildProgress events, one per stage and per crate; the last carries BuildStatus
  GetBuildStatus ──────────▶ the record + "is it still this design?" (identity)
  ListFlashDevices ────────▶ firmware::flash::discover: RPI-RP2 volumes, probe-rs probes
  FlashFirmware ───────────▶ firmware::flash::flash(device, artifact)
  ◀── FlashProgress events: Preparing · Writing · Restarting · Completed | Failed
```

| Piece                          | Where                                             | Owns                                                                                                                          | Never                                                   |
| ------------------------------ | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| the plan                       | `crates/bdl-daemon/src/firmware/mod.rs` `Plan`    | the board's entry, the crate's directory, cargo's target directory, the runtime directory, the settings, the compiler options | a judgment about the design                             |
| the build                      | `firmware::run`                                   | the six stages, every failure as a value with its stage and code, the record                                                  | a pin, a profile, a refusal of its own about the design |
| readiness                      | `firmware/readiness.rs`                           | `build_ready` and the ordered `build_blockers` from the deployment report and the compiler's dry run                          | a judgment: it orders what the judges said              |
| the image                      | `firmware/uf2.rs`                                 | the ELF's flash segments laid into 256-byte pages in the UF2 container, tagged with the family id                             | linking, relocation, a check beyond the container's     |
| the flash                      | `firmware/flash.rs`                               | discovery (bootloader volumes, probes), the copy, the wait for the restart, `probe-rs download` / `reset`                     | choosing between several devices                        |
| the requests                   | `firmware/protocol.rs`                            | the thread a build or flash runs on, the events, the status, the refusals (`build.busy`, `flash.ambiguous_device`, …)         | a second build at once; a flash of a stale image        |
| the target's flash metadata    | `crates/bdl-codegen-rust/src/targets` `FlashSpec` | the UF2 family id, the bootloader volume's label, the flash range, the `probe-rs` chip name — per entry, like its triple      | tooling                                                 |
| the generated `rust-toolchain` | `targets::Entry::toolchain_file`                  | the channel the runtime was written against and the board's target, so `rustup` installs both where the crate is built        | the host's default toolchain                            |
| the demo templates             | `crates/bdl-daemon/src/templates.rs`              | the Button → Lamp design, guided and wired, as textual BDL written by `init_project_with_source`                              | a project kind of its own                               |
| the page                       | `apps/studio/lib/ui/pages/firmware_section.dart`  | one primary action per state from the daemon's facts; the events shown; the choice between devices                            | a tool, a judgment, a guess                             |

## The build, stage by stage

1. **Checking** — `compile_for_target(snapshot, board, options)` in memory. Not
   generated means the deployment is not ready: the failure is `build.not_ready`
   with the compiler's own diagnostics (`adapter.*`, `backend.*`, the analysis's
   errors). Nothing was written.
2. **Generating** — the crate's files under `<project>/build/<target>/`; a file
   whose content is unchanged is left alone so cargo's fingerprints hold. The
   compiler options point the crate at the runtime crates by absolute path
   (`BDL_RUNTIME_DIR`, else the `runtime/` found above the running `bdld`, else
   above the working directory); `require_complete`, bounded memory.
3. **Preparing** — the runtime directory exists (`build.runtime_missing`),
   `cargo --version` answers (`build.toolchain_missing`; `$CARGO`, then `PATH`,
   then `~/.cargo/bin` — Studio is launched without a shell), the sysroot has
   the board's target (`build.target_missing`, with the one `rustup target add`
   to run — the check runs in the crate's directory so the crate's
   `rust-toolchain.toml` applies and rustup installs what it lists).
4. **Compiling** —
   `cargo build --release --target <triple> --features <feature> --bin <package>-<feature> --message-format=json-render-diagnostics`
   in the crate's directory, `CARGO_TARGET_DIR` the crate's own `target/`,
   `RUSTFLAGS` cleared. Stdout is read line by line as it comes: a
   `compiler-artifact` is one crate done (an event with the count; the one whose
   `target.name` is the firmware binary names the ELF — cargo's own word for
   where it is, never a guessed path); a `compiler-message`'s rendered text is a
   detail line for the advanced view; `build-finished` says whether it
   succeeded. Stderr is kept for the failure. A cancel flag set from the
   coordinator ends the build at the next line (`build.cancelled`, the child
   killed). A non-zero exit is `build.cargo_failed` with the last 200 lines — or
   `build.target_missing` when the words say the target is absent.
5. **Packaging** — for an entry with a UF2 family, the ELF's `PT_LOAD` segments
   whose load address lies in the flash range become pages; segments in RAM
   (`.bss`, a `.data` image already placed in flash by its paddr) are not part
   of the image. `<package>-<feature>.uf2` beside the crate. An entry without a
   family (the Arduino) keeps the ELF as the artifact.
6. **Completed** — `bdl-build.json` beside the crate: the target, package,
   triple, command, directory and the `Artifact` (path, kind, ELF, identity,
   time, revision, compiler version, size). A reopened project reads it.

## The identity, and what "stale" means

```text
identity = sha256( "bdl-firmware-identity/1", compiler version, target id,
                   tick_micros, triple, every generated file's path and text )
```

The generated crate is a deterministic function of the design, the deployment,
the board and the settings — everything the firmware is made of — so its content
is the artifact's identity. `GetBuildStatus` recomputes the identity of what the
project would build now (a dry `compile_for_target`) and `artifact_fresh` is
equality. A moved node changes nothing; a pin, a profile, a formula or the board
changes the crate. `FlashFirmware` refuses a stale artifact
(`flash.artifact_stale`); Studio offers _Build again_ and never _Flash_ while
stale, and says the board runs an earlier design when it was flashed. No build
database: a hash and a record file.

What the identity does not cover: an edit to the runtime crates themselves
(cargo recompiles them on the next build; the record says nothing). A
developer's concern, noted as deferred in the change fragment.

## Readiness, once

The page must offer _Build_ exactly when the build's first stage would pass,
without a second judgment in Dart. `AnalyzeDeployment` therefore carries
`build_ready` and `build_blockers[]`, composed in the daemon from what the
judges already said, in one order:

1. the deployment report's `missing[]` (the design's own problems first, then
   the deployment's: an output or a Source without a device, a device for
   nothing, an invalid realization or provider);
2. the placement's dead end (`placement_blocked`, with the device);
3. what the firmware pass alone refuses — the compiler's dry run's error
   diagnostics not already named: a provider this board's adapter cannot read,
   an unbounded collection, a profile with no sink here (`adapter.*`,
   `backend.*`), with the providing device attached to a provider refusal;
4. a board with no target entry (`adapter.target_unsupported`).

`build_ready` is "the dry run generated and nothing above is listed". Each
blocker carries a code, product language, an explanation and the object it is
about (output, device, relationship) so a page can lead to it.

## Flashing

Two ways, both the daemon's:

- **The bootloader volume.** The RP2040's ROM bootloader mounts as a drive named
  `RPI-RP2` while BOOTSEL is held at plug-in. Discovery reads `INFO_UF2.TXT`
  under the platform's mount roots (`/Volumes`; `/media/<user>`,
  `/run/media/<user>`, `/media`, `/mnt`; the drive letters D–Z) — or
  `BDL_UF2_ROOTS` — and names every volume whose board id matches the entry's
  label. Flashing writes the UF2 in one go, syncs, then waits up to ten seconds
  (`BDL_FLASH_RESTART_WAIT_MS`) for the volume to disappear: the board
  restarted. A volume that stays says so in the outcome — the image was written;
  the board did not leave bootloader mode by itself — rather than claiming a
  restart it did not see. No tool, no probe, no second board: the demo's path.
- **A debug probe.** When `probe-rs` is installed (`BDL_PROBE_RS`, `PATH`,
  `~/.cargo/bin`) and the entry names a chip, `probe-rs list` is parsed
  (`[n]: name -- vid:pid:serial (kind)`) into devices, and a flash runs
  `probe-rs download --chip <chip> --probe <selector> <elf>` then `reset`.
  Second because it needs hardware the demo forbids (a probe on the SWD pins).

`ListFlashDevices` returns both the devices and the methods with whether each is
available here and what to do (_Hold BOOTSEL…_, _probe-rs is not installed…_).
`FlashFirmware` with no device named takes the one reachable device and refuses
otherwise: `flash.no_device`, `flash.ambiguous_device` (several — a client must
choose), `flash.unknown_device` (the named one went away), `flash.no_artifact`,
`flash.artifact_stale`, `flash.busy`. A running flash reports Preparing ·
Writing · Restarting · Completed, or Failed with `flash.write_failed`,
`flash.device_gone`, `flash.tool_failed`, `flash.unsupported`.

## The requests

`BuildFirmware { target_id, revision? }` answers `Ack` at once (or refuses:
`build.busy`, `build.unknown_target`, `build.stale_revision`) and the build runs
on its own thread against the snapshot taken then;
`BuildProgress { build_id, target_id, stage, message, done?, detail, status? }`
events follow, the last with the `BuildStatus`. `GetBuildStatus { target_id }`
answers
`BuildStatus { running, stage, artifact?, artifact_fresh, failure?, generated_dir, command, triple, output[] }`
for any target at any time. `CancelBuild` sets the flag. `ListFlashDevices` /
`FlashFirmware` / `FlashProgress` as above. `ListTemplates` and
`InitProject.template` are the demos. All additive in protocol 0.26
(`docs/spec/protocol.md`).

The coordinator handles these five before the pure dispatch because they need
the event sender; one build and one flash at a time; the session is never
touched from the worker thread — it got a clone of the snapshot.

## The command line

`bdld build <project> --target <board>` runs the same `run` printing each stage,
and on failure the stage, code, message, explanation, the compiler's diagnostics
and the tool's output; exit 1 refused or failed, 2 did not open.
`bdld flash <project> --target <board> [--device ID] [--list]` runs the same
discovery and flash. `bdld init <dir> [--template ID]` and `bdld templates`
write and list the demos. `--json` on each.

## The page

`FirmwareSection` (Studio) renders one state from `DeploymentAnalysis.build_*`
and `FirmwareState` (the last `BuildStatus`, the running build's or flash's last
event, the devices as last listed, the last flash): a progression Deployment ·
Build · Flash · Observe and one primary action — the first blocker with its
remedy, _Build_, the running stage with Stop, the failure with _Build again_ and
the Details open, the fresh image with the board or the way to make one
reachable, the flash's outcome with the trial named in the design's own Sources
and outputs, the stale image with _Build again_ only. `GetBuildStatus` is asked
after every deployment analysis (so after every revision), and the devices
whenever a fresh image appears or on _Look again_. The reasoning and the tools
studied are in `studio-ui.md` § 15.

## Evidence

`crates/bdl-daemon/tests/firmware_e2e.rs` (the templates; readiness — the
smallest blocker first, cleared as the deployment is made, a pin the board
lacks, a provider the Nano cannot read; the stages with a stand-in cargo; a
compiler failure with its words; the missing runtime; freshness across a pin
edit, its reversal and a reopen; a flash with no device, two, a stale image, and
one written to a pretend volume; a probe listed and flashed through a stand-in
`probe-rs`; the real cross-build to a UF2 whose first block is the boot block),
`crates/bdl-daemon/src/firmware/uf2.rs` (pages, a shared page, every refusal),
`flash.rs` (the probe list), `tests/cli.rs`;
`apps/studio/test/firmware_test.dart` (every transition and page state),
`firmware_e2e_test.dart` (the wired demo through the real `bdld`). The wiring
and the LED are not verified on hardware: `docs/evidence/pico-smoke-test.md`.
