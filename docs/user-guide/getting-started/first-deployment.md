# Your first deployment check

**Goal.** Find out whether the lamp from the [first tutorial](first-behavior.md)
fits an Arduino Nano — and see what "does not fit" looks like.

**Time.** Ten minutes.

Deployment in Behavior Designer today means _checking placement_: which board
pin each physical output's device would use, or why none can. It does not build
firmware or flash a board yet.

## 1. Open the Deploy page

Click **Deploy** in the page bar (or **⌘3**). On the left, a **Target** pop-up
and a **Devices** list; in the middle, the verdict for the chosen board.

Until you choose a board the page says _Choose a board to see whether this
design fits it._

## 2. Choose a board

In **Target**, choose **Arduino Nano**. The verdict reads:

> Fits Arduino Nano so far — the binding is not finished.

and under it: _No device on arduino_nano for: Light Output._ (this line names
the board by its short id). The design has an output but nothing says what piece
of hardware realises it.

## 3. Add a device

A **device** is the hardware that realises one output on the board: a PWM
channel for a dimmable light, a digital output for a relay, an H-bridge for a
motor.

1. Click **Add device**. A row appears in _Devices_.
2. Name it `PWM light`. In its kind pop-up choose **PWM channel**. In its output
   pop-up choose **Light Output**.

The verdict changes to **Feasible on Arduino Nano.** and a _Placement on
arduino_nano_ table shows the one line the lamp needs: `PWM light`, its _PWM_
requirement, and the pin it was given, such as `D3`.

**What you made.** A deployment configuration for one board. It lives in the
project with the design, but it is a separate layer: the Design page's verdicts
did not change when you added the device, and they will not change if you pick
another board.

![The Deploy page as one column: the Target pop-up showing Arduino Nano with 22 resources; the green verdict Feasible on Arduino Nano; the Devices section with a card named PWM light of kind PWM channel realising Light Output, its PWM requirement and an empty pin field; then Placement on arduino_nano with one row: PWM light, PWM light PWM, arrow D3.](../assets/studio/deploy-page.png)

_The Deploy page: Arduino Nano chosen, one PWM device on Light Output, the
verdict and the placement._

## 4. Make it fail on purpose

Each device row has one pin field per requirement. In `PWM light`'s pin field
type `D4` — a pin that cannot do PWM on the Nano.

> Not feasible on Arduino Nano.
>
> D4 cannot carry PWM light PWM on arduino_nano. The pin chosen by hand, D4,
> cannot carry PWM light PWM here.

Clear the field: feasible again. Now switch **Target** to **Big board (mock)**:
feasible there too, on a different pin. Switch back.

**What happened.** Feasibility is a fact about _this design on this board_. The
board choice is a session preference — it is not saved with the project — and
the placement is recomputed for whatever board is chosen.

![The verdict Not feasible on Arduino Nano in red, the device card with D4 typed into its pin field, and below it a red-bordered box headed D4 cannot carry PWM light PWM on arduino_nano, explaining that the pin chosen by hand cannot carry the requirement here.](../assets/getting-started/deploy-dead-end.png)

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
- **Not connected to an output: …** — a device with no output chosen.
- **Nothing on Arduino Nano can carry …** — the board has no free pin with that
  capability; try another kind of device, another board, or fewer devices.

## Next

You have built, simulated and placed a behavior. Read the
[Concepts](../concepts/concepts.md) pages for the ideas you used, or go straight
to [From sensor to output](../workflows/sensor-to-output.md) to extend the lamp
with an ambient-light sensor.
