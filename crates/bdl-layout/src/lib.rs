//! The layout service (ADR-0023 §7).
//!
//! A project may have sources and identities but no position for some or
//! all of its entities: a hand-written project, an item added in the Code
//! view or by an external editor.  [`place_missing`] places exactly the
//! entities that have no position — deterministically, near what they read
//! and produce, without overlapping what is already placed — and never
//! moves an entity that has one.  Its input is the semantic graph and the
//! existing layout; its output is the layout with the gaps filled and the
//! list of what was placed.  It reads no semantics into geometry and writes
//! no geometry into semantics (ADR-0003).
//!
//! Whole-graph relayout is not this service: it happens only on explicit
//! request and is not implemented here.

#![forbid(unsafe_code)]

use bdl_model::layout::{Layout, Point};
use bdl_model::surface::Design;
use bdl_model::{DeclId, OutputId, SemanticId};
use bdl_system::{BehaviorSystem, BindingEnd};
use std::collections::BTreeMap;

/// Node metrics, as Studio draws them (`NodeMetrics` in
/// `apps/studio/lib/ui/canvas/canvas_geometry.dart`).  The service places
/// rectangles of these sizes; a canvas that draws differently still gets
/// non-overlapping, left-to-right positions.
pub mod metrics {
    pub const CONCEPT_WIDTH: f64 = 168.0;
    pub const CONCEPT_HEIGHT: f64 = 26.0;
    pub const MAPPING_WIDTH: f64 = 200.0;
    pub const INSTANCE_WIDTH: f64 = 208.0;
    pub const HEADER_HEIGHT: f64 = 26.0;
    pub const ROW_HEIGHT: f64 = 22.0;
    pub const BODY_HEIGHT: f64 = 22.0;
    /// The column of each kind of node: concepts read into relationships,
    /// relationships (and instances) drive sinks.
    pub const COLUMN_GAP: f64 = 320.0;
    pub const ORIGIN_X: f64 = 48.0;
    pub const ORIGIN_Y: f64 = 48.0;
    /// Space kept between a placed node and anything else.
    pub const GAP: f64 = 16.0;
    /// The step a candidate position moves down by while it overlaps.
    pub const STEP: f64 = 8.0;
}

/// One node the service placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    /// The component whose body canvas the node is on; `None` for the
    /// system canvas.
    pub component: Option<u64>,
    pub node: Node,
    pub at: Point,
}

/// A canvas node the layout keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node {
    Concept(SemanticId),
    Mapping(DeclId),
    Output(OutputId),
    /// A component instance, by raw id (the system canvas only).
    Instance(u64),
}

/// The layout with every gap filled, and what filled it.
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub layout: Layout,
    pub placed: Vec<Placed>,
}

impl Placement {
    /// Whether the service changed anything.
    pub fn is_empty(&self) -> bool {
        self.placed.is_empty()
    }
}

/// Place every entity of `system` that `layout` has no position for: the
/// system canvas (concepts, relationships, instances, sinks) and each
/// component's body canvas.  Positioned entities, group boxes and
/// viewports are untouched; positions of entities that no longer exist
/// are kept (they may come back through undo).
pub fn place_missing(system: &BehaviorSystem, layout: &Layout) -> Placement {
    let mut out = layout.clone();
    let mut placed = Vec::new();
    place_canvas(&system.base, Some(system), &mut out, None, &mut placed);
    for (cid, component) in &system.components {
        let body = out.components.entry(cid.raw()).or_default();
        place_canvas(&component.body, None, body, Some(cid.raw()), &mut placed);
        if body == &Layout::default() {
            out.components.remove(&cid.raw());
        }
    }
    Placement {
        layout: out,
        placed,
    }
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn center_y(&self) -> f64 {
        self.y + self.h / 2.0
    }
    fn overlaps(&self, other: &Rect) -> bool {
        let g = metrics::GAP;
        self.x < other.x + other.w + g
            && other.x < self.x + self.w + g
            && self.y < other.y + other.h + g
            && other.y < self.y + self.h + g
    }
}

/// The nodes of one canvas with their sizes, in the order they are placed.
struct Canvas<'a> {
    design: &'a Design,
    system: Option<&'a BehaviorSystem>,
    /// Rectangles of everything positioned so far, by node.
    rects: BTreeMap<Node, Rect>,
}

impl Canvas<'_> {
    fn size(&self, node: Node) -> (f64, f64) {
        use metrics::*;
        match node {
            Node::Concept(_) => (CONCEPT_WIDTH, CONCEPT_HEIGHT),
            Node::Mapping(d) => {
                let inputs = self
                    .design
                    .mappings
                    .get(&d)
                    .map_or(0, |m| m.signature.inputs.len());
                let rows = inputs.max(1) as f64;
                (
                    MAPPING_WIDTH,
                    HEADER_HEIGHT + rows * ROW_HEIGHT + BODY_HEIGHT,
                )
            }
            Node::Output(_) => (CONCEPT_WIDTH, HEADER_HEIGHT + ROW_HEIGHT),
            Node::Instance(raw) => {
                let ports = self
                    .system
                    .and_then(|s| {
                        let inst = s.instances.values().find(|i| i.id.raw() == raw)?;
                        s.components.get(&inst.component)
                    })
                    .map_or(0, |c| c.interface.ports.len());
                let rows = ports.max(1) as f64;
                (
                    INSTANCE_WIDTH,
                    HEADER_HEIGHT + rows * ROW_HEIGHT + BODY_HEIGHT,
                )
            }
        }
    }

    fn column(node: Node) -> f64 {
        use metrics::*;
        ORIGIN_X
            + COLUMN_GAP
                * match node {
                    Node::Concept(_) => 0.0,
                    Node::Mapping(_) | Node::Instance(_) => 1.0,
                    Node::Output(_) => 2.0,
                }
    }

    /// The nodes this one is drawn next to: what it reads and what reads
    /// it.  Only positioned neighbours count; the mean of their centres is
    /// where the node wants to be.
    fn neighbours(&self, node: Node) -> Vec<Node> {
        let d = self.design;
        match node {
            Node::Concept(c) => d
                .mappings
                .values()
                .filter(|m| m.signature.inputs.contains(&c) || m.signature.output == c)
                .map(|m| Node::Mapping(m.id))
                .collect(),
            Node::Mapping(id) => {
                let Some(m) = d.mappings.get(&id) else {
                    return Vec::new();
                };
                let mut out: Vec<Node> = m
                    .signature
                    .inputs
                    .iter()
                    .map(|c| Node::Concept(*c))
                    .collect();
                if out.is_empty() {
                    // a source reads nothing: it sits beside what it produces
                    out.push(Node::Concept(m.signature.output));
                }
                if let Some(o) = m.drives {
                    out.push(Node::Output(o));
                }
                if let Some(s) = self.system {
                    for b in s.bindings.values() {
                        for end in [b.source, b.destination] {
                            match end {
                                BindingEnd::Base { decl } if decl == id => {
                                    let other = if b.source == end {
                                        b.destination
                                    } else {
                                        b.source
                                    };
                                    if let BindingEnd::Port(p) = other {
                                        out.push(Node::Instance(p.instance.raw()));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                out
            }
            Node::Output(o) => d
                .mappings
                .values()
                .filter(|m| m.drives == Some(o))
                .map(|m| Node::Mapping(m.id))
                .collect(),
            Node::Instance(raw) => {
                let Some(s) = self.system else {
                    return Vec::new();
                };
                let mut out = Vec::new();
                for b in s.bindings.values() {
                    let ends = [b.source, b.destination];
                    let mine = ends
                        .iter()
                        .any(|e| matches!(e, BindingEnd::Port(p) if p.instance.raw() == raw));
                    if !mine {
                        continue;
                    }
                    for e in ends {
                        match e {
                            BindingEnd::Base { decl } => out.push(Node::Mapping(decl)),
                            BindingEnd::Port(p) if p.instance.raw() != raw => {
                                out.push(Node::Instance(p.instance.raw()))
                            }
                            BindingEnd::Port(_) => {}
                        }
                    }
                }
                out
            }
        }
    }

    /// Where `node` goes: its column; the mean centre of its positioned
    /// neighbours (else below everything in its column); then down in
    /// [`metrics::STEP`]s until it overlaps nothing.
    fn place(&mut self, node: Node) -> Point {
        let (w, h) = self.size(node);
        let x = Self::column(node);
        let centres: Vec<f64> = self
            .neighbours(node)
            .into_iter()
            .filter_map(|n| self.rects.get(&n).map(Rect::center_y))
            .collect();
        let mut y = if centres.is_empty() {
            self.rects
                .values()
                .filter(|r| r.x == x)
                .map(|r| r.y + r.h + metrics::GAP)
                .fold(metrics::ORIGIN_Y, f64::max)
        } else {
            (centres.iter().sum::<f64>() / centres.len() as f64 - h / 2.0).max(metrics::ORIGIN_Y)
        };
        y = (y / metrics::STEP).round() * metrics::STEP;
        loop {
            let candidate = Rect { x, y, w, h };
            if !self.rects.values().any(|r| r.overlaps(&candidate)) {
                self.rects.insert(node, candidate);
                return Point { x, y };
            }
            y += metrics::STEP;
        }
    }
}

fn place_canvas(
    design: &Design,
    system: Option<&BehaviorSystem>,
    layout: &mut Layout,
    component: Option<u64>,
    placed: &mut Vec<Placed>,
) {
    let mut canvas = Canvas {
        design,
        system,
        rects: BTreeMap::new(),
    };
    // What is already positioned, at its drawn size.
    let known: Vec<(Node, Point)> = layout
        .concepts
        .iter()
        .map(|(id, p)| (Node::Concept(*id), *p))
        .chain(
            layout
                .mappings
                .iter()
                .map(|(id, p)| (Node::Mapping(*id), *p)),
        )
        .chain(layout.outputs.iter().map(|(id, p)| (Node::Output(*id), *p)))
        .chain(
            layout
                .instances
                .iter()
                .map(|(id, p)| (Node::Instance(*id), *p)),
        )
        .collect();
    for (node, p) in known {
        let (w, h) = canvas.size(node);
        canvas.rects.insert(
            node,
            Rect {
                x: p.x,
                y: p.y,
                w,
                h,
            },
        );
    }
    // Collapsed group boxes take space too.
    for (i, g) in layout.groups.values().filter(|g| g.collapsed).enumerate() {
        canvas.rects.insert(
            Node::Instance(u64::MAX - i as u64),
            Rect {
                x: g.x,
                y: g.y,
                w: g.width.max(metrics::INSTANCE_WIDTH),
                h: g.height.max(metrics::HEADER_HEIGHT),
            },
        );
    }

    // Concepts first (relationships are placed beside what they read),
    // then relationships, then instances beside what they are bound to,
    // then sinks beside what drives them — each in id order.
    let mut todo: Vec<Node> = Vec::new();
    todo.extend(
        design
            .concepts
            .keys()
            .filter(|id| !layout.concepts.contains_key(id))
            .map(|id| Node::Concept(*id)),
    );
    todo.extend(
        design
            .mappings
            .keys()
            .filter(|id| !layout.mappings.contains_key(id))
            .map(|id| Node::Mapping(*id)),
    );
    if let Some(s) = system {
        todo.extend(
            s.instances
                .keys()
                .map(|id| id.raw())
                .filter(|raw| !layout.instances.contains_key(raw))
                .map(Node::Instance),
        );
    }
    todo.extend(
        design
            .outputs
            .keys()
            .filter(|id| !layout.outputs.contains_key(id))
            .map(|id| Node::Output(*id)),
    );
    for node in todo {
        let at = canvas.place(node);
        match node {
            Node::Concept(id) => {
                layout.concepts.insert(id, at);
            }
            Node::Mapping(id) => {
                layout.mappings.insert(id, at);
            }
            Node::Output(id) => {
                layout.outputs.insert(id, at);
            }
            Node::Instance(raw) => {
                layout.instances.insert(raw, at);
            }
        }
        placed.push(Placed {
            component,
            node,
            at,
        });
    }
}
