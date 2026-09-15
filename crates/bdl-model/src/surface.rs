//! The surface model: what a designer authors.
//!
//! This is the *canonical* project content.  It is deliberately a surface
//! representation — mapping blocks with signatures over concepts and an
//! optional definition — not the kernel: elaboration (a later crate) turns
//! it into the Design IR and the Reactive Core IR.  The model may be
//! incomplete at any time; incompleteness is a legal state, not an error.
//!
//! Display names live here because designers need them, but nothing refers
//! to anything by name: all references are by stable id.

use crate::dim::Dim;
use crate::ids::{DeclId, IdAllocator, Revision, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An immutable view of the project at one revision.  Edits produce a new
/// snapshot (see [`crate::edit`]); analyses take snapshots by value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub revision: Revision,
    pub design: Design,
}

impl ProjectSnapshot {
    pub fn new(design: Design) -> Self {
        ProjectSnapshot {
            revision: Revision::INITIAL,
            design,
        }
    }
}

/// The persisted semantic content of a project.  Ordered maps keep every
/// traversal deterministic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Design {
    pub name: String,
    pub concepts: BTreeMap<SemanticId, Concept>,
    pub mappings: BTreeMap<DeclId, MappingBlock>,
    pub ids: IdAllocator,
}

impl Design {
    pub fn empty(name: impl Into<String>) -> Self {
        Design {
            name: name.into(),
            concepts: BTreeMap::new(),
            mappings: BTreeMap::new(),
            ids: IdAllocator::default(),
        }
    }

    pub fn concept(&self, id: SemanticId) -> Option<&Concept> {
        self.concepts.get(&id)
    }

    pub fn mapping(&self, id: DeclId) -> Option<&MappingBlock> {
        self.mappings.get(&id)
    }

    /// Mappings whose signature mentions `concept` (as input or output).
    pub fn mappings_using(&self, concept: SemanticId) -> impl Iterator<Item = &MappingBlock> {
        self.mappings
            .values()
            .filter(move |m| m.signature.mentions(concept))
    }
}

/// A semantic property: `Tilt`, `Brightness`, `Held`.  Identity is the
/// `SemanticId`; the name is mutable documentation.  The representation is a
/// write-once binding (the kernel's `Θ s = some R`); a concept without one is
/// a legal, still-open declaration of intent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    pub id: SemanticId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representation: Option<Representation>,
}

/// What a concept is represented by.  Must be a semantic-free *data* type
/// (kernel `ConceptEnv.WF`); the enum makes that unrepresentable otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Representation {
    /// A physical quantity of dimension `dim` (`q dim`).  Dimensionless
    /// levels such as `Brightness ∈ [0,1]` use `Dim::ZERO`.
    Quantity { dim: Dim },
    /// A truth value (`bool`).
    Boolean,
    /// A count (`nat`).
    Count,
}

/// A mapping block: a declaration with a signature over concepts and an
/// optional definition.  `definition == None` is the paper's "hole" — an
/// unresolved declaration, distinguished by nothing else.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingBlock {
    pub id: DeclId,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub signature: Signature,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<Definition>,
}

impl MappingBlock {
    pub fn is_unresolved(&self) -> bool {
        self.definition.is_none()
    }
}

/// `(A₁, …, Aₙ) -> B` over concepts.  A nullary signature `() -> B` is a
/// value declaration of type `B`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub inputs: Vec<SemanticId>,
    pub output: SemanticId,
}

impl Signature {
    pub fn mentions(&self, concept: SemanticId) -> bool {
        self.output == concept || self.inputs.contains(&concept)
    }
}

/// A definition attached to a mapping block.  All forms elaborate to the same
/// kernel realization; the surface keeps the authoring form so it can be
/// re-edited in the way it was written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Definition {
    /// A formula in the surface expression language over the signature's
    /// input names.  Parsed and checked by the compiler, never by the editor.
    Formula { source: String },
}
