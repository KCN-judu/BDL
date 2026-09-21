//! The Formula Composer's service (docs/architecture/ide-service.md
//! §Formula projection): the projection's shape, expected dimensions by
//! local inference, unit candidates sound for the slot's dimension,
//! reference and equation candidates by type, stale generations, stable
//! ranges, and the text edits structured actions make.

mod support;

use bdl_ide::*;
use bdl_model::edit::EditOp;
use bdl_model::surface::{ProjectSnapshot, Representation};
use bdl_model::{ConceptId, DeclId, Dim};
use support::*;

const FORCE: Dim = Dim {
    mass: 1,
    length: 1,
    time: -2,
    ..Dim::ZERO
};
const TORQUE: Dim = Dim {
    mass: 1,
    length: 2,
    time: -2,
    ..Dim::ZERO
};
const SPEED: Dim = Dim {
    length: 1,
    time: -1,
    ..Dim::ZERO
};

struct Physics {
    snapshot: ProjectSnapshot,
    speed_of: DeclId,
    torque_of: DeclId,
    length: ConceptId,
}

/// `Length`, `Time`, `Force`, `Speed`, `Torque` and three length-valued
/// relationships; `speedOf : Length -> Speed`, `torqueOf : Force -> Torque`.
fn physics() -> Physics {
    let s = ProjectSnapshot::new(bdl_model::surface::Design::empty("physics"));
    let mk = |s: &ProjectSnapshot, name: &str, dim: Dim| {
        let a =
            bdl_model::edit::apply_edit(s, &concept(name, Some(Representation::Quantity { dim })))
                .expect("concept");
        (a.snapshot, a.outcome.created_concept.expect("id"))
    };
    let (s, length) = mk(&s, "Length", Dim::LENGTH);
    let (s, time) = mk(&s, "Time", Dim::TIME);
    let (s, force) = mk(&s, "Force", FORCE);
    let (s, speed) = mk(&s, "Speed", SPEED);
    let (s, torque) = mk(&s, "Torque", TORQUE);
    let (s, mass) = mk(&s, "Mass", Dim::MASS);
    let m = |s: &ProjectSnapshot, name: &str, inputs: Vec<ConceptId>, out: ConceptId| {
        let a = bdl_model::edit::apply_edit(s, &mapping(name, inputs, out)).expect("mapping");
        (a.snapshot, a.outcome.created_mapping.expect("id"))
    };
    let (s, _arm) = m(&s, "armLength", vec![], length);
    let (s, _wheel) = m(&s, "wheelRadius", vec![], length);
    let (s, _cycle) = m(&s, "cycleTime", vec![], time);
    let (s, _load) = m(&s, "load", vec![], mass);
    let (s, speed_of) = m(&s, "speedOf", vec![length], speed);
    let (s, torque_of) = m(&s, "torqueOf", vec![force], torque);
    let (s, _limit) = m(&s, "speedLimit", vec![], speed);
    Physics {
        snapshot: s,
        speed_of,
        torque_of,
        length,
    }
}

fn node<'a>(p: &'a FormulaProjection, id: &str) -> &'a FormulaNode {
    p.root
        .as_ref()
        .and_then(|r| r.find(id))
        .unwrap_or_else(|| panic!("no node {id} in {p:#?}"))
}

fn dim_of(t: &Option<TypeView>) -> Option<Dim> {
    t.as_ref().and_then(|t| t.dim)
}

// ---- projection shape -------------------------------------------------------------

#[test]
fn the_projection_is_the_surface_tree_with_types_expected_types_and_ranges() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(lamp.dim_by_tilt, "clamp(Tilt / (90 deg), 0, 1)");
    let snap = host.snapshot();
    let p = formula_projection(&snap, lamp.dim_by_tilt).expect("projection");
    assert!(p.parse_ok && p.complete && p.slots.is_empty());
    assert_eq!(p.draft_generation, Some(snap.generation()));
    assert_eq!(
        p.result.as_ref().map(|t| t.concept),
        Some(Some(lamp.brightness))
    );

    let root = node(&p, "r");
    assert!(matches!(&root.kind, NodeKind::Call { name, equation: true, .. } if name == "clamp"));
    assert_eq!(root.range, TextRange::new(0, p.source.len() as u32));
    assert_eq!(root.children.len(), 3);
    // the result of the whole formula is the output concept's kind
    assert_eq!(dim_of(&root.expected), Some(Dim::ZERO));
    assert_eq!(dim_of(&root.actual), Some(Dim::ZERO));

    let quotient = node(&p, "r.0");
    assert!(matches!(&quotient.kind, NodeKind::Binary { op } if op == "/"));
    assert_eq!(quotient.text, "Tilt / (90 deg)");
    let tilt = node(&p, "r.0.0");
    assert!(
        matches!(&tilt.kind, NodeKind::Reference { name, entity: Some(EntityRef::Concept(c)), .. } if name == "Tilt" && *c == lamp.tilt)
    );
    assert_eq!(
        tilt.actual.as_ref().map(|t| t.kind),
        Some(TypeKindView::Concept)
    );
    assert_eq!(dim_of(&tilt.actual), Some(Dim::ANGLE));
    // the denominator: a quantity literal with its unit, parentheses in its range
    let ninety = node(&p, "r.0.1");
    assert!(
        matches!(&ninety.kind, NodeKind::Quantity { coordinate, unit, unit_id: Some(id), .. } if coordinate == "90" && unit == "deg" && id == "angle.deg")
    );
    assert_eq!(
        &p.source[ninety.range.start as usize..ninety.range.end as usize],
        "(90 deg)"
    );
    assert_eq!(dim_of(&ninety.expected), Some(Dim::ANGLE));
    assert!(
        ninety
            .because
            .contains("an angle ÷ an angle = a dimensionless quantity"),
        "{}",
        ninety.because
    );
    // clamp's bounds expect what clamp's value is
    assert_eq!(dim_of(&node(&p, "r.1").expected), Some(Dim::ZERO));
    assert!(node(&p, "r.1")
        .because
        .contains("clamp(x, low, high) takes low"));
    // stamps and identities: the same draft again gives the same tree
    let again = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert_eq!(again.root, p.root);
}

#[test]
fn unsupported_forms_are_opaque_regions_and_parse_failures_have_no_tree() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    // a match is structured (0.27): the subject, then one arm per case,
    // each with its pattern and its body
    host.set_definition_draft(lamp.dim_by_tilt, "match Tilt { _ => 0 }");
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(matches!(&node(&p, "r").kind, NodeKind::Match));
    assert_eq!(node(&p, "r").children.len(), 2);
    assert_eq!(node(&p, "r.0").role, "subject");
    assert!(matches!(&node(&p, "r.1").kind, NodeKind::Arm { pattern, .. } if pattern == "_"));
    assert_eq!(node(&p, "r.1.0").role, "body");
    assert!(p.complete, "a structured form is still a valid formula");
    // the one form that stays opaque: the empty product
    host.set_definition_draft(lamp.dim_by_tilt, "dimByTilt(())");
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(
        matches!(&node(&p, "r.0").kind, NodeKind::Opaque { what } if what.contains("empty product"))
    );
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt / (");
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(!p.parse_ok && p.root.is_none() && !p.complete);
    assert!(p
        .unplaced
        .iter()
        .any(|d| d.code.starts_with("formula.parse")));
    // an empty definition: no tree, one result to produce
    host.clear_definition_draft(lamp.dim_by_tilt);
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(p.parse_ok && p.root.is_none() && !p.complete && p.draft_generation.is_none());
}

// ---- local inference ---------------------------------------------------------------

#[test]
fn a_slot_takes_its_dimension_from_the_operator_and_the_known_side() {
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    // ? / 1 s = Speed ⇒ Length
    host.set_definition_draft(ph.speed_of, "? / 1 s");
    let p = formula_projection(&host.snapshot(), ph.speed_of).expect("projection");
    assert_eq!(p.slots, vec!["r.0".to_string()]);
    assert!(!p.complete);
    assert_eq!(dim_of(&node(&p, "r.0").expected), Some(Dim::LENGTH));
    assert!(node(&p, "r.0")
        .diagnostics
        .iter()
        .any(|d| d.code == "formula.slot.empty"));
    // Length / ? = Speed ⇒ Time
    host.set_definition_draft(ph.speed_of, "Length / ?");
    let p = formula_projection(&host.snapshot(), ph.speed_of).expect("projection");
    assert_eq!(dim_of(&node(&p, "r.1").expected), Some(Dim::TIME));
    // Force * ? = Torque ⇒ Length, and ? * Force too
    host.set_definition_draft(ph.torque_of, "Force * ?");
    let p = formula_projection(&host.snapshot(), ph.torque_of).expect("projection");
    assert_eq!(dim_of(&node(&p, "r.1").expected), Some(Dim::LENGTH));
    host.set_definition_draft(ph.torque_of, "? * Force");
    let p = formula_projection(&host.snapshot(), ph.torque_of).expect("projection");
    assert_eq!(dim_of(&node(&p, "r.0").expected), Some(Dim::LENGTH));
    // sums propagate the result to both sides
    host.set_definition_draft(ph.torque_of, "? + ?");
    let p = formula_projection(&host.snapshot(), ph.torque_of).expect("projection");
    assert_eq!(dim_of(&node(&p, "r.0").expected), Some(TORQUE));
    assert_eq!(dim_of(&node(&p, "r.1").expected), Some(TORQUE));
    assert_eq!(p.slots, vec!["r.0".to_string(), "r.1".to_string()]);
    // nested: (? / 1 s) * Force = Torque ⇒ the quotient is Length, its numerator Length·Time
    host.set_definition_draft(ph.torque_of, "(? / 1 s) * Force");
    let p = formula_projection(&host.snapshot(), ph.torque_of).expect("projection");
    assert_eq!(dim_of(&node(&p, "r.0").expected), Some(Dim::LENGTH));
    assert_eq!(
        dim_of(&node(&p, "r.0.0").expected),
        Some(Dim::LENGTH + Dim::TIME)
    );
}

#[test]
fn a_product_of_two_unknowns_is_insufficient_information_not_a_guess() {
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    host.set_definition_draft(ph.torque_of, "? * ?");
    let snap = host.snapshot();
    let p = formula_projection(&snap, ph.torque_of).expect("projection");
    assert_eq!(node(&p, "r.0").expected, None);
    assert_eq!(node(&p, "r.1").expected, None);
    let slot = formula_slot(&snap, ph.torque_of, "r.0").expect("slot");
    assert!(slot.insufficient);
    assert!(slot.units.is_empty(), "nothing is offered by dimension");
    assert!(
        slot.explanation.starts_with("Not enough is known yet"),
        "{}",
        slot.explanation
    );
    // the whole product still knows what it must be
    assert_eq!(dim_of(&node(&p, "r").expected), Some(TORQUE));
}

// ---- candidates --------------------------------------------------------------------

#[test]
fn unit_candidates_are_exactly_the_registered_units_of_the_slots_dimension() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt / ?");
    let snap = host.snapshot();
    let slot = formula_slot(&snap, lamp.dim_by_tilt, "r.1").expect("slot");
    assert_eq!(dim_of(&slot.expected), Some(Dim::ANGLE));
    let symbols: Vec<&str> = slot.units.iter().map(|u| u.symbol.as_str()).collect();
    assert_eq!(symbols, vec!["rad", "deg", "turn"]);
    assert!(
        slot.explanation
            .contains("Expected: an angle, because an angle ÷ an angle = a dimensionless quantity"),
        "{}",
        slot.explanation
    );
    assert!(slot.technical.contains("q[") || slot.technical.contains("expected"));
    // every registered angle unit, and nothing of another dimension
    for u in bdl_elab::units::UNITS {
        assert_eq!(
            slot.units.iter().any(|c| c.id == u.id),
            u.dim == Dim::ANGLE,
            "{}",
            u.symbol
        );
    }
    // a literal's own pop-up offers the units of *its* dimension — the
    // switch keeps the quantity — even where the position expects another
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt + 90 mm");
    let slot = formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r.1").expect("slot");
    // (a sum's sides must be the result: dimensionless, like Brightness)
    assert_eq!(dim_of(&slot.expected), Some(Dim::ZERO));
    assert!(
        slot.units.iter().all(|u| u.measures == "a length"),
        "{:?}",
        slot.units
    );
    // a length slot offers the length units, with inch and ft
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    host.set_definition_draft(ph.speed_of, "? / 1 s");
    let slot = formula_slot(&host.snapshot(), ph.speed_of, "r.0").expect("slot");
    let symbols: Vec<&str> = slot.units.iter().map(|u| u.symbol.as_str()).collect();
    assert_eq!(symbols, vec!["m", "mm", "cm", "km", "inch", "ft"]);
}

#[test]
fn reference_candidates_come_from_the_design_by_type_never_by_name() {
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    host.set_definition_draft(ph.speed_of, "? / 1 s");
    let slot = formula_slot(&host.snapshot(), ph.speed_of, "r.0").expect("slot");
    let labels: Vec<&str> = slot.references.iter().map(|r| r.label.as_str()).collect();
    // the input Length first, then the two length-valued relationships;
    // cycleTime (a time) and load (a mass) are not offered
    assert_eq!(labels, vec!["Length", "armLength", "wheelRadius"]);
    assert_eq!(
        slot.references[0].entity,
        Some(EntityRef::Concept(ph.length))
    );
    assert!(slot
        .references
        .iter()
        .all(|r| r.produces.contains("Length")));
    // a callable relationship is offered with a slot per input when its
    // result fits; the mapping being defined is never offered to itself
    let limit = ph
        .snapshot
        .design
        .mappings
        .values()
        .find(|m| m.name == "speedLimit")
        .expect("speedLimit")
        .id;
    host.set_definition_draft(limit, "? + ?");
    let slot = formula_slot(&host.snapshot(), limit, "r.0").expect("slot");
    let call = slot
        .references
        .iter()
        .find(|r| r.label == "speedOf(…)")
        .expect("speedOf");
    assert_eq!(call.insert, "speedOf(?)");
    assert!(!slot
        .references
        .iter()
        .any(|r| r.label == "armLength" || r.label == "speedLimit"));
}

#[test]
fn equation_candidates_are_the_schemes_whose_result_fits_and_whose_capabilities_hold() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt / ?");
    let slot = formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r.1").expect("slot");
    let names: Vec<&str> = slot.equations.iter().map(|e| e.name.as_str()).collect();
    for want in ["min", "max", "clamp", "sum", "getOrElse", "id"] {
        assert!(names.contains(&want), "{want} missing from {names:?}");
    }
    // a truth-valued equation does not produce an angle
    for not in ["any", "all", "contains", "inRange"] {
        assert!(!names.contains(&not), "{not} offered for an angle");
    }
    let clamp = slot
        .equations
        .iter()
        .find(|e| e.name == "clamp")
        .expect("clamp");
    assert_eq!(clamp.insert, "clamp(?, ?, ?)");
    assert_eq!(clamp.shape, "clamp(x, low, high)");
    // a truth-valued position — a choice's condition — offers the
    // truth-valued equations and no angle-valued one
    host.set_definition_draft(lamp.dim_by_tilt, "if ? then 1 else 0");
    let slot = formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r.0").expect("slot");
    let names: Vec<&str> = slot.equations.iter().map(|e| e.name.as_str()).collect();
    for want in ["any", "all", "contains", "inRange"] {
        assert!(names.contains(&want), "{want} missing from {names:?}");
    }
    for not in ["min", "max", "clamp", "sum"] {
        assert!(!names.contains(&not), "{not} offered for a truth value");
    }
}

// ---- generations and stability --------------------------------------------------------

#[test]
fn a_projection_carries_its_generation_and_a_newer_draft_supersedes_it() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt / ?");
    let old = host.snapshot();
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt / (90 deg)");
    let new = host.snapshot();
    let p_old = formula_projection(&old, lamp.dim_by_tilt).expect("old");
    let p_new = formula_projection(&new, lamp.dim_by_tilt).expect("new");
    assert!(p_old.draft_generation < p_new.draft_generation);
    assert!(p_old.stamp != p_new.stamp);
    assert_eq!(p_old.source, "Tilt / ?");
    assert_eq!(p_new.source, "Tilt / (90 deg)");
    // ranges are byte offsets into the projection's own source
    let n = node(&p_new, "r.1");
    assert_eq!(
        &p_new.source[n.range.start as usize..n.range.end as usize],
        "(90 deg)"
    );
    // an unchanged prefix keeps its ids and ranges across generations
    assert_eq!(node(&p_old, "r.0").range, node(&p_new, "r.0").range);
    assert_eq!(node(&p_old, "r.0").id, node(&p_new, "r.0").id);
}

#[test]
fn diagnostics_are_placed_on_the_innermost_node_and_kept_as_text_ranges() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(lamp.dim_by_tilt, "Tilt + (90 mm)");
    let snap = host.snapshot();
    let p = formula_projection(&snap, lamp.dim_by_tilt).expect("projection");
    let verdict = draft_verdict(&snap, lamp.dim_by_tilt).expect("verdict");
    let text_codes: Vec<&str> = verdict
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert!(text_codes.contains(&"dimension.mismatch"), "{text_codes:?}");
    // the same diagnostic, on the node the compiler's span falls in
    let mut placed = Vec::new();
    fn collect<'a>(n: &'a FormulaNode, out: &mut Vec<(&'a str, &'a str)>) {
        for d in &n.diagnostics {
            out.push((n.id.as_str(), d.code.as_str()));
        }
        n.children.iter().for_each(|c| collect(c, out));
    }
    collect(p.root.as_ref().expect("tree"), &mut placed);
    assert!(
        placed.iter().any(|(_, c)| *c == "dimension.mismatch"),
        "{placed:?}"
    );
    let total = placed.len() + p.unplaced.len();
    assert_eq!(
        total,
        verdict.diagnostics.len(),
        "one diagnostic, two projections"
    );
}

// ---- composing ---------------------------------------------------------------------

fn composed(host: &mut IdeHost, m: DeclId, source: &str, op: ComposeOp) -> ComposeResult {
    compose(&host.snapshot(), m, source, &op).expect("compose")
}

#[test]
fn the_tilt_to_brightness_walkthrough_is_a_sequence_of_text_edits() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    // 1. an empty formula is one slot; select Tilt
    let r = composed(
        &mut host,
        m,
        "",
        ComposeOp::Fill {
            node: "r".into(),
            text: "Tilt".into(),
        },
    );
    assert_eq!(r.source, "Tilt");
    assert_eq!(r.edits[0].range, TextRange::new(0, 0));
    // 2. insert Divide
    let r = composed(
        &mut host,
        m,
        "Tilt",
        ComposeOp::Operator {
            node: "r".into(),
            op: "/".into(),
            before: false,
        },
    );
    assert_eq!(r.source, "Tilt / ?");
    assert_eq!(r.select.as_deref(), Some("r.1"));
    // 3–7. the slot is an angle: a literal, deg, 90
    let r = composed(
        &mut host,
        m,
        "Tilt / ?",
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "1".into(),
        },
    );
    assert_eq!(r.source, "Tilt / 1");
    let r = composed(
        &mut host,
        m,
        "Tilt / 1",
        ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "angle.deg".into(),
            preserve_value: true,
        },
    );
    assert_eq!(r.source, "Tilt / 1 deg");
    let r = composed(
        &mut host,
        m,
        "Tilt / 1 deg",
        ComposeOp::SetCoordinate {
            node: "r.1".into(),
            text: "90".into(),
        },
    );
    assert_eq!(r.source, "Tilt / 90 deg");
    // 9. wrap in clamp
    let r = composed(
        &mut host,
        m,
        "Tilt / 90 deg",
        ComposeOp::Call {
            node: "r".into(),
            name: "clamp".into(),
            arity: 3,
        },
    );
    assert_eq!(r.source, "clamp(Tilt / 90 deg, ?, ?)");
    assert_eq!(r.select.as_deref(), Some("r.1"));
    // 10. enter 0 and 1
    let r = composed(
        &mut host,
        m,
        &r.source,
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "0".into(),
        },
    );
    assert_eq!(r.select.as_deref(), Some("r.2"));
    let r = composed(
        &mut host,
        m,
        &r.source,
        ComposeOp::Fill {
            node: "r.2".into(),
            text: "1".into(),
        },
    );
    assert_eq!(r.source, "clamp(Tilt / 90 deg, 0, 1)");
    assert_eq!(
        r.select.as_deref(),
        Some("r.2"),
        "no slot left: the edited node"
    );
    // 11. the compiler reports it compatible with Brightness
    host.set_definition_draft(m, &r.source);
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    assert!(p.complete);
    assert_eq!(dim_of(&node(&p, "r").actual), Some(Dim::ZERO));
}

#[test]
fn operators_parenthesise_what_binds_weaker_and_removing_a_slot_removes_its_operator() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    // a sum under a division needs parentheses; a product does not
    let r = composed(
        &mut host,
        m,
        "Tilt + 1 deg",
        ComposeOp::Operator {
            node: "r".into(),
            op: "/".into(),
            before: false,
        },
    );
    assert_eq!(r.source, "(Tilt + 1 deg) / ?");
    let r = composed(
        &mut host,
        m,
        "Tilt * 2",
        ComposeOp::Operator {
            node: "r".into(),
            op: "/".into(),
            before: false,
        },
    );
    assert_eq!(r.source, "Tilt * 2 / ?");
    // filling a slot with a sum under a product parenthesises the sum
    let r = composed(
        &mut host,
        m,
        "Tilt * ?",
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "1 + 2".into(),
        },
    );
    assert_eq!(r.source, "Tilt * (1 + 2)");
    // `? op node`
    let r = composed(
        &mut host,
        m,
        "Tilt",
        ComposeOp::Operator {
            node: "r".into(),
            op: "-".into(),
            before: true,
        },
    );
    assert_eq!(r.source, "? - Tilt");
    assert_eq!(r.select.as_deref(), Some("r.0"));
    // removing an operand slot removes the operator; removing a value leaves a slot
    let r = composed(
        &mut host,
        m,
        "Tilt / ?",
        ComposeOp::Remove { node: "r.1".into() },
    );
    assert_eq!(r.source, "Tilt");
    let r = composed(
        &mut host,
        m,
        "Tilt / 90 deg",
        ComposeOp::Remove { node: "r.1".into() },
    );
    assert_eq!(r.source, "Tilt / ?");
    // a value replaced keeps its parentheses' place
    let r = composed(
        &mut host,
        m,
        "Tilt / (90 deg)",
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "45 deg".into(),
        },
    );
    assert_eq!(r.source, "Tilt / 45 deg");
}

#[test]
fn switching_a_literals_unit_preserves_the_quantity_unless_told_otherwise() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    let r = composed(
        &mut host,
        m,
        "Tilt / 180 deg",
        ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "angle.rad".into(),
            preserve_value: true,
        },
    );
    assert_eq!(r.source, "Tilt / 3.141592653589793 rad");
    let r = composed(
        &mut host,
        m,
        "Tilt / 180 deg",
        ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "angle.rad".into(),
            preserve_value: false,
        },
    );
    assert_eq!(r.source, "Tilt / 180 rad");
    let r = composed(
        &mut host,
        m,
        "Tilt / 1 turn",
        ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "angle.deg".into(),
            preserve_value: true,
        },
    );
    assert_eq!(r.source, "Tilt / 360 deg");
    // a unit of another dimension cannot keep the value: refused, never silently reinterpreted
    let err = compose(
        &host.snapshot(),
        m,
        "Tilt / 180 deg",
        &ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "length.mm".into(),
            preserve_value: true,
        },
    )
    .expect_err("refused");
    assert!(matches!(err, QueryError::NotApplicable { .. }));
    // the same edits as byte ranges against the draft
    let r = composed(
        &mut host,
        m,
        "Tilt / 180 deg",
        ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "angle.turn".into(),
            preserve_value: true,
        },
    );
    assert_eq!(r.edits.len(), 1);
    assert_eq!(r.edits[0].range, TextRange::new(7, 14));
    assert_eq!(r.edits[0].new_text, "0.5 turn");
}

#[test]
fn a_slot_on_a_project_with_a_scoped_or_unknown_mapping_never_panics() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let unknown = DeclId::from_raw(4242);
    assert!(formula_projection(&host.snapshot(), unknown).is_err());
    assert!(formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r.9.9").is_err());
    host.set_definition_draft(lamp.dim_by_tilt, "(((");
    // without a tree the root position still says what the formula must produce
    let root = formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r").expect("root");
    assert_eq!(
        root.expected.as_ref().and_then(|t| t.concept),
        Some(lamp.brightness)
    );
    assert!(formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r.0").is_err());
    assert!(compose(
        &host.snapshot(),
        lamp.dim_by_tilt,
        "(((",
        &ComposeOp::Remove { node: "r".into() }
    )
    .is_err());
    // a committed formula projects without a draft
    let s = edit(&lamp.snapshot, formula(lamp.dim_by_tilt, "Tilt / 90 deg"));
    let mut host = IdeHost::new(s);
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(p.complete && p.draft_generation.is_none());
    let _ = EditOp::DeleteConcept { id: lamp.tilt };
}

// ---- P10b hardening ------------------------------------------------------------------

/// A relationship's argument is nominal: `dimByTilt(?)` reads a Tilt, so a
/// concept of the same representation (`Yaw : Angle`) is never offered,
/// nor is a plain angle-valued relationship; the formula's *result*
/// position observes any value of the representation (ADR-0013), so
/// there both fit.
#[test]
fn dimension_equality_never_admits_a_nominally_wrong_reference() {
    let lamp = lamp();
    let s = edit(
        &lamp.snapshot,
        concept("Yaw", Some(Representation::Quantity { dim: Dim::ANGLE })),
    );
    let yaw = s
        .design
        .concepts
        .values()
        .find(|c| c.name == "Yaw")
        .expect("Yaw")
        .id;
    let s = edit(&s, mapping("heading", vec![], yaw));
    let s = edit(&s, mapping("spin", vec![], lamp.tilt));
    let s = edit(&s, mapping("level", vec![], lamp.brightness));
    let level = s
        .design
        .mappings
        .values()
        .find(|m| m.name == "level")
        .expect("level")
        .id;
    let mut host = IdeHost::new(s);
    // the argument of a relationship: only Tilt values
    host.set_definition_draft(level, "dimByTilt(?)");
    let slot = formula_slot(&host.snapshot(), level, "r.0").expect("slot");
    assert!(
        slot.expected
            .as_ref()
            .is_some_and(|t| t.nominal && t.concept == Some(lamp.tilt)),
        "{:?}",
        slot.expected
    );
    let labels: Vec<&str> = slot.references.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(
        labels,
        vec!["spin"],
        "heading (a Yaw) is an angle but not a Tilt"
    );
    // an equation's argument bound to a concept by the other argument
    host.set_definition_draft(level, "min(spin, ?)");
    let slot = formula_slot(&host.snapshot(), level, "r.1").expect("slot");
    let labels: Vec<&str> = slot.references.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, vec!["spin"]);
    // units of the dimension are still offered: a literal is observed with it
    assert_eq!(
        slot.units
            .iter()
            .map(|u| u.symbol.as_str())
            .collect::<Vec<_>>(),
        vec!["rad", "deg", "turn"]
    );
    // the result position: any dimensionless value, ranked — Brightness first
    host.set_definition_draft(level, "?");
    let slot = formula_slot(&host.snapshot(), level, "r").expect("slot");
    assert!(slot.expected.as_ref().is_some_and(|t| !t.nominal));
    assert!(slot.references.iter().any(|r| r.label == "dimByTilt(…)"));
    assert!(
        !slot.references.iter().any(|r| r.label == "spin"),
        "an angle is not dimensionless"
    );
}

/// Only a literal owns a unit: a reference, a slot, a call or an operator
/// never takes one, and a literal's unit switch keeps the quantity by
/// default while its coordinate edit changes it.
#[test]
fn only_a_quantity_literal_has_an_editable_unit() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    host.set_definition_draft(m, "clamp(Tilt / 90 deg, ?, 1)");
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    // the projection: a unit only on the literal
    fn units(n: &FormulaNode, out: &mut Vec<(String, String)>) {
        if let NodeKind::Quantity { unit, .. } = &n.kind {
            out.push((n.id.clone(), unit.clone()));
        }
        n.children.iter().for_each(|c| units(c, out));
    }
    let mut found = Vec::new();
    units(p.root.as_ref().unwrap(), &mut found);
    assert_eq!(found, vec![("r.0.1".to_string(), "deg".to_string())]);
    assert!(matches!(node(&p, "r.0.0").kind, NodeKind::Reference { .. }));
    // SetUnit on anything but a number is refused
    for id in ["r", "r.0", "r.0.0", "r.1"] {
        let err = compose(
            &host.snapshot(),
            m,
            "clamp(Tilt / 90 deg, ?, 1)",
            &ComposeOp::SetUnit {
                node: id.into(),
                unit_id: "angle.rad".into(),
                preserve_value: true,
            },
        )
        .expect_err(id);
        assert!(matches!(err, QueryError::NotApplicable { .. }), "{id}");
    }
    // a slot: no unit until a number is written; a bare number takes one
    let r = composed(
        &mut host,
        m,
        "clamp(Tilt / 90 deg, ?, 1)",
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "0".into(),
        },
    );
    assert_eq!(r.source, "clamp(Tilt / 90 deg, 0, 1)");
    // the coordinate edit is a different quantity in the same unit
    let r = composed(
        &mut host,
        m,
        "Tilt / 90 deg",
        ComposeOp::SetCoordinate {
            node: "r.1".into(),
            text: "45".into(),
        },
    );
    assert_eq!(r.source, "Tilt / 45 deg");
    // the unit switch keeps the quantity (the default of every picker)
    let r = composed(
        &mut host,
        m,
        "Tilt / 90 deg",
        ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "angle.turn".into(),
            preserve_value: true,
        },
    );
    assert_eq!(r.source, "Tilt / 0.25 turn");
    // a unit switch never lands on an affine chart: none is registered
    let err = compose(
        &host.snapshot(),
        m,
        "Tilt / 90 deg",
        &ComposeOp::SetUnit {
            node: "r.1".into(),
            unit_id: "temperature.celsius".into(),
            preserve_value: true,
        },
    )
    .expect_err("affine");
    assert!(matches!(err, QueryError::NotApplicable { .. }));
}

/// Precedence: every composed source means what the structure meant.
#[test]
fn composed_sources_keep_the_intended_precedence() {
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    let m = ph.torque_of;
    let cases: &[(&str, ComposeOp, &str)] = &[
        // wrapping a sum under a product parenthesises the sum
        (
            "a + b * c",
            ComposeOp::Operator {
                node: "r".into(),
                op: "*".into(),
                before: false,
            },
            "(a + b * c) * ?",
        ),
        // wrapping only the product: the product is a child of the sum, and
        // the new sum on the right side of a sum is grouped (`a + (x + ?)`)
        (
            "a + b * c",
            ComposeOp::Operator {
                node: "r.1".into(),
                op: "/".into(),
                before: false,
            },
            "a + b * c / ?",
        ),
        (
            "a + b * c",
            ComposeOp::Operator {
                node: "r.1".into(),
                op: "+".into(),
                before: false,
            },
            "a + (b * c + ?)",
        ),
        (
            "a + b * c",
            ComposeOp::Operator {
                node: "r.0".into(),
                op: "-".into(),
                before: false,
            },
            "a - ? + b * c",
        ),
        // an explicit group stays a group
        (
            "(a + b) * c",
            ComposeOp::Operator {
                node: "r.0".into(),
                op: "-".into(),
                before: false,
            },
            "((a + b) - ?) * c",
        ),
        // a quotient of a quotient keeps its nesting
        (
            "a / (b / c)",
            ComposeOp::Operator {
                node: "r".into(),
                op: "/".into(),
                before: true,
            },
            "? / (a / (b / c))",
        ),
        (
            "a / (b / c)",
            ComposeOp::Fill {
                node: "r.1".into(),
                text: "b * c".into(),
            },
            "a / (b * c)",
        ),
        (
            "a / (b / c)",
            ComposeOp::Fill {
                node: "r.1".into(),
                text: "b".into(),
            },
            "a / b",
        ),
        // unary minus binds tighter than any operator
        (
            "-a",
            ComposeOp::Operator {
                node: "r".into(),
                op: "*".into(),
                before: false,
            },
            "-a * ?",
        ),
        (
            "a * ?",
            ComposeOp::Fill {
                node: "r.1".into(),
                text: "-b".into(),
            },
            "a * -b",
        ),
        // a comparison under arithmetic is grouped; arithmetic under a comparison is not
        (
            "a < b",
            ComposeOp::Operator {
                node: "r".into(),
                op: "+".into(),
                before: false,
            },
            "(a < b) + ?",
        ),
        (
            "a + b",
            ComposeOp::Operator {
                node: "r".into(),
                op: "<".into(),
                before: false,
            },
            "a + b < ?",
        ),
        (
            "a < ?",
            ComposeOp::Fill {
                node: "r.1".into(),
                text: "b + c".into(),
            },
            "a < b + c",
        ),
        (
            "a * ?",
            ComposeOp::Fill {
                node: "r.1".into(),
                text: "b < c".into(),
            },
            "a * (b < c)",
        ),
        // nested calls: an argument is never parenthesised, a call is an atom
        (
            "min(a, b)",
            ComposeOp::Operator {
                node: "r".into(),
                op: "*".into(),
                before: false,
            },
            "min(a, b) * ?",
        ),
        (
            "min(a, ?)",
            ComposeOp::Fill {
                node: "r.1".into(),
                text: "b + c".into(),
            },
            "min(a, b + c)",
        ),
        (
            "min(a, b)",
            ComposeOp::Call {
                node: "r.1".into(),
                name: "max".into(),
                arity: 2,
            },
            "min(a, max(b, ?))",
        ),
        (
            "a * b",
            ComposeOp::Call {
                node: "r".into(),
                name: "clamp".into(),
                arity: 3,
            },
            "clamp(a * b, ?, ?)",
        ),
    ];
    for (source, op, want) in cases {
        let r =
            compose(&host.snapshot(), m, source, op).unwrap_or_else(|e| panic!("{source}: {e:?}"));
        assert_eq!(&r.source, want, "{source} + {op:?}");
        // and the result parses to what the text says
        assert!(bdl_syntax::formula(&r.source).is_ok(), "{}", r.source);
    }
    let _ = &mut host;
}

/// A structured form keeps its exact source, is selectable as a whole,
/// and the structure around it stays editable.
#[test]
fn opaque_forms_keep_their_source_and_the_structure_around_them_stays_editable() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    let src = "(match Tilt { _ => 1 }) * ?";
    host.set_definition_draft(m, src);
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    let opaque = node(&p, "r.0");
    assert!(matches!(&opaque.kind, NodeKind::Match));
    assert_eq!(
        &src[opaque.range.start as usize..opaque.range.end as usize],
        "(match Tilt { _ => 1 })"
    );
    assert_eq!(opaque.children.len(), 2);
    assert_eq!(
        dim_of(&node(&p, "r.1").expected),
        Some(Dim::ZERO),
        "the slot beside it is still inferred"
    );
    // the slot beside it is filled; the opaque text is untouched, byte for byte
    let r = composed(
        &mut host,
        m,
        src,
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "2".into(),
        },
    );
    assert_eq!(r.source, "(match Tilt { _ => 1 }) * 2");
    // the opaque node itself can be replaced or removed as a whole
    let r = composed(&mut host, m, src, ComposeOp::Remove { node: "r.0".into() });
    assert_eq!(r.source, "? * ?");
    let r = composed(
        &mut host,
        m,
        src,
        ComposeOp::Operator {
            node: "r.0".into(),
            op: "+".into(),
            before: false,
        },
    );
    assert_eq!(r.source, "((match Tilt { _ => 1 }) + ?) * ?");
    // a temporal form is structured too: the initial value and the value
    host.set_definition_draft(lamp.dim_by_tilt, "delay(0, Tilt)");
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    assert!(matches!(&node(&p, "r").kind, NodeKind::Delay));
    assert_eq!(node(&p, "r.0").role, "initial");
    assert_eq!(node(&p, "r.1").role, "value");
}

// ---- boolean logic and choices -------------------------------------------------------

struct AirConditioner {
    snapshot: ProjectSnapshot,
    ctrl: DeclId,
    button_held: ConceptId,
    switch_state: ConceptId,
}

/// `RoomTemp` (a temperature), `ButtonHeld` and `SwitchState` (true or
/// false), `AirConditionerCtrl : RoomTemp -> ButtonHeld -> SwitchState`,
/// and `Armed`, a boolean-valued value with no inputs.
fn air_conditioner() -> AirConditioner {
    let s = ProjectSnapshot::new(bdl_model::surface::Design::empty("ac"));
    let mk = |s: &ProjectSnapshot, name: &str, rep: Representation| {
        let a = bdl_model::edit::apply_edit(s, &concept(name, Some(rep))).expect("concept");
        (a.snapshot, a.outcome.created_concept.expect("id"))
    };
    let (s, room_temp) = mk(
        &s,
        "RoomTemp",
        Representation::Quantity {
            dim: Dim::TEMPERATURE,
        },
    );
    let (s, button_held) = mk(&s, "ButtonHeld", Representation::Boolean);
    let (s, switch_state) = mk(&s, "SwitchState", Representation::Boolean);
    let a = bdl_model::edit::apply_edit(
        &s,
        &mapping(
            "AirConditionerCtrl",
            vec![room_temp, button_held],
            switch_state,
        ),
    )
    .expect("mapping");
    let ctrl = a.outcome.created_mapping.expect("id");
    let s = edit(&a.snapshot, mapping("Armed", vec![], switch_state));
    AirConditioner {
        snapshot: s,
        ctrl,
        button_held,
        switch_state,
    }
}

/// `if c then a else b` is a structured node with three children: the
/// condition expects true or false; both outcomes expect what the choice
/// gives — the position's expectation, else what the other outcome is.
#[test]
fn a_choice_is_a_structured_node_whose_parts_expect_what_a_choice_demands() {
    let ac = air_conditioner();
    let mut host = IdeHost::new(ac.snapshot.clone());
    host.set_definition_draft(
        ac.ctrl,
        "if RoomTemp > 299.15 K && ButtonHeld then true else false",
    );
    let p = formula_projection(&host.snapshot(), ac.ctrl).expect("projection");
    assert!(p.parse_ok && p.complete, "{:?}", p.unplaced);
    let root = node(&p, "r");
    assert!(matches!(root.kind, NodeKind::If));
    assert_eq!(root.children.len(), 3);
    let cond = node(&p, "r.0");
    assert!(matches!(&cond.kind, NodeKind::Compare { op } if op == "&&"));
    assert_eq!(
        cond.expected.as_ref().map(|t| t.kind),
        Some(TypeKindView::Boolean)
    );
    assert_eq!(
        cond.because,
        "a choice asks a question: the condition is true or false"
    );
    // inside the condition: `&&` asks true or false of both sides, `>`
    // puts two temperatures side by side
    assert_eq!(
        node(&p, "r.0.1").expected.as_ref().map(|t| t.kind),
        Some(TypeKindView::Boolean)
    );
    assert_eq!(
        dim_of(&node(&p, "r.0.0.1").expected),
        Some(Dim::TEMPERATURE)
    );
    // both outcomes: the switch state the formula must produce
    for id in ["r.1", "r.2"] {
        let o = node(&p, id);
        assert!(matches!(o.kind, NodeKind::Bool { .. }));
        assert_eq!(
            o.expected.as_ref().and_then(|t| t.concept),
            Some(ac.switch_state),
            "{id}"
        );
        assert!(
            o.because.contains("both outcomes of a choice"),
            "{}",
            o.because
        );
    }
    // nothing expected of the choice (the subject of `in` is not locally
    // determined): the outcome already written tells the other what it
    // must be
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(
        lamp.dim_by_tilt,
        "(if ? then 90 deg else ?) in Tilt .. Tilt",
    );
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(node(&p, "r.0").expected.is_none());
    assert_eq!(dim_of(&node(&p, "r.0.2").expected), Some(Dim::ANGLE));
    assert!(node(&p, "r.0.2").because.contains("the same kind of value"));
    assert_eq!(
        node(&p, "r.0.0").expected.as_ref().map(|t| t.kind),
        Some(TypeKindView::Boolean)
    );
    // the slots in Tab order include the choice's
    assert_eq!(p.slots, vec!["r.0.0", "r.0.2"]);
}

/// A position that expects true or false — a logical operand, a
/// condition, a boolean concept — offers the two truth values, and the
/// references that produce a truth value rank first; a quantity position
/// offers neither.
#[test]
fn a_boolean_slot_offers_true_and_false_and_ranks_boolean_references_first() {
    let ac = air_conditioner();
    let mut host = IdeHost::new(ac.snapshot.clone());
    host.set_definition_draft(ac.ctrl, "RoomTemp > 299.15 K && ?");
    let slot = formula_slot(&host.snapshot(), ac.ctrl, "r.1").expect("slot");
    assert_eq!(slot.booleans, vec!["true", "false"]);
    let labels: Vec<(&str, u8)> = slot
        .references
        .iter()
        .map(|r| (r.label.as_str(), r.relevance))
        .collect();
    assert_eq!(
        labels,
        vec![("ButtonHeld", 90), ("Armed", 75)],
        "never RoomTemp"
    );
    assert_eq!(
        slot.references[0].entity,
        Some(EntityRef::Concept(ac.button_held))
    );
    assert!(slot.units.is_empty());
    // the result position: a SwitchState is true or false
    host.set_definition_draft(ac.ctrl, "?");
    let slot = formula_slot(&host.snapshot(), ac.ctrl, "r").expect("slot");
    assert_eq!(slot.booleans, vec!["true", "false"]);
    // the concept itself (Armed produces a SwitchState) before a value of
    // the same kind
    assert_eq!(
        slot.references
            .iter()
            .map(|r| r.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Armed", "ButtonHeld"]
    );
    // a condition
    host.set_definition_draft(ac.ctrl, "if ? then true else false");
    let slot = formula_slot(&host.snapshot(), ac.ctrl, "r.0").expect("slot");
    assert_eq!(slot.booleans, vec!["true", "false"]);
    assert_eq!(
        slot.explanation,
        "Expected: true or false, because a choice asks a question: the condition is true or false."
    );
    // a negation
    host.set_definition_draft(ac.ctrl, "!?");
    let slot = formula_slot(&host.snapshot(), ac.ctrl, "r.0").expect("slot");
    assert_eq!(slot.booleans, vec!["true", "false"]);
    // a temperature: no truth value fits
    host.set_definition_draft(ac.ctrl, "RoomTemp > ?");
    let slot = formula_slot(&host.snapshot(), ac.ctrl, "r.1").expect("slot");
    assert!(slot.booleans.is_empty());
    assert_eq!(
        slot.references
            .iter()
            .map(|r| r.label.as_str())
            .collect::<Vec<_>>(),
        vec!["RoomTemp"],
        "a truth value is no temperature"
    );
}

/// The acceptance walkthrough: from an empty formula, structured actions
/// alone make `RoomTemp > 299.15 K && ButtonHeld` and
/// `if RoomTemp > 299.15 K && ButtonHeld then true else false`, and the
/// text is exactly that.
#[test]
fn the_air_conditioner_walkthrough_is_a_sequence_of_structured_actions() {
    let ac = air_conditioner();
    let mut host = IdeHost::new(ac.snapshot.clone());
    let m = ac.ctrl;
    let fill = |node: &str, text: &str| ComposeOp::Fill {
        node: node.into(),
        text: text.into(),
    };
    let after = |node: &str, op: &str| ComposeOp::Operator {
        node: node.into(),
        op: op.into(),
        before: false,
    };
    // the condition first
    let r = composed(&mut host, m, "", fill("r", "RoomTemp"));
    assert_eq!(r.source, "RoomTemp");
    let r = composed(&mut host, m, &r.source, after("r", ">"));
    assert_eq!(
        (r.source.as_str(), r.select.as_deref()),
        ("RoomTemp > ?", Some("r.1"))
    );
    let r = composed(&mut host, m, &r.source, fill("r.1", "299.15 K"));
    assert_eq!(r.source, "RoomTemp > 299.15 K");
    // `&&` after the comparison: no parentheses, the comparison binds tighter
    let r = composed(&mut host, m, &r.source, after("r", "&&"));
    assert_eq!(
        (r.source.as_str(), r.select.as_deref()),
        ("RoomTemp > 299.15 K && ?", Some("r.1"))
    );
    let r = composed(&mut host, m, &r.source, fill("r.1", "ButtonHeld"));
    assert_eq!(r.source, "RoomTemp > 299.15 K && ButtonHeld");
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    let _ = &p;
    // the same, as the condition of a choice, built from the choice down
    let r = composed(&mut host, m, "", ComposeOp::Choose { node: "r".into() });
    assert_eq!(
        (r.source.as_str(), r.select.as_deref()),
        ("if ? then ? else ?", Some("r.0"))
    );
    let r = composed(&mut host, m, &r.source, fill("r.0", "RoomTemp"));
    let r = composed(&mut host, m, &r.source, after("r.0", ">"));
    let r = composed(&mut host, m, &r.source, fill("r.0.1", "299.15 K"));
    let r = composed(&mut host, m, &r.source, after("r.0", "&&"));
    let r = composed(&mut host, m, &r.source, fill("r.0.1", "ButtonHeld"));
    assert_eq!(
        r.source,
        "if RoomTemp > 299.15 K && ButtonHeld then ? else ?"
    );
    assert_eq!(
        r.select.as_deref(),
        Some("r.1"),
        "the next slot in Tab order"
    );
    let r = composed(&mut host, m, &r.source, fill("r.1", "true"));
    assert_eq!(r.select.as_deref(), Some("r.2"));
    let r = composed(&mut host, m, &r.source, fill("r.2", "false"));
    assert_eq!(
        r.source,
        "if RoomTemp > 299.15 K && ButtonHeld then true else false"
    );
    host.set_definition_draft(m, &r.source);
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    assert!(p.complete, "{:?}", p.unplaced);
}

/// `!` is the prefix form: no slot, the operand parenthesised only when
/// it binds weaker; a choice wraps a node as one outcome; the logical
/// operators and a choice parenthesise as their precedence demands; an
/// empty logical operand or negation is removed with its operator.
#[test]
fn negation_choice_and_the_logical_operators_compose_with_the_right_parentheses() {
    let ac = air_conditioner();
    let mut host = IdeHost::new(ac.snapshot.clone());
    let m = ac.ctrl;
    let not = |node: &str| ComposeOp::Operator {
        node: node.into(),
        op: "!".into(),
        before: false,
    };
    let r = composed(&mut host, m, "ButtonHeld", not("r"));
    assert_eq!(
        (r.source.as_str(), r.select.as_deref()),
        ("!ButtonHeld", Some("r"))
    );
    let r = composed(&mut host, m, "!ButtonHeld", not("r"));
    assert_eq!(r.source, "!!ButtonHeld");
    let r = composed(&mut host, m, "ButtonHeld && Armed", not("r"));
    assert_eq!(r.source, "!(ButtonHeld && Armed)");
    let r = composed(&mut host, m, "ButtonHeld && Armed", not("r.1"));
    assert_eq!(r.source, "ButtonHeld && !Armed");
    let r = composed(&mut host, m, "RoomTemp > 299.15 K", not("r"));
    assert_eq!(r.source, "!(RoomTemp > 299.15 K)");
    let r = composed(&mut host, m, "?", not("r"));
    assert_eq!(r.source, "!?");
    // `||` under `&&` and the other way round
    let and = |node: &str| ComposeOp::Operator {
        node: node.into(),
        op: "&&".into(),
        before: false,
    };
    let or = |node: &str| ComposeOp::Operator {
        node: node.into(),
        op: "||".into(),
        before: false,
    };
    let r = composed(&mut host, m, "ButtonHeld || Armed", and("r"));
    assert_eq!(r.source, "(ButtonHeld || Armed) && ?");
    let r = composed(&mut host, m, "ButtonHeld && Armed", or("r"));
    assert_eq!(r.source, "ButtonHeld && Armed || ?");
    let r = composed(&mut host, m, "ButtonHeld && Armed", or("r.1"));
    assert_eq!(r.source, "ButtonHeld && (Armed || ?)");
    // a choice wraps a node as its `then` outcome; as an operand it is
    // parenthesised, its parts never are
    let choose = |node: &str| ComposeOp::Choose { node: node.into() };
    let r = composed(&mut host, m, "ButtonHeld", choose("r"));
    assert_eq!(
        (r.source.as_str(), r.select.as_deref()),
        ("if ? then ButtonHeld else ?", Some("r.0"))
    );
    let r = composed(&mut host, m, "ButtonHeld && Armed", choose("r.1"));
    assert_eq!(r.source, "ButtonHeld && (if ? then Armed else ?)");
    let r = composed(&mut host, m, "ButtonHeld && Armed", choose("r"));
    assert_eq!(r.source, "if ? then ButtonHeld && Armed else ?");
    let r = composed(&mut host, m, "if ? then ButtonHeld else ?", and("r"));
    assert_eq!(r.source, "(if ? then ButtonHeld else ?) && ?");
    let r = composed(&mut host, m, "if ? then ButtonHeld else ?", and("r.1"));
    assert_eq!(r.source, "if ? then ButtonHeld && ? else ?");
    let r = composed(&mut host, m, "if ? then ButtonHeld else ?", or("r.0"));
    assert_eq!(r.source, "if ? || ? then ButtonHeld else ?");
    // filling a slot with a choice needs parentheses under an operator
    let r = composed(
        &mut host,
        m,
        "ButtonHeld && ?",
        ComposeOp::Fill {
            node: "r.1".into(),
            text: "if Armed then true else false".into(),
        },
    );
    assert_eq!(r.source, "ButtonHeld && (if Armed then true else false)");
    // removing an empty logical operand removes the operator; an empty
    // negation is nothing
    let remove = |node: &str| ComposeOp::Remove { node: node.into() };
    let r = composed(&mut host, m, "ButtonHeld && ?", remove("r.1"));
    assert_eq!(
        (r.source.as_str(), r.select.as_deref()),
        ("ButtonHeld", Some("r"))
    );
    let r = composed(&mut host, m, "? || Armed", remove("r.0"));
    assert_eq!(r.source, "Armed");
    let r = composed(&mut host, m, "!?", remove("r.0"));
    assert_eq!((r.source.as_str(), r.select.as_deref()), ("?", Some("r")));
    let r = composed(&mut host, m, "ButtonHeld && !?", remove("r.1.0"));
    assert_eq!(r.source, "ButtonHeld && ?");
    // a choice's outcome or condition becomes a slot again, never less
    let r = composed(
        &mut host,
        m,
        "if Armed then ButtonHeld else false",
        remove("r.0"),
    );
    assert_eq!(r.source, "if ? then ButtonHeld else false");
    let r = composed(
        &mut host,
        m,
        "if ? then ButtonHeld else false",
        remove("r.0"),
    );
    assert_eq!(r.source, "if ? then ButtonHeld else false");
    // the whole choice
    let r = composed(
        &mut host,
        m,
        "if Armed then ButtonHeld else false",
        remove("r"),
    );
    assert_eq!(r.source, "?");
}

mod round_trip {
    use super::*;
    use proptest::prelude::*;

    /// Small arithmetic over the lamp's names, numbers and units.
    fn arb_expr() -> impl Strategy<Value = String> {
        let leaf = prop_oneof![
            Just("Tilt".to_string()),
            Just("1".to_string()),
            Just("2.5".to_string()),
            Just("90 deg".to_string()),
            Just("?".to_string()),
        ];
        leaf.prop_recursive(3, 24, 2, |inner| {
            prop_oneof![
                (
                    inner.clone(),
                    prop_oneof![Just("+"), Just("-"), Just("*"), Just("/"), Just("<")],
                    inner.clone()
                )
                    .prop_map(|(a, op, b)| format!("{a} {op} {b}")),
                (
                    inner.clone(),
                    prop_oneof![Just("&&"), Just("||")],
                    inner.clone()
                )
                    .prop_map(|(a, op, b)| format!("{a} {op} {b}")),
                inner.clone().prop_map(|a| format!("({a})")),
                inner.clone().prop_map(|a| format!("-{a}")),
                inner.clone().prop_map(|a| format!("!{a}")),
                (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("min({a}, {b})")),
                (inner.clone(), inner.clone(), inner)
                    .prop_map(|(c, a, b)| format!("if {c} then {a} else {b}")),
            ]
        })
    }

    /// The tree without spans: what a formula means syntactically.
    fn shape(e: &bdl_syntax::SurfaceExpr) -> String {
        use bdl_syntax::ExprKind as K;
        match &e.kind {
            K::Name(n) => n.clone(),
            K::Number { literal, unit } => match unit {
                Some(u) => format!("{}{}", literal.as_str(), u.name),
                None => literal.as_str().to_owned(),
            },
            K::Hole => "?".into(),
            K::Unary { op, expr } => format!("({op:?} {})", shape(expr)),
            K::Binary { op, lhs, rhs } => format!("({} {op:?} {})", shape(lhs), shape(rhs)),
            K::If { cond, then, els } => {
                format!(
                    "(if {} then {} else {})",
                    shape(cond),
                    shape(then),
                    shape(els)
                )
            }
            K::Call { callee, args } => format!(
                "{}[{}]",
                shape(callee),
                args.iter().map(shape).collect::<Vec<_>>().join(",")
            ),
            other => format!("{other:?}"),
        }
    }

    fn all_ids(n: &FormulaNode, out: &mut Vec<String>) {
        out.push(n.id.clone());
        n.children.iter().for_each(|c| all_ids(c, out));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// parse → project → a no-op structured edit (fill every node with
        /// its own text) → parse: the same tree.
        #[test]
        fn filling_a_node_with_its_own_text_is_a_no_op(src in arb_expr()) {
            let lamp = lamp();
            let mut host = IdeHost::new(lamp.snapshot.clone());
            let m = lamp.dim_by_tilt;
            let Ok(before) = bdl_syntax::formula(&src) else { return Ok(()) };
            host.set_definition_draft(m, &src);
            let p = formula_projection(&host.snapshot(), m).expect("projection");
            let root = p.root.expect("tree");
            let mut ids = Vec::new();
            all_ids(&root, &mut ids);
            for id in ids {
                let n = root.find(&id).unwrap();
                let r = compose(&host.snapshot(), m, &src, &ComposeOp::Fill { node: id.clone(), text: n.text.clone() }).expect("compose");
                let after = bdl_syntax::formula(&r.source).unwrap_or_else(|e| panic!("{}: {e:?}", r.source));
                prop_assert_eq!(shape(&after), shape(&before), "{} ← {} at {}", r.source, src, id);
            }
        }

        /// Wrapping a node in an operator and removing the new slot gives
        /// the original meaning back; a unit switch there and back keeps
        /// the tree's shape (the coordinate may be rewritten).
        #[test]
        fn wrap_then_remove_restores_the_meaning(src in arb_expr(), op in prop_oneof![Just("+"), Just("*"), Just("/"), Just("<"), Just("&&"), Just("||")]) {
            let lamp = lamp();
            let mut host = IdeHost::new(lamp.snapshot.clone());
            let m = lamp.dim_by_tilt;
            let Ok(before) = bdl_syntax::formula(&src) else { return Ok(()) };
            host.set_definition_draft(m, &src);
            let p = formula_projection(&host.snapshot(), m).expect("projection");
            let root = p.root.expect("tree");
            let mut ids = Vec::new();
            all_ids(&root, &mut ids);
            for id in ids {
                let w = compose(&host.snapshot(), m, &src, &ComposeOp::Operator { node: id.clone(), op: op.to_string(), before: false }).expect("wrap");
                let slot = w.select.clone().expect("a new slot");
                let back = compose(&host.snapshot(), m, &w.source, &ComposeOp::Remove { node: slot }).expect("remove");
                let after = bdl_syntax::formula(&back.source).unwrap_or_else(|e| panic!("{}: {e:?}", back.source));
                prop_assert_eq!(shape(&after), shape(&before), "{} ← {} ← {} at {}", back.source, w.source, src, id);
            }
        }
    }
}

/// Cost evidence, not a threshold: `cargo test -p bdl-ide --test
/// formula_composer composer_latency -- --ignored --nocapture` prints the
/// projection and slot-query times for a small, a nested and a
/// candidate-rich formula.  A projection re-elaborates one formula over
/// the snapshot's analysis; nothing else is recomputed.
#[test]
#[ignore]
fn composer_latency_measurement() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    let nested = "clamp(((Tilt + 1 deg) * 2 - (Tilt / (90 deg)) * 3) / (1 + 2 * (3 - min(Tilt, 4 deg) / (5 deg))), min(0, max(1, 2)), 1)";
    for (label, src, node) in [
        ("small", "Tilt / ?", "r.1"),
        ("nested", nested, "r.0.0.0.0"),
        ("candidate-rich", "min(?, ?)", "r.0"),
    ] {
        host.set_definition_draft(m, src);
        let snap = host.snapshot();
        let n = 200;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            let _ = formula_projection(&snap, m).expect("projection");
        }
        let proj = t0.elapsed() / n;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            let _ = formula_slot(&snap, m, node).expect("slot");
        }
        let slot = t0.elapsed() / n;
        let t0 = std::time::Instant::now();
        let s = host.snapshot();
        let snapshot = t0.elapsed();
        eprintln!(
            "latency {label}: projection {:?}, slot query {:?}, snapshot (unchanged overlay) {:?}, source {} bytes",
            proj, slot, snapshot, src.len()
        );
        let _ = s;
    }
}

// ---- P11: the natural forms in the projection ----------------------------------------

struct Readings {
    snapshot: ProjectSnapshot,
    angles_ok: DeclId,
    normalized: DeclId,
    tilt: ConceptId,
}

/// `Tilt : q angle`, `Angles : List<q angle>`, `Ok : Bool`, `Levels :
/// List<q0>`; `angles : Angles` (a relationship without inputs),
/// `anglesOk : Angles -> Ok`, `normalized : Angles -> Levels`, `limit : Tilt`.
fn readings() -> Readings {
    let s = ProjectSnapshot::new(bdl_model::surface::Design::empty("readings"));
    let mk = |s: &ProjectSnapshot, name: &str, rep: Representation| {
        let a = bdl_model::edit::apply_edit(s, &concept(name, Some(rep))).expect("concept");
        (a.snapshot, a.outcome.created_concept.expect("id"))
    };
    let angle = Representation::Quantity { dim: Dim::ANGLE };
    let (s, tilt) = mk(&s, "Tilt", angle.clone());
    let (s, angles) = mk(&s, "Angles", Representation::list(angle));
    let (s, ok) = mk(&s, "Ok", Representation::Boolean);
    let (s, levels) = mk(
        &s,
        "Levels",
        Representation::list(Representation::Quantity { dim: Dim::ZERO }),
    );
    let m = |s: &ProjectSnapshot, name: &str, inputs: Vec<ConceptId>, out: ConceptId| {
        let a = bdl_model::edit::apply_edit(s, &mapping(name, inputs, out)).expect("mapping");
        (a.snapshot, a.outcome.created_mapping.expect("id"))
    };
    let (s, _angles) = m(&s, "angles", vec![], angles);
    let (s, _limit) = m(&s, "limit", vec![], tilt);
    let (s, angles_ok) = m(&s, "anglesOk", vec![angles], ok);
    let (s, normalized) = m(&s, "normalized", vec![angles], levels);
    Readings {
        snapshot: s,
        angles_ok,
        normalized,
        tilt,
    }
}

#[test]
fn a_binder_projects_structurally_with_a_local_and_its_element_type() {
    let r = readings();
    let mut host = IdeHost::new(r.snapshot.clone());
    host.set_definition_draft(
        r.angles_ok,
        "all angle in Angles: angle in -45 deg .. 45 deg",
    );
    let snap = host.snapshot();
    let p = formula_projection(&snap, r.angles_ok).expect("projection");
    assert!(p.parse_ok && p.complete, "{p:#?}");
    let root = node(&p, "r");
    assert!(
        matches!(&root.kind, NodeKind::Binder { form, param, param_type: Some(t) }
            if form == "all" && param == "angle" && t.dim == Some(Dim::ANGLE)),
        "{root:#?}"
    );
    assert_eq!(root.children.len(), 2);
    // the collection is the input, a collection of angles
    let coll = node(&p, "r.0");
    assert!(matches!(
        &coll.kind,
        NodeKind::Reference {
            local: false,
            entity: Some(_),
            ..
        }
    ));
    assert_eq!(
        coll.actual
            .as_ref()
            .and_then(|t| t.element.as_ref())
            .and_then(|e| e.dim),
        Some(Dim::ANGLE)
    );
    // the body is a membership test in a range; its subject is the local
    let body = node(&p, "r.1");
    assert!(matches!(&body.kind, NodeKind::Compare { op } if op == "in"));
    assert_eq!(dim_of(&body.expected), None);
    assert!(
        body.because.contains("all asks a question of every angle"),
        "{}",
        body.because
    );
    let subject = node(&p, "r.1.0");
    assert!(
        matches!(&subject.kind, NodeKind::Reference { name, local: true, entity: None } if name == "angle")
    );
    assert_eq!(dim_of(&subject.actual), Some(Dim::ANGLE));
    let range = node(&p, "r.1.1");
    assert!(matches!(range.kind, NodeKind::Range));
    assert_eq!(range.children.len(), 2);
    // both ends expect the subject's kind
    for id in ["r.1.1.0", "r.1.1.1"] {
        let end = node(&p, id);
        assert_eq!(dim_of(&end.expected), Some(Dim::ANGLE), "{id}: {end:#?}");
        assert!(
            end.because.contains("comparable with angle"),
            "{}",
            end.because
        );
    }
    assert!(matches!(&node(&p, "r.1.1.0").kind, NodeKind::Unary { .. }));
    assert!(matches!(
        &node(&p, "r.1.1.1").kind,
        NodeKind::Quantity { .. }
    ));
    // the slot query on an endpoint offers angle units, and the local as a
    // reference is not a design entity
    host.set_definition_draft(r.angles_ok, "all angle in Angles: angle in ? .. ?");
    let snap = host.snapshot();
    let slot = formula_slot(&snap, r.angles_ok, "r.1.1.0").expect("slot");
    assert_eq!(dim_of(&slot.expected), Some(Dim::ANGLE));
    assert_eq!(
        slot.units
            .iter()
            .map(|u| u.symbol.as_str())
            .collect::<Vec<_>>(),
        vec!["rad", "deg", "turn"]
    );
    let slot = formula_slot(&snap, r.angles_ok, "r.1.1.1").expect("slot");
    assert_eq!(dim_of(&slot.expected), Some(Dim::ANGLE));
    assert!(slot.references.iter().any(|c| c.label == "limit"));
}

#[test]
fn binder_bodies_expect_bool_or_the_element_of_the_result() {
    let r = readings();
    let mut host = IdeHost::new(r.snapshot.clone());
    host.set_definition_draft(r.angles_ok, "any angle in Angles: ?");
    let snap = host.snapshot();
    let slot = formula_slot(&snap, r.angles_ok, "r.1").expect("slot");
    assert!(matches!(
        slot.expected.as_ref().map(|t| t.kind),
        Some(TypeKindView::Boolean)
    ));
    assert!(
        slot.explanation.contains("true or false"),
        "{}",
        slot.explanation
    );
    // filter: Bool body, and the result is the collection itself
    host.set_definition_draft(r.normalized, "filter angle in Angles: ?");
    let slot = formula_slot(&host.snapshot(), r.normalized, "r.1").expect("slot");
    assert!(matches!(
        slot.expected.as_ref().map(|t| t.kind),
        Some(TypeKindView::Boolean)
    ));
    // map: the body is one element of what the mapping produces (Levels
    // is a collection of dimensionless quantities)
    host.set_definition_draft(r.normalized, "map angle in Angles: ?");
    let slot = formula_slot(&host.snapshot(), r.normalized, "r.1").expect("slot");
    assert_eq!(dim_of(&slot.expected), Some(Dim::ZERO), "{slot:#?}");
    assert!(
        slot.explanation.contains("map makes a collection"),
        "{}",
        slot.explanation
    );
    // the local is offered in the body by the slot's references? no — the
    // slot's references are the design's; the local appears through the
    // projection's `local` reference nodes and text completion
    let p = formula_projection(&host.snapshot(), r.normalized).expect("projection");
    let root = node(&p, "r");
    assert!(
        matches!(&root.kind, NodeKind::Binder { form, param_type: Some(t), .. }
        if form == "map" && t.dim == Some(Dim::ANGLE))
    );
    // a product of two unknowns stays insufficient inside a binder
    host.set_definition_draft(r.normalized, "map angle in Angles: ? * ?");
    let slot = formula_slot(&host.snapshot(), r.normalized, "r.1.0").expect("slot");
    assert!(slot.insufficient);
}

#[test]
fn wrapping_in_a_binder_and_inserting_a_range_are_text_edits_with_fresh_names() {
    let r = readings();
    let mut host = IdeHost::new(r.snapshot.clone());
    let m = r.angles_ok;
    // wrap the input in `all`: the local is the collection's singular
    let res = composed(
        &mut host,
        m,
        "Angles",
        ComposeOp::Binder {
            node: "r".into(),
            form: "all".into(),
        },
    );
    assert_eq!(res.source, "all angle in Angles: ?");
    assert_eq!(res.select.as_deref(), Some("r.1"));
    // the relationship `angles` is taken: `angle` is still free
    let res = composed(
        &mut host,
        m,
        "angles",
        ComposeOp::Binder {
            node: "r".into(),
            form: "any".into(),
        },
    );
    assert_eq!(res.source, "any angle in angles: ?");
    // a collection that is not a plural name gets `item`; nested, `item2`
    let res = composed(
        &mut host,
        m,
        "map x in Angles: ?",
        ComposeOp::Binder {
            node: "r.1".into(),
            form: "filter".into(),
        },
    );
    assert_eq!(res.source, "map x in Angles: filter item in ?: ?");
    let res = composed(
        &mut host,
        m,
        "all item in Angles: any item in Angles: ?",
        ComposeOp::Binder {
            node: "r.1.1".into(),
            form: "all".into(),
        },
    );
    assert_eq!(
        res.source,
        "all item in Angles: any item in Angles: all item2 in ?: ?"
    );
    // a fresh name never captures: `limit` is a relationship, so a
    // collection called `limits` gives `item`
    let res = composed(
        &mut host,
        m,
        "limits",
        ComposeOp::Binder {
            node: "r".into(),
            form: "all".into(),
        },
    );
    assert_eq!(res.source, "all item in limits: ?");
    // in the collection position a binder is parenthesised; in the body not
    let res = composed(
        &mut host,
        m,
        "all angle in Angles: angle < limit",
        ComposeOp::Binder {
            node: "r.0".into(),
            form: "filter".into(),
        },
    );
    assert_eq!(
        res.source,
        "all angle in (filter item in Angles: ?): angle < limit"
    );
    // the range: `node in ? .. ?`, selecting the low end
    let res = composed(
        &mut host,
        m,
        "all angle in Angles: angle",
        ComposeOp::Range { node: "r.1".into() },
    );
    assert_eq!(res.source, "all angle in Angles: angle in ? .. ?");
    assert_eq!(res.select.as_deref(), Some("r.1.1.0"));
    // a sum as the subject needs no parentheses; a comparison does
    let res = composed(
        &mut host,
        m,
        "limit + 1 deg",
        ComposeOp::Range { node: "r".into() },
    );
    assert_eq!(res.source, "limit + 1 deg in ? .. ?");
    let res = composed(
        &mut host,
        m,
        "limit < 1 deg",
        ComposeOp::Range { node: "r".into() },
    );
    assert_eq!(res.source, "(limit < 1 deg) in ? .. ?");
    // a binder as an operand is parenthesised by the operator action
    let res = composed(
        &mut host,
        m,
        "all angle in Angles: angle < limit",
        ComposeOp::Operator {
            node: "r".into(),
            op: "&&".into(),
            before: false,
        },
    );
    assert_eq!(res.source, "(all angle in Angles: angle < limit) && ?");
    // an unknown form is refused
    assert!(matches!(
        compose(
            &host.snapshot(),
            m,
            "Angles",
            &ComposeOp::Binder {
                node: "r".into(),
                form: "each".into()
            }
        ),
        Err(QueryError::NotApplicable { .. })
    ));
    let _ = r.tilt;
}

#[test]
fn natural_form_mistakes_sit_on_their_nodes_in_the_projection() {
    let r = readings();
    let mut host = IdeHost::new(r.snapshot.clone());
    host.set_definition_draft(r.angles_ok, "all x in 5: true");
    let p = formula_projection(&host.snapshot(), r.angles_ok).expect("projection");
    let coll = node(&p, "r.0");
    assert!(
        coll.diagnostics
            .iter()
            .any(|d| d.code == "formula.binder.not_a_collection"),
        "{p:#?}"
    );
    host.set_definition_draft(r.angles_ok, "all angle in Angles: angle in 2 s .. 3 s");
    let p = formula_projection(&host.snapshot(), r.angles_ok).expect("projection");
    let lo = node(&p, "r.1.1.0");
    assert!(
        lo.diagnostics
            .iter()
            .any(|d| d.code == "formula.range.endpoint"
                && d.message == "This range endpoint must be an angle."),
        "{p:#?}"
    );
    // a local outside its body is unknown, and the fix names the locals
    // in scope where one exists
    host.set_definition_draft(r.angles_ok, "(all angle in Angles: true) && angle < limit");
    let p = formula_projection(&host.snapshot(), r.angles_ok).expect("projection");
    let stray = node(&p, "r.1.0");
    assert!(matches!(
        &stray.kind,
        NodeKind::Reference { local: false, .. }
    ));
    assert!(stray
        .diagnostics
        .iter()
        .any(|d| d.code == "formula.name.unknown"));
    host.set_definition_draft(r.angles_ok, "all angle in Angles: angel < limit");
    let p = formula_projection(&host.snapshot(), r.angles_ok).expect("projection");
    let typo = node(&p, "r.1.0");
    let d = typo
        .diagnostics
        .iter()
        .find(|d| d.code == "formula.name.unknown")
        .expect("unknown");
    assert!(
        d.fixes.iter().any(|f| f.contains("Locals in scope: angle")),
        "{:?}",
        d.fixes
    );
}

#[test]
fn completion_offers_binder_locals_in_bodies_and_binder_templates_over_collections() {
    let r = readings();
    let mut host = IdeHost::new(r.snapshot.clone());
    let src = "all angle in Angles: an";
    host.set_definition_draft(r.angles_ok, src);
    let snap = host.snapshot();
    let items = completion(
        &snap,
        &CompletionContext::Formula {
            mapping: r.angles_ok,
            offset: src.len() as u32,
        },
    );
    let local = items
        .iter()
        .find(|i| i.kind == CompletionKind::Local)
        .expect("the local");
    assert_eq!(local.label, "angle");
    assert!(items
        .iter()
        .all(|i| i.kind != CompletionKind::Local || i.label == "angle"));
    assert!(
        local.relevance
            > items
                .iter()
                .filter(|i| i.label == "angles")
                .map(|i| i.relevance)
                .max()
                .unwrap_or(0)
    );
    // not in the collection position
    let src = "all angle in an";
    host.set_definition_draft(r.angles_ok, src);
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: r.angles_ok,
            offset: src.len() as u32,
        },
    );
    assert!(
        items.iter().all(|i| i.kind != CompletionKind::Local),
        "{items:#?}"
    );
    // the template, with the one collection in scope filled in when
    // there is exactly one (here there are two: the input and `angles`)
    let src = "al";
    host.set_definition_draft(r.angles_ok, src);
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: r.angles_ok,
            offset: 2,
        },
    );
    let t = items
        .iter()
        .find(|i| i.kind == CompletionKind::Keyword && i.label.starts_with("all "))
        .expect("template");
    assert_eq!(t.label, "all item in collection: …");
    assert_eq!(t.insert, "all item in ");
    // a mapping without a collection in sight offers no template
    let lamp = lamp();
    let mut host2 = IdeHost::new(lamp.snapshot.clone());
    host2.set_definition_draft(lamp.dim_by_tilt, "al");
    let items = completion(
        &host2.snapshot(),
        &CompletionContext::Formula {
            mapping: lamp.dim_by_tilt,
            offset: 2,
        },
    );
    assert!(
        items.iter().all(|i| !i.label.starts_with("all ")),
        "{items:#?}"
    );
}

// ---- apply and positional completion (0.28) ------------------------------------------

/// Apply: a name that is an equation or a rule becomes a call with one
/// slot per argument, the first selected; a value or a literal is refused.
#[test]
fn apply_turns_a_name_into_a_call_with_the_compilers_arity() {
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    let r = composed(
        &mut host,
        ph.speed_of,
        "clamp",
        ComposeOp::Apply { node: "r".into() },
    );
    assert_eq!(r.source, "clamp(?, ?, ?)");
    assert_eq!(r.select.as_deref(), Some("r.0"));
    // a rule of the design: its signature's arity
    let r = composed(
        &mut host,
        ph.torque_of,
        "speedOf",
        ComposeOp::Apply { node: "r".into() },
    );
    assert_eq!(r.source, "speedOf(?)");
    // a value or a literal cannot be applied
    host.set_definition_draft(ph.speed_of, "armLength");
    let e = compose(
        &host.snapshot(),
        ph.speed_of,
        "armLength",
        &ComposeOp::Apply { node: "r".into() },
    )
    .expect_err("a value is not applied");
    assert!(matches!(e, QueryError::NotApplicable { .. }));
    let e = compose(
        &host.snapshot(),
        ph.speed_of,
        "1 m",
        &ComposeOp::Apply { node: "r".into() },
    )
    .expect_err("a literal is not applied");
    assert!(matches!(e, QueryError::NotApplicable { .. }));
}

/// Completion in the text of a formula ranks by what the *position*
/// expects — the projection's local inference at the offset — not only
/// by what the whole formula produces: in the numerator of `? / cycleTime`
/// for a speed a length-valued name outranks a speed-valued one.
#[test]
fn text_completion_ranks_by_the_positions_expected_type() {
    let ph = physics();
    let mut host = IdeHost::new(ph.snapshot.clone());
    host.set_definition_draft(ph.speed_of, "? / cycleTime");
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: ph.speed_of,
            offset: 0,
        },
    );
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    let pos = |name: &str| labels.iter().position(|l| *l == name).unwrap_or(usize::MAX);
    assert!(pos("armLength") < pos("speedLimit"), "{labels:?}");
    assert!(pos("wheelRadius") < pos("load"), "{labels:?}");
    // in the denominator a time is expected: cycleTime ranks first
    host.set_definition_draft(ph.speed_of, "armLength / ?");
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: ph.speed_of,
            offset: 12,
        },
    );
    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
    let pos = |name: &str| labels.iter().position(|l| *l == name).unwrap_or(usize::MAX);
    assert!(pos("cycleTime") < pos("armLength"), "{labels:?}");
    assert!(pos("cycleTime") < pos("speedLimit"), "{labels:?}");
}
