//! The layout service places what has no position, deterministically,
//! beside what it reads, without moving anything (ADR-0023 §7).  The
//! nodes are the concept ladder's (ADR-0044): Sem blocks, their mapping
//! blocks, sinks and instances — never a concept, never a rule.

#![allow(clippy::unwrap_used)]

use bdl_layout::{metrics, place_missing, place_missing_with, Node, References};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::layout::{GroupBox, Layout, Point};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ConceptId, DeclId, Dim, OutputId};
use bdl_system::{
    apply_system_edit, BehaviorSystem, BindingEnd, PortKind, SystemEditOp, SystemSnapshot,
};

const TILT: DeclId = DeclId::from_raw(0);
const DIM_BY_TILT: DeclId = DeclId::from_raw(1);
const BRIGHTNESS: DeclId = DeclId::from_raw(2);
const LIGHT: OutputId = OutputId::from_raw(0);

/// `tilt : Tilt` (a Source Sem block), the rule `dimByTilt : Tilt ->
/// Brightness` (a template, not drawn), `brightness : Brightness =
/// dimByTilt(tilt)` (a Sem block with its mapping block) driving `light`;
/// plus an unread concept.
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
                output: ConceptId::from_raw(0),
            },
            definition: None,
            clock: None,
        },
        EditOp::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![ConceptId::from_raw(0)],
                output: ConceptId::from_raw(1),
            },
            definition: Some(Definition::Formula {
                source: "Tilt / 90 deg".into(),
            }),
            clock: None,
        },
        EditOp::CreateMapping {
            name: "brightness".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: ConceptId::from_raw(1),
            },
            definition: Some(Definition::Formula {
                source: "dimByTilt(tilt)".into(),
            }),
            clock: None,
        },
        EditOp::CreateOutput {
            name: "light".into(),
            description: String::new(),
            accepts: ConceptId::from_raw(1),
            clock: None,
        },
        EditOp::SetMappingDrive {
            id: BRIGHTNESS,
            output: Some(LIGHT),
        },
    ];
    for op in &ops {
        s = apply_edit(&s, op).unwrap().snapshot;
    }
    s.design
}

/// The read edges the analysis would report for [`lamp`]: the mapping
/// block of `brightness` names `tilt` and the rule.
fn lamp_refs() -> References {
    let mut refs = References::default();
    refs.edges
        .insert(None, vec![(BRIGHTNESS, TILT), (BRIGHTNESS, DIM_BY_TILT)]);
    refs
}

fn rect(system: &BehaviorSystem, layout: &Layout, node: Node) -> (f64, f64, f64, f64) {
    use metrics::*;
    let rows = |n: usize| n.max(1) as f64;
    match node {
        Node::Mapping(id) => {
            let p = layout.mappings[&id];
            (p.x, p.y, SEM_WIDTH, SEM_HEIGHT)
        }
        Node::Definition(id) => {
            let p = layout.definitions[&id];
            // the lamp's one mapping block reads one Sem block
            (
                p.x,
                p.y,
                MAPPING_WIDTH,
                HEADER_HEIGHT + rows(1) * ROW_HEIGHT + BODY_HEIGHT,
            )
        }
        Node::Output(id) => {
            let p = layout.outputs[&id];
            (p.x, p.y, SEM_WIDTH, HEADER_HEIGHT + ROW_HEIGHT)
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

/// The nodes the canvas draws: Sem blocks (unit-domain declarations),
/// their mapping blocks, sinks, instances.
fn drawn_nodes(system: &BehaviorSystem, layout: &Layout) -> Vec<Node> {
    let mut v: Vec<Node> = layout
        .mappings
        .keys()
        .filter(|m| system.base.mappings[m].signature.is_unit_domain())
        .map(|m| Node::Mapping(*m))
        .collect();
    v.extend(layout.definitions.keys().map(|m| Node::Definition(*m)));
    v.extend(layout.outputs.keys().map(|o| Node::Output(*o)));
    v.extend(layout.instances.keys().map(|i| Node::Instance(*i)));
    v
}

fn assert_no_overlap(system: &BehaviorSystem, layout: &Layout) {
    let nodes = drawn_nodes(system, layout);
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
    let a = place_missing_with(&system, &Layout::default(), &lamp_refs());
    let b = place_missing_with(&system, &Layout::default(), &lamp_refs());
    assert_eq!(a, b, "deterministic");
    // two Sem blocks, one mapping block, one sink — no concept, no rule
    assert_eq!(a.placed.len(), 2 + 1 + 1);
    let l = &a.layout;
    assert!(l.concepts.is_empty(), "a concept is a template, not a node");
    assert!(
        !l.mappings.contains_key(&DIM_BY_TILT),
        "a rule is a template, not a node"
    );
    assert_no_overlap(&system, l);
    // the mapping block is attached left of the Sem block it produces,
    // which sits left of the sink it drives
    let block = l.definitions[&BRIGHTNESS];
    let sem = l.mappings[&BRIGHTNESS];
    let light = l.outputs[&LIGHT];
    assert!(block.x < sem.x && sem.x < light.x);
    assert!(
        (block.x + metrics::MAPPING_WIDTH + 2.0 * metrics::GAP - sem.x).abs() < 1e-9,
        "attached: block {block:?} sem {sem:?}"
    );
    assert!(
        (light.y - sem.y).abs() < 60.0,
        "light {light:?} sem {sem:?}"
    );
    // a second pass places nothing
    assert!(place_missing_with(&system, l, &lamp_refs()).is_empty());
}

#[test]
fn positioned_nodes_never_move_and_a_mapping_block_attaches_to_its_sem_block() {
    let system = BehaviorSystem::from_flat(lamp());
    let mut layout = Layout::default();
    layout.mappings.insert(TILT, Point { x: 10.0, y: 10.0 });
    layout
        .mappings
        .insert(BRIGHTNESS, Point { x: 900.0, y: 700.0 });
    let before = layout.clone();
    let p = place_missing_with(&system, &layout, &lamp_refs());
    let l = &p.layout;
    for (id, at) in &before.mappings {
        assert_eq!(l.mappings[id], *at);
    }
    assert!(p
        .placed
        .iter()
        .all(|x| x.node != Node::Mapping(TILT) && x.node != Node::Mapping(BRIGHTNESS)));
    assert_no_overlap(&system, l);
    // the mapping block goes beside the Sem block the designer placed far
    // down, centred on it
    let block = l.definitions[&BRIGHTNESS];
    assert!((block.x - (900.0 - metrics::MAPPING_WIDTH - 2.0 * metrics::GAP)).abs() < 1e-9);
    assert!((block.y - 700.0).abs() < 40.0, "{block:?}");
    // and light follows brightness (its column lies under the attached
    // block here, so it steps below it)
    let light = l.outputs[&LIGHT];
    assert!((light.y - 700.0).abs() < 120.0, "{light:?}");
}

/// A project laid out before ADR-0044: relationships and concepts have
/// positions, mapping blocks none.  Only the mapping blocks are placed;
/// the stored concept and rule positions are kept as written.
#[test]
fn an_old_layout_gets_its_mapping_blocks_and_nothing_else_moves() {
    let system = BehaviorSystem::from_flat(lamp());
    let mut layout = Layout::default();
    layout
        .concepts
        .insert(ConceptId::from_raw(0), Point { x: 48.0, y: 48.0 });
    layout
        .concepts
        .insert(ConceptId::from_raw(1), Point { x: 48.0, y: 148.0 });
    layout.mappings.insert(TILT, Point { x: 368.0, y: 48.0 });
    layout
        .mappings
        .insert(DIM_BY_TILT, Point { x: 368.0, y: 148.0 });
    layout
        .mappings
        .insert(BRIGHTNESS, Point { x: 368.0, y: 300.0 });
    layout.outputs.insert(LIGHT, Point { x: 688.0, y: 300.0 });
    let p = place_missing(&system, &layout);
    assert_eq!(
        p.placed.iter().map(|x| x.node).collect::<Vec<_>>(),
        vec![Node::Definition(BRIGHTNESS)]
    );
    let l = &p.layout;
    assert_eq!(l.concepts, layout.concepts);
    assert_eq!(l.mappings, layout.mappings);
    assert_eq!(l.outputs, layout.outputs);
    assert_no_overlap(&system, l);
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
    let l = place_missing_with(&system, &layout, &lamp_refs()).layout;
    let boxr = (l.groups[&7].x, l.groups[&7].y, 208.0, 120.0);
    for n in drawn_nodes(&system, &l) {
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
    // give the body a concept and a Sem block, expose the Sem block
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
                output: ConceptId::from_raw(0),
            },
            definition: None,
            clock: None,
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
    // brightness (base) feeds the instance's required port
    system = apply_system_edit(
        &system,
        &SystemEditOp::BindPorts {
            source: BindingEnd::Base { decl: BRIGHTNESS },
            destination: BindingEnd::port(inst, port),
            transport: None,
        },
    )
    .unwrap()
    .snapshot;

    let mut layout = Layout::default();
    layout
        .mappings
        .insert(BRIGHTNESS, Point { x: 368.0, y: 500.0 });
    let p = place_missing_with(&system.system, &layout, &lamp_refs());
    let l = &p.layout;
    assert_no_overlap(&system.system, l);
    let at = l.instances[&inst.raw()];
    assert!((at.y - 500.0).abs() < 200.0, "beside what feeds it: {at:?}");
    let body = &l.components[&cid.raw()];
    assert!(body.concepts.is_empty(), "no concept node in a body either");
    assert!(body.mappings.contains_key(&DeclId::from_raw(0)));
    assert!(p
        .placed
        .iter()
        .any(|x| x.component == Some(cid.raw()) && x.node == Node::Mapping(DeclId::from_raw(0))));
    assert!(place_missing_with(&system.system, l, &lamp_refs()).is_empty());
}
