# Deploy

The Deploy page (⌘3) answers _does this design fit this board_ — and then puts
it on the board. It is a placement check: for each physical output's device,
which board resources it would use — or the first reason it cannot be placed.
The page is one column: the **Target** pop-up, the **verdict**, the **Devices**,
once a board has answered the **Placement**, and last the **Firmware** — one
action at a time, from _Build_ to _Flash_ to trying the product
([Firmware](#firmware)).

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the orange verdict Fits Arduino Nano so far — the binding is not finished; the Devices section with a card named pwmLight of kind PWM channel for light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3; and the information line tilt has no device on Arduino Nano.](../assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on light, the placement,
and the Source still to provide._

## Target

The boards the compiler service knows. Today: **Arduino Nano**, **Big board
(mock)**, a test target with more PWM pins, and **Raspberry Pi Pico (RP2040)**.
Firmware can be built for the Pico and the Nano; the Pico is the one Studio
flashes ([Firmware](#firmware); the Nano's firmware is built with
`bdld build --target arduino_nano` and loaded with `avrdude` by hand,
[CLI](../reference/cli.md)). The board choice is a **session preference**: it is
not saved with the project, and changing it never changes the design.

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

## Firmware

The last section is where the board gets the design. Its header shows where you
are — **Deployment · Build · Flash · Observe**, the steps done with a check, the
current one in bold — and the card below has one primary action for that step,
or the one thing that stands in its way. Every word on it is the compiler
service's: Studio asks whether a build would pass, runs nothing itself, and
watches the service's progress.

![The Deploy page: the Target pop-up showing Raspberry Pi Pico (RP2040); the green verdict Feasible on Raspberry Pi Pico (RP2040); two device cards — button, a Digital input for pressed — Source with the provider GPIO input, active low and pin GP2, and led, a Digital output for lamp with the realization GPIO, on/off and pin GP25 — each with its judgments checked; the placement table with button and led on GP2 and GP25; and at the bottom the Firmware section with the steps Deployment done, Build current, Flash and Observe to come, a green dot with Ready to build for Raspberry Pi Pico (RP2040), and one primary button, Build for Raspberry Pi Pico (RP2040).](../assets/getting-started/pico-firmware-ready.png)

_The wired demo on the Deploy page: the Pico chosen, both devices admissible and
placed, and the one thing left to do — Build._

| The card says                                                                                                                         | Meaning, and what to do                                                                                                                                                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Not ready to build** — _pressed has no device on Raspberry Pi Pico._                                                                | one thing stands in the way, the smallest first; its second line says what to do. A blocker about the design itself (_lit is not fully defined_) offers **Fix it on the Design page**; one about a device is on this page, above. The list behind it is the service's, in its order: the design, then the devices, then what this board's firmware cannot do (_… which Arduino Nano cannot read yet_) |
| **Ready to build for Raspberry Pi Pico (RP2040).** — **Build for …**                                                                  | everything the firmware needs holds. The button starts the build                                                                                                                                                                                                                                                                                                                                      |
| **Building…** _Compiling · 84 crates compiled_ — **Stop**                                                                             | the stages as they happen: _Checking the deployment_, _Generating the crate_, _Preparing the toolchain_, _Compiling_ (with the count), _Writing the image_. The first build of a board fetches its libraries — minutes; later builds take seconds. Stop ends it; nothing reaches the board                                                                                                            |
| **The build did not complete** — _The Rust target for … is not installed._                                                            | the stage that failed, in red, with the one thing to do (_Run `rustup target add thumbv6m-none-eabi` once, then build again._); **Build again**; **Details** opens with the compiler's words                                                                                                                                                                                                          |
| **Firmware built at 23:33** _6144 bytes_ — **Build again**                                                                            | the image is current: it was made from the design and deployment on screen. Under it, the board                                                                                                                                                                                                                                                                                                       |
| **No board is reachable.** _Hold BOOTSEL while plugging the board in over USB; it appears as a drive named RPI-RP2._ — **Look again** | nothing to write to; the line says how to make the Pico reachable (its own bootloader, no tool, no probe). Look again after plugging it in                                                                                                                                                                                                                                                            |
| _Raspberry Pi Pico in BOOTSEL mode (RPI-RP2) — /Volumes/RPI-RP2_ — **Flash**                                                          | one board: Flash writes the image to it                                                                                                                                                                                                                                                                                                                                                               |
| **Several boards are reachable — choose one.** with a pop-up                                                                          | two Picos in bootloader mode (or a debug probe beside one): Studio never picks. Choose, then Flash                                                                                                                                                                                                                                                                                                    |
| **Flashing…** _Writing the image · Restarting the board_                                                                              | the image is copied; the Pico restarts into it and its drive disappears                                                                                                                                                                                                                                                                                                                               |
| **Flashed to … at 23:35** — _Try it: act on pressed; lamp should follow the design._                                                  | done, with the trial to make, in the design's names. That the flash succeeded is all Studio knows; the behavior is yours to see                                                                                                                                                                                                                                                                       |
| **The flash did not complete** — _The board is no longer in bootloader mode._                                                         | the reason and the remedy; the image is still current, Flash again when the board is back                                                                                                                                                                                                                                                                                                             |
| **The firmware is from an earlier design or deployment.** — **Build again**                                                           | the design, a device, a pin or a profile changed since the image was built (a moved node does not count): the image is stale, and only Build again is offered — never Flash. If it had been flashed: _The board runs an earlier design — build and flash again to update it._                                                                                                                         |

**Details** (closed unless a build failed) holds the developer's facts: the
generated crate's folder (`build/rp2040_pico/` inside the project), the exact
`cargo` command, the Rust target, the image's path, and the compiler's output.
Nothing in the normal flow needs them.

**What "current" means.** The service keeps, beside the image, a fingerprint of
exactly what it was built from — the generated code, the board, the settings,
the compiler version. After every change it compares; the card says _stale_ the
moment they differ. An image is never flashed while stale, and a board is never
said to run the design on screen when it does not.

## What "feasible" leaves out

Electrical and numeric constraints: current, voltage, PWM frequency, timer
resolution, bus speed. _Feasible_ means the pins can be allocated, not that the
circuit works. A later layer may narrow _feasible_; it will never widen it.

## What is not there

Reading values back from the board (the Monitor page is a placeholder); flashing
the Arduino Nano from Studio (its firmware builds; `avrdude` loads it); a debug
probe as the first-choice path (probe-rs is used when it is installed and a
probe is wired to the Pico's SWD pins, and listed beside the USB bootloader);
the board's tick and the domains' periods chosen here (`bdld compile --period`
only).

## Related

[Your first board](../getting-started/pico-demo.md) ·
[Your first deployment check](../getting-started/first-deployment.md) ·
[Physical outputs](../concepts/physical-outputs.md) ·
[Troubleshooting: deployment](../troubleshooting/deployment-errors.md)
