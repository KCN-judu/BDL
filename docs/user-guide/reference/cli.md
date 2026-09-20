# Command line

`bdld` is Studio's compiler service; it also has three headless
commands that read a project — flat, system or text — with the same
loader Studio and the language server use.

| Command | What it does | Exit code |
| --- | --- | --- |
| `bdld check <project>` | opens the project and prints every finding (`error[code] place: message`, with `file:line:col` for findings about the source files), then one line: name, counts, *checks, outputs complete* / *checks, outputs not yet complete* / *N error(s)* | 0 no errors · 1 errors · 2 the project did not open |
| `bdld compile <project> [--out DIR]` | generates the Rust crate into `DIR` (default `target/bdl`); refuses while the design is not ready, printing why | 0 written · 1 not ready · 2 did not open |
| `bdld compile <project> --target rp2040_pico [--tick-micros N] [--period domain=N …]` | also generates the firmware for that board beside the core: every output needs a device with a realization placed on the board, and a design with a Source is refused (no device provides values yet); `--tick-micros` is the firmware's base tick (default 10000); build it with `cargo build --release --target thumbv6m-none-eabi --features rp2040` in `DIR` | 0 written · 1 refused, printing why · 2 did not open or unknown board |
| `bdld simulate <project> [--ticks N] [--input rel=value …]` | runs the reference evaluator for `N` activations (default 1) with constant inputs and prints every relationship's value per tick | 0 · 1 findings · 2 did not open or a runtime error |
| `bdld migrate-unit-domain <project> [--dry-run] [--json]` | rewrites every legacy zero-input signature `mapping f : A` to the preferred `mapping f : () -> A`, one insertion each — comments, spacing, definitions and identities untouched; refuses if the design would change; `--dry-run` only reports | 0 · 2 did not open or refused |
| `bdld migrate-drive-by <project> [--dry-run] [--json]` | rewrites every legacy drive `drive o = m` to the preferred `drive o by m` — the `=` becomes `by`, nothing else moves; the same guards and options as above | 0 · 2 did not open or refused |

`--json` on any of them prints one JSON object instead of text.

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
