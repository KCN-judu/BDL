# Deploy

The Deploy page (⌘3) answers _does this design fit this board_. It is a
placement check: for each physical output's device, which board resources it
would use — or the first reason it cannot be placed. The page is one column: the
**Target** pop-up, the **verdict**, the **Devices** and, once a board has
answered, the **Placement**. It does not build firmware or flash a board yet;
see [What is not there](#what-is-not-there).

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the orange verdict Fits Arduino Nano so far — the binding is not finished; the Devices section with a card named pwmLight of kind PWM channel for light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3; and the information line tilt has no device on Arduino Nano.](../assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on light, the placement,
and the Source still to provide._

## Target

The boards the compiler service knows. Today: **Arduino Nano**, **Big board
(mock)**, a test target with more PWM pins, and **Raspberry Pi Pico (RP2040)**.
Firmware can be generated for the Pico and the Nano
(`bdld compile --target rp2040_pico` / `--target arduino_nano`,
[CLI](../reference/cli.md); flashing is not in Studio yet). The board choice is
a **session preference**: it is not saved with the project, and changing it
never changes the design.

Until a board is chosen the page says _Choose a board to see whether this design
fits it._

## Devices

A **device** realises one physical output on the board, or provides one
[Source](../concepts/relationships.md) to it — one or the other, never both.
**Add device** creates a row; in it you set

- a **name**;
- a **kind** — _PWM channel_ (a dimmable light, a servo signal), _Digital
  output_ (a relay, a switched load), _Digital input_ (a button, a switch),
  _H-bridge channel_ (a motor: one PWM line plus one direction line), _I²C
  sensor_, _Quadrature encoder_, _UART_;
- what it is **for**: an output it realises, or a Source it provides — the
  pop-up lists the outputs by name and the Sources as _tilt — Source_ (or _not
  connected_);
- one **pin field per requirement** of the kind — leave empty to let the
  placement choose, or type a board pin name (`D3`, `A4`) to fix it by hand;
- **Remove**.

The kind decides what the device needs from the board (a PWM channel needs one
PWM-capable pin; an H-bridge needs a PWM pin and a digital pin; an I²C sensor
needs SDA and SCL on the same bus). You never edit those requirements; the pin
fields and the realization are the only manual choices.

### Realization

Once a board has answered, each device card shows a **Realization** pop-up: how
the output's value becomes the command the device takes. The choices are the
compiler's profiles — _PWM, 8-bit duty_ (a level 0–100 becomes a duty 0–255),
_PWM, 4 levels_ (four duties; nearby levels share one), _I2C register, 8-bit_
(register 42 and a value 0–255), _GPIO, on/off_ (a truth value as written),
_H-bridge, signed level_ (direction and duty) — with the ones that fit what the
output carries listed first and the others marked _does not fit_. Choosing one
also sets the device's kind to what the profile needs. _None — place by kind_
leaves the device placed as before and generates no command.

Beside the pop-up the card shows the three checks a realization must pass —
**encoder** (the profile's conversion is a well-typed pure function), **fits**
(it converts exactly what this output carries), **placed** (the board has the
pins) — and, when one fails, the sentence that says which: _pwmLight cannot
realise light with `gpio_level`: the output carries q[1] but the profile encodes
bool._ A failing realization blocks deployment until changed; a missing one does
not.

A realization is deployment data like the kind and the pins: choosing, changing
or removing one changes nothing on the Design page, in the simulator's samples
or in any verdict about the design.

Devices are saved with the project. They are deployment data, not design data:
adding, changing or removing one leaves every Design-page verdict unchanged.

### Provider

A device that is for a Source shows a **Provider** pop-up instead: how the
device's reading becomes the value the Source carries. The choices are the
catalogue's input profiles, the ones that fit the Source first, the others
marked _— does not fit_, and _None — place by kind_. Today there are two, both
reading a digital line as on/off: _GPIO input, active high_ (the line high is
_on_) and _GPIO input, active low_ (the line low is _on_; a button to ground).
Choosing one also sets the device's kind to what the profile needs.

Beside the pop-up: the raw reading type (_raw reading bool_), and four checks —
**transducer** (the profile's conversion is a well-typed pure function),
**fits** (it produces exactly what this Source carries), **placed** (the board
has the line), **readable** (this board's firmware can read the profile) — with
the sentence that says which fails: _sensor cannot provide tilt with
`gpio_level_in`: the Source carries q[1] but the profile reads bool._ A failing
provider blocks deployment until changed; a missing one does not. _Readable_
failing is different: the design and the placement are fine, but this board's
firmware has no reader for the profile yet (the Arduino Nano reads none; the
Raspberry Pi Pico reads both) — the placement stands and the firmware is refused
by name.

A Source no device provides keeps the verdict at _so far_ with the line _tilt
has no device on Arduino Nano._ — a state, like an output without a device.
Every project written before providers existed reads this way until a device is
bound; nothing in the design changes when one is.

The inspector's _Realization_ row for a Source shows the same fact for the board
chosen here: _Provided by the environment; no device on Raspberry Pi Pico yet._,
_Provided by sensor on Raspberry Pi Pico._, or _Provided by sensor as GPIO
input, active low on Raspberry Pi Pico._; with no board chosen it says _Provided
by the environment; no device is bound yet._

## The verdict

| Verdict                                                     | Meaning                                                                                                                                                                                                                                                                                                                                                                                       |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Feasible on Arduino Nano.**                               | every device's requirements were placed on distinct pins with the right capabilities; the _Placement on …_ table lists device · requirement · → pin                                                                                                                                                                                                                                           |
| **Fits Arduino Nano so far — the binding is not finished.** | what is bound fits, but an output has no device (_No device on … for: …_), a Source has no device (_… has no device on Arduino Nano._) or a device nothing (_Not connected to an output or Source: …_)                                                                                                                                                                                        |
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
