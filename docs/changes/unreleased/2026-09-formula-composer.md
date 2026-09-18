# The Formula Composer foundation (P10a); protocol 0.12

- Date: 2026-09-18
- Area: language, textual, ide, protocol, daemon, studio
- Affected: designers, protocol clients, developers
- Related: ADR-0028, ISS-0004

## What changed

- **A slot.** `?` is an expression not yet written: `tilt / ?` parses, formats
  as itself, and is refused by elaboration with `formula.slot.empty` (_This slot
  is empty; it expects an angle._). A formula with a slot is an incomplete
  definition, like a declared relationship; it may be saved and is invalid until
  filled. `docs/spec/textual-syntax.md` §16.
- **Units.** The registry (`bdl-elab::units`) gives every unit a stable id, a
  symbol, a dimension and a linear chart; `convert` is the one conversion
  operation. New units: `turn`, `inch`, `ft`. °C and °F stay unregistered
  (affine; ISS-0004).
- **The IDE service** answers the Formula Composer: the projection of a formula
  (the surface tree with the elaborator's types, expected types by local
  dimension inference, diagnostics placed on nodes), what a slot expects and
  what fits (units of the solved dimension, references by type, equations by
  scheme and capability), and the text a structured action makes.
- **Protocol 0.12** (additive): `GetFormulaProjection`, `GetFormulaSlot`,
  `ComposeFormula`; `DefinitionDraftAnalysis.projection`; the refusal
  `formula.not_applicable`.
- **Studio**: the relationship inspector's definition editor has a **Formula |
  Text** switch. Formula (the default) draws the compiler's projection as the
  expression — reference chips with socket glyphs, literals as a coordinate and
  a unit pop-up, dashed slots, operators, calls, unsupported forms as text —
  and, for the selected component, what it expects and why, with a number entry
  (units of the expected kind), references, folded equations, or the operators /
  Compare / Function / Remove. Every action becomes a compiler answer put into
  the draft; save, revert, conflicts and diagnostics are unchanged. Text that
  does not parse keeps the exact text and offers _Edit as text_.

## Compatibility and migration

- Designers: the editor opens in the Formula view; **Text** is one click away
  and everything typed before is unchanged. A `?` may now appear in a formula
  and in a saved `.bdl` file; it is an incomplete definition, not an error of
  the file.
- Project files: nothing changes on disk except that `?` is legal in a formula.
- Protocol clients: 0.12 is additive; a 0.11 client works against a 0.12 daemon
  and never sees the new fields.
- Developers: `bdl_elab::trace_formula_in` and `bdl_ide::formula` are new;
  `bdl_elab::units::UnitDef` gained `id` and `chart()`; `CompletionKind` and the
  existing draft path are unchanged; existing Studio tests that type into the
  text field start in Text mode (`ComposerState(formulaMode: false)`).

## Evidence

`crates/bdl-ide/tests/formula_composer.rs`,
`crates/bdl-syntax/src/parser/tests.rs`
(`a_slot_parses_wherever_a_value_may_stand_and_formats_as_itself`),
`crates/bdl-elab/src/units.rs`
(`ids_are_unique_and_conversion_preserves_the_quantity`),
`crates/bdl-daemon/tests/stdio_e2e.rs` (`definition_drafts_over_stdio`),
`apps/studio/test/formula_composer_test.dart`,
`apps/studio/test/formula_composer_e2e_test.dart`;
`docs/user-guide/assets/studio/formula-composer.png`; commits `7644d40`,
`8dc014b`, `fac0c91`.
