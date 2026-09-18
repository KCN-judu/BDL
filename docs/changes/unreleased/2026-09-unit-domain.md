# One canonical type per relationship: the unit domain (ADR-0029)

- Date: 2026-09-18
- Area: language, textual, compiler, ide, studio, protocol
- Affected: designers, developers, protocol clients
- Related: ADR-0029, ISS-0014

## What changed

- **A relationship without inputs has the type `() -> B`**, not `B`:
  `mapping TempSensor : RoomTemp` is shorthand for
  `mapping TempSensor : () -> RoomTemp`, and both are one declaration with one
  canonical type `domain(inputs) -> B`, the domain of no inputs being the empty
  product `()`. Two inputs may be spelled `(A, B) -> C` (= `A -> B -> C`). The
  kernel encodes the canonical type by currying and unit elimination
  (`() -> B` as `B`), so nothing in Core, the evaluator or the generated code
  changes; the unique argument is erased (`bdl_ir::ty`).
- **`()` is the empty product** — as a type it opens a signature, as a value
  it is the argument of a relationship without inputs: `f`, `f()` and `f(())`
  are one reference. Elsewhere `()` is refused (_`()` carries no value here._);
  as a concept's value form, after an input or as an output it is refused in
  those words. Never the word _unit_ (a measurement unit), never `_`.
- **No special category.** Everything that follows from the unit domain — read
  as a value, may drive an output, be a parameter, be transported, hold memory,
  stand as a simulation input when unresolved — asks one predicate
  (`Signature::is_unit_domain`, Studio's `isUnitDomain`). Hover carries
  `type: () -> RoomTemp`; Explain adds `canonical type` and the empty-product
  sentence; the canvas still draws no input socket, and the Composer offers the
  relationship as itself.
- Diagnostics: _f reads nothing: its only argument is `()`._ (fix: _Write f._)
  replaces _it is a value, not something to apply_.

## Compatibility and migration

- Designers: every existing project parses and means what it did; the
  formatter never rewrites `mapping f : B` to `() -> B` or back. Nothing to do.
- Project files: nothing.
- Protocol clients: protocol **0.14**, additive — `TypeView.kind` may be
  `unit`; `Signature` is unchanged (an empty `inputs` is the unit domain).
- Developers: `bdl_ir::Ty::Unit`, `Ty::of_signature`, `Ty::kernel_of_signature`,
  `Ty::canonical_mapping_ty`, `Ty::domain_of`, `Ty::uncurry`;
  `Signature::is_unit_domain`; `bdl_syntax::TypeKind::{Unit, Tuple}`,
  `ExprKind::Unit`, `ast::Type::{Unit, Tuple}`, `ast::Expr::Unit`;
  `pretty::mapping_type`; `TypeKindView::Unit`. Every `Ty` match gains an arm.
- Formal: the normalization is an engineering choice above the Lean kernel;
  ISS-0014 records the theorem to prove.

## Evidence

`crates/bdl-ir/src/ty.rs`, `crates/bdl-syntax/src/parser/tests.rs`,
`crates/bdl-ide-db/src/textual.rs`, `crates/bdl-elab/tests/expressions.rs`,
`crates/bdl-ide/tests/acceptance.rs`.
