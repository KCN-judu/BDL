//! The layout service places what has no position, deterministically,
//! beside what it reads, without moving anything (ADR-0023 §7).

#![allow(clippy::unwrap_used)]

use bdl_layout::{metrics, place_missing, Node};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::layout::{GroupBox, Layout, Point};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, Dim, OutputId, SemanticId};
use bdl_system::{
    apply_system_edit, BehaviorSystem, BindingEnd, PortKind, SystemEditOp, SystemSnapshot,
};

/// Tilt → dimByTilt → Brightness → light, plus an unread concept.
fn lamp() -> Design {
    let mut s = ProjectSnapshot::new(Design::empty("lamp"));
    let ops = [
        EditOp::CreateConcept {
            name: "Tilt".into(),
            description: String::new(),
            representation: Some(Representation::Quantity { dim: Dim::ANGLE }),
        },
        EditOp::CreateConcept {
            name: "Brightness".into(),
            description: String::new(),
            representation: Some(Representation::Quantity { dim: Dim::ZERO }),
        },
        EditOp::CreateConcept {
            name: "Spare".into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        },
        EditOp::CreateMapping {
            name: "tilt".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: SemanticId::from_raw(0),
            },
        },
        EditOp::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![SemanticId::from_raw(0)],
                output: SemanticId::from_raw(1),
            },
        },
        EditOp::AttachDefinition {
            id: DeclId::from_raw(1),
            definition: Definition::Formula {
                source: "Tilt / 90 deg".into(),
            },
        },
        EditOp::CreateOutput {
            name: "light".into(),
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
        Node::Instance(raw) => {
            let p = layout.instances[&raw];
            let inst = system
                .instances
                .values()
                .find(|i| i.id.raw() == raw)
                .unwrap();
            let ports = system.components[&inst.component].interface.ports.len();
            (
                p.x,
                p.y,
                INSTANCE_WIDTH,
                HEADER_HEIGHT + rows(ports) * ROW_HEIGHT + BODY_HEIGHT,
            )
        }
    }
}

fn overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

fn all_nodes(layout: &Layout) -> Vec<Node> {
    let mut v: Vec<Node> = layout.concepts.keys().map(|c| Node::Concept(*c)).collect();
    v.extend(layout.mappings.keys().map(|m| Node::Mapping(*m)));
    v.extend(layout.outputs.keys().map(|o| Node::Output(*o)));
    v.extend(layout.instances.keys().map(|i| Node::Instance(*i)));
    v
}

fn assert_no_overlap(system: &BehaviorSystem, layout: &Layout) {
    let nodes = all_nodes(layout);
    for (i, a) in nodes.iter().enumerate() {
        for b in &nodes[i + 1..] {
            assert!(
                !overlap(rect(system, layout, *a), rect(system, layout, *b)),
                "{a:?} overlaps {b:?}"
            );
        }
    }
}

#[test]
fn an_empty_layout_is_filled_left_to_right_and_the_same_way_twice() {
    let system = BehaviorSystem::from_flat(lamp());
    let a = place_missing(&system, &Layout::default());
    let b = place_missing(&system, &Layout::default());
    assert_eq!(a, b, "deterministic");
    assert_eq!(a.placed.len(), 3 + 2 + 1);
    let l = &a.layout;
    assert_no_overlap(&system, l);
    // columns: concepts, relationships, sinks
    let tilt = l.concepts[&SemanticId::from_raw(0)];
    let dim = l.mappings[&DeclId::from_raw(1)];
    let light = l.outputs[&OutputId::from_raw(0)];
    assert!(tilt.x < dim.x && dim.x < light.x);
    // the sink sits beside the relationship that drives it, which sits
    // beside the concept it reads
    assert!(
        (light.y - dim.y).abs() < 60.0,
        "light {light:?} dim {dim:?}"
    );
    assert!((dim.y - tilt.y).abs() < 120.0, "dim {dim:?} tilt {tilt:?}");
    // a second pass places nothing
    assert!(place_missing(&system, l).is_empty());
}

#[test]
fn positioned_nodes_never_move_and_new_ones_land_beside_what_they_read() {
    let system = BehaviorSystem::from_flat(lamp());
    let mut layout = Layout::default();
    layout
        .concepts
        .insert(SemanticId::from_raw(0), Point { x: 900.0, y: 700.0 });
    layout
        .mappings
        .insert(DeclId::from_raw(0), Point { x: 10.0, y: 10.0 });
    let before = layout.clone();
    let p = place_missing(&system, &layout);
    let l = &p.layout;
    for (id, at) in &before.concepts {
        assert_eq!(l.concepts[id], *at);
    }
    for (id, at) in &before.mappings {
        assert_eq!(l.mappings[id], *at);
    }
    assert!(p
        .placed
        .iter()
        .all(|x| x.node != Node::Concept(SemanticId::from_raw(0))
            && x.node != Node::Mapping(DeclId::from_raw(0))));
    assert_no_overlap(&system, l);
    // dimByTilt reads Tilt, which the designer put far down: it follows
    let dim = l.mappings[&DeclId::from_raw(1)];
    assert!((dim.y - 700.0).abs() < 80.0, "{dim:?}");
    // and light follows dimByTilt
    let light = l.outputs[&OutputId::from_raw(0)];
    assert!((light.y - dim.y).abs() < 60.0, "{light:?}");
}

#[test]
fn a_collapsed_group_box_is_not_covered() {
    let system = BehaviorSystem::from_flat(lamp());
    let mut layout = Layout::default();
    layout.groups.insert(
        7,
        GroupBox {
            x: metrics::ORIGIN_X + metrics::COLUMN_GAP,
            y: metrics::ORIGIN_Y,
            width: 208.0,
            height: 120.0,
            collapsed: true,
        },
    );
    let l = place_missing(&system, &layout).layout;
    let boxr = (l.groups[&7].x, l.groups[&7].y, 208.0, 120.0);
    for n in all_nodes(&l) {
        assert!(
            !overlap(rect(&system, &l, n), boxr),
            "{n:?} covers the group box"
        );
    }
    assert_eq!(l.groups, layout.groups, "boxes are untouched");
}

#[test]
fn instances_and_component_bodies_are_placed_too() {
    let mut system = SystemSnapshot::new(BehaviorSystem::from_flat(lamp()));
    let ops = [SystemEditOp::CreateComponent {
        name: "Dimmer".into(),
        description: String::new(),
    }];
    for op in &ops {
        system = apply_system_edit(&system, op).unwrap().snapshot;
    }
    let cid = *system.system.components.keys().next().unwrap();
    // give the body a concept and a relationship, expose the relationship
    let body_ops = [
        EditOp::CreateConcept {
            name: "Level".into(),
            description: String::new(),
            representation: Some(Representation::Quantity { dim: Dim::ZERO }),
        },
        EditOp::CreateMapping {
            name: "level".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: SemanticId::from_raw(0),
            },
        },
    ];
    for op in body_ops {
        system = apply_system_edit(
            &system,
            &SystemEditOp::EditComponentBody { component: cid, op },
        )
        .unwrap()
        .snapshot;
    }
    system = apply_system_edit(
        &system,
        &SystemEditOp::DeclarePort {
            component: cid,
            decl: DeclId::from_raw(0),
            kind: PortKind::Required,
            name: "level".into(),
            description: String::new(),
        },
    )
    .unwrap()
    .snapshot;
    system = apply_system_edit(
        &system,
        &SystemEditOp::CreateInstance {
            component: cid,
            name: "dimmer".into(),
        },
    )
    .unwrap()
    .snapshot;
    let inst = *system.system.instances.keys().next().unwrap();
    let port = *system.system.components[&cid]
        .interface
        .ports
        .keys()
        .next()
        .unwrap();
    // dimByTilt (base) feeds the instance's required port
    system = apply_system_edit(
        &system,
        &SystemEditOp::BindPorts {
            source: BindingEnd::Base {
                decl: DeclId::from_raw(1),
            },
            destination: BindingEnd::port(inst, port),
            transport: None,
        },
    )
    .unwrap()
    .snapshot;

    let mut layout = Layout::default();
    layout
        .mappings
        .insert(DeclId::from_raw(1), Point { x: 368.0, y: 500.0 });
    let p = place_missing(&system.system, &layout);
    let l = &p.layout;
    assert_no_overlap(&system.system, l);
    let at = l.instances[&inst.raw()];
    assert!((at.y - 500.0).abs() < 200.0, "beside what feeds it: {at:?}");
    let body = &l.components[&cid.raw()];
    assert!(body.concepts.contains_key(&SemanticId::from_raw(0)));
    assert!(body.mappings.contains_key(&DeclId::from_raw(0)));
    assert!(p.placed.iter().any(
        |x| x.component == Some(cid.raw()) && x.node == Node::Concept(SemanticId::from_raw(0))
    ));
    assert!(place_missing(&system.system, l).is_empty());
}
