---
id: ADR-0029
status: accepted
date: 2026-09-18
area: language
supersedes: []
superseded-by: []
related: ["ADR-0010", "ADR-0013", "ADR-0025", "ISS-0014"]
fv:
  [
    "engineering choice: the kernel (`BDL/Core/Base.lean`) has no unit type and
    types a declaration without inputs at its output; the production canonical
    type `() -> B` is a normalization above the kernel, encoded into it by unit
    elimination — ISS-0014 asks for the theorem",
  ]
---

# ADR-0029: A relationship has one canonical type, `domain(inputs) -> B`, whose domain for no inputs is the empty product `()`; the kernel encodes it by currying and unit elimination

## Status

Accepted (the Unit-domain normalization pass, 2026-09-18).

## Context

A relationship declared `mapping f : B` was a special case at every layer: the
model documented it as "a value declaration of type `B`", the elaborator, the
IDE, the system layer and Studio each tested `inputs.is_empty()` for their own
reasons (a value to reference rather than call, a simulation input when
unresolved, a legal driver of an output, a legal parameter port, a legal
transport end, no `delay` under inputs), and no record said what the type of
such a relationship _is_. The formal kernel types a declaration at its
`expectedType`, which for no inputs is the output type `B` — the kernel has no
unit type, its arrows are binary, and `delay` is typed only outside every binder
(`Typing.lean`), so a realization of `B` can never be a lambda over a unit
argument without changing the reactive semantics of memory.

## Decision

- A relationship `(A₁, …, Aₙ) -> B` over concepts has **one canonical type**,
  `domain(inputs) -> B`, with `domain([]) = ()` (the empty product,
  `Product([]) ≅ ()`), `domain([A]) = A`, `domain([A, B, …]) = A × (B × …)`.
  `mapping f : B` is shorthand for `mapping f : () -> B`; both have the type
  `() -> B`. `bdl_ir::Ty::Unit` is that empty product; `Ty::of_signature` is the
  canonical type.
- The **kernel interface** (`DeclInterface.expectedType`, what a realization is
  checked against and what `declRef` has as a term) is the canonical type under
  two isomorphisms the kernel works with directly: currying
  (`(A × B) -> C ≅ A -> B -> C`) and unit elimination (`() -> B ≅ B`).
  `Ty::kernel_of_signature` and `Ty::canonical_mapping_ty` are the two
  directions and are inverse over signatures. `Ty::Unit` never appears in a Core
  term, a representation or a runtime value; the generated core erases the
  unique argument (a zero-argument accessor) — representation erasure, not a
  semantic exception.
- The surface spells the empty product `()` — as a type it opens a signature
  (`mapping f : () -> B`), as a value it is the argument of a relationship
  without inputs (`f(())`, the same as `f` and `f()`); a product domain
  `(A, B) -> C` is `A -> B -> C`. Never the word _unit_ (a measurement unit in
  BDL), never `_`. The formatter keeps the spelling authored.
- A relationship without inputs is **not a category**: it keeps its identity,
  realization, timing domain, dependencies, commitments and roles. Everything
  that follows from its domain being `()` — it is read as a value, it may drive
  an output, be a parameter, be transported, hold memory, stand as a simulation
  input when unresolved — is asked of the signature through one predicate
  (`Signature::is_unit_domain`, Studio's `isUnitDomain`), never of a separate
  kind. On the canvas the unit domain is drawn as no socket; the Composer offers
  the relationship as itself; Explain shows `canonical type: () -> B` and the
  empty-product sentence.

## Alternatives

- **Elaborate `() -> B` literally**: realizations as `λ(). body`, references as
  `app (declRef f) ()`, `Ty::Unit` and a unit literal in Core. Rejected: `delay`
  and `sync` are typed only outside binders and memory is a property of the
  declaration evaluated once per activation; a lambda realization would be
  re-applied per reference and would depart from `Reactive.lean`. The kernel is
  the authority (ADR-0010); the normalization lives above it.
- **A product domain in the kernel encoding** (`prod A B -> C`): would change
  every realization, call and the Lean correspondence for no semantic gain;
  currying is the equivalent encoding the kernel already uses.
- **A mapping-only pseudo-unit** (a `Domain` enum beside `Ty`): a second type
  language for one case. Rejected; `Ty::Unit` is a real type with the ordinary
  predicates (data, sem-free, no grant).
- **Collapse to a value declaration**: loses the mapping's roles and identity
  (§10 of the brief); rejected.

## Consequences

- `docs/spec/textual-syntax.md` §4.1–4.3 and §11 record the shorthand law, the
  `()` and `(A, B)` spellings and the kernel encoding; `docs/spec/kernel.md`
  notes `Ty::Unit` as a production extension beyond the Lean `Ty`; ISS-0014 asks
  the formal development for the theorem that the normalization makes no typing,
  evaluation or clock distinction.
- Diagnostics say `()`: _f reads nothing: its only argument is `()`._, _`()`
  carries no value here._; hover carries `type: () -> RoomTemp`.
- Protocol 0.14 (additive): `TypeView.kind` may be `unit`; `Signature` is
  documented as the presentation of the canonical type.
- Remaining `inputs.is_empty()` tests are presentation (socket rows, dialog
  heights, `f(…)` vs `f` labels) or the named predicate.

## Amendment (2026-09-18, the preferred spelling)

Extends the decision; every sentence above stays true.

- **`() -> A` is the preferred textual spelling** of a relationship without
  inputs; `mapping f : A` is compatibility syntax only — accepted, meaning the
  same declaration, no longer described as an equal alternative. Everything that
  generates source (`bdl-text::print` for the Code view, write-back and
  legacy-project migration; the IDE's `render_module`; hover's surface line;
  examples and documentation) writes `() -> A`. Port types keep the bare output:
  a port's grammar has no domain to spell.
- **The shorthand is a hint with a quick fix**, never an error:
  `text.legacy_unit_domain` (`SemanticSeverity::Hint`, LSP hint tagged
  deprecated) at the signature of a `mapping` written with a bare type, and
  nowhere else; _Make empty domain explicit_ inserts `() ->` and nothing else.
  Hover and Explain show `declared spelling` beside the canonical type.
- **Formatting is never a migration**: the formatter keeps the authored
  spelling. The migration is opt-in, `bdld migrate-unit-domain`
  (`bdl_syntax::migrate`, `bdl_text::make_unit_domains_explicit`): lossless
  outside the inserted text, identities and the design verified unchanged before
  a byte is written.
- **Source is a role, not a type.** A `() -> A` relationship may be shown as a
  source on the canvas (no input socket) and offered as a value; `Source` is not
  a type constructor and not part of the textual type. The reference spelling
  stays `f`: the signature exposes the empty domain, the observation of the
  value erases the unique argument — `f()` and `f(())` remain accepted and are
  not preferred.
- **Staged policy.** Stage 1 (now): explicit form preferred, shorthand accepted,
  hint + quick fix, opt-in migration. Stage 2: the hint becomes a warning;
  generated code never emits the shorthand (already the case). Stage 3: a
  language edition or major syntax revision may remove the shorthand with the
  migration applied automatically. No removal version is scheduled — the
  language has no versioning policy yet; scheduling one is a decision of its
  own.
