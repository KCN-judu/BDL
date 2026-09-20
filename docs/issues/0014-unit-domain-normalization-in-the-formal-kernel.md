---
id: ISS-0014
state: resolved
area: formal
opened: 2026-09-18
resolved-by: [ADR-0032]
related: ["ADR-0029", "ADR-0010"]
---

# ISS-0014: Prove the Unit-domain normalization of relationships without inputs in the formal kernel

## Problem

Production gives every relationship one canonical type `domain(inputs) -> B`
with the empty product `()` as the domain of no inputs, and encodes it into the
kernel by currying and unit elimination (`() -> B` as `B`; ADR-0029,
`bdl_ir::ty`). The formal kernel (`BDL/Core/Base.lean`) has no unit type: a
declaration without inputs has `expectedType = B`, and `delay`/`sync` are typed
only in the empty context. The claim that the normalization is a pure re-typing
— that `() -> B` and `B` differ in no typing, evaluation or clock judgement, and
that `Product([]) ≅ ()` — is stated in production and not proved.

## Why it matters

ADR-0010 makes Lean the semantic authority. A canonical type the kernel does not
have is an engineering normalization until a theorem says the two encodings
agree; until then the correspondence row is _engineering choice_, and a future
kernel change (a unit type, product domains, memory under a unit binder) could
silently diverge from what production displays and normalizes.

## Current evidence

- `crates/bdl-ir/src/ty.rs`: `Ty::Unit`, `Ty::of_signature`,
  `Ty::kernel_of_signature`, `Ty::canonical_mapping_ty`, tested inverse over
  signatures
  (`a_relationship_without_inputs_has_the_unit_domain_and_the_encodings_are_inverse`).
- `crates/bdl-ide-db/src/textual.rs`
  `the_shorthand_and_the_explicit_unit_domain_are_one_declaration`: the
  shorthand and the explicit spelling bind and elaborate identically.
- `crates/bdl-elab/tests/expressions.rs`
  `a_unit_domain_relationship_is_read_as_a_value_and_its_argument_is_erased`:
  `f`, `f()` and `f(())` are one `declRef`.
- The kernel: `Typing.lean` types `delay` only in `[]`; `Reactive.lean`
  evaluates a declaration's realization once per activation.

## Dependencies

A formal phase in `BDL_FV`: either a `Ty.unit` with `arr unit B ≅ B` as an
explicit elaboration isomorphism (the production reading), or a proof that the
surface normalization is conservative over the existing kernel — typing,
evaluation and clocks unchanged — in the style of Phase 11's `Natural.lean`.

## Resolution

Resolved by FV Phase 12 (`BDL/Surface/UnitDomain.lean`,
`docs/notes/unit-domain-normalization.md`, `dd44a84`): the canonical type lives
in an interface layer `CTy` above the kernel, unit elimination is a total
function `elim` with `elim_canonical : elim (canonical s) = some (encode s)`,
the encodings are inverse over concept signatures (`encode_decode`,
`decode_encode`, `encode_injective`, `canonical_injective`), a declaration
without inputs has no binder (`zero_input_obligation`, `lams_typed`) and holds
memory (`zero_input_memory`), and `f`, `f()`, `f(())` are one reference
(`refForms_agree`). The kernel gains no unit type. Recorded in
`docs/project/formal-correspondence.md`; ADR-0032 builds the Source role on the
same phase.
