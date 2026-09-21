//! System elaboration: `BehaviorSystem → ProjectSnapshot` + provenance
//! (FV `flatten`, docs/architecture/behavior-systems.md §5–§8).
//!
//! 1. every instance is instantiated: its component body is copied with
//!    every private identity replaced by the instance's flat id from the
//!    freshening table (FV `Ren.inst`), shared concepts and external sinks
//!    kept as the system's, clock parameters substituted (`κ`);
//! 2. parameters are realised with their closed values;
//! 3. every binding realises its destination port with a reference to the
//!    source — `Definition::Reference`, the kernel's `declRef`, or `sync`
//!    from the source's domain when transported (FV `bindingBody`);
//! 4. unbound required ports stay unresolved (FV Theorem H);
//! 5. the origin map records where every flat entity came from.
//!
//! Nothing here allocates: the table is complete by construction (the edit
//! model completes it), and a missing entry is an internal diagnostic.
//! Nothing here checks types, clocks or causality: the flat design goes to
//! the existing compiler for that.  Composition-level checks that the flat
//! compiler cannot phrase (a port that is not open, a shared concept that
//! disagrees, a missing clock argument) are reported as `system.*`
//! diagnostics alongside.

use crate::ids::{ComponentId, ComponentInstanceId};
use crate::model::*;
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_model::surface::{
    ClockDomain, Concept, Definition, DeviceBinding, FormulaScope, MappingBlock, PhysicalOutput,
    ProjectSnapshot, Signature, Transport,
};
use bdl_model::{ClockId, ConceptId, DeclId, DeviceId, OutputId};
use std::collections::BTreeMap;

/// Where a flat entity came from.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Origin {
    pub instance: ComponentInstanceId,
    pub component: ComponentId,
    pub local: LocalEntity,
}

/// Flat identity → origin, per sort, plus the ports by their flat
/// declaration.  Entities of the base design have no origin: they are the
/// system's own.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct OriginMap {
    pub decls: BTreeMap<DeclId, Origin>,
    pub sems: BTreeMap<ConceptId, Origin>,
    pub clocks: BTreeMap<ClockId, Origin>,
    pub outputs: BTreeMap<OutputId, Origin>,
    pub devices: BTreeMap<DeviceId, Origin>,
    /// The flat declaration of every port of every instance.
    pub ports: BTreeMap<DeclId, PortRef>,
    /// Display names of instances, for labels.
    pub instance_names: BTreeMap<ComponentInstanceId, String>,
}

impl OriginMap {
    pub fn decl_of_port(&self, r: PortRef) -> Option<DeclId> {
        self.ports.iter().find(|(_, p)| **p == r).map(|(d, _)| *d)
    }
    /// `instance.local` for a flat declaration, else `None` (a base
    /// declaration).
    pub fn label_of_decl(&self, d: DeclId, local_name: &str) -> Option<String> {
        let o = self.decls.get(&d)?;
        let inst = self.instance_names.get(&o.instance)?;
        Some(format!("{inst}.{local_name}"))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlattenedSystem {
    /// The ordinary flat design, at the system's revision.  Derived: never
    /// edited, never persisted.
    pub snapshot: ProjectSnapshot,
    pub origins: OriginMap,
    /// Composition-level diagnostics (`system.*`), sorted.
    pub diagnostics: Vec<Diagnostic>,
}

/// The renaming of one instance (FV `Ren.inst`): total on the sorts the
/// table covers, `None` where the table has no entry or a clock argument
/// is missing.
struct Ren<'a> {
    system: &'a BehaviorSystem,
    instance: &'a ComponentInstance,
    component: &'a BehaviorComponent,
}

impl Ren<'_> {
    fn flat(&self, e: LocalEntity) -> Option<u64> {
        self.system.flat_ids.get(self.instance.id, e)
    }
    fn decl(&self, d: DeclId) -> Option<DeclId> {
        self.flat(LocalEntity::Decl(d)).map(DeclId::from_raw)
    }
    fn sem(&self, s: ConceptId) -> Option<ConceptId> {
        match self.component.shared_concepts.get(&s) {
            Some(g) => Some(*g),
            None => self.flat(LocalEntity::Sem(s)).map(ConceptId::from_raw),
        }
    }
    fn clock(&self, c: ClockId) -> Option<ClockId> {
        if self.component.interface.is_clock_param(c) {
            self.instance.clock_bindings.get(&c).copied()
        } else {
            self.flat(LocalEntity::Clock(c)).map(ClockId::from_raw)
        }
    }
    fn output(&self, o: OutputId) -> Option<OutputId> {
        match self.component.external_outputs.get(&o) {
            Some(g) => Some(*g),
            None => self.flat(LocalEntity::Output(o)).map(OutputId::from_raw),
        }
    }
    fn device(&self, d: DeviceId) -> Option<DeviceId> {
        self.flat(LocalEntity::Device(d)).map(DeviceId::from_raw)
    }
    fn name(&self, local: &str) -> String {
        format!("{}.{local}", self.instance.name)
    }
    fn origin(&self, local: LocalEntity) -> Origin {
        Origin {
            instance: self.instance.id,
            component: self.component.id,
            local,
        }
    }
}

pub fn flatten(snapshot: &SystemSnapshot) -> FlattenedSystem {
    let s = &snapshot.system;
    let mut design = s.base.clone();
    let mut origins = OriginMap::default();
    let mut diags: Vec<Diagnostic> = Vec::new();
    let internal = |what: String| {
        Diagnostic::error("system.internal", Entity::Project, "The system could not be assembled.")
            .explain("An instance refers to something its component no longer has, or the freshening table is incomplete. This is a compiler bug; the design itself is fine.")
            .technical(what)
    };

    for inst in s.instances.values() {
        origins.instance_names.insert(inst.id, inst.name.clone());
        let Some(component) = s.components.get(&inst.component) else {
            diags.push(internal(format!(
                "instance {} of unknown component {}",
                inst.id, inst.component
            )));
            continue;
        };
        let ren = Ren {
            system: s,
            instance: inst,
            component,
        };
        let body = &component.body;

        // Concepts: private ones are fresh; shared ones must exist in the
        // system and agree on the representation when both have one.
        for c in body.concepts.values() {
            match component.shared_concepts.get(&c.id) {
                Some(g) => match s.base.concepts.get(g) {
                    None => diags.push(
                        Diagnostic::error("system.shared_concept_missing", Entity::Project, format!("{} shares a concept the system no longer has.", inst.name))
                            .explain(format!("The component's {} stands for a system concept; choose which one, or make it private.", c.name))
                            .technical(format!("{} shared_concepts[{}] = {g}, absent from base", component.id, c.id)),
                    ),
                    Some(sys) => {
                        if let (Some(a), Some(b)) = (&c.representation, &sys.representation) {
                            if a != b {
                                diags.push(
                                    Diagnostic::error("system.shared_concept_disagrees", Entity::Concept { id: *g }, format!("{} means {} differently from the system.", inst.name, sys.name))
                                        .explain("A shared concept has one representation, the system's; the component must agree or own its own concept.")
                                        .technical(format!("{} local {} rep {a:?} ≠ system {g} rep {b:?}", component.id, c.id)),
                                );
                            }
                        }
                    }
                },
                None => match ren.sem(c.id) {
                    Some(f) => {
                        origins.sems.insert(f, ren.origin(LocalEntity::Sem(c.id)));
                        design.concepts.insert(
                            f,
                            Concept {
                                id: f,
                                name: ren.name(&c.name),
                                description: c.description.clone(),
                                representation: c.representation.clone(),
                                ordered: c.ordered,
                            },
                        );
                    }
                    None => diags.push(internal(format!("no flat id for {}.{}", inst.id, c.id))),
                },
            }
        }

        // Clocks: private ones are fresh domains; parameters need an argument.
        for k in body.clocks.values() {
            if component.interface.is_clock_param(k.id) {
                match inst.clock_bindings.get(&k.id) {
                    Some(sys) if s.base.clocks.contains_key(sys) => {}
                    Some(sys) => diags.push(
                        Diagnostic::error("system.clock_argument_unknown", Entity::Project, format!("{} is set to a timing domain the system no longer has.", inst.name))
                            .technical(format!("{} κ({}) = {sys}, absent from base", inst.id, k.id)),
                    ),
                    None => diags.push(
                        Diagnostic::error("system.clock_argument_missing", Entity::Project, format!("{} does not say which timing domain {} is.", inst.name, k.name))
                            .explain("A component written against a timing domain parameter runs in whatever domain the instance names; choose one.")
                            .technical(format!("{} κ({}) unset", inst.id, k.id)),
                    ),
                }
            } else if let Some(f) = ren.clock(k.id) {
                origins
                    .clocks
                    .insert(f, ren.origin(LocalEntity::Clock(k.id)));
                design.clocks.insert(
                    f,
                    ClockDomain {
                        id: f,
                        name: ren.name(&k.name),
                    },
                );
            } else {
                diags.push(internal(format!("no flat id for {}.{}", inst.id, k.id)));
            }
        }

        // Outputs: private sinks are fresh; external ones are the system's.
        for o in body.outputs.values() {
            match component.external_outputs.get(&o.id) {
                Some(g) => match s.base.outputs.get(g) {
                    None => diags.push(
                        Diagnostic::error(
                            "system.external_output_missing",
                            Entity::Project,
                            format!("{} drives an output the system no longer has.", inst.name),
                        )
                        .technical(format!(
                            "{} external_outputs[{}] = {g}, absent from base",
                            component.id, o.id
                        )),
                    ),
                    Some(sys) => {
                        let accepts = ren.sem(o.accepts);
                        let clock = o.clock.and_then(|c| ren.clock(c));
                        if accepts != Some(sys.accepts) || (o.clock.is_some() && clock != sys.clock)
                        {
                            diags.push(
                                Diagnostic::error("system.external_output_disagrees", Entity::Project, format!("{} expects {} to accept something else.", inst.name, sys.name))
                                    .explain("An external output is the system's: the component's view of what it accepts and when must match.")
                                    .technical(format!("{} local {} accepts {accepts:?} clock {clock:?} vs system {g} accepts {} clock {:?}", component.id, o.id, sys.accepts, sys.clock)),
                            );
                        }
                    }
                },
                None => match (ren.output(o.id), ren.sem(o.accepts)) {
                    (Some(f), Some(accepts)) => {
                        origins
                            .outputs
                            .insert(f, ren.origin(LocalEntity::Output(o.id)));
                        design.outputs.insert(
                            f,
                            PhysicalOutput {
                                id: f,
                                name: ren.name(&o.name),
                                description: o.description.clone(),
                                accepts,
                                clock: o.clock.and_then(|c| ren.clock(c)),
                                required: o.required,
                            },
                        );
                    }
                    _ => diags.push(internal(format!("no flat id for {}.{}", inst.id, o.id))),
                },
            }
        }

        // Devices: always private (deployment multiplies them per instance).
        for d in body.devices.values() {
            match ren.device(d.id) {
                Some(f) => {
                    origins
                        .devices
                        .insert(f, ren.origin(LocalEntity::Device(d.id)));
                    design.devices.insert(
                        f,
                        DeviceBinding {
                            id: f,
                            name: ren.name(&d.name),
                            kind: d.kind,
                            output: d.output.and_then(|o| ren.output(o)),
                            realization: d.realization.clone(),
                            source: d.source.and_then(|s| ren.decl(s)),
                            provider: d.provider.clone(),
                            fixed_pins: d.fixed_pins.clone(),
                        },
                    );
                }
                None => diags.push(internal(format!("no flat id for {}.{}", inst.id, d.id))),
            }
        }

        // Mappings: fresh ids, renamed signatures, pinned formula scopes.
        let scope_mappings: BTreeMap<String, DeclId> = body
            .mappings
            .values()
            .filter_map(|m| ren.decl(m.id).map(|f| (m.name.clone(), f)))
            .collect();
        let scope_concepts: BTreeMap<String, ConceptId> = body
            .concepts
            .values()
            .filter_map(|c| ren.sem(c.id).map(|f| (c.name.clone(), f)))
            .collect();
        let concept_name = |id: ConceptId| {
            body.concepts
                .get(&id)
                .map(|c| c.name.clone())
                .unwrap_or_default()
        };
        for m in body.mappings.values() {
            let Some(f) = ren.decl(m.id) else {
                diags.push(internal(format!("no flat id for {}.{}", inst.id, m.id)));
                continue;
            };
            let inputs: Option<Vec<ConceptId>> =
                m.signature.inputs.iter().map(|c| ren.sem(*c)).collect();
            let (Some(inputs), Some(output)) = (inputs, ren.sem(m.signature.output)) else {
                diags.push(internal(format!(
                    "signature of {}.{} names an unknown concept",
                    inst.id, m.id
                )));
                continue;
            };
            let definition = match &m.definition {
                None => None,
                Some(Definition::Formula { source }) => Some(Definition::ScopedFormula {
                    source: source.clone(),
                    scope: FormulaScope {
                        // The body's own names for its inputs: its textual
                        // parameters where it has them, the concepts'
                        // names otherwise (TEXTUAL_SYNTAX §14.4).
                        inputs: m
                            .signature
                            .inputs
                            .iter()
                            .enumerate()
                            .map(|(i, c)| {
                                m.parameters
                                    .get(i)
                                    .filter(|p| !p.is_empty())
                                    .cloned()
                                    .unwrap_or_else(|| concept_name(*c))
                            })
                            .collect(),
                        mappings: scope_mappings.clone(),
                        concepts: scope_concepts.clone(),
                    },
                }),
                // A packaged body already carries pinned scopes over its own
                // (local) ids: rename them through this instance.
                Some(Definition::ScopedFormula { source, scope }) => {
                    Some(Definition::ScopedFormula {
                        source: source.clone(),
                        scope: FormulaScope {
                            inputs: scope.inputs.clone(),
                            mappings: scope
                                .mappings
                                .iter()
                                .filter_map(|(n, d)| ren.decl(*d).map(|f| (n.clone(), f)))
                                .collect(),
                            concepts: scope
                                .concepts
                                .iter()
                                .filter_map(|(n, c)| ren.sem(*c).map(|f| (n.clone(), f)))
                                .collect(),
                        },
                    })
                }
                Some(Definition::Reference { target, transport }) => match ren.decl(*target) {
                    Some(t) => Some(Definition::Reference {
                        target: t,
                        transport: transport.as_ref().and_then(|tr| {
                            ren.clock(tr.source).map(|source| Transport {
                                source,
                                init: tr.init.clone(),
                            })
                        }),
                    }),
                    None => {
                        diags.push(internal(format!(
                            "reference in {}.{} to unknown {target}",
                            inst.id, m.id
                        )));
                        None
                    }
                },
            };
            origins.decls.insert(f, ren.origin(LocalEntity::Decl(m.id)));
            if let Some(p) = component.interface.port_for_decl(m.id) {
                origins.ports.insert(
                    f,
                    PortRef {
                        instance: inst.id,
                        port: p.id,
                    },
                );
            }
            design.mappings.insert(
                f,
                MappingBlock {
                    id: f,
                    name: ren.name(&m.name),
                    description: m.description.clone(),
                    signature: Signature { inputs, output },
                    definition,
                    clock: m.clock.and_then(|c| ren.clock(c)),
                    drives: m.drives.and_then(|o| ren.output(o)),
                    parameters: m.parameters.clone(),
                },
            );
        }

        // Parameters: closed values realise their (open) port declarations.
        for (port_id, value) in &inst.parameter_bindings {
            let Some(port) = component.interface.ports.get(port_id) else {
                diags.push(internal(format!(
                    "{} values unknown port {port_id}",
                    inst.id
                )));
                continue;
            };
            let Some(f) = ren.decl(port.decl) else {
                continue;
            };
            realise(
                &mut design,
                f,
                Definition::ScopedFormula {
                    source: value.source.clone(),
                    scope: FormulaScope::default(),
                },
            );
        }
    }

    // Bindings: one realization step each (FV `applyBinding`).  A base end
    // is its own flat declaration; a port end is the instance's flat copy.
    for b in s.bindings.values() {
        let flat_of = |e: BindingEnd| -> Option<DeclId> {
            match e {
                BindingEnd::Base { decl } => Some(decl),
                BindingEnd::Port(r) => {
                    let p = s.port(r)?;
                    s.flat_ids
                        .get(r.instance, LocalEntity::Decl(p.decl))
                        .map(DeclId::from_raw)
                }
            }
        };
        let label = |e: BindingEnd| -> String {
            match e {
                BindingEnd::Base { decl } => s
                    .base
                    .mappings
                    .get(&decl)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| decl.to_string()),
                BindingEnd::Port(r) => {
                    let i = s.instances.get(&r.instance).map(|i| i.name.as_str());
                    let p = s.port(r).map(|p| p.name.as_str());
                    format!("{}.{}", i.unwrap_or("?"), p.unwrap_or("?"))
                }
            }
        };
        if let (BindingEnd::Port(sr), BindingEnd::Port(dr)) = (b.source, b.destination) {
            if s.port(sr).is_none() || s.port(dr).is_none() {
                diags.push(internal(format!("binding {} names a missing port", b.id)));
                continue;
            }
        }
        let (Some(src), Some(dst)) = (flat_of(b.source), flat_of(b.destination)) else {
            diags.push(internal(format!(
                "binding {} has no flat declarations",
                b.id
            )));
            continue;
        };
        let transport = match &b.transport {
            None => None,
            Some(t) => match design.mappings.get(&src).and_then(|m| m.clock) {
                Some(source) => Some(Transport {
                    source,
                    init: t.init.clone(),
                }),
                None => {
                    diags.push(
                        Diagnostic::error("system.transport_without_source_domain", Entity::Mapping { id: dst }, format!("{} is carried across domains, but {} has no timing domain.", label(b.destination), label(b.source)))
                            .explain("Transport reads the source's last activation strictly before now; a source with no domain has no activations. Bind it directly instead.")
                            .technical(format!("binding {}: transport with Κ src = none", b.id)),
                    );
                    None
                }
            },
        };
        realise(
            &mut design,
            dst,
            Definition::Reference {
                target: src,
                transport,
            },
        );
    }

    sort_diagnostics(&mut diags);
    FlattenedSystem {
        snapshot: ProjectSnapshot {
            revision: snapshot.revision,
            design,
        },
        origins,
        diagnostics: diags,
    }
}

/// Realise a flat declaration.  A declaration that is not open keeps its
/// own definition (write-once); `validate` reports it.
fn realise(design: &mut bdl_model::surface::Design, flat: DeclId, definition: Definition) {
    if let Some(m) = design.mappings.get_mut(&flat) {
        if m.definition.is_none() {
            m.definition = Some(definition);
        }
    }
}
