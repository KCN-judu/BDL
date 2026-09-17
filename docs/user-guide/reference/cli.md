# Command line

`bdld` is Studio's compiler service; it also has three headless
commands that read a project — flat, system or text — with the same
loader Studio and the language server use.

| Command | What it does | Exit code |
| --- | --- | --- |
| `bdld check <project>` | opens the project and prints every finding (`error[code] place: message`, with `file:line:col` for findings about the source files), then one line: name, counts, *checks, outputs complete* / *checks, outputs not yet complete* / *N error(s)* | 0 no errors · 1 errors · 2 the project did not open |
| `bdld compile <project> [--out DIR]` | generates the Rust crate into `DIR` (default `target/bdl`); refuses while the design is not ready, printing why | 0 written · 1 not ready · 2 did not open |
| `bdld simulate <project> [--ticks N] [--input rel=value …]` | runs the reference evaluator for `N` activations (default 1) with constant inputs and prints every relationship's value per tick | 0 · 1 findings · 2 did not open or a runtime error |

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
tick 0: brightness = Brightness(0.5)  dimByTilt = <function>
tick 1: brightness = Brightness(0.5)  dimByTilt = <function>
```

Values print as *Concept(value)*; a relationship with inputs prints as
`<function>` (it is applied where it is called, not sampled); an input
you set is not listed, the relationships computed from it are.

## Related

[Project files](project-files.md) · [Textual BDL](../textual/overview.md)
