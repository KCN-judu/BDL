# ADR-0017: Behaviour systems are a surface layer that flattens into the flat design

## Status

Accepted (behaviour-system milestone; FV Phase 8a).

## Context

Designers need to name a behaviour, give it an interface, instantiate it
several times, bind instances together and package compositions. The
formal development (`BDL_FV/BDL/Behavior`) shows this needs no kernel
construct: components, instances and bindings are surface objects that
elaborate, by renaming and one realization step per binding, into an
ordinary design under the existing five judgments. Production already has
a complete flat pipeline that every tool consumes through
`ProjectSnapshot`.

## Decision

* A dedicated crate, `bdl-system`, owns the authored `BehaviorSystem`
  (base design + components + instances + bindings + exports), its
  revisioned edit model, freshening, flattening, composition validation,
  acceptance levels, packaging and persistence. It depends on the
  compiler crates; nothing depends on it except the daemon and the
  protocol.
* Flattening produces a `ProjectSnapshot`, not a new IR: the existing
  elaboration, checker, causality, clocks, outputs, simulation,
  deployment, lowering, codegen and IDE consume it unchanged. There is no
  system type checker, evaluator, clock judgment or code generator.
* The kernel (`bdl-ir`, `bdl-check`, `bdl-reactive`, `bdl-exec-ir`) is
  untouched. The flat surface gains two identity-bearing definition forms
  that only flattening produces: `Definition::Reference` (a binding, the
  kernel's `declRef`/`sync`) and `Definition::ScopedFormula` (a body
  formula with its names pinned to identities).
* Identity: component-local ids live in the component body's own
  allocator; flat ids of private entities come from a persisted
  freshening table allocated by the edit model from the base design's
  allocator (never reused, never renumbered). Component, instance, port,
  binding and export ids are new sorts. Display names are never identity.
* Bindings are by `(instance, port)` identity; a binding is a realization
  step; destinations are bound at most once; cross-domain bindings are
  explicit transports with a closed initial value.
* A system project persists the system only; its flat design is derived.
  Flat projects keep their format and behaviour byte for byte.

## Consequences

* Every property of a composed system is established by the existing
  flat analyses on the flattened design and projected back through the
  origin map; composition-level diagnostics add the instance/port
  vocabulary, never a second verdict.
* Two locally valid components can still be rejected after composition
  (an instantaneous cycle, two drivers of one shared sink) — by the
  existing rules.
* Hierarchy is packaging: a flattened system is a component body.
* Cross-project portable packages, the hierarchical Studio canvas and
  interface-refinement versioning are later milestones
  (`docs/BEHAVIOR_SYSTEMS.md`, limitations).
