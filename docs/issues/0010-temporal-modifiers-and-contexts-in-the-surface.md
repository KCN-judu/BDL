---
id: ISS-0010
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/03-open-questions.md"]
---
# ISS-0010: Temporal modifiers and contexts in the surface

## Problem

The paper's surface vocabulary has temporal modifiers (`for`, `after … by`,
`while`, `until`) and contexts (`StateHandler`) that the paper does not
execute; BDL has reserved the words `context` and `require` and elaborates
none of them.

## Why it matters

They are the paper's answer to *when* a behaviour holds; without them
timing is expressed only through domains and `delay`/`sync`.

## Current evidence

* Production: `docs/TEXTUAL_SYNTAX.md` §12 (reserved words), `bdl-elab`
  refuses contexts with their own clock domain (DI-6).
* Original entries: `docs/03-open-questions.md` §C (modifier list and
  shapes, context representation).

## Dependencies

* A surface shape for each modifier and its desugaring into the existing
  Core (no kernel change) or an issue per modifier if one needs more.

## Resolution

Open.
