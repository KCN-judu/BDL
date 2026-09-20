---
id: ADR-0042
status: accepted
date: 2026-09-21
area: studio
supersedes: []
superseded-by: []
related: [ADR-0028, ADR-0040, ADR-0018, ADR-0034, ADR-0041]
fv:
  [
    "engineering choice: a projection of the one authored text; nothing formal
    is claimed",
    "production-tested:
    `apps/studio/test/{caret_test,formula_composer_test,formula_keyboard_e2e_test,expanded_formula_test}.dart`,
    `crates/bdl-ide/tests/formula_composer.rs`",
  ]
---

# ADR-0042: The Formula view is typed structure over the compiler's tree — a caret read off the projection, keys that are text edits or compiler actions, a rendering that draws the mathematics, and the same rendering unfolded on the canvas

## Status

Accepted (the authoring UX slice of 2026-09-21). Extends ADR-0028 — one authored
formula text, one parser and type system, every structured action a text edit —
with how the keyboard and the pointer share that text; consumes ADR-0040's
structured node kinds, roles and unit renderings. Nothing here changes what a
formula means.

## Context

ADR-0028 made the Formula view a projection of the draft text with a palette of
structured actions, and typing "the primary path" in the sense that the Text
view was always there. In use, the Formula view was a pointer surface with a few
shortcuts: a key acted on the _selected component_, not at a place; there was no
caret, so a designer who wanted to write `clamp(Tilt / 90 deg, 0, 1)` either
switched to the Text view or clicked five slots; `match`, `let` blocks, rules,
collections and the temporal boundaries were monospace text inside the
structure; a `/` was drawn as `÷` in a row, so a quotient of two quantities did
not look like the quotient it is; and a saved formula was invisible on the
canvas — the node showed a definition line and nothing of what it said, so
reading a design meant opening every relationship.

The references were the mature math editors — GeoGebra's equation input
(fractions built from `/`, the caret staying in a denominator until an arrow key
leaves it, a closing bracket exiting a group, command completion while typing),
Desmos, MathLive (Tab across placeholders, Home / End to a group's ends, a
"speak parent" reading) and Wolfram's typeset input — read for their interaction
model, not their styling. Three constraints bounded the design: Studio owns no
parser, no type checker and no dimension algebra (ADR-0028, ADR-0040); a formula
stays one text so the Text view and the file are always right; and a casual
click on the canvas must never rewrite a formula.

## Decision

1. **The caret is a projection of the compiler's tree.** The Formula view keeps
   a structural caret — a byte offset and the _stop_ it stands at (`CaretState`,
   `app/caret.dart`) — whose stops are read off the `FormulaProjection`:
   _before_ and _after_ every part at its byte range, _in_ an empty slot, _open_
   / _close_ just inside a parenthesised part, and the character positions
   inside a leaf (a name, a number, a unit — text edited as text), sorted in
   reading order with the part each belongs to. Nothing is painted over
   character offsets: where a stop is on screen is the rendered part's layout
   (`FormulaGeometry`, keys per node); what it means is the tree. Left and Right
   walk the stops (the last stop in a denominator is followed by the stop after
   the fraction — how a nested part is left); Up and Down go to the nearest stop
   on the row above or below (a numerator from its denominator, a branch from
   the next); Home and End to the enclosing part's first and last stop, then the
   formula's; Tab and ⇧Tab to the next and previous empty slot; `)` leaves the
   enclosing parentheses, `,` moves to the next argument. These are the
   positions the compiler's `NavigateFormula` (0.27) moves between; Studio walks
   them locally so that no key waits for a round trip, and never invents a stop
   the tree does not have.
2. **A key is a text edit of the draft or a compiler action, never a local
   interpretation.** Letters, digits and a space after a number are text edits
   of the draft at the caret's byte offset (`DefinitionDraftChanged`), read by
   the compiler like any typing in the Text view; an operator, `(` on a name
   (`clamp` → `clamp(?, ?, ?)`, `ComposeAction.apply`, 0.28), `!`, and deleting
   a whole part are `ComposeFormula` actions the compiler answers with text; a
   key that cannot act where the caret stands is refused with a sentence (_type
   an operator first_). While the text differs from what the compiler last read,
   the part being typed into is shown as text in place (`PendingText`) and the
   rest keeps its structure; the picture follows the compiler's next reading. A
   structured action is refused until then (ADR-0028's stale-projection policy).
   Completion is asked on every typed character at the caret's byte offset
   (`CompleteDefinitionDraft`, ranked by the position's expected type —
   `bdl_ide::completion::positional_expectation` — and by the prefix) and shown
   beside the caret; Return or Tab accepts, Tab moves on when the candidate is
   already written.
3. **Pointer and keyboard act on the one draft at once.** A click places the
   caret at the nearest stop _and_ selects the part; the palette (the slot
   panel, the unit pop-up, the operator and form buttons) acts on the selected
   part as before; the caret follows every compiler answer to the part it
   selects (`select`). Neither mode is entered or left: mode switches are not
   counted because there are none.
4. **The rendering draws the mathematics the tree states.** A `/` is a fraction
   (numerator over a rule over the denominator; the operator's stop is the
   rule); parentheses grow with their content; a call is its name and
   parentheses; a choice is a branch diagram — `if` and the condition on the
   spine, `then` and `else` outcomes on their rows; a `match` is the subject on
   the spine and one row per arm, the pattern (the arm's own text, in the
   local's italics) before `⇒` and the body; a `let` block is one row per
   binding over a rule over the result; a rule is its parameters, `⇒`, the body;
   a collection or a group is its items between their delimiters; `delay` and
   `sync` are a region with a bar on its left — the temporal boundary — the word
   and the arguments; a composite unit is drawn as the compiler renders it
   (`unit_display`, `m/s²`). Only `()` stays opaque, as the compiler says. Every
   part has a reading for assistive technology in product words (_Tilt over 90
   deg_, _a choice: if …, then …, else …_, _a match on Tilt with 2 cases_, _a
   delay boundary of …_), assembled from the tree, never from the text.
5. **A saved formula unfolds on its node.** A mapping node with a definition
   carries a disclosure at its definition line; opening it (or _Show Formula_ in
   the menu) extends the node downward with the same rendering, read-only,
   dense, with the first finding under it and _Edit Formula_ to open the
   inspector. The projection comes from the daemon per revision
   (`GetFormulaProjection` → `EditorState.formulaPreviews`); which nodes are
   open, and the measured height of each, is `EditorState.expandedFormulas` —
   layout, never a revision, never persisted. Tapping the unfolded formula
   selects the node and nothing else: editing is the inspector's.

## Alternatives

- **Ask the daemon for every caret move (`NavigateFormula`) and every keystroke
  (`ComposeAction.insert`).** Rejected for Studio: a round trip per arrow key is
  felt, and the stops are a pure function of a tree Studio already holds. The
  daemon's operations stay served for a client without the tree (a future web or
  LSP client) and are the reference the local walk is tested against in spirit:
  the same before / after / open / close positions.
- **A second AST in Dart for local parsing of the typed text.** Rejected by
  ADR-0028 and again here: the pending region shows the typed text as text and
  waits for the compiler; no local parse, no local type, no local unit.
- **Painting a caret over character offsets of the source.** Rejected: the
  Formula view is not a text field with decorations; a caret between two
  characters of `90 deg` and a caret in an empty slot are different kinds of
  place, and Up / Down, Home / End and Tab are defined on the tree, not on
  columns.
- **A whole-keyboard palette (every operator, every function as a button).**
  Rejected: the palette stays contextual — what the slot expects, what fits the
  selected part — and the keyboard is the way to write; a big keyboard is a
  worse Text view.
- **Rendering the formula on the canvas from its text.** Rejected: the canvas
  never parses (ADR-0034's rule for dependencies holds for pictures too); the
  daemon's projection is the only source, fetched per revision.
- **Persisting which formulas are unfolded.** Rejected: it is a view state like
  a collapsed group's box, and a project file must not change because a designer
  looked at something.

## Consequences

- Task C of the UX tasks — `Brightness = clamp(Tilt / 90 deg, 0, 1)` by keyboard
  only — completes in 22 keys and one click with no mode switch, no correction
  and one completion (`apps/studio/test/formula_keyboard_e2e_test.dart`, against
  the real `bdld`); the mixed task E in 14 keys and 4 pointer actions with two
  changes of hand and no correction; the conditional task F in 15 keys and 2
  pointer actions.
- The daemon gains `ComposeAction.apply` (protocol 0.28) and the positional
  ranking of text completion; nothing else on the wire changes.
- `docs/architecture/studio-ui.md` §2 (the unfolded formula, the disclosure) and
  §4b (_Typed structure_), `docs/spec/protocol.md`, the user guide's _Formula
  editor_ and _Canvas_ pages, `docs/project/status.md` and the change fragment
  `2026-09-authoring-ux.md` change with this record.
- Harder: the caret model is a second place that must agree with the tree's byte
  ranges — the tests pin it (`caret_test.dart`); a compiler change to node
  ranges is visible as a caret that lands one character off, which is the
  intended failure mode (it is never a wrong formula).
