# Collections, grouped values and the equation library; protocol 0.11

- Date: 2026-09-18
- Area: language, compiler, runtime, codegen, textual, protocol, ide, studio
- Affected: designers, project authors, protocol clients, developers
- Related: ADR-0024, ADR-0025, ISS-0011, ISS-0012, ISS-0013

## What changed

- **The data core** now has finite collections (`List<R>`), grouped values
  (`Pair<R₁, R₂>`) and optional values (`Option<R>`) as value forms of a
  concept, in formulas as `[a, b]`, `(a, b)`, `Some(e)`/`None`; all are data:
  remembered, transported, compared. `docs/spec/equation-library.md` §1.
- **The equation library** — `min`, `max`, `clamp`, `inRange`, `inInterval`,
  `minBy`, `maxBy`, `foldr`, `any`, `all`, `contains`/`oneOf` (`x in [a, b]`),
  `map`, `filter`, `append`, `sum`, `zip`, `optElim`, `mapOpt`, `getOrElse`,
  `length`, `head`, `reverse`, `take`, `drop`, `cons`, `toList`, `pair`,
  `first`, `second` — is callable from any formula; a rule (`x => …`) is given
  to `any`, `all`, `map`, `filter`, `foldr`, `minBy`, `maxBy`, `optElim`,
  `mapOpt`. A relationship of the design with the same name wins.
- **Equality** works on every data kind and on values of one concept;
  `Brightness == Opacity` is refused (`semantic.concept_mismatch`). Number
  patterns on counts work.
- **Order is a declaration.** `<`, `min`, `max`, `clamp`, `inRange` between two
  values of a concept need `ordered concept C : …` (text) or the _Order_
  checkbox (Studio); `Mode < Mode`, `Pair < Pair`, `List < List`,
  `None < Some(x)`, `Bool < Bool` are refused with `semantic.no_order`. A
  concept beside a plain value still compares by representation (ISS-0012).
- **Text**: keywords `in` and `ordered`, tokens `[` `]`, the forms of
  `docs/spec/textual-syntax.md` §15; a concept named `in` or `ordered` is no
  longer an identifier.
- **Generated Rust**: a design that carries a collection is generated with the
  runtime's `collections` feature (`alloc::Vec`, last element first), derives
  `Clone` instead of `Copy`, and needs an allocator on its target; the manifest
  says `requires_allocator` (ADR-0024). Every design's core now reads
  declarations and cells by clone (`read_decl(&x, …)`); a design without
  collections is otherwise generated as before.
- **Protocol 0.11** (additive): `Representation.optional` / `list` / `pair`,
  `ConceptView.ordered`, `EditOp.set_concept_ordered`, `Value.list` / `pair`,
  the draft-completion kind `equation`.
- **Studio**: three new socket shapes (⧉ collection, ▯ grouped value, ◎ optional
  value), the concept inspector's value-form pop-up with nested forms and the
  _Order_ checkbox, structured simulation inputs written as
  `[…]`/`(…)`/`none`/`some(…)`; equations complete in the definition editor with
  their meaning.
- **IDE / LSP**: equation completions (`CompletionKind::Equation`), the
  semantic-token type `macro` for an applied equation, `parameter` for a rule's
  inputs.

## Compatibility and migration

- **Designers / project authors.** Existing projects open unchanged; nothing is
  ordered until declared, and no existing formula needs the declaration (a
  concept beside a plain value compares as before). A project that named a
  concept or relationship `in` or `ordered` must rename it (the loader reports a
  syntax fault at the item).
- **Protocol clients at 0.10** keep working (`major` unchanged). A client that
  switches on `Representation.kind` must accept `optional`, `list` and `pair`;
  one that switches on `Value.kind` must accept `list` and `pair`.
- **Developers.** `bdl_model::Representation` is `Clone`, no longer `Copy`;
  `Prim::Eq { dim }` is `Prim::Eq { ty }`; `bdl_reactive::Value::List` holds a
  shared `List`, not a `Vec`; `bdl_runtime_core::read_decl`/`read_input` take
  `&Option<T>`; `EXEC_IR_VERSION` is 2; goldens were regenerated
  (`BDL_UPDATE_GOLDEN=1`).

## Evidence

`crates/bdl-reactive/src/eval.rs` (lists, pairs, fold, equality, temporal
interaction), `crates/bdl-equations/src/lib.rs`,
`crates/bdl-elab/tests/equations.rs` (cases A–N, the negative, nominality,
dimension and performance cases), `crates/bdl-lower/src/lib.rs`
(`folds_lists_pairs_and_function_arguments_lower_first_order_and_agree`),
`crates/bdl-compiler/tests/backend_differential.rs` (`collections`, `buffer`),
`crates/bdl-compiler/tests/golden/{collections,buffer}`,
`crates/bdl-syntax/src/parser/tests.rs`, `crates/bdl-text/tests/workspace.rs`
(`equations_collections_and_ordered_concepts_round_trip_through_text`),
`crates/bdl-ide/tests/acceptance.rs`
(`equations_of_the_library_complete_by_prefix_in_the_designers_words`),
`runtime/bdl-runtime-core` (`collections`),
`apps/studio/test/structured_values_test.dart`.
