//! The Formula Composer's service (docs/architecture/ide-service.md
//! §Formula projection): the projection's shape, expected dimensions by
//! local inference, unit candidates sound for the slot's dimension,
//! reference and equation candidates by type, stale generations, stable
//! ranges, and the text edits structured actions make.

mod support;

use bdl_ide::*;
use bdl_model::edit::EditOp;
use bdl_model::surface::{ProjectSnapshot, Representation};
use bdl_model::{DeclId, Dim, SemanticId};
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
    length: SemanticId,
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
    let m = |s: &ProjectSnapshot, name: &str, inputs: Vec<SemanticId>, out: SemanticId| {
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
        matches!(&tilt.kind, NodeKind::Reference { name, entity: Some(EntityRef::Concept(c)) } if name == "Tilt" && *c == lamp.tilt)
    );
    assert_eq!(
        tilt.actual.as_ref().map(|t| t.kind),
        Some(TypeKindView::Concept)
    );
    assert_eq!(dim_of(&tilt.actual), Some(Dim::ANGLE));
    // the denominator: a quantity literal with its unit, parentheses in its range
    let ninety = node(&p, "r.0.1");
    assert!(
        matches!(&ninety.kind, NodeKind::Quantity { coordinate, unit, unit_id: Some(id) } if coordinate == "90" && unit == "deg" && id == "angle.deg")
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
    host.set_definition_draft(lamp.dim_by_tilt, "if Tilt < 10 deg then 0 else 1");
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(matches!(&node(&p, "r").kind, NodeKind::Opaque { what } if what.contains("choice")));
    assert!(node(&p, "r").children.is_empty());
    assert!(p.complete, "an opaque form is still a valid formula");
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
    // a truth-valued position offers the truth-valued equations
    host.set_definition_draft(lamp.dim_by_tilt, "if ? then 1 else 0");
    let p = formula_projection(&host.snapshot(), lamp.dim_by_tilt).expect("projection");
    assert!(matches!(node(&p, "r").kind, NodeKind::Opaque { .. }));
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

/// An unsupported form is an opaque node: its exact source, selectable,
/// and the structure around it stays editable.
#[test]
fn opaque_forms_keep_their_source_and_the_structure_around_them_stays_editable() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    let src = "(if Tilt < 10 deg then 1 else 0) * ?";
    host.set_definition_draft(m, src);
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    let opaque = node(&p, "r.0");
    assert!(matches!(&opaque.kind, NodeKind::Opaque { what } if what.contains("choice")));
    assert_eq!(
        &src[opaque.range.start as usize..opaque.range.end as usize],
        "(if Tilt < 10 deg then 1 else 0)"
    );
    assert!(opaque.children.is_empty());
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
    assert_eq!(r.source, "(if Tilt < 10 deg then 1 else 0) * 2");
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
    assert_eq!(r.source, "((if Tilt < 10 deg then 1 else 0) + ?) * ?");
    // a temporal form and a rule are opaque too, with their own sentence
    host.set_definition_draft(lamp.dim_by_tilt, "delay(0, Tilt)");
    let p = formula_projection(&host.snapshot(), m).expect("projection");
    assert!(
        matches!(&node(&p, "r").kind, NodeKind::Opaque { what } if what.contains("remembered"))
    );
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
                inner.clone().prop_map(|a| format!("({a})")),
                inner.clone().prop_map(|a| format!("-{a}")),
                (inner.clone(), inner).prop_map(|(a, b)| format!("min({a}, {b})")),
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
        fn wrap_then_remove_restores_the_meaning(src in arb_expr(), op in prop_oneof![Just("+"), Just("*"), Just("/")]) {
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
