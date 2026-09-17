---
kind: evidence
area: behavior-systems
status: current
---
# Behaviour systems

Reusable behaviour components, instantiated with fresh identity, bound by
identity, composed into a system, and flattened into the same flat BDL
design the existing compiler, simulator, deployment analysis and backend
already accept. The production milestone of the formal Phase 8a
(`BDL_FV` bf19162: `BDL/Behavior/`, `BEHAVIOR_NOTE.md`,
`BEHAVIOR_SYSTEM_REQUIREMENTS.md`, D-64..D-73). Implementation design:
`docs/architecture/behavior-systems.md`. Crate: `crates/bdl-system`.

```
BehaviorSystem  ──flatten──▶  ProjectSnapshot  ──▶  analyze · simulate · deploy · lower · codegen
 (authored)                     (derived)                          (existing, unchanged)
```

There is one BDL. A product is a component at one scale; a system is
composition at a larger scale; both elaborate into the same five kernel
judgments. **No kernel construct was added** (`bdl-ir`, `bdl-check`,
`bdl-reactive`, `bdl-exec-ir` are untouched by this milestone). What was
added to the flat surface is two identity-bearing definition forms —
`Definition::Reference` (a binding: `declRef` / `sync`) and
`Definition::ScopedFormula` (a component formula with its names pinned to
identities) — and their elaboration; both are produced by flattening, never
authored.

## Production ↔ FV correspondence

*Implementation correspondence*, not a proof: each row names the production
object or function and the formal definition or theorem it follows.

| Production (`bdl-system`) | FV (`BDL/Behavior/`) |
|---|---|
| `BehaviorComponent { body: Design, interface, shared_concepts, external_outputs }` | `BehaviorComponent { design, iface, internalSem, internalOut, width }` (Component.lean) |
| `BehaviorInterface { ports: Port { kind: Required \| Provided \| Parameter, decl, contract: PortContract { signature, commitments, clock: Agnostic \| Parameter \| Private } }, clock_params }` — the contract is stored on the port, never read off the body | `BehaviorInterface { required, provided, params, clockParams }`, `Port = (DeclId, DeclInterface, Option ClockId)` (Interface.lean) |
| `contract::realizes(component)` → `component.port_declaration_missing`, `component.port_signature_mismatch`, `component.port_clock_mismatch`, `component.required_port_realized`, `component.parameter_invalid`, `component.port_commitment_unrealized`, … — decided on the component alone, once, for every instance | `BehaviorComponent.Realizes` (Component.lean): every port is a declaration with exactly the advertised interface and clock, required ports and parameters unresolved, parameters data-typed and agnostic |
| `contract::binding_compatibility(system, binding)` — resolved contract signatures equal (`ResolvedConcept::Shared(system id)` / `Private{instance, local}`), commitments included, clocks equal / agnostic source / explicit transport | `BindingWF` on the two interfaces (System.lean; D-68) |
| `PortContract::is_refined_by`, `contract::component_substitutable(old, new, used_ports, used_clock_params)`, `ReplaceInstanceComponent` | `IfaceRefines`, `Substitutable`, `replace`, `substitute_composeWF` (Substitution.lean): existing wiring stays well formed; **no behavioural claim** |
| `ComponentInstance { component, clock_bindings }` | `Inst { comp, κ }` (Instantiate.lean); the instance index is a stable `ComponentInstanceId` |
| `FlatIds` — one flat id per `(instance, private local entity)`, issued by the base allocator; `flatten` reads it | `Ren.inst C W k κ`, `fresh W k n`, `decode` (Instantiate.lean); **Theorem A** `inst_decl_disjoint`, `inst_sem_disjoint`, `inst_out_disjoint`, `inst_clock_disjoint`, `inst_*_not_global` — production discharges injectivity and disjointness by the allocator's never-reuse contract (tested: `instances_never_alias_and_shared_identity_is_kept`) |
| `validate_composition` = `realizes` for every component + `binding_compatibility` for every binding, phrased as `component.*` / `system.binding_*` / `system.transport_*` diagnostics | `ComposeWF` (System.lean): `InstsWF` (every instance's component realizes) and `BindingWF` for every binding |
| binding = `Definition::Reference { target, transport }` on the destination, elaborated to `declRef target` / `sync src init (declRef target)` (`bdl-elab::elaborate_reference`) | `applyBinding` = a Phase-1 `realize` step with `bindingBody` (System.lean); **Theorem C** `binding_satisfies` — the existing checker verifies the realization against the port's interface |
| `flatten`: base ∪ instantiated fragments, then one realization per binding and per parameter value; unbound required ports left unresolved | `flatten = ⟨flattenΔ, unionΘ, unionΚ, unionΩ, unionβ⟩` (System.lean) |
| `analyze_system` = `flatten` then `bdl_compiler::analyze` (typing, `Causal`, `Clocked`, `DriveWF`, `SingleDriver`, `CompleteOutputs` on the flat design) | **Theorem D** `flatten_WF`, **E** `flatten_globalWF` (Preservation.lean) — the flat judgments are the only judgments |
| causality: the existing `bdl-reactive::causality` on the flat design; cycles projected to instances and ports | **Theorem F** `flatten_causal` under `InstAcyclic` (D-69); production runs the authoritative check and does not need the sufficient condition (test: `two_locally_causal_components_can_close_an_instantaneous_cycle`) |
| clocks: `clock_bindings` substituted at flattening; the existing `Clocked` check on the flat design; transport = `sync` with the strictly-before rule | **Theorem G** `flatten_wellClocked`, `Clocked.rename` for any κ (D-70); `MEv` unchanged (test: `cross_domain_binding_needs_transport_and_keeps_strictly_before`) |
| `PortStatus::Open`, `Acceptance::Open` — an unbound required port is `MappingStatus::Declared`, never an error | **Theorem H** `open_port_stays_open`, `Design.Open` / `Design.Executable` (test: `an_unbound_required_port_is_open_not_invalid`) |
| single driver: the existing `output.multiple_drivers` after flattening, projected to both instances | **Theorem I** `flatten_singleDriver`, `ExternalSingleDriver` (D-68; test: `two_instances_driving_one_shared_sink_hit_the_existing_single_driver_rule`) |
| `system_and_hand_written_flat_agree_on_every_observable`, `generated_programs_of_system_and_flat_agree_on_the_host`: same statuses, traces, outputs, exec-IR and generated-Rust behaviour as a hand-written flat design | **Theorem J** `modular_iff_flat` — **restricted**: proved for the single-domain semantics `Ev` on wiring designs with direct/constant bindings (D-73). The production test is a differential check of one example, not a proof; it says nothing beyond that example and the FV's fragment |
| `package_system` → `BehaviorComponent` whose body is the flattened design; `InstallComponent` renumbers it into an outer system | `toComponent`, `flatWidth` (D-72); the `Realizes` obligation for a package is checked by construction here (port shapes) — the FV leaves it as a decidable side condition |
| `shared_concepts` / `external_outputs` tables; everything else private and freshened | `internalSem` / `internalOut`, `unionΘ` / `unionΩ` keeping globals (D-66) |
| parameters: `Parameter` ports with a closed formula value (`ParameterValue`), elaborated as a nullary formula with no relationship in scope | `params`, `BindSrc.const`, `HasType.refFree_env_irrelevant` (§14 of the requirements) |
| `SystemEditOutcome { invalidates, origin_decls (flat), instances }` | Phase-1 refinement-vs-edit discipline extended to instances (§44 of the brief) |

### Phase 8b — grouping, boundaries, packaging (`BDL_FV` cf2fc5e; ADR-0019)

| Production (`bdl-system`) | FV (`BDL/Behavior/`) |
|---|---|
| `BehaviorGroup { id, scope: GroupScope { SystemBase \| Component }, name, description, members: Vec<DeclId> }` on `BehaviorSystem.groups`; never read by `flatten`, `validate_composition` or any analysis; members validated against the scope's design (the system's own, or a component body) | `BehaviorGroup { id, members }`, `GroupedDesign { design, groups }` for *any* design, `eraseGroups` (Group.lean) — production first supported base-only groups; the scoped-groups milestone generalises the scope (ADR-0019 amendment) |
| `group::apply_group_edit` — `CreateGroup`, `DeleteGroup`, `AddMember`, `RemoveMember`, `MoveMember`, `MergeGroups`, `SplitGroup` (+ rename / describe); no revision, no invalidation set, an *authoring generation* instead | `group`, `ungroup`, `addMember`, `removeMember`, `move`, `merge`, `split` — **Theorems A–G**: `eraseGroups` of every operation is the design itself (`rfl`); every kernel judgment (`WF`, `Causal`, `WellClocked`, `DriveWF`, `SingleDriver`, `CompleteOutputs`, `Ev`) is the ungrouped design's (test `grouping_is_semantically_transparent`: same flat design value, diagnostics, dependency graph, clocks, outputs, simulation trace and deployment requirements across every operation) |
| `boundary::group_boundary(base, flat analysis, members)` → `crossing_in`, `crossing_out`, `open_members`, `driven_members`, `private_candidates`, `external_inputs`, `external_outputs`, `clocks`, `internal_edges` — off `bdl-reactive::DependencyGraph` | `Boundary.crossIn`, `crossOut`, `openMembers`, `drivenMembers`, `privateMembers`, `externalInputs = crossIn ++ openMembers`, `externalOutputs = crossOut`, `InternalEdge`, `clocksOf` (Boundary.lean); **Theorems I–L** `mem_crossIn`, `mem_crossOut`, `internal_not_crossIn`, `internal_only_not_crossOut` |
| aggregate sockets of a collapsed group in Studio are these lists drawn; `canLink` refuses them; the dependency edges stay between declarations | **Theorem H** `socket_no_fanout`: membership in the input socket adds no `DependsOn` edge (test `boundary_is_a_projection_and_sockets_add_no_dependency`) |
| `clocks` include domains observed only through `sync` (`collect_sync_clocks` over the IR) | `clocksOf` = every clock the design uses; BEHAVIOR_GROUPING_NOTE §II.4 (test `clock_coverage_includes_domains_read_only_through_sync`) |
| `extract::extract_group`: body = members unchanged + unresolved copies of crossing-in (required ports), provided = crossing-out, every used clock a parameter, concepts shared, sinks external (or moved inside on request), drives kept; base keeps an unresolved copy of each crossing-out member; one instance; bindings `Base(r) → Port(inst, req_r)` and `Port(inst, prov_p) → Base(p)` | `Extract.restrict`, `template`, `Input.comp`, `Input.resid`, `Input.bindings`, `Input.system` (Extract.lean); `ExtractPreservation.lean` — extraction well formed, `Causal` and `SingleDriver` preserved by subdividing edges (never gated on `InstAcyclic`; test `a_bidirectional_boundary_is_not_a_cycle`) |
| `extract::preview_extraction` — the same decisions, mutating nothing; `ExtractionChoices { name, instance_name, keep_internal, internalize_sinks }` | the four choices the FV cannot infer (BEHAVIOR_GROUPING_NOTE §II.5) |
| `BindingEnd::{Port, Base}` — the base as the residual (DI-37) | the residual as instance 0 |
| `extraction_is_a_differential_witness_of_theorem_r`, Studio `system_e2e_test` (same simulated values before and after packaging; same deployment requirements) | **Theorem R** `orig_iff_flat` — **restricted** to single-domain wiring designs without transported bindings; production claims the differential tests it runs, including a `sync`-only clock and a transported binding *after* packaging, which the theorem does not cover |

## What the surface adds, and what it does not

Added: `ComponentId`, `ComponentInstanceId`, `PortId`, `BindingId`,
`ExportId`; the authored `BehaviorSystem`; twenty-five system edit ops; the
freshening table; the origin map; composition diagnostics (`system.*`);
`Acceptance { Invalid, Open, Executable }`; packaging; a system project
format; protocol 0.6 views and requests.

Not added, by design: a kernel term for components, instances or bindings;
a component typing or clock judgment; a recursive system datatype; a
runtime parameter channel; frequency-based clock compatibility; name-based
identity anywhere; body substitution at binding (a binding is a reference,
so write-once and provenance hold).

## Identity, in production terms

| Layer | Identity | Meaning |
|---|---|---|
| authoring vocabulary | `TemplateId` (Standard Concept Library) | how a concept was *made*; two concepts from one template are two concepts |
| project semantics | `SemanticId`, `DeclId`, `ClockId`, `OutputId`, `DeviceId` | the flat design's nominal identities — the system's own, or an instance's freshened ones |
| reusable behaviour | `ComponentId`, `PortId` | a definition and its public boundary |
| occurrence | `ComponentInstanceId`, `BindingId`, `ExportId` | one use of a definition, and how it is wired |

Sharing is never inferred from names or templates: a component says which
of its concepts stand for a system concept (`ShareConcept`); everything
else is private and fresh per instance.

## Limitations (not solved in this milestone)

* **Cross-project portable packages.** Components are project-local. A
  component imported into another project would need a portable
  semantic-interface mechanism (concepts identified across projects); the
  `shared_concepts` table assumes the enclosing project's ids. Not
  started.
* **Hierarchical Studio canvas** — built (docs/architecture/studio-ui.md §11): the
  system canvas with instance nodes from contracts, bindings, group
  regions, the packaging sheet, and one component's source at a time. Not
  built: nested systems (a component whose body is itself a system — the
  model has none, D-72), multi-selection grouping gestures beyond
  drag-in/out, and a minimap.
* **Package marketplace / recursive distribution.** None.
* **Semantic equivalence beyond the FV fragment.** Theorems J and R cover
  single-domain wiring with direct and constant bindings; the multi-domain
  case with transported bindings and higher-order bodies is not proved,
  and production claims only the differential tests it runs.
* **Packaging a group inside a component** is deferred: bodies are flat
  designs, and lowering a nested extraction now would put a second
  semantic truth beside the body (docs/architecture/behavior-systems.md
  §13, option A). Grouping inside a component works fully; the inspector
  says packaging comes with nested components.
* **Boundary of a group containing an instance's port declaration**: not
  reachable from Studio (groups hold base relationships); the Rust
  boundary would handle it, extraction copies it like any crossing-in
  declaration.
* **Behavioural substitutability.** Interface-refining substitution
  (`component_substitutable`, `ReplaceInstanceComponent`) keeps the
  wiring well formed and nothing more: no behavioural equivalence between
  a component and a version of it is claimed or checked. Versions are
  `DuplicateComponent` copies (same port and local ids); an independently
  authored component cannot substitute even with equal promises (DI-34).
  Commitments (`PropertyId`) are carried in contracts but the surface
  never establishes any yet.
* **Layout** for system projects: `ui/layout.json` holds the system canvas
  (base ids, instance ids, group boxes) and one canvas per component
  body (`Layout.components`); nothing per instance — an instance is a
  node, its body has one picture.
* **IDE surfaces** (hover, completion, rename, references) operate on the
  derived flat design; a reference definition hovers as "bound to
  sensor.tiltValue" and scoped formulas complete over their pinned names,
  but renaming a body relationship through the flat design is refused
  (the flat design is derived) — rename in the component body instead.
