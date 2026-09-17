---
kind: architecture
area: runtime
status: current
---

# Supplied Rust computation blocks

_Design, not implementation: nothing below exists in the code yet
(`docs/project/status.md`, roadmap). The obligations follow ADR-0005._

Expert-supplied Rust is a _computation boundary_, not an escape from BDL
semantics.

## Two shapes

```rust
trait BdlPureComponent<I, O>          { fn eval(input: &I) -> O; }
trait BdlStatefulComponent {
    type Input; type Output; type State;
    fn init() -> Self::State;
    fn step(state: &mut Self::State, input: &Self::Input) -> Self::Output;
}
```

The exact API is not frozen until a prototype exists. A stateful component is
instantiated into fresh declarations at each use site (temporal state belongs to
declarations, not functions — the kernel's top-level `delay` restriction).

## What a component never receives

GPIO or PWM handles, `OutputId` writers, embedded-hal devices, or any BDL output
capability. Physical effect still goes
`BDL value → one final OutputId driver → platform adapter → hardware`. Otherwise
supplied code would bypass `SingleDriver`, semantic identity, clock discipline
and hardware allocation.

## Trust classification (v0.1)

Supplied Rust is **trusted implementation code** with a constrained BDL-facing
API. The trait boundary is _not_ a sandbox: source compiled into firmware can
use `unsafe`, platform crates and global state if the build allows it.
Documentation must never claim otherwise.

## Manifest: what BDL cannot derive

Each component carries a manifest declaring what the kernel derives for a native
mapping but must be _told_ for a supplied one:

| Obligation                    | Evidence origin (one of)                            |
| ----------------------------- | --------------------------------------------------- |
| input / output types          | declared                                            |
| determinism, totality         | declared · tested · statically checked              |
| output range, monotonicity    | declared · tested · model-checked · formally proved |
| worst-case state size         | declared · statically checked                       |
| expected update domain / rate | declared                                            |
| execution-time bound          | declared · measured                                 |

Evidence origins are displayed with different strength. A latency bound resting
on a declared worst-case time must not look like one derived from the design.
