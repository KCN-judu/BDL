# Your first behavior

**Goal.** A desk lamp whose brightness follows how far its head is tilted:
upright is off, flat is full. By the end you will have a design with two
concepts, three relationships, one timing domain and one physical output —
checked, saved, and ready to [simulate](first-simulation.md).

**Time.** About twenty minutes.

You will meet each idea when the design needs it. The
[Concepts](../concepts/concepts.md) pages explain each one properly
afterwards.

## 1. Create a project

1. Launch Studio ([Install and launch](install-and-launch.md)).
2. Click **New Project…**, choose a location and name the folder `lamp`.

The workspace opens on the **Design** page: a sidebar on the left, an empty
canvas in the middle, an inspector on the right, a status line and the page
bar along the bottom. The canvas says what to do first: add a concept from
the sidebar.

## 2. Add the two concepts

A **concept** is a value with a meaning. The lamp has two: how far it is
tilted, and how bright it is.

1. In the sidebar's **Project** tab, click **+** next to *Concepts*.
2. In the *New concept* sheet: Name `Tilt`. Under *Value* choose
   **Quantity**, and for the unit choose **angle** (the symbol `rad`
   appears in its own column). Click **Create**.
3. Again **+** next to *Concepts*: Name `Brightness`, **Quantity**, unit
   **no unit**. Click **Create**.

Each concept is one row on the canvas with a round socket at each end. The
round shape means *quantity*; the colour is the concept's own and will
appear wherever *that* concept is used. Brightness is a quantity with no
unit: a level from 0 to 1.

> The sheet previews the row as you type. A hollow ring instead of a
> filled socket would mean the value form is not chosen yet — that is
> allowed, and the guide comes back to it.

**What you made.** Two named meanings. Nothing about numbers yet. The
sidebar's **Library** tab offers ready-made concepts (*Tilt* and
*Brightness* are both there); dragging one onto the canvas is the same as
filling in the sheet.

## 3. Add the relationship

A **relationship** says how one concept follows from others.

1. Click **+** next to *Mappings* in the sidebar (the sheet is titled *New
   mapping*; the guide says *relationship* — they are the same thing).
2. Name `dimByTilt`. Under *Reads*, switch on **Tilt**. Under *Produces*,
   choose **Brightness**. Click **Create**.

A node appears with one input socket (Tilt, on the left) and one output
socket (Brightness, on the right). It is drawn **dashed** with the word
*declared* in its header: the relationship exists and has a signature, but
no formula yet. That is not an error. You could stop here, save, and come
back tomorrow.

<!-- figure F3 -->

## 4. Write the formula

1. Click the `dimByTilt` node. The inspector shows it.
2. In the **Relationship** section, type into the field:

   ```
   Tilt / 90 deg
   ```

   As you type, the line under the field says *Checking…* and then
   *Valid definition*. Press **⌘↩** or click **Add definition**.

The node now shows its formula in the body and is drawn solid.

**What you made.** A rule: brightness is the tilt divided by ninety
degrees. Try changing `90 deg` to `90 s` and watch the line under the field
turn red: it says what Brightness is and what the formula produces
instead — an angle divided by a time is not a plain number. Put `deg`
back. Every formula is checked this way, for
units and for meaning, as you type — and nothing is saved to the design
until you press *Add definition*.

> **⌃Space** in the field opens completion: the names you can use here
> (*Tilt*), units after a number, keywords. Hovering a name for a moment
> shows what it is.

<!-- figure F4 -->

## 5. Bring the tilt in from outside

`dimByTilt` is a rule, not a value: it needs a tilt to work on. Where does
the tilt come from? From a sensor — from outside the design. In BDL a
value that arrives from outside is a relationship that **reads nothing**,
produces the concept, and has **no formula**.

1. **+** next to *Mappings*: Name `tilt`, read nothing, produce **Tilt**.
   Create.

Leave it declared. Its dashed outline now means "supplied from outside":
in the simulator you will type its value; on a device it will be the
sensor.

## 6. Compute the lamp's brightness

The value the lamp actually shows is `dimByTilt` applied to `tilt`. That
is again a relationship that reads nothing and produces Brightness — this
time with a formula.

1. **+** next to *Mappings*: Name `brightness`, read nothing, produce
   **Brightness**. Create.
2. Select it and enter the formula `dimByTilt(tilt)`. *Add definition.*

Completion offers `dimByTilt(` because it is a relationship with an
input, and `tilt` because it is a value. The formula field is the only
place where relationships are combined; links on the canvas show *which
concepts* a relationship reads, not the arithmetic.

**What you made.** Three relationships: an input (`tilt`), a rule
(`dimByTilt`) and a computed value (`brightness`). The status line at the
bottom counts them.

## 7. Give the values a rhythm

A value that comes from outside has a rhythm: the sensor reports every so
often. In BDL that rhythm is a named **timing domain**, and every value
that updates on its own belongs to one.

1. **+** next to *Timing domains*: name it `interaction`. Create.
2. Select `tilt`. In the inspector's **Timing** section, set **Updates in**
   to *interaction*.
3. Select `brightness` and do the same.

`dimByTilt` stays on *Any timing domain*: it is a pure rule and takes the
rhythm of whatever applies it. The domain's name appears quietly at the
right edge of the two nodes.

**What you made.** A design in which "when does this update" is an explicit
decision. A domain is a name, not a rate — how often *interaction* ticks
is chosen when you simulate or deploy, not here.

## 8. Add the light

Brightness is a value inside the design. The lamp itself is a **physical
output**: the place where a value leaves the design for the world.

1. **+** next to *Outputs*. In the *New output* sheet: Name `Light Output`,
   **Accepts** Brightness, **Updates in** interaction, **Required** on.
   Create.

A sink node appears at the right edge of the canvas, drawn dashed: it has a
domain but nothing drives it yet. The status line says *outputs
incomplete*.

2. Drag from `brightness`'s output socket onto the sink's socket. (Or
   select the output and pick `brightness` in the **Connect** pop-up of its
   inspector; the pop-up marks relationships that *have inputs*, which
   cannot drive an output.)

The sink becomes solid. Only a relationship that *reads nothing* and
produces the accepted concept in the same domain can drive an output.
`brightness` qualifies; connecting `dimByTilt` instead would be accepted
as an edit and then reported under the output as a connection that does
not fit.

**What you made.** A complete design. The status line no longer says
*outputs incomplete*; it still counts *1 not yet defined* — that is
`tilt`, the input, which is meant to stay without a formula. Press **⌘S**
to save.

<!-- figure F5 -->

## What you have

```
 ○ Tilt                 tilt ──▶ (Tilt)                     interaction
 ○ Brightness           dimByTilt : Tilt → Brightness   =  Tilt / 90 deg
                        brightness ──▶ (Brightness)       =  dimByTilt(tilt)   interaction
                        Light Output ◀── brightness                              interaction
```

The design is *executable*: every relationship the output depends on is
defined or is an input, checks, has a rhythm, and the output has exactly
one driver.

## If something does not work

* **The formula line is red.** Read it: it names the problem in terms of
  your concepts ("this divides an angle by a time"). See
  [Types, units and concepts](../troubleshooting/type-and-concept-errors.md).
* **`brightness` will not connect to the output.** Check its *Updates in*
  matches the output's, and that it reads nothing. See
  [Connections](../troubleshooting/connection-errors.md).
* **The status line says *N unsaved definitions*.** A formula is typed but
  not added; select the node and press *Add definition* or *Revert*.
* **Nothing checks and the bottom line says *Compiler not connected*.**
  See [Install and launch](install-and-launch.md).

## Next

[Your first simulation](first-simulation.md) — tilt the lamp and watch the
brightness.
