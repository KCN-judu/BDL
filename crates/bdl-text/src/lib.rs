//! Textual BDL workspaces (ADR-0020).
//!
//! ```text
//!   src/**/*.bdl ──parse──▶ CST ──lower──▶ surface items ──build──▶ BehaviorSystem
//!        ▲                                     │                        │
//!        │                          .bdl/identities.json         flatten · analyze · …
//!        │                          (source key → stable id)          (existing)
//!        └──────── write-back: item-level splices of a changed model ◀─┘
//! ```
//!
//! * [`workspace`] — what a text project is on disk: the manifest kind,
//!   source discovery in sorted path order, the sidecars.
//! * [`identity`] — the source-identity table and the deterministic
//!   reconciliation of source keys against it (retained / allocated /
//!   dropped / renamed / ambiguous).
//! * [`build`] — from lowered items of every file to one authored
//!   `BehaviorSystem`, with a fault per thing the text could not mean and
//!   an anchor per entity into the source.
//! * [`print`] — the canonical spelling of one item from the model.
//! * [`splice`] — writing a changed model back as surgical edits to the
//!   files that declared each item.
//!
//! What is deliberately not here: meaning (the compiler), the IDE state
//! (`bdl-ide-db`), any transport.  Text is one surface over the one model;
//! this crate is the bridge and nothing else.

#![forbid(unsafe_code)]

pub mod build;
pub mod identity;
pub mod migrate;
pub mod names;
pub mod print;
pub mod splice;
pub mod workspace;

pub use build::{build_system, Anchor, AnchorRole, BuildResult, LoadFault, TextEntity, TextFault};
pub use identity::{IdentityTable, KeyEntry, Reconciliation, IDENTITIES_SCHEMA_VERSION};
pub use migrate::{
    make_unit_domains_explicit, migrate_legacy, MigrationReport, UnitDomainMigration,
};
pub use names::{identifier_from, is_identifier, why_not_identifier};
pub use splice::{write_back, WriteBack};
pub use workspace::{
    discover_sources, init_project, init_text_project, load_project, load_project_with,
    load_text_project, load_workspace, save_project, save_text_project, write_authoring,
    write_identities, LoadedWorkspace, SourceFile, TextError, AUTHORING_FILE, IDENTITIES_FILE,
    SOURCE_DIR,
};
