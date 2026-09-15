//! Target-specific hardware validation (`BDL_FV/BDL/Validation/Hardware.lean`).
//!
//! This is a **validation layer**: nothing here enters types, typing, the
//! reactive semantics or the drive edges.  Feasibility is a relation between
//! a design's *requirements* and a *target's* resources — `Deployable(design,
//! target)` — and is re-solved from scratch whenever either changes; it is
//! never a commitment a declaration carries.
//!
//! * [`model`]   — `Capability`, `Resource` (capabilities + per-capability
//!   backing unit), `Hardware` (resources + capability-specific sharing),
//!   `Requirement` (identity, capability, fixed resource, unit-relation group).
//! * [`solve`]   — validity (`ReqOK`, `Compatible`, `ValidFor`), a
//!   deterministic exhaustive solver, and first-dead-end diagnosis.
//! * [`devices`] — device kinds → requirements (the solver never knows what
//!   an IMU is).
//! * [`boards`]  — concrete targets as data.

#![forbid(unsafe_code)]

pub mod boards;
pub mod devices;
pub mod model;
pub mod solve;

pub use model::{
    Capability, GroupId, Hardware, Requirement, RequirementId, Resource, ResourceId, UnitId,
    UnitRel,
};
pub use solve::{diagnose, solve, validate, Assignment, DeadEnd, DeadEndReason, Violation};
