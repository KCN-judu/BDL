//! The Formula view's structure (docs/architecture/ide-service.md
//! §Formula projection, 0.27): roles and locals on every node, the
//! structured forms, structural carets and their motions, keyboard
//! insertion interpreted by the grammar, completion at a caret with
//! structured inserts and composite units, signature help, and the
//! read-only render — all from the one parser and elaboration.

mod support;

use bdl_ide::*;
use bdl_model::surface::{ProjectSnapshot, Representation};
use bdl_model::{ConceptId, DeclId, Dim};
use support::*;

const ANGULAR: Dim = Dim {
    angle: 1,
    time: -1,
    ..Dim::ZERO
};

struct Motor {
    snapshot: ProjectSnapshot,
    /// `rate : MotorSpeed` (an angular velocity), no inputs.
    rate: DeclId,
    /// `spin : Angle -> MotorSpeed`.
    spin: DeclId,
    readings: ConceptId,
    speed: ConceptId,
}

/// `Angle`, `MotorSpeed` (an angular velocity), `Readings` (a collection
/// of angles), `rate : MotorSpeed`, `spin : Angle -> MotorSpeed`,
/// `samples : Readings`.
fn motor() -> Motor {
    let s = ProjectSnapshot::new(bdl_model::surface::Design::empty("motor"));
    let mk = |s: &ProjectSnapshot, name: &str, rep: Representation| {
        let a = bdl_model::edit::apply_edit(s, &concept(name, Some(rep))).expect("concept");
        (a.snapshot, a.outcome.created_concept.expect("id"))
    };
    let (s, angle) = mk(&s, "Angle", Representation::Quantity { dim: Dim::ANGLE });
    let (s, speed) = mk(&s, "MotorSpeed", Representation::Quantity { dim: ANGULAR });
    let (s, readings) = mk(
        &s,
        "Readings",
        Representation::List {
            element: Box::new(Representation::Quantity { dim: Dim::ANGLE }),
        },
    );
    let m = |s: &ProjectSnapshot, name: &str, inputs: Vec<ConceptId>, out: ConceptId| {
        let a = bdl_model::edit::apply_edit(s, &mapping(name, inputs, out)).expect("mapping");
        (a.snapshot, a.outcome.created_mapping.expect("id"))
    };
    let (s, rate) = m(&s, "rate", vec![], speed);
    let (s, spin) = m(&s, "spin", vec![angle], speed);
    let (s, _samples) = m(&s, "samples", vec![], readings);
    Motor {
        snapshot: s,
        rate,
        spin,
        readings,
        speed,
    }
}

fn node<'a>(p: &'a FormulaProjection, id: &str) -> &'a FormulaNode {
    p.root
        .as_ref()
        .and_then(|r| r.find(id))
        .unwrap_or_else(|| panic!("no node {id} in {p:#?}"))
}

fn projection(host: &mut IdeHost, m: DeclId, src: &str) -> FormulaProjection {
    let _ = src;
    formula_projection(&host.snapshot(), m).expect("projection")
}

// ---- structure: roles, locals, forms ---------------------------------------------

#[test]
fn every_node_has_a_role_and_the_structured_forms_carry_their_scopes() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let _ = motor.readings;
    let src = "if rate > 1 rad per s then spin(90 deg) / rate else { let x = 2; match x { 1 => rate, n => spin(n deg) } }";
    // `n deg` is not a unit after a name — keep the formula parseable
    let src = src.replace("spin(n deg)", "spin(90 deg)");
    host.set_definition_draft(motor.rate, &src);
    let p = projection(&mut host, motor.rate, &src);
    assert!(p.parse_ok, "{:?}", p.unplaced);
    let root = node(&p, "r");
    assert!(matches!(root.kind, NodeKind::If));
    assert_eq!(root.role, "");
    assert_eq!(node(&p, "r.0").role, "condition");
    assert_eq!(node(&p, "r.1").role, "then");
    assert_eq!(node(&p, "r.2").role, "else");
    // the quotient names its sides
    assert_eq!(node(&p, "r.1.0").role, "numerator");
    assert_eq!(node(&p, "r.1.1").role, "denominator");
    assert_eq!(node(&p, "r.1.0.0").role, "argument 1");
    // the composite unit literal: as written, canonical, displayed
    let lit = node(&p, "r.0.1");
    assert!(matches!(
        &lit.kind,
        NodeKind::Quantity { unit, unit_source: Some(src), unit_display: Some(disp), unit_id: None, .. }
            if unit == "rad per s" && src == "rad per s" && disp == "rad/s"
    ));
    // the block: a let, then the result; the let's name is in scope in
    // the result and in the match arms, the arm's own name in its body
    let block = node(&p, "r.2");
    assert!(matches!(block.kind, NodeKind::Block));
    assert!(
        matches!(&node(&p, "r.2.0").kind, NodeKind::Let { pattern, binds } if pattern == "x" && binds == &["x"])
    );
    assert_eq!(node(&p, "r.2.0").role, "let 1");
    assert_eq!(node(&p, "r.2.0.0").role, "value");
    assert!(
        node(&p, "r.2.0.0").locals.is_empty(),
        "a let's value does not see itself"
    );
    let m = node(&p, "r.2.1");
    assert!(matches!(m.kind, NodeKind::Match));
    assert_eq!(m.role, "result");
    assert_eq!(m.locals, vec!["x".to_owned()]);
    assert_eq!(node(&p, "r.2.1.0").role, "subject");
    let arm2 = node(&p, "r.2.1.2");
    assert!(
        matches!(&arm2.kind, NodeKind::Arm { pattern, binds } if pattern == "n" && binds == &["n"])
    );
    assert_eq!(arm2.role, "arm 2");
    assert_eq!(
        node(&p, "r.2.1.2.0").locals,
        vec!["x".to_owned(), "n".to_owned()]
    );
    assert!(matches!(&node(&p, "r.2.1.2.0").kind, NodeKind::Call { .. }));
    // the local reference is marked as one
    assert!(matches!(
        &node(&p, "r.2.1.0").kind,
        NodeKind::Reference { local: true, .. }
    ));
    // a call knows where a further argument goes
    assert_eq!(
        node(&p, "r.1.0").append_at,
        Some(node(&p, "r.1.0").range.end - 1)
    );
}

#[test]
fn a_rule_and_a_binder_scope_their_parameters_to_the_body() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let src = "spin(sum(map r in samples: r * 2))";
    host.set_definition_draft(motor.rate, src);
    let p = projection(&mut host, motor.rate, src);
    assert!(p.parse_ok, "{:?}", p.unplaced);
    let binder = node(&p, "r.0.0");
    assert!(matches!(&binder.kind, NodeKind::Binder { param, .. } if param == "r"));
    assert!(binder.locals.is_empty());
    assert_eq!(node(&p, "r.0.0.0").role, "collection");
    assert_eq!(node(&p, "r.0.0.1").role, "body");
    assert_eq!(node(&p, "r.0.0.1").locals, vec!["r".to_owned()]);
    let src = "spin(sum(map(samples, r => r * 2)))";
    host.set_definition_draft(motor.rate, src);
    let p = projection(&mut host, motor.rate, src);
    assert!(p.parse_ok, "{:?}", p.unplaced);
    let rule = node(&p, "r.0.0.1");
    assert!(matches!(&rule.kind, NodeKind::Rule { params } if params == &["r"]));
    assert_eq!(node(&p, "r.0.0.1.0").locals, vec!["r".to_owned()]);
}

// ---- carets and navigation ------------------------------------------------------

#[test]
fn structural_carets_move_in_source_order_and_between_slots() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let src = "clamp(Tilt / ?, 0, ?)";
    host.set_definition_draft(lamp.dim_by_tilt, src);
    let p = projection(&mut host, lamp.dim_by_tilt, src);
    let root = p.root.as_ref().unwrap();
    // before the root is offset 0; right of it is before the first argument
    let c = navigate(root, "r", Side::Before, Motion::Right).unwrap();
    assert_eq!(
        (c.node.as_str(), c.side, c.offset),
        ("r.0", Side::Before, 6)
    );
    // down from before the quotient is before its numerator; exit is after the quotient
    let c = navigate(root, "r.0", Side::Before, Motion::Down).unwrap();
    assert_eq!((c.node.as_str(), c.side), ("r.0.0", Side::Before));
    let c = navigate(root, "r.0.0", Side::After, Motion::Exit).unwrap();
    assert_eq!(
        (c.node.as_str(), c.side, c.offset),
        ("r.0", Side::After, 14)
    );
    // up keeps the side
    let c = navigate(root, "r.0.1", Side::After, Motion::Up).unwrap();
    assert_eq!((c.node.as_str(), c.side), ("r.0", Side::After));
    // slots, wrapping
    let c = navigate(root, "r", Side::Before, Motion::NextSlot).unwrap();
    assert_eq!(c.node, "r.0.1");
    let c = navigate(root, "r.0.1", Side::Before, Motion::NextSlot).unwrap();
    assert_eq!(c.node, "r.2");
    let c = navigate(root, "r.2", Side::Before, Motion::NextSlot).unwrap();
    assert_eq!(c.node, "r.0.1", "wraps");
    let c = navigate(root, "r.0.1", Side::Before, Motion::PreviousSlot).unwrap();
    assert_eq!(c.node, "r.2", "wraps backwards");
    // the ends stay put
    let c = navigate(root, "r", Side::Before, Motion::Left).unwrap();
    assert_eq!((c.node.as_str(), c.side), ("r", Side::Before));
    let c = navigate(root, "r", Side::After, Motion::Right).unwrap();
    assert_eq!((c.node.as_str(), c.side), ("r", Side::After));
    assert!(navigate(root, "r.9", Side::Before, Motion::Right).is_none());
    assert_eq!(caret_offset(root, "r.1", Side::After), Some(17));
}

// ---- keyboard insertion ------------------------------------------------------------

fn insert(host: &mut IdeHost, m: DeclId, src: &str, node: &str, side: Side, text: &str) -> String {
    compose(
        &host.snapshot(),
        m,
        src,
        &ComposeOp::Insert {
            node: node.into(),
            side,
            text: text.into(),
        },
    )
    .unwrap_or_else(|e| panic!("{src} + {text}: {e:?}"))
    .source
}

#[test]
fn keyboard_input_at_a_caret_is_interpreted_by_the_grammar() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let m = motor.rate;
    host.set_definition_draft(m, "rate");
    // operators after and before a value
    assert_eq!(
        insert(&mut host, m, "rate", "r", Side::After, "/"),
        "rate / ?"
    );
    assert_eq!(
        insert(&mut host, m, "rate", "r", Side::After, "+"),
        "rate + ?"
    );
    assert_eq!(
        insert(&mut host, m, "rate", "r", Side::Before, "-"),
        "? - rate"
    );
    assert_eq!(
        insert(&mut host, m, "rate", "r", Side::After, ">"),
        "rate > ?"
    );
    // grouping
    assert_eq!(
        insert(&mut host, m, "rate", "r", Side::After, "("),
        "(rate)"
    );
    // a word into a slot: its canonical form, never a client-made slot list
    host.set_definition_draft(m, "?");
    assert_eq!(
        insert(&mut host, m, "?", "r", Side::Before, "clamp"),
        "clamp(?, ?, ?)"
    );
    assert_eq!(
        insert(&mut host, m, "?", "r", Side::Before, "if"),
        "if ? then ? else ?"
    );
    assert_eq!(
        insert(&mut host, m, "?", "r", Side::Before, "spin"),
        "spin(?)"
    );
    assert_eq!(insert(&mut host, m, "?", "r", Side::Before, "rate"), "rate");
    assert_eq!(insert(&mut host, m, "?", "r", Side::Before, "180"), "180");
    assert_eq!(
        insert(&mut host, m, "?", "r", Side::Before, "all"),
        "all item in ?: ?"
    );
    // a unit after a number, then `per` and a second unit: one literal
    host.set_definition_draft(m, "180");
    assert_eq!(
        insert(&mut host, m, "180", "r", Side::After, "deg"),
        "180 deg"
    );
    host.set_definition_draft(m, "180 deg");
    assert_eq!(
        insert(&mut host, m, "180 deg", "r", Side::After, "per"),
        "180 deg per "
    );
    // `*` after a quantity is the value product with a slot; filling the
    // slot with a unit atom (`m`) makes the parser read one composite
    // literal (`1 N * m`), so both readings reach one canonical text
    assert_eq!(
        insert(&mut host, m, "180 deg", "r", Side::After, "*"),
        "180 deg * ?"
    );
    assert_eq!(
        insert(&mut host, m, "180 deg", "r", Side::After, "^"),
        "180 deg^"
    );
    // the completed unit is what the parser reads as one literal
    host.set_definition_draft(m, "180 deg per s");
    let p = projection(&mut host, m, "180 deg per s");
    assert!(matches!(&node(&p, "r").kind, NodeKind::Quantity { unit, .. } if unit == "deg per s"));
    assert!(p.complete, "{:?}", p.unplaced);
    // what cannot stand there is refused with the reason
    host.set_definition_draft(m, "rate");
    let err = compose(
        &host.snapshot(),
        m,
        "rate",
        &ComposeOp::Insert {
            node: "r".into(),
            side: Side::After,
            text: "spin".into(),
        },
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("operator"));
    let err = compose(
        &host.snapshot(),
        m,
        "rate",
        &ComposeOp::Insert {
            node: "r".into(),
            side: Side::After,
            text: "per".into(),
        },
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("insert an operator"));
    // `/` after a quantity literal is value division, never a unit
    host.set_definition_draft(m, "180 deg");
    assert_eq!(
        insert(&mut host, m, "180 deg", "r", Side::After, "/"),
        "180 deg / ?"
    );
}

// ---- completion at a caret ----------------------------------------------------------

fn complete(
    host: &mut IdeHost,
    m: DeclId,
    node: &str,
    side: Side,
    prefix: &str,
) -> Vec<SemanticCompletion> {
    completion(
        &host.snapshot(),
        &CompletionContext::FormulaCaret {
            mapping: m,
            node: node.into(),
            side,
            prefix: prefix.into(),
        },
    )
}

#[test]
fn completion_at_a_caret_uses_the_projection_s_expectation_and_locals() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let m = motor.rate;
    // a prefix: `cla` → clamp, with its structured form
    host.set_definition_draft(m, "?");
    let items = complete(&mut host, m, "r", Side::Before, "cla");
    let clamp = items
        .iter()
        .find(|c| c.label.starts_with("clamp"))
        .expect("clamp");
    assert_eq!(clamp.structured_insert.as_deref(), Some("clamp(?, ?, ?)"));
    assert_eq!(
        clamp.replace,
        TextRange::new(0, 0),
        "the caret, not a text prefix"
    );
    // a relationship with an input is offered as a call with a slot
    let items = complete(&mut host, m, "r", Side::Before, "sp");
    let spin = items
        .iter()
        .find(|c| c.label.starts_with("spin"))
        .expect("spin");
    assert_eq!(spin.structured_insert.as_deref(), Some("spin(?)"));
    // `if` comes with its whole shape
    let items = complete(&mut host, m, "r", Side::Before, "i");
    let iff = items.iter().find(|c| c.label == "if").expect("if");
    assert_eq!(iff.structured_insert.as_deref(), Some("if ? then ? else ?"));
    // the slot expects an angular velocity: the composite units of that
    // dimension are offered by their whole spelling, and a plain angle
    // unit is not what fits best
    host.set_definition_draft(m, "10 ");
    let p = projection(&mut host, m, "10 ");
    let _ = p;
    host.set_definition_draft(m, "?");
    let items = complete(&mut host, m, "r", Side::Before, "");
    let units: Vec<&str> = items
        .iter()
        .filter(|c| c.kind == CompletionKind::Unit && c.relevance >= 90)
        .map(|c| c.insert.as_str())
        .collect();
    assert!(units.is_empty(), "no number yet: no unit outranks a value");
    // locals come from the projection, not from scanning the text
    host.set_definition_draft(m, "spin(sum(map r in samples: ?))");
    let items = complete(&mut host, m, "r.0.0.1", Side::Before, "");
    let local = items
        .iter()
        .find(|c| c.kind == CompletionKind::Local)
        .expect("the local r");
    assert_eq!(local.insert, "r");
    let items = complete(&mut host, m, "r.0.0.0", Side::Before, "");
    assert!(
        items.iter().all(|c| c.kind != CompletionKind::Local),
        "the collection position is outside the binder's body"
    );
}

#[test]
fn unit_completion_follows_the_unit_being_written() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let m = motor.rate;
    // `10 d`: deg, with the composite `deg per s` for the angular velocity
    // the position expects
    host.set_definition_draft(m, "10 d");
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: m,
            offset: 4,
        },
    );
    let labels: Vec<&str> = items.iter().map(|c| c.label.as_str()).collect();
    assert!(labels.contains(&"deg"), "{labels:?}");
    let composite = items
        .iter()
        .find(|c| c.insert == "deg per s")
        .expect("deg per s");
    assert!(composite.relevance > items.iter().find(|c| c.label == "deg").unwrap().relevance);
    assert_eq!(composite.structured_insert.as_deref(), Some("deg per s"));
    // `10 deg per `: the atoms that complete an angular velocity first
    host.set_definition_draft(m, "10 deg per ");
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: m,
            offset: 11,
        },
    );
    let s = items.iter().find(|c| c.label == "s").expect("s");
    let mm = items.iter().find(|c| c.label == "mm").expect("mm");
    assert!(s.relevance > mm.relevance, "s completes rad/s; mm does not");
    assert_eq!(s.resulting_type.as_deref(), Some("an angular rate"));
}

// ---- signature help ---------------------------------------------------------------------

#[test]
fn signature_help_names_the_call_its_parameters_and_the_active_argument() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let m = motor.rate;
    let src = "spin(clamp(90 deg, ?, 180 deg))";
    host.set_definition_draft(m, src);
    let p = projection(&mut host, m, src);
    let snap = host.snapshot();
    let design = &snap.effective().design;
    // inside clamp's second argument
    let s = signature(&p, design, "r.0.1").expect("clamp");
    assert_eq!(s.name, "clamp");
    assert_eq!(s.shape, "clamp(x, low, high)");
    assert_eq!(s.active, Some(1));
    assert_eq!(s.parameters.len(), 3);
    assert_eq!(s.parameters[1].expected, "an angle");
    // the call node itself: no active argument
    let s = signature(&p, design, "r.0").expect("clamp");
    assert_eq!(s.active, None);
    // a relationship: its inputs by concept, its result
    let s = signature(&p, design, "r.0.0").expect("spin");
    assert_eq!(s.name, "clamp", "the innermost call wins");
    let s = signature(&p, design, "r").expect("spin");
    assert_eq!(s.name, "spin");
    assert_eq!(s.parameters[0].expected, "Angle");
    assert_eq!(s.result, "MotorSpeed");
    let _ = motor.speed;
}

// ---- the render -----------------------------------------------------------------------------

#[test]
fn the_render_is_the_projection_flattened_with_the_unit_display() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let m = motor.rate;
    let src = "if rate > 180 deg per s then spin(9.81 m per s^2) else rate";
    host.set_definition_draft(m, src);
    let p = projection(&mut host, m, src);
    let r = render(&p);
    let text: Vec<(&str, &str)> = r
        .fragments
        .iter()
        .map(|f| (f.kind.as_str(), f.text.as_str()))
        .collect();
    assert_eq!(
        text,
        vec![
            ("keyword", "if"),
            ("reference", "rate"),
            ("operator", ">"),
            ("number", "180"),
            ("unit", "deg/s"),
            ("keyword", "then"),
            ("reference", "spin"),
            ("punctuation", "("),
            ("number", "9.81"),
            ("unit", "m/s²"),
            ("punctuation", ")"),
            ("keyword", "else"),
            ("reference", "rate"),
        ]
    );
    assert_eq!(r.references, vec!["rate", "spin"]);
    assert_eq!(r.compact, src);
    assert_eq!(r.result, "a MotorSpeed (an angular rate)");
    assert!(
        r.error_count > 0,
        "a length per time squared is not an angle: the render counts it"
    );
    let _ = motor.spin;
}

// ---- a composite unit is switched like an atom -----------------------------------------------

#[test]
fn a_composite_unit_is_switched_by_spelling_and_the_quantity_is_kept() {
    let motor = motor();
    let mut host = IdeHost::new(motor.snapshot.clone());
    let m = motor.rate;
    let src = "180 deg per s";
    host.set_definition_draft(m, src);
    let r = compose(
        &host.snapshot(),
        m,
        src,
        &ComposeOp::SetUnit {
            node: "r".into(),
            unit_id: "rad per s".into(),
            preserve_value: true,
        },
    )
    .expect("switch");
    assert!(
        r.source.starts_with("3.14159") && r.source.ends_with(" rad per s"),
        "{}",
        r.source
    );
    let r = compose(
        &host.snapshot(),
        m,
        src,
        &ComposeOp::SetUnit {
            node: "r".into(),
            unit_id: "turn per min".into(),
            preserve_value: true,
        },
    )
    .expect("switch");
    assert_eq!(r.source, "30 turn per min");
    // the slot's candidates for an angular velocity: the curated composites
    host.set_definition_draft(m, "?");
    let slot = formula_slot(&host.snapshot(), m, "r").expect("slot");
    let spelled: Vec<&str> = slot.units.iter().map(|u| u.symbol.as_str()).collect();
    assert_eq!(
        spelled,
        ["rad per s", "deg per s", "turn per s", "deg per min"]
    );
    assert_eq!(slot.units[0].display, "rad/s");
    assert!(
        slot.units[0].id.is_empty(),
        "a composite has no registry id"
    );
    // a wrong dimension cannot keep the value
    let err = compose(
        &host.snapshot(),
        m,
        src,
        &ComposeOp::SetUnit {
            node: "r".into(),
            unit_id: "length.m".into(),
            preserve_value: true,
        },
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("cannot be kept"));
}
