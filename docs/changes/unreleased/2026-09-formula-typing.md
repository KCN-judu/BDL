# Studio: the Formula view types at its own caret; the formula sheet

- Date: 2026-09-21
- Area: studio
- Affected: designers, developers
- Related: ADR-0045 (supersedes ADR-0042's key model), ADR-0028,
  docs/architecture/studio-ui.md §3, §4b

## What changed

- **Typing at any pace.** The Formula view's keys act on the text and the caret
  the composer holds as of the last key, not on the compiler's last reading:
  characters land in the order typed (before, `ab` then `c` at a designer's pace
  gave `acb`), ⌫ takes the character just typed, the caret is drawn inside the
  part being typed and reaches the end of the field, and two keys in one frame
  both land. What the text can decide is done at once — a character, a space
  before a unit, a slot's sign (`-?`), `<=` from `<` then `=`, `(?)`, `)` past a
  parenthesis, ⌫ / ⌦ by character with a slot left where the last character of a
  value was and a slot removed with the operator that opened it. Text the
  compiler could not read is edited as text in the Formula view, so it can be
  mended there.
- **A structural key waits its turn, never dropped.** An operator on a part, `(`
  after a name, `!`, ⌫ of a whole part, `,` and `)` into the tree are the
  compiler's as before; typed while the compiler has not read what precedes
  them, they are queued in order, the debounced check is sent at once, and they
  replay when the reading arrives (before, they were silently swallowed:
  `tilt/90 deg` typed in one go gave `tilt / ?`). When the reading arrives the
  part under the caret is selected, so the palette is about it.
- **The formula sheet.** _Edit…_ beside the Formula | Text switch (⌘E from the
  inspector's field) opens the definition over the design with room: the same
  editor over the same draft, as an equation — the socket glyph of what the
  relationship produces, its name, `=`, the expression at 20 pt — with the
  verdict and findings under it and _This position_ (what the selected position
  expects and what fits) in a column beside it. _Revert_ left; _Done_ or Esc
  closes with the draft kept; _Save_ or ⌘↩ saves and closes. The inspector's
  field stays for a quick edit and says _Editing in the sheet._ meanwhile.
- **Developers.** `FormulaComposer` keeps `_source` / `_caret` and a key queue;
  `app/caret.dart` gains the text-level rules (`textCharacterAt`,
  `textOperatorAt`, `textBackspaceAt`, `textDeleteAt`, `shiftedStops`,
  `NeedsStructure`) and the tree-level ones take the current text and caret byte
  (`source:`, `at:`); `DefinitionDraftFlushRequested` → `FlushDraftCheck` sends
  a debounced draft check now; `FormulaSheetOpened` / `FormulaSheetDismissed`,
  `EditorState.formulaSheet`; `DefinitionEditor.layout`, `FormulaPalette` (the
  slot panel, public); `MacType.display`. No protocol change.

## Compatibility and migration

Nothing: no persisted file, protocol message or observable semantics changed.
The keyboard brief (`formula_keyboard_e2e_test.dart`) still passes key by key;
the new `formula_typing_e2e_test.dart` types without waiting for a reading.

## Evidence

`apps/studio/test/formula_typing_e2e_test.dart` (a word, two keys in one frame,
End then typing, ⌫ at pace, a whole formula in one go, a sign and a negation,
unreadable text mended — all against the real `bdld` with the real 150 ms
debounce), `apps/studio/test/formula_sheet_e2e_test.dart` (_Edit…_, ⌘E, Esc with
the draft kept, ⌘↩ saving and closing), `apps/studio/test/caret_test.dart` (the
text rules, the shifted stops), `apps/studio/test/formula_composer_test.dart` (a
structural key queued, flushed and replayed), and the unchanged keyboard brief
`apps/studio/test/formula_keyboard_e2e_test.dart`.
