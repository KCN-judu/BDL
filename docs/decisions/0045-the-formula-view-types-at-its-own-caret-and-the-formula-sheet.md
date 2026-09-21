---
id: ADR-0045
status: accepted
date: 2026-09-21
area: studio
supersedes: [ADR-0042]
superseded-by: []
related: [ADR-0028, ADR-0040, ADR-0042, ADR-0001]
fv:
  [
    "engineering choice: the editor's own text and caret, the compiler's reading
    downstream; nothing formal is claimed",
    "production-tested:
    `apps/studio/test/{formula_typing_e2e_test,formula_sheet_e2e_test,caret_test,formula_composer_test,formula_keyboard_e2e_test}.dart`
    against the real `bdld`",
  ]
---

# ADR-0045: The Formula view types at its own caret — every key acts on the text Studio holds, now; a key that needs the compiler's tree waits its turn, never dropped — and the definition has a sheet of its own

## Status

Accepted (2026-09-21). Supersedes ADR-0042 on its key model — the last sentence
of its decision 1 (_never invents a stop the tree does not have_) and its
decision 2 (_a key is a text edit or a compiler action; a structured action is
refused until the reading arrives_) — which was **wrong on the same evidence**:
the keyboard tasks it was tested by waited for the compiler's reading after
every key, so the bugs of typing at a designer's pace (characters landing out of
order, ⌫ taking the wrong character, an operator or a `(` silently dropped, the
caret drawn before the word just typed) never appeared. ADR-0042's decisions 3
(pointer and keyboard on the one draft), 4 (the rendering draws the mathematics)
and 5 (a saved formula unfolds on its node) stand unchanged and are carried here
by reference; ADR-0028's rule — one authored text, no parser in Dart, every
structured action a text edit — is untouched.

## Context

The Formula view's keys were computed against the compiler's last reading of the
text: a typed letter was inserted at the byte of the _stop_ the caret had
resolved to in that reading, ⌫ at the stop's byte, an operator as a
`ComposeFormula` action refused while the reading was not of the text on screen.
The draft check is debounced (150 ms) and a reading is a round trip, so at a
designer's pace three or four characters land before the first is read. Each one
after the first was inserted at a stale byte (`ab` then `c` gave `acb`), ⌫
removed the character before the stale byte, an operator or a `(` typed while
the reading was pending was swallowed, and the caret was drawn at the stale stop
— before the word being typed, which is why it "could not reach the end". Two
keys in one frame acted on the same widget text and the second undid the first.

The reference read for this was GeoGebra's editor
(`org.geogebra.editor.share.controller.EditorState`, `CursorController`,
`InputController`, `ArgumentHelper`): the editor owns a sequence tree and a
cursor (`currentNode`, `currentOffset`); every key mutates that tree at that
cursor, synchronously; `/` takes the operand before the cursor as a numerator
(`ArgumentHelper.passArgument`), `(` opens a group, a typed character replaces
the placeholder it touches, ⌫ before a function steps into its last argument;
the serializer produces text on demand and the parser reads it afterwards. The
principle is not the tree — ADR-0028 rejected a second tree in Dart — but the
**ownership of the cursor**: the parser never moves it, and no key waits for the
parser.

The second pressure was room. The composer lived in the inspector's 290 pt
column with the palette under the field: a fraction with a call around it
wrapped, the palette pushed the verdict off screen, and the sheet-sized task —
_write this relationship's definition until it is right_ — had no surface of its
own.

## Decision

1. **The editor owns its text and its caret.** `FormulaComposer` keeps the draft
   text and the caret byte as of the last key (`_source`, `_caret`), ahead of
   the rebuild a dispatch schedules; every key is computed against them and the
   store follows (`DefinitionDraftChanged`, `FormulaCaretMoved`). Two keys in
   one frame both land, in order. The compiler's reading, when it arrives,
   redraws the picture and never moves the caret; when it is not of the text on
   screen, the stops of the last good reading are shifted to the text as it is
   now (`shiftedStops`) and the part being typed is text with the caret inside
   it, measured in the text's own style.
2. **What the text can decide, the text decides — now** (`app/caret.dart`, _the
   sequence, as text_): a character replaces the `?` it touches, extends the
   word it touches, or begins a value where one may begin; a space after a
   number starts its unit; in a slot `-` and `!` are a sign (`-?`, a negative
   number is typed as one) and the other operators split it (`? + ?`); `=` after
   `<` or `>` makes one operator; `(` groups a slot or opens a group where a
   value may begin; `)` steps past the parenthesis before it; ⌫ takes a
   character, the last character of a value leaves a slot in its place, a slot
   goes with the operator that opened it, and before a part the caret steps back
   over the separators (an operator is never deleted on its own, so the text
   stays readable); ⌦ mirrors it; Tab is the next `?` of the text. A value typed
   right after a complete part is refused with the sentence as before. Text the
   compiler could not read is shown as text and edited as text, so what was
   typed can be mended without leaving the view.
3. **What needs the tree waits for the tree, in order.** An operator on a part
   (the compiler puts it after the part with a slot for the other side,
   parenthesised where the part's place needs it — a `+` typed in a denominator
   stays in the denominator, as GeoGebra's field would), `(` after a name (the
   compiler's arity), `!` on a part, ⌫ / ⌦ of a whole structure, `,` and `)`
   when the text does not show the next slot or the parenthesis right there:
   these are `ComposeFormula` actions or tree moves, taken at once when the
   reading is current, and otherwise **queued** — the key kept as pressed, the
   debounced check flushed (`DefinitionDraftFlushRequested` →
   `FlushDraftCheck`), every later key queued behind it, the queue replayed in
   order when the reading (or the action's answer) arrives. Nothing typed is
   dropped; Esc empties the queue with the caret. A structured action still goes
   out only over the text the compiler read (ADR-0028's stale-projection
   policy): the queue is how the composer honours it without losing a key.
4. **When the reading arrives, the part under the caret is selected**, so the
   palette is about the same part the keys are — what a compiler answer did
   already.
5. **The definition has a sheet.** _Edit…_ beside the Formula | Text switch (⌘E
   from the inspector's field) opens the formula sheet over the design
   (`EditorState.formulaSheet`, `ui/formula_sheet.dart`): the same
   `DefinitionEditor` and the same composer over the same draft, laid out as an
   equation — the socket glyph of what the relationship produces, its name, `=`,
   the expression at the display size (`MacType.display`, 20 pt, the one display
   size of the type scale) in one field — over the verdict and the findings;
   beside it, in a column of its own, _This position_: what the selected
   position expects and what fits (the palette), or how to get there and what
   the formula may read. _Revert_ at the left; _Done_ closes with the draft kept
   (Esc, after the caret and completion), _Save_ / _Add definition_ commits and
   closes (⌘↩). The inspector's section says _Editing in the sheet._ meanwhile;
   another selection closes the sheet. Nothing is held in the sheet: it is a
   projection of the editor's state, and the inspector's field is unchanged for
   a quick edit.

## Alternatives

- **A local sequence tree in Dart, GeoGebra's model outright.** Rejected again
  (ADR-0028): precedence and parenthesisation would move into Studio. The text
  _is_ the sequence; the byte is the cursor; the text rules above are the ones
  that need no grammar, and the queue is the price of leaving the rest to the
  compiler — a flush and one round trip, not a debounce.
- **Every operator as a text insertion (`a + ?` at the caret).** Considered and
  rejected: at the end of what is being typed it is what the compiler would
  answer, but in a denominator or a numerator it changes the fraction
  (`tilt / 9 + ?` reads as `(tilt / 9) + ?`), where GeoGebra's field and the
  compiler's `compose` both keep the operand in its place. The cost of waiting
  for the reading is smaller than the cost of a wrong tree.
- **Locking the keyboard while the compiler reads.** Rejected: that is what was
  happening, by accident, and it is the bug.
- **The sheet as the only editor, the inspector a read-only preview.** Rejected
  for now: the inspector's field is the quick edit and every existing task and
  test goes through it; the sheet is the roomy one. If the inspector's field
  proves to be only a preview in use, that is a later decision.
- **A floating window for the editor.** Rejected: a sheet attaches to the task
  with a clear end (write this definition, save or close), as the platform has
  it.

## Consequences

- Typing at any pace gives the text in the order typed; the caret is where the
  designer left it; a structural key is answered as soon as the compiler has
  read what precedes it, never dropped. The realistic-pace suite
  (`formula_typing_e2e_test.dart`) is the guard: it never waits for a reading
  between the keys of a word, which the keyboard brief did after every key.
- The type scale gains one size, `MacType.display` (20), for the sheet's
  expression only (`docs/architecture/studio-ui.md` §3).
- A new action and effect (`DefinitionDraftFlushRequested`, `FlushDraftCheck`)
  let a waiting key skip the debounce; no protocol change.
- Records changed: `docs/architecture/studio-ui.md` §3, §4b (_Typed structure_,
  _The formula sheet_); `docs/project/status.md`; the fragment
  `2026-09-formula-typing.md`; `docs/user-guide/studio/formula-editor.md` and
  `reference/keyboard-and-mouse.md` (with `VERIFICATION.md`).
