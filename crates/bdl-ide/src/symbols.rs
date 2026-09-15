//! Symbols: the outline of the project or of one document, from the
//! model, not from syntax nodes.

use crate::hover::{hover, EntityStatus};
use bdl_ide_db::{AnalysisSnapshot, DocumentId, EntityKind, EntityRef, ProjectionAnchor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSymbol {
    pub entity: EntityRef,
    pub kind: EntityKind,
    pub name: String,
    /// The signature or representation, one line.
    pub detail: String,
    pub status: EntityStatus,
    /// Where it is, on every surface (text anchors of `Declaration` and
    /// `Name`, the canvas node).
    pub anchors: Vec<ProjectionAnchor>,
}

/// Every concept, mapping, output, clock and device of the snapshot, in
/// `EntityRef` order (concepts, mappings, clocks, outputs, devices).
pub fn symbols(snapshot: &AnalysisSnapshot) -> Vec<SemanticSymbol> {
    snapshot
        .index()
        .entities()
        .filter_map(|(e, _)| symbol(snapshot, e))
        .collect()
}

/// The symbols declared in one document, in document order.
pub fn document_symbols(snapshot: &AnalysisSnapshot, document: DocumentId) -> Vec<SemanticSymbol> {
    let Some(doc) = snapshot.document(document) else {
        return Vec::new();
    };
    doc.declared
        .iter()
        .filter_map(|e| symbol(snapshot, *e))
        .map(|mut s| {
            s.anchors.retain(|a| a.document() == Some(document));
            s
        })
        .collect()
}

fn symbol(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Option<SemanticSymbol> {
    let h = hover(snapshot, entity)?;
    let anchors = snapshot
        .projections()
        .anchors_of(entity)
        .iter()
        .filter(|a| {
            matches!(
                a.role,
                bdl_ide_db::EntityRole::Declaration | bdl_ide_db::EntityRole::Name
            )
        })
        .cloned()
        .collect();
    Some(SemanticSymbol {
        entity,
        kind: h.kind,
        name: h.title,
        detail: h.signature.unwrap_or_default(),
        status: h.status,
        anchors,
    })
}
