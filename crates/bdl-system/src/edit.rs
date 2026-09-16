//! Revisioned, pure system edits: `snapshot + SystemEditOp → new snapshot +
//! SystemEditOutcome`, the same discipline as `bdl_model::edit`.  Nothing
//! is mutated outside this module.
//!
//! Two things happen on every successful edit besides the op itself:
//!
//! * the **freshening table** is completed — every private entity of every
//!   instance has a flat id, allocated from the base allocator in `BTreeMap`
//!   order (docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md §5) — so `flatten` never
//!   allocates and never renumbers;
//! * the outcome says which instances the edit reaches, so invalidation is
//!   per instance of the touched component and never spills to unrelated
//!   components (§44 of the brief).

use crate::ids::{BindingId, ComponentId, ComponentInstanceId, ExportId, PortId};
use crate::model::*;
use bdl_model::edit::{apply_edit, EditError, EditKind, EditOp, EditOutcome, Invalidation};
use bdl_model::surface::{Design, ProjectSnapshot};
use bdl_model::{ClockId, DeclId, OutputId, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "edit", rename_all = "snake_case")]
pub enum SystemEditOp {
    /// An ordinary flat edit of the system's own design (shared concepts,
    /// system domains, external sinks, devices, top-level relationships).
    Base {
        op: EditOp,
    },

    CreateComponent {
        name: String,
        #[serde(default)]
        description: String,
    },
    RenameComponent {
        id: ComponentId,
        name: String,
    },
    SetComponentDescription {
        id: ComponentId,
        description: String,
    },
    /// Refused while the component is instantiated.
    DeleteComponent {
        id: ComponentId,
    },
    /// Add a component built elsewhere — a packaged system (`package_system`)
    /// or an imported definition.  Its ids are renumbered from this
    /// system's allocator; its body is taken as is.
    InstallComponent {
        component: Box<BehaviorComponent>,
    },
    /// An ordinary flat edit of a component's body, with its own allocator.
    EditComponentBody {
        component: ComponentId,
        op: EditOp,
    },

    /// Expose a body declaration as a port.
    DeclarePort {
        component: ComponentId,
        decl: DeclId,
        kind: PortKind,
        name: String,
        #[serde(default)]
        description: String,
    },
    RenamePort {
        component: ComponentId,
        port: PortId,
        name: String,
    },
    /// Refused while the port is bound, exported or given a parameter value.
    RetirePort {
        component: ComponentId,
        port: PortId,
    },
    /// Make a body clock a parameter of the interface (or private again).
    SetClockParameter {
        component: ComponentId,
        clock: ClockId,
        parameter: bool,
    },
    /// Make a body concept stand for a system concept (`None`: private again).
    ShareConcept {
        component: ComponentId,
        local: SemanticId,
        system: Option<SemanticId>,
    },
    /// Make a body sink stand for a system sink (`None`: private again).
    ExternalizeOutput {
        component: ComponentId,
        local: OutputId,
        system: Option<OutputId>,
    },

    CreateInstance {
        component: ComponentId,
        name: String,
    },
    RenameInstance {
        id: ComponentInstanceId,
        name: String,
    },
    /// Refused while a binding or export refers to the instance.
    DeleteInstance {
        id: ComponentInstanceId,
    },
    SetClockArgument {
        instance: ComponentInstanceId,
        parameter: ClockId,
        clock: Option<ClockId>,
    },
    SetParameterArgument {
        instance: ComponentInstanceId,
        port: PortId,
        value: Option<ParameterValue>,
    },

    BindPorts {
        source: PortRef,
        destination: PortRef,
        #[serde(default)]
        transport: Option<BindingTransport>,
    },
    UnbindPorts {
        binding: BindingId,
    },
    ExportPort {
        port: PortRef,
        name: String,
    },
    HidePort {
        export: ExportId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "refused", rename_all = "snake_case")]
pub enum SystemEditError {
    #[error("a name is required")]
    EmptyName,
    #[error("a component named `{name}` already exists")]
    DuplicateComponentName { name: String },
    #[error("unknown component {id}")]
    UnknownComponent { id: ComponentId },
    #[error("component {id} is instantiated {} time(s)", instances.len())]
    ComponentInUse {
        id: ComponentId,
        instances: Vec<ComponentInstanceId>,
    },
    #[error("unknown instance {id}")]
    UnknownInstance { id: ComponentInstanceId },
    #[error("an instance named `{name}` already exists")]
    DuplicateInstanceName { name: String },
    #[error("instance {id} is still referred to by {} binding(s)/export(s)", bindings.len() + exports.len())]
    InstanceInUse {
        id: ComponentInstanceId,
        bindings: Vec<BindingId>,
        exports: Vec<ExportId>,
    },
    #[error("unknown port {port} of component {component}")]
    UnknownPort {
        component: ComponentId,
        port: PortId,
    },
    #[error("a port named `{name}` already exists on the component")]
    DuplicatePortName { name: String },
    #[error("declaration {decl} is not in the component body")]
    NotABodyDeclaration { decl: DeclId },
    #[error("declaration {decl} is already exposed as port {port}")]
    DeclAlreadyExposed { decl: DeclId, port: PortId },
    #[error("port {port} is still in use")]
    PortInUse { port: PortId },
    #[error("port {port} backs a body declaration this edit would remove")]
    PortBacked { port: PortId },
    #[error("clock {clock} is not in the component body")]
    NotABodyClock { clock: ClockId },
    #[error("clock parameter {clock} is still assigned by {} instance(s)", instances.len())]
    ClockParamInUse {
        clock: ClockId,
        instances: Vec<ComponentInstanceId>,
    },
    #[error("concept {id} is not in the component body")]
    NotABodyConcept { id: SemanticId },
    #[error("output {id} is not in the component body")]
    NotABodyOutput { id: OutputId },
    #[error("unknown system concept {id}")]
    UnknownSystemConcept { id: SemanticId },
    #[error("unknown system output {id}")]
    UnknownSystemOutput { id: OutputId },
    #[error("unknown system timing domain {id}")]
    UnknownSystemClock { id: ClockId },
    #[error("{clock} is not a clock parameter of the instance's component")]
    NotAClockParameter { clock: ClockId },
    #[error("port {port} is not a parameter")]
    NotAParameter { port: PortId },
    #[error("a binding source must be a provided port")]
    SourceNotProvided { port: PortRef },
    #[error("a binding destination must be a required port or a parameter")]
    DestinationNotRequired { port: PortRef },
    #[error("the destination port is already bound by {binding}")]
    DestinationBound { binding: BindingId },
    #[error("the port is exported as a system input ({export})")]
    PortExported { export: ExportId },
    #[error("the port has a parameter value")]
    PortHasValue,
    #[error("unknown binding {id}")]
    UnknownBinding { id: BindingId },
    #[error("unknown export {id}")]
    UnknownExport { id: ExportId },
    #[error("an export named `{name}` already exists")]
    DuplicateExportName { name: String },
    #[error("only a required port can be exported")]
    ExportNotRequired { port: PortRef },
    #[error(transparent)]
    Base(EditError),
    #[error("in the body of component {component}: {error}")]
    Body {
        component: ComponentId,
        error: EditError,
    },
}

/// What a system edit did.  `invalidates` and `origin_decls` are in the
/// **flattened** design's terms so the existing invalidation model applies
/// unchanged; `instances` are the instances the edit reaches.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SystemEditOutcome {
    pub kind: Option<EditKind>,
    pub invalidates: BTreeSet<Invalidation>,
    pub origin_decls: BTreeSet<DeclId>,
    pub instances: BTreeSet<ComponentInstanceId>,
    pub created_component: Option<ComponentId>,
    pub created_instance: Option<ComponentInstanceId>,
    pub created_port: Option<PortId>,
    pub created_binding: Option<BindingId>,
    pub created_export: Option<ExportId>,
    /// The flat outcome of a `Base` or `EditComponentBody` op, as the flat
    /// model reported it (body ids are local).
    pub inner: Option<EditOutcome>,
}

impl SystemEditOutcome {
    fn refinement() -> Self {
        SystemEditOutcome {
            kind: Some(EditKind::Refinement),
            ..Default::default()
        }
    }
    fn edit(invalidates: impl IntoIterator<Item = Invalidation>) -> Self {
        SystemEditOutcome {
            kind: Some(EditKind::Edit),
            invalidates: invalidates.into_iter().collect(),
            ..Default::default()
        }
    }
    fn touching(mut self, i: Invalidation) -> Self {
        self.invalidates.insert(i);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedSystem {
    pub snapshot: SystemSnapshot,
    pub outcome: SystemEditOutcome,
}

fn valid_name(name: &str) -> Result<String, SystemEditError> {
    let n = name.trim();
    if n.is_empty() {
        Err(SystemEditError::EmptyName)
    } else {
        Ok(n.to_owned())
    }
}

fn component_mut(
    s: &mut BehaviorSystem,
    id: ComponentId,
) -> Result<&mut BehaviorComponent, SystemEditError> {
    s.components
        .get_mut(&id)
        .ok_or(SystemEditError::UnknownComponent { id })
}

fn instance_mut(
    s: &mut BehaviorSystem,
    id: ComponentInstanceId,
) -> Result<&mut ComponentInstance, SystemEditError> {
    s.instances
        .get_mut(&id)
        .ok_or(SystemEditError::UnknownInstance { id })
}

fn port_of(s: &BehaviorSystem, r: PortRef) -> Result<Port, SystemEditError> {
    let inst = s
        .instances
        .get(&r.instance)
        .ok_or(SystemEditError::UnknownInstance { id: r.instance })?;
    s.components
        .get(&inst.component)
        .and_then(|c| c.interface.ports.get(&r.port))
        .cloned()
        .ok_or(SystemEditError::UnknownPort {
            component: inst.component,
            port: r.port,
        })
}

/// Every private entity of a component body, in a fixed order.
pub fn private_entities(c: &BehaviorComponent) -> Vec<LocalEntity> {
    let b = &c.body;
    let mut out = Vec::new();
    out.extend(b.mappings.keys().map(|d| LocalEntity::Decl(*d)));
    out.extend(
        b.concepts
            .keys()
            .filter(|s| c.is_private_concept(**s))
            .map(|s| LocalEntity::Sem(*s)),
    );
    out.extend(
        b.clocks
            .keys()
            .filter(|k| c.is_private_clock(**k))
            .map(|k| LocalEntity::Clock(*k)),
    );
    out.extend(
        b.outputs
            .keys()
            .filter(|o| c.is_private_output(**o))
            .map(|o| LocalEntity::Output(*o)),
    );
    out.extend(b.devices.keys().map(|d| LocalEntity::Device(*d)));
    out
}

/// Complete the freshening table: allocate a flat id for every private
/// entity of every instance that has none, and drop entries of instances
/// or entities that no longer exist.  Existing entries are never touched.
pub fn ensure_flat_ids(s: &mut BehaviorSystem) {
    let mut wanted: BTreeMap<(ComponentInstanceId, LocalEntity), ()> = BTreeMap::new();
    for inst in s.instances.values() {
        if let Some(c) = s.components.get(&inst.component) {
            for e in private_entities(c) {
                wanted.insert((inst.id, e), ());
            }
        }
    }
    s.flat_ids
        .entries
        .retain(|e| wanted.contains_key(&(e.instance, e.local)));
    for (instance, local) in wanted.keys() {
        if s.flat_ids.get(*instance, *local).is_some() {
            continue;
        }
        let flat = match local {
            LocalEntity::Decl(_) => {
                let (id, ids) = s.base.ids.fresh_decl();
                s.base.ids = ids;
                id.raw()
            }
            LocalEntity::Sem(_) => {
                let (id, ids) = s.base.ids.fresh_semantic();
                s.base.ids = ids;
                id.raw()
            }
            LocalEntity::Clock(_) => {
                let (id, ids) = s.base.ids.fresh_clock();
                s.base.ids = ids;
                id.raw()
            }
            LocalEntity::Output(_) => {
                let (id, ids) = s.base.ids.fresh_output();
                s.base.ids = ids;
                id.raw()
            }
            LocalEntity::Device(_) => {
                let (id, ids) = s.base.ids.fresh_device();
                s.base.ids = ids;
                id.raw()
            }
        };
        s.flat_ids.entries.push(FlatIdEntry {
            instance: *instance,
            local: *local,
            flat,
        });
    }
    s.flat_ids
        .entries
        .sort_by(|a, b| (a.instance, a.local).cmp(&(b.instance, b.local)));
}

fn flat_decl(s: &BehaviorSystem, r: PortRef) -> Option<DeclId> {
    let port = s.port(r)?;
    s.flat_ids
        .get(r.instance, LocalEntity::Decl(port.decl))
        .map(DeclId::from_raw)
}

/// Apply one system edit.  Errors leave the input untouched.
pub fn apply_system_edit(
    snapshot: &SystemSnapshot,
    op: &SystemEditOp,
) -> Result<AppliedSystem, SystemEditError> {
    let mut s = snapshot.system.clone();
    let mut outcome = match op {
        SystemEditOp::Base { op } => {
            let base = ProjectSnapshot::new(s.base.clone());
            let applied = apply_edit(&base, op).map_err(SystemEditError::Base)?;
            s.base = applied.snapshot.design;
            let inner = applied.outcome;
            SystemEditOutcome {
                kind: inner.kind,
                invalidates: inner.invalidates.clone(),
                origin_decls: inner.origin_decls.clone(),
                inner: Some(inner),
                ..Default::default()
            }
        }
        SystemEditOp::CreateComponent { name, description } => {
            let name = valid_name(name)?;
            if s.components.values().any(|c| c.name == name) {
                return Err(SystemEditError::DuplicateComponentName { name });
            }
            let id = s.ids.fresh_component();
            s.components.insert(
                id,
                BehaviorComponent {
                    id,
                    name: name.clone(),
                    description: description.clone(),
                    body: Design::empty(name),
                    interface: BehaviorInterface::default(),
                    shared_concepts: BTreeMap::new(),
                    external_outputs: BTreeMap::new(),
                    stamp: 0,
                },
            );
            let mut o = SystemEditOutcome::refinement();
            o.created_component = Some(id);
            o
        }
        SystemEditOp::InstallComponent { component } => {
            let name = valid_name(&component.name)?;
            if s.components.values().any(|c| c.name == name) {
                return Err(SystemEditError::DuplicateComponentName { name });
            }
            let id = s.ids.fresh_component();
            let mut c = (**component).clone();
            c.id = id;
            c.name = name;
            let mut ports = BTreeMap::new();
            for p in c.interface.ports.values() {
                let pid = s.ids.fresh_port();
                ports.insert(
                    pid,
                    Port {
                        id: pid,
                        ..p.clone()
                    },
                );
            }
            c.interface.ports = ports;
            s.components.insert(id, c);
            let mut o = SystemEditOutcome::refinement();
            o.created_component = Some(id);
            o
        }
        SystemEditOp::RenameComponent { id, name } => {
            let name = valid_name(name)?;
            if s.components.values().any(|c| c.name == name && c.id != *id) {
                return Err(SystemEditError::DuplicateComponentName { name });
            }
            component_mut(&mut s, *id)?.name = name;
            SystemEditOutcome::refinement()
        }
        SystemEditOp::SetComponentDescription { id, description } => {
            component_mut(&mut s, *id)?.description = description.clone();
            SystemEditOutcome::refinement()
        }
        SystemEditOp::DeleteComponent { id } => {
            component_mut(&mut s, *id)?;
            let instances: Vec<_> = s.instances_of(*id).map(|i| i.id).collect();
            if !instances.is_empty() {
                return Err(SystemEditError::ComponentInUse { id: *id, instances });
            }
            s.components.remove(id);
            SystemEditOutcome::refinement()
        }
        SystemEditOp::EditComponentBody { component, op } => {
            let c = component_mut(&mut s, *component)?;
            // A port's declaration cannot be removed from under it.
            if let EditOp::DeleteMapping { id } = op {
                if let Some(p) = c.interface.port_for_decl(*id) {
                    return Err(SystemEditError::PortBacked { port: p.id });
                }
            }
            if let EditOp::DeleteClockDomain { id } = op {
                if c.interface.is_clock_param(*id) {
                    let instances: Vec<_> = s.instances_of(*component).map(|i| i.id).collect();
                    return Err(SystemEditError::ClockParamInUse {
                        clock: *id,
                        instances,
                    });
                }
            }
            let c = component_mut(&mut s, *component)?;
            let body = ProjectSnapshot::new(c.body.clone());
            let applied = apply_edit(&body, op).map_err(|error| SystemEditError::Body {
                component: *component,
                error,
            })?;
            c.body = applied.snapshot.design;
            c.stamp += 1;
            // Tables that named a removed entity are pruned.
            let concepts: BTreeSet<_> = c.body.concepts.keys().copied().collect();
            c.shared_concepts.retain(|l, _| concepts.contains(l));
            let outputs: BTreeSet<_> = c.body.outputs.keys().copied().collect();
            c.external_outputs.retain(|l, _| outputs.contains(l));
            let inner = applied.outcome;
            let instances: BTreeSet<_> = s.instances_of(*component).map(|i| i.id).collect();
            let mut o = SystemEditOutcome {
                kind: inner.kind,
                invalidates: inner.invalidates.clone(),
                instances,
                inner: Some(inner.clone()),
                ..Default::default()
            };
            // Every instance of the component carries the edit: the
            // flattened counterparts of the body's origin declarations.
            for inst in s.instances_of(*component) {
                for d in &inner.origin_decls {
                    if let Some(f) = s.flat_ids.get(inst.id, LocalEntity::Decl(*d)) {
                        o.origin_decls.insert(DeclId::from_raw(f));
                    }
                }
            }
            o
        }
        SystemEditOp::DeclarePort {
            component,
            decl,
            kind,
            name,
            description,
        } => {
            let name = valid_name(name)?;
            let c = component_mut(&mut s, *component)?;
            if !c.body.mappings.contains_key(decl) {
                return Err(SystemEditError::NotABodyDeclaration { decl: *decl });
            }
            if let Some(p) = c.interface.port_for_decl(*decl) {
                return Err(SystemEditError::DeclAlreadyExposed {
                    decl: *decl,
                    port: p.id,
                });
            }
            if c.interface.ports.values().any(|p| p.name == name) {
                return Err(SystemEditError::DuplicatePortName { name });
            }
            let id = s.ids.fresh_port();
            let c = component_mut(&mut s, *component)?;
            c.interface.ports.insert(
                id,
                Port {
                    id,
                    name,
                    description: description.clone(),
                    kind: *kind,
                    decl: *decl,
                },
            );
            c.stamp += 1;
            let mut o = SystemEditOutcome::refinement();
            o.created_port = Some(id);
            o
        }
        SystemEditOp::RenamePort {
            component,
            port,
            name,
        } => {
            let name = valid_name(name)?;
            let c = component_mut(&mut s, *component)?;
            if c.interface
                .ports
                .values()
                .any(|p| p.name == name && p.id != *port)
            {
                return Err(SystemEditError::DuplicatePortName { name });
            }
            c.interface
                .ports
                .get_mut(port)
                .ok_or(SystemEditError::UnknownPort {
                    component: *component,
                    port: *port,
                })?
                .name = name;
            SystemEditOutcome::refinement()
        }
        SystemEditOp::RetirePort { component, port } => {
            let c = component_mut(&mut s, *component)?;
            if !c.interface.ports.contains_key(port) {
                return Err(SystemEditError::UnknownPort {
                    component: *component,
                    port: *port,
                });
            }
            let in_use = s.bindings.values().any(|b| {
                (b.source.port == *port || b.destination.port == *port)
                    && s.instances
                        .get(&b.source.instance)
                        .map(|i| i.component == *component)
                        .unwrap_or(false)
            }) || s.exports.values().any(|e| {
                e.port.port == *port
                    && s.instances
                        .get(&e.port.instance)
                        .map(|i| i.component == *component)
                        .unwrap_or(false)
            }) || s
                .instances_of(*component)
                .any(|i| i.parameter_bindings.contains_key(port));
            if in_use {
                return Err(SystemEditError::PortInUse { port: *port });
            }
            let c = component_mut(&mut s, *component)?;
            c.interface.ports.remove(port);
            c.stamp += 1;
            SystemEditOutcome::refinement()
        }
        SystemEditOp::SetClockParameter {
            component,
            clock,
            parameter,
        } => {
            let c = component_mut(&mut s, *component)?;
            if !c.body.clocks.contains_key(clock) {
                return Err(SystemEditError::NotABodyClock { clock: *clock });
            }
            let was = c.interface.is_clock_param(*clock);
            if was && !*parameter {
                let instances: Vec<_> = s
                    .instances_of(*component)
                    .filter(|i| i.clock_bindings.contains_key(clock))
                    .map(|i| i.id)
                    .collect();
                if !instances.is_empty() {
                    return Err(SystemEditError::ClockParamInUse {
                        clock: *clock,
                        instances,
                    });
                }
            }
            let c = component_mut(&mut s, *component)?;
            if *parameter && !was {
                c.interface.clock_params.push(*clock);
            } else if !*parameter && was {
                c.interface.clock_params.retain(|k| k != clock);
            }
            c.stamp += 1;
            let instances: BTreeSet<_> = s.instances_of(*component).map(|i| i.id).collect();
            let mut o = SystemEditOutcome::edit([Invalidation::Clock]);
            o.instances = instances;
            o
        }
        SystemEditOp::ShareConcept {
            component,
            local,
            system,
        } => {
            if let Some(sys) = system {
                if !s.base.concepts.contains_key(sys) {
                    return Err(SystemEditError::UnknownSystemConcept { id: *sys });
                }
            }
            let c = component_mut(&mut s, *component)?;
            if !c.body.concepts.contains_key(local) {
                return Err(SystemEditError::NotABodyConcept { id: *local });
            }
            match system {
                Some(sys) => {
                    c.shared_concepts.insert(*local, *sys);
                }
                None => {
                    c.shared_concepts.remove(local);
                }
            }
            c.stamp += 1;
            let instances: BTreeSet<_> = s.instances_of(*component).map(|i| i.id).collect();
            let mut o =
                SystemEditOutcome::edit([Invalidation::Semantic, Invalidation::Realization]);
            o.instances = instances;
            o
        }
        SystemEditOp::ExternalizeOutput {
            component,
            local,
            system,
        } => {
            if let Some(sys) = system {
                if !s.base.outputs.contains_key(sys) {
                    return Err(SystemEditError::UnknownSystemOutput { id: *sys });
                }
            }
            let c = component_mut(&mut s, *component)?;
            if !c.body.outputs.contains_key(local) {
                return Err(SystemEditError::NotABodyOutput { id: *local });
            }
            match system {
                Some(sys) => {
                    c.external_outputs.insert(*local, *sys);
                }
                None => {
                    c.external_outputs.remove(local);
                }
            }
            c.stamp += 1;
            let instances: BTreeSet<_> = s.instances_of(*component).map(|i| i.id).collect();
            let mut o = SystemEditOutcome::edit([Invalidation::Output, Invalidation::Deployment]);
            o.instances = instances;
            o
        }
        SystemEditOp::CreateInstance { component, name } => {
            let name = valid_name(name)?;
            if !s.components.contains_key(component) {
                return Err(SystemEditError::UnknownComponent { id: *component });
            }
            if s.instances.values().any(|i| i.name == name) {
                return Err(SystemEditError::DuplicateInstanceName { name });
            }
            let id = s.ids.fresh_instance();
            s.instances.insert(
                id,
                ComponentInstance {
                    id,
                    component: *component,
                    name,
                    clock_bindings: BTreeMap::new(),
                    parameter_bindings: BTreeMap::new(),
                },
            );
            let mut o = SystemEditOutcome::refinement();
            o.created_instance = Some(id);
            o.instances.insert(id);
            o
        }
        SystemEditOp::RenameInstance { id, name } => {
            let name = valid_name(name)?;
            if s.instances.values().any(|i| i.name == name && i.id != *id) {
                return Err(SystemEditError::DuplicateInstanceName { name });
            }
            instance_mut(&mut s, *id)?.name = name;
            SystemEditOutcome::refinement()
        }
        SystemEditOp::DeleteInstance { id } => {
            instance_mut(&mut s, *id)?;
            let bindings: Vec<_> = s
                .bindings
                .values()
                .filter(|b| b.source.instance == *id || b.destination.instance == *id)
                .map(|b| b.id)
                .collect();
            let exports: Vec<_> = s
                .exports
                .values()
                .filter(|e| e.port.instance == *id)
                .map(|e| e.id)
                .collect();
            if !bindings.is_empty() || !exports.is_empty() {
                return Err(SystemEditError::InstanceInUse {
                    id: *id,
                    bindings,
                    exports,
                });
            }
            let decls: BTreeSet<DeclId> = s
                .flat_ids
                .for_instance(*id)
                .filter_map(|e| match e.local {
                    LocalEntity::Decl(_) => Some(DeclId::from_raw(e.flat)),
                    _ => None,
                })
                .collect();
            s.instances.remove(id);
            let mut o = SystemEditOutcome::edit([
                Invalidation::Realization,
                Invalidation::Reactive,
                Invalidation::Clock,
                Invalidation::Output,
                Invalidation::Deployment,
            ]);
            o.origin_decls = decls;
            o.instances.insert(*id);
            o
        }
        SystemEditOp::SetClockArgument {
            instance,
            parameter,
            clock,
        } => {
            if let Some(c) = clock {
                if !s.base.clocks.contains_key(c) {
                    return Err(SystemEditError::UnknownSystemClock { id: *c });
                }
            }
            let comp = s
                .component_of(*instance)
                .ok_or(SystemEditError::UnknownInstance { id: *instance })?;
            if !comp.interface.is_clock_param(*parameter) {
                return Err(SystemEditError::NotAClockParameter { clock: *parameter });
            }
            let inst = instance_mut(&mut s, *instance)?;
            match clock {
                Some(c) => {
                    inst.clock_bindings.insert(*parameter, *c);
                }
                None => {
                    inst.clock_bindings.remove(parameter);
                }
            }
            let mut o = SystemEditOutcome::edit([
                Invalidation::Clock,
                Invalidation::Reactive,
                Invalidation::Output,
            ]);
            o.instances.insert(*instance);
            o
        }
        SystemEditOp::SetParameterArgument {
            instance,
            port,
            value,
        } => {
            let comp = s
                .component_of(*instance)
                .ok_or(SystemEditError::UnknownInstance { id: *instance })?;
            let p = comp
                .interface
                .ports
                .get(port)
                .ok_or(SystemEditError::UnknownPort {
                    component: comp.id,
                    port: *port,
                })?;
            if p.kind != PortKind::Parameter {
                return Err(SystemEditError::NotAParameter { port: *port });
            }
            let r = PortRef {
                instance: *instance,
                port: *port,
            };
            if let Some(b) = s.binding_into(r) {
                return Err(SystemEditError::DestinationBound { binding: b.id });
            }
            let inst = instance_mut(&mut s, *instance)?;
            match value {
                Some(v) => {
                    inst.parameter_bindings.insert(*port, v.clone());
                }
                None => {
                    inst.parameter_bindings.remove(port);
                }
            }
            let mut o =
                SystemEditOutcome::edit([Invalidation::Realization, Invalidation::Reactive]);
            o.instances.insert(*instance);
            o.origin_decls.extend(flat_decl(&s, r));
            o
        }
        SystemEditOp::BindPorts {
            source,
            destination,
            transport,
        } => {
            let sp = port_of(&s, *source)?;
            let dp = port_of(&s, *destination)?;
            if sp.kind != PortKind::Provided {
                return Err(SystemEditError::SourceNotProvided { port: *source });
            }
            if !matches!(dp.kind, PortKind::Required | PortKind::Parameter) {
                return Err(SystemEditError::DestinationNotRequired { port: *destination });
            }
            if let Some(b) = s.binding_into(*destination) {
                return Err(SystemEditError::DestinationBound { binding: b.id });
            }
            if let Some(e) = s.export_of(*destination) {
                return Err(SystemEditError::PortExported { export: e.id });
            }
            if s.instances
                .get(&destination.instance)
                .is_some_and(|i| i.parameter_bindings.contains_key(&destination.port))
            {
                return Err(SystemEditError::PortHasValue);
            }
            let id = s.ids.fresh_binding();
            s.bindings.insert(
                id,
                Binding {
                    id,
                    source: *source,
                    destination: *destination,
                    transport: transport.clone(),
                },
            );
            // Binding is a realization step: a refinement of the destination.
            let mut o = SystemEditOutcome::refinement()
                .touching(Invalidation::Realization)
                .touching(Invalidation::Reactive)
                .touching(Invalidation::Clock)
                .touching(Invalidation::Output);
            o.created_binding = Some(id);
            o.instances.insert(source.instance);
            o.instances.insert(destination.instance);
            o.origin_decls.extend(flat_decl(&s, *destination));
            o
        }
        SystemEditOp::UnbindPorts { binding } => {
            let b = s
                .bindings
                .remove(binding)
                .ok_or(SystemEditError::UnknownBinding { id: *binding })?;
            let mut o = SystemEditOutcome::edit([
                Invalidation::Realization,
                Invalidation::Reactive,
                Invalidation::Clock,
                Invalidation::Output,
            ]);
            o.instances.insert(b.source.instance);
            o.instances.insert(b.destination.instance);
            o.origin_decls.extend(flat_decl(&s, b.destination));
            o
        }
        SystemEditOp::ExportPort { port, name } => {
            let name = valid_name(name)?;
            let p = port_of(&s, *port)?;
            if p.kind != PortKind::Required {
                return Err(SystemEditError::ExportNotRequired { port: *port });
            }
            if let Some(b) = s.binding_into(*port) {
                return Err(SystemEditError::DestinationBound { binding: b.id });
            }
            if let Some(e) = s.export_of(*port) {
                return Err(SystemEditError::PortExported { export: e.id });
            }
            if s.exports.values().any(|e| e.name == name) {
                return Err(SystemEditError::DuplicateExportName { name });
            }
            let id = s.ids.fresh_export();
            s.exports.insert(
                id,
                Export {
                    id,
                    port: *port,
                    name,
                },
            );
            let mut o = SystemEditOutcome::refinement();
            o.created_export = Some(id);
            o.instances.insert(port.instance);
            o
        }
        SystemEditOp::HidePort { export } => {
            let e = s
                .exports
                .remove(export)
                .ok_or(SystemEditError::UnknownExport { id: *export })?;
            let mut o = SystemEditOutcome::refinement();
            o.instances.insert(e.port.instance);
            o
        }
    };
    ensure_flat_ids(&mut s);
    // A body edit reaches every instance's flattened declarations, which
    // may only exist after the table was completed (a new body mapping).
    if let SystemEditOp::EditComponentBody { component, .. } = op {
        if outcome.origin_decls.is_empty() {
            if let Some(inner) = &outcome.inner {
                for inst in s.instances_of(*component) {
                    for d in &inner.origin_decls {
                        if let Some(f) = s.flat_ids.get(inst.id, LocalEntity::Decl(*d)) {
                            outcome.origin_decls.insert(DeclId::from_raw(f));
                        }
                    }
                }
            }
        }
    }
    Ok(AppliedSystem {
        snapshot: SystemSnapshot {
            revision: snapshot.revision.next(),
            system: s,
        },
        outcome,
    })
}

/// Convenience for tests and tooling: apply a sequence, stopping at the
/// first error.
pub fn apply_all(
    snapshot: &SystemSnapshot,
    ops: &[SystemEditOp],
) -> Result<SystemSnapshot, SystemEditError> {
    let mut s = snapshot.clone();
    for op in ops {
        s = apply_system_edit(&s, op)?.snapshot;
    }
    Ok(s)
}
