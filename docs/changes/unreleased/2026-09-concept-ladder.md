# The concept ladder's vocabulary: `ConceptId`, the `concept.*` codes (ADR-0043)

- Date: 2026-09-21
- Area: language, compiler, ide, studio, protocol
- Affected: developers, protocol clients reading diagnostic codes
- Related: ADR-0043, FVD-0164, ISS-0020

## What changed

- **A concept is a type, a Sem block its instance.** The type-level identity is
  `bdl_model::ConceptId` (was `SemanticId`) in every crate, the protocol's
  comments and Studio; `fresh_concept`, `Hover::concept_type`,
  `TypeErrorKind::RepOfNonConcept` follow. Doc comments and the current records
  say _concept identity_, _Sem value_, _Sem type_, _concept-free_;
  `docs/spec/kernel.md` §1 states the ladder; the guide's terminology defines
  concept, Sem block, mapping block and rule.
- **Diagnostic codes of the concept level are `concept.*`**: `concept.mismatch`,
  `concept.no_order`, `concept.unbound_representation`,
  `concept.construction_not_granted` (were `semantic.*`) and
  `type.rep_of_non_concept` (was `type.rep_of_non_semantic`).
- **Nothing else changed**: no semantics, no analysis result, no generated code,
  no project-file byte, no wire name. "Semantic" keeps its other sense (the IDE
  service's semantic tokens and actions, the semantic project, the
  design-vs-deployment split) and the instance level keeps `Value::Semantic` /
  `SemanticValue` (a Sem value of a concept).

## Compatibility and migration

- **Designers**: nothing. Studio's labels are unchanged.
- **Project files and generated crates**: nothing. The project file's
  `ids.next_semantic` and the manifest's `concepts[].semantic_id` keep their
  keys (`#[serde(rename)]`); no schema version changes.
- **Protocol clients**: the version stays 0.29. A client that matches the old
  code spellings still decodes: `bdl_diagnostics::CODE_ALIASES` maps every old
  code to its current name — `Code::new` and deserialization apply it, and
  Studio's `kDiagnosticCodeAliases` does the same. Update matches on
  `semantic.*` / `type.rep_of_non_semantic` at leisure; the old spellings are
  never emitted again.
- **Rust developers**: `bdl_model::SemanticId` no longer exists; use
  `ConceptId`. The raw integer and its serialization are identical.

## Evidence

`crates/bdl-model/src/ids.rs` (`ConceptId`, the `next_semantic` key kept),
`crates/bdl-codegen-rust/src/manifest.rs` (the `semantic_id` key kept),
`crates/bdl-diagnostics/src/lib.rs` (`CODE_ALIASES`,
`an_old_code_spelling_decodes_to_its_current_name`),
`apps/studio/lib/l10n/diagnostics.dart` (`kDiagnosticCodeAliases`); every
existing test under the new names — `cargo test --workspace`, `flutter test`;
the golden crates under `crates/bdl-compiler/tests/golden` and the fixtures
byte-identical.
