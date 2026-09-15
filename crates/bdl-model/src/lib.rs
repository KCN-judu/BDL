//! Canonical BDL project model.
//!
//! This crate owns what a BDL project *is* independently of any editor:
//!
//! * [`ids`] — strongly typed stable identities (`SemanticId`, `DeclId`, …).
//!   Identity is never a display name.
//! * [`dim`] — physical dimensions as exponent vectors (the `q Dim` of the
//!   kernel).
//! * [`surface`] — the designer-level *surface model*: concepts, mapping
//!   blocks, their signatures and optional definitions.  It may be
//!   incomplete; an unresolved mapping is an ordinary mapping with
//!   `definition == None`.
//! * [`edit`] — the revisioned edit model: `apply_edit(snapshot, op)` is a
//!   pure function from one immutable [`ProjectSnapshot`] to the next.
//! * [`persist`] — crash-safe on-disk representation with `schema_version`.
//!
//! Nothing here depends on Flutter, on the compiler passes, or on hardware.
//! Everything downstream (elaboration, checking, codegen) consumes
//! snapshots produced here.

#![forbid(unsafe_code)]

pub mod dim;
pub mod edit;
pub mod ids;
pub mod layout;
pub mod persist;
pub mod surface;

pub use dim::Dim;
pub use edit::{apply_edit, Applied, EditError, EditKind, EditOp, EditOutcome, Invalidation};
pub use ids::{ClockId, DeclId, DeviceId, OutputId, Revision, SemanticId};
pub use surface::{
    Concept, Definition, Design, MappingBlock, ProjectSnapshot, Representation, Signature,
};

/// Version of the semantic project schema written by [`persist`].
pub const PROJECT_SCHEMA_VERSION: u32 = 1;
/// Version of the UI layout schema written by [`persist`].
pub const LAYOUT_SCHEMA_VERSION: u32 = 1;
