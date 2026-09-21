# Incomplete design

None of the situations on this page is an error. Each is a piece of the design
that is not decided yet; the tool tells you which, and works with everything
else. The idea behind this is in
[Incomplete designs](../concepts/incomplete-designs.md).

## The inspector says _declared_; the status line says _N not yet defined_

**What it is.** A rule — a relationship that reads something (or backs a port of
the component you are editing) — with no formula yet. It is in the sidebar's
_Mappings_ list, not on the canvas. A block that has no formula is a Source —
drawn with a bar at its left edge and counted as _N sources_, not as not yet
defined: nothing is missing from it.

**What to do.** Select the rule in the sidebar and write the formula in
_Relationship_. If it was meant to be an input from outside, remove its reads
instead: it becomes a block, a Source.

## _tilt needs a value before simulation can step._ (Simulate)

**What it is.** An input has no value for the run.

**What to do.** Type one in its control on the left. Step is enabled once every
input has a value.

## _Tilt needs a value form (Quantity, On / off or Count) before tilt can be given a value._ (Simulate)

**What it is.** The concept an input produces is still _decide later_, so there
is no control that could take a value.

**What to do.** Select the concept (the _Show_ link) and choose its _Value_. Or
use the fix _Choose what Tilt is represented by_.

## _Checked once Temperature's value is decided._ / _Temperature has no representation yet, so this formula cannot be checked._

**What it is.** The formula reads a concept whose value form is not chosen. The
relationship is _open_: not wrong, waiting. Its node is solid with a hollow
socket.

**What to do.** Decide the concept's value form. The formula is checked at once;
nothing needs retyping. Code: `concept.unbound_representation` (information).

## _dimByTilt has no definition. A relationship that reads something needs one before the design can run._ (Simulate)

**What it is.** A rule without a formula is applied by something the simulator
needs.

**What to do.** Write the formula. (A _value_ without a formula is an input and
is fine; a _rule_ without one has nothing to compute.)

## _level has no valid definition._ (Simulate)

**What it is.** A relationship the run needs has a formula that does not check.
This one _is_ an error — see
[Types, units and concepts](type-and-concept-errors.md).

## _light has no final target yet._ / _Undriven — the design is incomplete without a driver._ / status line _outputs incomplete_

**What it is.** A required output with no driver. The design is incomplete, not
wrong.

**What to do.** Connect the value that should drive it: drag from its output
socket to the sink, pick it in the output's _Connect_ pop-up, or take the fix
_Connect a driver to light_. If the output is not essential, switch _Required_
off. Code: `output.missing_driver` (information).

## _light has no timing domain yet, so this connection cannot be checked._

**What it is.** The output is _open_ — it has no domain, so nothing can be
checked against it.

**What to do.** Set _Updates in_ on the output. Code: `output.clock_unset`
(information).

## _Open: nothing supplies it yet._ (an instance's port)

**What it is.** A required port with no binding. The instance works with what it
has; the simulator asks for the port's value as an input.

**What to do.** Draw a link from a provided port or a top-level value to it — or
leave it, if the value really comes from outside.

## _Fits Arduino Nano so far — the binding is not finished._ (Deploy)

**What it is.** Every device that is bound fits, but an output has no device or
a device no output.

**What to do.** _Add device_ and choose the output; or choose an output in the
device row. See [Deployment](deployment-errors.md).

## Related

[Incomplete designs](../concepts/incomplete-designs.md) ·
[Status meanings](../reference/status-meanings.md)
