# Your first deployment check

**Goal.** Find out whether the lamp from the [first tutorial](first-behavior.md)
fits an Arduino Nano — and see what "does not fit" looks like.

**Time.** Ten minutes.

This page is the _placement check_: which board pin each physical output's
device would use, or why none can. Building the firmware and putting it on a
board is the page after — [Your first board](pico-demo.md) — and needs a board
this build has firmware for; the Nano here is a placement exercise.

## 1. Open the Deploy page

Click **Deploy** in the page bar (or **⌘3**). On the left, a **Target** pop-up
and a **Devices** list; in the middle, the verdict for the chosen board.

Until you choose a board the page says _Choose a board to see whether this
design fits it._

## 2. Choose a board

In **Target**, choose **Arduino Nano**. The verdict reads:

> Fits Arduino Nano so far — the binding is not finished.

and under it: _No device on arduino_nano for: light._ (this line names the board
by its short id). The design has an output but nothing says what piece of
hardware realises it.

## 3. Add a device

A **device** is the hardware that realises one output on the board: a PWM
channel for a dimmable light, a digital output for a relay, an H-bridge for a
motor.

1. Click **Add device**. A row appears in _Devices_.
2. Name it `pwmLight`. In its kind pop-up choose **PWM channel**. In its output
   pop-up choose **light**.

The line about `light` disappears and a _Placement on arduino_nano_ table shows
the one line the lamp needs: `pwmLight`, its _PWM_ requirement, and the pin it
was given, such as `D3`. The verdict stays **Fits Arduino Nano so far — the
binding is not finished.**, and one line remains under it: _tilt has no device
on Arduino Nano._ The lamp's output is placed; its Source is not.

**What you made.** A deployment configuration for one board. It lives in the
project with the design, but it is a separate layer: the Design page's verdicts
did not change when you added the device, and they will not change if you pick
another board.

**About `tilt`.** A [Source](../concepts/relationships.md) is a value the
environment provides, and on a board a device has to provide it — the same kind
of binding as for an output, chosen in the device's pop-up (_tilt — Source_)
with a **Provider** instead of a Realization. The providers this build has read
a digital line as on/off; `tilt` is an angle, so nothing fits it yet, and the
deployment honestly stays _not finished_. That is a state, not an error: the
design simulates, and the placement of `light` is real.

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the orange verdict Fits Arduino Nano so far — the binding is not finished; the Devices section with a card named pwmLight of kind PWM channel for light, its Realization pop-up at None — place by kind with the note that no raw command is generated until a profile is chosen, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: pwmLight, pwmLight PWM, arrow D3; and the information line tilt has no device on Arduino Nano.](../assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on light, the placement,
and the Source still to provide._

## 4. Make it fail on purpose

Each device row has one pin field per requirement. In `pwmLight`'s pin field
type `D4` — a pin that cannot do PWM on the Nano.

> Not feasible on Arduino Nano.
>
> D4 cannot carry pwmLight PWM on arduino_nano. The pin chosen by hand, D4,
> cannot carry pwmLight PWM here.

Clear the field: it fits again. Now switch **Target** to **Big board (mock)**:
it fits there too, on a different pin. Switch back.

**What happened.** Feasibility is a fact about _this design on this board_. The
board choice is a session preference — it is not saved with the project — and
the placement is recomputed for whatever board is chosen.

![The verdict Not feasible on Arduino Nano in red, the device card with D4 typed into its pin field, and below it a red-bordered box headed D4 cannot carry pwmLight PWM on arduino_nano, explaining that the pin chosen by hand cannot carry the requirement here.](../assets/getting-started/deploy-dead-end.png)

_Not feasible: the pin chosen by hand cannot carry PWM._

## What "feasible" means, and does not

_Feasible_ means every device's requirements can be placed on distinct pins with
the right capabilities. It does not mean the circuit will work electrically:
current, voltage and timing resolution are not modelled. When several placements
are possible the tool shows one; when none is, it shows the first dead end it
hit, which is _a_ conflict, not necessarily the only one.

## If something does not work

- **The status line still says _not causal_ or _outputs incomplete_.** The
  Deploy verdict is only about the board; the design itself is not ready. Fix it
  on the Design page first. See
  [Deployment](../troubleshooting/deployment-errors.md).
- **Not connected to an output or Source: …** — a device with nothing chosen in
  its pop-up.
- **… has no device on Arduino Nano.** — a Source no device provides; the
  deployment stays _not finished_ until one does.
- **Nothing on Arduino Nano can carry …** — the board has no free pin with that
  capability; try another kind of device, another board, or fewer devices.

## Next

You have built, simulated and placed a behavior. With a Raspberry Pi Pico at
hand, [Your first board](pico-demo.md) builds a design and runs it. Or read the
[Concepts](../concepts/concepts.md) pages for the ideas you used, or go straight
to [From sensor to output](../workflows/sensor-to-output.md) to extend the lamp
with an ambient-light sensor.
