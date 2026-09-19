---
kind: architecture
area: compiler
status: current
---

# Relationship roles: Source, Rule, Value

One question has one answer: _what is relationship X at revision R?_ The answer
is one of three **roles**, derived — never persisted, never authored, never an
identity — from two facts of the authored design, and stated by Rust on every
projection and every analysis. Everything else a surface shows about a
relationship is a **state** that varies within a role. This page is the
normative matrix; the user guide and Studio's design standard say the same in
the designer's words.

## The rule

```text
role(x) = Rule    if x reads something          (its canonical type is an arrow)
        = Value   if x reads nothing and has a realization
        = Source  if x reads nothing and has none
```

- _reads something_: the signature has inputs; the canonical type is
  `A₁ -> … -> B` (ADR-0029; `Ty::Arr`).
- _has a realization_: a definition is attached — a formula (checked or not), a
  reference a binding made when the system was flattened, memory (`delay`), a
  constant, a parameter's argument. **Not** a draft: a definition draft is
  authoring state (ADR-0030), and the role follows the committed revision until
  the draft is committed.

The one home of the predicate is `bdl_model::RelationshipRole` /
`MappingBlock::role`. The compiler states it in `MappingAnalysis.role`; the
daemon states it on every `MappingView.role` (protocol 0.20) — of the flat
design, of a component body, and of the system's base design, where a base
relationship a binding realises is stated as a Value because its flattened copy
carries the binding. `bdl-ide::relationship_role` reads the committed design for
hover and Explain. Studio's `relationshipRole` maps the enum and nothing else;
no surface re-derives the role from a shape, a name or a formula.

Formally (FV Phase 12, `BDL/Surface/UnitDomain.lean`): `Source Δ d` is "no
realization", `SimulationInput = Source ∧ UnitDomain`, and `resolved_not_source`
says a realized `() -> B` never consults the environment. A Source that Phase 13
provisions gains a realization and is a Value by the same rule (§ Phase 13).

## The matrix

| Role   | Derived from                                | Tick value?                           | Simulate input? | Trace column? | Formula use                 | Can drive an output?                                                     |
| ------ | ------------------------------------------- | ------------------------------------- | --------------- | ------------- | --------------------------- | ------------------------------------------------------------------------ |
| Source | unit domain, no realization                 | yes once supplied and its domain ticks | yes             | yes           | by reference (`tilt`)       | yes — `DriveWF`: its type is the sink's and its domain is the sink's     |
| Rule   | a domain with inputs                        | no — a function                       | no              | no            | applied (`dim(tilt)`)       | no — its type is an arrow, `output.type_mismatch`                        |
| Value  | unit domain, a realization                  | yes                                   | no              | yes           | by reference (`brightness`) | yes — `DriveWF`, as a Source                                             |

- **Simulate input**: the evaluator needs a value per tick for every Source of
  the flat design and nothing else; Studio's input controls are exactly those.
  Nothing is defaulted: a Source with no value given is _no value yet_ — never
  `false`, `0`, none or the first choice — and the run does not step until every
  Source has one.
- **Trace column / probe**: a Source and a Value are sampled at every tick of
  their domain and can be probed; a Rule has no column, its probe describes the
  function and names its appliers, and offers `rule.apply` when nothing applies
  it.
- **Output driver**: the output pass (`bdl-output`, `DriveWF`) accepts any
  driver whose type is what the sink accepts and whose domain is the sink's — a
  Source as well as a Value; a Rule's type is an arrow and is refused
  (`output.type_mismatch`). Realization is not required to drive: a Source
  driving a sink is executable once its value is supplied (the simulator's
  input; at deployment a device, ISS-0016). Studio offers _Drives_ to Sources
  and Values and never to Rules; the physical sink stays `OutputId` + the drive
  edge (`docs/spec/kernel.md` §Outputs, `DriveWF`; no `A -> ()`).

## Roles and states

| Fact                          | Role it can occur in | Where it is stated                                                                    |
| ----------------------------- | -------------------- | ------------------------------------------------------------------------------------- |
| declared (no definition)      | Rule                 | `MappingView.definition` absent; the canvas dashes it, the status line counts it      |
| invalid / open / valid        | Rule, Value          | `MappingAnalysis.status`                                                              |
| applied by nothing            | Rule                 | `reactive.rule_unapplied` (info); the hollow output socket, _not applied_             |
| applied by …                  | Rule (and any)       | `MappingAnalysis.applied_by`                                                          |
| references …                  | Rule, Value          | `MappingAnalysis.references`                                                          |
| driven / drives               | Source, Value        | `MappingView.drives_output_id`, `OutputAnalysis`                                      |
| bound to …                    | Value                | a binding whose destination is the relationship (`SystemView.bindings`)               |
| backs a port                  | Source, Value        | `ComponentView.ports[].decl`; `bdl-ide::port_backed`                                  |
| clocked                       | any                  | `MappingView.clock_id`                                                                |
| no value yet                  | Source               | Studio's simulation inputs (never a project fact)                                     |
| draft differs                 | any                  | Studio's drafts; the role is the committed one                                        |

The compiler's `MappingStatus::Declared` means "no realization" for any role
(a Source is `Declared` there); the product word _declared_ is the state of a
Rule with no definition. `Open` is a definition waiting on a concept's value
form; `Invalid` a definition that does not check.

## The component boundary

The role is a fact of one design. A component body is a design; the system's
base is a design; the flat design is a design. The same declaration can have a
different role in each, and every surface reads the design it shows:

| Declaration                                        | in the body        | in the flat design                             | at the top level (system view's base) |
| -------------------------------------------------- | ------------------ | ---------------------------------------------- | ------------------------------------- |
| backs a required port, unresolved                  | Source (the port)  | bound instance: Value; unbound instance: Source | —                                     |
| backs a parameter port                             | Source (the port)  | Value (the instance's argument)                | —                                     |
| backs a provided port, realized inside             | Value              | Value                                          | —                                     |
| an internal `() -> A` with no definition            | Source             | Source                                         | —                                     |
| a rule                                             | Rule               | Rule                                           | —                                     |
| a base `() -> A` a binding realises                | —                  | Value (the binding's reference)                | Value                                 |
| a base `() -> A` nothing binds                     | —                  | Source                                         | Source                                |

_Who provides a Source_ is a fact beside the role (`bdl-ide::Provider`): the
environment, or — inside a body — the port it backs (the instance's binding or
argument). A required port is a hole the composer fills, not a hole the
designer fills: inside the component it is complete, and Studio wears the
port's word instead of _Source_. An unbound required port of an instance is a
Source of the system (FV Theorem H): a simulation input, exactly as the
evaluator needs.

Semantic actions and entity hover are flat-entity services (DI-40): for a rule
inside an instance the flat analysis states `reactive.rule_unapplied`, and
`rule.apply` is **blocked** with the instance named — the value belongs in the
component's source, where the flat names (`instance.local`) do not exist.

## Dependencies

`references` are the declarations a realization names, by identity — the
kernel's `dependsOn`, `DependencyGraph::all` — and `applied_by` is their
inverse: the **direct** reverse edges, never transitive. A Rule referenced by
another Rule is applied; the note walks up to the outermost rule nothing
applies, and one value applying that one settles the chain. Studio consumes
both fields and never inverts, scans a formula or parses displayed text; the
canvas's reference edges are presentation, identity-based, not editable and not
drop targets (ADR-0034).

## Phase 13

FV Phase 13 (provision) realizes an abstract Source by a device profile's raw
input and a checked transducer. In this model a provisioned Source is a Value:
it gains a realization, so the same rule gives the same answer, and no
`ProvisionedSource` role exists or is needed. Ownership, when it is built
(PRP-0001, not implemented): the device profile and raw type in the device
catalogue (`hardware/`, never the Standard Library — an authoring catalogue of
fragments); the transducer a checked BDL formula; _Fits_ in deployment
analysis; the provision transformation in the compiler's deployment pass; the
raw input in the platform adapter; the induced input a testing oracle. Strict
refinement (`Trace(device) ⊆ Trace(abstract)`) is the normal case and no
surface reports its absence; declaration commitments are empty today, so
deployment commitment checking is vacuous.

## Evidence

`crates/bdl-model/src/edit.rs`
(`create_mapping_with_definition_and_clock_checks_structure_and_keeps_the_text`),
`crates/bdl-compiler/src/lib.rs` (`the_analysis_states_the_role_and_a_state_never_moves_it`,
`a_rule_nothing_applies_is_stated_until_a_value_applies_it`),
`crates/bdl-ide/tests/source_role.rs`, `crates/bdl-ide/tests/rule_apply.rs`,
`crates/bdl-daemon/tests/system_e2e.rs`
(`the_role_is_one_answer_across_the_component_boundary`),
`apps/studio/test/source_role_test.dart`, `apps/studio/test/unapplied_rule_test.dart`
(the acceptance scenario: roles, undo/redo, a `delay` value, save and reopen),
`apps/studio/test/simulation_test.dart`, `apps/studio/test/outputs_test.dart`.
