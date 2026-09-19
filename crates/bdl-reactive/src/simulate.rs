//! Simulation: a schedule, input traces, and the tick loop over [`crate::eval`].
//!
//! A [`Schedule`] says which domains activate at which global tick; rates
//! are deployment data and never part of a `ClockId`.  An [`InputTrace`]
//! supplies values for unresolved declarations per tick; a missing value is
//! a structured error, never a default.  The trace records every evaluated
//! declaration's value per tick, in the design's own terms.

use crate::causality::CausalityAnalysis;
use crate::eval::{self, RuntimeError, State, TickInput};
use crate::value::Value;
use bdl_ir::DesignIr;
use bdl_model::{ClockId, DeclId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Which domains activate when.  `period` 1 = every tick; a domain
/// activates at tick `t` iff `t % period == 0`.  Domains not listed never
/// activate.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Schedule {
    pub periods: BTreeMap<ClockId, u64>,
}

impl Schedule {
    /// Every domain of the design, every tick.
    pub fn always(ir: &DesignIr) -> Schedule {
        Schedule {
            periods: ir.clocks.values().map(|c| (*c, 1)).collect(),
        }
    }
    pub fn active_at(&self, tick: u64) -> BTreeSet<ClockId> {
        self.periods
            .iter()
            .filter(|(_, p)| **p > 0 && tick % **p == 0)
            .map(|(c, _)| *c)
            .collect()
    }
}

/// Values for unresolved declarations, per declaration per tick.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct InputTrace {
    pub samples: BTreeMap<DeclId, BTreeMap<u64, Value>>,
}

impl InputTrace {
    pub fn set(&mut self, decl: DeclId, tick: u64, value: Value) {
        self.samples.entry(decl).or_default().insert(tick, value);
    }
    /// A constant series for `decl` from `values` starting at tick 0.
    pub fn series(&mut self, decl: DeclId, values: impl IntoIterator<Item = Value>) {
        for (t, v) in values.into_iter().enumerate() {
            self.set(decl, t as u64, v);
        }
    }
    fn at(&self, tick: u64) -> TickInput {
        TickInput {
            values: self
                .samples
                .iter()
                .filter_map(|(d, s)| s.get(&tick).map(|v| (*d, v.clone())))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TickSample {
    pub tick: u64,
    pub active: BTreeSet<ClockId>,
    /// Declarations due at this tick and their values, inputs echoed as fed.
    pub values: BTreeMap<DeclId, Value>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SimulationTrace {
    pub ticks: Vec<TickSample>,
}

impl SimulationTrace {
    /// The series of one declaration: `(tick, value)` for every tick it was
    /// evaluated at.
    pub fn series(&self, decl: DeclId) -> Vec<(u64, &Value)> {
        self.ticks
            .iter()
            .filter_map(|t| t.values.get(&decl).map(|v| (t.tick, v)))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, thiserror::Error, Serialize, Deserialize)]
pub enum SimulationError {
    #[error("the design is not causal; simulation needs an acyclic instantaneous graph")]
    NotCausal,
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

/// A deterministic run: the same design, schedule and inputs give the same
/// trace.  `step` never mutates anything but this struct's own fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Simulation {
    ir: DesignIr,
    schedule: Schedule,
    inputs: InputTrace,
    state: State,
    next_tick: u64,
    trace: SimulationTrace,
}

impl Simulation {
    pub fn new(
        ir: DesignIr,
        causality: &CausalityAnalysis,
        schedule: Schedule,
        inputs: InputTrace,
    ) -> Result<Simulation, SimulationError> {
        if !causality.valid {
            return Err(SimulationError::NotCausal);
        }
        Ok(Simulation {
            ir,
            schedule,
            inputs,
            state: State::default(),
            next_tick: 0,
            trace: SimulationTrace::default(),
        })
    }

    pub fn tick(&self) -> u64 {
        self.next_tick
    }
    pub fn trace(&self) -> &SimulationTrace {
        &self.trace
    }
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Evaluate the next global tick.
    pub fn step(&mut self) -> Result<&TickSample, SimulationError> {
        let tick = self.next_tick;
        let active = self.schedule.active_at(tick);
        let input = self.inputs.at(tick);
        let outcome = eval::step(&self.ir, tick, &active, &self.state, &input)?;
        self.state = outcome.next;
        self.next_tick += 1;
        self.trace.ticks.push(TickSample {
            tick,
            active,
            values: outcome.values,
        });
        Ok(self.trace.ticks.last().expect("just pushed"))
    }

    pub fn run(&mut self, ticks: u64) -> Result<&SimulationTrace, SimulationError> {
        for _ in 0..ticks {
            self.step()?;
        }
        Ok(&self.trace)
    }

    /// Back to tick 0 with the same design, schedule and inputs.
    pub fn reset(&mut self) {
        self.state = State::default();
        self.next_tick = 0;
        self.trace = SimulationTrace::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causality::check_causality;
    use crate::graph::analyze_dependencies;
    use crate::test_designs::*;
    use bdl_ir::{Declaration, Expr, Interface, Prim, Ty};
    use bdl_model::Dim;

    /// tilt : Tilt (input) · dimByTilt : Tilt → Brightness (λ) · brightness := dimByTilt tilt
    fn lamp() -> DesignIr {
        let mut ir = DesignIr::default();
        lamp_concepts(&mut ir);
        let f = Expr::Lam {
            dom: Ty::sem(s(0)),
            body: Box::new(Expr::mk(
                s(1),
                Expr::apps(
                    Expr::prim(Prim::Div {
                        d1: Dim::ANGLE,
                        d2: Dim::ANGLE,
                    }),
                    [
                        Expr::rep(Expr::Var { index: 0 }),
                        lit_dim(Dim::ANGLE, std::f64::consts::FRAC_PI_2),
                    ],
                ),
            )),
        };
        for (n, name, ty, real, clock) in [
            (0, "tilt", Ty::sem(s(0)), None, Some(c(0))),
            (
                1,
                "dimByTilt",
                Ty::arr(Ty::sem(s(0)), Ty::sem(s(1))),
                Some(f),
                None,
            ),
            (
                2,
                "brightness",
                Ty::sem(s(1)),
                Some(Expr::app(Expr::decl(d(1)), Expr::decl(d(0)))),
                Some(c(0)),
            ),
        ] {
            ir.decls.insert(
                d(n),
                Declaration {
                    id: d(n),
                    name: name.into(),
                    interface: Interface {
                        expected_type: ty,
                        commitments: vec![],
                    },
                    realization: real,
                },
            );
            if let Some(cl) = clock {
                ir.clocks.insert(d(n), cl);
            }
        }
        ir.clock_names.insert(c(0), "interaction".into());
        ir
    }

    fn deg(x: f64) -> Value {
        Value::sem(s(0), Value::q(Dim::ANGLE, x.to_radians()))
    }

    #[test]
    fn the_lamp_executes_over_four_ticks() {
        let ir = lamp();
        let causality = check_causality(&ir, &analyze_dependencies(&ir));
        assert!(causality.valid);
        let mut inputs = InputTrace::default();
        inputs.series(d(0), [deg(0.0), deg(30.0), deg(60.0), deg(90.0)]);
        let mut sim = Simulation::new(
            ir.clone(),
            &causality,
            Schedule::always(&ir),
            inputs.clone(),
        )
        .unwrap();
        let trace = sim.run(4).unwrap().clone();
        let brightness: Vec<f64> = trace
            .series(d(2))
            .iter()
            .map(|(_, v)| v.unwrap_semantic().unwrap().as_quantity().unwrap().1)
            .collect();
        for (got, want) in brightness.iter().zip([0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0]) {
            assert!((got - want).abs() < 1e-12, "{got} vs {want}");
        }
        assert!(matches!(trace.ticks[0].values[&d(2)], Value::Semantic { id, .. } if id == s(1)));
        // determinism: a fresh run is identical, and the trace serialises
        let mut again =
            Simulation::new(ir.clone(), &causality, Schedule::always(&ir), inputs).unwrap();
        assert_eq!(again.run(4).unwrap(), &trace);
        let json = serde_json::to_string(&trace).unwrap();
        let back: SimulationTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(back, trace);
    }

    #[test]
    fn a_missing_input_stops_the_run_with_the_tick_named() {
        let ir = lamp();
        let causality = check_causality(&ir, &analyze_dependencies(&ir));
        let mut inputs = InputTrace::default();
        inputs.series(d(0), [deg(0.0), deg(30.0)]);
        let mut sim =
            Simulation::new(ir.clone(), &causality, Schedule::always(&ir), inputs).unwrap();
        let err = sim.run(3).unwrap_err();
        assert_eq!(
            err,
            SimulationError::Runtime(RuntimeError::MissingInput {
                decl: d(0),
                tick: 2
            })
        );
        assert_eq!(sim.trace().ticks.len(), 2);
    }

    #[test]
    fn prev_tilt_is_a_delay_cell_and_reset_restarts() {
        // prevTilt := delay 0deg tilt  (temporal, Core level)
        let mut ir = lamp();
        ir.decls.insert(
            d(3),
            Declaration {
                id: d(3),
                name: "prevTilt".into(),
                interface: Interface {
                    expected_type: Ty::sem(s(0)),
                    commitments: vec![],
                },
                realization: Some(Expr::delay(
                    Expr::mk(s(0), lit_dim(Dim::ANGLE, 0.0)),
                    Expr::decl(d(0)),
                )),
            },
        );
        ir.clocks.insert(d(3), c(0));
        let causality = check_causality(&ir, &analyze_dependencies(&ir));
        let mut inputs = InputTrace::default();
        inputs.series(d(0), [deg(10.0), deg(20.0), deg(30.0)]);
        let mut sim =
            Simulation::new(ir.clone(), &causality, Schedule::always(&ir), inputs).unwrap();
        sim.run(3).unwrap();
        let prev: Vec<f64> = sim
            .trace()
            .series(d(3))
            .iter()
            .map(|(_, v)| {
                v.unwrap_semantic()
                    .unwrap()
                    .as_quantity()
                    .unwrap()
                    .1
                    .to_degrees()
            })
            .collect();
        assert!(
            (prev[0] - 0.0).abs() < 1e-9
                && (prev[1] - 10.0).abs() < 1e-9
                && (prev[2] - 20.0).abs() < 1e-9,
            "{prev:?}"
        );
        sim.reset();
        assert_eq!(sim.tick(), 0);
        sim.run(1).unwrap();
        assert!(
            (sim.trace().series(d(3))[0]
                .1
                .unwrap_semantic()
                .unwrap()
                .as_quantity()
                .unwrap()
                .1)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn schedules_activate_by_period_and_a_non_causal_design_is_refused() {
        let s = Schedule {
            periods: [(c(0), 1), (c(1), 3)].into_iter().collect(),
        };
        assert_eq!(s.active_at(0), [c(0), c(1)].into_iter().collect());
        assert_eq!(s.active_at(1), [c(0)].into_iter().collect());
        assert_eq!(s.active_at(3), [c(0), c(1)].into_iter().collect());
        let ir = ir_with_decls(&[(0, Some(Expr::decl(d(1)))), (1, Some(Expr::decl(d(0))))]);
        let causality = check_causality(&ir, &analyze_dependencies(&ir));
        assert_eq!(
            Simulation::new(
                ir.clone(),
                &causality,
                Schedule::always(&ir),
                InputTrace::default()
            )
            .unwrap_err(),
            SimulationError::NotCausal
        );
    }
}
