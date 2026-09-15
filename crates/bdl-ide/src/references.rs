//! References and navigation, by identity.
//!
//! `references(Concept(s))` answers from the entity index — the mappings
//! whose signatures mention `s`, the sinks that accept it, the formula
//! bodies that read it — and then *places* each reference through the
//! projection map: a byte range in the document that declares the
//! referrer, a port or a terminal on the canvas.  A name that merely
//! looks alike (in a comment, in another entity's name) is not a
//! reference, because nothing here compares strings.

use bdl_ide_db::{
    AnalysisSnapshot, DocumentId, EntityRef, EntityRole, ProjectionAnchor, SemanticReference,
    VisualElementRef,
};
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceResult {
    pub entity: EntityRef,
    /// The semantic references, before any placement.
    pub references: Vec<SemanticReference>,
    /// Where the entity is declared, on every surface.
    pub declaration: Vec<ProjectionAnchor>,
    /// Every placed reference site (text ranges and visual elements).
    pub anchors: Vec<ProjectionAnchor>,
}

impl ReferenceResult {
    pub fn text_anchors(&self, document: DocumentId) -> impl Iterator<Item = &ProjectionAnchor> {
        self.anchors
            .iter()
            .filter(move |a| a.document() == Some(document))
    }
    pub fn visual_elements(&self) -> impl Iterator<Item = VisualElementRef> + '_ {
        self.anchors.iter().filter_map(|a| a.visual_element())
    }
}

/// Every reference to `entity`, by identity, placed on every surface.
/// An unknown entity yields an empty result, never an error.
pub fn references(snapshot: &AnalysisSnapshot, entity: EntityRef) -> ReferenceResult {
    let map = snapshot.projections();
    let references = snapshot.index().references_to(entity).to_vec();
    let declaration: Vec<ProjectionAnchor> = map
        .anchors_of(entity)
        .iter()
        .filter(|a| matches!(a.role, EntityRole::Name | EntityRole::Declaration))
        .cloned()
        .collect();
    let mut anchors: Vec<ProjectionAnchor> = Vec::new();
    // Text and visual sites that name the entity directly.
    for a in map.anchors_of(entity) {
        if a.role == EntityRole::Reference {
            anchors.push(a.clone());
        }
    }
    // Sites of the referrers in the referring role (a mapping's input
    // port, a sink's terminal), for surfaces that have no name site.
    for r in &references {
        for a in map.anchors_for(r.referrer, r.role) {
            if a.visual_element().is_some() && !anchors.contains(a) {
                anchors.push(a.clone());
            }
        }
    }
    anchors.sort();
    anchors.dedup();
    ReferenceResult {
        entity,
        references,
        declaration,
        anchors,
    }
}

/// Where an entity is declared: its name site in text, its node on the
/// canvas.
pub fn definition_of(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Vec<ProjectionAnchor> {
    references(snapshot, entity).declaration
}

/// The entity under a byte offset in a document, with the role of the
/// site (`Name` at a declaration, `Reference` at a use, …).
pub fn entity_at(
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    offset: u32,
) -> Option<(EntityRef, EntityRole)> {
    snapshot
        .projections()
        .anchor_at(document, offset)
        .map(|a| (a.entity, a.role))
}

/// The concept a name inside a mapping's formula refers to, by
/// body-relative offset (the Studio definition field).
pub fn entity_at_formula(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    offset: u32,
) -> Option<EntityRef> {
    snapshot
        .index()
        .formula_name_at(mapping, offset)
        .map(|(_, c)| EntityRef::Concept(c))
}
