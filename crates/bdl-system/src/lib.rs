//! Behaviour systems for production BDL: reusable components, instances
//! and bindings that elaborate into the same flat design the existing
//! compiler accepts.  No kernel construct, no second checker, no second
//! evaluator (FV Phase 8a, D-64; docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md).
//!
//! ```text
//! BehaviorSystem ──flatten──▶ ProjectSnapshot ──bdl_compiler::analyze──▶ ProjectAnalysis
//!        ▲                          │                                        │
//!   system edits                origins (provenance)                 origin projection
//! ```

#![forbid(unsafe_code)]

pub mod analyze;
pub mod edit;
pub mod flatten;
pub mod ids;
pub mod model;
pub mod package;
pub mod persist;
pub mod validate;

pub use analyze::{analyze_system, Acceptance, PortStatus, SystemAnalysis};
pub use edit::{
    apply_system_edit, AppliedSystem, SystemEditError, SystemEditOp, SystemEditOutcome,
};
pub use flatten::{flatten, FlattenedSystem, Origin, OriginMap};
pub use ids::{BindingId, ComponentId, ComponentInstanceId, ExportId, PortId, SystemIdAllocator};
pub use model::*;
pub use package::{package_system, PackageError, PackageInterface};
