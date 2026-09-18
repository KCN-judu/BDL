//! Capacity of a cross-domain window — the production counterpart of
//! `BDL/Validation/Capacity.lean` (FV Phase 9a).
//!
//! The kernel semantics of a buffered transport is the unbounded list; a
//! deployment has finite memory, so it must show that no window ever
//! exceeds its capacity.  Everything here is a function of the
//! [`Schedule`] alone — rates are deployment data — and of a pair of
//! domains: the *window* at tick `t` is the source's activations since
//! the destination's last activation strictly before `t`
//! (`windowTicks S src dst t`).
//!
//! * [`window_ticks`] / [`window_len`] — the window and its size;
//! * [`capacity_sufficient`] — every window up to a horizon fits
//!   (`CapacitySufficient`, decidable);
//! * [`required_capacity`] — the least sufficient capacity for a horizon
//!   (`requiredCapacity`; `requiredCapacity_sufficient` is a test here);
//! * [`required_capacity_periodic`] — for the periodic schedules
//!   production has, the exact bound at every horizon: the windows repeat
//!   with the two periods' least common multiple.  It never exceeds the
//!   destination's period, FV's `periodic_capacity_sufficient` (tested);
//! * [`drop_oldest`] / [`drop_newest`] — the overflow *policies* as
//!   functions of the unbounded window.  Under sufficient capacity both
//!   are the identity (`sufficient_capacity_preserves`, tested); under an
//!   insufficient one they change the trace, so only rejecting such a
//!   deployment keeps the unbounded semantics.
//!
//! None of this touches typing or list semantics: a design bounds a
//! window by writing the bound (`take cap …`), and validation compares it
//! with what the schedule requires (`bdl-compiler::collections`).

use crate::simulate::Schedule;
use bdl_model::ClockId;

/// The last activation of `c` strictly before `t`, if any (`prevAct`).
pub fn prev_activation(s: &Schedule, c: ClockId, t: u64) -> Option<u64> {
    let p = *s.periods.get(&c)?;
    if p == 0 || t == 0 {
        return None;
    }
    Some((t - 1) / p * p)
}

/// The activations of `src` in `[lo, hi)` (`srcTicks`).
pub fn source_ticks(s: &Schedule, src: ClockId, lo: u64, hi: u64) -> Vec<u64> {
    let Some(p) = s.periods.get(&src).copied().filter(|p| *p > 0) else {
        return Vec::new();
    };
    if hi <= lo {
        return Vec::new();
    }
    let first = lo.div_ceil(p) * p;
    (first..hi).step_by(p as usize).collect()
}

/// The source activations the destination sees as new at `t`
/// (`windowTicks`): those since its previous activation, or since the
/// start when it has none.
pub fn window_ticks(s: &Schedule, src: ClockId, dst: ClockId, t: u64) -> Vec<u64> {
    let lo = prev_activation(s, dst, t).unwrap_or(0);
    source_ticks(s, src, lo, t)
}

pub fn window_len(s: &Schedule, src: ClockId, dst: ClockId, t: u64) -> u64 {
    window_ticks(s, src, dst, t).len() as u64
}

/// `CapacitySufficient S src dst cap T`: every window up to and including
/// `horizon` holds at most `cap` values.
pub fn capacity_sufficient(
    s: &Schedule,
    src: ClockId,
    dst: ClockId,
    cap: u64,
    horizon: u64,
) -> bool {
    (0..=horizon).all(|t| window_len(s, src, dst, t) <= cap)
}

/// `requiredCapacity S src dst T`: the least sufficient capacity for the
/// horizon.
pub fn required_capacity(s: &Schedule, src: ClockId, dst: ClockId, horizon: u64) -> u64 {
    (0..=horizon)
        .map(|t| window_len(s, src, dst, t))
        .max()
        .unwrap_or(0)
}

/// The required capacity at every horizon for the periodic schedule:
/// the windows repeat with the least common multiple of the two periods,
/// so the finite horizon covers all of them.  `None` when either domain
/// never activates.
pub fn required_capacity_periodic(s: &Schedule, src: ClockId, dst: ClockId) -> Option<u64> {
    let ps = s.periods.get(&src).copied().filter(|p| *p > 0)?;
    let pd = s.periods.get(&dst).copied().filter(|p| *p > 0)?;
    let horizon = lcm(ps, pd).saturating_add(pd);
    Some(required_capacity(s, src, dst, horizon))
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}
fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

/// Keep the newest `cap` entries of a window (oldest first).
pub fn drop_oldest<T: Clone>(cap: usize, w: &[T]) -> Vec<T> {
    w[w.len().saturating_sub(cap)..].to_vec()
}

/// Keep the oldest `cap` entries of a window (oldest first).
pub fn drop_newest<T: Clone>(cap: usize, w: &[T]) -> Vec<T> {
    w[..cap.min(w.len())].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn c(n: u64) -> ClockId {
        ClockId::from_raw(n)
    }
    fn sched(fast: u64, slow: u64) -> Schedule {
        let mut s = Schedule::default();
        s.periods.insert(c(0), fast);
        s.periods.insert(c(1), slow);
        s
    }

    #[test]
    fn the_window_is_the_source_activations_since_the_last_destination_activation() {
        let s = sched(1, 3);
        assert_eq!(window_ticks(&s, c(0), c(1), 0), Vec::<u64>::new());
        assert_eq!(window_ticks(&s, c(0), c(1), 1), vec![0]);
        assert_eq!(window_ticks(&s, c(0), c(1), 3), vec![0, 1, 2]);
        assert_eq!(window_ticks(&s, c(0), c(1), 4), vec![3]);
        assert_eq!(window_ticks(&s, c(0), c(1), 6), vec![3, 4, 5]);
        // the other way round: at most one slow activation per fast window
        assert_eq!(window_ticks(&s, c(1), c(0), 4), vec![3]);
        assert_eq!(window_ticks(&s, c(1), c(0), 5), Vec::<u64>::new());
        let s = sched(2, 3);
        assert_eq!(window_ticks(&s, c(0), c(1), 3), vec![0, 2]);
        assert_eq!(window_ticks(&s, c(0), c(1), 6), vec![4]);
        assert_eq!(required_capacity_periodic(&s, c(0), c(1)), Some(2));
        assert_eq!(
            required_capacity_periodic(&sched(1, 3), c(0), c(1)),
            Some(3)
        );
        assert_eq!(
            required_capacity_periodic(&sched(3, 1), c(0), c(1)),
            Some(1)
        );
    }

    proptest! {
        /// `requiredCapacity_sufficient`.
        #[test]
        fn required_capacity_is_sufficient(fast in 1u64..6, slow in 1u64..9, horizon in 0u64..60) {
            let s = sched(fast, slow);
            let cap = required_capacity(&s, c(0), c(1), horizon);
            prop_assert!(capacity_sufficient(&s, c(0), c(1), cap, horizon));
            prop_assert!(cap == 0 || !capacity_sufficient(&s, c(0), c(1), cap - 1, horizon));
        }

        /// `periodic_capacity_sufficient`: one destination period always
        /// suffices, and the periodic bound is exact at every horizon.
        #[test]
        fn the_periodic_bound_is_exact_and_within_one_destination_period(fast in 1u64..6, slow in 1u64..9, horizon in 0u64..200) {
            let s = sched(fast, slow);
            let cap = required_capacity_periodic(&s, c(0), c(1)).unwrap();
            prop_assert!(cap <= slow);
            prop_assert!(capacity_sufficient(&s, c(0), c(1), cap, horizon));
            prop_assert!(required_capacity(&s, c(0), c(1), horizon) <= cap);
            // and reached: the finite horizon that covers the repetition
            prop_assert_eq!(required_capacity(&s, c(0), c(1), 2 * fast * slow + slow), cap);
        }

        /// `sufficient_capacity_preserves`: both policies are the identity
        /// on a window that fits; on one that does not, each keeps its end.
        #[test]
        fn policies_are_the_identity_under_sufficient_capacity(w in prop::collection::vec(0i64..100, 0..12), cap in 0usize..14) {
            if w.len() <= cap {
                prop_assert_eq!(drop_oldest(cap, &w), w.clone());
                prop_assert_eq!(drop_newest(cap, &w), w.clone());
            } else {
                prop_assert_eq!(drop_oldest(cap, &w), w[w.len() - cap..].to_vec());
                prop_assert_eq!(drop_newest(cap, &w), w[..cap].to_vec());
                prop_assert_ne!(drop_oldest(cap, &w), w.clone());
            }
        }
    }
}
