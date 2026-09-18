---
id: ADR-0026
status: accepted
date: 2026-09-18
area: language
supersedes: []
superseded-by: []
related: ["ADR-0013", "ADR-0025", "ISS-0012"]
fv:
  [
    "informed by FV: BDL_FV Phase 9c (`Poly.lean`, `Ty.ordB`) puts the order
    policy on comparisons *between concept values*; `rep` is free everywhere and
    `lt d (rep a) (rep b)` is a legal kernel term whatever `OrdDecl` says",
  ]
---

# ADR-0026: A concept value beside a plain value of its representation is observed; the order declaration governs comparisons between concept values

## Status

Accepted (2026-09-18, the P9 hardening pass; closes ISS-0012).

## Context

Phase 9c made order a declaration: `b1 < b2`, `min(b1, b2)`, `clamp(b1, lo, hi)`
between values of a concept are legal only while the concept is declared
ordered, and a numeric encoding is never a magnitude (`Mode < Mode` is refused
however `Mode` is represented). A concept value beside a _plain_ value of its
own representation — `tilt < 10 deg`, `min(brightness, 0.5)`, `mode < 3` — has
always been observed and compared as that representation (ADR-0013, the
arithmetic rule), so the declaration is not consulted on that path. ISS-0012
asked whether that is coherent, and listed three answers: order a
quantity-represented concept by default (contradicting "never inferred"),
require the declaration for every mixed comparison (every physical design writes
`tilt < 10 deg` today), or keep the mixed case a representation comparison by
design.

## Decision

The mixed case stays a representation comparison, as a rule with three clauses,
tested as a matrix (`crates/bdl-elab/tests/equations.rs`,
`mixed_comparisons_over_overlapping_representations`):

- Two values of **one concept** compare as that concept: `==` always; `<`,
  `min`, `max`, `clamp`, `inRange` only while the concept is declared ordered
  (`semantic.no_order` otherwise); `minBy`/`maxBy` with a rule always.
- Two values of **different concepts** never compare, whatever their
  representations (`semantic.concept_mismatch`), and never through an equation.
- A concept value beside a **plain value of its representation** is observed
  (`rep`), and the comparison is the representation's: the designer wrote a
  number, which is a statement about the representation. The result of `min`,
  `max`, `clamp` in that case is the plain representation, re-wrapped only by a
  mapping whose declared result is that concept — as arithmetic has always been.

The order declaration therefore answers exactly one question — _may two values
of this concept be put in order?_ — and nothing else. A declaration is needed
only where the representation could not answer for the designer (a concept
against itself); a plain value beside a concept carries the designer's own
answer.

## Alternatives

- **Order by default for quantity-represented concepts**: rejected — it is the
  inference Phase 9c forbids, and it would make `Mode < Mode` legal the moment
  `Mode` is a number.
- **The declaration for every mixed comparison**: rejected — a migration of
  every existing design for no gained meaning: `tilt < 10 deg` is unambiguous.
- **Refuse `min(o, 0.5)` while `Opacity` is unordered but allow `o < 0.5`**:
  rejected — one rule for `<` and the equations that are defined by it (`min`,
  `max`, `clamp` are `lt` at the instance, ADR-0025).

## Consequences

- `docs/spec/equation-library.md` §Mixed comparisons states the rule; ISS-0012
  is resolved by this record.
- The one remaining asymmetry is visible and intended: `min(o1, o2)` is refused
  while `min(o1, 0.5)` is not. Explain shows the observation (`rep o1`) so a
  designer can see which comparison was made.
- Records changed: `docs/spec/equation-library.md`, `docs/issues/0012-…`,
  `docs/project/status.md`, the change fragment `2026-09-p9-hardening.md`.
