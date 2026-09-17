---
id: ISS-0003
state: open
area: formal
opened: 2026-09-15
resolved-by: []
related: ["docs/DESIGN_ISSUES.md#di-5", "docs/03-open-questions.md"]
---
# ISS-0003: Interface-level references and the evidence model

## Problem

The kernel's interfaces carry *commitments* (`PropertyId`) whose evidence
relation distinguishes monotone from goal-relative evidence; commitments
that mention other declarations are not modelled, and production carries the
`commitments` vector without checking any of it.

## Why it matters

Invalidation today follows realization dependencies only; a commitment
that names another declaration would need its own dependency edge, and
declared (author-asserted) evidence would need to be told apart from
computed evidence in Studio.

## Current evidence

* Production: `Interface { expected_type, commitments }` in `bdl-ir`; the
  checker builds interfaces with `commitments: vec![]` (`crates/bdl-check/src/typing.rs`);
  no `Evidence` relation exists.
* Formal: `BDL/Core/Satisfaction.lean`, `BDL/Core/Interface.lean`.
* Original entries: DI-5, and `docs/03-open-questions.md` §B (the concrete
  `PropertyId` set, the two kinds of evidence) and §D (declared vs computed
  evidence in the UI).

## Dependencies

* A decision on the first concrete `PropertyId`s and how evidence is
  produced (a checker pass, a declaration by a component author, a test).

## Resolution

Open.
