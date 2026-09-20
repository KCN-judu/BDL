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

The monograph's surface vocabulary (Part II, _Time_; designed, not offered) has
temporal modifiers (`for`, `after … by`, `while`, `until`) and contexts
(`StateHandler`) that it does not execute; BDL has reserved the words `context`
and `require` and elaborates none of them.

## Why it matters

They are the design's answer to _when_ a behaviour holds; without them timing is
expressed only through domains and `delay`/`sync`.

## Current evidence

- Production: `docs/spec/textual-syntax.md` §12 (reserved words), `bdl-elab`
  refuses contexts with their own clock domain (DI-6).
- Original entries: `docs/03-open-questions.md` §C (modifier list and shapes,
  context representation).

## Dependencies

- A surface shape for each modifier and its desugaring into the existing Core
  (no kernel change) or an issue per modifier if one needs more.

## Resolution

Open.
