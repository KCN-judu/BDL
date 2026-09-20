# Formula editor

The **Relationship** section of a relationship's inspector is where a formula is
written. It is not a plain text box: the compiler checks what you type as you
type it, and nothing reaches the design until you say so.

The editor has two views of the same formula, chosen with the **Formula | Text**
switch at its top. **Formula** shows the expression as the mathematics it is —
references, numbers with their units, a fraction for a division, a function and
its arguments, a choice as a branch — with a caret you type at and a selected
part the buttons beneath act on, and tells you what each empty slot expects.
**Text** is the formula as written. Switching does nothing to the formula: both
edit one draft, and what you build in one is what you read in the other.

## Typing a formula

The Formula view is written at a **caret**, like a text field, but the caret
moves through the formula's parts rather than its characters: before and after
each part, inside an empty slot, inside a name or a number, just inside a pair
of parentheses. Click where you want to write, or start typing in an empty
formula (_Type to write, or choose a part_).

- **Letters and digits** type into the slot or the name or number the caret
  touches. Typing `Til` opens the completion list at the caret — the concepts
  this relationship reads, the design's relationships, the equations, ranked by
  what this place expects — and **Return** or **Tab** takes the highlighted row.
  A **space after a number** starts its unit: `90` `⎵` `deg`.
- **`+ − * /`, `< >`, `=`, `&`, `|`** put that operator after the part the caret
  touches, with a slot for the other side; `/` draws a fraction and puts the
  caret in the denominator. **`!`** negates the part.
- **`(`** after an equation's name applies it: `clamp` becomes `clamp(?, ?, ?)`
  with the first slot ready to type into. `(` in an empty slot opens a group.
- **← →** move to the previous / next place — out of a denominator, past a
  parenthesis, into the next part. **↑ ↓** move between the rows of a fraction
  or a choice. **Home / End** go to the ends of the enclosing part, and again to
  the ends of the formula. **Tab / ⇧Tab** jump to the next / previous empty
  slot. **`)`** leaves the parentheses you are in; **`,`** moves to the next
  argument.
- **⌫ / ⌦** delete a character of a name or a number, or a whole part when the
  caret is beside one — a slot's operator goes with it.

So `clamp(Tilt / 90 deg, 0, 1)` is typed as `clamp` `(` `Tilt` `/` `90` `⎵deg`
`,` `0` `,` `1` — 22 keys, and the completion list would have taken `clamp`
after `cl` and `Tilt` after `Ti`. The part you are typing into is shown as text
until the compiler has read it — a moment — and the rest of the formula keeps
its shape. A key that cannot act where the caret is says why beneath the field
(_Type an operator before adding a value here._) and changes nothing.

The pointer works alongside: clicking a part places the caret there _and_
selects the part, so the buttons beneath the field (below) act on it; the caret
follows every action to the part that comes next. There is no mode to switch.

## Assembling a formula

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula drawn as a fraction — a Tilt chip over a rule over a dashed empty slot, selected, with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, a folded Equations row and a Choose button.](../assets/studio/formula-composer.png)

_The Formula view with the denominator slot selected: the quotient drawn as a
fraction; the compiler says the slot expects an angle and why, and offers a
number with the angle units, the references that fit and the equations whose
result fits._

An empty formula is one **slot** — a dashed box reading `?`, the place where a
value is still to be written. Click a slot and the compiler says what it expects
there and offers what fits:

- **A number.** Type it, choose its unit from the pop-up, press Return or
  **Insert**. The pop-up lists only the units of the kind of value the slot
  expects — for an angle, _rad_, _deg_, _turn_; never a length or a time. A slot
  that expects a plain number has no unit to choose.
- **A reference.** The concepts this relationship reads and the design's
  relationships whose value is the right kind — each with what it produces. A
  reference is inserted as it is: its kind comes from its declaration, and it
  gets no unit pop-up. A value — a Source such as `tilt`, or a computed value —
  is written by its name alone (`tilt`, never `tilt()`); a rule is applied with
  its arguments (`dimByTilt(?)`).
- **An equation.** Folded under _Equations_: the library's equations whose
  result can be the value expected here (for an angle, `min`, `max`, `clamp`,
  `sum` …; not `any`, which produces true or false). Choosing one inserts it
  with a slot per argument.
- **A truth value.** Where the slot expects true or false — the side of an
  `and`, a condition, a concept with the On / off value form — there is no
  number to type: two buttons, **true** and **false**, stand in its place, and
  _References_ lists the values that are true or false, the concepts this
  relationship reads first.
- **A form.** **Choose** opens a choice in the slot, `if ? then ? else ?`; on a
  slot that expects true or false, **not** opens a negation, `not ?`.

Click a part that is already there and the compiler says what it is. Above it,
the actions on that part: **+ − × ÷** put that operator after it with a new slot
for the other side, **and** / **or** likewise for a part that is (or may be)
true or false, **not** negates that part in place (no new slot), **Compare**
puts `<`, `==` and the rest after it, **Function** wraps it in an equation
(`clamp(…, ?, ?)`), **Each element** reads a collection element by element
(`all reading in readings: ?` — the editor picks a readable name for the
element, `reading` for `readings`, `item` otherwise), **Range** asks whether the
value lies between two ends (`… in ? .. ?`), **Choose** makes the part one
outcome of a choice (`if ? then … else ?`, the condition selected next), and
**Remove** turns it back into a slot — removing the slot next to an operator
removes the operator with it, and an empty `not ?` goes with its slot.
Parentheses are added where the operators need them: a sum divided by something
becomes `(a + b) / ?`, an `or` under an `and` becomes `(a or b) and ?`, and a
choice under anything is `(if … then … else …)`.

The line under the field — _Expected: an angle, because an angle ÷ an angle = a
dimensionless quantity._ — is the compiler's reasoning in plain words. It works
out what a slot must be from what is around it: the result the relationship must
produce, and the other side of the operator. `? / 1 s` for a speed expects a
length; `Force * ?` for a torque expects a length. When nothing around a slot is
known yet — a product of two slots — it says so and offers no unit; fill the
other side first. **Explain** shows the same in the compiler's notation.

A **number with a unit** is its number and its unit; a unit made of several —
`m per s^2` in the text — is drawn the way it is read, `m/s²`. Selecting the
number shows the unit pop-up. Editing the number is a new quantity in the same
unit. Choosing another unit from the pop-up keeps the quantity and rewrites the
number: `180 deg` becomes `3.141592653589793 rad`. The two are different things,
and the pop-up never does the first.

A **division** is drawn as a fraction, the numerator over the denominator; the
line between them is the division itself — click it to select the whole
quotient. A **choice** is drawn as a branch: `if` and its condition on the
spine, `then` and `else` with their outcomes on the rows under it. Each part is
an ordinary component: the condition expects true or false, and both outcomes
expect what the choice must give — the relationship's result at the top, or,
inside a larger formula, whatever the other outcome already is. The logical
operators are shown as the words **and**, **or** and **not** (`&&`, `||` and `!`
in the text), in the weight of the language's own words. A **`match`** is its
subject on the spine and one row per case — the pattern, `⇒`, the outcome; a
block with **`let`** is one row per local binding over a line over the result; a
rule `x => …` is its parameter, `⇒`, the body; a collection `[…]` or a group
`(…)` its items; **`delay`** and **`sync`** a shaded region with a bar on its
left, the word and its arguments. Every part reads aloud to a screen reader as
what it is — _Tilt over 90 deg_, _a choice: if Held, then 1, else 0_.

A formula over a collection is drawn the way it reads:
`all reading in readings:` on one line and the condition indented under it. The
element's name is in italics wherever it appears — it belongs to this formula,
not to the design, so renaming a concept never touches it — and selecting it
says what one element is. A range is its two ends around `..`; each end expects
the same kind as the value before `in`, so its unit pop-up lists that kind's
units.

The formula is ordinary text underneath: `clamp(Tilt / 90 deg, 0, 1)` reads
exactly so in the **Text** view, and a formula typed as text appears in the
**Formula** view — with `?` wherever text left a slot;
`all reading in readings: reading < limit` and
`if RoomTemp > 299.15 K && ButtonHeld then true else false` typed as text come
back as the same words, every part selectable. Only an empty group `()` is shown
as text. Text that cannot be read as a formula keeps exactly what you typed; the
Formula view shows no parts for it, says _The text cannot be read as a formula._
and offers **Edit as text**. After any change the Formula view waits for the
compiler's reading of the new text — _Waiting for the compiler to read the
formula…_, the parts dimmed — before it offers the next action, so nothing you
click ever acts on text that has already changed.

## The text field and its verdict

![The Relationship section of the inspector: the formula field containing Tilt / 90 s with an unsaved marker in the section header, and under it a red message saying Brightness is a dimensionless quantity but this formula produces an angular rate, the offending span quoted, the explanation that the mapping's signature promises Brightness, and Revert and Save definition buttons.](../assets/studio/formula-verdict.png)

_The formula field with a draft that does not check: the red verdict line says
what Brightness is and what the formula produces instead._

- **The hint** shown in the empty field names what you can write over:
  _expression over Tilt, Held_ — the concepts this relationship reads — or
  _expression with no inputs_ for a value.
- **The verdict line**: _Checking…_ while the compiler looks, then _Valid
  definition_, or an orange line for something still to decide (_Tilt has no
  representation yet._), or a red line with the first finding's message. Orange
  is _not yet_; red is _wrong now_.
- **Underlines** mark the spans each finding is about, in the field itself, and
  each finding is repeated below with its excerpt, an explanation and any fixes.
  Above, the whole formula is the span: `Tilt / 90 s` is an angle over a time,
  an angular rate, where Brightness must be a plain number.
- **The buttons**: **Add definition** when the relationship has no formula yet,
  **Save definition** when it has one; **Revert** while the text differs from
  what is saved; **Detach definition** to remove the formula and return the
  relationship to _declared_. They are the same in both views: a formula with a
  slot still in it can be saved, and is _invalid_ until the slot is filled.

## Drafts

Typing creates a **draft**. The draft is part of the project: it survives
switching selection and switching pages, is saved with the project (whether or
not it checks, and even when the field is empty), and comes back in the editor
when you reopen. The status line counts _N definitions not added_; adding it is
a design change. The canvas does not change — a draft must not make an
unresolved relationship look defined.

When the design changes under a draft — you changed a concept's unit, or undo
replaced the saved formula — the draft is re-checked against the new design, and
if the saved formula itself changed while you were typing, the field shows a
notice with **Reload** (take the saved one) or **Keep mine** (keep typing over
it). Nothing is overwritten silently.

Saving an **invalid** formula is allowed: the relationship becomes _invalid_
with the same finding you saw while typing. A design may record a formula before
it is right, just as a concept may exist before its value form is chosen.

## Keys

| Key                         | Does                                                                                                                         |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **⌘↩**                      | Add / Save the definition                                                                                                    |
| **Esc**                     | Revert a draft (a second Esc when completion is open closes it first)                                                        |
| **Return**                  | in the Text view a new line — formulas may span lines; in a number entry, insert the number                                  |
| **⌃Space**                  | open completion (both views; the Formula view also opens it as you type a name)                                              |
| **↑ / ↓**, **Return / Tab** | move in and accept from the completion list                                                                                  |
| **← →**                     | Formula view: the previous / next place — out of a denominator, past a parenthesis, into the next part                       |
| **↑ ↓**                     | Formula view: the row above / below (a numerator from its denominator, a branch from the next)                               |
| **Home / End**              | Formula view: the ends of the enclosing part; again, the ends of the formula                                                 |
| **Tab / ⇧Tab**              | Formula view: the next / previous empty slot                                                                                 |
| **) ,**                     | Formula view: leave the parentheses / move to the next argument                                                              |
| **letters, digits, space**  | Formula view: type into the slot or the name or number at the caret; a space after a number starts its unit                  |
| **+ − \* /**, **< >**       | Formula view: put that operator after the part at the caret (or the selected part), with a slot for the other side           |
| **=**, **&**, **\|**        | likewise `==`, `&&` (and), `\|\|` (or); `<=`, `>=` and `!=` are in the **Compare** pop-up                                    |
| **!**                       | negate the part in place (`not …`)                                                                                           |
| **(**                       | Formula view: apply the name before the caret (`clamp` → `clamp(?, ?, ?)`), or group a slot                                  |
| **⌫ / ⌦**                   | Formula view: a character of a name or number, or the whole part beside the caret (an empty slot takes its operator with it) |
| **⌘S**                      | _Save project_ — never saves a draft; a dirty formula and an unsaved project are two different states                        |

## Completion and hover

**⌃Space** opens a list at the caret — in the Formula view it also opens as you
type a name: the concepts in scope, the design's relationships (rules come with
a `(`), units after a number, keywords, and `delay(…)` / `sync(…)` where they
are allowed. The list is the compiler's, filtered and ordered by it — in a
formula, by what the place you are typing in expects: in `? / CycleTime` the
lengths come before a speed — and each row shows the kind and the resulting
type. It re-asks on every keystroke while open.

Resting the pointer on a name for a moment shows a **card**: what the name is,
its value form, its status, a description. Never a formal term — those are in
_Explain_.

## What is checked

Dimensions (units must work out), meaning (the result must be the concept the
signature promises), shape (a rule where a value is expected, a value applied
like a rule, a call with the wrong number of arguments), names (unknown,
ambiguous, or a concept named that this relationship does not read), and the
placement of `delay` / `sync`. Timing across domains and instantaneous cycles
are checked once the formula is saved, because they depend on the rest of the
design; timing findings appear under **Timing**, connection findings under
**Drives**, and the rest (an instantaneous cycle, say) under the
**Relationship** section beneath the editor — all on the relationship they
concern.

## Related

[Relationships](../concepts/relationships.md) ·
[Formula language](../reference/formula-language.md) ·
[Types, units and concepts](../troubleshooting/type-and-concept-errors.md)
