//! Overlays: uncommitted authoring state layered over the committed
//! project.
//!
//! ```text
//! committed ProjectSnapshot  +  OverlaySet  =  what analysis sees
//! ```
//!
//! One mechanism serves every surface.  A Studio formula draft and an
//! unsaved textual document are the same class of thing: text the designer
//! is still typing, which the compiler must judge *now* and which must not
//! touch the project until the designer commits.  Overlays never create a
//! project revision; the committed snapshot is untouched while they exist.
//!
//! Every change to the set bumps a monotonic [`OverlayGeneration`], so a
//! result computed against an older set can be recognised as stale.

use crate::text::DocumentId;
use bdl_model::DeclId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Identity of one overlay for the life of a host.  Never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OverlayId(pub u64);

impl fmt::Display for OverlayId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ov#{}", self.0)
    }
}

/// Monotonic counter over *every* change to a host's overlay set (an
/// insert, an update, a removal).  Two snapshots with equal committed
/// revision and equal generation describe the same world.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct OverlayGeneration(pub u64);

impl OverlayGeneration {
    pub const INITIAL: OverlayGeneration = OverlayGeneration(0);
    #[must_use]
    pub const fn next(self) -> OverlayGeneration {
        OverlayGeneration(self.0 + 1)
    }
}

impl fmt::Display for OverlayGeneration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g{}", self.0)
    }
}

/// What an overlay stands in for.  There is at most one overlay per key:
/// a second draft for the same mapping *replaces* the first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "key", rename_all = "snake_case")]
pub enum OverlayKey {
    /// The definition of one mapping (the Studio formula editor).
    MappingDefinition { mapping: DeclId },
    /// The content of one textual document (an editor buffer).
    TextDocument { document: DocumentId },
}

/// The uncommitted content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "overlay", rename_all = "snake_case")]
pub enum Overlay {
    /// A candidate formula for `mapping`, as if it were attached.
    MappingDefinitionDraft { mapping: DeclId, source: String },
    /// The current text of a document, as if it were on disk.
    TextDocument {
        document: DocumentId,
        source: String,
    },
}

impl Overlay {
    pub fn key(&self) -> OverlayKey {
        match self {
            Overlay::MappingDefinitionDraft { mapping, .. } => {
                OverlayKey::MappingDefinition { mapping: *mapping }
            }
            Overlay::TextDocument { document, .. } => OverlayKey::TextDocument {
                document: *document,
            },
        }
    }

    pub fn source(&self) -> &str {
        match self {
            Overlay::MappingDefinitionDraft { source, .. }
            | Overlay::TextDocument { source, .. } => source,
        }
    }
}

/// One overlay in the set, with the generation at which its content was
/// last set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlayEntry {
    pub id: OverlayId,
    /// The set generation at which this content was established.  A
    /// result tagged with an older generation for this key is stale.
    pub generation: OverlayGeneration,
    pub overlay: Overlay,
}

/// The overlays of one host.  Ordered by key so composition over the
/// committed snapshot is deterministic.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlaySet {
    entries: BTreeMap<OverlayKey, OverlayEntry>,
    next_id: u64,
    generation: OverlayGeneration,
}

impl OverlaySet {
    pub fn generation(&self) -> OverlayGeneration {
        self.generation
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, key: OverlayKey) -> Option<&OverlayEntry> {
        self.entries.get(&key)
    }

    pub fn iter(&self) -> impl Iterator<Item = &OverlayEntry> {
        self.entries.values()
    }

    /// Insert or replace the overlay for its key.  Returns the entry's id
    /// (kept across updates of the same key) and the new generation.
    /// Setting identical content is still a change: the generation moves,
    /// because the client asked a new question.
    pub fn upsert(&mut self, overlay: Overlay) -> (OverlayId, OverlayGeneration) {
        self.generation = self.generation.next();
        let key = overlay.key();
        let id = match self.entries.get(&key) {
            Some(e) => e.id,
            None => {
                let id = OverlayId(self.next_id);
                self.next_id += 1;
                id
            }
        };
        self.entries.insert(
            key,
            OverlayEntry {
                id,
                generation: self.generation,
                overlay,
            },
        );
        (id, self.generation)
    }

    /// Remove the overlay for a key.  Returns what was removed; a missing
    /// key is not an error and does not move the generation.
    pub fn remove(&mut self, key: OverlayKey) -> Option<OverlayEntry> {
        let removed = self.entries.remove(&key)?;
        self.generation = self.generation.next();
        Some(removed)
    }

    /// Drop every overlay whose key no longer refers to anything (a draft
    /// for a deleted mapping).  Returns how many were dropped.
    pub fn retain_keys(&mut self, mut keep: impl FnMut(OverlayKey) -> bool) -> usize {
        let before = self.entries.len();
        self.entries.retain(|k, _| keep(*k));
        let dropped = before - self.entries.len();
        if dropped > 0 {
            self.generation = self.generation.next();
        }
        dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_keeps_id_per_key_and_bumps_generation() {
        let mut set = OverlaySet::default();
        let m = DeclId::from_raw(17);
        let (id1, g1) = set.upsert(Overlay::MappingDefinitionDraft {
            mapping: m,
            source: "T".into(),
        });
        let (id2, g2) = set.upsert(Overlay::MappingDefinitionDraft {
            mapping: m,
            source: "Ti".into(),
        });
        assert_eq!(id1, id2);
        assert!(g2 > g1);
        assert_eq!(set.len(), 1);
        let (id3, g3) = set.upsert(Overlay::TextDocument {
            document: DocumentId(0),
            source: String::new(),
        });
        assert_ne!(id3, id1);
        assert!(g3 > g2);
        assert_eq!(set.len(), 2);
        assert!(set
            .remove(OverlayKey::MappingDefinition { mapping: m })
            .is_some());
        assert!(set.generation() > g3);
        let g = set.generation();
        assert!(set
            .remove(OverlayKey::MappingDefinition { mapping: m })
            .is_none());
        assert_eq!(set.generation(), g, "removing nothing is not a change");
    }
}
