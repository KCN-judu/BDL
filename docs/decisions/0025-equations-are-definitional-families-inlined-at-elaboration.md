---
id: ADR-0025
status: accepted
date: 2026-09-18
area: language
supersedes: []
superseded-by: []
related:
  ["ADR-0001", "ADR-0010", "ADR-0013", "ADR-0011", "ISS-0012", "ISS-0005"]
fv:
  [
    "formally proved (model): matching is sound and complete
    (`Poly.matchTy_sound`, `matchTy_complete`, `Scheme.instantiate_sound`);
    every library entry types at its scheme (`Stdlib.*_typed`) and adds no
    privilege (`lib_expansion`, `Comb.noConstruct`, `lib_clocked`); nominality
    and dimensions survive generics (`Generic.generic_preserves_identity`,
    `generic_preserves_dimension`); `fold_total`; `Cap.eq_iff_data`,
    `Cap.ord_data`, `lt_rejected`, `min_mode_rejected` (Phase 9c)",
    "design recommendation only: the surface vocabulary {Data, Eq, Ord} and the
    minimality of the basis (`POLYMORPHIC_EQUATION_LANGUAGE_NOTE.md` §8)",
  ]
---

# ADR-0025: Equations are definitional families inlined at elaboration; the kernel stays monomorphic; order is a declaration, not a property of data

## Status

Accepted (2026-09-18, the equation-library milestone; consumes BDL_FV Phases
9a–9c, `fad79d9`, `ee2862e`, `6a8ee96`).

## Context

Designers need reusable equations over finite collections, grouped values,
optional values and ranges — `min`, `clamp`, `any`, `all`, `map`, membership —
that keep nominal identity and dimensions. The formal development found the
smallest basis that supports every tested case: products and one list recursor
in the kernel, structural equality at every data type, ordering on quantities
only, and rank-1 polymorphism realised as families of monomorphic terms
instantiated by first-order matching against closed types. It also found (Phase
9c) that "is data" does not imply "has an order": a mode encoded as a number, a
pair, a list, an optional value have equality and no order.

## Decision

- **The kernel** (`bdl-ir`) gains exactly `list`, `prod`, the list and pair
  operators, `eq` at any data type, and `fold`; `lt` compares quantities only.
  No type variable, constraint, class, set, record, interval or sum type enters
  it.
- **An equation is a definitional family** (`bdl-equations`): a scheme over type
  and dimension variables with the closed capability vocabulary {Data, Eq, Ord},
  and one closed Core builder per instance. A use is resolved by matching the
  scheme against the arguments' closed kinds in order and inlining the built
  term; the checker then types it like any term. No unification of open types,
  no generalisation, no inference of a scheme from a body; public helpers carry
  explicit schemes.
- **Order is by declaration.** `Ord` holds for a quantity and for a concept the
  designer declared ordered (`Concept::ordered`, `ordered concept C : …`)
  represented by a quantity; such a concept compares through its representation
  and keeps its identity. It is never inferred from the value form. Comparators
  (`minBy`, `maxBy`) are the escape hatch and need no declaration.
- **Two concept values compare as concepts** (`==` structural on the same
  concept; `<` only when ordered; another concept is a nominal error). A concept
  beside a plain value of its own representation is observed and compared as
  that representation, as arithmetic has always been (ADR-0013).
- **Rules** (`x => e`) exist only as arguments to equations and are inlined;
  they are never values, and the higher-order boundary of ADR-0013/DI-24 stays
  closed at the surface and in lowering.

## Alternatives

- A `map`/`filter`/`any` primitive each: rejected — one recursor derives them
  all and keeps the kernel small (D-89).
- Structural ordering on every data type (Phase 9b's first form): rejected on
  the Phase 9c audit — `mode1 < mode2`, `None < Some x`, lexicographic pairs
  have no behaviour-design meaning (D-98).
- Inferring order from a numeric representation: rejected — a code is not a
  magnitude.
- User-defined type classes, dictionaries, System F, higher-rank types: rejected
  in the formal development; no tested case needs them (D-92, D-93).
- Library entries as declarations of the design: rejected — they would be
  monomorphic and enter the dependency graph (D-94).

## Consequences

- Elaboration observes concept values inside collections and grouped values
  where an equation or the produced concept's form needs their representations;
  the mixed concept-beside-plain-value case bypasses the order declaration
  (ISS-0012).
- A relationship named like an equation shadows it.
- Records changed: `docs/spec/equation-library.md` (new),
  `docs/spec/textual-syntax.md` §15, `docs/spec/kernel.md` §9b,
  `docs/architecture/ir.md`, `docs/project/formal-correspondence.md`,
  `docs/project/status.md`, the change fragment.
