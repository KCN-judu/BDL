//! The Studio formula editor's query: the verdict on one mapping whose
//! definition is a draft overlay.
//!
//! ```text
//!   host.set_definition_draft(m, text)      (overlay)
//!   let snap = host.snapshot();             (AnalysisSnapshot)
//!   draft_verdict(&snap, m)                 (this)
//! ```
//!
//! The verdict is the ordinary analysis of the mapping in the effective
//! world — the same ladder, the same diagnostics a committed definition
//! gets — tagged with the stamp and the draft's own generation so the
//! client can drop anything a newer keystroke superseded.

use crate::diagnostics::{lift_for_mapping, SemanticDiagnostic};
use crate::QueryError;
use bdl_compiler::{MappingAnalysis, MappingStatus};
use bdl_ide_db::{AnalysisSnapshot, EntityRef, OverlayGeneration, OverlayKey, SnapshotStamp};
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DraftVerdict {
    /// The world the verdict describes.
    pub stamp: SnapshotStamp,
    pub mapping: DeclId,
    /// The generation at which the draft's text was set (from the overlay
    /// entry), for clients that key on the draft rather than the world.
    pub draft_generation: OverlayGeneration,
    pub status: MappingStatus,
    /// False when the draft did not parse.
    pub parse_ok: bool,
    /// The same verdict a committed definition gets; spans index the
    /// draft source.
    pub analysis: MappingAnalysis,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

/// The verdict on `mapping`'s draft in this snapshot.  `NotApplicable`
/// when the mapping has no draft overlay here; `UnknownEntity` when the
/// mapping does not exist (the overlay is faulted).
pub fn draft_verdict(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
) -> Result<DraftVerdict, QueryError> {
    let key = OverlayKey::MappingDefinition { mapping };
    let Some(applied) = snapshot.overlay(key) else {
        return Err(QueryError::NotApplicable {
            reason: format!("mapping {mapping} has no definition draft in this snapshot"),
        });
    };
    if applied.fault.is_some() {
        return Err(QueryError::UnknownEntity {
            entity: EntityRef::Mapping(mapping),
        });
    }
    let analysis =
        snapshot
            .analysis()
            .mappings
            .get(&mapping)
            .cloned()
            .ok_or(QueryError::UnknownEntity {
                entity: EntityRef::Mapping(mapping),
            })?;
    let parse_ok = !analysis
        .diagnostics
        .iter()
        .any(|d| d.code.as_str().starts_with("formula.parse."));
    Ok(DraftVerdict {
        stamp: snapshot.stamp(),
        mapping,
        draft_generation: applied.generation,
        status: analysis.status,
        parse_ok,
        diagnostics: lift_for_mapping(snapshot, mapping),
        analysis,
    })
}
