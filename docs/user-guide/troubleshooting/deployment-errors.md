# Deployment

Verdicts on the Deploy page are about **this design on this board**. They never
change the design, and choosing another board never changes a Design-page
verdict. The rules are in [Deploy](../studio/deploy.md).

## _Fits Arduino Nano so far — the binding is not finished._ with _No device on arduino_nano for: light._

**What it means.** An output has a domain but no device realises it on the
board.

**What to do.** _Add device_, choose a kind that can carry the output (a PWM
channel for a dimmable light, a digital output for a switched load) and choose
the output in the row. Code: `deploy.output_unrealised` (information).

## _… so far …_ with _Not connected to an output or Source: pwmLight._

**What it means.** A device is for nothing.

**What to do.** Choose an output or a Source in the device row's pop-up, or
_Remove_ the device. Code: `deploy.device_unbound` (information).

## _… so far …_ with _tilt has no device on Arduino Nano._

**What it means.** A Source — a value the environment provides — has no device
providing it on this board. The design simulates as before; it cannot run on the
board until one does.

**What to do.** _Add device_, choose _Digital input_ if the Source is on/off,
choose the Source (_tilt — Source_) in the pop-up and a **Provider**. A Source
that carries something no provider reads yet (an angle, a temperature) stays
here; the deployment is _not finished_, not wrong. Code:
`deploy.source_unprovided` (information).

## _sensor has no provider chosen for tilt._

**What it means.** The device is for the Source but no provider profile is
chosen: it places on the board by its kind and reads nothing.

**What to do.** Choose a profile in the **Provider** pop-up. Code:
`deploy.provider_unspecified` (information).

## _sensor cannot provide tilt with `gpio_level_in`: the Source carries q[1] but the profile reads bool._

**What it means.** The chosen provider produces a different value form than the
Source carries. This blocks deployment.

**What to do.** Choose a provider marked as fitting, or change the Source's
concept on the Design page. Code: `deploy.provider_incompatible`.

## _sensor refers to a provider profile this version does not know._

**What it means.** The project names a provider id this build's catalogue does
not have. The design is unchanged.

**What to do.** Choose an available provider. Code:
`deploy.provider_unknown_profile`. The same for a realization:
`deploy.realization_unknown_profile`.

## _sensor provides tilt as `gpio_level_in`, which Arduino Nano cannot read yet._

**What it means.** The provider fits and the line is placed, but this board's
firmware has no reader for the profile (only the Raspberry Pi Pico reads the
GPIO input profiles today). The verdict is unaffected; firmware for this board
is refused.

**What to do.** Choose a board that reads it, or wait. Code:
`deploy.provider_unsupported` (warning); the firmware's refusal is
`adapter.provider_unsupported`.

## _sensor and sensor2 both provide tilt._

**What it means.** Two devices are for the same Source; a Source is provided by
exactly one.

**What to do.** Change one device's pop-up. Code: `deploy.source_contested`.

## _Not feasible on Arduino Nano._ — _Could not place pwmLight PWM of pwmLight._ — _Nothing on arduino_nano can carry pwmLight PWM._

**What it means.** No pin on the board offers what one requirement needs (PWM,
an interrupt, an I²C bus…), or every pin that does is already taken by another
device — the page lists the blocking pins and who holds each.

**Why.** Placement is exact: each requirement needs a distinct resource with the
right capability, on the same unit where the kind demands it (I²C SDA and SCL on
one bus; a UART's TX and RX on one unit).

**What to do.** Fewer devices of that kind, a kind with lighter needs, a pin
fixed by hand to free a contested one, or another board. The Nano has six PWM
pins; _Big board (mock)_ has twelve. Code: `deploy.infeasible`.

## _Not feasible …_ — _The pin chosen by hand, D4, cannot carry pwmLight PWM here._

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
