# Command line

`bdld` is Studio's compiler service; it also has headless commands
that read a project — flat, system or text — with the same loader
Studio and the language server use, and the build and flash commands
Studio's Deploy page runs through it.

| Command | What it does | Exit code |
| --- | --- | --- |
| `bdld check <project>` | opens the project and prints every finding (`error[code] place: message`, with `file:line:col` for findings about the source files), then one line: name, counts, *checks, outputs complete* / *checks, outputs not yet complete* / *N error(s)* | 0 no errors · 1 errors · 2 the project did not open |
| `bdld compile <project> [--out DIR]` | generates the Rust crate into `DIR` (default `target/bdl`); refuses while the design is not ready, printing why | 0 written · 1 not ready · 2 did not open |
| `bdld compile <project> --target rp2040_pico [--tick-micros N] [--period domain=N …]` | also generates the firmware for that board beside the core: every output needs a device with a realization placed on the board, and every Source a device with a provider the board can read (an unprovided Source is refused by name, `adapter.source_unprovided`); `--tick-micros` is the firmware's base tick (default 10000); build it with `cargo build --release --target thumbv6m-none-eabi --features rp2040` in `DIR` | 0 written · 1 refused, printing why · 2 did not open or unknown board |
| `bdld compile <project> --target arduino_nano …` | the same for the Arduino Nano over `avr-hal`: a blocking tick loop, PWM on the board's timers, no collections; build it with `cargo +nightly-2025-04-27 build --release --target avr-none -Zbuild-std=core --features arduino_nano` in `DIR` (needs that nightly with `rust-src`, and `avr-gcc`); the exact command is in `bdl-manifest.json` under `adapter.build` | as above |
| `bdld build <project> --target rp2040_pico` | the whole path to the board's image: judges the deployment (*Checking*), writes the crate under `<project>/build/rp2040_pico/` (*Generating*; with a `rust-toolchain.toml` that pins the runtime's toolchain and the board's target, so `rustup` installs both on first use), finds cargo and the runtime crates (*Preparing*; `BDL_RUNTIME_DIR` names the repository's `runtime/` when `bdld` is not run from a checkout), runs `cargo build` printing each crate as it compiles (*Compiling*), writes the UF2 image beside the ELF (*Packaging*) and a `bdl-build.json` record with the image's fingerprint; a failure names its stage and code (`build.not_ready`, `build.toolchain_missing`, `build.target_missing`, `build.runtime_missing`, `build.cargo_failed`, `build.package_failed`) and prints the tool's words | 0 built · 1 refused or failed, printing why · 2 did not open or unknown board |
| `bdld flash <project> --target rp2040_pico [--device ID] [--list]` | writes the last built image to the board: with `--list`, the devices reachable now and how (the Pico's bootloader drive `RPI-RP2` — hold BOOTSEL while plugging in — and, when `probe-rs` is installed, its probes); with exactly one device none need be named; with several, `--device` chooses one and nothing is written otherwise; a stale image (the project changed since the build) is refused: build again first | 0 flashed · 1 failed · 2 nothing built, stale, no or ambiguous device |
| `bdld init <dir> [--name N] [--template ID]` · `bdld templates` | creates a project, empty or from a template — `button-lamp` (the Button → Lamp design for the Raspberry Pi Pico) and `button-lamp-configured` (the same with its devices and pins), as `templates` lists them | 0 · 2 could not create |
| `bdld simulate <project> [--ticks N] [--input rel=value …]` | runs the reference evaluator for `N` activations (default 1) with constant inputs and prints every relationship's value per tick | 0 · 1 findings · 2 did not open or a runtime error |
| `bdld migrate-unit-domain <project> [--dry-run] [--json]` | rewrites every legacy zero-input signature `mapping f : A` to the preferred `mapping f : () -> A`, one insertion each — comments, spacing, definitions and identities untouched; refuses if the design would change; `--dry-run` only reports | 0 · 2 did not open or refused |
| `bdld migrate-drive-by <project> [--dry-run] [--json]` | rewrites every legacy drive `drive o = m` to the preferred `drive o by m` — the `=` becomes `by`, nothing else moves; the same guards and options as above | 0 · 2 did not open or refused |

`--json` on any of them prints one JSON object instead of text.

## A board, end to end

```bash
bdld init my-lamp --template button-lamp-configured
bdld build my-lamp --target rp2040_pico
bdld flash my-lamp --target rp2040_pico --list
bdld flash my-lamp --target rp2040_pico
```

```text
[Checking] Checking the deployment on Raspberry Pi Pico (RP2040)
[Generating] Writing the generated crate
[Preparing] Locating the toolchain
[Compiling] Compiling the firmware for Raspberry Pi Pico (RP2040)
[Compiling] Compiled embassy_rp (170 crates)
…
[Packaging] Writing the image for the board
[Completed] Firmware for Raspberry Pi Pico (RP2040) built
UF2: my-lamp/build/rp2040_pico/bdl_design_my_lamp-rp2040.uf2 (6144 bytes)
```

The build's stand-in when nothing is plugged in: `flash --list` prints
*no device reachable* and the ways to make one reachable.

## Inputs

`--input name=value` sets a relationship that reads nothing to a
constant for every tick: `true` / `false` for a *Bool* concept, a whole
number for a *Count*, a number in the concept's base unit otherwise
(radians for an angle, metres for a length, seconds for a time). A
relationship left without a value and without a definition stops the run
with a *missing input* error.

```bash
bdld simulate lamp --ticks 2 --input tilt=0.7853981633974483
```

```text
tick 0: tilt = Tilt(0.785398 [rad])  dimByTilt = <function>  brightness = Brightness(0.5)
tick 1: tilt = Tilt(0.785398 [rad])  dimByTilt = <function>  brightness = Brightness(0.5)
```

Values print as *Concept(value)* — a truth value as `on` / `off`, a
number with six significant digits — the same text Studio shows; a
relationship with inputs prints as `<function>` (it is applied where it
is called, not sampled); an input you set is listed as the evaluator
read it, at the ticks its domain activated.

## Related

[Project files](project-files.md) · [Textual BDL](../textual/overview.md)
