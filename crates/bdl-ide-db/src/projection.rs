//! Projections: where a semantic entity shows up on a surface.
//!
//! Textual and visual authoring are two projections of one model.  A
//! [`ProjectionAnchor`] ties `(entity, role)` to a place on one surface —
//! a byte range in a document, or a structural element of the canvas — so
//! that every semantic result (a diagnostic, a reference, a rename) can be
//! *placed* by an adapter without the adapter re-deriving what it means.
//!
//! Neither location is identity.  Deleting the anchor does not delete the
//! entity; a rename moves text but keeps the `EntityRef`.

use crate::entity::{EntityRef, EntityRole};
use crate::text::{DocumentId, TextRange};
use bdl_model::{DeclId, DeviceId, OutputId, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Which surface an anchor lives on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "projection", rename_all = "snake_case")]
pub enum ProjectionId {
    /// A textual document (one per open `.bdl` document).
    Text { document: DocumentId },
    /// The Studio canvas and inspector.  One visual projection per
    /// snapshot; identities inside it are the model's own.
    Visual,
}

/// A structural element of the visual projection.  These are the only
/// things Studio needs to highlight; they are derived from the model, not
/// from canvas geometry (positions are layout, never semantics).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "element", content = "id", rename_all = "snake_case")]
pub enum VisualElementRef {
    ConceptNode(SemanticId),
    MappingNode(DeclId),
    /// The `index`-th input socket of a mapping node.
    InputPort {
        mapping: DeclId,
        index: u16,
    },
    /// The output socket of a mapping node.
    OutputPort(DeclId),
    /// The edge from a mapping to the sink it drives.
    DriveEdge {
        mapping: DeclId,
        output: OutputId,
    },
    /// The sink itself.
    OutputTerminal(OutputId),
    /// The clock badge on a mapping node.
    ClockBadge(DeclId),
    /// The clock badge on an output terminal.
    OutputClockBadge(OutputId),
    DeviceNode(DeviceId),
    /// The inspector's definition field of a mapping.
    DefinitionField(DeclId),
    /// The inspector's signature editor of a mapping.
    SignatureField(DeclId),
    /// The inspector's representation chooser of a concept.
    RepresentationField(SemanticId),
    /// The inspector's name field of any entity.
    NameField(EntityRef),
}

/// Where on a surface an anchor is.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "location", rename_all = "snake_case")]
pub enum ProjectionLocation {
    Text(TextRange),
    Visual(VisualElementRef),
}

/// `(entity, role)` placed on one surface.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProjectionAnchor {
    pub entity: EntityRef,
    pub role: EntityRole,
    pub projection: ProjectionId,
    pub location: ProjectionLocation,
}

impl ProjectionAnchor {
    pub fn text(
        entity: EntityRef,
        role: EntityRole,
        document: DocumentId,
        range: TextRange,
    ) -> ProjectionAnchor {
        ProjectionAnchor {
            entity,
            role,
            projection: ProjectionId::Text { document },
            location: ProjectionLocation::Text(range),
        }
    }

    pub fn visual(entity: EntityRef, role: EntityRole, element: VisualElementRef) -> Self {
        ProjectionAnchor {
            entity,
            role,
            projection: ProjectionId::Visual,
            location: ProjectionLocation::Visual(element),
        }
    }

    pub fn text_range(&self) -> Option<TextRange> {
        match &self.location {
            ProjectionLocation::Text(r) => Some(*r),
            ProjectionLocation::Visual(_) => None,
        }
    }

    pub fn document(&self) -> Option<DocumentId> {
        match self.projection {
            ProjectionId::Text { document } => Some(document),
            ProjectionId::Visual => None,
        }
    }

    pub fn visual_element(&self) -> Option<VisualElementRef> {
        match &self.location {
            ProjectionLocation::Visual(v) => Some(*v),
            ProjectionLocation::Text(_) => None,
        }
    }
}

/// The index of anchors of one snapshot: `(entity, role) → anchors`, plus a
/// per-document ordering for position lookup.  Deterministic: `BTreeMap`s
/// and anchors sorted by range.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionMap {
    by_entity: BTreeMap<EntityRef, Vec<ProjectionAnchor>>,
    by_document: BTreeMap<DocumentId, Vec<ProjectionAnchor>>,
}

impl ProjectionMap {
    pub fn insert(&mut self, anchor: ProjectionAnchor) {
        if let Some(doc) = anchor.document() {
            self.by_document
                .entry(doc)
                .or_default()
                .push(anchor.clone());
        }
        self.by_entity
            .entry(anchor.entity)
            .or_default()
            .push(anchor);
    }

    pub fn extend(&mut self, other: ProjectionMap) {
        for anchors in other.by_entity.into_values() {
            for a in anchors {
                self.insert(a);
            }
        }
    }

    /// Sort every list so lookups are deterministic regardless of the
    /// order anchors were discovered in.
    pub fn finish(&mut self) {
        for v in self.by_entity.values_mut() {
            v.sort();
            v.dedup();
        }
        for v in self.by_document.values_mut() {
            v.sort_by_key(|a| (a.text_range(), a.role));
            v.dedup();
        }
    }

    /// Every anchor of an entity, on every surface.
    pub fn anchors_of(&self, entity: EntityRef) -> &[ProjectionAnchor] {
        self.by_entity.get(&entity).map_or(&[], Vec::as_slice)
    }

    /// Anchors of an entity in one role.
    pub fn anchors_for(
        &self,
        entity: EntityRef,
        role: EntityRole,
    ) -> impl Iterator<Item = &ProjectionAnchor> {
        self.anchors_of(entity)
            .iter()
            .filter(move |a| a.role == role)
    }

    /// Text anchors of an entity in one document.
    pub fn text_anchors(
        &self,
        entity: EntityRef,
        document: DocumentId,
    ) -> impl Iterator<Item = &ProjectionAnchor> {
        self.anchors_of(entity)
            .iter()
            .filter(move |a| a.document() == Some(document))
    }

    /// Every text anchor in a document, by position.
    pub fn document_anchors(&self, document: DocumentId) -> &[ProjectionAnchor] {
        self.by_document.get(&document).map_or(&[], Vec::as_slice)
    }

    /// The innermost anchor covering a byte offset in a document.
    /// "Innermost" is the shortest covering range; ties prefer the more
    /// specific role (a `Name` over a `Declaration`).
    pub fn anchor_at(&self, document: DocumentId, offset: u32) -> Option<&ProjectionAnchor> {
        self.document_anchors(document)
            .iter()
            .filter(|a| a.text_range().is_some_and(|r| r.contains(offset)))
            .min_by_key(|a| {
                let r = a.text_range().unwrap_or(TextRange::empty_at(0));
                (r.len(), role_specificity(a.role))
            })
    }

    pub fn documents(&self) -> impl Iterator<Item = DocumentId> + '_ {
        self.by_document.keys().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.by_entity.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ProjectionAnchor> {
        self.by_entity.values().flatten()
    }
}

/// Lower is more specific.  A whole-declaration anchor always loses to any
/// part of it.
fn role_specificity(role: EntityRole) -> u8 {
    match role {
        EntityRole::Name | EntityRole::Reference => 0,
        EntityRole::Input { .. } | EntityRole::Output => 1,
        EntityRole::Representation | EntityRole::Definition | EntityRole::Signature => 2,
        EntityRole::DriveEdge
        | EntityRole::ClockBinding
        | EntityRole::DeviceBinding
        | EntityRole::Requirement
        | EntityRole::Description => 3,
        EntityRole::Declaration => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn innermost_anchor_wins_at_a_position() {
        let doc = DocumentId(1);
        let m = EntityRef::Mapping(DeclId::from_raw(17));
        let c = EntityRef::Concept(SemanticId::from_raw(2));
        let mut map = ProjectionMap::default();
        map.insert(ProjectionAnchor::text(
            m,
            EntityRole::Declaration,
            doc,
            TextRange::new(0, 40),
        ));
        map.insert(ProjectionAnchor::text(
            m,
            EntityRole::Name,
            doc,
            TextRange::new(8, 17),
        ));
        map.insert(ProjectionAnchor::text(
            m,
            EntityRole::Input { index: 0 },
            doc,
            TextRange::new(20, 24),
        ));
        map.insert(ProjectionAnchor::visual(
            c,
            EntityRole::Declaration,
            VisualElementRef::ConceptNode(SemanticId::from_raw(2)),
        ));
        map.finish();
        assert_eq!(
            map.anchor_at(doc, 10).map(|a| a.role),
            Some(EntityRole::Name)
        );
        assert_eq!(
            map.anchor_at(doc, 22).map(|a| a.role),
            Some(EntityRole::Input { index: 0 })
        );
        assert_eq!(
            map.anchor_at(doc, 30).map(|a| a.role),
            Some(EntityRole::Declaration)
        );
        assert_eq!(map.anchor_at(doc, 41), None);
        assert_eq!(map.anchors_of(c).len(), 1);
        assert_eq!(map.anchors_for(m, EntityRole::Name).count(), 1);
    }
}
