//! The boundary of a group — a *projection* of the design and a member
//! list, never a declaration (FV Phase 8b `Boundary.lean`).
//!
//! Everything here is read off the existing dependency graph
//! (`bdl_reactive::DependencyGraph`, the kernel's `DependsOn`): what a
//! member depends on outside the group is crossing in, what outside
//! depends on inside is crossing out, a member with no realization is
//! open, a member with a drive edge is driven.  The aggregate sockets Studio
//! draws on a collapsed group are these lists; they add no edge to the
//! graph (Theorem H, `socket_no_fanout`) and no fan-out to the language.

use bdl_compiler::ProjectAnalysis;
use bdl_ir::Expr;
use bdl_model::surface::Design;
use bdl_model::{ClockId, DeclId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GroupBoundary {
    /// Members that exist as relationships, in the group's order.
    pub members: Vec<DeclId>,
    /// FV `crossIn`: non-members some member depends on.
    pub crossing_in: Vec<DeclId>,
    /// FV `crossOut`: members some non-member depends on.
    pub crossing_out: Vec<DeclId>,
    /// FV `openMembers`: members with no realization.
    pub open_members: Vec<DeclId>,
    /// FV `drivenMembers`: members that drive a physical sink.
    pub driven_members: Vec<DeclId>,
    /// FV `privateMembers`: members neither crossing out nor driving.
    pub private_candidates: Vec<DeclId>,
    /// FV `externalInputs = crossIn ++ openMembers`.
    pub external_inputs: Vec<DeclId>,
    /// FV `externalOutputs = crossOut`.  Sinks are `driven_members`, never
    /// turned into semantic ports.
    pub external_outputs: Vec<DeclId>,
    /// Every timing domain the members and the crossing-in declarations
    /// use, including domains read only through `sync` inside a member's
    /// formula: the clock parameters an extraction needs.
    pub clocks: Vec<ClockId>,
    /// Internal edges: producer and consumer both members.
    pub internal_edges: Vec<(DeclId, DeclId)>,
}

/// The boundary of `members` in `design`, with the dependency graph and IR
/// of `analysis` (the flattened design's; base relationships keep their
/// identities there).
pub fn group_boundary(
    design: &Design,
    analysis: &ProjectAnalysis,
    members: &[DeclId],
) -> GroupBoundary {
    let deps = &analysis.dependencies;
    let members: Vec<DeclId> = members
        .iter()
        .copied()
        .filter(|d| design.mappings.contains_key(d))
        .collect();
    let in_group: BTreeSet<DeclId> = members.iter().copied().collect();
    let outside: Vec<DeclId> = deps
        .nodes
        .iter()
        .copied()
        .filter(|d| !in_group.contains(d))
        .collect();

    let crossing_in: Vec<DeclId> = outside
        .iter()
        .copied()
        .filter(|r| members.iter().any(|m| deps.depends_on(*m, *r)))
        .collect();
    let crossing_out: Vec<DeclId> = members
        .iter()
        .copied()
        .filter(|p| outside.iter().any(|u| deps.depends_on(*u, *p)))
        .collect();
    let open_members: Vec<DeclId> = members
        .iter()
        .copied()
        .filter(|m| {
            design
                .mappings
                .get(m)
                .is_some_and(|x| x.definition.is_none())
        })
        .collect();
    let driven_members: Vec<DeclId> = members
        .iter()
        .copied()
        .filter(|m| design.mappings.get(m).is_some_and(|x| x.drives.is_some()))
        .collect();
    let private_candidates: Vec<DeclId> = members
        .iter()
        .copied()
        .filter(|m| !crossing_out.contains(m) && !driven_members.contains(m))
        .collect();
    let internal_edges: Vec<(DeclId, DeclId)> = members
        .iter()
        .flat_map(|a| members.iter().map(move |b| (*a, *b)))
        .filter(|(a, b)| deps.depends_on(*a, *b))
        .collect();

    let mut clocks: BTreeSet<ClockId> = BTreeSet::new();
    for d in members.iter().chain(crossing_in.iter()) {
        if let Some(c) = design.mappings.get(d).and_then(|m| m.clock) {
            clocks.insert(c);
        }
    }
    for m in &members {
        if let Some(e) = analysis
            .ir
            .decls
            .get(m)
            .and_then(|d| d.realization.as_ref())
        {
            collect_sync_clocks(e, &mut clocks);
        }
    }

    let mut external_inputs = crossing_in.clone();
    external_inputs.extend(open_members.iter().copied());
    GroupBoundary {
        members,
        external_outputs: crossing_out.clone(),
        crossing_in,
        crossing_out,
        open_members,
        driven_members,
        private_candidates,
        external_inputs,
        clocks: clocks.into_iter().collect(),
        internal_edges,
    }
}

/// Domains a formula observes through `sync`, which `Κ` alone does not
/// list (a body-only clock: BEHAVIOR_GROUPING_NOTE §II.4).
pub fn collect_sync_clocks(e: &Expr, out: &mut BTreeSet<ClockId>) {
    match e {
        Expr::Sync { src, init, e } => {
            out.insert(*src);
            collect_sync_clocks(init, out);
            collect_sync_clocks(e, out);
        }
        Expr::Delay { init, e } => {
            collect_sync_clocks(init, out);
            collect_sync_clocks(e, out);
        }
        Expr::App { f, a } => {
            collect_sync_clocks(f, out);
            collect_sync_clocks(a, out);
        }
        Expr::Lam { body, .. } => collect_sync_clocks(body, out),
        Expr::Rep { e } | Expr::Mk { e, .. } => collect_sync_clocks(e, out),
        _ => {}
    }
}
