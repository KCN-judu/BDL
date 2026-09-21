//! Stable identities.
//!
//! Each identity is a distinct newtype so the type system keeps the kernel's
//! sorts apart: a concept is a *type*, a declaration is a *value*, a clock is
//! a *domain*, an output is a *resource*.  They are allocated sequentially
//! per project (see [`IdAllocator`]) and persisted with the project, so they
//! survive rename, layout changes, reopen, code generation, deployment and
//! telemetry.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! stable_id {
    ($(#[$meta:meta])* $name:ident, $prefix:literal) => {
        $(#[$meta])*
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

stable_id!(
    /// Identity of a concept (the kernel's `ConceptId`): the nominal type
    /// `sem C`, a *template*.  A Sem block is its instance (a unit-domain
    /// declaration of type `sem C`, one value per tick), a value `sem C v`
    /// the instance's state.  Several Sem blocks of one concept are
    /// ordinary.
    ConceptId,
    "sem#"
);
stable_id!(
    /// Identity of a design declaration (the kernel's `DeclId`).  A mapping
    /// block, an elaborated context activation, a transport — every
    /// declaration the design contains has one.
    DeclId,
    "decl#"
);
stable_id!(
    /// Identity of a clock domain (nominal; never a rate).
    ClockId,
    "clk#"
);
stable_id!(
    /// Identity of a physical sink (nominal; never a pin).
    OutputId,
    "out#"
);
stable_id!(
    /// Identity of a device binding — the deployment-layer object that says
    /// what kind of hardware realises a sink (or feeds a sensor) and
    /// generates its resource requirements.
    DeviceId,
    "dev#"
);

/// Monotone project revision.  Every edit produces the next revision; every
/// asynchronous analysis result carries the revision it was computed for so
/// stale results can be discarded.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Revision(u64);

impl Revision {
    pub const INITIAL: Revision = Revision(0);
    pub const fn raw(self) -> u64 {
        self.0
    }
    pub const fn from_raw(raw: u64) -> Self {
        Revision(raw)
    }
    #[must_use]
    pub const fn next(self) -> Revision {
        Revision(self.0 + 1)
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

/// Per-project allocator of fresh identities.  Persisted with the design so
/// that identities are never reused, even after deletion.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdAllocator {
    /// The project file spells this counter `next_semantic` (ADR-0043:
    /// the key predates the concept ladder; the bytes stay).
    #[serde(rename = "next_semantic")]
    next_concept: u64,
    next_decl: u64,
    next_clock: u64,
    next_output: u64,
    #[serde(default)]
    next_device: u64,
}

impl IdAllocator {
    #[must_use]
    pub fn fresh_concept(&self) -> (ConceptId, IdAllocator) {
        let id = ConceptId(self.next_concept);
        (
            id,
            IdAllocator {
                next_concept: self.next_concept + 1,
                ..self.clone()
            },
        )
    }
    #[must_use]
    pub fn fresh_decl(&self) -> (DeclId, IdAllocator) {
        let id = DeclId(self.next_decl);
        (
            id,
            IdAllocator {
                next_decl: self.next_decl + 1,
                ..self.clone()
            },
        )
    }
    #[must_use]
    pub fn fresh_clock(&self) -> (ClockId, IdAllocator) {
        let id = ClockId(self.next_clock);
        (
            id,
            IdAllocator {
                next_clock: self.next_clock + 1,
                ..self.clone()
            },
        )
    }
    #[must_use]
    pub fn fresh_output(&self) -> (OutputId, IdAllocator) {
        let id = OutputId(self.next_output);
        (
            id,
            IdAllocator {
                next_output: self.next_output + 1,
                ..self.clone()
            },
        )
    }
    /// An allocator that will never issue an identity at or below the
    /// given raw values (per sort): for a design assembled from copies of
    /// another design's entities, which keep their identities.
    #[must_use]
    pub fn covering(
        &self,
        concept: Option<u64>,
        decl: Option<u64>,
        clock: Option<u64>,
        output: Option<u64>,
        device: Option<u64>,
    ) -> IdAllocator {
        let above = |next: u64, used: Option<u64>| used.map_or(next, |u| next.max(u + 1));
        IdAllocator {
            next_concept: above(self.next_concept, concept),
            next_decl: above(self.next_decl, decl),
            next_clock: above(self.next_clock, clock),
            next_output: above(self.next_output, output),
            next_device: above(self.next_device, device),
        }
    }
    #[must_use]
    pub fn fresh_device(&self) -> (DeviceId, IdAllocator) {
        let id = DeviceId(self.next_device);
        (
            id,
            IdAllocator {
                next_device: self.next_device + 1,
                ..self.clone()
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_never_reuses() {
        let a = IdAllocator::default();
        let (s0, a) = a.fresh_concept();
        let (s1, a) = a.fresh_concept();
        let (d0, _) = a.fresh_decl();
        assert_ne!(s0, s1);
        assert_eq!(s0.raw(), 0);
        assert_eq!(s1.raw(), 1);
        assert_eq!(d0.raw(), 0); // sorts are independent
    }

    #[test]
    fn ids_serialize_transparently() {
        let json = serde_json::to_string(&ConceptId(7)).unwrap();
        assert_eq!(json, "7");
        let back: ConceptId = serde_json::from_str("7").unwrap();
        assert_eq!(back, ConceptId(7));
    }
}
