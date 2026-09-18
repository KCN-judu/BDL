//! Collections at deployment: how large the lists a program carries can
//! get, what a cross-domain window needs under the deployment schedule,
//! and whether that fits a target with finite memory.
//!
//! ```text
//! ExecIr ──bounds::analyse──▶ Shape per declaration / cell
//!                                   │
//! Schedule ──capacity────────▶ required window capacity per (src, dst)
//!                                   │
//!                                   ▼
//!                       CollectionsReport + deployment.* diagnostics
//! ```
//!
//! The list semantics is untouched (ADR-0024): a design bounds its
//! collections by writing the bound (`take cap …`), this module tells the
//! designer where it has not, and a target that requires bounded memory
//! refuses to deploy an unbounded design instead of dropping values
//! silently (FV `Validation/Capacity.lean`: only rejection keeps the
//! unbounded semantics).  Specification: docs/spec/deployment-capacity.md.

use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_exec_ir::bounds::{self, Bound, Bounds, Shape};
use bdl_exec_ir::{Activation, DeclKind, ExecIr};
use bdl_model::{ClockId, DeclId};
use bdl_reactive::capacity;
use bdl_reactive::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What the program's collections need of a target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionsReadiness {
    /// No list anywhere: allocation-free, `Copy` end to end.
    ScalarOnly,
    /// Lists whose sizes are fixed by the design: an allocator, and a
    /// memory need the report states.
    Bounded,
    /// Lists as large as what the host supplies: an allocator, and the
    /// platform adapter must bound its inputs.
    InputBounded,
    /// A remembered collection grows without bound: not deployable on
    /// finite memory as written.
    Unbounded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellCapacity {
    pub slot: u32,
    /// The owning declaration.
    pub decl_id: DeclId,
    pub name: String,
    pub ty: String,
    pub shape: Shape,
    /// Upper bound in bytes as the generated core stores it; `None` when
    /// not bounded by the design.
    pub bytes: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclCapacity {
    pub decl_id: DeclId,
    pub name: String,
    pub ty: String,
    pub shape: Shape,
    pub bytes: Option<u64>,
    /// An unresolved declaration of list type: the host bounds it.
    pub input: bool,
}

/// One cross-domain transport carrying lists, with what the schedule
/// requires of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowCapacity {
    pub source: ClockId,
    pub source_name: String,
    pub destination: ClockId,
    pub destination_name: String,
    /// The most source activations between two destination activations
    /// under the schedule (`required_capacity_periodic`); `None` without
    /// a schedule.
    pub required: Option<u64>,
    /// The list-carrying `sync` cells from source to destination: the
    /// owning declaration and the bound the design gives the list.
    pub carried: Vec<(DeclId, String, Bound)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionsReport {
    pub readiness: CollectionsReadiness,
    pub cells: Vec<CellCapacity>,
    pub decls: Vec<DeclCapacity>,
    pub windows: Vec<WindowCapacity>,
    /// Upper bound of the state's heap in bytes; `None` when unbounded.
    pub state_bytes_max: Option<u64>,
    /// Upper bound of one tick's declaration values in bytes.
    pub tick_bytes_max: Option<u64>,
}

/// The report for a lowered program under an optional deployment
/// schedule.
pub fn collections_report(ir: &ExecIr, schedule: Option<&Schedule>) -> CollectionsReport {
    let b: Bounds = bounds::analyse(ir);
    let mut cells = Vec::new();
    let mut state_bytes = Some(0u64);
    for (c, shape) in ir.cells.iter().zip(&b.cells) {
        let owner = ir.decl(c.owner);
        let bytes = shape.bytes(&c.ty);
        state_bytes = match (state_bytes, bytes) {
            (Some(a), Some(x)) => Some(a + x),
            _ => None,
        };
        cells.push(CellCapacity {
            slot: c.slot.0,
            decl_id: owner.map(|d| d.id).unwrap_or(DeclId::from_raw(0)),
            name: owner.map(|d| d.name.clone()).unwrap_or_default(),
            ty: bdl_check::pretty::kernel(&c.ty),
            shape: shape.clone(),
            bytes,
        });
    }
    let mut decls = Vec::new();
    let mut tick_bytes = Some(0u64);
    for (d, shape) in ir.decls.iter().zip(&b.decls) {
        let bytes = shape.bytes(&d.ty);
        tick_bytes = match (tick_bytes, bytes) {
            (Some(a), Some(x)) => Some(a + x),
            _ => None,
        };
        decls.push(DeclCapacity {
            decl_id: d.id,
            name: d.name.clone(),
            ty: bdl_check::pretty::kernel(&d.ty),
            shape: shape.clone(),
            bytes,
            input: matches!(d.kind, DeclKind::Input { .. }) && bdl_exec_ir::ty_uses_lists(&d.ty),
        });
    }
    // windows: list-carrying sync cells grouped by (source, destination)
    let mut windows: BTreeMap<(ClockId, ClockId), WindowCapacity> = BTreeMap::new();
    for (c, shape) in ir.cells.iter().zip(&b.cells) {
        let Some(owner) = ir.decl(c.owner) else {
            continue;
        };
        let Activation::Domain { clock: dst } = owner.activation else {
            continue;
        };
        if c.writer == dst || !bdl_exec_ir::ty_uses_lists(&c.ty) {
            continue;
        }
        let name = |slot| {
            ir.clocks
                .iter()
                .find(|k| k.slot == slot)
                .map(|k| (k.id, k.name.clone()))
        };
        let (Some((src_id, src_name)), Some((dst_id, dst_name))) = (name(c.writer), name(dst))
        else {
            continue;
        };
        let bound = match shape {
            Shape::List { bound, .. } => *bound,
            s if s.is_unbounded() => Bound::Unbounded,
            s if s.depends_on_input() => Bound::Input,
            _ => Bound::Finite { elements: 0 },
        };
        windows
            .entry((src_id, dst_id))
            .or_insert_with(|| WindowCapacity {
                source: src_id,
                source_name: src_name,
                destination: dst_id,
                destination_name: dst_name,
                required: schedule
                    .and_then(|s| capacity::required_capacity_periodic(s, src_id, dst_id)),
                carried: Vec::new(),
            })
            .carried
            .push((owner.id, owner.name.clone(), bound));
    }
    let readiness = if !ir.uses_lists() {
        CollectionsReadiness::ScalarOnly
    } else if b.cells.iter().any(Shape::is_unbounded) || b.decls.iter().any(Shape::is_unbounded) {
        CollectionsReadiness::Unbounded
    } else if b.cells.iter().any(Shape::depends_on_input)
        || b.decls.iter().any(Shape::depends_on_input)
    {
        CollectionsReadiness::InputBounded
    } else {
        CollectionsReadiness::Bounded
    };
    CollectionsReport {
        readiness,
        cells,
        decls,
        windows: windows.into_values().collect(),
        state_bytes_max: state_bytes,
        tick_bytes_max: tick_bytes,
    }
}

/// How strict a deployment is about memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryPolicy {
    /// A host with virtual memory: growth is reported, never refused.
    #[default]
    Host,
    /// A target with finite memory: an unbounded collection refuses the
    /// deployment; an input-sized one is the platform adapter's to bound.
    Bounded,
}

/// The designer-facing diagnostics of a report.
pub fn collections_diagnostics(r: &CollectionsReport, policy: MemoryPolicy) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut said = std::collections::BTreeSet::new();
    for c in &r.cells {
        if !c.shape.is_unbounded() || !said.insert(c.decl_id) {
            continue;
        }
        let d = match policy {
            MemoryPolicy::Bounded => Diagnostic::error,
            MemoryPolicy::Host => Diagnostic::warning,
        };
        out.push(
            d(
                "deployment.unbounded_list_state",
                Entity::Mapping { id: c.decl_id },
                format!("{} keeps every value it has ever received.", c.name),
            )
            .explain(format!(
                "A relationship that adds to a remembered collection at every activation grows without bound, and a deployed core has no room for that. Keep only what is needed: the newest N values are take(N, …).{}",
                match policy {
                    MemoryPolicy::Bounded => " This target has finite memory, so the design is not deployed as written.",
                    MemoryPolicy::Host => "",
                }
            ))
            .technical(format!(
                "state cell {} ({}): shape {:?}; FV Capacity.lean: only rejecting an insufficient deployment keeps the unbounded semantics",
                c.slot, c.ty, c.shape
            )),
        );
    }
    if policy == MemoryPolicy::Bounded {
        for d in r.decls.iter().filter(|d| d.input) {
            out.push(
                Diagnostic::warning(
                    "deployment.list_input_unbounded",
                    Entity::Mapping { id: d.decl_id },
                    format!("{} is a collection supplied from outside; its size is the platform's to bound.", d.name),
                )
                .explain("The design does not limit how many values arrive here at once. The platform adapter must state the most it will supply, or the design can keep a fixed number with take(N, …).")
                .technical(format!("input of type {}; bound Input", d.ty)),
            );
        }
    }
    for w in &r.windows {
        let Some(required) = w.required else {
            continue;
        };
        for (id, name, bound) in &w.carried {
            let Bound::Finite { elements } = bound else {
                continue;
            };
            if *elements >= required {
                continue;
            }
            out.push(
                Diagnostic::warning(
                    "deployment.window_capacity",
                    Entity::Mapping { id: *id },
                    format!(
                        "Between two activations of {}, {} produces up to {} value{}, but {} keeps {}.",
                        w.destination_name,
                        w.source_name,
                        required,
                        if required == 1 { "" } else { "s" },
                        name,
                        elements
                    ),
                )
                .explain(format!(
                    "Values may be dropped before {} reads them. If keeping only the newest is intended, nothing to do; otherwise keep at least {}.",
                    w.destination_name, required
                ))
                .technical(format!(
                    "required capacity {required} for ({} → {}) under the schedule; sync cell bound {elements}; FV requiredCapacity / dropOldest",
                    w.source_name, w.destination_name
                )),
            );
        }
    }
    sort_diagnostics(&mut out);
    out
}
