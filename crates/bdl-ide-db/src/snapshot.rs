//! The immutable analysis snapshot: one semantic world, stamped.
//!
//! An [`AnalysisSnapshot`] is what every IDE query runs against.  It is
//! built once from a committed [`ProjectSnapshot`] and an [`OverlaySet`],
//! runs the compiler over the *effective* design (committed + overlays),
//! and carries the projection map and entity index for that world.  It is
//! never mutated: the host builds a new one when anything changes, and a
//! query that holds an `Arc<AnalysisSnapshot>` keeps seeing the world it
//! started in, whatever the host does meanwhile.

use crate::cancel::{CancellationToken, Cancelled};
use crate::entity::EntityRef;
use crate::index::EntityIndex;
use crate::overlay::{Overlay, OverlayEntry, OverlayGeneration, OverlayId, OverlayKey, OverlaySet};
use crate::projection::ProjectionMap;
use crate::stamp::SnapshotStamp;
use crate::text::{DocumentId, DocumentUri};
use crate::textual::{bind_document, TextDocumentState};
use crate::visual::visual_projection;
use crate::workspace::{compose_text, register_names, TextGround, TextWorld};
use bdl_compiler::ProjectAnalysis;
use bdl_model::surface::{Definition, ProjectSnapshot};
use bdl_model::{DeclId, Revision};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Why an overlay could not be applied.  The snapshot still exists — the
/// overlay simply had no effect — and the fault is reported with it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "fault", rename_all = "snake_case")]
pub enum OverlayFault {
    /// A definition draft for a mapping that is not in the design.
    UnknownMapping { mapping: DeclId },
}

/// One overlay as it went into a snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedOverlay {
    pub id: OverlayId,
    pub key: OverlayKey,
    /// The set generation at which this overlay's content was set.
    pub generation: OverlayGeneration,
    pub fault: Option<OverlayFault>,
}

/// The immutable semantic world at one stamp.
#[derive(Debug)]
pub struct AnalysisSnapshot {
    stamp: SnapshotStamp,
    committed: Arc<ProjectSnapshot>,
    effective: ProjectSnapshot,
    analysis: ProjectAnalysis,
    overlays: Vec<AppliedOverlay>,
    documents: BTreeMap<DocumentId, TextDocumentState>,
    uris: BTreeMap<DocumentId, DocumentUri>,
    projections: ProjectionMap,
    index: EntityIndex,
    /// Present for a text workspace: the authored system behind the flat
    /// design, and the identities as the build decided them.
    text: Option<Arc<TextWorld>>,
    /// The body declarations backing a port, when this is a component
    /// body's host: their role is the port's.
    port_backed: BTreeMap<DeclId, bdl_system::PortKind>,
}

impl AnalysisSnapshot {
    /// Compose the world of a text project (ADR-0020): the sources on disk
    /// with open buffers substituted, built and flattened.  Definition
    /// drafts apply on top, as for a flat ground.
    pub fn compose_text(
        ground: &TextGround,
        overlays: &OverlaySet,
        uris: &BTreeMap<DocumentId, DocumentUri>,
        token: &CancellationToken,
    ) -> Result<AnalysisSnapshot, Cancelled> {
        token.check()?;
        let mut buffers = BTreeMap::new();
        let mut applied = Vec::new();
        for e in overlays.iter() {
            if let Overlay::TextDocument { document, source } = &e.overlay {
                buffers.insert(*document, source.clone());
                applied.push(AppliedOverlay {
                    id: e.id,
                    key: e.overlay.key(),
                    generation: e.generation,
                    fault: None,
                });
            }
        }
        let documents: BTreeMap<String, DocumentId> = uris
            .iter()
            .filter_map(|(id, uri)| crate::workspace::file_path(uri).map(|p| (p.to_owned(), *id)))
            .collect();
        let (mut effective, world, states, mut projections, names) =
            compose_text(ground, &buffers, &documents);
        token.check()?;
        for e in overlays.iter() {
            if let Overlay::MappingDefinitionDraft { mapping, source } = &e.overlay {
                let fault = match effective.design.mappings.get_mut(mapping) {
                    Some(m) => {
                        m.definition = Some(Definition::Formula {
                            source: source.clone(),
                        });
                        None
                    }
                    None => Some(OverlayFault::UnknownMapping { mapping: *mapping }),
                };
                applied.push(AppliedOverlay {
                    id: e.id,
                    key: e.overlay.key(),
                    generation: e.generation,
                    fault,
                });
            }
        }
        applied.sort_by_key(|a| a.key);
        token.check()?;
        let analysis = bdl_compiler::analyze(&effective);
        token.check()?;
        projections.extend(visual_projection(&effective.design));
        projections.finish();
        let mut index = EntityIndex::build(&effective.design);
        register_names(&mut index, &names);
        let committed = Arc::new(ProjectSnapshot {
            revision: ground.revision,
            design: world.flattened.snapshot.design.clone(),
        });
        let uris = uris
            .iter()
            .filter(|(id, _)| states.contains_key(id))
            .map(|(id, uri)| (*id, uri.clone()))
            .collect();
        Ok(AnalysisSnapshot {
            stamp: SnapshotStamp::new(ground.revision, overlays.generation()),
            committed,
            effective,
            analysis,
            overlays: applied,
            documents: states,
            uris,
            projections,
            index,
            text: Some(Arc::new(world)),
            port_backed: BTreeMap::new(),
        })
    }

    /// The same snapshot over a component body whose `ports` are backed by
    /// these declarations.
    pub fn with_port_backed(mut self, ports: BTreeMap<DeclId, bdl_system::PortKind>) -> Self {
        self.port_backed = ports;
        self
    }

    /// The port a declaration backs, if any: a component body's port
    /// (this host's) or, in a text workspace, an instance's port in the
    /// flattened design.
    pub fn port_of(&self, decl: DeclId) -> Option<bdl_system::PortKind> {
        if let Some(k) = self.port_backed.get(&decl) {
            return Some(*k);
        }
        let world = self.text.as_deref()?;
        let port = world.flattened.origins.ports.get(&decl)?;
        let instance = world.system.instances.get(&port.instance)?;
        let component = world.system.components.get(&instance.component)?;
        component.interface.ports.get(&port.port).map(|p| p.kind)
    }

    /// The authored system and its flattening, for a text workspace.
    pub fn text(&self) -> Option<&TextWorld> {
        self.text.as_deref()
    }

    /// Compose and analyse.  Pure apart from the token, which is polled
    /// between the phases (overlay application, compiler, indexing) so a
    /// superseded snapshot stops early.
    pub fn compose(
        committed: Arc<ProjectSnapshot>,
        overlays: &OverlaySet,
        uris: &BTreeMap<DocumentId, DocumentUri>,
        token: &CancellationToken,
    ) -> Result<AnalysisSnapshot, Cancelled> {
        token.check()?;
        let mut design = committed.design.clone();
        let mut applied = Vec::new();
        let mut documents = BTreeMap::new();
        let mut projections = ProjectionMap::default();

        // Text documents first: they may declare the concepts and mappings
        // a definition draft then targets.
        for e in overlays.iter() {
            if let Overlay::TextDocument { document, source } = &e.overlay {
                let (state, anchors) = bind_document(&mut design, *document, source);
                projections.extend(anchors);
                documents.insert(*document, state);
                applied.push(AppliedOverlay {
                    id: e.id,
                    key: e.overlay.key(),
                    generation: e.generation,
                    fault: None,
                });
            }
        }
        token.check()?;
        for e in overlays.iter() {
            if let Overlay::MappingDefinitionDraft { mapping, source } = &e.overlay {
                let fault = match design.mappings.get_mut(mapping) {
                    Some(m) => {
                        m.definition = Some(Definition::Formula {
                            source: source.clone(),
                        });
                        None
                    }
                    None => Some(OverlayFault::UnknownMapping { mapping: *mapping }),
                };
                applied.push(AppliedOverlay {
                    id: e.id,
                    key: e.overlay.key(),
                    generation: e.generation,
                    fault,
                });
            }
        }
        applied.sort_by_key(|a| a.key);

        let effective = ProjectSnapshot {
            revision: committed.revision,
            design,
        };
        token.check()?;
        let analysis = bdl_compiler::analyze(&effective);
        token.check()?;
        projections.extend(visual_projection(&effective.design));
        projections.finish();
        let index = EntityIndex::build(&effective.design);
        let uris = uris
            .iter()
            .filter(|(id, _)| documents.contains_key(id))
            .map(|(id, uri)| (*id, uri.clone()))
            .collect();
        Ok(AnalysisSnapshot {
            stamp: SnapshotStamp::new(committed.revision, overlays.generation()),
            committed,
            effective,
            analysis,
            overlays: applied,
            documents,
            uris,
            projections,
            index,
            text: None,
            port_backed: BTreeMap::new(),
        })
    }

    /// The world this snapshot describes.
    pub fn stamp(&self) -> SnapshotStamp {
        self.stamp
    }
    pub fn revision(&self) -> Revision {
        self.stamp.revision
    }
    pub fn generation(&self) -> OverlayGeneration {
        self.stamp.generation
    }

    /// The committed project, untouched by any overlay.
    pub fn committed(&self) -> &ProjectSnapshot {
        &self.committed
    }

    /// The project as analysed: committed plus overlays.  Same revision as
    /// the committed snapshot — overlays are not revisions.
    pub fn effective(&self) -> &ProjectSnapshot {
        &self.effective
    }

    /// The compiler's verdict on the effective project.
    pub fn analysis(&self) -> &ProjectAnalysis {
        &self.analysis
    }

    pub fn overlays(&self) -> &[AppliedOverlay] {
        &self.overlays
    }

    pub fn has_overlays(&self) -> bool {
        !self.overlays.is_empty()
    }

    pub fn overlay(&self, key: OverlayKey) -> Option<&AppliedOverlay> {
        self.overlays.iter().find(|o| o.key == key)
    }

    /// Whether the mapping's definition in this world comes from a draft.
    pub fn is_drafted(&self, mapping: DeclId) -> bool {
        self.overlay(OverlayKey::MappingDefinition { mapping })
            .is_some_and(|o| o.fault.is_none())
    }

    pub fn projections(&self) -> &ProjectionMap {
        &self.projections
    }

    pub fn index(&self) -> &EntityIndex {
        &self.index
    }

    pub fn document(&self, id: DocumentId) -> Option<&TextDocumentState> {
        self.documents.get(&id)
    }

    pub fn documents(&self) -> impl Iterator<Item = &TextDocumentState> {
        self.documents.values()
    }

    pub fn document_uri(&self, id: DocumentId) -> Option<&DocumentUri> {
        self.uris.get(&id)
    }

    pub fn document_by_uri(&self, uri: &DocumentUri) -> Option<DocumentId> {
        self.uris.iter().find(|(_, u)| *u == uri).map(|(id, _)| *id)
    }

    /// The document that declares an entity, if any text does.
    pub fn declaring_document(&self, entity: EntityRef) -> Option<DocumentId> {
        self.documents
            .values()
            .find(|d| d.declares(entity))
            .map(|d| d.document)
    }

    /// The display name of an entity in this world.
    pub fn name_of(&self, entity: EntityRef) -> Option<&str> {
        self.index.name(entity)
    }

    /// True when the entity exists in this world.
    pub fn exists(&self, entity: EntityRef) -> bool {
        self.index.exists(entity)
    }
}

/// Re-exported for adapters that keep the raw entry around.
pub type OverlayEntryRef<'a> = &'a OverlayEntry;
