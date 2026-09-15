# Hardware model

The Phase-7 validation layer of the paper, kept independent of any HAL.

## Pipeline

```
OutputId  →  DeviceKind  →  Requirements  →  solve(board)  →  Assignment
                                                  ↓ (none)
                                              diagnose → Explanation
```

The design (`Δ, Κ, Ω, β`) is never an argument of the solver. Swapping the
board re-solves the same requirements with the design untouched.

## Boards are declarative data

`hardware/boards/<board>.toml` (schema versioned) states, per resource:
capabilities, per-capability *unit* (the timer or peripheral behind it),
and the board-wide sharing policy (buses shareable, everything else
exclusive). Example vocabulary:

```
digitalIn digitalOut pwm analogIn interrupt i2cSDA i2cSCL spiMOSI spiMISO spiSCK spiSS uartTX uartRX
```

No Rust source fragments live here. A *separate* platform mapping
(`hardware/platforms/<board>.toml`, planned) answers "logical resource GP15
→ HAL expression `p.PIN_15`".

## Devices generate requirements

`hardware/devices/<kind>.toml`: an H-bridge channel needs `pwm` +
`digitalOut`; an I2C sensor needs `i2cSDA` + `i2cSCL` on the *same* unit; a
quadrature encoder needs two `interrupt` lines; a UART needs `uartTX` +
`uartRX` on the same unit. A requirement may be pinned to a fixed resource
(manual pin choice) — a deployment constraint, not a design change.

## Solver

Unary + binary constraints ⇒ validity is prefix-closed ⇒ exhaustive DFS with
pairwise pruning is sound and complete (proved in Lean). The Rust port must:

* iterate resources and requirements in a documented stable order so the
  assignment is deterministic;
* report `Explanation::{NoCapableResource, Blocked { blockers }}` as *a*
  dead end under greedy placement, meaningful only after `solve` returned
  none — not a minimal unsat core.

## Golden cases from the formal development

Arduino Nano (table in `docs/02-kernel-spec.md` §9) — kept as allocation
examples even though the first firmware target is RP2040:

* 4 × H-bridge + IMU: SAT, `M1→D3/D0 M2→D5/D1 M3→D6/D2 M4→D9/D4 IMU→A4/A5`
* 7 × PWM: UNSAT (six PWM pins); SAT on the mock "big" board
* 2 × interrupt + 6 × PWM: UNSAT (D3 is both) — counting is not feasibility
* 4 × PWM on distinct timers: UNSAT (three timers)
* two I2C sensors on A4/A5: SAT (bus sharing); two PWM pinned to one pin: refused

## Feasibility is not typing

Board changes and manual pins invalidate `Deployment` only. Feasibility is
target-relative evidence, re-established from scratch, never merged with the
refinement-surviving evidence of the checker.
