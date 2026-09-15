//! UI layout — deliberately *not* semantics.
//!
//! Canvas positions are persisted in their own file (`ui/layout.json`) keyed
//! by stable entity id, so that moving a node never touches the design and a
//! design can be opened by a tool that has no canvas at all.

use crate::ids::{DeclId, OutputId, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Layout {
    #[serde(default)]
    pub concepts: BTreeMap<SemanticId, Point>,
    #[serde(default)]
    pub mappings: BTreeMap<DeclId, Point>,
    /// Physical outputs are canvas nodes too (sinks at the right edge).
    #[serde(default)]
    pub outputs: BTreeMap<OutputId, Point>,
}
