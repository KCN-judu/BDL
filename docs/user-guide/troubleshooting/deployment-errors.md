# Deployment

Verdicts on the Deploy page are about **this design on this board**. They never
change the design, and choosing another board never changes a Design-page
verdict. The rules are in [Deploy](../studio/deploy.md).

## _Fits Arduino Nano so far — the binding is not finished._ with _No device on arduino_nano for: Light Output._

**What it means.** An output has a domain but no device realises it on the
board.

**What to do.** _Add device_, choose a kind that can carry the output (a PWM
channel for a dimmable light, a digital output for a switched load) and choose
the output in the row. Code: `deploy.output_unrealised` (information).

## _… so far …_ with _Not connected to an output: PWM light._

**What it means.** A device has no output.

**What to do.** Choose one in the device row, or _Remove_ the device. A sensor
that feeds the design rather than realising an output may stay unconnected.
Code: `deploy.device_unbound` (information).

## _Not feasible on Arduino Nano._ — _Could not place PWM light PWM of PWM light._ — _Nothing on arduino_nano can carry PWM light PWM._

**What it means.** No pin on the board offers what one requirement needs (PWM,
an interrupt, an I²C bus…), or every pin that does is already taken by another
device — the page lists the blocking pins and who holds each.

**Why.** Placement is exact: each requirement needs a distinct resource with the
right capability, on the same unit where the kind demands it (I²C SDA and SCL on
one bus; a UART's TX and RX on one unit).

**What to do.** Fewer devices of that kind, a kind with lighter needs, a pin
fixed by hand to free a contested one, or another board. The Nano has six PWM
pins; _Big board (mock)_ has twelve. Code: `deploy.infeasible`.

## _Not feasible …_ — _The pin chosen by hand, D4, cannot carry PWM light PWM here._

**What it means.** The pin you typed in the device row is not on this board or
lacks the capability.

**What to do.** Clear the field to let the placement choose, or type a pin that
has the capability (the placement table shows what each placed pin can do).
Code: `deploy.infeasible`.

## The verdict is feasible but the status line still says _not causal_ or _outputs incomplete_

**What it means.** The board fits the devices, but the design itself is not
ready — a formula does not check, a cycle exists, or a required output is
undriven. The Deploy page does not restate design problems.

**What to do.** Fix them on the Design page; the Deploy verdict is unaffected
either way.

## _Could not analyse: …_

**What it means.** The request itself failed — the compiler service is not
connected, or the project moved on while the analysis ran. Choose the board
again, or check the status line's connection.

## The dead end names a conflict that is not the real one

**What it means.** _Not feasible_ reports the **first** dead end the placement
hit, under its own order. It is one true conflict, not necessarily the one you
would have named.

**What to do.** Resolve it and check again; if a second one appears, it was
there too.

## Related

[Deploy](../studio/deploy.md) ·
[Physical outputs](../concepts/physical-outputs.md)
