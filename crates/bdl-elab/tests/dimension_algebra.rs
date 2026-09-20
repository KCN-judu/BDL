//! The dimension algebra, through the real pipeline: text → parser →
//! elaborator → checker → evaluator.  Establishes what already holds
//! before any unit surface is added: `q[d₁] * q[d₂] : q[d₁ + d₂]`,
//! `q[d₁] / q[d₂] : q[d₁ − d₂]`, signed integral exponents, cancellation,
//! and that every named derived quantity of the vocabulary is the
//! composition of base dimensions its name says.  Nothing here spells a
//! composite unit: the operands are concepts of the named dimensions.

#![allow(clippy::unwrap_used)]

use bdl_check::{infer, Grant};
use bdl_elab::{elaborate_design, RealizationOutcome};
use bdl_ir::Ty;
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::quantity::{self, QuantityDef};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, Dim, SemanticId};
use bdl_reactive::eval::{step, State, TickInput};
use bdl_reactive::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A design with one concept per named quantity (`Length`, `Speed`, …),
/// each represented by that quantity's dimension, plus a fresh result
/// concept per case.
struct Bench {
    s: ProjectSnapshot,
    concepts: BTreeMap<&'static str, SemanticId>,
}

impl Bench {
    fn new() -> Bench {
        let mut s = ProjectSnapshot::new(Design::empty("dims"));
        let mut concepts = BTreeMap::new();
        for q in quantity::QUANTITIES {
            let a = apply_edit(
                &s,
                &EditOp::CreateConcept {
                    name: q.type_name.into(),
                    description: String::new(),
                    representation: Some(Representation::Quantity { dim: q.dim }),
                },
            )
            .unwrap();
            s = a.snapshot;
            concepts.insert(q.type_name, a.outcome.created_concept.unwrap());
        }
        Bench { s, concepts }
    }

    fn concept(&mut self, name: &str, dim: Dim) -> SemanticId {
        let a = apply_edit(
            &self.s,
            &EditOp::CreateConcept {
                name: name.into(),
                description: String::new(),
                representation: Some(Representation::Quantity { dim }),
            },
        )
        .unwrap();
        self.s = a.snapshot;
        a.outcome.created_concept.unwrap()
    }

    /// `f(inputs…) = formula` producing a concept of `dim`; the elaborated
    /// Core is re-typed by the checker and its result dimension returned.
    fn dim_of(&mut self, inputs: &[&str], formula: &str, dim: Dim) -> Dim {
        let out = self.concept("Result", dim);
        let inputs: Vec<SemanticId> = inputs.iter().map(|n| self.concepts[n]).collect();
        let a = apply_edit(
            &self.s,
            &EditOp::CreateMapping {
                name: "f".into(),
                description: String::new(),
                signature: Signature {
                    inputs: inputs.clone(),
                    output: out,
                },
                definition: Some(Definition::Formula {
                    source: formula.into(),
                }),
                clock: None,
            },
        )
        .unwrap();
        self.s = a.snapshot;
        let id = a.outcome.created_mapping.unwrap();
        let e = elaborate_design(&self.s.design);
        let m = &e.mappings[&id];
        let RealizationOutcome::Elaborated(r) = &m.outcome else {
            panic!("`{formula}` did not elaborate: {:?}", m.diagnostics);
        };
        // the checker is the authority: the realization types against the
        // interface, and the inner term (under the constructor of the
        // result concept) has the dimension the algebra says
        bdl_check::check_realization(&e.ir, id).unwrap();
        // peel the curried parameters (one lambda per input, outermost
        // first) and the result concept's constructor
        let mut ctx: Vec<Ty> = Vec::new();
        let mut inner = &r.expr;
        while let bdl_ir::Expr::Lam { dom, body } = inner {
            // the checker's context is innermost first
            ctx.insert(0, dom.clone());
            inner = body;
        }
        let inner = match inner {
            bdl_ir::Expr::Mk { e, .. } => e.as_ref(),
            other => other,
        };
        let ty = infer(&e.ir, &Grant::All, &ctx, inner).unwrap();
        let Ty::Q { dim } = ty else {
            panic!("`{formula}` is not a quantity: {ty:?}");
        };
        let a = apply_edit(&self.s, &EditOp::DeleteMapping { id }).unwrap();
        self.s = a.snapshot;
        let a = apply_edit(&self.s, &EditOp::DeleteConcept { id: out }).unwrap();
        self.s = a.snapshot;
        dim
    }

    /// A closed formula evaluated for one tick: the canonical magnitude
    /// and dimension of its value.
    fn eval(&mut self, formula: &str, dim: Dim) -> (Dim, f64) {
        let out = self.concept("Result", dim);
        let a = apply_edit(
            &self.s,
            &EditOp::CreateMapping {
                name: "v".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: out,
                },
                definition: Some(Definition::Formula {
                    source: formula.into(),
                }),
                clock: None,
            },
        )
        .unwrap();
        self.s = a.snapshot;
        let id: DeclId = a.outcome.created_mapping.unwrap();
        let e = elaborate_design(&self.s.design);
        assert!(
            matches!(e.mappings[&id].outcome, RealizationOutcome::Elaborated(_)),
            "`{formula}`: {:?}",
            e.mappings[&id].diagnostics
        );
        let outcome = step(
            &e.ir,
            0,
            &BTreeSet::new(),
            &State::default(),
            &TickInput::default(),
        )
        .unwrap();
        let v = match &outcome.values[&id] {
            Value::Semantic { repr, .. } => (**repr).clone(),
            other => other.clone(),
        };
        let Value::Quantity { dim, value } = v else {
            panic!("`{formula}` is not a quantity: {v:?}");
        };
        let a = apply_edit(&self.s, &EditOp::DeleteMapping { id }).unwrap();
        self.s = a.snapshot;
        let a = apply_edit(&self.s, &EditOp::DeleteConcept { id: out }).unwrap();
        self.s = a.snapshot;
        (dim, value)
    }
}

fn q(name: &str) -> &'static QuantityDef {
    quantity::by_type_name(name).unwrap_or_else(|| panic!("no quantity {name}"))
}

fn dim(name: &str) -> Dim {
    q(name).dim
}

fn scaled(d: Dim, n: i8) -> Dim {
    Dim {
        length: d.length * n,
        mass: d.mass * n,
        time: d.time * n,
        current: d.current * n,
        temperature: d.temperature * n,
        amount: d.amount * n,
        luminous: d.luminous * n,
        angle: d.angle * n,
    }
}

// ---- §3 multiplication, through the pipeline ---------------------------------

#[test]
fn multiplication_adds_dimensions_through_the_pipeline() {
    let mut b = Bench::new();
    let cases: &[(&[&str], &str, Dim)] = &[
        (
            &["Length", "Length"],
            "Length * Length",
            scaled(Dim::LENGTH, 2),
        ),
        (
            &["Length", "Length", "Length"],
            "Length * Length * Length",
            scaled(Dim::LENGTH, 3),
        ),
        (&["Force", "Length"], "Force * Length", dim("Torque")),
        (&["Voltage", "Current"], "Voltage * Current", dim("Power")),
        (&["Scalar", "Length"], "Scalar * Length", Dim::LENGTH),
        (&["Angle", "Angle"], "Angle * Angle", scaled(Dim::ANGLE, 2)),
    ];
    for (inputs, formula, expected) in cases {
        assert_eq!(b.dim_of(inputs, formula, *expected), *expected, "{formula}");
    }
}

// ---- §4 division, through the pipeline ---------------------------------------

#[test]
fn division_subtracts_dimensions_and_cancels_through_the_pipeline() {
    let mut b = Bench::new();
    let cases: &[(&[&str], &str, Dim)] = &[
        (&["Length", "Time"], "Length / Time", dim("Speed")),
        (&["Speed", "Time"], "Speed / Time", dim("Acceleration")),
        (&["Angle", "Time"], "Angle / Time", dim("AngularVelocity")),
        (
            &["Force", "Length"],
            "Force / Length / Length",
            dim("Pressure"),
        ),
        (&["Power", "Current"], "Power / Current", dim("Voltage")),
        (&["Angle"], "Angle / Angle", Dim::ZERO),
        (&["Length"], "Length / Length", Dim::ZERO),
        // a product cancelled again: `(m · s) / s` is a length
        (&["Length", "Time"], "Length * Time / Time", Dim::LENGTH),
        // a square divided by its base: `m² / m` is a length
        (&["Length"], "Length * Length / Length", Dim::LENGTH),
        // a negative exponent on its own: `1 / s`
        (&["Time"], "1 / Time", dim("Frequency")),
    ];
    for (inputs, formula, expected) in cases {
        assert_eq!(b.dim_of(inputs, formula, *expected), *expected, "{formula}");
    }
}

/// The evaluator multiplies and divides canonical magnitudes with the
/// dimensions the checker gave: `(2 m) * (3 m) = 6 m²`, `(6 m) / (2 s)
/// = 3 m/s`, `(90 deg) / (90 deg) = 1`.
#[test]
fn the_evaluator_carries_the_algebra_on_canonical_magnitudes() {
    let mut b = Bench::new();
    let (d, v) = b.eval("(2 m) * (3 m)", scaled(Dim::LENGTH, 2));
    assert_eq!(d, scaled(Dim::LENGTH, 2));
    assert_eq!(v, 6.0);
    let (d, v) = b.eval("(6 m) / (2 s)", dim("Speed"));
    assert_eq!((d, v), (dim("Speed"), 3.0));
    let (d, v) = b.eval("(90 deg) / (90 deg)", Dim::ZERO);
    assert_eq!(d, Dim::ZERO);
    assert!((v - 1.0).abs() < 1e-12);
    // `(1000 mm) / (1 s)` is one metre per second, canonically
    let (d, v) = b.eval("(1000 mm) / (1 s)", dim("Speed"));
    assert_eq!(d, dim("Speed"));
    assert!((v - 1.0).abs() < 1e-9);
}

// ---- §5 every named derived quantity, systematically -------------------------

/// The decomposition of every named quantity from base dimensions, as a
/// formula over the base concepts; the pipeline's result must be the
/// vocabulary's declared dimension.
#[test]
fn every_named_derived_quantity_is_its_decomposition_from_base_dimensions() {
    let decompositions: &[(&str, &str)] = &[
        ("Speed", "Length / Time"),
        ("Acceleration", "Length / Time / Time"),
        ("AngularVelocity", "Angle / Time"),
        ("Frequency", "Scalar / Time"),
        ("Force", "Mass * Length / Time / Time"),
        ("Pressure", "Mass / Length / Time / Time"),
        ("Torque", "Mass * Length * Length / Time / Time"),
        ("Power", "Mass * Length * Length / Time / Time / Time"),
        (
            "Voltage",
            "Mass * Length * Length / Time / Time / Time / Current",
        ),
        ("Illuminance", "Luminous * Angle * Angle / Length / Length"),
    ];
    let base: &[&str] = &[
        "Scalar",
        "Angle",
        "Length",
        "Time",
        "Mass",
        "Current",
        "Temperature",
        "Amount",
        "Luminous",
    ];
    // every derived quantity of the vocabulary has a row here
    let derived: BTreeSet<&str> = quantity::QUANTITIES
        .iter()
        .filter(|q| !base.contains(&q.type_name))
        .map(|q| q.type_name)
        .collect();
    let rows: BTreeSet<&str> = decompositions.iter().map(|(n, _)| *n).collect();
    assert_eq!(derived, rows, "a derived quantity without a decomposition");
    let mut b = Bench::new();
    for (name, formula) in decompositions {
        let expected = dim(name);
        // the declared dimension is the algebra's, symbolically …
        let mut acc = Dim::ZERO;
        let mut op = '*';
        for tok in formula.split_whitespace() {
            match tok {
                "*" | "/" => op = tok.chars().next().unwrap(),
                n => {
                    let d = dim(n);
                    acc = if op == '*' { acc + d } else { acc - d };
                }
            }
        }
        assert_eq!(acc, expected, "{name}: Dim algebra");
        // … and through the pipeline, with `Scalar` written as `1`
        let inputs: Vec<&str> = formula
            .split_whitespace()
            .filter(|t| *t != "*" && *t != "/" && *t != "Scalar")
            .collect();
        let formula = formula.replace("Scalar", "1");
        assert_eq!(b.dim_of(&inputs, &formula, expected), expected, "{name}");
    }
}

/// `Dim` is an abelian group under `+` with inverse via `−`, and its
/// exponents are signed integers: `L²`, `L³`, `T⁻¹`, `L T⁻²`, `A T⁻¹`,
/// `M L² T⁻³` are all representable and equal to their named quantity
/// where one exists.
#[test]
fn signed_integral_exponents_represent_the_named_dimensions() {
    let l2 = Dim::LENGTH + Dim::LENGTH;
    let l3 = l2 + Dim::LENGTH;
    let t_1 = Dim::ZERO - Dim::TIME;
    let lt_2 = Dim::LENGTH - Dim::TIME - Dim::TIME;
    let at_1 = Dim::ANGLE - Dim::TIME;
    let ml2t_3 = Dim::MASS + l2 - Dim::TIME - Dim::TIME - Dim::TIME;
    assert_eq!((l2.length, l3.length), (2, 3));
    assert_eq!(t_1, dim("Frequency"));
    assert_eq!(lt_2, dim("Acceleration"));
    assert_eq!(at_1, dim("AngularVelocity"));
    assert_eq!(ml2t_3, dim("Power"));
    // cancellation
    assert_eq!(l2 - Dim::LENGTH, Dim::LENGTH);
    assert_eq!(at_1 + Dim::TIME, Dim::ANGLE);
    assert_eq!(Dim::ANGLE - Dim::ANGLE, Dim::ZERO);
    // the vocabulary names each of these, and by_dim finds it
    for d in [t_1, lt_2, at_1, ml2t_3] {
        assert!(quantity::by_dim(d).is_some(), "{d:?} unnamed");
    }
    assert!(quantity::by_dim(l2).is_none(), "area is not named (yet)");
}

// ---- composite unit literals, through the pipeline ---------------------------

/// A literal with a composite unit elaborates to the dimensioned literal
/// the algebra says: the dimension of the expression, the canonical
/// magnitude `coordinate × scale`, evaluated by the reference evaluator.
#[test]
fn composite_unit_literals_elaborate_to_the_algebra_s_dimension_and_scale() {
    let mut b = Bench::new();
    let pi = std::f64::consts::PI;
    let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0);
    let cases: &[(&str, &str, f64)] = &[
        ("180 deg per s", "AngularVelocity", pi),
        ("1 turn per s", "AngularVelocity", 2.0 * pi),
        ("60 deg per min", "AngularVelocity", pi / 180.0),
        ("1000 mm per s", "Speed", 1.0),
        ("36 km per h", "Speed", 10.0),
        ("60 m per min", "Speed", 1.0),
        ("9.81 m per s^2", "Acceleration", 9.81),
        ("9810 mm per s^2", "Acceleration", 9.81),
        ("1 N * m", "Torque", 1.0),
        ("1 kg * m per s^2", "Force", 1.0),
        ("2 s^-1", "Frequency", 2.0),
        // the existing atomic literals mean what they meant
        ("90 deg", "Angle", pi / 2.0),
        ("2.5 s", "Time", 2.5),
        ("300 lx", "Illuminance", 300.0),
        ("25.4 mm", "Length", 0.0254),
    ];
    for (formula, name, magnitude) in cases {
        let (d, v) = b.eval(formula, dim(name));
        assert_eq!(d, dim(name), "{formula}");
        assert!(close(v, *magnitude), "{formula}: {v} ≠ {magnitude}");
    }
    // the same physical quantity, two spellings: one magnitude
    let (_, a) = b.eval("1 N * m", dim("Torque"));
    let (_, c) = b.eval("1 kg * m^2 per s^2", dim("Torque"));
    assert!(close(a, c));
    // and division of two literals agrees with the composite literal
    let (d1, v1) = b.eval("(180 deg) / (1 s)", dim("AngularVelocity"));
    let (d2, v2) = b.eval("180 deg per s", dim("AngularVelocity"));
    assert_eq!(d1, d2);
    assert!(close(v1, v2));
}

/// The diagnostics a designer sees: an unknown factor, an affine factor,
/// an oversized power, a unit of the wrong dimension for the position.
#[test]
fn composite_unit_faults_are_named_for_the_designer() {
    let mut b = Bench::new();
    let diag = |b: &mut Bench, formula: &str, out_dim: Dim| -> Vec<(String, String)> {
        let out = b.concept("Result", out_dim);
        let a = apply_edit(
            &b.s,
            &EditOp::CreateMapping {
                name: "v".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: out,
                },
                definition: Some(Definition::Formula {
                    source: formula.into(),
                }),
                clock: None,
            },
        )
        .unwrap();
        b.s = a.snapshot;
        let id = a.outcome.created_mapping.unwrap();
        let e = elaborate_design(&b.s.design);
        let ds: Vec<(String, String)> = e.mappings[&id]
            .diagnostics
            .iter()
            .map(|d| (d.code.as_str().to_owned(), d.message.clone()))
            .collect();
        let a = apply_edit(&b.s, &EditOp::DeleteMapping { id }).unwrap();
        b.s = a.snapshot;
        let a = apply_edit(&b.s, &EditOp::DeleteConcept { id: out }).unwrap();
        b.s = a.snapshot;
        ds
    };
    let d = diag(&mut b, "10 m per furlong", dim("Speed"));
    assert_eq!(d[0].0, "formula.unit.unknown");
    assert_eq!(d[0].1, "`furlong` in `m per furlong` is not a unit.");
    let d = diag(&mut b, "10 furlong", Dim::LENGTH);
    assert_eq!(d[0].1, "`furlong` is not a unit.");
    let d = diag(&mut b, "10 m^1000", Dim::LENGTH);
    assert_eq!(d[0].0, "formula.unit.exponent");
    let d = diag(&mut b, "10 deg per s", Dim::LENGTH);
    assert_eq!(d[0].0, "formula.unit.dimension");
    assert_eq!(
        d[0].1,
        "`deg per s` is a unit of an angular rate, but this value must be a length."
    );
    // the affine charts are not surface symbols; the fault exists for
    // the day they are (tested at the algebra level in bdl-model)
    let d = diag(&mut b, "10 °C per s", dim("Speed"));
    assert!(!d.is_empty());
}
