---
kind: guide
area: deployment
status: current
---

# From a mapping to a pin, end to end

Companion to `docs/guides/execution-walkthrough.md`. That one follows a value
through a tick; this one follows a design _out_ of the language: from a
clock-consistent relationship to a physical output, a device, the requirements
it implies, a board, and a placement — or the reason there is none. Every step
is a pure function you can call from a test; the exact cases are
`crates/bdl-compiler/src/lib.rs::tests::{output_ladder_…, deployment_…}`,
`crates/bdl-hardware/tests/solver.rs` and
`crates/bdl-daemon/tests/stdio_e2e.rs::outputs_and_deployment_over_stdio`.

```text
semantic mapping            cruise : () -> Speed  := 0.5
      ↓  SetMappingClock
clock-consistent decl       Κ cruise = main            (ClockConsistent)
      ↓  CreateOutput / SetMappingDrive
physical output             motor : accepts Speed, clock main;  β cruise = motor   (Driven; output_complete)
      ↓  CreateDevice
device binding              drive : HBridgeChannel, output motor
      ↓  requirements_for
generated requirements      drive/0 pwm,  drive/1 digital_out
      ↓  ListTargets
target board                arduino_nano   (D3 D5 D6 D9 D10 D11 pwm; D0 D1 D2 D4 … digital_out)
      ↓  solve
allocation                  drive/0 → D3,  drive/1 → D0
      ↓
result                      Feasible          |   Infeasible: "D4 cannot carry drive PWM on arduino_nano."
```

## 0. The design

```text
concept Speed          representation = quantity, dimensionless    (sem#0 : q[1])
clock   main
mapping cruise : () -> Speed    formula 0.5          Κ = main
output  motor  accepts Speed    clock main   required
```

`cruise` has no inputs on purpose: an output takes the _value_ of a
relationship, and only a relationship without inputs has the type of a value
(`sem Speed`, not `sem Tilt → sem Speed`) — DI-20.

## 1. Semantic analysis (`bdl-compiler::analyze`)

Passes 1–10 as before: `cruise` elaborates to `mk Speed 0.5`, checks against its
interface, has no dependencies, no cycle, reads nothing across domains →
`ClockConsistent`. Nothing here knows an output exists.

## 2. The output pass (`bdl-output::check_outputs`)

Elaboration put `motor` into `Ω` because it has a domain
(`OutputSpec { accepts: sem Speed, clock: main }`); an output without a domain
is _open_: absent from `Ω`, listed in `open_outputs`, reported
`output.clock_unset` (info), neither driven nor missing.

Before the connection: `states[motor] = Undriven`, `partial_wf = true` (nothing
is wrong), `executable = false`, `output.missing_driver` (info: "motor has no
final target yet."). The design is valid and unfinished.

After `SetMappingDrive { cruise, motor }`, `β cruise = motor` and `DriveWF`
holds edge by edge:

| check        | kernel                      | what fails it                                                         |
| ------------ | --------------------------- | --------------------------------------------------------------------- |
| sink exists  | `Ω o = some spec`           | an open output → `output.clock_unset`                                 |
| exact type   | `Δ.tyView d = spec.accepts` | a mapping with inputs, or of another concept → `output.type_mismatch` |
| exact domain | `Κ d = some spec.clock`     | a domain-agnostic or other-domain driver → `output.clock_mismatch`    |

`SingleDriver`: one declaration per sink. A second driver makes the sink
`Conflict`, every claimant carries `output.multiple_drivers` ("motor already has
a final target." / "Combine competing values upstream into one relationship,
then connect that result to motor."), and _no_ policy — no last-writer, no
priority — merges them. Values compose in relationships; physical outputs do not
merge.

`CompleteOutputs`: every _required_ sink with a domain is `Driven`. Then
`executable`, and with no open output, `output_complete`. None of this moves
`cruise` on the ladder: an output fault is a fault of the connection, so a
contested driver stays `ClockConsistent` and carries the diagnostic.

## 3. A device realises the output (`CreateDevice`)

`drive : HBridgeChannel, output = motor`. The surface says only the kind.
`bdl-hardware::devices::requirements_for` derives, in a fixed order:

```text
RequirementId { device: drive, index: 0 }  pwm          label "drive PWM"
RequirementId { device: drive, index: 1 }  digital_out  label "drive direction"
```

An I2C sensor would add `i2c_sda`, `i2c_scl` in a `Same`-unit group; an encoder
two `interrupt`s; a UART `uart_tx` + `uart_rx` on the same unit. The solver
never learns what any of these are.

The semantic analysis of §2 is byte-for-byte the same whether `motor` is
realised by an `HBridgeChannel`, a `PwmChannel` or a `DigitalOutput` (tested:
`deployment_is_target_relative_and_separate_from_semantics`).

## 4. A target (`ListTargets` → `arduino_nano`)

`Hardware { resources, shareable }` from `bdl-hardware::boards` /
`hardware/boards/arduino_nano.toml`: D3 D5 D6 D9 D10 D11 carry `pwm` (units =
timers 2 0 0 1 1 2), D2 D3 carry `interrupt`, A4/A5 carry `i2c_sda`/`i2c_scl` on
unit 0 and are shareable for those capabilities only, D0/D1 are the UART, every
D pin is `digital_out`.

## 5. Allocation (`bdl-hardware::solve`)

Requirements are ordered fewest-candidates-first (ties by id), candidates by
resource id; DFS with pairwise `Compatible` pruning:

```text
drive/0 pwm          → D3     (first pwm pin)
drive/1 digital_out  → D0     (first digital pin still compatible: D3 is taken, exclusive)
```

`validate` confirms the witness (every requirement placed, `ReqOK` each,
`Compatible` all pairs) — the same three predicates the Lean development proves
the solver sound and complete for; the Rust port is checked against a
brute-force oracle on generated small cases.

Result:

```text
DeploymentAnalysis { target: arduino_nano, status: Feasible, assignment: { drive/0 → D3, drive/1 → D0 }, diagnostics: [] }
```

The revision it was computed for is stamped on it; nothing is cached.

## 6. The same design, made infeasible without changing it

`SetDevicePin { drive, index 0, "D4" }` — the designer wants the PWM on D4. D4
has no `pwm`:

```text
status: Infeasible
dead_end: { requirement: drive/0, reason: FixedUnavailable { D4 }, placed: {} }
deploy.infeasible  "D4 cannot carry drive PWM on arduino_nano."
    explanation: "The pin chosen by hand for drive is not on this board or lacks the
                  needed function. Pick another pin, or let the placement choose."
```

`RunAnalysis` still says `output_complete = true`; `cruise` is still
`ClockConsistent`. Feasibility changed, the design did not.

With seven `PwmChannel` devices the Nano is `Infeasible` with reason
`Blocked { D3 held by L1/0, D5 by L2/0, … }` for `L7/0` — every PWM pin is taken
— and `big_board` (six more PWM pins) is `Feasible`. Two interrupts plus six PWM
is infeasible although the Nano has exactly two interrupt pins and six PWM pins:
D3 is both. Counting is not feasibility.

The dead end is _a_ conflict under one placement order, not a minimal
unsatisfiable core (DI-21); the model has no electrical constraints, so
`Feasible` means "pins can be allocated" (DI-22).

## 6a. What Studio is handed

`bdld` answers `AnalyzeDeployment` with the analysis above _and_ a read model
composed from it and the semantic analysis (`bdl_compiler::deployment_report`,
docs/architecture/deployment-read-model.md): for §5, `deployable = true` and two
`rows` — _motor · drive · H-bridge channel · PWM · D3: digital in, digital out,
PWM (timer 2), interrupt_ and _… · direction · digital out · D0: …_; for §6,
`status = Infeasible`, `design_ready = true`, and a `blocker` — _drive · PWM ·
fixed_unavailable D4 · "D4 cannot carry drive PWM on arduino_nano."_. Nothing in
it needs the solver's vocabulary to render.

## 7. Stopping early is fine

Each of these is a valid, saveable design: `cruise` declared with no formula;
`ClockConsistent` with no output; an output with no domain (open); an output
with no driver (`missing_driver`, info); a driven output with no device
(`Incomplete`, `deploy.output_unrealised`, info); a device bound to nothing
(`Incomplete`, `deploy.device_unbound`, info). Only a contested or ill-formed
connection, and an infeasible placement, are errors — and the second is an error
_about a target_.
