# Deploy

The Deploy page (⌘3) answers _does this design fit this board_. It is a
placement check: for each physical output's device, which board resources it
would use — or the first reason it cannot be placed. The page is one column: the
**Target** pop-up, the **verdict**, the **Devices** and, once a board has
answered, the **Placement**. It does not build firmware or flash a board yet;
see [What is not there](#what-is-not-there).

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the green verdict Feasible on Arduino Nano; the Devices section with a card named pwmLight of kind PWM channel realising light, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3.](../assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on light, the verdict and
the placement._

## Target

The boards the compiler service knows. Today: **Arduino Nano** and **Big board
(mock)**, a test target with more PWM pins. The board choice is a **session
preference**: it is not saved with the project, and changing it never changes
the design.

Until a board is chosen the page says _Choose a board to see whether this design
fits it._

## Devices

A **device** realises one physical output on the board. **Add device** creates a
row; in it you set

- a **name**;
- a **kind** — _PWM channel_ (a dimmable light, a servo signal), _Digital
  output_ (a relay, a switched load), _H-bridge channel_ (a motor: one PWM line
  plus one direction line), _I²C sensor_, _Quadrature encoder_, _UART_;
- the **output** it realises (or none);
- one **pin field per requirement** of the kind — leave empty to let the
  placement choose, or type a board pin name (`D3`, `A4`) to fix it by hand;
- **Remove**.

The kind decides what the device needs from the board (a PWM channel needs one
PWM-capable pin; an H-bridge needs a PWM pin and a digital pin; an I²C sensor
needs SDA and SCL on the same bus). You never edit those requirements; the pin
fields are the only manual choice.

Devices are saved with the project. They are deployment data, not design data:
adding, changing or removing one leaves every Design-page verdict unchanged.

A [Source](canvas.md) — a value the environment provides — has no device binding
yet: no kind on this page provides a value to the design, and the inspector's
_Realization_ row says so (_Provided by the environment; no device is bound
yet._). Binding a sensor, a button or an analog line to a Source is deployment
work that will arrive with the first embedded platform; the design does not
change when it does.

## The verdict

| Verdict                                                     | Meaning                                                                                                                                                                                                                                                                                                                                                                                       |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Feasible on Arduino Nano.**                               | every device's requirements were placed on distinct pins with the right capabilities; the _Placement on …_ table lists device · requirement · → pin                                                                                                                                                                                                                                           |
| **Fits Arduino Nano so far — the binding is not finished.** | what is bound fits, but an output has no device (_No device on … for: …_) or a device no output (_Not connected to an output: …_)                                                                                                                                                                                                                                                             |
| **Not feasible on Arduino Nano.**                           | one requirement could not be placed; a red box names it and why — _arduino_nano has nothing that can carry pwmLight PWM._ (no pin has the capability), _D4 cannot carry pwmLight PWM on arduino_nano._ with _The pin chosen by hand, D4, cannot carry … here._ (a pin fixed by hand), or the pins that block it, each with what holds it; _Placed before the dead end:_ lists what was placed |

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
