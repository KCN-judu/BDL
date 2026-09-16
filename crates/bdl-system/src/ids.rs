//! Identity sorts of the system layer.  None of them is a kernel identity:
//! a component, an instance, a port, a binding and an export are surface
//! objects (D-64), and they never overload `DeclId` / `SemanticId` /
//! `ClockId` / `OutputId`.  Sequential, per project, persisted, never
//! reused — the same contract as `bdl-model::IdAllocator`.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! system_id {
    ($(#[$m:meta])* $name:ident, $prefix:literal) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const fn from_raw(raw: u64) -> Self {
                Self(raw)
            }
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }
    };
}

system_id!(
    /// A reusable behaviour definition (FV `BehaviorComponent`).
    ComponentId,
    "comp#"
);
system_id!(
    /// One occurrence of a component in a system (FV `Inst`, by position;
    /// here a stable id so renames and reorderings change nothing).
    ComponentInstanceId,
    "inst#"
);
system_id!(
    /// A port of a component's public interface (FV `Port.id`, which is a
    /// template `DeclId`; here its own id so a port survives its
    /// declaration being re-created).
    PortId,
    "port#"
);
system_id!(
    /// A binding between two ports of two instances.
    BindingId,
    "bind#"
);
system_id!(
    /// A required port declared as an input of the whole system.
    ExportId,
    "export#"
);
system_id!(
    /// An authoring group of base relationships (FV Phase 8b `GroupId`):
    /// identity for a cognitive unit, never a kernel term.
    BehaviorGroupId,
    "group#"
);

/// Allocator of the system-layer sorts.  Kept beside the base design's
/// `IdAllocator` (which issues every flat sort, global and freshened alike).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemIdAllocator {
    next_component: u64,
    next_instance: u64,
    next_port: u64,
    next_binding: u64,
    next_export: u64,
    #[serde(default)]
    next_group: u64,
}

impl SystemIdAllocator {
    pub fn fresh_component(&mut self) -> ComponentId {
        let id = ComponentId(self.next_component);
        self.next_component += 1;
        id
    }
    pub fn fresh_instance(&mut self) -> ComponentInstanceId {
        let id = ComponentInstanceId(self.next_instance);
        self.next_instance += 1;
        id
    }
    pub fn fresh_port(&mut self) -> PortId {
        let id = PortId(self.next_port);
        self.next_port += 1;
        id
    }
    pub fn fresh_binding(&mut self) -> BindingId {
        let id = BindingId(self.next_binding);
        self.next_binding += 1;
        id
    }
    pub fn fresh_export(&mut self) -> ExportId {
        let id = ExportId(self.next_export);
        self.next_export += 1;
        id
    }
    pub fn fresh_group(&mut self) -> BehaviorGroupId {
        let id = BehaviorGroupId(self.next_group);
        self.next_group += 1;
        id
    }
}
