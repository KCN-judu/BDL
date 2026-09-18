//! Physical outputs (`BDL_FV/BDL/Core/Output.lean`).
//!
//! A declaration computes a value; it does not move hardware.  Physical
//! effect happens only through an explicit drive edge `β d = some o` from
//! a declaration to a nominal sink `o`, and the edge neither converts nor
//! transports: the driver's expected type must *equal* the sink's accepted
//! type and the driver's domain must *equal* the sink's domain
//! (`DriveWF`).  Each sink has at most one driver (`SingleDriver`); every
//! combination of contributors is ordinary computation upstream of that
//! one edge.  Executable designs additionally drive every required sink
//! (`CompleteOutputs`); partial designs need not.
//!
//! Nothing here touches evaluation: [`output_values`] merely projects a
//! simulation sample through the drive edges.

#![forbid(unsafe_code)]

use bdl_check::pretty;
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_ir::DesignIr;
use bdl_model::{DeclId, OutputId};
use bdl_reactive::{TickSample, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Why a drive edge is not well formed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EdgeFault {
    /// The sink is not declared (in this IR: has no domain yet).
    UnknownOutput,
    /// Driver's expected type ≠ sink's accepted type.
    TypeMismatch {
        expected: bdl_ir::Ty,
        found: bdl_ir::Ty,
    },
    /// Driver's domain ≠ sink's domain (`None` = agnostic driver).
    ClockMismatch {
        expected: bdl_model::ClockId,
        found: Option<bdl_model::ClockId>,
    },
}

/// Where a sink stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputState {
    /// No driver; fine for a partial design, missing for an executable one.
    Undriven,
    /// Exactly one well-formed driver.
    Driven,
    /// A driver whose edge is ill formed.
    IllFormed,
    /// More than one declaration claims the sink.
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputAnalysis {
    /// `d → o` edges that satisfy `DriveWF`.
    pub valid_bindings: BTreeMap<DeclId, OutputId>,
    /// Ill-formed edges, by driver.
    pub faults: BTreeMap<DeclId, EdgeFault>,
    /// Sinks with more than one driver, and who claims them.
    pub conflicts: BTreeMap<OutputId, BTreeSet<DeclId>>,
    /// Required sinks with no driver.
    pub missing_required: BTreeSet<OutputId>,
    pub states: BTreeMap<OutputId, OutputState>,
    /// `DriveWF ∧ SingleDriver`.
    pub partial_wf: bool,
    /// `partial_wf ∧ CompleteOutputs required`.
    pub executable: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Check every drive edge, the single-driver invariant, and completeness
/// against `required`.  Sinks must have a domain to be in `ir.outputs`; a
/// sink still without one is simply absent here and reported by the
/// caller as open.
pub fn check_outputs(ir: &DesignIr, required: &BTreeSet<OutputId>) -> OutputAnalysis {
    let mut a = OutputAnalysis {
        valid_bindings: BTreeMap::new(),
        faults: BTreeMap::new(),
        conflicts: BTreeMap::new(),
        missing_required: BTreeSet::new(),
        states: ir
            .outputs
            .keys()
            .map(|o| (*o, OutputState::Undriven))
            .collect(),
        partial_wf: true,
        executable: true,
        diagnostics: Vec::new(),
    };
    let decl_name = |d: DeclId| {
        ir.decls
            .get(&d)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| d.to_string())
    };
    let out_name = |o: OutputId| {
        ir.outputs
            .get(&o)
            .map(|x| x.name.clone())
            .or_else(|| ir.output_names.get(&o).cloned())
            .unwrap_or_else(|| o.to_string())
    };
    let clock_name = |c: Option<bdl_model::ClockId>| match c {
        Some(c) => ir
            .clock_names
            .get(&c)
            .cloned()
            .unwrap_or_else(|| c.to_string()),
        None => "no domain".to_string(),
    };

    // DriveWF per edge.
    let mut claimants: BTreeMap<OutputId, BTreeSet<DeclId>> = BTreeMap::new();
    for (d, o) in &ir.drives {
        claimants.entry(*o).or_default().insert(*d);
        let entity = Entity::Mapping { id: *d };
        let Some(spec) = ir.outputs.get(o) else {
            a.faults.insert(*d, EdgeFault::UnknownOutput);
            a.diagnostics.push(
                Diagnostic::info("output.clock_unset", entity, format!("{} has no timing domain yet, so this connection cannot be checked.", out_name(*o)))
                    .explain("Say which domain the output updates in; the driver must update in the same one.")
                    .technical(format!("Ω {o} = none")),
            );
            continue;
        };
        let Some(ty) = ir.ty_view(*d) else { continue };
        if ty != &spec.accepts {
            a.faults.insert(
                *d,
                EdgeFault::TypeMismatch {
                    expected: spec.accepts.clone(),
                    found: ty.clone(),
                },
            );
            a.diagnostics.push(
                Diagnostic::error(
                    "output.type_mismatch",
                    entity,
                    format!("{} expects {}, but {} produces {}.", out_name(*o), pretty::describe(ir, &spec.accepts), decl_name(*d), pretty::describe(ir, ty)),
                )
                .explain(if matches!(ty, bdl_ir::Ty::Arr { .. }) {
                    "A relationship with inputs is not a value: an output takes the value of a relationship that has no inputs. Connect the relationship that combines the sources into the final value."
                } else {
                    "A connection to a physical output converts nothing: the relationship must already produce exactly what the output accepts."
                })
                .technical(format!("DriveWF: Δ.tyView {d} = {} ≠ Ω {o}.accepts = {}", pretty::kernel(ty), pretty::kernel(&spec.accepts))),
            );
            continue;
        }
        let dc = ir.clocks.get(d).copied();
        if dc != Some(spec.clock) {
            a.faults.insert(
                *d,
                EdgeFault::ClockMismatch {
                    expected: spec.clock,
                    found: dc,
                },
            );
            a.diagnostics.push(
                Diagnostic::error(
                    "output.clock_mismatch",
                    entity,
                    format!("{} updates in the {} domain, but its driver {} updates in {}.", out_name(*o), clock_name(Some(spec.clock)), decl_name(*d), clock_name(dc)),
                )
                .explain("A connection to a physical output transports nothing: bring the value into the output's domain first (read it there with a stated initial value), then connect.")
                .technical(format!("DriveWF: Κ {d} = {} ≠ Ω {o}.clock = {}", dc.map(|c| c.to_string()).unwrap_or_default(), spec.clock)),
            );
            continue;
        }
        a.valid_bindings.insert(*d, *o);
    }

    // SingleDriver per sink.
    for (o, ds) in &claimants {
        if ds.len() > 1 {
            a.conflicts.insert(*o, ds.clone());
            let names: Vec<String> = ds.iter().map(|d| decl_name(*d)).collect();
            for d in ds {
                a.diagnostics.push(
                    Diagnostic::error("output.multiple_drivers", Entity::Mapping { id: *d }, format!("{} already has a final target.", out_name(*o)))
                        .explain(format!("{} all connect to {}. Combine competing values upstream into one relationship, then connect that result to {}.", names.join(", "), out_name(*o), out_name(*o)))
                        .technical(format!("SingleDriver violated for {o}: {}", ds.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "))),
                );
            }
        }
    }

    // States and completeness.
    for (o, st) in a.states.iter_mut() {
        let ds = claimants.get(o);
        *st = match ds {
            None => OutputState::Undriven,
            Some(ds) if ds.len() > 1 => OutputState::Conflict,
            Some(ds) => {
                let d = ds.iter().next().copied().expect("non-empty");
                if a.valid_bindings.contains_key(&d) {
                    OutputState::Driven
                } else {
                    OutputState::IllFormed
                }
            }
        };
    }
    a.partial_wf = a.faults.is_empty() && a.conflicts.is_empty();
    for o in required {
        if !ir.outputs.contains_key(o) {
            continue;
        }
        if a.states.get(o) != Some(&OutputState::Driven)
            && !a.conflicts.contains_key(o)
            && !claimants.contains_key(o)
        {
            a.missing_required.insert(*o);
            a.diagnostics.push(
                Diagnostic::info("output.missing_driver", Entity::Project, format!("{} has no final target yet.", out_name(*o)))
                    .explain("Connect exactly one relationship that produces what the output accepts, in the output's domain.")
                    .technical(format!("CompleteOutputs: {o} undriven")),
            );
        }
    }
    a.executable = a.partial_wf && a.missing_required.is_empty();
    sort_diagnostics(&mut a.diagnostics);
    a
}

/// The values physically committed at a tick: each valid drive edge maps
/// the driver's sample to its sink.  A pure projection; no state changes.
pub fn output_values(
    sample: &TickSample,
    bindings: &BTreeMap<DeclId, OutputId>,
) -> BTreeMap<OutputId, Value> {
    bindings
        .iter()
        .filter_map(|(d, o)| sample.values.get(d).map(|v| (*o, v.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_ir::{ConceptBinding, Declaration, Expr, Interface, OutputSpec, Prim, Scalar, Ty};
    use bdl_model::{ClockId, Dim, SemanticId};

    fn d(n: u64) -> DeclId {
        DeclId::from_raw(n)
    }
    fn o(n: u64) -> OutputId {
        OutputId::from_raw(n)
    }
    fn c(n: u64) -> ClockId {
        ClockId::from_raw(n)
    }
    fn s(n: u64) -> SemanticId {
        SemanticId::from_raw(n)
    }

    /// Brightness (sem 0), MotorAngle (sem 1); light : Brightness @ c0, motor : MotorAngle @ c1.
    fn ir() -> DesignIr {
        let mut ir = DesignIr::default();
        ir.concepts.insert(
            s(0),
            ConceptBinding {
                id: s(0),
                name: "Brightness".into(),
                representation: Some(Ty::q(Dim::ZERO)),
                ordered: false,
            },
        );
        ir.concepts.insert(
            s(1),
            ConceptBinding {
                id: s(1),
                name: "MotorAngle".into(),
                representation: Some(Ty::q(Dim::ANGLE)),
                ordered: false,
            },
        );
        ir.outputs.insert(
            o(0),
            OutputSpec {
                id: o(0),
                name: "light".into(),
                accepts: Ty::sem(s(0)),
                clock: c(0),
            },
        );
        ir.outputs.insert(
            o(1),
            OutputSpec {
                id: o(1),
                name: "motor".into(),
                accepts: Ty::sem(s(1)),
                clock: c(1),
            },
        );
        ir.clock_names.insert(c(0), "interaction".into());
        ir.clock_names.insert(c(1), "motor".into());
        ir
    }

    fn decl(ir: &mut DesignIr, n: u64, name: &str, ty: Ty, clock: Option<ClockId>) {
        ir.decls.insert(
            d(n),
            Declaration {
                id: d(n),
                name: name.into(),
                interface: Interface {
                    expected_type: ty,
                    commitments: vec![],
                },
                realization: None,
            },
        );
        if let Some(cl) = clock {
            ir.clocks.insert(d(n), cl);
        }
    }

    fn required(ns: &[u64]) -> BTreeSet<OutputId> {
        ns.iter().map(|n| o(*n)).collect()
    }

    #[test]
    fn one_valid_driver_and_no_driver_are_both_partially_well_formed() {
        let mut ir = ir();
        decl(&mut ir, 0, "lampTarget", Ty::sem(s(0)), Some(c(0)));
        ir.drives.insert(d(0), o(0));
        let a = check_outputs(&ir, &required(&[0, 1]));
        assert!(a.partial_wf);
        assert_eq!(a.valid_bindings[&d(0)], o(0));
        assert_eq!(a.states[&o(0)], OutputState::Driven);
        // motor is required but undriven: not executable, not an error
        assert!(!a.executable);
        assert_eq!(a.missing_required, required(&[1]));
        assert_eq!(a.states[&o(1)], OutputState::Undriven);
        let missing = &a.diagnostics[0];
        assert_eq!(missing.code.as_str(), "output.missing_driver");
        assert!(!missing.is_error());
        assert_eq!(missing.message, "motor has no final target yet.");
        // with motor not required, the design is executable
        assert!(check_outputs(&ir, &required(&[0])).executable);
    }

    #[test]
    fn two_drivers_are_a_conflict_with_no_implicit_policy() {
        let mut ir = ir();
        decl(&mut ir, 0, "dimByTilt", Ty::sem(s(0)), Some(c(0)));
        decl(&mut ir, 1, "warmPulse", Ty::sem(s(0)), Some(c(0)));
        ir.drives.insert(d(0), o(0));
        ir.drives.insert(d(1), o(0));
        let a = check_outputs(&ir, &required(&[0]));
        assert!(!a.partial_wf && !a.executable);
        assert_eq!(a.conflicts[&o(0)], [d(0), d(1)].into_iter().collect());
        assert_eq!(a.states[&o(0)], OutputState::Conflict);
        assert_eq!(a.diagnostics[0].code.as_str(), "output.multiple_drivers");
        assert_eq!(
            a.diagnostics[0].message,
            "light already has a final target."
        );
        assert!(a.diagnostics[0]
            .explanation
            .contains("Combine competing values upstream"));
    }

    #[test]
    fn wrong_clock_and_wrong_type_are_rejected_exact_match_accepted() {
        let mut ir = ir();
        decl(&mut ir, 0, "fastTarget", Ty::sem(s(1)), Some(c(0))); // right type, wrong clock
        decl(&mut ir, 1, "brightForMotor", Ty::sem(s(0)), Some(c(1))); // wrong type, right clock
        decl(&mut ir, 2, "motorTarget", Ty::sem(s(1)), Some(c(1))); // exact
        decl(&mut ir, 3, "rawAngle", Ty::q(Dim::ANGLE), Some(c(1))); // representation, not the concept
        for n in 0..4 {
            ir.drives.insert(d(n), o(1));
        }
        let a = check_outputs(&ir, &required(&[]));
        assert!(matches!(a.faults[&d(0)], EdgeFault::ClockMismatch { .. }));
        assert!(matches!(a.faults[&d(1)], EdgeFault::TypeMismatch { .. }));
        assert!(matches!(a.faults[&d(3)], EdgeFault::TypeMismatch { .. }));
        assert_eq!(
            a.valid_bindings.keys().copied().collect::<Vec<_>>(),
            vec![d(2)]
        );
        let clock = a
            .diagnostics
            .iter()
            .find(|x| x.code.as_str() == "output.clock_mismatch")
            .unwrap();
        assert_eq!(
            clock.message,
            "motor updates in the motor domain, but its driver fastTarget updates in interaction."
        );
        let ty = a
            .diagnostics
            .iter()
            .find(|x| x.code.as_str() == "output.type_mismatch")
            .unwrap();
        assert_eq!(
            ty.message,
            "motor expects MotorAngle, but brightForMotor produces Brightness."
        );
    }

    #[test]
    fn upstream_composition_with_one_final_driver_is_accepted() {
        // base, corr : Brightness contribute; final := max-ish combination drives.
        let mut ir = ir();
        decl(&mut ir, 0, "base", Ty::sem(s(0)), Some(c(0)));
        decl(&mut ir, 1, "corr", Ty::sem(s(0)), Some(c(0)));
        decl(&mut ir, 2, "lampTarget", Ty::sem(s(0)), Some(c(0)));
        ir.decls.get_mut(&d(2)).unwrap().realization = Some(Expr::mk(
            s(0),
            Expr::apps(
                Expr::prim(Prim::Add { dim: Dim::ZERO }),
                [Expr::rep(Expr::decl(d(0))), Expr::rep(Expr::decl(d(1)))],
            ),
        ));
        ir.drives.insert(d(2), o(0));
        let a = check_outputs(&ir, &required(&[0]));
        assert!(a.partial_wf && a.executable);
        assert_eq!(a.valid_bindings.len(), 1);
    }

    #[test]
    fn output_values_project_samples_through_edges() {
        let sample = TickSample {
            tick: 3,
            active: BTreeSet::new(),
            values: [
                (d(2), Value::sem(s(0), Value::scalar(0.5))),
                (d(0), Value::scalar(9.0)),
            ]
            .into_iter()
            .collect(),
        };
        let bindings: BTreeMap<DeclId, OutputId> = [(d(2), o(0))].into_iter().collect();
        let out = output_values(&sample, &bindings);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[&o(0)],
            Value::sem(
                s(0),
                Value::Quantity {
                    dim: Dim::ZERO,
                    value: 0.5
                }
            )
        );
        let _ = Prim::Lit {
            dim: Dim::ZERO,
            value: Scalar(0.0),
        };
    }
}
