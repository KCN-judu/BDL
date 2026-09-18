# Formula editor

The **Relationship** section of a relationship's inspector is where a formula is
written. It is not a plain text box: the compiler checks what you type as you
type it, and nothing reaches the design until you say so.

The editor has two views of the same formula, chosen with the **Formula | Text**
switch at its top. **Formula** shows the expression as parts you assemble —
references, numbers with their units, operators, functions — and tells you what
each empty slot expects. **Text** is the formula as written. Switching does
nothing to the formula: both edit one draft, and what you build in one is what
you read in the other.

## Assembling a formula

![The Relationship section of the inspector in Formula view: a Formula | Text switch, then the formula as components — a Tilt chip, a division sign and a dashed empty slot with a red underline — and beneath it the line Expected: an angle, because an angle ÷ an angle = a dimensionless quantity with an Explain link, a number entry with a unit pop-up reading rad and an Insert button, a References list with Tilt and tilt, and a folded Equations row.](../assets/studio/formula-composer.png)

_The Formula view with the denominator slot selected: the compiler says the slot
expects an angle and why, and offers a number with the angle units, the
references that fit and the equations whose result fits._

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
  gets no unit pop-up.
- **An equation.** Folded under _Equations_: the library's equations whose
  result can be the value expected here (for an angle, `min`, `max`, `clamp`,
  `sum` …; not `any`, which produces true or false). Choosing one inserts it
  with a slot per argument.

Click a part that is already there and the compiler says what it is. Above it,
the actions on that part: **+ − × ÷** put that operator after it with a new slot
for the other side, **Compare** likewise for `<`, `==` and the rest,
**Function** wraps it in an equation (`clamp(…, ?, ?)`), and **Remove** turns it
back into a slot — removing the slot next to an operator removes the operator
with it. Parentheses are added where the operators need them: a sum divided by
something becomes `(a + b) / ?`.

The line under the field — _Expected: an angle, because an angle ÷ an angle = a
dimensionless quantity._ — is the compiler's reasoning in plain words. It works
out what a slot must be from what is around it: the result the relationship must
produce, and the other side of the operator. `? / 1 s` for a speed expects a
length; `Force * ?` for a torque expects a length. When nothing around a slot is
known yet — a product of two slots — it says so and offers no unit; fill the
other side first. **Explain** shows the same in the compiler's notation.

A **number with a unit** is two fields: the number and its unit. Editing the
number is a new quantity in the same unit. Choosing another unit from the pop-up
keeps the quantity and rewrites the number: `180 deg` becomes
`3.141592653589793 rad`. The two are different things, and the pop-up never does
the first.

The formula is ordinary text underneath: `clamp(Tilt / 90 deg, 0, 1)` reads
exactly so in the **Text** view, and a formula typed as text appears in the
**Formula** view — with `?` wherever text left a slot. Some forms — `if`,
`match`, a block with `let`, a rule `x => …`, a collection or grouped literal,
`delay` / `sync` — are shown as text in the Formula view and edited in the Text
view. Text that cannot be read as a formula keeps exactly what you typed; the
Formula view shows the last readable form dimmed, says so, and offers **Edit as
text**.

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

Typing creates a **draft**. The draft is yours: it survives switching selection,
switching pages and closing the project (it comes back when you reopen). The
status line counts _N unsaved definitions_. The canvas does not change — a draft
must not make an unresolved relationship look defined.

When the design changes under a draft — you changed a concept's unit, or undo
replaced the saved formula — the draft is re-checked against the new design, and
if the saved formula itself changed while you were typing, the field shows a
notice with **Reload** (take the saved one) or **Keep mine** (keep typing over
it). Nothing is overwritten silently.

Saving an **invalid** formula is allowed: the relationship becomes _invalid_
with the same finding you saw while typing. A design may record a formula before
it is right, just as a concept may exist before its value form is chosen.

## Keys

| Key                         | Does                                                                                                  |
| --------------------------- | ----------------------------------------------------------------------------------------------------- |
| **⌘↩**                      | Add / Save the definition                                                                             |
| **Esc**                     | Revert a draft (a second Esc when completion is open closes it first)                                 |
| **Return**                  | in the Text view a new line — formulas may span lines; in a number entry, insert the number           |
| **⌃Space**                  | open completion (Text view)                                                                           |
| **↑ / ↓**, **Return / Tab** | move in and accept from the completion list                                                           |
| **Tab**                     | in the Formula view, move to the next part                                                            |
| **+ − \* /**, **⌫**         | on a selected part in the Formula view: put that operator after it; remove it                         |
| **⌘S**                      | _Save project_ — never saves a draft; a dirty formula and an unsaved project are two different states |

## Completion and hover

In the Text view, **⌃Space** opens a list at the caret: the concepts in scope,
the design's relationships (rules come with a `(`), units after a number,
keywords, and `delay(…)` / `sync(…)` where they are allowed. The list is the
compiler's, filtered and ordered by it; each row shows the kind and the
resulting type. It re-asks on every keystroke while open.

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
