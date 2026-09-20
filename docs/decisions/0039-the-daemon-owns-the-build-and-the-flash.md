---
id: ADR-0039
status: accepted
date: 2026-09-20
area: daemon
supersedes: []
superseded-by: []
related: [ADR-0002, ADR-0015, ADR-0037, ADR-0038]
fv: []
---

# ADR-0039: The daemon owns the build and the flash; an artifact's identity is the content of what it was built from

## Status

Accepted (the first hardware demo: Button → rule → Lamp on a Raspberry Pi Pico,
roadmap priorities 2 and 3).

## Context

Since ADR-0037 and ADR-0038 a design with a Source and an output compiles to
firmware for the Pico, but only by hand: `bdld compile --target`, then `cargo`
in the generated crate, then a tool of the reader's choosing to reach the board.
A designer who is not a Rust developer could not get from the Deploy page to a
running board, and nothing in the product knew whether an image on a board was
the design on screen.

Three things bounded the choice. ADR-0002 makes `bdld` the process boundary:
Studio renders what the daemon says and runs no tool. The demo must need no
hardware beyond the Pico, a button and a cable (no debug probe). And the
compiler's judgments about what can be built are already made, in
`compile_for_target`; a second opinion in Dart is exactly what ADR-0001 forbids.

## Decision

1. **The daemon builds.** `BuildFirmware` runs, on a thread of the daemon's, the
   whole path — the compiler's own check, the crate written under
   `<project>/build/<target>/`, the toolchain located, `cargo build` driven with
   its JSON messages read as they come, the image packaged — and reports each
   stage as an event with a product-language message and the tool's words as
   details. A failure is a value: the stage it happened in, a stable code, a
   message, a remedy, the diagnostics or the tool's tail. The same runs as
   `bdld build`.
2. **The daemon flashes.** `FlashFirmware` writes the image to one device
   through a method the daemon owns: the board's own bootloader as a mounted
   volume (the Pico's `RPI-RP2`) by copying the UF2, or `probe-rs` when it is
   installed and a probe is wired. `ListFlashDevices` names what either reaches
   now. With several devices and none named, nothing is written.
3. **The artifact's identity is a content hash** of the generated crate, the
   board, the settings and the compiler version, recorded beside the image. The
   daemon judges freshness by recomputing it; a stale image is never flashed and
   never presented as current. No build database.
4. **Readiness is composed once, in the daemon.** `AnalyzeDeployment` carries
   `build_ready` and the ordered `build_blockers` — the report's missing items,
   the placement's dead end, the firmware pass's own refusals — so a client
   offers Build exactly when the build's first stage would pass and names the
   smallest blocker without a rule of its own.
5. **Flash metadata is target metadata.** The UF2 family id, the bootloader
   volume's label, the flash range and the `probe-rs` chip name sit on the
   codegen target entry beside its triple; the generated crate carries a
   `rust-toolchain.toml` pinning the runtime's channel and the board's target.
6. **The demo is a template of the daemon's**, written through the same `init`
   path as an empty project, in two variants: the design alone, and the design
   with its deployment.

## Alternatives

- **Studio drives cargo and copies files.** Rejected: it violates ADR-0002,
  duplicates the toolchain search and the target's metadata in Dart, and puts a
  process the product depends on where no test of the daemon can reach it.
- **probe-rs as the only flash path.** Rejected for this slice: a bare Pico has
  no probe; the roadmap's intent (probe-rs) survives as the second method, used
  when the tool and a probe exist. Programming the Pico as a probe of another
  Pico needs two boards and wiring the demo must not require.
- **`picotool` / `elf2uf2-rs` as external tools.** Rejected: the UF2 container
  is a hundred lines of pure code and the daemon must not depend on a tool a
  tester has to install; the converter is tested against the real cross-build.
- **Staleness by revision number.** Rejected: a layout change bumps the revision
  and would mark a correct image stale; a content hash is exact and costs a dry
  compile.
- **Persisting the chosen target with the project.** Not changed: the board
  remains a session preference (ADR-0015's stance); the build directory is per
  target, so switching boards keeps each board's record.

## Consequences

- Studio's Deploy page gains a Firmware section with one primary action per
  state and never a judgment; `docs/architecture/studio-ui.md` § 15.
- Protocol 0.26 (additive): the build, status, cancel, flash-devices, flash and
  templates requests, the two event kinds, `BuildBlocker`,
  `InitProject.template`.
- The generated crate for a board is built wherever the project lives, not in a
  checkout; the runtime crates are still path dependencies, found above the
  daemon or by `BDL_RUNTIME_DIR` — the packaging of the runtime for an installed
  Studio is open.
- Records: `docs/architecture/firmware-build.md`, `status.md` rows, the change
  fragment `2026-09-firmware-build-flash.md`; roadmap priorities 2 and 3
  deleted, the first external UX study placed first.
