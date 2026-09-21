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
//! Whole-graph arrangement is the other service here — [`arrange`]: the
//! explicit *Arrange Automatically* command, and the first opening of a
//! project with no position at all — deterministic and layout-only like
//! [`place_missing`], and unlike it free to move everything.

#![forbid(unsafe_code)]

use bdl_model::layout::{Layout, Point};
use bdl_model::surface::Design;
use bdl_model::{DeclId, OutputId};
use bdl_system::{BehaviorSystem, BindingEnd};
use std::collections::BTreeMap;

/// Node metrics, as Studio draws them (`NodeMetrics` in
/// `apps/studio/lib/ui/canvas/canvas_geometry.dart`).  The service places
/// rectangles of these sizes; a canvas that draws differently still gets
/// non-overlapping, left-to-right positions.
///
/// The nodes are the concept ladder's (ADR-0044): a **Sem block** (a
/// unit-domain declaration; `Node::Mapping`, one row with the name and its
/// concept), its **mapping block** (`Node::Definition`, the definition
/// drawn as a node to its left, with a row per Sem block it reads), sinks
/// and instances.  A concept is a template and a rule a template: neither
/// is a node.
pub mod metrics {
    /// A Sem block: the header and its concept row.
    pub const SEM_WIDTH: f64 = 168.0;
    pub const SEM_HEIGHT: f64 = 48.0;
    pub const MAPPING_WIDTH: f64 = 200.0;
    pub const INSTANCE_WIDTH: f64 = 208.0;
    pub const HEADER_HEIGHT: f64 = 26.0;
    pub const ROW_HEIGHT: f64 = 22.0;
    pub const BODY_HEIGHT: f64 = 22.0;
    /// The column of each kind of node: mapping blocks left of the Sem
    /// blocks they produce, Sem blocks (and instances) left of the sinks
    /// they drive.
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
    /// A Sem block: a unit-domain declaration (`Layout::mappings`).
    Mapping(DeclId),
    /// A Sem block's mapping block: its definition drawn as a node
    /// (`Layout::definitions`), keyed by the same declaration.
    Definition(DeclId),
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
    place_missing_with(system, layout, &References::default())
}

/// [`place_missing`] with the read edges known: a mapping block is sized
/// by what it reads and placed beside it.
pub fn place_missing_with(
    system: &BehaviorSystem,
    layout: &Layout,
    refs: &References,
) -> Placement {
    let mut out = layout.clone();
    let mut placed = Vec::new();
    let none: Vec<(DeclId, DeclId)> = Vec::new();
    place_canvas(
        &system.base,
        Some(system),
        &mut out,
        None,
        refs.edges.get(&None).unwrap_or(&none),
        &mut placed,
    );
    for (cid, component) in &system.components {
        let body = out.components.entry(cid.raw()).or_default();
        place_canvas(
            &component.body,
            None,
            body,
            Some(cid.raw()),
            refs.edges.get(&Some(cid.raw())).unwrap_or(&none),
            &mut placed,
        );
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
    /// The read edges, `referencing → referenced` (ADR-0034's
    /// `dependsOn`): what a mapping block reads.
    references: &'a [(DeclId, DeclId)],
    /// Rectangles of everything positioned so far, by node.
    rects: BTreeMap<Node, Rect>,
}

/// The Sem blocks of a design: its unit-domain declarations.  A rule
/// (an arrow-typed declaration) is a template and is not drawn.
fn is_sem(design: &Design, d: DeclId) -> bool {
    design
        .mappings
        .get(&d)
        .is_some_and(|m| m.signature.is_unit_domain())
}

/// Whether the Sem block has a mapping block: a definition of its own.
fn has_definition(design: &Design, d: DeclId) -> bool {
    design
        .mappings
        .get(&d)
        .is_some_and(|m| m.signature.is_unit_domain() && m.definition.is_some())
}

impl Canvas<'_> {
    /// The Sem blocks a mapping block reads, in id order: the referenced
    /// declarations that are Sem blocks (a referenced rule is named in the
    /// block's header and is no edge).
    fn reads(&self, d: DeclId) -> Vec<DeclId> {
        let mut out: Vec<DeclId> = self
            .references
            .iter()
            .filter(|(from, to)| *from == d && *to != d && is_sem(self.design, *to))
            .map(|(_, to)| *to)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    fn size(&self, node: Node) -> (f64, f64) {
        use metrics::*;
        match node {
            Node::Mapping(_) => (SEM_WIDTH, SEM_HEIGHT),
            Node::Definition(d) => {
                let rows = self.reads(d).len().max(1) as f64;
                (
                    MAPPING_WIDTH,
                    HEADER_HEIGHT + rows * ROW_HEIGHT + BODY_HEIGHT,
                )
            }
            Node::Output(_) => (SEM_WIDTH, HEADER_HEIGHT + ROW_HEIGHT),
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
                    Node::Definition(_) => 0.0,
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
            // a mapping block sits beside the Sem blocks it reads and the
            // one it produces
            Node::Definition(id) => {
                let mut out: Vec<Node> = self.reads(id).into_iter().map(Node::Mapping).collect();
                out.push(Node::Mapping(id));
                out
            }
            Node::Mapping(id) => {
                let Some(m) = d.mappings.get(&id) else {
                    return Vec::new();
                };
                // its own mapping block, the mapping blocks that read it,
                // and the sink it drives
                let mut out: Vec<Node> = Vec::new();
                if has_definition(d, id) {
                    out.push(Node::Definition(id));
                }
                for (from, to) in self.references {
                    if *to == id && *from != id && has_definition(d, *from) {
                        out.push(Node::Definition(*from));
                    }
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
        // a mapping block is attached to its Sem block: directly to its
        // left, centred on it, when the Sem block already has a place
        let attached = match node {
            Node::Definition(d) => self.rects.get(&Node::Mapping(d)).copied(),
            _ => None,
        };
        let x = match attached {
            Some(sem) => sem.x - w - 2.0 * metrics::GAP,
            None => Self::column(node),
        };
        let centres: Vec<f64> = match attached {
            Some(sem) => vec![sem.center_y()],
            None => self
                .neighbours(node)
                .into_iter()
                .filter_map(|n| self.rects.get(&n).map(Rect::center_y))
                .collect(),
        };
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
    references: &[(DeclId, DeclId)],
    placed: &mut Vec<Placed>,
) {
    let mut canvas = Canvas {
        design,
        system,
        references,
        rects: BTreeMap::new(),
    };
    // What is already positioned, at its drawn size — the nodes the
    // canvas draws: a rule's or a concept's stored position takes no
    // space, since neither is a node (ADR-0044).
    let known: Vec<(Node, Point)> = layout
        .mappings
        .iter()
        .filter(|(id, _)| is_sem(design, **id))
        .map(|(id, p)| (Node::Mapping(*id), *p))
        .chain(
            layout
                .definitions
                .iter()
                .filter(|(id, _)| has_definition(design, **id))
                .map(|(id, p)| (Node::Definition(*id), *p)),
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

    // Sem blocks first, then their mapping blocks (attached to them),
    // then instances beside what they are bound to, then sinks beside
    // what drives them — each in id order.
    let mut todo: Vec<Node> = Vec::new();
    todo.extend(
        design
            .mappings
            .keys()
            .filter(|id| is_sem(design, **id) && !layout.mappings.contains_key(id))
            .map(|id| Node::Mapping(*id)),
    );
    todo.extend(
        design
            .mappings
            .keys()
            .filter(|id| has_definition(design, **id) && !layout.definitions.contains_key(id))
            .map(|id| Node::Definition(*id)),
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
            Node::Mapping(id) => {
                layout.mappings.insert(id, at);
            }
            Node::Definition(id) => {
                layout.definitions.insert(id, at);
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

// ---------------------------------------------------------------------------
// Whole-graph arrangement
// ---------------------------------------------------------------------------

/// Arrange every visible node of every canvas from scratch: the explicit
/// *Arrange Automatically* command, and the first opening of a project
/// whose layout has no position at all.  Unlike [`place_missing`] this
/// moves what is already positioned; like it, it is deterministic, reads
/// the semantic graph only to order and align, and writes geometry only
/// (ADR-0003).  The scene it arranges is the one the canvas draws
/// (ADR-0044): every Sem block, its mapping block, every sink and
/// instance, a collapsed group as one box standing for its hidden
/// members, every produce, read, drive and binding edge as supplied —
/// nothing is hidden, inserted or rerouted.  A read edge is a reference
/// edge, so an arrangement without [`References`] chains produce and
/// drive edges alone.
///
/// Left to right by rank (the longest path from a node nothing feeds:
/// what reads follows what it reads, what is driven follows its driver);
/// within a rank, ordered by the mean position of what feeds it so edges
/// cross as little as a few sweeps allow; then aligned to the centre of
/// what feeds it and spread apart until nothing overlaps.  Viewports,
/// expanded groups' boxes (they follow their members) and the positions
/// of members hidden in a collapsed group are untouched.
pub fn arrange(system: &BehaviorSystem, layout: &Layout) -> Layout {
    arrange_with(system, layout, &References::default())
}

/// The reference edges of a design (ADR-0034): `referencing → referenced`,
/// as the analysis reports `dependsOn` — a value's formula naming another
/// relationship.  Not part of the model, so the caller supplies them; the
/// arrangement then draws what references after what it references, as
/// the canvas draws the edge.  Empty: signature edges alone.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct References {
    /// Per canvas: `None` the system canvas, `Some(component)` a body.
    pub edges: BTreeMap<Option<u64>, Vec<(DeclId, DeclId)>>,
}

/// [`arrange`] with the reference edges taken into account.
pub fn arrange_with(system: &BehaviorSystem, layout: &Layout, refs: &References) -> Layout {
    let mut out = layout.clone();
    let none: Vec<(DeclId, DeclId)> = Vec::new();
    arrange_canvas(
        &system.base,
        Some(system),
        &mut out,
        refs.edges.get(&None).unwrap_or(&none),
    );
    for (cid, component) in &system.components {
        let body = out.components.entry(cid.raw()).or_default();
        arrange_canvas(
            &component.body,
            None,
            body,
            refs.edges.get(&Some(cid.raw())).unwrap_or(&none),
        );
        if body == &Layout::default() {
            out.components.remove(&cid.raw());
        }
    }
    out
}

/// Whether a canvas has any position at all: the first-open policy
/// arranges a canvas with none and only fills the gaps of one with some.
pub fn has_positions(layout: &Layout) -> bool {
    !(layout.concepts.is_empty()
        && layout.mappings.is_empty()
        && layout.definitions.is_empty()
        && layout.outputs.is_empty()
        && layout.instances.is_empty())
}

/// A node the arrangement moves: a canvas node, or a collapsed group's box
/// standing for its members.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Item {
    Node(Node),
    Group(u64),
}

fn visit(v: usize, succ: &[Vec<usize>], state: &mut [u8], order: &mut Vec<usize>) {
    state[v] = 1;
    for &w in &succ[v] {
        if state[w] == 0 {
            visit(w, succ, state, order);
        }
    }
    state[v] = 2;
    order.push(v);
}

fn arrange_canvas(
    design: &Design,
    system: Option<&BehaviorSystem>,
    layout: &mut Layout,
    references: &[(DeclId, DeclId)],
) {
    use metrics::*;

    // Hidden members: relationships inside a collapsed group are drawn as
    // the group's box; the box is the item, the members keep their places.
    let mut hidden_by: BTreeMap<DeclId, u64> = BTreeMap::new();
    if let Some(s) = system {
        for (gid, g) in &s.groups {
            let collapsed = layout.groups.get(&gid.raw()).is_some_and(|b| b.collapsed);
            if collapsed {
                for m in &g.members {
                    hidden_by.insert(*m, gid.raw());
                }
            }
        }
    }
    let item_of_decl = |d: DeclId| match hidden_by.get(&d) {
        Some(g) => Item::Group(*g),
        None => Item::Node(Node::Mapping(d)),
    };
    // A hidden member's mapping block is hidden with it.
    let block_of_decl = |d: DeclId| match hidden_by.get(&d) {
        Some(g) => Item::Group(*g),
        None => Item::Node(Node::Definition(d)),
    };

    // The items, in a stable order: Sem blocks, their mapping blocks,
    // then boxes, instances and sinks.
    let mut items: Vec<Item> = Vec::new();
    items.extend(
        design
            .mappings
            .keys()
            .filter(|d| is_sem(design, **d) && !hidden_by.contains_key(d))
            .map(|d| Item::Node(Node::Mapping(*d))),
    );
    items.extend(
        design
            .mappings
            .keys()
            .filter(|d| has_definition(design, **d) && !hidden_by.contains_key(d))
            .map(|d| Item::Node(Node::Definition(*d))),
    );
    let mut groups: Vec<u64> = hidden_by.values().copied().collect();
    groups.sort_unstable();
    groups.dedup();
    items.extend(groups.iter().map(|g| Item::Group(*g)));
    if let Some(s) = system {
        items.extend(
            s.instances
                .keys()
                .map(|i| Item::Node(Node::Instance(i.raw()))),
        );
    }
    items.extend(design.outputs.keys().map(|o| Item::Node(Node::Output(*o))));
    if items.is_empty() {
        return;
    }
    let index: BTreeMap<Item, usize> = items.iter().enumerate().map(|(i, n)| (*n, i)).collect();
    let n = items.len();

    // Directed edges, left to right, as the canvas draws them.
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut push = |a: Item, b: Item| {
        if let (Some(&x), Some(&y)) = (index.get(&a), index.get(&b)) {
            if x != y {
                edges.push((x, y));
            }
        }
    };
    for m in design.mappings.values() {
        if !is_sem(design, m.id) {
            continue;
        }
        let me = item_of_decl(m.id);
        // the produce edge: a Sem block's mapping block into it
        if has_definition(design, m.id) {
            push(block_of_decl(m.id), me);
        }
        if let Some(o) = m.drives {
            push(me, Item::Node(Node::Output(o)));
        }
    }
    // The read edges: a Sem block into the mapping block whose definition
    // names it (the analysis's `dependsOn`, ADR-0034); a referenced rule
    // is a template, named in the block's header, and is no edge.
    for (referencing, referenced) in references {
        if referencing == referenced
            || !has_definition(design, *referencing)
            || !is_sem(design, *referenced)
        {
            continue;
        }
        push(item_of_decl(*referenced), block_of_decl(*referencing));
    }
    if let Some(s) = system {
        let end_item = |e: BindingEnd| match e {
            BindingEnd::Base { decl } => item_of_decl(decl),
            BindingEnd::Port(p) => Item::Node(Node::Instance(p.instance.raw())),
        };
        for b in s.bindings.values() {
            push(end_item(b.source), end_item(b.destination));
        }
    }
    edges.sort_unstable();
    edges.dedup();
    let mut succ: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
    for &(a, b) in &edges {
        succ[a].push(b);
        pred[b].push(a);
    }

    // Ranks: the longest path from a root; a back edge of a cycle (memory
    // through `delay`: a relationship reading what it produces) is skipped
    // in the depth-first order the items are listed in.
    let mut state = vec![0u8; n]; // 0 new, 1 on the stack, 2 done
    let mut post = Vec::with_capacity(n);
    for v in 0..n {
        if state[v] == 0 {
            visit(v, &succ, &mut state, &mut post);
        }
    }
    let topo: Vec<usize> = post.iter().rev().copied().collect();
    let mut position = vec![0; n];
    for (i, &v) in topo.iter().enumerate() {
        position[v] = i;
    }
    let mut rank = vec![0usize; n];
    for &v in &topo {
        for &w in &succ[v] {
            if position[w] > position[v] {
                rank[w] = rank[w].max(rank[v] + 1);
            }
        }
    }
    let ranks = rank.iter().copied().max().unwrap_or(0) + 1;
    let mut layers: Vec<Vec<usize>> = vec![Vec::new(); ranks];
    for v in 0..n {
        layers[rank[v]].push(v);
    }

    // Order within a rank by the mean order of what feeds it (then of what
    // it feeds), a few sweeps each way; ties keep the listed order.
    let mut order: Vec<f64> = (0..n).map(|v| v as f64).collect();
    for sweep in 0..4 {
        let down = sweep % 2 == 0;
        let layer_order: Vec<usize> = if down {
            (0..ranks).collect()
        } else {
            (0..ranks).rev().collect()
        };
        for r in layer_order {
            let mut keyed: Vec<(f64, usize, (u8, u64))> = layers[r]
                .iter()
                .map(|&v| {
                    let nb: Vec<usize> = if down {
                        pred[v].iter().copied().filter(|&p| rank[p] < r).collect()
                    } else {
                        succ[v].iter().copied().filter(|&s| rank[s] > r).collect()
                    };
                    let key = if nb.is_empty() {
                        order[v]
                    } else {
                        nb.iter().map(|&x| order[x]).sum::<f64>() / nb.len() as f64
                    };
                    (key, v, index_key(items[v]))
                })
                .collect();
            keyed.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.2.cmp(&b.2))
            });
            for (i, (_, v, _)) in keyed.iter().enumerate() {
                order[*v] = i as f64;
            }
            layers[r] = keyed.into_iter().map(|(_, v, _)| v).collect();
        }
    }

    // Sizes, as the canvas draws them.
    let measure = Canvas {
        design,
        system,
        references,
        rects: BTreeMap::new(),
    };
    let sizes: Vec<(f64, f64)> = items
        .iter()
        .map(|item| match *item {
            Item::Node(node) => measure.size(node),
            Item::Group(g) => {
                let b = layout.groups.get(&g).copied().unwrap_or_default();
                (
                    b.width.max(INSTANCE_WIDTH),
                    b.height.max(HEADER_HEIGHT + ROW_HEIGHT + BODY_HEIGHT),
                )
            }
        })
        .collect();

    // Coordinates: a column per rank; within it, each node centred on what
    // feeds it, in order, pushed down until it clears the one above.
    let row_gap = 2.0 * GAP;
    let mut x = vec![0.0; n];
    let mut y = vec![0.0; n];
    let mut col_x = ORIGIN_X;
    for (r, layer) in layers.iter().enumerate() {
        let widest = layer.iter().map(|&v| sizes[v].0).fold(0.0, f64::max);
        let mut floor = ORIGIN_Y;
        for &v in layer {
            let (_, h) = sizes[v];
            let feeders: Vec<usize> = pred[v].iter().copied().filter(|&p| rank[p] < r).collect();
            let wanted = if feeders.is_empty() {
                floor
            } else {
                let centre = feeders
                    .iter()
                    .map(|&p| y[p] + sizes[p].1 / 2.0)
                    .sum::<f64>()
                    / feeders.len() as f64;
                (centre - h / 2.0).max(ORIGIN_Y)
            };
            let top = ((wanted.max(floor)) / STEP).round() * STEP;
            x[v] = col_x;
            y[v] = top;
            floor = top + h + row_gap;
        }
        col_x += widest.max(SEM_WIDTH) + (COLUMN_GAP - MAPPING_WIDTH);
    }

    for (i, item) in items.iter().enumerate() {
        let at = Point { x: x[i], y: y[i] };
        match *item {
            Item::Node(Node::Mapping(id)) => {
                layout.mappings.insert(id, at);
            }
            Item::Node(Node::Definition(id)) => {
                layout.definitions.insert(id, at);
            }
            Item::Node(Node::Output(id)) => {
                layout.outputs.insert(id, at);
            }
            Item::Node(Node::Instance(raw)) => {
                layout.instances.insert(raw, at);
            }
            Item::Group(g) => {
                let b = layout.groups.entry(g).or_default();
                b.x = at.x;
                b.y = at.y;
                b.collapsed = true;
            }
        }
    }
}

/// A stable tie-break: kind, then id.
fn index_key(item: Item) -> (u8, u64) {
    match item {
        Item::Node(Node::Definition(d)) => (0, d.raw()),
        Item::Node(Node::Mapping(d)) => (1, d.raw()),
        Item::Group(g) => (2, g),
        Item::Node(Node::Instance(i)) => (3, i),
        Item::Node(Node::Output(o)) => (4, o.raw()),
    }
}
