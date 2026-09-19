# One derived role — Source, Rule, Value — stated by the daemon (protocol 0.20)

- Date: 2026-09-20
- Area: model, compiler, ide, protocol, studio
- Affected: protocol clients, developers
- Related: ADR-0032 (amended), ADR-0034, ADR-0030,
  [2026-09 unapplied rule](2026-09-unapplied-rule.md), PRP-0001

## What changed

- **The role has one home.** `bdl_model::RelationshipRole { Source, Rule, Value
  }` and `MappingBlock::role` state what a relationship is from two authored
  facts — whether it reads anything, whether it has a realization. The compiler
  carries it in `MappingAnalysis.role`; the daemon on every `MappingView.role`;
  `bdl-ide::relationship_role` reads the committed design, so a definition
  draft never changes a role until it commits. `RelationshipRole::Mapping` and
  `RelationshipRole::Port(_)` are gone: a port-backed declaration keeps its
  role and carries the port as a fact (`bdl-ide::{port_backed, provider,
  Provider}`); hover and Explain say `role: Source | Rule | Value`, `port:
  required | provided | parameter`, and a Source's `provision` (the environment,
  or the port's binding).
- **Studio re-derives nothing.** `relationshipRole` maps the daemon's enum;
  `isSource`, `isDeclared`, the canvas, the inspector, the Simulate page (inputs,
  columns, probes), the output inspector's driver candidates and the status line
  all read it. _Declared_ is a rule with no formula; a Source and a value are
  complete. `appliersOf` / `referrersOf` read `MappingAnalysis.applied_by`
  instead of inverting `references`. Semantic actions, like entity hover, are
  requested in the system context only (DI-40).
- **`rule.apply`** is blocked for a rule inside an instance (a flattened
  `instance.local` name), with the instance named: the value belongs in the
  component's source. Candidates are Sources and values (never rules) at the
  same level, matched by concept identity.
- **One revision per authored step**: a library transaction and a rename
  expansion advance the revision once.
- `docs/architecture/relationship-roles.md` is the normative matrix: role,
  states, the component boundary, dependencies, output eligibility, Phase 13.

## Compatibility and migration

- Designers: nothing to do. The words on screen are the ones already there
  (_Source_, _Rule_, _Value_, _declared_, _not applied_, _no value yet_); a
  port-backed relationship of an open component is now named by its port for
  assistive technology.
- Project files: nothing — the role is never persisted.
- Protocol clients: protocol **0.20**, additive — `MappingView.role`,
  `MappingAnalysis.role`, `MappingAnalysis.applied_by`, the enum
  `RelationshipRole`. A 0.19 client ignores them; Studio 0.20 requires them
  (a view without a role is not this daemon's).
- Developers: `bdl_model::RelationshipRole`, `MappingBlock::role`;
  `bdl_compiler::MappingAnalysis.role`; `bdl_ide::{relationship_role,
  port_backed, provider, Provider}` (`provision` and the `Mapping` / `Port`
  variants are gone); `bdl_protocol::convert::role_to_pb`,
  `system::base_projection`; Studio `RelationshipRole { source, rule, value }`,
  `RoleFacts`, test fixtures `test/support/roles.dart` (`mappingView`,
  `withAppliedBy`).

## Evidence

`docs/architecture/relationship-roles.md` § Evidence.
