---
id: ADR-0043
status: accepted
date: 2026-09-21
area: language
supersedes: []
superseded-by: []
related: [ADR-0013, ADR-0032, ADR-0034, ADR-0041, ISS-0020]
fv:
  [
    "informed by FV: the ladder is FVD-0163 (the Sem-block model, Phase 21) and
    FVD-0164 (the naming, Phase 22, `a931712`) — definitional; the formal
    development's rename is an α-conversion with every theorem statement and the
    axiom base unchanged, and nothing formal is claimed for this record",
    "production-tested: every existing test passes under the new names only
    (`cargo test --workspace`, `flutter test`, the golden crates and the
    protocol fixtures byte-identical); `bdl-diagnostics` tests the code alias
    table",
  ]
---

# ADR-0043: Vocabulary — the concept ladder: a concept is a type named by `ConceptId`, a Sem block is its instance, and "semantic" no longer names the type level

## Status

Accepted (the vocabulary alignment with FV Phase 22, 2026-09-21; the last step
before the Sem-block canvas work of ISS-0020). Changes no semantics: an
α-conversion of identifiers plus the words the records and Studio use.

## Context

Production and the formal development had put the word _semantic_ on the type
side (`SemanticId`, "semantic identity", "a semantic type") while the product
word _Sem_ names the instance — the block on the canvas that holds one value per
tick. A reader, human or agent, could not tell from the names which level is the
type and which the instance, and Phase 20 of the formal development had grown an
invariant ("one Sem block per concept") out of that ambiguity. FV Phase 22 fixed
the ladder and renamed its code (FVD-0164); production follows with the same
rename and the same vocabulary, before any canvas work makes the instance level
visible.

The ladder (FVD-0164, `docs/spec/kernel.md` §1):

> representation → **concept** (`ConceptId`; the nominal type `sem C`; a
> template) → **Sem block** (a unit-domain declaration of type `sem C`; an
> instance, one value per tick; its definition is its mapping block and one
> producer; none = Source) → **value** (`sem C v`). In parallel: **rule**
> (arrow-typed declaration; a template) → **mapping block** (a Sem block's
> definition; an instance). `sem C` reads "a Sem of `C`". Several Sem blocks of
> one concept are ordinary.

## Decision

1. **`bdl_model::ConceptId` is the type-level identity** (was `SemanticId`),
   everywhere in Rust and Dart. The raw integer and its serialization are
   unchanged (`#[serde(transparent)]` over `u64`); the two persisted keys that
   spell the old word keep their bytes through `#[serde(rename)]` — the project
   file's `ids.next_semantic` (`IdAllocator::next_concept`) and the generated
   manifest's `concepts[].semantic_id` (`ConceptEntry::concept_id`). No schema
   version, protocol version or generated file changes.
2. **Identifiers whose "semantic" meant the concept level say `concept`**:
   `fresh_concept`, `Hover::concept_type`, `TypeErrorKind::RepOfNonConcept`, the
   tests `mk_yields_the_requested_concept_type`, `concept_types_are_not_raw`,
   `closures_apply_and_keep_concept_identity`. Doc comments say _concept
   identity_, _Sem value_, _Sem type_, _concept-free_ (the FV docstring rule).
3. **The diagnostic namespace of the concept level is `concept.*`**:
   `concept.mismatch`, `concept.no_order`, `concept.unbound_representation`,
   `concept.construction_not_granted` (were `semantic.*`), and
   `type.rep_of_non_concept` (was `type.rep_of_non_semantic`).
   `bdl_diagnostics::CODE_ALIASES` maps every old spelling to its current name;
   `Code::new` and deserialization apply it, Studio's `kDiagnosticCodeAliases`
   does the same, so an old log, fixture or client still names the same finding.
   A code is a value on the wire, not a name: no protocol bump.
4. **"Semantic" keeps its other sense** — _of the behaviour model, as opposed to
   representation, layout or hardware_ — wherever that is what it means:
   `SemanticToken*`, `SemanticAction*`, `SemanticDiagnostic`,
   `SemanticSeverity`, `SemanticCompletion`, `SemanticAnchor`, `SemanticHover`,
   `SemanticOperation`, `SemanticEditPlan`, `SemanticReference`,
   `SemanticSymbol`, `ListSemanticActions` (the IDE service's analysis);
   `INVALIDATION_SEMANTIC` (analysis vs layout); `MissingKind::is_semantic` and
   Studio's `_isSemantic` (the design vs the deployment); "the semantic
   project", "semantic analysis", "nothing semantic" in prose; Flutter's
   accessibility `Semantics`; the l10n keys `*NodeSemantics`, `*Semantics` (what
   a node means to a screen reader). None of these names the type level.
5. **The instance level keeps the product word**:
   `bdl_reactive::Value::Semantic { id, repr }`, the protocol's
   `SemanticValue { concept_id, repr }` and the `Value.semantic` case,
   `DynValue::Semantic` and the telemetry's `"kind":"semantic"` are _a Sem value
   of concept `id`_ — the instance level, which is what _Sem_ means (FV keeps
   `Ty.sem` and `Value.sem` for the same reason). Renaming them would change
   wire names and generated bytes for a spelling only.
6. **Product vocabulary.** The type is a _concept_; a unit-domain relationship
   is a _Sem block_ — a _Source_ when nothing defines it, a _value_ when its
   _mapping block_ (the definition drawn as the node) does; an arrow-typed
   relationship is a _rule_, a template as a concept is. Studio's roles
   (ADR-0032: Source / Rule / Value) already partition the ladder this way and
   keep their labels; the guide's terminology defines the words; "semantic" is
   never a noun for the type.
7. **Historical text stays.** ADR bodies before this one, the paper and its
   digest, the archived ledgers and the closed change histories keep
   `SemanticId`; FV theorem names quoted anywhere
   (`semantic_identity_mismatch_rejected`,
   `temporal_state_preserves_semantic_identity`, …) keep their spelling — in a
   theorem name "semantic identity" reads "concept identity".

## Alternatives

- **Keep `SemanticId` and explain the ladder in prose** — rejected: the names
  would keep saying the opposite of the ladder to every new reader (FVD-0164's
  own reason).
- **Rename `Value::Semantic` / `SemanticValue` to `Sem`** — rejected here: the
  instance level is what _Sem_ already means, and the rename would touch wire
  names and generated Dart for no change in meaning; recorded as a possible
  follow-up, not a debt.
- **Migrate the persisted keys** (`next_semantic`, `semantic_id`) — rejected: a
  schema bump and a migration for every project file and generated manifest, to
  change a spelling nobody reads; `#[serde(rename)]` keeps the bytes and the
  Rust names say the right thing.
- **Keep the `semantic.*` diagnostic namespace** — rejected: those four codes
  are exactly the concept level (two concepts where one is needed, a concept
  without order, a concept's representation unbound, a concept's construction
  not granted); the alias table makes the rename free for consumers.

## Consequences

- `SemanticId` does not occur in Rust, proto or Dart; `ConceptId` does
  (`crates/bdl-model/src/ids.rs`). The Rust API is source-incompatible for the
  renamed items (a workspace-internal break; nothing published depends on it).
- Project files, generated crates, manifests, telemetry and the protocol are
  byte-identical (the golden crates under `crates/bdl-compiler/tests/golden`,
  the daemon's e2e fixtures and the Studio fixtures are unchanged).
- Records: `docs/spec/kernel.md` §1 states the ladder and names `ConceptId` in
  the grammar; `bdl-model::surface` restates it on `Concept`, `MappingBlock`,
  `RelationshipRole` and `role()`; `docs/architecture/compiler-pipeline.md` and
  `docs/spec/protocol.md` carry the code alias table;
  `docs/project/formal-correspondence.md` has the FVD-0164 row; the guide's
  terminology defines concept, Sem block, mapping block and rule; a change
  fragment.
- Not decided here: the Sem-block canvas (ISS-0020), any rename of the instance
  level's identifiers, any change to the roles' labels.
