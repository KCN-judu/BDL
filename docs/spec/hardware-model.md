---
kind: specification
area: deployment
status: current
---

# Hardware model

The Phase-7 validation layer of the paper, kept independent of any HAL.
Implemented in `crates/bdl-hardware` (model, devices, boards, solver) and driven
by `bdl-compiler::analyze_deployment`.

## Pipeline

```text
OutputId  →  DeviceBinding{kind, realization?, fixed_pins}  →  Requirements  →  solve(board)  →  Assignment
                                                                                    ↓ (none)
                                                                                diagnose → DeadEnd
```

A binding may also name an **output realization profile** (`realization`,
ADR-0036): the profile prescribes the kind, and its encoder is judged apart from
this pipeline — the solver never sees an encoder, the encoder never sees a board
(`docs/architecture/output-realization.md`). The kind stays the whole of what
the _solver_ reads.

The design (`Δ, Κ, Ω, β`) is never an argument of the solver. Swapping the board
re-solves the same requirements with the design untouched. Hardware allocation
may not change `Ty`, `HasType`, `Grant`, causality, clocks, the evaluator or a
mapping's meaning — it decides only whether the behaviour can be realised on a
target (ADR-0006, ADR-0015).

## Model (`bdl-hardware::model`)

```text
Capability   digital_in digital_out pwm analog_in interrupt i2c_sda i2c_scl
             spi_mosi spi_miso spi_sck spi_ss uart_tx uart_rx        (opaque to the solver)
ResourceId   "D3", "A4", …                                          (board-local name)
Resource     { id, capabilities: set, units: Capability → UnitId }   (the timer/peripheral behind each capability)
Hardware     { name, resources: [Resource], shareable: set<Capability> }
Requirement  { id: RequirementId{device, index}, capability,
               fixed: Option<ResourceId>, group: Option<(GroupId, Same | Distinct)>, label }
Assignment   RequirementId → ResourceId
```

`RequirementId { device, index }` is the device binding's stable id plus the
index in its kind's requirement list: independent of solver traversal, stable
under unrelated edits, and the key of `DeviceBinding.fixed_pins`. Changing a
device's _kind_ re-indexes its requirements (DI-23).

Constraints, exactly the Lean development's:

- `ReqOK r a` — `a` has `r.capability`, and `r.fixed` (if any) is `a`.
- `Compatible (r₁,a₁) (r₂,a₂)` — if `a₁ = a₂` then both requirements need the
  same capability and that capability is board-shareable; if both are in the
  same group, `Same` ⇒ `unit_of(a₁) = unit_of(a₂)`, `Distinct` ⇒ they differ.
- `ValidFor` = every requirement assigned, `ReqOK` each, `Compatible` all pairs.
  `validate` lists every violation of a candidate assignment.

Sharing is per capability: A4 carries two I2C sensors' SDA lines but only one
digital output. Units are per capability too: D3's PWM is timer 2, its interrupt
is INT1.

## Boards are declarative data

`Hardware` is plain serde data; `hardware/boards/<name>.toml` is its TOML form
(`arduino_nano.toml`, `big_board.toml` are generated from `bdl-hardware::boards`
and checked in; the round-trip test fails when they drift —
`BDL_WRITE_BOARDS=1 cargo test -p bdl-hardware` regenerates). Besides the
resources, a board carries chooser metadata the solver never reads —
`display_name`, `description`, `family` — and `boards::describe(&hw)` derives a
`TargetDescriptor` (id, wording, resource count, per-capability counts and
shareability) for `ListTargets`. `Capability::label()`,
`Hardware::describe_resource(id)` (_D3: digital in, digital out, PWM (timer 2),
interrupt_) and `devices::device_kind_label(kind)` are the designer-facing
wording used by the Deploy read model
(docs/architecture/deployment-read-model.md). No Rust source fragments live
here. The mapping "logical resource `GP15` → HAL expression `p.PIN_15`" is the
target entry's (`bdl-codegen-rust::targets::rp2040`, derived from the pad number
and checked against the board file's PWM unit,
docs/architecture/embedded-adapter.md), not a second file. Boards:
`arduino_nano`, `big_board` (mock) and `rp2040_pico` (the first embedded target,
ADR-0037). The registry (`boards::registry`) is what `ListTargets` reports;
loading boards from the directory at runtime is the next step.

## Devices generate requirements (`bdl-hardware::devices`)

The surface says only a `DeviceKind`; requirements are derived mechanically, in
a fixed order, and the solver never learns what an IMU is:

| Kind                | Requirements (index: capability) | Relation  |
| ------------------- | -------------------------------- | --------- |
| `PwmChannel`        | 0: pwm                           |           |
| `DigitalOutput`     | 0: digital_out                   |           |
| `HBridgeChannel`    | 0: pwm, 1: digital_out           |           |
| `I2cSensor`         | 0: i2c_sda, 1: i2c_scl           | Same unit |
| `QuadratureEncoder` | 0: interrupt, 1: interrupt       |           |
| `Uart`              | 0: uart_tx, 1: uart_rx           | Same unit |

`DeviceBinding.fixed_pins[index] = "D3"` pins one requirement to a named
resource (manual pin choice) — a deployment constraint, not a design change; an
unknown or incapable name is a `FixedUnavailable` dead end, not an edit error.

## Solver (`bdl-hardware::solve`)

Unary + binary constraints ⇒ validity is prefix-closed ⇒ exhaustive DFS with
pairwise pruning is sound and complete (proved in Lean; the Rust port is
_tested_ against a brute-force oracle on generated small cases, not verified).
Determinism: requirements are ordered by fewest candidates first, ties by
`RequirementId`; candidates by `ResourceId`; the same
`(Hardware, [Requirement])` always yields the same `Assignment`.

`diagnose` reports _a_ dead end under greedy placement in that order —

```text
DeadEnd { requirement, reason: NoCapableResource | FixedUnavailable{fixed} | Blocked{candidates: [(resource, held_by)]}, placed }
```

— meaningful only after `solve` returned none. It is not a minimal unsatisfiable
subset (DI-21). Numeric/electrical constraints (current, voltage, timing) are
out of scope (DI-22).

## Golden cases from the formal development

Arduino Nano (table in `docs/spec/kernel.md` §9) — kept as allocation examples
even though the first firmware target is RP2040:

- 4 × H-bridge + IMU: SAT, `M1→D3/D0 M2→D5/D1 M3→D6/D2 M4→D9/D4 IMU→A4/A5`
- 7 × PWM: UNSAT (six PWM pins); SAT on the mock "big" board
- 2 × interrupt + 6 × PWM: UNSAT (D3 is both) — counting is not feasibility
- 4 × PWM on distinct timers: UNSAT (three timers)
- two I2C sensors on A4/A5: SAT (bus sharing); two PWM pinned to one pin:
  refused

All five are tests in `crates/bdl-hardware/tests/solver.rs`; the Nano case
resolves to exactly that structure.

## Feasibility is not typing

Board changes and manual pins invalidate `Deployment` only. Feasibility is
target-relative evidence, re-established from scratch, never merged with the
refinement-surviving evidence of the checker. `DeploymentAnalysis` is computed
per `(revision, target)`, is never cached with the project, and carries the
status `Feasible | Infeasible | Incomplete` beside — never inside —
`MappingStatus` (docs/architecture/compiler-pipeline.md).
