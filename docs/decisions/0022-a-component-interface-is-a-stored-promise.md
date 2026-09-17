---
id: ADR-0022
status: accepted
date: 2026-09-16
area: behavior-systems
supersedes: []
superseded-by: []
renumbered-from: ADR-0018
related: []
fv: ["informed by FV: BDL_FV Phase 8a — BDL/Behavior/Interface.lean, Substitution.lean"]
---
# ADR-0022: A component's public interface is a stored promise, realized by its body

## Status

Accepted (contract-hardening milestone).

## Context

ADR-0021 introduced behaviour components whose ports were links into the
component body: the "contract" a binding checked against was whatever the
backing declaration happened to look like. Editing the body silently
changed what every instance promised, and composition could not be
decided without the bodies. The formal `Port` carries an advertised
interface and clock; `Realizes` relates the body to it; `BindingWF`,
`IfaceRefines` and `Substitutable` are stated on interfaces alone.

## Decision

* `Port.contract: PortContract { signature, commitments, clock }` is stored
  on the port, in component-local terms (a shared concept resolves to the
  system concept through the component's table; a clock is `Agnostic`, a
  `Parameter` or `Private` local clock — never a system `ClockId`).
  `port.decl` remains the link to the declaration meant to realize it.
* `DeclarePort` snapshots the contract once; `ChangePortContract` is the
  only way to move it (a refinement when FV `IfaceRefines` holds, an
  `Interface` edit otherwise, naming the reopened bindings);
  `SetClockParameter` rewrites the clock's *role* in contracts that name
  it; `RebindPortDeclaration` moves the link only.
* `contract::realizes` is the production `Realizes`, decided per
  component; failures are `component.*` diagnostics that name the
  component, the port and the backing declaration. Bindings keep their
  identity; the flat compiler judges their meaning against the actual
  body.
* `contract::binding_compatibility` and `contract::component_substitutable`
  decide composition and substitution on contracts over resolved concept
  identities — no body is inspected at a use site, no representation
  equality is ever taken for identity.
* Versions are `DuplicateComponent` copies (same port and local ids);
  `ReplaceInstanceComponent` substitutes a refining version at an
  instance and keeps every binding, export, parameter value and flat
  identity.
* `body_stamp` and `interface_stamp` count implementation and interface
  changes separately. Schema 2 persists contracts; schema 1 is migrated
  once on load.

## Consequences

* Clients of a component depend on what it promises, not on how its body
  looks today; the next Studio milestone can treat a component as a black
  box from `PortView.contract`.
* Flattening is unchanged for a realizing component.
* No behavioural equivalence is claimed by substitution (FV Substitution
  says the same).
