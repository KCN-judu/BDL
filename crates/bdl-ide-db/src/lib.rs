//! IDE ground state for BDL.
//!
//! ```text
//!   committed ProjectSnapshot  +  OverlaySet   ──▶  IdeHost  ──snapshot()──▶  AnalysisSnapshot
//!                                                                              (immutable, stamped)
//! ```
//!
//! This crate owns the *state* an IDE service needs and nothing of what it
//! *means*:
//!
//! * [`entity`] — [`EntityRef`] / [`EntityRole`]: stable identity and the
//!   aspect of an entity a location is about.  Never a name, never a
//!   position.
//! * [`overlay`] — uncommitted authoring state (a Studio formula draft, an
//!   unsaved textual document), one mechanism for every surface, with a
//!   monotonic generation.
//! * [`text`] / [`projection`] / [`textual`] / [`visual`] — documents, byte
//!   ranges, and the anchors that place `(entity, role)` on a text or the
//!   canvas.  Textual and visual authoring are projections of one model.
//! * [`index`] — names and references by identity.
//! * [`snapshot`] — the immutable [`AnalysisSnapshot`]: committed +
//!   overlays composed, analysed by `bdl-compiler`, indexed and projected.
//! * [`host`] — the one mutable place ([`IdeHost`]), from which snapshots
//!   are taken.
//! * [`stamp`] / [`cancel`] — revision/generation stamps that make stale
//!   results recognisable, and co-operative cancellation.
//!
//! Semantic queries (diagnostics, hover, completion, references, rename,
//! actions) live in `bdl-ide`; transports (`bdld`, `bdl-lsp`) adapt them.
//! See `docs/IDE_SERVICE_ARCHITECTURE.md`.

#![forbid(unsafe_code)]

pub mod cancel;
pub mod entity;
pub mod host;
pub mod index;
pub mod overlay;
pub mod projection;
pub mod snapshot;
pub mod stamp;
pub mod text;
pub mod textual;
pub mod visual;

pub use cancel::{CancelScope, CancellationToken, Cancelled, RequestId, RequestTracker};
pub use entity::{EntityKind, EntityRef, EntityRole};
pub use host::{CommitEffect, IdeConfig, IdeHost};
pub use index::{EntityIndex, SemanticReference};
pub use overlay::{Overlay, OverlayEntry, OverlayGeneration, OverlayId, OverlayKey, OverlaySet};
pub use projection::{
    ProjectionAnchor, ProjectionId, ProjectionLocation, ProjectionMap, VisualElementRef,
};
pub use snapshot::{AnalysisSnapshot, AppliedOverlay, OverlayFault};
pub use stamp::{ResultGate, SnapshotStamp};
pub use text::{DocumentId, DocumentKind, DocumentUri, EditConflict, TextEdit, TextRange};
pub use textual::{BindingFault, TextDocumentState};
