//! Snapshot stamps and the stale-result gate.
//!
//! Every analysis snapshot — and therefore every query result — carries
//! the committed revision and the overlay generation it was computed for.
//! Adapters compare stamps, never timestamps or arrival order: the
//! backstop that makes out-of-order delivery harmless even when a
//! cancellation was missed.

use crate::overlay::OverlayGeneration;
use bdl_model::Revision;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The semantic world a result describes: which committed revision, and
/// which overlay generation on top of it.  Totally ordered — a newer
/// revision beats any generation, then generations compare — so "is this
/// newer than what I show?" is `>`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SnapshotStamp {
    pub revision: Revision,
    pub generation: OverlayGeneration,
}

impl SnapshotStamp {
    pub const fn new(revision: Revision, generation: OverlayGeneration) -> SnapshotStamp {
        SnapshotStamp {
            revision,
            generation,
        }
    }
}

impl fmt::Display for SnapshotStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.revision, self.generation)
    }
}

/// Keeps only the newest result a client has seen.  `offer` answers
/// whether the caller may publish: a result older than, or equal to, the
/// one already shown is refused.  Deterministic whatever order results
/// arrive in — deliver generations `2, 1, 3` and only `3` is ever the last
/// thing accepted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResultGate {
    shown: Option<SnapshotStamp>,
}

impl ResultGate {
    pub fn new() -> ResultGate {
        ResultGate::default()
    }

    /// The stamp currently on show, if any.
    pub fn shown(&self) -> Option<SnapshotStamp> {
        self.shown
    }

    /// Accept `stamp` if it is strictly newer than what is shown.
    pub fn offer(&mut self, stamp: SnapshotStamp) -> bool {
        if self.shown.is_some_and(|s| stamp <= s) {
            return false;
        }
        self.shown = Some(stamp);
        true
    }

    /// Accept `stamp` only if it is newer than what is shown *and* is
    /// exactly the world the host is at now (`current`): the strict form
    /// for clients that never want an intermediate state on screen.
    pub fn offer_current(&mut self, stamp: SnapshotStamp, current: SnapshotStamp) -> bool {
        if stamp != current {
            return false;
        }
        self.offer(stamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(n: u64) -> SnapshotStamp {
        SnapshotStamp::new(Revision::from_raw(3), OverlayGeneration(n))
    }

    #[test]
    fn out_of_order_delivery_leaves_only_the_newest_visible() {
        let mut gate = ResultGate::new();
        assert!(gate.offer(g(2)));
        assert!(!gate.offer(g(1)), "older than what is shown");
        assert!(gate.offer(g(3)));
        assert!(!gate.offer(g(3)), "the same world twice is not news");
        assert_eq!(gate.shown(), Some(g(3)));
    }

    #[test]
    fn a_newer_revision_beats_any_generation() {
        let old = SnapshotStamp::new(Revision::from_raw(1), OverlayGeneration(99));
        let new = SnapshotStamp::new(Revision::from_raw(2), OverlayGeneration(0));
        assert!(new > old);
        let mut gate = ResultGate::new();
        assert!(gate.offer(old));
        assert!(gate.offer(new));
        assert!(!gate.offer(old));
    }

    #[test]
    fn strict_form_shows_only_the_current_world() {
        let mut gate = ResultGate::new();
        assert!(!gate.offer_current(g(2), g(3)));
        assert!(gate.offer_current(g(3), g(3)));
        assert_eq!(gate.shown(), Some(g(3)));
    }
}
