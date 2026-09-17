# Formula editor

The **Relationship** section of a relationship's inspector is where a formula is
written. It is not a plain text box: the compiler checks what you type as you
type it, and nothing reaches the design until you say so.

## The field and its verdict

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
  relationship to _declared_.

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
| **Return**                  | a new line — formulas may span lines                                                                  |
| **⌃Space**                  | open completion                                                                                       |
| **↑ / ↓**, **Return / Tab** | move in and accept from the completion list                                                           |
| **⌘S**                      | _Save project_ — never saves a draft; a dirty formula and an unsaved project are two different states |

## Completion and hover

**⌃Space** opens a list at the caret: the concepts in scope, the design's
relationships (rules come with a `(`), units after a number, keywords, and
`delay(…)` / `sync(…)` where they are allowed. The list is the compiler's,
filtered and ordered by it; each row shows the kind and the resulting type. It
re-asks on every keystroke while open.

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
