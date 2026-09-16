# Behaviour systems

Reusable behaviour components, instantiated with fresh identity, bound by
identity, composed into a system, and flattened into the same flat BDL
design the existing compiler, simulator, deployment analysis and backend
already accept. The production milestone of the formal Phase 8a
(`BDL_FV` bf19162: `BDL/Behavior/`, `BEHAVIOR_NOTE.md`,
`BEHAVIOR_SYSTEM_REQUIREMENTS.md`, D-64..D-73). Implementation design:
`docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md`. Crate: `crates/bdl-system`.

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
| `BehaviorInterface { ports: Port { kind: Required \| Provided \| Parameter, decl }, clock_params }` | `BehaviorInterface { required, provided, params, clockParams }`, `Port = (DeclId, DeclInterface, Option ClockId)` (Interface.lean) |
| `ComponentInstance { component, clock_bindings }` | `Inst { comp, κ }` (Instantiate.lean); the instance index is a stable `ComponentInstanceId` |
| `FlatIds` — one flat id per `(instance, private local entity)`, issued by the base allocator; `flatten` reads it | `Ren.inst C W k κ`, `fresh W k n`, `decode` (Instantiate.lean); **Theorem A** `inst_decl_disjoint`, `inst_sem_disjoint`, `inst_out_disjoint`, `inst_clock_disjoint`, `inst_*_not_global` — production discharges injectivity and disjointness by the allocator's never-reuse contract (tested: `instances_never_alias_and_shared_identity_is_kept`) |
| `validate_composition`: `system.binding_type_mismatch`, `system.binding_needs_transport`, `system.transport_*`, `system.port_not_open`, `system.parameter_not_a_value` | `BindingWF` (System.lean): type equality after renaming, clock compatibility or explicit transport with a closed init; `Realizes` (Component.lean): required/params unresolved, params data-typed |
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

## What the surface adds, and what it does not

Added: `ComponentId`, `ComponentInstanceId`, `PortId`, `BindingId`,
`ExportId`; the authored `BehaviorSystem`; twenty system edit ops; the
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
* **Hierarchical Studio canvas.** Studio receives `SystemView` and can show
  the derived flat design (marked `PROJECT_KIND_SYSTEM`, with a reference
  definition rendered as an empty formula by the current inspector), but
  has no component/instance/port editor, nested navigation, or grouping.
  The next milestone.
* **Package marketplace / recursive distribution.** None.
* **Semantic equivalence beyond the FV fragment.** Theorem J covers
  single-domain wiring with direct and constant bindings; the multi-domain
  case with transported bindings and higher-order bodies is not proved,
  and production claims only the differential tests it runs.
* **Behavioural substitutability.** `IfaceRefines` / `substitute_composeWF`
  (interface refinement) is not implemented; there is no versioning of
  components beyond the `stamp`, and no claim of behavioural equivalence
  between a component and a refinement of it.
* **Layout** for system projects is keyed by flat ids in `ui/layout.json`;
  a per-instance canvas layout is part of the Studio milestone.
* **IDE surfaces** (hover, completion, rename, references) operate on the
  derived flat design; a reference definition hovers as "bound to
  sensor.tiltValue" and scoped formulas complete over their pinned names,
  but renaming a body relationship through the flat design is refused
  (the flat design is derived) — rename in the component body instead.
