//! Whole-graph arrangement: deterministic, layout-only, left to right by
//! rank, no overlaps, the topology as supplied (a relationship that
//! produces a concept and drives a sink keeps both edges forward), nothing
//! hidden or moved that should not be.

#![allow(clippy::unwrap_used)]

use bdl_layout::{arrange, has_positions, metrics, place_missing, Node};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::layout::{GroupBox, Layout, Point};
use bdl_model::surface::{Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, OutputId, SemanticId};
use bdl_system::{apply_group_edit, BehaviorSystem, GroupEditOp, GroupScope};

/// Pressed → lit → Lit; lit → lamp (the demo: one relationship producing
/// a concept and driving a sink), plus a second stage `again : Lit → Twice`.
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
                output: SemanticId::from_raw(0),
            },
            definition: None,
            clock: None,
        },
        EditOp::CreateMapping {
            name: "lit".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![SemanticId::from_raw(0)],
                output: SemanticId::from_raw(1),
            },
            definition: None,
            clock: None,
        },
        EditOp::CreateMapping {
            name: "again".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![SemanticId::from_raw(1)],
                output: SemanticId::from_raw(2),
            },
            definition: None,
            clock: None,
        },
        EditOp::CreateOutput {
            name: "lamp".into(),
            description: String::new(),
            accepts: SemanticId::from_raw(1),
            clock: None,
        },
        EditOp::SetMappingDrive {
            id: DeclId::from_raw(1),
            output: Some(OutputId::from_raw(0)),
        },
    ];
    for op in &ops {
        s = apply_edit(&s, op).unwrap().snapshot;
    }
    s.design
}

fn rect(system: &BehaviorSystem, layout: &Layout, node: Node) -> (f64, f64, f64, f64) {
    use metrics::*;
    let rows = |n: usize| n.max(1) as f64;
    match node {
        Node::Concept(id) => {
            let p = layout.concepts[&id];
            (p.x, p.y, CONCEPT_WIDTH, CONCEPT_HEIGHT)
        }
        Node::Mapping(id) => {
            let p = layout.mappings[&id];
            let inputs = system.base.mappings[&id].signature.inputs.len();
            (
                p.x,
                p.y,
                MAPPING_WIDTH,
                HEADER_HEIGHT + rows(inputs) * ROW_HEIGHT + BODY_HEIGHT,
            )
        }
        Node::Output(id) => {
            let p = layout.outputs[&id];
            (p.x, p.y, CONCEPT_WIDTH, HEADER_HEIGHT + ROW_HEIGHT)
        }
        Node::Instance(_) => unreachable!(),
    }
}

fn overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

fn nodes(layout: &Layout) -> Vec<Node> {
    let mut v: Vec<Node> = layout.concepts.keys().map(|c| Node::Concept(*c)).collect();
    v.extend(layout.mappings.keys().map(|m| Node::Mapping(*m)));
    v.extend(layout.outputs.keys().map(|o| Node::Output(*o)));
    v
}

fn assert_no_overlap(system: &BehaviorSystem, layout: &Layout) {
    let all = nodes(layout);
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert!(
                !overlap(rect(system, layout, *a), rect(system, layout, *b)),
                "{a:?} overlaps {b:?}"
            );
        }
    }
}

#[test]
fn the_demo_is_arranged_left_to_right_with_every_edge_forward_and_nothing_overlapping() {
    let system = BehaviorSystem::from_flat(button_lamp());
    let l = arrange(&system, &Layout::default());
    assert_no_overlap(&system, &l);
    let pressed_c = l.concepts[&SemanticId::from_raw(0)];
    let lit_c = l.concepts[&SemanticId::from_raw(1)];
    let twice_c = l.concepts[&SemanticId::from_raw(2)];
    let pressed = l.mappings[&DeclId::from_raw(0)];
    let lit = l.mappings[&DeclId::from_raw(1)];
    let again = l.mappings[&DeclId::from_raw(2)];
    let lamp = l.outputs[&OutputId::from_raw(0)];
    // rank: pressed (0) → Pressed (1) → lit (2) → Lit, lamp (3) → again (4) → Twice (5)
    assert!(pressed.x < pressed_c.x, "the source before its concept");
    assert!(pressed_c.x < lit.x, "the concept before what reads it");
    assert!(
        lit.x < lit_c.x && lit.x < lamp.x,
        "both of lit's edges go forward"
    );
    assert!(
        (lit_c.x - lamp.x).abs() < 1e-9,
        "the produced concept and the driven sink share the rank after lit"
    );
    assert!(lit_c.x < again.x && again.x < twice_c.x);
    // what is fed sits beside what feeds it
    assert!(
        (lit.y + 35.0 - (lit_c.y + 13.0)).abs() < 80.0,
        "lit {lit:?} Lit {lit_c:?}"
    );
    assert!(pressed_c.y >= metrics::ORIGIN_Y && pressed.y >= metrics::ORIGIN_Y);
}

#[test]
fn arranging_is_deterministic_and_moves_authored_positions() {
    let system = BehaviorSystem::from_flat(button_lamp());
    let mut authored = Layout::default();
    authored
        .concepts
        .insert(SemanticId::from_raw(0), Point { x: 900.0, y: 900.0 });
    authored
        .mappings
        .insert(DeclId::from_raw(1), Point { x: 5.0, y: 5.0 });
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
        a.concepts, c.concepts,
        "the authored positions do not steer the result"
    );
    assert_eq!(a.mappings, c.mappings);
    assert_ne!(
        a.concepts[&SemanticId::from_raw(0)],
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
            members: vec![DeclId::from_raw(1), DeclId::from_raw(2)],
        },
    )
    .unwrap();
    let gid = outcome.created_group.unwrap().raw();
    let mut before = Layout::default();
    before
        .mappings
        .insert(DeclId::from_raw(1), Point { x: 777.0, y: 777.0 });
    before
        .mappings
        .insert(DeclId::from_raw(2), Point { x: 778.0, y: 778.0 });
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
    let l = arrange(&system, &before);
    // the box moved into the flow, right of Pressed and left of Lit
    let g = l.groups[&gid];
    assert!(g.collapsed);
    assert!(l.concepts[&SemanticId::from_raw(0)].x < g.x);
    assert!(g.x < l.concepts[&SemanticId::from_raw(1)].x);
    // the members inside it were not touched
    assert_eq!(
        l.mappings[&DeclId::from_raw(1)],
        Point { x: 777.0, y: 777.0 }
    );
    assert_eq!(
        l.mappings[&DeclId::from_raw(2)],
        Point { x: 778.0, y: 778.0 }
    );
    // an expanded group: its members are ordinary nodes, its box is derived
    let mut expanded = before.clone();
    expanded.groups.get_mut(&gid).unwrap().collapsed = false;
    let l2 = arrange(&system, &expanded);
    assert_ne!(
        l2.mappings[&DeclId::from_raw(1)],
        Point { x: 777.0, y: 777.0 }
    );
    assert_no_overlap(&system, &l2);
}

#[test]
fn a_cycle_through_memory_still_arranges() {
    // r reads Twice and produces Twice (a delay-like memory): a cycle the
    // rank must not loop on.
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
                inputs: vec![SemanticId::from_raw(0)],
                output: SemanticId::from_raw(0),
            },
            definition: None,
            clock: None,
        },
    ];
    for op in &ops {
        s = apply_edit(&s, op).unwrap().snapshot;
    }
    let system = BehaviorSystem::from_flat(s.design);
    let l = arrange(&system, &Layout::default());
    assert_no_overlap(&system, &l);
    assert_eq!(l.concepts.len(), 1);
    assert_eq!(l.mappings.len(), 1);
}
