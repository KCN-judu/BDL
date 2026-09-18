---
id: ADR-0028
status: accepted
date: 2026-09-18
area: studio
supersedes: []
superseded-by: []
related:
  ["ADR-0001", "ADR-0013", "ADR-0017", "ADR-0023", "ADR-0025", "ISS-0004"]
fv:
  [
    "informed by FV: BDL_FV Phase 10 (`Surface/Composer.lean`,
    `Surface/Units.lean`, 7f80e88) — partial expressions with holes, local
    bidirectional dimension inference (`solve_sound`, `solve_complete`),
    candidate units sound and complete relative to the registry
    (`candidates_sound`, `candidates_complete`); a unit is elaboration data,
    never a kernel construct",
  ]
---

# ADR-0028: The Formula Composer is a projection of the draft text, computed by the IDE service; a slot is surface syntax, never a kernel term

## Status

Accepted (2026-09-18, Production Phase P10a — Structured Formula Composer
Foundation).

## Context

The Studio definition editor was a text field over the compiler's verdict
(ADR-0013, `docs/architecture/studio-ui.md` §4a). A structured authoring surface
was wanted — assemble `clamp(tilt / (90 deg), 0, 1)` without typing, with the
compiler saying what each slot expects and offering only what fits — under the
standing constraints: one authored source (the formula text), no parser, type
checker, dimension algebra or equation matcher in Dart (ADR-0001), no second
formula store or persistence, no new kernel feature. The formal development
(Phase 10) gives the exact fragment: partial expressions with typed holes, a
one-pass local `solve` over the dimension group, candidates from the unit
registry — and says holes and inference belong to the surface editor, not the
kernel.

Two shapes were possible for the incomplete state. An **authoring overlay** in
the IDE host holding a compiler-owned partial expression beside the draft text,
mutated by structured operations and rendered to text only once complete; or a
**slot in the text** — a surface token `?` that parses wherever a value may
stand and is refused by elaboration — so the incomplete formula _is_ the draft
text and every structured action is a byte-range edit of it.

## Decision

- **The Composer is a view.** `bdl-ide::formula` builds a `FormulaProjection`
  from the effective definition text — the surface tree of `bdl-syntax` and the
  typing trace of `bdl-elab` — on every request, and never stores it. Node
  identity is the tree path, stable within one draft generation; it is not an
  entity and does not enter `.bdl/identities.json`.
- **A slot is surface syntax.** `?` is a `SlotExpr` of the textual grammar
  (`docs/spec/textual-syntax.md` §16), lowered to `ExprKind::Hole`, refused by
  elaboration with `formula.slot.empty`; it never reaches Core, the checker, the
  evaluator or generated code. A formula with a slot is a definition that does
  not check — the same state as any unfinished draft — and may be saved as such.
- **Every structured action is a text edit.** `compose` answers a `ComposeOp`
  (fill, operator, call, set unit, set coordinate, remove) with byte-range edits
  of the draft and the whole new source; Studio puts it into the draft through
  `DefinitionDraftChanged`, and the ordinary `AnalyzeDefinitionDraft` follows.
  One commit path, one conflict model, one set of diagnostics.
- **Inference and candidates are the compiler's.** Expected types come from
  `solve` (Composer.lean's rules, no search); units from the registry
  (`units_for`); references from the design by type; equations from P9's scheme
  matcher and capability check. Studio stores the mode, the selection and the
  open pop-up — never a parsed tree, an inferred dimension or a unit rule.
- **Units are a registry, conversion is one operation.** A unit has a stable id,
  a symbol, a dimension and a chart onto the canonical magnitude
  (`bdl-elab::units`); `convert(x, from, to)` is the only conversion operation
  and hides the chart's shape. Linear charts only today; an affine chart is a
  new variant of the same enum, not a new API. The Composer's unit pop-up
  belongs to a literal (a quantity the designer authored) and switches its unit
  with the quantity kept; a reference's kind is its declaration's and is never
  rewritten.

## Alternatives

- **An authoring overlay of partial expressions**: rejected — a second
  representation of the formula to keep in step with the text, invisible in the
  Text view, with its own persistence question and its own protocol of
  operations; the `?` in the text costs one token and keeps every existing path
  (drafts, conflicts, save, LSP) untouched.
- **Building the tree in Dart from the projection**: rejected — precedence,
  parenthesisation and unit conversion would move into Studio (ADR-0001).
- **A kernel hole term**: rejected — the kernel has no notion of an unfinished
  expression and needs none; the formal development places holes in the editor.
- **`in` as the inch symbol**: rejected — `in` is the membership keyword; the
  registry spells it `inch`.

## Consequences

- Text and Formula stay synchronized by construction: there is nothing to
  synchronize.
- A saved `.bdl` may contain `?`; loaders and the formatter accept it; the
  design is incomplete there exactly as with a declared relationship.
- Affine units (°C, °F) wait for the formal point/difference follow-up
  (ISS-0004); nothing in the protocol or Studio interaction assumes scale-only
  conversion.
- Records changed: `docs/spec/textual-syntax.md` §6, §16;
  `docs/spec/protocol.md` (0.12); `docs/architecture/ide-service.md`,
  `docs/architecture/studio-ui.md` §4b; `docs/project/status.md`,
  `docs/project/formal-correspondence.md`;
  `docs/user-guide/studio/formula-editor.md`; the change fragment
  `2026-09-formula-composer.md`.
