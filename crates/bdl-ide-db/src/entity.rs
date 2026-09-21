//! Stable semantic entity references and the roles a location can play.
//!
//! An [`EntityRef`] is the IDE's name for a thing in the design.  It is
//! built from the model's stable ids and never from a display name or a
//! text position: renaming keeps the reference, moving text keeps the
//! reference, and two surfaces that show the same mapping hold the same
//! `EntityRef`.
//!
//! An [`EntityRole`] says *which aspect* of an entity a diagnostic, an
//! action or a projection anchor is about — its name, its definition, its
//! drive edge — so that a text editor can underline the right span and the
//! canvas can highlight the right port without either re-deriving meaning.

use bdl_diagnostics::Entity;
use bdl_model::{ClockId, ConceptId, DeclId, DeviceId, OutputId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A reference to a semantic entity by stable identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum EntityRef {
    /// The whole design (project-level notes such as a missing driver).
    Project,
    Concept(ConceptId),
    Mapping(DeclId),
    Clock(ClockId),
    Output(OutputId),
    Device(DeviceId),
    /// One requirement of a device binding (`RequirementId` in
    /// `bdl-hardware`: the device and its local index).
    Requirement {
        device: DeviceId,
        index: u16,
    },
    /// A behaviour component (a text or system project); raw `ComponentId`.
    Component(u64),
    /// A component's port; raw ids.
    Port {
        component: u64,
        port: u64,
    },
    /// A component instance; raw `ComponentInstanceId`.
    Instance(u64),
    /// A binding between ports; raw `BindingId`.
    Binding(u64),
    /// An exported port; raw `ExportId`.
    Export(u64),
}

impl EntityRef {
    /// The entity a compiler diagnostic names.
    pub fn from_diagnostic(entity: Entity) -> EntityRef {
        match entity {
            Entity::Project => EntityRef::Project,
            Entity::Concept { id } => EntityRef::Concept(id),
            Entity::Mapping { id } => EntityRef::Mapping(id),
        }
    }

    pub fn as_mapping(self) -> Option<DeclId> {
        match self {
            EntityRef::Mapping(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_concept(self) -> Option<ConceptId> {
        match self {
            EntityRef::Concept(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_output(self) -> Option<OutputId> {
        match self {
            EntityRef::Output(o) => Some(o),
            _ => None,
        }
    }

    /// The sort of entity, for symbol kinds and token types.
    pub fn kind(self) -> EntityKind {
        match self {
            EntityRef::Project => EntityKind::Project,
            EntityRef::Concept(_) => EntityKind::Concept,
            EntityRef::Mapping(_) => EntityKind::Mapping,
            EntityRef::Clock(_) => EntityKind::Clock,
            EntityRef::Output(_) => EntityKind::Output,
            EntityRef::Device(_) => EntityKind::Device,
            EntityRef::Requirement { .. } => EntityKind::Requirement,
            EntityRef::Component(_) => EntityKind::Component,
            EntityRef::Port { .. } => EntityKind::Port,
            EntityRef::Instance(_) => EntityKind::Instance,
            EntityRef::Binding(_) => EntityKind::Binding,
            EntityRef::Export(_) => EntityKind::Export,
        }
    }
}

impl fmt::Display for EntityRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityRef::Project => write!(f, "project"),
            EntityRef::Concept(s) => write!(f, "{s}"),
            EntityRef::Mapping(d) => write!(f, "{d}"),
            EntityRef::Clock(c) => write!(f, "{c}"),
            EntityRef::Output(o) => write!(f, "{o}"),
            EntityRef::Device(d) => write!(f, "{d}"),
            EntityRef::Requirement { device, index } => write!(f, "{device}/{index}"),
            EntityRef::Component(c) => write!(f, "component#{c}"),
            EntityRef::Port { component, port } => write!(f, "component#{component}/port#{port}"),
            EntityRef::Instance(i) => write!(f, "instance#{i}"),
            EntityRef::Binding(b) => write!(f, "binding#{b}"),
            EntityRef::Export(e) => write!(f, "export#{e}"),
        }
    }
}

/// The sort of an entity, without its identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Project,
    Concept,
    Mapping,
    Clock,
    Output,
    Device,
    Requirement,
    Component,
    Port,
    Instance,
    Binding,
    Export,
}

/// Which aspect of an entity a location, diagnostic or action concerns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "snake_case")]
pub enum EntityRole {
    /// The entity as a whole (its declaration).
    Declaration,
    /// The display name at its declaration site.
    Name,
    /// A use of the entity's name somewhere else (a reference site).
    Reference,
    /// The `i`-th input of a mapping's signature.
    Input { index: u16 },
    /// The output concept of a mapping's signature, or the concept an
    /// output accepts.
    Output,
    /// The whole signature `(A, B) -> C` as authored.
    Signature,
    /// A concept's representation binding.
    Representation,
    /// A mapping's definition (formula body).
    Definition,
    /// The drive edge `β d = o`.
    DriveEdge,
    /// The clock-domain assignment of a mapping or output.
    ClockBinding,
    /// The device that realises an output.
    DeviceBinding,
    /// A hardware requirement of a device.
    Requirement,
    /// The free-text description.
    Description,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_refs_round_trip_through_json() {
        for e in [
            EntityRef::Project,
            EntityRef::Concept(ConceptId::from_raw(2)),
            EntityRef::Mapping(DeclId::from_raw(17)),
            EntityRef::Requirement {
                device: DeviceId::from_raw(1),
                index: 3,
            },
        ] {
            let json = serde_json::to_string(&e).expect("serializes");
            let back: EntityRef = serde_json::from_str(&json).expect("deserializes");
            assert_eq!(back, e, "{json}");
        }
        assert_eq!(
            serde_json::to_string(&EntityRef::Mapping(DeclId::from_raw(17))).expect("json"),
            r#"{"kind":"mapping","id":17}"#
        );
    }
}
