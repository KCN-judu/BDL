# Deploy

The Deploy page (⌘3) answers *does this design fit this board*. It is a
placement check: for each physical output's device, which board resources
it would use — or the first reason it cannot be placed. It does not build
firmware or flash a board yet; see [What is not there](#what-is-not-there).

```
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

The boards the compiler service knows. Today: **Arduino Nano** and **Big
board (mock)**, a test target with more PWM pins. The board choice is a
**session preference**: it is not saved with the project, and changing it
never changes the design.

Until a board is chosen the page says *Choose a board to see whether this
design fits it.*

## Devices (left)

A **device** realises one physical output on the board. **Add device**
creates a row; in it you set

* a **name**;
* a **kind** — *PWM channel* (a dimmable light, a servo signal), *Digital
  output* (a relay, a switched load), *H-bridge channel* (a motor: one
  PWM line plus one direction line), *I²C sensor*, *Quadrature encoder*,
  *UART*;
* the **output** it realises (or none, for a sensor that feeds the
  design);
* one **pin field per requirement** of the kind — leave empty to let the
  placement choose, or type a board pin name (`D3`, `A4`) to fix it by
  hand;
* **Remove**.

The kind decides what the device needs from the board (a PWM channel
needs one PWM-capable pin; an H-bridge needs a PWM pin and a digital pin;
an I²C sensor needs SDA and SCL on the same bus). You never edit those
requirements; the pin fields are the only manual choice.

Devices are saved with the project. They are deployment data, not design
data: adding, changing or removing one leaves every Design-page verdict
unchanged.

## The verdict (centre)

| Verdict | Meaning |
|---|---|
| **Feasible on Arduino Nano.** | every device's requirements were placed on distinct pins with the right capabilities; the *Placement* table lists device → requirement → pin, with what the pin can do |
| **Fits Arduino Nano so far — the binding is not finished.** | what is bound fits, but an output has no device (*No device on … for: …*) or a device no output (*Not connected to an output: …*) |
| **Not feasible on Arduino Nano.** | one requirement could not be placed; the page names it — *Could not place PWM light PWM of PWM light.* — and why: *Nothing on … can carry …* (no pin has the capability), *The pin chosen by hand, D4, cannot carry … here.*, or the pins that block it, each with what holds it; *Placed before the dead end:* lists what was placed |

The verdict is only about the board. Whether the **design** itself is
ready — every relationship checks, no instantaneous cycle, every required
output driven — is the Design page's business and is not restated here:
a design whose formulas are wrong can still be *feasible*, and the status
line at the bottom keeps showing *not causal* or *outputs incomplete*
while you are on this page. Both have to hold before anything could run.

*Not feasible* reports the **first dead end** the placement hit under its
own order — one conflict, honestly named, not necessarily the only one.

## What "feasible" leaves out

Electrical and numeric constraints: current, voltage, PWM frequency,
timer resolution, bus speed. *Feasible* means the pins can be allocated,
not that the circuit works. A later layer may narrow *feasible*; it will
never widen it.

## What is not there

Generating and building firmware from the Deploy page, flashing a board,
and reading values back from it. The compiler can already generate a
Rust core for a design and checks it trace for trace against the
simulator in the repository's tests, but that path has no Studio surface yet; the roadmap
in `docs/project/roadmap.md` lists it.

## Related

[Your first deployment check](../getting-started/first-deployment.md) ·
[Physical outputs](../concepts/physical-outputs.md) ·
[Troubleshooting: deployment](../troubleshooting/deployment-errors.md)
