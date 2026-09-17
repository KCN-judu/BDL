---
id: ADR-0013
status: accepted
date: 2026-09-15
area: language
supersedes: []
superseded-by: []
related: []
fv: []
---
# ADR-0013: Formula language v0 — names, rep/mk insertion, units, crate split

**Status**: accepted (2026-09-15)

## Context
The first compiler slice needs a designer-facing formula syntax that
elaborates into the kernel's Core terms without exposing `rep`/`mk`,
de Bruijn indices or dimension-indexed primitives.

## Decisions
1. **Syntax** is a small expression language (`+ - * / < <= > >= == != && ||
   ! if-then-else`, names, numbers with an optional unit, booleans,
   parentheses), parsed by a hand-written Pratt parser in `bdl-syntax`. No
   loops, mutation or user functions. The surface AST carries spans and is
   never reused as Core IR.
2. **Input names are the concepts' display names** at analysis time
   (exact match, else unique case-insensitive match). Resolution yields an
   index into the signature; the Core term holds only de Bruijn indices.
   The formula text is stored as written and re-resolved on every analysis,
   so renaming a concept makes a formula that still uses the old name report
   `formula.name.unknown` with the current names as fixes. A concept of the
   project that is not an input is reported as `formula.name.not_an_input`.
   An explicit parameter-name layer was judged unnecessary now; `Signature`
   is unchanged.
3. **Automatic `rep`/`mk`**: inputs appear in the formula as their
   representation (`rep (var i)`); the whole formula is wrapped in
   `mk B`, which the declaration's own signature grants. The elaborator
   never emits `mk` of any other concept; the checker refuses one anyway.
4. **Units** are surface: `90 deg` becomes `lit[rad] (90·π/180)`. Linear
   scaling only; the table lives in `bdl-elab::units`.
5. **Where dimensions are checked**: the elaborator infers representation-
   level types to choose primitive instances (`add[rad]` vs `add[s]`) and
   reports mismatches with spans in product language; `bdl-check` then
   re-derives the Core term's type under the grant and is the authority.
   Nothing about dimensions is a separate pass.
6. **Crates**: `bdl-diagnostics` (shared vocabulary), `bdl-syntax`,
   `bdl-elab`, `bdl-check`, `bdl-compiler` (`analyze(snapshot)`), each a real
   boundary with the dependency direction
   `model → ir → {syntax → elab, check} → compiler → protocol → daemon`.

## Consequences
* Formulas are text in the project file; nothing semantic is stored
  pre-elaborated, so a compiler change re-analyses everything.
* Boolean equality is encoded (`(a∧b)∨(¬a∧¬b)`) because the kernel has no
  boolean `eq`; count (`nat`) arithmetic is unsupported until the kernel
  gains `nat` primitives (DESIGN_ISSUES DI-12).
