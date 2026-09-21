//! UI layout — deliberately *not* semantics.
//!
//! Canvas positions are persisted in their own file (`ui/layout.json`) keyed
//! by stable entity id, so that moving a node never touches the design and a
//! design can be opened by a tool that has no canvas at all.

use crate::ids::{ConceptId, DeclId, OutputId};
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
    pub concepts: BTreeMap<ConceptId, Point>,
    #[serde(default)]
    pub mappings: BTreeMap<DeclId, Point>,
    /// Physical outputs are canvas nodes too (sinks at the right edge).
    #[serde(default)]
    pub outputs: BTreeMap<OutputId, Point>,
    /// Component-instance nodes of a system canvas, by raw instance id
    /// (the id sort belongs to `bdl-system`; layout only stores the number).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub instances: BTreeMap<u64, Point>,
    /// Behaviour groups, by raw group id: where the collapsed box stands,
    /// how large it is, and whether it is collapsed.  Membership is the
    /// system's (concept-free authoring metadata); this is the picture.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<u64, GroupBox>,
    /// The canvas of each component's body (component-local ids), by raw
    /// component id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub components: BTreeMap<u64, Layout>,
    /// Where the designer left this canvas (pan and zoom).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct GroupBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub collapsed: bool,
}
