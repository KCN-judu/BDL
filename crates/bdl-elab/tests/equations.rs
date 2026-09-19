//! The equation language end to end at the elaboration layer: the cases
//! of `BDL_FV/BDL/Experiments/EquationExamples.lean` (A–M) and the Phase-9c
//! capability boundary (`eq_accepted`, `lt_rejected`, `min_mode_rejected`),
//! written in textual BDL, elaborated, re-checked by `bdl-check` and run
//! by the reference evaluator.  Every negative case names the diagnostic a
//! designer sees.

#![allow(clippy::unwrap_used)]

use bdl_check::pretty;
use bdl_diagnostics::Diagnostic;
use bdl_elab::{elaborate_design, RealizationOutcome};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, Dim, SemanticId};
use bdl_reactive::eval::{step, State, TickInput};
use bdl_reactive::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Brightness : q0, ordered · Opacity : q0 · Mode : q0 (not ordered) ·
/// Temperature : q K · Readings : List<Temperature> · Severities :
/// List<Scalar> · Humidity : Scalar · Climate : Pair<Temperature, Scalar> ·
/// Held : Bool · Length : q m · Duration : q s · Level : Count ·
/// MaybeTemp : Option<Temperature>.
struct Fixture {
    s: ProjectSnapshot,
    ids: BTreeMap<&'static str, SemanticId>,
}

impl Fixture {
    fn new() -> Fixture {
        let mut s = ProjectSnapshot::new(Design::empty("equations"));
        let mut ids = BTreeMap::new();
        let temp = Representation::Quantity {
            dim: Dim::TEMPERATURE,
        };
        let scalar = Representation::Quantity { dim: Dim::ZERO };
        for (name, rep, ordered) in [
            ("Brightness", scalar.clone(), true),
            ("Opacity", scalar.clone(), false),
            ("Mode", scalar.clone(), false),
            ("Temperature", temp.clone(), false),
            ("Readings", Representation::list(temp.clone()), false),
            ("Severities", Representation::list(scalar.clone()), false),
            ("Humidity", scalar.clone(), false),
            (
                "Climate",
                Representation::pair(temp.clone(), scalar.clone()),
                false,
            ),
            ("Held", Representation::Boolean, false),
            (
                "Length",
                Representation::Quantity { dim: Dim::LENGTH },
                false,
            ),
            (
                "Duration",
                Representation::Quantity { dim: Dim::TIME },
                false,
            ),
            ("Level", Representation::Count, false),
            ("MaybeTemp", Representation::optional(temp.clone()), false),
            (
                "Flags",
                Representation::list(Representation::Boolean),
                false,
            ),
        ] {
            let a = apply_edit(
                &s,
                &EditOp::CreateConcept {
                    name: name.into(),
                    description: String::new(),
                    representation: Some(rep),
                },
            )
            .unwrap();
            s = a.snapshot;
            let id = a.outcome.created_concept.unwrap();
            if ordered {
                s = apply_edit(&s, &EditOp::SetConceptOrdered { id, ordered: true })
                    .unwrap()
                    .snapshot;
            }
            ids.insert(name, id);
        }
        Fixture { s, ids }
    }

    fn c(&self, name: &str) -> SemanticId {
        self.ids[name]
    }

    fn mapping(
        &mut self,
        name: &str,
        inputs: &[&str],
        output: &str,
        formula: Option<&str>,
    ) -> DeclId {
        let a = apply_edit(
            &self.s,
            &EditOp::CreateMapping {
                name: name.into(),
                description: String::new(),
                signature: Signature {
                    inputs: inputs.iter().map(|i| self.c(i)).collect(),
                    output: self.c(output),
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        self.s = a.snapshot;
        let id = a.outcome.created_mapping.unwrap();
        if let Some(f) = formula {
            let a = apply_edit(
                &self.s,
                &EditOp::AttachDefinition {
                    id,
                    definition: Definition::Formula { source: f.into() },
                },
            )
            .unwrap();
            self.s = a.snapshot;
        }
        id
    }

    /// Elaborate the design; every elaborated realization is re-checked.
    fn elaborate(&self) -> (bdl_elab::Elaboration, BTreeMap<DeclId, Vec<Diagnostic>>) {
        let e = elaborate_design(&self.s.design);
        let mut diags = BTreeMap::new();
        for (id, m) in &e.mappings {
            if let RealizationOutcome::Elaborated(r) = &m.outcome {
                let checked = bdl_check::check_realization(&e.ir, *id);
                assert!(
                    checked.is_ok(),
                    "checker refused {}: {:?}",
                    pretty::expr(&r.expr),
                    checked
                );
            }
            diags.insert(*id, m.diagnostics.clone());
        }
        (e, diags)
    }

    fn codes(&self, id: DeclId) -> Vec<String> {
        let (_, d) = self.elaborate();
        d[&id].iter().map(|x| x.code.as_str().to_owned()).collect()
    }

    fn message(&self, id: DeclId) -> String {
        let (_, d) = self.elaborate();
        d[&id]
            .iter()
            .map(|x| x.message.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn ok(&self, id: DeclId) {
        let (e, d) = self.elaborate();
        assert!(
            matches!(e.mappings[&id].outcome, RealizationOutcome::Elaborated(_)),
            "{:?}",
            d[&id]
        );
    }

    /// One tick of the reference evaluator with the given inputs (no
    /// domains: everything is evaluated every tick).
    fn run(&self, inputs: &[(DeclId, Value)]) -> BTreeMap<DeclId, Value> {
        let (e, d) = self.elaborate();
        for (id, ds) in &d {
            assert!(ds.iter().all(|x| !x.is_error()), "{id}: {ds:?}");
        }
        let input = TickInput {
            values: inputs.iter().cloned().collect(),
        };
        let o = step(&e.ir, 0, &BTreeSet::new(), &State::default(), &input).unwrap();
        o.values
    }
}

fn q(v: f64) -> Value {
    Value::scalar(v)
}
fn k(v: f64) -> Value {
    Value::q(Dim::TEMPERATURE, v)
}
fn sem(f: &Fixture, name: &str, v: Value) -> Value {
    Value::sem(f.c(name), v)
}
fn inner(v: &Value) -> Value {
    v.unwrap_semantic().unwrap().clone()
}

// ---- A: clamp brightness ------------------------------------------------------

#[test]
fn a_clamp_brightness_keeps_the_concept_and_needs_the_order() {
    let mut f = Fixture::new();
    let b_in = f.mapping("bIn", &[], "Brightness", None);
    f.mapping("bMin", &[], "Brightness", Some("0.1"));
    f.mapping("bMax", &[], "Brightness", Some("0.9"));
    let out = f.mapping("bOut", &[], "Brightness", Some("clamp(bIn, bMin, bMax)"));
    f.ok(out);
    let v = f.run(&[(b_in, sem(&f, "Brightness", q(0.05)))]);
    assert_eq!(v[&out], sem(&f, "Brightness", q(0.1)));
    let v = f.run(&[(b_in, sem(&f, "Brightness", q(2.0)))]);
    assert_eq!(v[&out], sem(&f, "Brightness", q(0.9)));
    // the same with Brightness not declared ordered: no default order
    let s = apply_edit(
        &f.s,
        &EditOp::SetConceptOrdered {
            id: f.c("Brightness"),
            ordered: false,
        },
    )
    .unwrap()
    .snapshot;
    f.s = s;
    assert_eq!(f.codes(out), vec!["semantic.no_order"]);
    assert!(
        f.message(out).contains("have no default order"),
        "{}",
        f.message(out)
    );
}

// ---- B: mode in a finite list ---------------------------------------------------

#[test]
fn b_mode_membership_in_a_finite_list() {
    let mut f = Fixture::new();
    let mode = f.mapping("mode", &[], "Mode", None);
    let auto = f.mapping("isAuto", &[], "Held", Some("mode in [1, 2]"));
    let auto2 = f.mapping("isAuto2", &[], "Held", Some("contains(mode, [1, 2, 2])"));
    let none = f.mapping("never", &[], "Held", Some("mode in []"));
    f.ok(auto);
    let v = f.run(&[(mode, sem(&f, "Mode", q(2.0)))]);
    assert_eq!(v[&auto], sem(&f, "Held", Value::boolean(true)));
    assert_eq!(v[&auto2], sem(&f, "Held", Value::boolean(true)));
    assert_eq!(v[&none], sem(&f, "Held", Value::boolean(false)));
    let v = f.run(&[(mode, sem(&f, "Mode", q(3.0)))]);
    assert_eq!(v[&auto], sem(&f, "Held", Value::boolean(false)));
}

// ---- C, D: all / any over readings ------------------------------------------------

#[test]
fn c_d_all_below_and_any_above_thresholds() {
    let mut f = Fixture::new();
    let temps = f.mapping("temps", &[], "Readings", None);
    let faults = f.mapping("faults", &[], "Severities", None);
    let all_below = f.mapping("allBelow", &[], "Held", Some("all(temps, t => t < 30 K)"));
    let any_high = f.mapping("anyHigh", &[], "Held", Some("any(faults, s => s > 2)"));
    let v = f.run(&[
        (
            temps,
            sem(&f, "Readings", Value::list([k(20.0), k(25.0), k(28.0)])),
        ),
        (faults, sem(&f, "Severities", Value::list([q(1.0), q(3.0)]))),
    ]);
    assert_eq!(inner(&v[&all_below]), Value::boolean(true));
    assert_eq!(inner(&v[&any_high]), Value::boolean(true));
    let v = f.run(&[
        (temps, sem(&f, "Readings", Value::list([k(20.0), k(35.0)]))),
        (faults, sem(&f, "Severities", Value::list([q(1.0), q(2.0)]))),
    ]);
    assert_eq!(inner(&v[&all_below]), Value::boolean(false));
    assert_eq!(inner(&v[&any_high]), Value::boolean(false));
    // an empty collection: all holds, any does not
    let v = f.run(&[
        (temps, sem(&f, "Readings", Value::list([]))),
        (faults, sem(&f, "Severities", Value::list([]))),
    ]);
    assert_eq!(inner(&v[&all_below]), Value::boolean(true));
    assert_eq!(inner(&v[&any_high]), Value::boolean(false));
}

// ---- E: a pair of temperature and humidity ---------------------------------------

#[test]
fn e_pair_of_temperature_and_humidity() {
    let mut f = Fixture::new();
    let t = f.mapping("t", &[], "Temperature", None);
    let h = f.mapping("h", &[], "Humidity", None);
    let th = f.mapping("th", &[], "Climate", Some("(t, h)"));
    let th_t = f.mapping("thT", &[], "Temperature", Some("first(th)"));
    let th_h = f.mapping("thH", &[], "Humidity", Some("second(th)"));
    let swapped = f.mapping("swapped", &[], "Humidity", Some("first(swap(th))"));
    let v = f.run(&[
        (t, sem(&f, "Temperature", k(25.0))),
        (h, sem(&f, "Humidity", q(45.0))),
    ]);
    assert_eq!(inner(&v[&th]), Value::pair(k(25.0), q(45.0)));
    assert_eq!(inner(&v[&th_t]), k(25.0));
    assert_eq!(inner(&v[&th_h]), q(45.0));
    assert_eq!(inner(&v[&swapped]), q(45.0));
}

// ---- F, G: map a calibration, zip two collections ------------------------------

#[test]
fn f_g_map_a_calibration_and_zip_two_collections() {
    let mut f = Fixture::new();
    let temps = f.mapping("temps", &[], "Readings", None);
    let calibrated = f.mapping(
        "calibrated",
        &[],
        "Readings",
        Some("map(temps, t => t + 5 K)"),
    );
    let sevs = f.mapping("sevs", &[], "Severities", None);
    let count = f.mapping("count", &[], "Humidity", Some("length(zip(temps, sevs))"));
    let total = f.mapping("total", &[], "Humidity", Some("sum(sevs)"));
    let warm = f.mapping(
        "warm",
        &[],
        "Readings",
        Some("filter(temps, t => t > 22 K)"),
    );
    let both = f.mapping("both", &[], "Severities", Some("append(sevs, [9])"));
    let v = f.run(&[
        (
            temps,
            sem(&f, "Readings", Value::list([k(20.0), k(25.0), k(28.0)])),
        ),
        (sevs, sem(&f, "Severities", Value::list([q(7.0), q(8.0)]))),
    ]);
    assert_eq!(
        inner(&v[&calibrated]),
        Value::list([k(25.0), k(30.0), k(33.0)])
    );
    assert_eq!(inner(&v[&count]), q(2.0));
    assert_eq!(inner(&v[&total]), q(15.0));
    assert_eq!(inner(&v[&warm]), Value::list([k(25.0), k(28.0)]));
    assert_eq!(inner(&v[&both]), Value::list([q(7.0), q(8.0), q(9.0)]));
    // zip truncates to the shorter and pairs in order
    let (e, _) = f.elaborate();
    let zipped = f.mapping("zipped", &[], "Held", Some("length(zip(temps, sevs)) == 2"));
    let _ = e;
    let v = f.run(&[
        (
            temps,
            sem(&f, "Readings", Value::list([k(1.0), k(2.0), k(3.0)])),
        ),
        (sevs, sem(&f, "Severities", Value::list([q(7.0), q(8.0)]))),
    ]);
    assert_eq!(inner(&v[&zipped]), Value::boolean(true));
}

// ---- H: Option fallback ----------------------------------------------------------

#[test]
fn h_option_fallback_and_option_equations() {
    let mut f = Fixture::new();
    let temps = f.mapping("temps", &[], "Readings", None);
    let first_or = f.mapping(
        "firstOr",
        &[],
        "Temperature",
        Some("getOrElse(head(temps), 0 K)"),
    );
    let doubled = f.mapping(
        "doubled",
        &[],
        "Temperature",
        Some("optElim(head(temps), 0 K, t => t + t)"),
    );
    let maybe = f.mapping(
        "maybe",
        &[],
        "MaybeTemp",
        Some("mapOpt(head(temps), t => t + 1 K)"),
    );
    let v = f.run(&[(temps, sem(&f, "Readings", Value::list([k(20.0), k(25.0)])))]);
    assert_eq!(inner(&v[&first_or]), k(20.0));
    assert_eq!(inner(&v[&doubled]), k(40.0));
    assert_eq!(inner(&v[&maybe]), Value::some(k(21.0)));
    let v = f.run(&[(temps, sem(&f, "Readings", Value::list([])))]);
    assert_eq!(inner(&v[&first_or]), k(0.0));
    assert_eq!(inner(&v[&doubled]), k(0.0));
    assert_eq!(inner(&v[&maybe]), Value::None);
}

// ---- I: dimension-preserving min -------------------------------------------------

#[test]
fn i_min_preserves_dimensions_and_refuses_mixed_ones() {
    let mut f = Fixture::new();
    let len = f.mapping("len", &[], "Length", None);
    let dur = f.mapping("dur", &[], "Duration", None);
    let shorter = f.mapping("shorter", &[], "Length", Some("min(len, 2 m)"));
    let v = f.run(&[
        (len, sem(&f, "Length", Value::q(Dim::LENGTH, 3.0))),
        (dur, sem(&f, "Duration", Value::q(Dim::TIME, 1.0))),
    ]);
    assert_eq!(inner(&v[&shorter]), Value::q(Dim::LENGTH, 2.0));
    let mixed = f.mapping("mixed", &[], "Length", Some("min(len, dur)"));
    assert_eq!(f.codes(mixed), vec!["semantic.concept_mismatch"]);
    // a Length beside a plain time: the concept is named, the time refused
    let mixed2 = f.mapping("mixed2", &[], "Length", Some("min(len, 2 s)"));
    assert_eq!(f.codes(mixed2), vec!["formula.equation.argument"]);
    assert!(f.message(mixed2).contains("Length") && f.message(mixed2).contains("a time"));
    // two plain quantities of different dimensions
    let mixed3 = f.mapping("mixed3", &[], "Length", Some("min(2 m, 2 s)"));
    assert_eq!(f.codes(mixed3), vec!["dimension.mismatch"]);
    // and through a collection: a sum keeps the dimension of its elements
    let temps = f.mapping("temps", &[], "Readings", None);
    let total = f.mapping("total", &[], "Temperature", Some("sum(temps)"));
    let wrong = f.mapping("wrong", &[], "Length", Some("sum(temps)"));
    f.ok(total);
    assert_eq!(f.codes(wrong), vec!["realization.type_mismatch"]);
    let _ = temps;
}

// ---- J: nominal-concept-preserving min ------------------------------------------

#[test]
fn j_min_at_brightness_keeps_brightness_and_refuses_opacity() {
    let mut f = Fixture::new();
    let b1 = f.mapping("b1", &[], "Brightness", None);
    let b2 = f.mapping("b2", &[], "Brightness", None);
    let o = f.mapping("o", &[], "Opacity", None);
    let dimmer = f.mapping("dimmer", &[], "Brightness", Some("min(b1, b2)"));
    let v = f.run(&[
        (b1, sem(&f, "Brightness", q(0.7))),
        (b2, sem(&f, "Brightness", q(0.4))),
        (o, sem(&f, "Opacity", q(0.1))),
    ]);
    assert_eq!(v[&dimmer], sem(&f, "Brightness", q(0.4)));
    // Brightness and Opacity are both dimensionless, and never the same
    let mixed = f.mapping("mixed", &[], "Brightness", Some("min(b1, o)"));
    assert_eq!(f.codes(mixed), vec!["semantic.concept_mismatch"]);
    assert!(f
        .message(mixed)
        .contains("Brightness and Opacity are different concepts"));
    let eq = f.mapping("eq", &[], "Held", Some("b1 == o"));
    assert_eq!(f.codes(eq), vec!["semantic.concept_mismatch"]);
    let member = f.mapping("member", &[], "Held", Some("o in [b1, b2]"));
    assert_eq!(f.codes(member), vec!["semantic.concept_mismatch"]);
    // a concept met by its plain representation is observed, as by `<`
    let half = f.mapping("half", &[], "Brightness", Some("min(b1, 0.5)"));
    f.ok(half);
}

// ---- K, L: range membership and a piecewise rule ----------------------------------

#[test]
fn k_l_range_membership_and_a_piecewise_rule() {
    let mut f = Fixture::new();
    let hum = f.mapping("hum", &[], "Humidity", None);
    let temps = f.mapping("temps", &[], "Readings", None);
    let faults = f.mapping("faults", &[], "Severities", None);
    let in_rng = f.mapping("inRng", &[], "Held", Some("inRange(hum, 30, 60)"));
    let in_iv = f.mapping("inIv", &[], "Held", Some("inInterval(hum, (30, 60))"));
    let all_below = f.mapping("allBelow", &[], "Held", Some("all(temps, t => t < 30 K)"));
    let any_high = f.mapping("anyHigh", &[], "Held", Some("any(faults, s => s > 2)"));
    let level = f.mapping(
        "level",
        &[],
        "Humidity",
        Some("if inRng && allBelow then 1 else if anyHigh then 2 else 0"),
    );
    let v = f.run(&[
        (hum, sem(&f, "Humidity", q(45.0))),
        (temps, sem(&f, "Readings", Value::list([k(20.0)]))),
        (faults, sem(&f, "Severities", Value::list([q(3.0)]))),
    ]);
    assert_eq!(inner(&v[&in_rng]), Value::boolean(true));
    assert_eq!(inner(&v[&in_iv]), Value::boolean(true));
    assert_eq!(inner(&v[&level]), q(1.0));
    let v = f.run(&[
        (hum, sem(&f, "Humidity", q(70.0))),
        (temps, sem(&f, "Readings", Value::list([k(20.0)]))),
        (faults, sem(&f, "Severities", Value::list([q(3.0)]))),
    ]);
    assert_eq!(inner(&v[&in_rng]), Value::boolean(false));
    assert_eq!(inner(&v[&level]), q(2.0));
    let _ = (all_below, any_high);
}

// ---- M, N and the Phase-9c boundary ----------------------------------------------

#[test]
fn m_n_equality_of_modes_accepted_ordering_rejected() {
    let mut f = Fixture::new();
    let m1 = f.mapping("m1", &[], "Mode", None);
    let m2 = f.mapping("m2", &[], "Mode", None);
    let same = f.mapping("same", &[], "Held", Some("m1 == m2"));
    let differ = f.mapping("differ", &[], "Held", Some("m1 != m2"));
    let v = f.run(&[(m1, sem(&f, "Mode", q(1.0))), (m2, sem(&f, "Mode", q(1.0)))]);
    assert_eq!(inner(&v[&same]), Value::boolean(true));
    assert_eq!(inner(&v[&differ]), Value::boolean(false));
    // Mode < Mode, min(Mode, Mode): no default order, however it is encoded
    let less = f.mapping("less", &[], "Held", Some("m1 < m2"));
    assert_eq!(f.codes(less), vec!["semantic.no_order"]);
    assert!(f
        .message(less)
        .contains("Mode values can be compared for equality, but they have no default order"));
    let smallest = f.mapping("smallest", &[], "Mode", Some("min(m1, m2)"));
    assert_eq!(f.codes(smallest), vec!["semantic.no_order"]);
    // the comparator escape hatch needs no declaration
    let by_rule = f.mapping(
        "byRule",
        &[],
        "Mode",
        Some("minBy(m1, m2, (a, b) => a == m1)"),
    );
    f.ok(by_rule);
    // but declaring Mode ordered admits it
    let s = apply_edit(
        &f.s,
        &EditOp::SetConceptOrdered {
            id: f.c("Mode"),
            ordered: true,
        },
    )
    .unwrap()
    .snapshot;
    f.s = s;
    f.ok(less);
    f.ok(smallest);
}

#[test]
fn equality_accepted_on_every_data_kind_ordering_rejected_off_quantities() {
    let mut f = Fixture::new();
    let th = f.mapping("th", &[], "Climate", None);
    let temps = f.mapping("temps", &[], "Readings", None);
    let held = f.mapping("held", &[], "Held", None);
    let flags = f.mapping("flags", &[], "Flags", None);
    let mt = f.mapping("mt", &[], "MaybeTemp", None);
    let lvl = f.mapping("lvl", &[], "Level", None);
    // accepted: pair, list, option, bool, count
    let pair_eq = f.mapping("pairEq", &[], "Held", Some("th == th"));
    let list_eq = f.mapping(
        "listEq",
        &[],
        "Held",
        Some("temps == reverse(reverse(temps))"),
    );
    let opt_eq = f.mapping("optEq", &[], "Held", Some("head(temps) == None"));
    let bool_eq = f.mapping("boolEq", &[], "Held", Some("held == true"));
    let count_eq = f.mapping(
        "countEq",
        &[],
        "Held",
        Some("match lvl { 3 => true, _ => false }"),
    );
    let lit_eq = f.mapping("litEq", &[], "Held", Some("(1, true) == (1, true)"));
    let v = f.run(&[
        (th, sem(&f, "Climate", Value::pair(k(1.0), q(2.0)))),
        (temps, sem(&f, "Readings", Value::list([k(1.0), k(2.0)]))),
        (held, sem(&f, "Held", Value::boolean(true))),
        (flags, sem(&f, "Flags", Value::list([Value::boolean(true)]))),
        (mt, sem(&f, "MaybeTemp", Value::None)),
        (lvl, sem(&f, "Level", Value::Nat { value: 3 })),
    ]);
    for (id, want) in [
        (pair_eq, true),
        (list_eq, true),
        (opt_eq, false),
        (bool_eq, true),
        (count_eq, true),
        (lit_eq, true),
    ] {
        assert_eq!(inner(&v[&id]), Value::boolean(want), "{id}");
    }
    // rejected: Pair < Pair, List < List, None < Some(x), Bool < Bool
    for (name, src) in [
        ("pairLt", "th < th"),
        ("listLt", "temps < temps"),
        ("optLt", "None < head(temps)"),
        ("boolLt", "held < true"),
        ("minPair", "min(th, th)"),
        ("minList", "min(temps, temps)"),
        ("minOpt", "min(mt, mt)"),
        ("minBool", "min(held, held)"),
        ("clampFlags", "clamp(flags, flags, flags)"),
    ] {
        let id = f.mapping(name, &[], "Held", Some(src));
        assert_eq!(
            f.codes(id),
            vec!["semantic.no_order"],
            "{name}: {}",
            f.message(id)
        );
    }
}

#[test]
fn rules_are_only_ever_arguments_and_say_what_they_need() {
    let mut f = Fixture::new();
    f.mapping("temps", &[], "Readings", None);
    let bad = [
        ("ruleValue", "x => x", "formula.rule.not_a_value"),
        ("noRule", "any(temps, temps)", "formula.rule.expected"),
        ("ruleWhereValue", "sum(t => t)", "formula.rule.unexpected"),
        ("arity", "any(temps)", "formula.equation.arity"),
        (
            "ruleArity",
            "any(temps, (a, b) => true)",
            "formula.rule.arity",
        ),
        ("notList", "any(3, t => true)", "formula.equation.argument"),
        (
            "unknownRule",
            "map(temps, t => nothing)",
            "formula.name.unknown",
        ),
        ("badBody", "all(temps, t => t)", "formula.equation.argument"),
    ];
    for (name, src, code) in bad {
        let id = f.mapping(name, &[], "Held", Some(src));
        assert!(
            f.codes(id).contains(&code.to_string()),
            "{name}: {:?} {}",
            f.codes(id),
            f.message(id)
        );
    }
    // memory stays outside every rule
    let mem = f.mapping(
        "mem",
        &[],
        "Held",
        Some("any(temps, t => delay(false, true))"),
    );
    assert!(f
        .codes(mem)
        .contains(&"formula.temporal.under_binder".to_string()));
    // a rule's parameter shadows an outer name, lexically
    let shadow = f.mapping(
        "shadow",
        &[],
        "Held",
        Some("any(temps, temps => temps > 1 K)"),
    );
    f.ok(shadow);
}

#[test]
fn a_large_collection_folds_maps_filters_and_appends_in_linear_time() {
    let mut f = Fixture::new();
    let sevs = f.mapping("sevs", &[], "Severities", None);
    let pipeline = f.mapping(
        "pipeline",
        &[],
        "Humidity",
        Some("sum(filter(map(append(sevs, sevs), s => s * 2), s => s > 1))"),
    );
    let n = 20_000u32;
    let raw: Vec<f64> = (0..n).map(|i| f64::from(i % 3)).collect();
    let items = Value::list(raw.iter().map(|x| q(*x)));
    let started = std::time::Instant::now();
    let v = f.run(&[(sevs, sem(&f, "Severities", items))]);
    let elapsed = started.elapsed();
    // the same pipeline in Rust: append, map (×2), filter (> 1), sum
    let expected: f64 = raw
        .iter()
        .chain(&raw)
        .map(|x| x * 2.0)
        .filter(|x| *x > 1.0)
        .sum();
    assert_eq!(inner(&v[&pipeline]), q(expected));
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "{elapsed:?}: pathological collection behaviour"
    );
}

// ---- ISS-0012 audit: every mixed comparison over overlapping representations ----

/// Brightness (ordered) and Opacity (not) are both dimensionless
/// quantities, Temperature is a kelvin quantity, Mode a count: the
/// representations overlap, the concepts never do.  Every pairing of the
/// comparison forms is pinned here so the mixed-comparison rule of
/// `docs/spec/equation-library.md` §Mixed comparisons is a tested fact:
///
/// * two values of one concept: `==` always; `<`, `min`, `max`, `clamp`
///   only while the concept is declared ordered; `minBy` always;
/// * two different concepts: never, whatever the representations;
/// * a concept beside a plain value of its representation: the concept
///   is observed and the comparison is the representation's — the
///   order declaration is not consulted (ADR-0013, the arithmetic rule),
///   and the result of `min`/`max`/`clamp` is the plain representation,
///   re-wrapped only by a mapping whose result is that concept.
#[test]
fn mixed_comparisons_over_overlapping_representations() {
    let mut f = Fixture::new();
    let b1 = f.mapping("b1", &[], "Brightness", None);
    let b2 = f.mapping("b2", &[], "Brightness", None);
    let o1 = f.mapping("o1", &[], "Opacity", None);
    let o2 = f.mapping("o2", &[], "Opacity", None);
    let t1 = f.mapping("t1", &[], "Temperature", None);
    let m1 = f.mapping("m1", &[], "Mode", None);
    let m2 = f.mapping("m2", &[], "Mode", None);
    let inputs = [
        (b1, sem(&f, "Brightness", q(0.7))),
        (b2, sem(&f, "Brightness", q(0.4))),
        (o1, sem(&f, "Opacity", q(0.7))),
        (o2, sem(&f, "Opacity", q(0.2))),
        (t1, sem(&f, "Temperature", k(300.0))),
        (m1, sem(&f, "Mode", q(2.0))),
        (m2, sem(&f, "Mode", q(1.0))),
    ];
    // ---- same concept ----
    let table: &[(&str, &str, &str, Option<&str>)] = &[
        // ordered Brightness: everything
        ("bEq", "Held", "b1 == b2", None),
        ("bLt", "Held", "b1 < b2", None),
        ("bMin", "Brightness", "min(b1, b2)", None),
        ("bMax", "Brightness", "max(b1, b2)", None),
        ("bClamp", "Brightness", "clamp(b1, b2, b2)", None),
        (
            "bMinBy",
            "Brightness",
            "minBy(b1, b2, (x, y) => x == b2)",
            None,
        ),
        // unordered Opacity: equality and the comparator only
        ("oEq", "Held", "o1 == o2", None),
        ("oLt", "Held", "o1 < o2", Some("semantic.no_order")),
        ("oMin", "Opacity", "min(o1, o2)", Some("semantic.no_order")),
        ("oMax", "Opacity", "max(o1, o2)", Some("semantic.no_order")),
        (
            "oClamp",
            "Opacity",
            "clamp(o1, o2, o2)",
            Some("semantic.no_order"),
        ),
        (
            "oMinBy",
            "Opacity",
            "minBy(o1, o2, (x, y) => x == o2)",
            None,
        ),
        // unordered Mode (a count encoding): the same
        ("mEq", "Held", "m1 == m2", None),
        ("mLt", "Held", "m1 < m2", Some("semantic.no_order")),
        ("mMin", "Mode", "min(m1, m2)", Some("semantic.no_order")),
        ("mMinBy", "Mode", "minBy(m1, m2, (x, y) => x == m2)", None),
        // ---- different concepts, same representation: never ----
        (
            "boEq",
            "Held",
            "b1 == o1",
            Some("semantic.concept_mismatch"),
        ),
        ("boLt", "Held", "b1 < o1", Some("semantic.concept_mismatch")),
        (
            "boMin",
            "Brightness",
            "min(b1, o1)",
            Some("semantic.concept_mismatch"),
        ),
        (
            "boMax",
            "Brightness",
            "max(b1, o1)",
            Some("semantic.concept_mismatch"),
        ),
        (
            "boClamp",
            "Brightness",
            "clamp(b1, o1, o2)",
            Some("semantic.concept_mismatch"),
        ),
        (
            "boMinBy",
            "Brightness",
            "minBy(b1, o1, (x, y) => true)",
            Some("semantic.concept_mismatch"),
        ),
        (
            "bmEq",
            "Held",
            "b1 == m1",
            Some("semantic.concept_mismatch"),
        ),
        ("btLt", "Held", "b1 < t1", Some("semantic.concept_mismatch")),
        // ---- a concept beside its plain representation: observed ----
        ("bPlainEq", "Held", "b1 == 0.7", None),
        ("bPlainLt", "Held", "b1 < 0.5", None),
        ("bPlainMin", "Brightness", "min(b1, 0.5)", None),
        ("oPlainEq", "Held", "o1 == 0.7", None),
        ("oPlainLt", "Held", "o1 < 0.5", None),
        ("oPlainMin", "Opacity", "min(o1, 0.5)", None),
        ("oPlainClamp", "Opacity", "clamp(o1, 0.1, 0.5)", None),
        ("mPlainLt", "Held", "m1 < 3", None),
        ("mPlainMin", "Mode", "min(m1, 3)", None),
        ("tPlainLt", "Held", "t1 < 310 K", None),
        // a plain value of another dimension: the dimension check
        ("tWrongDim", "Held", "t1 < 3", Some("dimension.mismatch")),
    ];
    let mut ids = BTreeMap::new();
    for (name, out, src, _) in table {
        ids.insert(*name, f.mapping(name, &[], out, Some(src)));
    }
    for (name, _, src, want) in table {
        let codes = f.codes(ids[name]);
        match want {
            None => assert!(codes.is_empty(), "{name} `{src}`: {}", f.message(ids[name])),
            Some(code) => assert_eq!(codes, vec![code.to_string()], "{name} `{src}`"),
        }
    }
    // the accepted ones evaluate as the representation says, and a
    // concept-typed result is the concept again
    let mut ok = Fixture::new();
    let b1 = ok.mapping("b1", &[], "Brightness", None);
    let b2 = ok.mapping("b2", &[], "Brightness", None);
    let o1 = ok.mapping("o1", &[], "Opacity", None);
    let m1 = ok.mapping("m1", &[], "Mode", None);
    let b_min = ok.mapping("bMin", &[], "Brightness", Some("min(b1, b2)"));
    let b_plain_min = ok.mapping("bPlainMin", &[], "Brightness", Some("min(b1, 0.5)"));
    let b_plain_lt = ok.mapping("bPlainLt", &[], "Held", Some("b1 < 0.5"));
    let o_plain_min = ok.mapping("oPlainMin", &[], "Opacity", Some("min(o1, 0.5)"));
    let o_plain_clamp = ok.mapping("oPlainClamp", &[], "Opacity", Some("clamp(o1, 0.1, 0.5)"));
    let m_plain_lt = ok.mapping("mPlainLt", &[], "Held", Some("m1 < 3"));
    let v = ok.run(&[
        (b1, sem(&ok, "Brightness", q(0.7))),
        (b2, sem(&ok, "Brightness", q(0.4))),
        (o1, sem(&ok, "Opacity", q(0.7))),
        (m1, sem(&ok, "Mode", q(2.0))),
    ]);
    assert_eq!(v[&b_min], sem(&ok, "Brightness", q(0.4)));
    assert_eq!(v[&b_plain_min], sem(&ok, "Brightness", q(0.5)));
    assert_eq!(inner(&v[&b_plain_lt]), Value::boolean(false));
    assert_eq!(v[&o_plain_min], sem(&ok, "Opacity", q(0.5)));
    assert_eq!(v[&o_plain_clamp], sem(&ok, "Opacity", q(0.5)));
    assert_eq!(inner(&v[&m_plain_lt]), Value::boolean(true));
    let _ = inputs;
}

/// §17 of the hardening brief: a designer who fails a capability (Data,
/// Eq, Ord) reads what the value cannot do, never how the elaborator
/// found out.  Implementation vocabulary is confined to the technical
/// line.
#[test]
fn capability_failures_speak_product_language() {
    let mut f = Fixture::new();
    f.mapping("m1", &[], "Mode", None);
    f.mapping("m2", &[], "Mode", None);
    f.mapping("o", &[], "Opacity", None);
    f.mapping("b", &[], "Brightness", None);
    f.mapping("th", &[], "Climate", None);
    f.mapping("temps", &[], "Readings", None);
    f.mapping("held", &[], "Held", None);
    let cases = [
        ("a", "Held", "m1 < m2"),
        ("b2", "Mode", "min(m1, m2)"),
        ("c2", "Mode", "clamp(m1, m2, m2)"),
        ("d", "Held", "th < th"),
        ("e2", "Held", "temps < temps"),
        ("f2", "Held", "held < true"),
        ("g", "Held", "b == o"),
        ("h2", "Brightness", "min(b, o)"),
        ("i2", "Held", "m1 == (x => x)"),
        ("j2", "Held", "any(temps, 3)"),
        ("k2", "Held", "contains(3, temps)"),
        ("l2", "Held", "inRange(m1, m2, m2)"),
    ];
    let ids: Vec<_> = cases
        .iter()
        .map(|(n, out, src)| f.mapping(n, &[], out, Some(src)))
        .collect();
    let (_, diags) = f.elaborate();
    let forbidden = [
        "scheme",
        "Scheme",
        "capability",
        "Capability",
        "solver",
        "α",
        "Ord ",
        "Ord(",
        "Cap::",
        "PTy",
        "instantiate",
        "subst",
        "Var(",
        "ordB",
        "Data",
    ];
    for ((name, _, src), id) in cases.iter().zip(&ids) {
        let ds = &diags[id];
        assert!(!ds.is_empty(), "{name} `{src}` should fail");
        for d in ds {
            for text in [&d.message, &d.explanation]
                .into_iter()
                .chain(d.fixes.iter())
            {
                for word in forbidden {
                    assert!(
                        !text.contains(word),
                        "{name} `{src}`: `{word}` in designer-facing text: {text}"
                    );
                }
            }
        }
    }
}

// ---- P11: the natural forms ----------------------------------------------------

/// `all x in xs: body` and `x in lo .. hi` are surface syntax: they lower
/// to the very Core the call forms lower to — the same expression, the
/// same typing, the same values.
#[test]
fn natural_forms_lower_to_the_same_core_as_the_call_forms() {
    let mut f = Fixture::new();
    let hum = f.mapping("hum", &[], "Humidity", None);
    let temps = f.mapping("temps", &[], "Readings", None);
    let faults = f.mapping("faults", &[], "Severities", None);
    let pairs = [
        (
            "Held",
            "all t in temps: t < 300 K",
            "all(temps, t => t < 300 K)",
        ),
        ("Held", "any s in faults: s > 2", "any(faults, s => s > 2)"),
        (
            "Severities",
            "map s in faults: s / 4",
            "map(faults, s => s / 4)",
        ),
        (
            "Readings",
            "filter t in temps: t in 250 K .. 300 K",
            "filter(temps, t => inRange(t, 250 K, 300 K))",
        ),
        ("Held", "hum in 30 .. 60", "inRange(hum, 30, 60)"),
        (
            "Held",
            "hum + 5 in 30 - 1 .. 60 + 1",
            "inRange(hum + 5, 30 - 1, 60 + 1)",
        ),
        (
            "Held",
            "all t in temps: any s in faults: t < 300 K && s > 2",
            "all(temps, t => any(faults, s => t < 300 K && s > 2))",
        ),
    ];
    let mut ids = Vec::new();
    for (i, (out, natural, call)) in pairs.iter().enumerate() {
        let a = f.mapping(&format!("n{i}"), &[], out, Some(natural));
        let b = f.mapping(&format!("c{i}"), &[], out, Some(call));
        ids.push((a, b));
    }
    let (e, d) = f.elaborate();
    for ((a, b), (_, natural, call)) in ids.iter().zip(&pairs) {
        assert!(d[a].iter().all(|x| !x.is_error()), "{natural}: {:?}", d[a]);
        let (RealizationOutcome::Elaborated(ra), RealizationOutcome::Elaborated(rb)) =
            (&e.mappings[a].outcome, &e.mappings[b].outcome)
        else {
            panic!("{natural} / {call}")
        };
        assert_eq!(
            pretty::expr(&ra.expr),
            pretty::expr(&rb.expr),
            "{natural} vs {call}"
        );
        assert_eq!(ra.expr, rb.expr, "{natural} vs {call}");
    }
    // and the values agree, tick for tick
    let inputs = [
        (hum, sem(&f, "Humidity", q(45.0))),
        (
            temps,
            sem(&f, "Readings", Value::list([k(260.0), k(310.0)])),
        ),
        (faults, sem(&f, "Severities", Value::list([q(1.0), q(3.0)]))),
    ];
    let v = f.run(&inputs);
    for (a, b) in &ids {
        assert_eq!(v[a], v[b]);
    }
    assert_eq!(inner(&v[&ids[0].0]), Value::boolean(false));
    assert_eq!(inner(&v[&ids[1].0]), Value::boolean(true));
    assert_eq!(inner(&v[&ids[3].0]), Value::list([k(260.0)]));
    assert_eq!(inner(&v[&ids[4].0]), Value::boolean(true));
    // the closed range: both ends belong
    let edge = f.mapping("edge", &[], "Held", Some("hum in 45 .. 60"));
    let edge2 = f.mapping("edge2", &[], "Held", Some("hum in 30 .. 45"));
    let v = f.run(&inputs);
    assert_eq!(inner(&v[&edge]), Value::boolean(true));
    assert_eq!(inner(&v[&edge2]), Value::boolean(true));
}

/// `x ?? d` is `getOrElse(x, d)`: the same Core, the same values, and its
/// own words when misused.
#[test]
fn coalesce_lowers_to_get_or_else() {
    let mut f = Fixture::new();
    let maybe = f.mapping("maybe", &[], "MaybeTemp", None);
    let hum = f.mapping("hum", &[], "Humidity", None);
    let a = f.mapping("a", &[], "Temperature", Some("maybe ?? 280 K"));
    let b = f.mapping("b", &[], "Temperature", Some("getOrElse(maybe, 280 K)"));
    let (e, d) = f.elaborate();
    assert!(d[&a].iter().all(|x| !x.is_error()), "{:?}", d[&a]);
    let (RealizationOutcome::Elaborated(ra), RealizationOutcome::Elaborated(rb)) =
        (&e.mappings[&a].outcome, &e.mappings[&b].outcome)
    else {
        panic!()
    };
    assert_eq!(ra.expr, rb.expr);
    let v = f.run(&[
        (maybe, sem(&f, "MaybeTemp", Value::some(k(290.0)))),
        (hum, sem(&f, "Humidity", q(1.0))),
    ]);
    assert_eq!(v[&a], v[&b]);
    assert_eq!(inner(&v[&a]), k(290.0));
    let v = f.run(&[
        (maybe, sem(&f, "MaybeTemp", Value::None)),
        (hum, sem(&f, "Humidity", q(1.0))),
    ]);
    assert_eq!(inner(&v[&a]), k(280.0));
    // misuse, in its own words
    let not_opt = f.mapping("notOpt", &[], "Humidity", Some("hum ?? 0"));
    assert_eq!(f.codes(not_opt), vec!["formula.coalesce.not_optional"]);
    assert!(
        f.message(not_opt)
            .contains("must be one that may be absent"),
        "{}",
        f.message(not_opt)
    );
    let bad_default = f.mapping("badDefault", &[], "Temperature", Some("maybe ?? 3 s"));
    assert_eq!(f.codes(bad_default), vec!["formula.coalesce.default"]);
    assert!(
        f.message(bad_default).contains("must be a temperature"),
        "{}",
        f.message(bad_default)
    );
}

/// The binder's local is one element of the collection — a temperature
/// for Readings — and lives only in the body; an inner binder shadows an
/// outer one lexically.
#[test]
fn binder_locals_are_elements_scoped_to_the_body() {
    let mut f = Fixture::new();
    f.mapping("temps", &[], "Readings", None);
    f.mapping("hum", &[], "Humidity", None);
    let typed = f.mapping("typed", &[], "Held", Some("all t in temps: t < 30"));
    assert_eq!(f.codes(typed), vec!["dimension.mismatch"]);
    assert!(
        f.message(typed).contains("a temperature"),
        "{}",
        f.message(typed)
    );
    // the local is not visible outside the body
    let outside = f.mapping(
        "outside",
        &[],
        "Held",
        Some("(all t in temps: t < 300 K) && t < 300 K"),
    );
    assert_eq!(f.codes(outside), vec!["formula.name.unknown"]);
    // shadowing: the inner `t` is the inner collection's element
    let shadow = f.mapping(
        "shadow",
        &[],
        "Held",
        Some("all t in temps: any t in [1, 2]: t > hum"),
    );
    f.ok(shadow);
    // a local may shadow a mapping, lexically
    let over = f.mapping("over", &[], "Held", Some("any hum in temps: hum > 1 K"));
    f.ok(over);
    // a name used as a plain mapping stays callable: `map`, `all`
    let named = f.mapping("all", &[], "Held", Some("true"));
    f.ok(named);
    let uses = f.mapping(
        "uses",
        &[],
        "Held",
        Some("all && (all t in temps: t < 300 K)"),
    );
    f.ok(uses);
}

#[test]
fn natural_form_mistakes_are_named_in_their_own_words() {
    let mut f = Fixture::new();
    f.mapping("temps", &[], "Readings", None);
    f.mapping("hum", &[], "Humidity", None);
    f.mapping("len", &[], "Length", None);
    f.mapping("m1", &[], "Mode", None);
    f.mapping("m2", &[], "Mode", None);
    let cases = [
        (
            "notColl",
            "Held",
            "all x in 5: true",
            "formula.binder.not_a_collection",
            "all expects a collection after 'in'.",
        ),
        (
            "notBool",
            "Readings",
            "filter x in temps: 3",
            "formula.binder.body",
            "The body of 'filter' must be true or false.",
        ),
        (
            "endpoint",
            "Held",
            "len in 2 s .. 3 s",
            "formula.range.endpoint",
            "This range endpoint must be a length.",
        ),
        // Mode values against Mode values: no default order (met by plain
        // numbers they are observed, exactly as `m1 < 1` is — ADR-0013)
        (
            "unordered",
            "Held",
            "m1 in m2 .. m2",
            "semantic.no_order",
            "no default order",
        ),
        (
            "bare",
            "Held",
            "1 .. 2",
            "formula.range.outside_in",
            "A range is written after `in`",
        ),
        (
            "mapBody",
            "Severities",
            "map t in temps: nothing",
            "formula.name.unknown",
            "nothing",
        ),
    ];
    for (name, out, src, code, words) in cases {
        let id = f.mapping(name, &[], out, Some(src));
        assert!(
            f.codes(id).contains(&code.to_string()),
            "{name}: {:?} {}",
            f.codes(id),
            f.message(id)
        );
        assert!(f.message(id).contains(words), "{name}: {}", f.message(id));
    }
    let (_, d) = f.elaborate();
    let endpoint = d
        .values()
        .flatten()
        .find(|x| x.code.as_str() == "formula.range.endpoint")
        .unwrap();
    assert_eq!(
        endpoint.explanation,
        "Both ends of the range must be comparable with len."
    );
    // a mismatch inside a binder's body is the body's own, in the
    // equation's words, not the binder's
    let inner_ = f.mapping(
        "inner",
        &[],
        "Held",
        Some("all t in temps: inRange(t, 1, 2)"),
    );
    assert!(
        !f.codes(inner_)
            .contains(&"formula.binder.not_a_collection".to_string()),
        "{:?}",
        f.codes(inner_)
    );
    // memory stays outside every binder, natural or not
    let mem = f.mapping(
        "mem",
        &[],
        "Held",
        Some("any t in temps: delay(false, true)"),
    );
    assert!(f
        .codes(mem)
        .contains(&"formula.temporal.under_binder".to_string()));
}
