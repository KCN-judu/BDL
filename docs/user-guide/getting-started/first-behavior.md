# Your first behavior

**Goal.** A desk lamp whose brightness follows how far its head is tilted:
upright is off, flat is full. By the end you will have a design with two
concepts, two blocks of them, one rule, one timing domain and one physical
output — checked, saved, and ready to [simulate](first-simulation.md).

**Time.** About twenty minutes.

You will meet each idea when the design needs it. The
[Concepts](../concepts/concepts.md) pages explain each one properly afterwards.

## 1. Create a project

1. Launch Studio ([Install and launch](install-and-launch.md)).
2. Click **New Project…**, choose a location and name the folder `lamp`.

The workspace opens on the **Design** page: a sidebar on the left, an empty
canvas in the middle, an inspector on the right, a status line and the page bar
along the bottom.

## 2. Add the two concepts

A **concept** is a value with a meaning — and a template: the blocks on the
canvas are made from it. The lamp has two: how far it is tilted, and how bright
it is.

1. In the sidebar's **Project** tab, click **+** next to _Concepts_.
2. In the _New concept_ sheet: Name `Tilt`. Under _Value_ choose **Quantity**,
   and for the unit choose **angle** (the symbol `rad` appears in its own
   column). Click **Create**.
3. Again **+** next to _Concepts_: Name `Brightness`, **Quantity**, unit **no
   unit**. Click **Create**.

The two concepts appear in the sidebar, not on the canvas: a concept is the
template, a **block** of it is the thing that holds a value. The round shape in
the sheet's preview means _quantity_; the colour is the concept's own and will
mark every socket that carries _that_ concept. Brightness is a quantity with no
unit: a level from 0 to 1.

> A hollow ring instead of a filled socket would mean the value form is not
> chosen yet — that is allowed, and the guide comes back to it.

**What you made.** Two named meanings. Nothing about numbers yet. The sidebar's
**Library** tab offers ready-made concepts (_Tilt_ and _Brightness_ are both
there).

## 3. Put a block of each on the canvas

A **block** is one value of a concept, updated once per tick. The lamp needs one
tilt and one brightness.

1. Right-click the empty canvas → **Add Block ▸** → **of Tilt**. A block named
   `tilt` lands where you clicked.
2. Right-click again → **Add Block ▸** → **of Brightness**: a block
   `brightness`.

Each block has one output socket on the right, in its concept's colour, and the
word _Source_ in its header with a bar down its left edge: a block with no
formula is **provided from outside** — the environment, a sensor — and stays so
until you say how it is computed. That is not an error. You could stop here,
save, and come back tomorrow.

![Two blocks one above the other, each with a bar at its left edge, an entry arrow and the word Source in its header and one output socket on the right: tilt with a socket labelled Tilt, and brightness with a socket labelled Brightness; no link joins them yet.](../assets/getting-started/declared-relationship.png)

_Two blocks before the formula: tilt, and brightness still provided from
outside._

(Right-clicking the canvas and choosing **Add Block ▸ New Concept ▸** makes a
concept _and_ a block of it in one go, where you clicked; dragging a Library row
onto the canvas does the same.)

## 4. Write the rule

How does brightness follow from the tilt? That is a **rule**: a relationship
that reads a concept and produces another. A rule is a template too — it is
applied inside a block's formula, and is not itself a block on the canvas.

1. Click **+** next to _Mappings_ in the sidebar (the sheet is titled _New
   mapping_; the guide says _relationship_ — they are the same thing).
2. Name `dimByTilt`. Under _Reads_, switch on **Tilt**. Under _Produces_, choose
   **Brightness**. Click **Create**.

`dimByTilt` appears in the sidebar's _Mappings_ list, selected, and the
inspector shows it: _Rule_, reads Tilt, produces Brightness, no formula yet.

1. In the **Relationship** section, the editor opens in its **Formula** view: an
   empty slot, `?`, and the words _produces a Brightness_. Click the slot. Under
   it, _References_ lists **Tilt** — click it. The slot becomes `Tilt`.
2. Click `Tilt` and press **÷**. The formula reads `Tilt ÷ ?`, and the new slot
   is selected: _Expected: an angle, because an angle ÷ an angle = a
   dimensionless quantity._
3. In the number entry type `90`, choose **deg** from the unit pop-up (only
   angle units are offered), and press Return. The formula reads `Tilt ÷ 90 deg`
   and the line under the field says _Valid definition_.

   If you would rather type, switch to **Text** and write it as text:

   ```text
   Tilt / 90 deg
   ```

   Both views edit the same formula. Press **⌘↩** or click **Add definition**.

![The Relationship section of the inspector in Formula view: a Formula | Text switch with an Edit… button at its right, then the formula drawn as a fraction — a Tilt chip over a rule over a dashed empty slot, selected, with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../assets/studio/formula-composer.png)

_The Formula view with the denominator slot selected: the quotient drawn as a
fraction; the compiler says the slot expects an angle and why, and offers a
number with the angle units, the references that fit and the equations whose
result fits._

**What you made.** A rule: a brightness is a tilt divided by ninety degrees. Try
it in the **Text** view: change `90 deg` to `90 s` and watch the line under the
field turn red: it says what Brightness is and what the formula produces instead
— an angle divided by a time is not a plain number. Put `deg` back. Every
formula is checked this way, for units and for meaning, as you type or as you
assemble it — and nothing is saved to the design until you press _Add
definition_.

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../assets/studio/formula-verdict.png)

_The formula field with a draft that does not check: the red verdict line says
what Brightness is and what the formula produces instead._

> In the Text view, **⌃Space** opens completion: the names you can use here
> (_Tilt_), units after a number, keywords. Hovering a name for a moment shows
> what it is. The Formula view asks for nothing: each slot lists what fits.

## 5. The tilt comes in from the environment

Where does the tilt come from? From the environment — a sensor, outside the
design. In BDL a value the environment provides is a **Source**: a block with
**no formula**. `tilt` already is one: it says _Source_ in its header, with the
bar down its left edge and no input socket. Nothing is missing. In the simulator
you will type its value; on a device the sensor will provide it.

## 6. Compute the lamp's brightness

The value the lamp actually shows is the rule applied to the tilt. That is
`brightness`'s formula.

1. Click the `brightness` block. The inspector shows it.
2. Enter the formula `dimByTilt(tilt)` — in the Formula view, click the slot and
   choose **dimByTilt** under _References_ (it becomes `dimByTilt(?)`), then
   click the new slot and choose **tilt**; or type it in the Text view. _Add
   definition._

Completion offers `dimByTilt(` because it is a rule, and `tilt` because it is a
block. On the canvas a **mapping block** appears beside `brightness` — the
formula, drawn as a node: its header names the rule it applies (`dimByTilt`), it
has an input socket labelled `tilt` — one for each block the formula reads — and
a link runs from `tilt`'s output socket into it; a short link joins it to
`brightness`, the block it defines. `brightness`'s header no longer says
_Source_: the value is computed. The rule's name is in the formula, where it is
applied; the rule stays in the sidebar.

> You can also wire by dragging: drop `tilt`'s output socket on a block that has
> no formula yet, or on a hollow `?` socket of a mapping block with an open
> position, and the formula gets the name. What the canvas draws is what the
> formula says — the formula is the one place where blocks are combined.

**What you made.** Two blocks — a Source (`tilt`) and a computed value
(`brightness`) — and a rule (`dimByTilt`) the value applies. The status line at
the bottom counts three relationships and one source.

## 7. Give the values a rhythm

A value that comes from outside has a rhythm: the sensor reports every so often.
In BDL that rhythm is a named **timing domain**, and every value that updates on
its own belongs to one.

1. **+** next to _Timing domains_: name it `interaction`. Create.
2. Select `tilt`. In the inspector's **Timing** section, set **Updates in** to
   _interaction_.
3. Select `brightness` and do the same.

`dimByTilt` stays on _Any timing domain_: it is a pure rule and takes the rhythm
of whatever applies it. The domain's name appears quietly at the right edge of
the two blocks.

**What you made.** A design in which "when does this update" is an explicit
decision. A domain is a name, not a rate — how often _interaction_ ticks is
chosen when you simulate or deploy, not here.

## 8. Add the light

Brightness is a value inside the design. The lamp itself is a **physical
output**: the place where a value leaves the design for the world.

1. **+** next to _Outputs_. In the _New output_ sheet: Name `light`, **Accepts**
   Brightness, **Updates in** interaction, **Required** on. Create.

A sink node appears at the right edge of the canvas, drawn dashed: it has a
domain but nothing drives it yet. The status line says _outputs incomplete_.

1. Drag from `brightness`'s output socket onto the sink's socket. (Or select the
   output and pick `brightness` in the **Connect** pop-up of its inspector; the
   pop-up marks rules — relationships that _have inputs_ — which cannot drive an
   output.)

The sink becomes solid. Only a block that produces the accepted concept in the
same domain can drive an output: `brightness` qualifies, `tilt` does not (the
wrong concept, and the socket refuses it), and a rule has no value to give.

**What you made.** A complete design. The status line no longer says _outputs
incomplete_, and nothing is _not yet defined_: `tilt`, the Source, is meant to
stay without a formula, and the line counts it as _1 source_. Press **⌘S** to
save.

## What you have

![The canvas left to right: the Source block tilt (a bar at its left edge, an entry arrow and the word Source in its header, one output socket labelled Tilt), a link from it into the mapping block dimByTilt, whose left socket is labelled tilt and whose formula line reads dimByTilt(tilt), a short link from its output socket into the block brightness, whose socket is labelled Brightness, and a link from brightness to the light sink at the right; tilt, brightness and the output carry the domain name interaction.](../assets/getting-started/complete-lamp.png)

_The finished lamp: the Source tilt, the mapping block applying the rule
dimByTilt, the block brightness it defines, and the driven light._

| Object         | Kind                                            | Formula           | Updates in    |
| -------------- | ----------------------------------------------- | ----------------- | ------------- |
| **Tilt**       | concept (a template), an angle                  |                   |               |
| **Brightness** | concept (a template), a plain number            |                   |               |
| **tilt**       | block of Tilt with no formula: a Source         | _none_            | _interaction_ |
| **dimByTilt**  | rule reading Tilt, producing Brightness         | `Tilt / 90 deg`   | any           |
| **brightness** | block of Brightness, computed: applies the rule | `dimByTilt(tilt)` | _interaction_ |
| **light**      | physical output driven by brightness            |                   | _interaction_ |

The design is _executable_: every block the output depends on is computed or is
a Source, checks, has a rhythm, and the output has exactly one driver.

## If something does not work

- **The formula line is red.** Read it: it names the problem in terms of your
  concepts ("this divides an angle by a time"). See
  [Types, units and concepts](../troubleshooting/type-and-concept-errors.md).
- **`brightness` will not connect to the output.** Check its _Updates in_
  matches the output's, and that it reads nothing. See
  [Connections](../troubleshooting/connection-errors.md).
- **The status line says _N definitions not added_.** A formula is typed but not
  added; select the block and press _Add definition_ or _Revert_.
- **Nothing checks and the bottom line says _Compiler not connected_.** See
  [Install and launch](install-and-launch.md).

## Next

[Your first simulation](first-simulation.md) — tilt the lamp and watch the
brightness.
