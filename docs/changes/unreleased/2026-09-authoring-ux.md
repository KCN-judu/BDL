# Value categories in the Library, the concept sheet, typed structure in the Formula view, formulas unfolded on the canvas (protocol 0.28)

- Date: 2026-09-21
- Area: studio, library, ide, protocol, daemon
- Affected: designers, protocol clients, library authors, developers
- Related: ADR-0041, ADR-0042, ADR-0040, ADR-0028, ISS-0019,
  [2026-09 composite units and the Formula view's structure](2026-09-composite-units-and-formula-structure.md),
  [2026-09 Standard Library](2026-09-standard-library.md)

## What changed

- **The Standard Library is 22 value categories** (`library/std/concepts.toml`
  0.3): four value forms — _On / off_, _Count_, _Level_, _Decide later_ — and
  one item per named quantity of the vocabulary (_Angle_ … _Illuminance_). The
  36 product presets (_Motor Angle_, _Servo Position_, _Fan Speed_, _Heater
  Power_, _Tilt_, _Dial Position_, …) and the 8 Source presets are gone; their
  words are search synonyms of the category they were (`servo`, `tilt`,
  `encoder` → _Angle_; `heater`, `fan`, `dimmer`, `battery`, `humidity` →
  _Level_). The Library tab is _Recent_, _Values_, _Quantities_ and a _Sources_
  row (the Source sheet); the canvas's _Add Concept ▸_ is _Recent_, the four
  forms, _Quantities ▸_, _More…_; _Add Source ▸_ is _New source…_ alone.
- **A concept is created on the concept sheet** with a name given first:
  choosing a category anywhere — the menu, a Library row, a drag onto the
  canvas, the Project tab's `+` — opens the sheet (name, category, _Measured
  in_, meaning, a preview of the node and its declaration); _Create_ is one
  `CreateConcept`; the concept lands where asked, selected and named. Nothing
  opens for renaming. _Measured in_ lists the compiler's units for the
  category's dimension (`rad`, `deg`, `turn`; `rad/s`, `deg/s`), preferred
  first, as a fact: no unit is chosen or stored (ISS-0019). A taken or
  unspellable name is said on the sheet before the daemon would refuse it.
- **The Formula view is typed structure**: a structural caret (a thin bar
  between parts, inside a slot, inside a leaf's text) that ← → ↑ ↓, Home, End,
  Tab / ⇧Tab, `)` and `,` move along the compiler's tree; letters, digits and a
  space after a number type at the caret as text the compiler reads;
  `+ − * / < > = & | !` put the operator on the part the caret touches with a
  slot for the other side; `(` after a name applies it (`clamp` →
  `clamp(?, ?, ?)`); ⌫ / ⌦ delete a character or a whole part; completion opens
  while typing (and on ⌃Space) ranked by what the position expects; a click
  places the caret and selects the part, and the palette acts on the selection
  as before. The part being typed into shows as text until the compiler has read
  it; everything else keeps its structure. `/` is drawn as a fraction; `match`,
  `let` blocks, rules, collections, groups, `delay` and `sync` are structure
  (rows, a spine, a bar), no longer monospace text; a composite unit is drawn as
  `m/s²`. Every part has a reading for assistive technology.
- **A saved formula unfolds on its node**: a disclosure at a relationship's
  definition line (or _Show Formula_ in its menu) extends the node with the
  formula rendered read-only, its first finding and _Edit Formula_. Which
  formulas are open is view state, never saved.
- **Protocol 0.28**: `ComposeAction.apply` — a reference naming an equation or a
  rule becomes a call with one slot per argument, the first selected (`clamp` →
  `clamp(?, ?, ?)`; `spin` → `spin(?)`); refused (`formula.not_applicable`) on a
  value, a literal or a name the compiler does not know as callable. Text
  completion inside a formula (`CompleteDefinitionDraft`) ranks by the expected
  type of the position the caret is in (`? / cycleTime` offers the lengths
  before the speed limit) — the same ranking the slot panel had, now at a byte
  offset.
- **Libraries from the environment**: `bdld` loads every file named in
  `BDL_LIBRARIES` (path-separated) beside the standard library; a file that
  fails to load is reported on stderr and skipped. The daemon's tests carry
  their product presets that way
  (`crates/bdl-daemon/tests/fixtures/presets.toml`).

## Compatibility and migration

- Designers: nothing in an existing project changes — a concept created from
  _Motor Angle_ is an `Angle` concept and stays one; the Library shows
  categories instead of presets, and a search for the old words finds the
  category. Creating a concept now asks for its name first.
- Library authors: schema 2 files load unchanged; a third-party library that
  ships product presets or Source items is shown in its own section, and its
  Concept items open the same sheet with their defaults. Set `BDL_LIBRARIES` to
  serve it.
- Protocol clients: 0.28 is additive (`apply`); a 0.27 client is compatible. The
  standard library no longer serves any `source`-category item or any
  `LibraryItemView.preset`; a client that listed the Source presets shows an
  empty list and should offer its generic Source path.
- Developers: `bdl_ide::ComposeOp::Apply { node }`,
  `bdl_ide::completion::positional_expectation`,
  `bdl_library::LibrarySet::from_environment`; Studio's `app/caret.dart`
  (`caretStops`, `KeyPlan`), `ui/formula_render.dart` (`FormulaRender`,
  `FormulaGeometry`, `describeNode`), `ui/concept_sheet.dart`,
  `ui/expanded_formula.dart`;
  `EditorState.{conceptSheet, expandedFormulas, formulaPreviews}`,
  `AppState.valueCategories`; `test/support/carets.dart` is gone (the stops come
  from the tree).

## Evidence

`crates/bdl-library` unit tests (the 22 items, their shape, the search synonyms,
the fixture library), `crates/bdl-ide/tests/formula_composer.rs`
(`apply_turns_a_name_into_a_call_with_the_compilers_arity`,
`text_completion_ranks_by_the_positions_expected_type`),
`crates/bdl-daemon/tests/{stdio_e2e,text_e2e,system_e2e}.rs` (the fixture
library over `BDL_LIBRARIES`); `apps/studio/test/caret_test.dart` (the stops,
every key), `concept_sheet_test.dart`, `library_test.dart`,
`library_e2e_test.dart` (the categories from `bdld`, the sheet, one edit),
`formula_composer_test.dart`, `formula_keyboard_e2e_test.dart` (tasks C, E, F, G
against the real `bdld`, with their tallies), `expanded_formula_test.dart`.
