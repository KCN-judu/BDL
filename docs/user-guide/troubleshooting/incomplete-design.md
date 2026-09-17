# Incomplete design

None of the situations on this page is an error. Each is a piece of the
design that is not decided yet; the tool tells you which, and works with
everything else. The idea behind this is in
[Incomplete designs](../concepts/incomplete-designs.md).

## A dashed node with the word *declared*; the status line says *N not yet defined*

**What it is.** A relationship with a signature and no formula.

**What to do.** If it is an input (reads nothing, supplied from outside):
nothing — give it a timing domain and it is finished. Otherwise select it
and write the formula in *Relationship*.

## *tilt needs a value before simulation can step.* (Simulate)

**What it is.** An input has no value for the run.

**What to do.** Type one in its control on the left. Step is enabled once
every input has a value.

## *Tilt needs a value form (Quantity, On / off or Count) before tilt can be given a value.* (Simulate)

**What it is.** The concept an input produces is still *decide later*, so
there is no control that could take a value.

**What to do.** Select the concept (the *Show* link) and choose its
*Value*. Or use the fix *Choose what Tilt is represented by*.

## *Checked once Temperature's value is decided.* / *Temperature has no representation yet, so this formula cannot be checked.*

**What it is.** The formula reads a concept whose value form is not
chosen. The relationship is *open*: not wrong, waiting. Its node is solid
with a hollow socket.

**What to do.** Decide the concept's value form. The formula is checked
at once; nothing needs retyping. Code: `semantic.unbound_representation`
(information).

## *dimByTilt has no definition. A relationship that reads something needs one before the design can run.* (Simulate)

**What it is.** A rule without a formula is applied by something the
simulator needs.

**What to do.** Write the formula. (A *value* without a formula is an
input and is fine; a *rule* without one has nothing to compute.)

## *level has no valid definition.* (Simulate)

**What it is.** A relationship the run needs has a formula that does not
check. This one *is* an error — see [Types, units and concepts](type-and-concept-errors.md).

## *Light Output has no final target yet.* / *Undriven — the design is incomplete without a driver.* / status line *outputs incomplete*

**What it is.** A required output with no driver. The design is
incomplete, not wrong.

**What to do.** Connect the value that should drive it: drag from its
output socket to the sink, pick it in the output's *Connect* pop-up, or
take the fix *Connect a driver to Light Output*. If the output is not
essential, switch *Required* off. Code: `output.missing_driver`
(information).

## *Light Output has no timing domain yet, so this connection cannot be checked.*

**What it is.** The output is *open* — it has no domain, so nothing can be
checked against it.

**What to do.** Set *Updates in* on the output. Code: `output.clock_unset`
(information).

## *Open: nothing supplies it yet.* (an instance's port)

**What it is.** A required port with no binding. The instance works with
what it has; the simulator asks for the port's value as an input.

**What to do.** Draw a link from a provided port or a top-level value to
it — or leave it, if the value really comes from outside.

## *Fits Arduino Nano so far — the binding is not finished.* (Deploy)

**What it is.** Every device that is bound fits, but an output has no
device or a device no output.

**What to do.** *Add device* and choose the output; or choose an output in
the device row. See [Deployment](deployment-errors.md).

## Related

[Incomplete designs](../concepts/incomplete-designs.md) ·
[Status meanings](../reference/status-meanings.md)
