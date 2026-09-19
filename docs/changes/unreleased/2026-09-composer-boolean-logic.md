# Boolean logic and choices in the Formula Composer

- Date: 2026-09-20
- Area: ide, studio, protocol
- Affected: designers, protocol clients, developers
- Related: ADR-0028, ADR-0001

## What changed

- **A choice is structure.** `if c then a else b` projects as a node of kind
  `if` with three children — the condition (expects true or false), the two
  outcomes (expect what the choice gives: the position's expectation, else what
  the other outcome already is) — no longer an opaque region. The Formula view
  draws it as `if` and its condition over indented `then` / `else`, every part
  selectable and fillable.
- **The logical operators are offered.** `&&`, `||` (a slot after the component)
  and `!` (prefix, in place, no slot) join the palette as the words _and_, _or_,
  _not_, shown only where the component may be a truth value; the keys `<`, `>`,
  `=` (`==`), `&`, `|`, `!` join `+ − * /`; a click now takes the focus so the
  keys act on the clicked component. **Choose** wraps a component as the `then`
  outcome of `if ? then … else ?` or opens one in a slot.
- **A truth-valued slot offers `true` and `false`**
  (`FormulaSlotResponse.booleans`) in place of the number entry, and the
  references that produce a truth value — the relationship's own inputs first,
  the expected concept itself before a value of the same kind.
- **Parenthesisation follows the parser's ladder** (`||` < `&&` < `==` `!=` <
  `<` `<=` `>` `>=` `in` < `..` < `??` < `+ −` < `* /` < unary): the composer
  previously ranked `&&` and `||` equal, so `a || b` under `&&` lost its
  grouping. A comparison under a comparison is parenthesised on either side
  (comparisons do not chain); a component that ends in a choice or a binder is
  parenthesised before anything written after it. Removing an empty operand of
  `&&` / `||` / a comparison removes the operator, as for arithmetic; an empty
  negation is its slot.
- The Formula and Text views now express the same set of forms for logic, and
  switching loses nothing:
  `if RoomTemp > 299.15 K && ButtonHeld then true else false` is built from an
  empty formula by structured actions alone and comes back fully structured when
  typed.

## Compatibility and migration

- Designers: nothing to do; `if` formulas that were edited as text are now
  assembled in place. `match`, a block with `let`, a rule, a collection or
  grouped literal, `delay` / `sync` stay text in the Formula view.
- Project files: nothing.
- Protocol clients: protocol **0.17**, additive without a bump —
  `FormulaNode.kind` may be `if`, `FormulaSlotResponse.booleans` is new,
  `ComposeAction.choose` is new, `ComposeAction.operator { op: "!" }` is the
  prefix form. An older client sees `if` as an unknown kind and ignores the new
  field.
- Developers: `bdl_ide::NodeKind::If`, `ComposeOp::Choose`, `SlotInfo.booleans`;
  `compose` with `Operator { op: "!" }`.

## Evidence

`crates/bdl-ide/tests/formula_composer.rs` (`a_choice_is_a_structured_node…`,
`a_boolean_slot_offers_true_and_false…`, `the_air_conditioner_walkthrough…`,
`negation_choice_and_the_logical_operators…`, the round-trip properties over
`if`, `!`, `&&`, `||`), `apps/studio/test/formula_composer_test.dart`,
`apps/studio/test/formula_composer_e2e_test.dart` (the air conditioner against
`bdld`).
