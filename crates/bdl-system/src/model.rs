//! The authored system model (docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md §1–§4).
//!
//! A [`BehaviorSystem`] is a flat `Design` (`base`) plus components,
//! instances, bindings and exports.  A flat project is the degenerate
//! system: `base` alone.  Everything here is a surface object that
//! `flatten` turns into an ordinary flat design; nothing is a kernel term.

use crate::ids::{
    BehaviorGroupId, BindingId, ComponentId, ComponentInstanceId, ExportId, PortId,
    SystemIdAllocator,
};
use bdl_ir::PropertyId;
use bdl_model::surface::{Design, MappingBlock, Signature};
use bdl_model::{ClockId, DeclId, DeviceId, OutputId, Revision, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bumped on any change to the persisted representation.  Schema 1 had
/// ports without explicit contracts (derived from the body); `persist`
/// migrates it forward once.
pub const SYSTEM_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortKind {
    /// An unresolved declaration of the body, to be bound by a composer
    /// (FV `required`).  Unbound, it stays *open* — never an error.
    Required,
    /// A declaration of the body offered to others (FV `provided`).
    Provided,
    /// An unresolved nullary declaration bound to a closed constant at
    /// instantiation (FV `params`).
    Parameter,
}

/// The timing side of a port contract, in component-local terms (FV
/// `Port.clock`).  A system `ClockId` never appears in a contract: a
/// parameter is mapped by each instance, a private clock is freshened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClockContract {
    /// The port serves any domain (the kernel's `Κ d = none`).
    Agnostic,
    /// The port updates in a clock parameter of the interface: the
    /// instance says which system domain that is.
    Parameter { clock: ClockId },
    /// The port updates in a domain private to the component.
    Private { clock: ClockId },
}

impl ClockContract {
    pub fn local(self) -> Option<ClockId> {
        match self {
            ClockContract::Agnostic => None,
            ClockContract::Parameter { clock } | ClockContract::Private { clock } => Some(clock),
        }
    }
}

/// What a port promises (FV `Port.iface` + `Port.clock`): the shape of
/// the declaration behind it — over component-local concepts, shared ones
/// standing for system concepts through the component's table — its
/// commitments, and its timing.  Stored explicitly; a body edit never
/// changes it (docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md §11).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortContract {
    pub signature: Signature,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commitments: Vec<PropertyId>,
    pub clock: ClockContract,
}

impl PortContract {
    /// The contract a body declaration has *today* — what `DeclarePort`
    /// snapshots once, and what `Realizes` compares against thereafter.
    pub fn of_declaration(m: &MappingBlock, interface: &BehaviorInterface) -> PortContract {
        PortContract {
            signature: m.signature.clone(),
            commitments: Vec::new(),
            clock: match m.clock {
                None => ClockContract::Agnostic,
                Some(c) if interface.is_clock_param(c) => ClockContract::Parameter { clock: c },
                Some(c) => ClockContract::Private { clock: c },
            },
        }
    }

    /// FV `IfaceRefines` for one port: same signature and clock; for a
    /// provided port the new commitments include the old, for a required
    /// port or parameter the old include the new.
    pub fn is_refined_by(&self, new: &PortContract, kind: PortKind) -> bool {
        self.signature == new.signature
            && self.clock == new.clock
            && match kind {
                PortKind::Provided => self.commitments.iter().all(|c| new.commitments.contains(c)),
                PortKind::Required | PortKind::Parameter => {
                    new.commitments.iter().all(|c| self.commitments.contains(c))
                }
            }
    }
}

/// A port: a public promise with its own stable id and display name, and
/// the body declaration intended to realize it.  What a composer sees of
/// the component; the body is not part of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub kind: PortKind,
    /// The body declaration meant to realize the contract (component-local
    /// `DeclId`).  A link, not the contract: `Realizes` checks the two agree.
    pub decl: DeclId,
    pub contract: PortContract,
}

/// The public boundary of a component (FV `BehaviorInterface`).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BehaviorInterface {
    /// Every port, by id; kinds partition them.
    pub ports: BTreeMap<PortId, Port>,
    /// Clock domains of the body that the instantiation maps to system
    /// domains (FV `clockParams`).  Every other body clock is private.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clock_params: Vec<ClockId>,
}

impl BehaviorInterface {
    pub fn ports_of(&self, kind: PortKind) -> impl Iterator<Item = &Port> {
        self.ports.values().filter(move |p| p.kind == kind)
    }
    pub fn port_for_decl(&self, decl: DeclId) -> Option<&Port> {
        self.ports.values().find(|p| p.decl == decl)
    }
    pub fn is_clock_param(&self, c: ClockId) -> bool {
        self.clock_params.contains(&c)
    }
}

/// A reusable behaviour (FV `BehaviorComponent`): an interface over an
/// ordinary flat design authored with the ordinary edit ops.  Project-local
/// in this milestone (docs/BEHAVIOR_SYSTEMS.md, limitations).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorComponent {
    pub id: ComponentId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// The body, over the component's own local identities and allocator.
    pub body: Design,
    pub interface: BehaviorInterface,
    /// Body concepts that stand for a system concept (FV: not `internalSem`).
    /// Every other body concept is private and freshened per instance.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub shared_concepts: BTreeMap<SemanticId, SemanticId>,
    /// Body sinks that stand for a system sink (FV: not `internalOut`).
    /// Every other body sink is private and freshened per instance.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub external_outputs: BTreeMap<OutputId, OutputId>,
    /// Bumped by every edit of the body (implementation); never by an
    /// interface edit.  A cache/invalidation stamp, never an identity.
    #[serde(default)]
    pub body_stamp: u64,
    /// Bumped by every edit of the public interface (ports, contracts,
    /// clock parameters, sharing tables).
    #[serde(default)]
    pub interface_stamp: u64,
}

impl BehaviorComponent {
    pub fn is_private_concept(&self, s: SemanticId) -> bool {
        !self.shared_concepts.contains_key(&s)
    }
    pub fn is_private_output(&self, o: OutputId) -> bool {
        !self.external_outputs.contains_key(&o)
    }
    pub fn is_private_clock(&self, c: ClockId) -> bool {
        !self.interface.is_clock_param(c)
    }
}

/// A closed constant for a parameter, as the designer writes it (`0.5`,
/// `30 deg`).  Elaborated as a nullary formula of the port's own
/// signature with no relationship in scope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParameterValue {
    pub source: String,
}

/// One occurrence of a component in a system (FV `Inst`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub id: ComponentInstanceId,
    pub component: ComponentId,
    pub name: String,
    /// Component clock parameter → system clock domain (FV `κ`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub clock_bindings: BTreeMap<ClockId, ClockId>,
    /// Parameter port → closed value.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub parameter_bindings: BTreeMap<PortId, ParameterValue>,
}

/// A port of an instance, by identity.  Display names never enter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PortRef {
    pub instance: ComponentInstanceId,
    pub port: PortId,
}

/// How a binding crosses timing domains: `None` is a direct reference
/// (same domain, or an agnostic source); `Some` transports through the
/// kernel's `sync` from the source's domain, starting from `init`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingTransport {
    /// A closed formula in the destination's units.
    pub init: String,
}

/// One end of a binding: a port of an instance, or a relationship of the
/// system's own design (FV Extract: the residual is "instance 0" — here it
/// is the base, so its declarations are bindable directly).  Untagged, so a
/// port end persists as before (`{instance, port}`) and a base end as
/// `{decl}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BindingEnd {
    Port(PortRef),
    Base { decl: DeclId },
}

impl BindingEnd {
    pub const fn port(instance: ComponentInstanceId, port: PortId) -> BindingEnd {
        BindingEnd::Port(PortRef { instance, port })
    }
    pub fn as_port(self) -> Option<PortRef> {
        match self {
            BindingEnd::Port(r) => Some(r),
            BindingEnd::Base { .. } => None,
        }
    }
    pub fn instance(self) -> Option<ComponentInstanceId> {
        self.as_port().map(|r| r.instance)
    }
    pub fn base_decl(self) -> Option<DeclId> {
        match self {
            BindingEnd::Base { decl } => Some(decl),
            BindingEnd::Port(_) => None,
        }
    }
}

impl From<PortRef> for BindingEnd {
    fn from(r: PortRef) -> BindingEnd {
        BindingEnd::Port(r)
    }
}

/// A required port of one instance taken from a provided port of another
/// (or of itself) — FV `Binding` with a port source — or, since Phase 8b,
/// an open relationship of the base taken from an instance's provided
/// port, or an instance's required port taken from a base relationship.
/// Elaborates to one realization step of the destination (D-67).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub id: BindingId,
    pub source: BindingEnd,
    pub destination: BindingEnd,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<BindingTransport>,
}

/// A required port declared as an input of the whole system: it stays
/// unresolved, and the system is executable with it as an input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Export {
    pub id: ExportId,
    pub port: PortRef,
    pub name: String,
}

/// A component-local entity, by sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "sort", content = "id", rename_all = "snake_case")]
pub enum LocalEntity {
    Decl(DeclId),
    Sem(SemanticId),
    Clock(ClockId),
    Output(OutputId),
    Device(DeviceId),
}

/// The freshening table (docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md §5): for every
/// private entity of every instance, the flat identity it was issued from
/// the base allocator.  Extended by the edit model, read by `flatten`.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FlatIds {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<FlatIdEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatIdEntry {
    pub instance: ComponentInstanceId,
    pub local: LocalEntity,
    /// The flat id's raw value; its sort is `local`'s sort.
    pub flat: u64,
}

impl FlatIds {
    pub fn get(&self, instance: ComponentInstanceId, local: LocalEntity) -> Option<u64> {
        self.entries
            .iter()
            .find(|e| e.instance == instance && e.local == local)
            .map(|e| e.flat)
    }
    pub fn for_instance(
        &self,
        instance: ComponentInstanceId,
    ) -> impl Iterator<Item = &FlatIdEntry> {
        self.entries.iter().filter(move |e| e.instance == instance)
    }
}

/// An authoring group over the base design's relationships (FV Phase 8b
/// `BehaviorGroup`): an identity, a name, a description and a member list
/// — and nothing semantic.  Types, formulas, clocks and drives stay on the
/// members; every kernel judgment of the system is literally the judgment
/// of the ungrouped design (`eraseGroups`).  Collapse state, position and
/// size are layout, kept out of here on purpose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorGroup {
    pub id: BehaviorGroupId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Base relationships, in authoring order, each in at most one group.
    #[serde(default)]
    pub members: Vec<DeclId>,
}

/// The authored truth of a system project.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviorSystem {
    /// The system-level flat design: shared concepts, system clock domains,
    /// external sinks and devices, top-level relationships.  Its allocator
    /// issues every flat id, freshened ones included.
    pub base: Design,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub components: BTreeMap<ComponentId, BehaviorComponent>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub instances: BTreeMap<ComponentInstanceId, ComponentInstance>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub bindings: BTreeMap<BindingId, Binding>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub exports: BTreeMap<ExportId, Export>,
    /// Authoring groups over `base`'s relationships.  Never read by
    /// `flatten`, `validate` or any analysis.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<BehaviorGroupId, BehaviorGroup>,
    #[serde(default)]
    pub flat_ids: FlatIds,
    #[serde(default)]
    pub ids: SystemIdAllocator,
}

impl BehaviorSystem {
    /// The degenerate system: a flat design and nothing else.  Its
    /// flattening is that design.
    pub fn from_flat(design: Design) -> BehaviorSystem {
        BehaviorSystem {
            base: design,
            components: BTreeMap::new(),
            instances: BTreeMap::new(),
            bindings: BTreeMap::new(),
            exports: BTreeMap::new(),
            groups: BTreeMap::new(),
            flat_ids: FlatIds::default(),
            ids: SystemIdAllocator::default(),
        }
    }

    pub fn empty(name: impl Into<String>) -> BehaviorSystem {
        BehaviorSystem::from_flat(Design::empty(name))
    }

    /// No components, instances or bindings: an ordinary flat design.
    pub fn is_flat(&self) -> bool {
        self.components.is_empty() && self.instances.is_empty() && self.bindings.is_empty()
    }

    pub fn component_of(&self, instance: ComponentInstanceId) -> Option<&BehaviorComponent> {
        self.instances
            .get(&instance)
            .and_then(|i| self.components.get(&i.component))
    }

    /// The port a reference names, when both the instance and the port exist.
    pub fn port(&self, r: PortRef) -> Option<&Port> {
        self.component_of(r.instance)
            .and_then(|c| c.interface.ports.get(&r.port))
    }

    /// The binding whose destination is `r`, if any (at most one: D-67).
    pub fn binding_into(&self, r: impl Into<BindingEnd>) -> Option<&Binding> {
        let r = r.into();
        self.bindings.values().find(|b| b.destination == r)
    }

    /// The bindings touching an instance, at either end.
    pub fn bindings_of_instance(
        &self,
        instance: ComponentInstanceId,
    ) -> impl Iterator<Item = &Binding> {
        self.bindings.values().filter(move |b| {
            b.source.instance() == Some(instance) || b.destination.instance() == Some(instance)
        })
    }

    pub fn export_of(&self, r: PortRef) -> Option<&Export> {
        self.exports.values().find(|e| e.port == r)
    }

    /// The group a base relationship belongs to, if any (at most one).
    pub fn group_of(&self, decl: DeclId) -> Option<&BehaviorGroup> {
        self.groups.values().find(|g| g.members.contains(&decl))
    }

    pub fn instances_of(&self, component: ComponentId) -> impl Iterator<Item = &ComponentInstance> {
        self.instances
            .values()
            .filter(move |i| i.component == component)
    }
}

/// The revisioned system, the analogue of `ProjectSnapshot`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub revision: Revision,
    pub system: BehaviorSystem,
}

impl SystemSnapshot {
    pub fn new(system: BehaviorSystem) -> SystemSnapshot {
        SystemSnapshot {
            revision: Revision::default(),
            system,
        }
    }
}
