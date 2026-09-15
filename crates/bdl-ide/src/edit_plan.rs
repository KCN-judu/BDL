//! Semantic edit plans: what a rename, an action or a refactoring would
//! do, as data any client can apply.
//!
//! A plan speaks two vocabularies and nothing else: the model's own
//! [`EditOp`]s (the only way a project changes, ADR-0009) and byte-range
//! [`TextEdit`]s for text that is not yet in the model (an open document,
//! a Studio draft).  Studio applies model operations through `bdld`; the
//! LSP adapter turns text operations into a `WorkspaceEdit` and model
//! operations into edits of the documents that project them.  Neither
//! invents mutation logic of its own.

use crate::invalidation::InvalidationPreview;
use bdl_ide_db::{DocumentId, EntityRef, OverlayGeneration, SnapshotStamp, TextEdit};
use bdl_model::{DeclId, EditOp, Revision};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One step of a plan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SemanticOperation {
    /// A revisioned edit of the committed model.
    Model { edit: EditOp },
    /// Edits to an open text document (applied by the editor; the model
    /// follows when the document is committed).
    Text {
        document: DocumentId,
        edits: Vec<TextEdit>,
    },
    /// Edits to a Studio definition draft that is not committed yet.
    DraftText {
        mapping: DeclId,
        edits: Vec<TextEdit>,
    },
}

/// What must still hold for the plan to be applied as computed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "precondition", rename_all = "snake_case")]
pub enum Precondition {
    /// The committed project is still at this revision.
    Revision { revision: Revision },
    /// The overlays are still at this generation (text ranges depend on
    /// it).
    Generation { generation: OverlayGeneration },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticEditPlan {
    pub title: String,
    pub operations: Vec<SemanticOperation>,
    pub affected_entities: BTreeSet<EntityRef>,
    pub invalidation: InvalidationPreview,
    pub preconditions: Vec<Precondition>,
}

impl SemanticEditPlan {
    pub fn new(title: impl Into<String>, stamp: SnapshotStamp) -> SemanticEditPlan {
        SemanticEditPlan {
            title: title.into(),
            operations: Vec::new(),
            affected_entities: BTreeSet::new(),
            invalidation: InvalidationPreview::none(),
            preconditions: vec![
                Precondition::Revision {
                    revision: stamp.revision,
                },
                Precondition::Generation {
                    generation: stamp.generation,
                },
            ],
        }
    }

    pub fn model_edits(&self) -> impl Iterator<Item = &EditOp> {
        self.operations.iter().filter_map(|o| match o {
            SemanticOperation::Model { edit } => Some(edit),
            _ => None,
        })
    }

    pub fn text_edits(&self, document: DocumentId) -> Vec<&TextEdit> {
        self.operations
            .iter()
            .filter_map(|o| match o {
                SemanticOperation::Text { document: d, edits } if *d == document => Some(edits),
                _ => None,
            })
            .flatten()
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Whether the plan still applies to the world at `stamp`.
    pub fn holds_at(&self, stamp: SnapshotStamp) -> bool {
        self.preconditions.iter().all(|p| match p {
            Precondition::Revision { revision } => *revision == stamp.revision,
            Precondition::Generation { generation } => *generation == stamp.generation,
        })
    }
}
