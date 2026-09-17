# Deploy

The Deploy page (⌘3) answers _does this design fit this board_. It is a
placement check: for each physical output's device, which board resources it
would use — or the first reason it cannot be placed. It does not build firmware
or flash a board yet; see [What is not there](#what-is-not-there).

```text
┌───────────────┬────────────────────────────────────────────────────────┐
│ Target        │ Feasible on Arduino Nano.                              │
│ [Arduino Nano]│                                                        │
│               │ Placement on arduino_nano                              │
│ Devices       │  PWM light   PWM   D3   digital in, digital out, PWM   │
│ [Add device]  │                                (timer 2), interrupt    │
│ PWM light     │                                                        │
│  PWM channel  │                                                        │
│  Light Output │                                                        │
│  PWM [    ]   │                                                        │
│  Remove       │                                                        │
└───────────────┴────────────────────────────────────────────────────────┘
```

## Target (left)

The boards the compiler service knows. Today: **Arduino Nano** and **Big board
(mock)**, a test target with more PWM pins. The board choice is a **session
preference**: it is not saved with the project, and changing it never changes
the design.

Until a board is chosen the page says _Choose a board to see whether this design
fits it._

## Devices (left)

A **device** realises one physical output on the board. **Add device** creates a
row; in it you set

- a **name**;
- a **kind** — _PWM channel_ (a dimmable light, a servo signal), _Digital
  output_ (a relay, a switched load), _H-bridge channel_ (a motor: one PWM line
  plus one direction line), _I²C sensor_, _Quadrature encoder_, _UART_;
- the **output** it realises (or none, for a sensor that feeds the design);
- one **pin field per requirement** of the kind — leave empty to let the
  placement choose, or type a board pin name (`D3`, `A4`) to fix it by hand;
- **Remove**.

The kind decides what the device needs from the board (a PWM channel needs one
PWM-capable pin; an H-bridge needs a PWM pin and a digital pin; an I²C sensor
needs SDA and SCL on the same bus). You never edit those requirements; the pin
fields are the only manual choice.

Devices are saved with the project. They are deployment data, not design data:
adding, changing or removing one leaves every Design-page verdict unchanged.

## The verdict (centre)

| Verdict                                                     | Meaning                                                                                                                                                                                                                                                                                                                               |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Feasible on Arduino Nano.**                               | every device's requirements were placed on distinct pins with the right capabilities; the _Placement_ table lists device → requirement → pin, with what the pin can do                                                                                                                                                                |
| **Fits Arduino Nano so far — the binding is not finished.** | what is bound fits, but an output has no device (_No device on … for: …_) or a device no output (_Not connected to an output: …_)                                                                                                                                                                                                     |
| **Not feasible on Arduino Nano.**                           | one requirement could not be placed; the page names it — _Could not place PWM light PWM of PWM light._ — and why: _Nothing on … can carry …_ (no pin has the capability), _The pin chosen by hand, D4, cannot carry … here._, or the pins that block it, each with what holds it; _Placed before the dead end:_ lists what was placed |

The verdict is only about the board. Whether the **design** itself is ready —
every relationship checks, no instantaneous cycle, every required output driven
— is the Design page's business and is not restated here: a design whose
formulas are wrong can still be _feasible_, and the status line at the bottom
keeps showing _not causal_ or _outputs incomplete_ while you are on this page.
Both have to hold before anything could run.

_Not feasible_ reports the **first dead end** the placement hit under its own
order — one conflict, honestly named, not necessarily the only one.

## What "feasible" leaves out

Electrical and numeric constraints: current, voltage, PWM frequency, timer
resolution, bus speed. _Feasible_ means the pins can be allocated, not that the
circuit works. A later layer may narrow _feasible_; it will never widen it.

## What is not there

Generating and building firmware from the Deploy page, flashing a board, and
reading values back from it. The compiler can already generate a Rust core for a
design and checks it trace for trace against the simulator in the repository's
tests, but that path has no Studio surface yet; the roadmap in
`docs/project/roadmap.md` lists it.

## Related

[Your first deployment check](../getting-started/first-deployment.md) ·
[Physical outputs](../concepts/physical-outputs.md) ·
[Troubleshooting: deployment](../troubleshooting/deployment-errors.md)
