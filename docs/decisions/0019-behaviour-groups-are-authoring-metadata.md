---
id: ADR-0019
status: accepted
date: 2026-09-17
area: behavior-systems
supersedes: []
superseded-by: []
related: []
fv: ["informed by FV: BDL_FV Phase 8b cf2fc5e — BDL/Behavior/Group.lean, Boundary.lean, Extract.lean, ExtractPreservation.lean"]
---
# ADR-0019: Behaviour groups are authoring metadata; aggregate sockets are a projection; packaging elaborates into the existing component model

## Status

Accepted (Studio behaviour-authoring milestone; FV Phase 8b, `BDL_FV`
`BDL/Behavior/Group.lean`, `Boundary.lean`, `Extract.lean`,
`ExtractPreservation.lean`, `BEHAVIOR_GROUPING_NOTE.md`).

## Context

A designer wants to gather several relationships into one cognitive unit
on the canvas, see what the unit reads and produces, and — when it is
ready — turn it into a reusable component with instances. Each of those
wants could have been a new language construct: a group node with ports,
a multi-output mapping, a tuple, a fan-out declaration. The formal work
shows none is needed: a group is an identity and a member list; its
boundary is a projection of the existing dependency relation; "package as
component" is a restriction of the design plus one instance and ordinary
bindings; and the kernel judgments of the grouped design are literally the
judgments of the ungrouped one.

## Decision

* **A `BehaviorGroup` is `{ id, name, description, members }` and nothing
  else** (`bdl-system::model`). It is stored on the system beside the
  base design and never read by `flatten`, `validate_composition` or any
  analysis. Collapse state, position and size live in `ui/layout.json`
  (`Layout.groups`), never on the group. A relationship is in at most one
  group (an authoring choice, DI-36).
* **Group edits are not revisions.** `GroupEditOp` (create, rename,
  describe, dissolve, add / remove / move member, merge, split) is applied
  by the daemon outside the revision stream (`ApplyGroupEdit`), bumps an
  *authoring generation* on the `SystemView`, pushes no `ProjectChanged`
  and no `AnalysisReady`, and carries no invalidation set — the flat
  design is the same value before and after (FV Theorems A–G, all `rfl`;
  test `grouping_is_semantically_transparent`). Studio keeps its analysis,
  its simulation run and its layout across them. The destructive delete
  (`DeleteGroupWithMembers`) is a system edit, because it deletes
  relationships.
* **The boundary is computed in Rust, off the flat dependency graph**
  (`bdl-system::boundary::group_boundary`): `crossing_in` (non-members a
  member depends on), `crossing_out` (members a non-member depends on),
  `open_members`, `driven_members`, `private_candidates`,
  `external_inputs = crossing_in ++ open_members`,
  `external_outputs = crossing_out`, and `clocks` (Κ of members and
  crossing-in declarations plus every domain a member observes through
  `sync`). It is served with the `SystemView` at every authoring
  generation from the revision's cached analysis. Studio never computes
  it. **Aggregate sockets on a collapsed group are a picture of these
  lists**: they accept no link, declare no fan-out and introduce no
  tuple; each edge attaches to the specific declaration (FV Theorem H,
  `socket_no_fanout`; test `boundary_is_a_projection_and_sockets_add_no_dependency`).
* **Packaging is elaboration into the existing model**
  (`bdl-system::extract`): the component's body is the members unchanged
  plus one unresolved copy of each crossing-in relationship (required
  ports), provided ports are the crossing-out members, every clock the
  members use is a clock parameter, concepts stay shared, sinks stay
  external unless the designer moves them inside, drives stay with their
  members; the base keeps an unresolved copy of each crossing-out member;
  one instance; ordinary bindings reconnect the two sides. The four
  choices the FV cannot infer (name, instance name, open members as
  inputs or internal, sinks external or internal) are
  `ExtractionChoices`; `PreviewComponentExtraction` shows their
  consequences and mutates nothing; `ExtractGroupAsComponent` is one
  atomic edit that retires the group.
* **Base ends on bindings.** To let the base be the FV residual,
  `Binding` ends are `BindingEnd::{Port(PortRef), Base { decl }}`: a base
  relationship may feed a required port and a provided port may realise an
  open base relationship. Contracts of base ends are read off the
  relationship (shared concepts, the system's domain); `flatten` realises
  a base destination with the same `Definition::Reference` as a port.
* **Causality is decided by the flat analysis, never by a coarse instance
  graph.** A group whose boundary crosses in both directions extracts
  fine as long as the declarations stay acyclic (FV `flat_causal` by
  subdividing edges; test `a_bidirectional_boundary_is_not_a_cycle`).

## Amendment (scoped groups, Studio interaction hardening)

The FV states grouping for *any* `GroupedDesign = Design + groups`;
production first supported base-only groups. This amendment generalises
the scope:

* `BehaviorGroup.scope: GroupScope { SystemBase | Component { component } }`
  — the authored design the members belong to, stated explicitly (never
  inferred from `DeclId` collisions, DI-42). Members are validated against
  that design; a group never spans two designs; moving a relationship
  across a component boundary is an extraction or a body edit, not
  grouping. Names are unique per scope.
* A component-scoped group is *adjacent* to the body: no `body_stamp`,
  `interface_stamp`, `Realizes`, instance freshening or flattening moves;
  its boundary is read off the body's own standalone analysis in
  component-local ids (never an instance's freshened ones). A version
  (`DuplicateComponent`) copies its groups under fresh ids (DI-43); a
  deleted component retires them; a deleted relationship leaves its group.
* One project-level authoring generation still suffices; a group edit
  names the generation it saw (`ApplyGroupEditRequest.base_generation`)
  and is refused as `group_edit.stale_generation` otherwise, so two
  clients never silently overwrite membership. The daemon keeps one
  history of semantic and authoring steps: undo/redo of a group edit
  moves the group table and the authoring generation, never the revision,
  and replays no compilation. A group edit dirties the project
  (`SystemView.dirty`) although the revision is unchanged.
* Aggregate sockets of a collapsed group stay projections, but act as
  *interaction proxies*: a link started or dropped on one resolves to the
  concrete declarations it stands for (a member's socket, or the member
  itself), one directly or several through a chooser; the committed edit
  always names the declaration. No binding to a group socket exists and
  no group-level dependency is ever added (FV Theorem H; regression tests
  `aggregate sockets resolve to concrete declarations, never the group`).
* Layout is context-scoped (`Layout.components[c]` carries a component
  canvas's nodes, group boxes and viewport); collapsing starts the box
  where the region was; moving a collapsed box carries its hidden members
  along, so expanding places the stored internal layout translated by the
  box's delta. Semantic zoom below 0.5× shows every group as its summary
  without touching the authored collapse state.
* Packaging a component-scoped group is deferred (option A, see
  docs/architecture/behavior-systems.md §13): component bodies are flat
  designs, and lowering a nested extraction by re-flattening would put a
  second semantic truth beside the body. Grouping inside a component is
  organisation only; the sheet says so.

## Consequences

* No new kernel term, runtime node, semantic port, evaluator or type
  checker; no `MultiOutputMapping`, tuple primitive, implicit fan-out,
  hidden arbitration or auto-`sync`.
* Studio's context model (system canvas / component source / atomic
  design) and its inspectors read every fact from the `SystemView`, the
  `SystemAnalysisView` and the preview; a generated `Reference` is shown
  ("takes its value from …", *Show Binding*) and never typed into.
* Claims: **proved** (FV) — group transparency, boundary characterisation,
  extraction well-formedness, causality and single-driver preservation,
  and single-domain, transport-free equivalence of the original and the
  packaged design (Theorem R); **production-tested** — the same
  observables (analysis, simulation trace, deployment requirements) across
  every group operation and across extraction for the tested designs,
  including a `sync`-only clock and a transported binding; **not proved**
  — equivalence with higher-order bodies or across transported
  multi-domain bindings.
* Group edits are not in the undo history (they are not revisions); the
  designer reverses them by the inverse operation.
