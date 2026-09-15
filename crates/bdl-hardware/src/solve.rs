//! Validity, the solver, and diagnosis.
//!
//! Every constraint is unary (`ReqOK`) or binary (`Compatible`), so validity
//! is prefix-closed and an exhaustive depth-first search that prunes on
//! them is complete as well as sound (proved in Lean for the abstract
//! model; tested here against brute force).
//!
//! **Variable order**: fewest candidates first, ties broken by
//! `RequirementId` — a heuristic that changes search cost, never the
//! answer's existence; the *assignment* returned is the first in this
//! documented order, hence deterministic.  **Value order**: the board's
//! declared resource order.
//!
//! `diagnose` reports the first dead end under greedy placement in the same
//! order: *a* conflict, not a minimal unsatisfiable core, and meaningful only
//! after `solve` returned `None`.

use crate::model::{Hardware, Requirement, RequirementId, ResourceId, UnitRel};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Assignment = BTreeMap<RequirementId, ResourceId>;

/// `ReqOK H req r`: the resource has the capability and honours a fixed choice.
pub fn req_ok(hw: &Hardware, req: &Requirement, r: &ResourceId) -> bool {
    hw.supports(r, req.capability) && req.fixed.as_ref().is_none_or(|f| f == r)
}

/// `Compatible H (a, ra) (b, rb)`.
pub fn compatible(
    hw: &Hardware,
    a: &Requirement,
    ra: &ResourceId,
    b: &Requirement,
    rb: &ResourceId,
) -> bool {
    if ra == rb && !(a.capability == b.capability && hw.is_shareable(a.capability)) {
        return false;
    }
    match (a.group, b.group) {
        (Some((ga, rel_a)), Some((gb, rel_b))) if ga == gb && rel_a == rel_b => {
            let ua = hw.unit_of(ra, a.capability);
            let ub = hw.unit_of(rb, b.capability);
            match rel_a {
                UnitRel::Same => ua == ub,
                UnitRel::Distinct => ua != ub,
            }
        }
        _ => true,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Violation {
    Unassigned {
        requirement: RequirementId,
    },
    UnknownRequirement {
        requirement: RequirementId,
    },
    Unsupported {
        requirement: RequirementId,
        resource: ResourceId,
    },
    FixedIgnored {
        requirement: RequirementId,
        fixed: ResourceId,
        resource: ResourceId,
    },
    Incompatible {
        a: RequirementId,
        b: RequirementId,
    },
}

/// `ValidFor H R A`: covers exactly `reqs`, every entry supported, every
/// pair compatible.  Returns every violation, in a stable order.
pub fn validate(hw: &Hardware, reqs: &[Requirement], a: &Assignment) -> Vec<Violation> {
    let mut v = Vec::new();
    let by_id: BTreeMap<RequirementId, &Requirement> = reqs.iter().map(|r| (r.id, r)).collect();
    for r in reqs {
        match a.get(&r.id) {
            None => v.push(Violation::Unassigned { requirement: r.id }),
            Some(res) => {
                if !hw.supports(res, r.capability) {
                    v.push(Violation::Unsupported {
                        requirement: r.id,
                        resource: res.clone(),
                    });
                } else if let Some(f) = &r.fixed {
                    if f != res {
                        v.push(Violation::FixedIgnored {
                            requirement: r.id,
                            fixed: f.clone(),
                            resource: res.clone(),
                        });
                    }
                }
            }
        }
    }
    for id in a.keys() {
        if !by_id.contains_key(id) {
            v.push(Violation::UnknownRequirement { requirement: *id });
        }
    }
    let entries: Vec<(&Requirement, &ResourceId)> = reqs
        .iter()
        .filter_map(|r| a.get(&r.id).map(|res| (r, res)))
        .collect();
    for (i, (ra, rra)) in entries.iter().enumerate() {
        for (rb, rrb) in &entries[i + 1..] {
            if !compatible(hw, ra, rra, rb, rrb) {
                v.push(Violation::Incompatible { a: ra.id, b: rb.id });
            }
        }
    }
    v
}

/// `candidates H req`, in the board's resource order.
pub fn candidates<'a>(hw: &'a Hardware, req: &Requirement) -> Vec<&'a ResourceId> {
    hw.resource_ids().filter(|r| req_ok(hw, req, r)).collect()
}

/// The documented variable order: fewest candidates first, then by id.
fn ordered<'a>(hw: &Hardware, reqs: &'a [Requirement]) -> Vec<&'a Requirement> {
    let mut v: Vec<&Requirement> = reqs.iter().collect();
    v.sort_by_key(|r| (candidates(hw, r).len(), r.id));
    v
}

/// Exhaustive DFS.  `Some` iff a valid assignment exists.
pub fn solve(hw: &Hardware, reqs: &[Requirement]) -> Option<Assignment> {
    let order = ordered(hw, reqs);
    let mut acc: Vec<(&Requirement, &ResourceId)> = Vec::new();
    if dfs(hw, &order, 0, &mut acc) {
        Some(
            acc.into_iter()
                .map(|(r, res)| (r.id, res.clone()))
                .collect(),
        )
    } else {
        None
    }
}

fn dfs<'a>(
    hw: &'a Hardware,
    order: &[&'a Requirement],
    i: usize,
    acc: &mut Vec<(&'a Requirement, &'a ResourceId)>,
) -> bool {
    if i == order.len() {
        return true;
    }
    let req = order[i];
    for r in candidates(hw, req) {
        if acc.iter().all(|(q, rq)| compatible(hw, q, rq, req, r)) {
            acc.push((req, r));
            if dfs(hw, order, i + 1, acc) {
                return true;
            }
            acc.pop();
        }
    }
    false
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeadEndReason {
    /// Nothing on the board has the capability (or the fixed pin lacks it).
    NoCapableResource,
    /// The manual pin choice is not on this board or lacks the capability.
    FixedUnavailable { fixed: ResourceId },
    /// Every candidate is taken by an already-placed requirement, or
    /// violates a unit relation with one.
    Blocked {
        candidates: Vec<(ResourceId, RequirementId)>,
    },
}

/// The first requirement (in solver order) that cannot be placed under
/// greedy placement, with what blocked each of its candidates.  *A*
/// conflict under one placement order — not a minimal unsat core.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeadEnd {
    pub requirement: RequirementId,
    pub reason: DeadEndReason,
    /// What greedy placement had already assigned when it got stuck.
    pub placed: Assignment,
}

pub fn diagnose(hw: &Hardware, reqs: &[Requirement]) -> Option<DeadEnd> {
    let order = ordered(hw, reqs);
    let mut acc: Vec<(&Requirement, &ResourceId)> = Vec::new();
    for req in order {
        let cands = candidates(hw, req);
        if cands.is_empty() {
            let reason = match &req.fixed {
                Some(f) => DeadEndReason::FixedUnavailable { fixed: f.clone() },
                None => DeadEndReason::NoCapableResource,
            };
            return Some(DeadEnd {
                requirement: req.id,
                reason,
                placed: placed(&acc),
            });
        }
        match cands
            .iter()
            .find(|r| acc.iter().all(|(q, rq)| compatible(hw, q, rq, req, r)))
        {
            Some(r) => acc.push((req, r)),
            None => {
                let blockers = cands
                    .iter()
                    .filter_map(|r| {
                        acc.iter()
                            .find(|(q, rq)| !compatible(hw, q, rq, req, r))
                            .map(|(q, _)| ((*r).clone(), q.id))
                    })
                    .collect();
                return Some(DeadEnd {
                    requirement: req.id,
                    reason: DeadEndReason::Blocked {
                        candidates: blockers,
                    },
                    placed: placed(&acc),
                });
            }
        }
    }
    None
}

fn placed(acc: &[(&Requirement, &ResourceId)]) -> Assignment {
    acc.iter().map(|(r, res)| (r.id, (*res).clone())).collect()
}

/// Every assignment of every requirement to every resource — the oracle
/// `solve` is tested against on small instances.
pub fn brute_force_satisfiable(hw: &Hardware, reqs: &[Requirement]) -> bool {
    let ids: Vec<&ResourceId> = hw.resource_ids().collect();
    if reqs.is_empty() {
        return true;
    }
    if ids.is_empty() {
        return false;
    }
    let n = reqs.len();
    let mut idx = vec![0usize; n];
    loop {
        let a: Assignment = reqs
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, ids[idx[i]].clone()))
            .collect();
        if validate(hw, reqs, &a).is_empty() {
            return true;
        }
        // increment the mixed-radix counter
        let mut k = 0;
        loop {
            if k == n {
                return false;
            }
            idx[k] += 1;
            if idx[k] < ids.len() {
                break;
            }
            idx[k] = 0;
            k += 1;
        }
    }
}
