//! Cancellation at the IDE service boundary.
//!
//! A [`CancellationToken`] is a shared flag a long-running query polls at
//! its own checkpoints.  The host hands one out per request through the
//! [`RequestTracker`], scoped to what the request depends on (one overlay
//! key, or the whole project), and flips every token in a scope when that
//! scope changes: typing `T`, `Ti`, `Til`, `Tilt` cancels three analyses
//! that could not matter any more.
//!
//! Cancellation is co-operative and best-effort.  The correctness backstop
//! is the stamp on every result ([`crate::stamp::ResultGate`]); a query
//! that never polled still cannot publish a stale answer, it can only waste
//! time.  The compiler's `analyze` is not itself cancellable today (it is
//! one pure call over a snapshot); the token is checked before and after
//! it, and inside the IDE-level loops that follow.

use crate::overlay::OverlayKey;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The error a cancelled query returns instead of a result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, thiserror::Error)]
#[error("the request was cancelled")]
pub struct Cancelled;

/// A shared, one-way cancellation flag.  Cloning shares the flag.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    flag: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> CancellationToken {
        CancellationToken::default()
    }

    /// A token that can never be cancelled, for synchronous callers.
    pub fn never() -> CancellationToken {
        CancellationToken::default()
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    /// The checkpoint call: `token.check()?` inside a query.
    pub fn check(&self) -> Result<(), Cancelled> {
        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Identity of one in-flight request, allocated by the tracker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(pub u64);

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "req#{}", self.0)
    }
}

/// What a request's answer depends on, and therefore what invalidates it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CancelScope {
    /// Any change to the committed project or to any overlay.
    Project,
    /// A change to one overlay (or to the committed project).
    Overlay(OverlayKey),
}

/// The in-flight requests of one host, by scope.
#[derive(Debug, Default)]
pub struct RequestTracker {
    next: u64,
    live: BTreeMap<RequestId, (CancelScope, CancellationToken)>,
}

impl RequestTracker {
    /// Register a request; the token is what the query polls.
    pub fn begin(&mut self, scope: CancelScope) -> (RequestId, CancellationToken) {
        let id = RequestId(self.next);
        self.next += 1;
        let token = CancellationToken::new();
        self.live.insert(id, (scope, token.clone()));
        (id, token)
    }

    /// The request finished (or was abandoned); forget it.
    pub fn end(&mut self, id: RequestId) {
        self.live.remove(&id);
    }

    /// Cancel one request explicitly (an LSP `$/cancelRequest`).
    pub fn cancel(&mut self, id: RequestId) -> bool {
        match self.live.remove(&id) {
            Some((_, token)) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    /// An overlay changed: cancel every request scoped to it and every
    /// project-scoped request.  Returns how many were cancelled.
    pub fn overlay_changed(&mut self, key: OverlayKey) -> usize {
        self.cancel_where(|scope| match scope {
            CancelScope::Project => true,
            CancelScope::Overlay(k) => *k == key,
        })
    }

    /// The committed project changed: everything in flight is obsolete.
    pub fn project_changed(&mut self) -> usize {
        self.cancel_where(|_| true)
    }

    pub fn live_count(&self) -> usize {
        self.live.len()
    }

    fn cancel_where(&mut self, mut pred: impl FnMut(&CancelScope) -> bool) -> usize {
        let mut n = 0;
        self.live.retain(|_, (scope, token)| {
            if pred(scope) {
                token.cancel();
                n += 1;
                false
            } else {
                true
            }
        });
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::DeclId;

    #[test]
    fn overlay_change_cancels_its_own_scope_and_project_scope_only() {
        let a = OverlayKey::MappingDefinition {
            mapping: DeclId::from_raw(1),
        };
        let b = OverlayKey::MappingDefinition {
            mapping: DeclId::from_raw(2),
        };
        let mut t = RequestTracker::default();
        let (_, ta) = t.begin(CancelScope::Overlay(a));
        let (_, tb) = t.begin(CancelScope::Overlay(b));
        let (_, tp) = t.begin(CancelScope::Project);
        assert_eq!(t.overlay_changed(a), 2);
        assert!(ta.is_cancelled());
        assert!(!tb.is_cancelled());
        assert!(tp.is_cancelled());
        assert_eq!(t.live_count(), 1);
        assert_eq!(t.project_changed(), 1);
        assert!(tb.is_cancelled());
        assert_eq!(tb.check(), Err(Cancelled));
    }
}
