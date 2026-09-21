//! Whole-graph arrangement (`bdl_layout::arrange`): left to right by rank
//! over the picture the canvas draws (ADR-0044) — Sem blocks, their
//! mapping blocks, sinks — deterministic, nothing overlapping.

#![allow(clippy::unwrap_used)]

use bdl_layout::{arrange, arrange_with, has_positions, metrics, place_missing, Node, References};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::layout::{GroupBox, Layout, Point};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ConceptId, DeclId, OutputId};
use bdl_system::{apply_group_edit, BehaviorSystem, GroupEditOp, GroupScope};

const PRESSED: DeclId = DeclId::from_raw(0);
const LIT_RULE: DeclId = DeclId::from_raw(1);
const LIT: DeclId = DeclId::from_raw(2);
const TWICE: DeclId = DeclId::from_raw(3);
const LAMP: OutputId = OutputId::from_raw(0);

/// The demo in the ladder's words: `pressed : Pressed` a Source Sem block;
/// `lit : Pressed -> Lit` a rule (a template, not drawn); `Lit :=
/// lit(pressed)` a Sem block with its mapping block, driving `lamp`; and a
/// second stage `twice : Twice := !lit` reading it.
fn button_lamp() -> Design {
    let mut s = ProjectSnapshot::new(Design::empty("demo"));
    let ops = [
        EditOp::CreateConcept {
            name: "Pressed".into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        },
        EditOp::CreateConcept {
            name: "Lit".into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        },
        EditOp::CreateConcept {
            name: "Twice".into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        },
        EditOp::CreateMapping {
            name: "pressed".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: ConceptId::from_raw(0),
            },
            definition: None,
            clock: None,
        },
        EditOp::CreateMapping {
            name: "litRule".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![ConceptId::from_raw(0)],
                output: ConceptId::from_raw(1),
            },
            definition: Some(Definition::Formula {
                source: "Pressed".into(),
            }),
            clock: None,
        },
        EditOp::CreateMapping {
            name: "lit".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: ConceptId::from_raw(1),
            },
            definition: Some(Definition::Formula {
                source: "litRule(pressed)".into(),
            }),
            clock: None,
        },
        EditOp::CreateMapping {
            name: "twice".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: ConceptId::from_raw(2),
            },
            definition: Some(Definition::Formula {
                source: "!lit".into(),
            }),
            clock: None,
        },
        EditOp::CreateOutput {
            name: "lamp".into(),
            description: String::new(),
            accepts: ConceptId::from_raw(1),
            clock: None,
        },
        EditOp::SetMappingDrive {
            id: LIT,
            output: Some(LAMP),
        },
    ];
    for op in &ops {
        s = apply_edit(&s, op).unwrap().snapshot;
    }
    s.design
}

/// The read edges of [`button_lamp`], as the analysis reports them.
fn demo_refs() -> References {
    let mut refs = References::default();
    refs.edges
        .insert(None, vec![(LIT, PRESSED), (LIT, LIT_RULE), (TWICE, LIT)]);
    refs
}

fn rect(layout: &Layout, node: Node) -> (f64, f64, f64, f64) {
    use metrics::*;
    match node {
        Node::Mapping(id) => {
            let p = layout.mappings[&id];
            (p.x, p.y, SEM_WIDTH, SEM_HEIGHT)
        }
        Node::Definition(id) => {
            let p = layout.definitions[&id];
            // every mapping block of the demo reads one Sem block
            (
                p.x,
                p.y,
                MAPPING_WIDTH,
                HEADER_HEIGHT + ROW_HEIGHT + BODY_HEIGHT,
            )
        }
        Node::Output(id) => {
            let p = layout.outputs[&id];
            (p.x, p.y, SEM_WIDTH, HEADER_HEIGHT + ROW_HEIGHT)
        }
        Node::Instance(_) => unreachable!(),
    }
}

fn overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

fn nodes(system: &BehaviorSystem, layout: &Layout) -> Vec<Node> {
    let mut v: Vec<Node> = layout
        .mappings
        .keys()
        .filter(|m| system.base.mappings[m].signature.is_unit_domain())
        .map(|m| Node::Mapping(*m))
        .collect();
    v.extend(layout.definitions.keys().map(|m| Node::Definition(*m)));
    v.extend(layout.outputs.keys().map(|o| Node::Output(*o)));
    v
}

fn assert_no_overlap(system: &BehaviorSystem, layout: &Layout) {
    let all = nodes(system, layout);
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert!(
                !overlap(rect(layout, *a), rect(layout, *b)),
                "{a:?} overlaps {b:?}"
            );
        }
    }
}

#[test]
fn the_demo_is_arranged_left_to_right_with_every_edge_forward_and_nothing_overlapping() {
    let system = BehaviorSystem::from_flat(button_lamp());
    let l = arrange_with(&system, &Layout::default(), &demo_refs());
    assert_no_overlap(&system, &l);
    // no concept node, no rule node
    assert!(l.concepts.is_empty());
    assert!(!l.mappings.contains_key(&LIT_RULE));
    assert!(!l.definitions.contains_key(&LIT_RULE));
    let pressed = l.mappings[&PRESSED];
    let lit_block = l.definitions[&LIT];
    let lit = l.mappings[&LIT];
    let twice_block = l.definitions[&TWICE];
    let twice = l.mappings[&TWICE];
    let lamp = l.outputs[&LAMP];
    // rank: pressed (0) → lit's block (1) → lit (2) → twice's block, lamp (3) → twice (4)
    assert!(
        pressed.x < lit_block.x,
        "the read Sem block before the block"
    );
    assert!(
        lit_block.x < lit.x,
        "the block before the Sem block it produces"
    );
    assert!(
        lit.x < twice_block.x && lit.x < lamp.x,
        "both of lit's edges go forward"
    );
    assert!(
        (twice_block.x - lamp.x).abs() < 1e-9,
        "the reading block and the driven sink share the rank after lit"
    );
    assert!(twice_block.x < twice.x);
    // what is fed sits beside what feeds it
    assert!(
        (lit_block.y + 35.0 - (lit.y + 24.0)).abs() < 80.0,
        "block {lit_block:?} lit {lit:?}"
    );
    assert!(pressed.y >= metrics::ORIGIN_Y);
}

#[test]
fn arranging_is_deterministic_and_moves_authored_positions() {
    let system = BehaviorSystem::from_flat(button_lamp());
    let mut authored = Layout::default();
    authored
        .concepts
        .insert(ConceptId::from_raw(0), Point { x: 900.0, y: 900.0 });
    authored.mappings.insert(LIT, Point { x: 5.0, y: 5.0 });
    authored.viewport = Some(bdl_model::layout::Viewport {
        x: 1.0,
        y: 2.0,
        zoom: 0.5,
    });
    let a = arrange(&system, &authored);
    let b = arrange(&system, &authored);
    let c = arrange(&system, &Layout::default());
    assert_eq!(a, b, "deterministic");
    assert_eq!(
        a.mappings, c.mappings,
        "the authored positions do not steer the result"
    );
    assert_eq!(a.definitions, c.definitions);
    assert_ne!(a.mappings[&LIT], Point { x: 5.0, y: 5.0 });
    // a concept's stored position is neither a node nor moved
    assert_eq!(
        a.concepts[&ConceptId::from_raw(0)],
        Point { x: 900.0, y: 900.0 }
    );
    assert_eq!(
        a.viewport, authored.viewport,
        "the viewport is the designer's"
    );
    assert!(has_positions(&a));
    assert!(!has_positions(&Layout::default()));
    // place_missing after an arrangement has nothing to do
    assert!(place_missing(&system, &a).is_empty());
}

#[test]
fn a_collapsed_group_is_one_box_and_its_hidden_members_keep_their_places() {
    let base = BehaviorSystem::from_flat(button_lamp());
    let (system, outcome) = apply_group_edit(
        &base,
        &GroupEditOp::CreateGroup {
            scope: GroupScope::SystemBase,
            name: "Lamp logic".into(),
            description: String::new(),
            members: vec![LIT],
        },
    )
    .unwrap();
    let gid = outcome.created_group.unwrap().raw();
    let mut before = Layout::default();
    before.mappings.insert(LIT, Point { x: 777.0, y: 777.0 });
    before.definitions.insert(LIT, Point { x: 700.0, y: 777.0 });
    before.groups.insert(
        gid,
        GroupBox {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 90.0,
            collapsed: true,
        },
    );
    let l = arrange_with(&system, &before, &demo_refs());
    // the box moved into the flow: right of pressed, left of twice's block
    let g = l.groups[&gid];
    assert!(g.collapsed);
    assert!(l.mappings[&PRESSED].x < g.x);
    assert!(g.x < l.definitions[&TWICE].x);
    // the member and its mapping block inside it were not touched
    assert_eq!(l.mappings[&LIT], Point { x: 777.0, y: 777.0 });
    assert_eq!(l.definitions[&LIT], Point { x: 700.0, y: 777.0 });
    // an expanded group: its members are ordinary nodes, its box is derived
    let mut expanded = before.clone();
    expanded.groups.get_mut(&gid).unwrap().collapsed = false;
    let l2 = arrange_with(&system, &expanded, &demo_refs());
    assert_ne!(l2.mappings[&LIT], Point { x: 777.0, y: 777.0 });
    assert_no_overlap(&system, &l2);
}

#[test]
fn a_cycle_through_memory_still_arranges() {
    // r reads itself (memory through `delay`): a cycle the rank must not
    // loop on.
    let mut s = ProjectSnapshot::new(Design::empty("loop"));
    let ops = [
        EditOp::CreateConcept {
            name: "Twice".into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        },
        EditOp::CreateMapping {
            name: "r".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: ConceptId::from_raw(0),
            },
            definition: Some(Definition::Formula {
                source: "delay(false, !r)".into(),
            }),
            clock: None,
        },
    ];
    for op in &ops {
        s = apply_edit(&s, op).unwrap().snapshot;
    }
    let system = BehaviorSystem::from_flat(s.design);
    let mut refs = References::default();
    refs.edges
        .insert(None, vec![(DeclId::from_raw(0), DeclId::from_raw(0))]);
    let l = arrange_with(&system, &Layout::default(), &refs);
    assert_no_overlap(&system, &l);
    assert_eq!(l.mappings.len(), 1);
    assert_eq!(l.definitions.len(), 1);
}
