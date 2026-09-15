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
    /// Identity of a semantic concept (the kernel's `SemanticId`; a nominal type).
    SemanticId,
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
    next_semantic: u64,
    next_decl: u64,
    next_clock: u64,
    next_output: u64,
}

impl IdAllocator {
    #[must_use]
    pub fn fresh_semantic(&self) -> (SemanticId, IdAllocator) {
        let id = SemanticId(self.next_semantic);
        (
            id,
            IdAllocator {
                next_semantic: self.next_semantic + 1,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_never_reuses() {
        let a = IdAllocator::default();
        let (s0, a) = a.fresh_semantic();
        let (s1, a) = a.fresh_semantic();
        let (d0, _) = a.fresh_decl();
        assert_ne!(s0, s1);
        assert_eq!(s0.raw(), 0);
        assert_eq!(s1.raw(), 1);
        assert_eq!(d0.raw(), 0); // sorts are independent
    }

    #[test]
    fn ids_serialize_transparently() {
        let json = serde_json::to_string(&SemanticId(7)).unwrap();
        assert_eq!(json, "7");
        let back: SemanticId = serde_json::from_str("7").unwrap();
        assert_eq!(back, SemanticId(7));
    }
}
