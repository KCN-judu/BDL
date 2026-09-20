---
kind: architecture
area: behavior-systems
status: current
---

# Behaviour systems — implementation design

How reusable behaviour components, instances and bindings enter production BDL
without a second language, a second checker, or a change to the kernel. The
formal reference is Phase 8a of `BDL_FV` (`BDL/Behavior/`,
`docs/notes/behavior-system-requirements.md`, FVD-0064 … FVD-0073); the
correspondence table is `docs/evidence/behavior-systems-correspondence.md`.
Written before the code; the code follows it.

```text
BehaviorSystem  (authored truth of a system project)
      │  bdl-system::flatten            system elaboration: instantiate, freshen,
      │                                 substitute clocks, realise parameters,
      ▼                                 union, realise bindings, build origins
ProjectSnapshot { design: Design }      an ordinary flat design — DERIVED
      │
      ▼
EXISTING, UNCHANGED: elaborate → check → causality → clocks → outputs
                     → simulate → deploy → lower → codegen → IDE
```

## 1. The canonical authored system model

`bdl_system::BehaviorSystem` (crate `crates/bdl-system`, depends on `bdl-model`
and, for validation only, on the compiler crates — never on Studio):

```text
BehaviorSystem {
    base:       Design                       the system-level flat design: shared concepts,
                                             system clock domains, external sinks, devices,
                                             and any top-level relationships
    components: BTreeMap<ComponentId, BehaviorComponent>
    instances:  BTreeMap<ComponentInstanceId, ComponentInstance>
    bindings:   BTreeMap<BindingId, Binding>
    exports:    BTreeMap<ExportId, Export>   required ports declared as system inputs
    flat_ids:   FlatIds                      the persisted freshening table (§5)
    ids:        SystemIdAllocator            component / instance / port / binding / export counters
}
```

A **flat project is the degenerate system**: `base` only, no components, no
instances, no bindings. Nothing about a flat project changes (§10).

`BehaviorComponent` (project-local; §14 of the brief — not a portable package
yet):

```text
BehaviorComponent {
    id, name, description,
    body:       Design                        an ORDINARY flat design over the component's
                                              own local ids and its own IdAllocator
    interface:  BehaviorInterface { required, provided, params: Vec<Port>, clock_params: Vec<ClockId> }
    shared_concepts:  BTreeMap<SemanticId(local), SemanticId(base)>   the concepts the body
                                              takes from the system instead of owning
    external_outputs: BTreeMap<OutputId(local), OutputId(base)>      sinks the body drives
                                              that belong to the system
    stamp: u64                                bumped by every body/interface edit
}
Port { id: PortId, decl: DeclId(local), kind: Required | Provided | Parameter, name, description,
       contract: PortContract { signature (local concepts), commitments: [PropertyId], clock: Agnostic | Parameter{clock} | Private{clock} } }
```

The body is authored with the existing flat `EditOp`s (concepts, mappings,
formulas, clocks, outputs, devices) — there is no second body language. A
required port and a parameter _are_ unresolved mappings of the body; a provided
port is a mapping of the body; ports are exposed by identity (`PortId` →
`DeclId(local)`), and consumers never see body names.

## 2. What remains the canonical flat execution model

`ProjectSnapshot { revision, design }`, exactly as today, is what every pass
consumes. For a system project it is **derived** by `flatten` on every commit
and never edited or persisted; for a flat project it is the authored truth.
`bdl_compiler::analyze`, simulation, deployment, lowering, codegen, `IdeHost`
and the LSP take it unchanged. There is no system type checker, evaluator, clock
judgment or code generator (FVD-0064).

## 3. Component-local identities

Inside a component body the ordinary id sorts (`DeclId`, `SemanticId`,
`ClockId`, `OutputId`, `DeviceId`) are **local**: they come from the body's own
`IdAllocator` and mean nothing outside the component — the production analogue
of FV's "identities below `width`". Two components may both have local
`DeclId 0`. A body entity is _private_ unless it is listed in `shared_concepts`
(a concept taken from the system), `external_outputs` (a sink of the system), or
`interface.clock_params` (a clock the instantiation maps to a system domain).
Sharing is never inferred from names or from Standard Library template origin
(§13, §59 of the brief): it is an explicit table on the component.

## 4. Instance identities

```text
ComponentInstance { id: ComponentInstanceId, component: ComponentId, name,
                    clock_bindings: BTreeMap<ClockId(local param), ClockId(base)>,
                    parameter_bindings: BTreeMap<PortId, ParameterValue { source: closed formula }> }
Binding  { id: BindingId, source: PortRef, destination: PortRef, transport: Option<Transport { init: closed formula }> }
PortRef  { instance: ComponentInstanceId, port: PortId }
Export   { id: ExportId, port: PortRef, name }
```

`ComponentId`, `ComponentInstanceId`, `PortId`, `BindingId`, `ExportId` are new
`u64` newtypes allocated from `SystemIdAllocator`, persisted, never reused. They
never overload `DeclId`/`SemanticId`/`ClockId`/ `OutputId`. Renaming an
instance, a port or a component changes a string and nothing else.

## 5. Fresh flattened ids

The FV encodes `fresh W k n = W·(k+1)+n` because Lean identities are naturals;
production keeps its sequential per-project allocator instead (§19 of the
brief). The **freshening table**

```text
FlatIds: BTreeMap<(ComponentInstanceId, LocalEntity), u64>
LocalEntity = Decl(DeclId) | Sem(SemanticId) | Clock(ClockId) | Output(OutputId) | Device(DeviceId)
```

is part of the authored system and is extended by the **edit model**, not by
flattening: after every system edit, `ensure_flat_ids` walks every instance and
every private entity of its component body in `BTreeMap` order and allocates a
missing entry from `base.ids` — the same allocator the system's own concepts,
clocks and outputs come from. Hence:

- _injective / collision-free_: one counter per sort issues every id, global and
  fresh alike;
- _deterministic_: the walk order is fixed and the table is persisted, so
  reopening gives the same ids;
- _stable across unrelated edits_: an entry, once allocated, is never
  renumbered; adding instance C or renaming lampA touches no existing entry;
  deleting an instance removes its entries and the numbers are never reused;
- _origin recoverable_: the table is the reverse map (§6);
- _compatible_: flattened ids are plain `DeclId` etc.

`flatten` is then a pure function of the snapshot; a private entity with no
entry is a `system.internal` diagnostic, never an allocation.

## 6. Provenance

`flatten` returns
`FlattenedSystem { snapshot, origins: OriginMap, diagnostics }` where

```text
OriginMap { decls: BTreeMap<DeclId, Origin>, sems, clocks, outputs, devices, ports: BTreeMap<DeclId, PortRef> }
Origin { instance: ComponentInstanceId, component: ComponentId, local: LocalEntity }
```

plus the forward lookup `flat_of(instance, local)`. Base entities have no origin
(they are the system's own). Every consumer that shows a flat id — diagnostics,
hover, simulation traces, deployment rows, the generated manifest — can project
it to `instance.local` through this map (`project_diagnostics` does so for
`ProjectAnalysis`).

## 7. Where flattening enters the pipeline

One pre-pass, before pass 1 of `docs/architecture/compiler-pipeline.md`:

```text
pass 0  system elaboration   BehaviorSystem → ProjectSnapshot (+ OriginMap, composition diagnostics)
```

`bdl_system::analyze_system(&SystemSnapshot) -> SystemAnalysis` = flatten, then
`bdl_compiler::analyze` on the result, then origin projection of the diagnostics
and the port/acceptance summary. Simulation and deployment likewise: flatten,
then the existing `Simulation` / `analyze_deployment`. In `bdld` the session
holds the system as authored truth and the flattened `ProjectSnapshot` as
`current`; every existing request sees `current` and works unchanged.

## 8. Bindings without synthesised text

A binding never becomes `Formula { source: "sensorTilt" }`. The flat surface
model gains one **identity-bearing** definition form:

```text
Definition::Reference { target: DeclId, transport: Option<Transport { source: ClockId, init: String }> }
```

which `bdl-elab` elaborates directly to `Expr::DeclRef { target }` or
`Expr::Sync { src, init, DeclRef target }` — exactly the FV's `bindingBody` —
and the existing checker verifies against the destination's interface. The
`init` (and a parameter's value) is a _closed_ formula text: it is elaborated
with no inputs and no relationship names in scope and must be reference-free and
delay-free, so no name can enter. The authored system never contains a
`Reference`; only the derived design does. Studio renders it as "takes its value
from …" with _Show Binding_, never as a formula field.

**Binding ends** (Phase 8b): `Binding { source, destination: BindingEnd }` with
`BindingEnd::Port(PortRef) | Base { decl }`. A base end is a relationship of the
system's own design: a base relationship may be the _source_ of a required port
(the base is the FV residual, its provided "ports" being its relationships), and
an _open_ base relationship may be the destination of a provided port.
`contract::resolve_end` gives every end the same shape (flat declaration,
resolved signature, resolved clock, commitments, label) so
`binding_compatibility` and `validate_composition` do not care which kind it is;
`flatten` realises a base destination with the same `Definition::Reference`.
Persistence is untagged: a port end is `{instance, port}` as before, a base end
is `{decl}`.

Choice A over B (flattening into `DesignIr`) because every downstream consumer —
persistence of drafts, IDE overlays, projections, simulation input naming,
deployment — starts from `ProjectSnapshot`; flattening to the surface reuses all
of it, and the only price is one enum variant that the IDE's text-oriented paths
treat as "has no formula text".

## 9. Persistence

The system is persisted as source text under `src/**/*.bdl` with the identity
sidecar `.bdl/identities.json` and the authoring sidecar `.bdl/authoring.json`
(docs/spec/project-format.md, ADR-0023); the flattened design is derived on open
and on every commit, never written. Two files claiming to be the truth of one
project is exactly what §7 of the brief forbids. Layout stays in
`ui/layout.json` keyed by flat ids. The JSON form (`design/system.bdl.json`) is
legacy: read once, on open, to migrate a project written before ADR-0023.

## 10. Ordinary flat projects

There is no separate flat project kind: every open project is a
`BehaviorSystem`, and a design with no components is the degenerate system whose
flattening is its own base (`BehaviorSystem::from_flat`, `SystemView.is_flat`).
`apply_edit`, undo, save, analyse, simulate, deploy, the IDE and the Smart Lamp
example behave as before; a flat `ApplyEdit` is the system edit `Base { op }` on
that base.

## 11. Public contract versus body

A port's `contract` is the promise (FV `Port.iface` + `Port.clock`), stored on
the port; `port.decl` is only the body declaration meant to realize it.
`DeclarePort` snapshots the contract from the declaration once; from then on

- a body edit never touches a contract — it may make the component stop
  **realizing** its interface (`contract::realizes`, the production
  `BehaviorComponent.Realizes`: declaration present, same signature, same clock
  role, required/parameter ports unresolved, parameters nullary and agnostic,
  commitments established), which `analyze_system` reports as `component.*`
  diagnostics naming the component, the port and the backing declaration;
  bindings keep their identity and their meaning is then judged by the flat
  compiler against the actual body, never silently re-derived;
- a contract changes only through `ChangePortContract` (classified as a
  refinement when the new contract refines the old — FV `IfaceRefines`, same
  signature and clock, commitments in the right direction — and an `Interface`
  edit otherwise, naming the bindings it reopens) or `SetClockParameter` (which
  rewrites the _role_ of a clock in every contract that names it);
  `RebindPortDeclaration` moves the link, not the promise;
- binding compatibility (`contract::binding_compatibility`, FV `BindingWF`) and
  substitutability (`contract::component_substitutable`, FV `Substitutable`) are
  decided on contracts alone, over **resolved** concept identities — a shared
  concept is the system concept it stands for, a private one is
  `(instance, local)` — so equal representations never make two concepts one,
  and no body is inspected at a use site.

Contract concepts and clocks are component-local: a system `ClockId` never
appears in a contract (an instance maps a `Parameter`; a `Private` clock is
freshened). `body_stamp` and `interface_stamp` count the two kinds of change
separately. A version is a `DuplicateComponent` — same port ids, same local ids
— so `ReplaceInstanceComponent` can substitute it at an instance while every
binding, export and parameter value stays in place and the instance keeps its
flat identities.

## Edit model and invalidation

`apply_system_edit(&SystemSnapshot, &SystemEditOp) -> Result<AppliedSystem, SystemEditError>`
— pure, revisioned, the same shape as `apply_edit`. Ops: `Base(EditOp)`,
`CreateComponent`, `RenameComponent`, `DeleteComponent`,
`EditComponentBody { component, op: EditOp }`, `DeclarePort`, `RetirePort`,
`RenamePort`, `ChangePortContract`, `RebindPortDeclaration`,
`DuplicateComponent`, `ReplaceInstanceComponent`, `SetClockParameter`,
`ShareConcept`, `ExternalizeOutput`, `CreateInstance`, `RenameInstance`,
`DeleteInstance`, `SetClockArgument`, `SetParameterArgument`, `BindPorts`,
`UnbindPorts`, `ExportPort`, `HidePort`, `ExtractGroupAsComponent`,
`DeleteGroupWithMembers`. Each returns a
`SystemEditOutcome { flat: EditOutcome-shaped invalidation, touched_instances }`:
a body edit invalidates every instance of that component (their flattened
declarations are the origin decls); a rename invalidates nothing semantic; a
binding change is `Realization` (+ `Reactive`, `Clock`, `Output` downstream); a
clock argument is `Clock`; a parameter argument is `Realization`; a contract
change is `Interface` and lists the bindings on the port (`bindings`) and the
instances of the component; a private body formula edit lists the instances only
and touches no binding. Unrelated components are never invalidated.

## Acceptance levels

`SystemAnalysis.acceptance ∈ { Invalid, Open, Executable }`: a composition
diagnostic or a flat error is `Invalid`; an unbound required port is `Open` (an
ordinary unresolved declaration — FV Theorem H); `Executable` needs every
required port bound or exported, and the existing `bdl_compiler::readiness`
(causal, clock-consistent, outputs complete). No new acceptance machinery.

## Packaging

`package_system(&system, interface: PackageInterface) -> BehaviorComponent`
flattens, validates that every chosen port is a declaration of the flattened
design with the advertised shape (required ⇒ unresolved, provided ⇒ present,
parameter ⇒ unresolved and nullary), and returns a component whose body is the
flattened design and whose private/shared partition is inherited (base concepts
of the inner system become the package's shared concepts if they came from a
system concept). Hierarchy is instantiating a package (FVD-0072); no recursive
system type.

## 12. Behaviour groups, boundaries, extraction (Phase 8b; ADR-0019)

`BehaviorSystem.groups: BTreeMap<BehaviorGroupId, BehaviorGroup { id, name, description, members: Vec<DeclId> }>`
— authoring metadata over the base design's relationships, each in at most one
group (DI-36). Nothing below the edit model reads it: `flatten`,
`validate_composition` and `analyze_system` are the same functions on a grouped
and an ungrouped system, so every judgment is literally the ungrouped design's
(FV Theorems A–G).

**Group edits are not revisions.**
`group::apply_group_edit(&BehaviorSystem, &GroupEditOp) -> (BehaviorSystem, GroupEditOutcome)`
has no revision, no invalidation set and no undo entry; the daemon applies it
outside the revision stream (`ApplyGroupEdit`, DI-38), bumps
`SystemState.authoring_generation`, and answers with the `SystemView` — no
`ProjectChanged`, no `AnalysisReady`. A semantic edit that deletes a member
prunes it from its group (`prune_groups`); the group itself stays.
`DeleteGroupWithMembers` and `ExtractGroupAsComponent` are ordinary revisioned
system edits.

**Boundary as projection.**

```text
boundary::group_boundary(base, analysis, members) -> GroupBoundary { members, crossing_in, crossing_out, open_members, driven_members, private_candidates, external_inputs, external_outputs, clocks, internal_edges }
```

It reads the flat `DependencyGraph` (base ids are preserved by flattening,
DI-39) and the IR (for `sync` domains, DI-41). It is computed per group in
`analyze_system` (`SystemAnalysis.groups`) and, for every authoring generation,
by the daemon from the revision's cached analysis (`SystemView.boundaries`).
Studio draws it; it never computes it.

**Extraction.** `extract::preview_extraction(snapshot, group, choices)` and
`extract::extract_group(...)` (called by `ExtractGroupAsComponent`):

```text
component.body      = members (unchanged) ∪ unresolve(crossing_in)   -- FV restrict(isMember, isCrossIn)
component.interface = required: crossing_in ∪ open members chosen as inputs;
                      provided: crossing_out; clock_params: every clock used
component.shared    = every concept the body mentions (same ids)
component.external  = driven sinks left outside (same ids); moved sinks become private
base                = base \ members ∪ unresolve(crossing_out)         -- FV restrict(isOutside, isCrossOut)
instance            = one, clocks mapped identically (c ↦ c)
bindings            = Base(r) → Port(inst, req_r) for r ∈ crossing_in;
                      Port(inst, prov_p) → Base(p) for p ∈ crossing_out
```

Identities are kept: members and crossing-in copies have the same `DeclId`
inside the body as they had in the base (the body's allocator is moved past them
with `Design::reserve_ids`); the instance's flat ids come from the freshening
table as for any instance. The group is retired. Causality is the flat
analysis's verdict afterwards — no coarse instance graph gates the extraction.

**Component-scoped drafts.** The daemon keeps one `IdeHost` per component body
on demand (`SystemState.component_ide`), re-seated on every commit; draft
requests carry an optional `component` so completion, hover and verdicts inside
a component's source use the body's names and analysis (DI-40).
`SystemAnalysis.component_analyses` holds each body's standalone analysis for
the source view.

## 13. Scoped groups, and why nested packaging waits (architecture note)

`GroupScope { SystemBase | Component { component } }` names the authored design
a group organises. Both designs are ordinary `Design`s, so the same
`group_boundary` serves both — off the flat analysis for the base (base ids are
preserved by flattening), off the body's standalone analysis for a component
(`SystemAnalysis.component_analyses`, the per-body `IdeHost` in the daemon).
Nothing about a component changes when its body is grouped: the group table
lives on the system, beside the components.

**Component-local "Package as component": deferred (option A).** The production
model has no nested system: a `BehaviorComponent.body` is a flat `Design`, and
hierarchy is packaging a flattened system into a component (FVD-0072).
Extracting a group _inside_ a body would have to produce a component _and an
instance inside the parent's body_; the parent's body cannot hold an instance,
so option B would lower the instance immediately by re-flattening it into the
parent's flat body (the Phase-8a `toComponent` strategy). That works, but the
result is two truths of one behaviour — the new component and the flattened copy
inside the parent — with no authored link between them: editing the component
would not reach the parent, and the parent's body would carry generated
`ScopedFormula`s and `Reference`s as if authored. That is exactly the "second
semantic truth" this architecture refuses (§9, §10). Until a hierarchical
authoring layer exists (a body that is itself a system, flattened recursively,
with provenance through both levels), a group inside a component is organisation
only, and `preview_extraction` refuses it with `extract.not_a_base_group`.
Grouping itself is not blocked by this: every group operation, boundary and
canvas behaviour works in both scopes.

**One history.** `SystemState.undo/redo: Vec<HistoryEntry>` interleaves
`Semantic(Box<BehaviorSystem>)` and `Authoring(groups)` entries in the order
they happened; `Undo`/`Redo` on a system project answer `SystemEditApplied` (the
system alongside) and push a `ProjectChanged` only for a semantic step. Group
edits therefore undo like any authored action without turning into semantic
edits.

**Stale metadata edits.** `ApplyGroupEditRequest.base_generation` is the
generation the client saw; a mismatch is `group_edit.stale_generation` and the
client re-fetches the system. The generation is project-level: one table, one
counter, and it invalidates only authoring projections (the group views,
boundaries and layout Studio derives), never a revision.
