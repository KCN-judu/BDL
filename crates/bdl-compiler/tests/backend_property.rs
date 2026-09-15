//! Property-based differential tests over a deliberately small subset of
//! the language: dimensionless quantities, constants, references,
//! `+ - * /`, `delay`, inputs, one or two domains.
//!
//! Two layers, both against the reference evaluator:
//! * in process, reference vs the exec-IR interpreter, hundreds of
//!   generated designs (`proptest`);
//! * compiled, reference vs generated Rust, a fixed seeded batch of
//!   designs so one `cargo build` covers them all (one crate per design
//!   would take minutes; the seed makes the batch reproducible).

#![allow(clippy::unwrap_used)]

mod support;

use bdl_exec_ir::interp::{self, CellState};
use bdl_ir::{DesignIr, Expr};
use bdl_model::DeclId;
use bdl_reactive::eval::{self, State, TickInput};
use bdl_reactive::{InputTrace, Schedule, Value};
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::{Config, TestRng, TestRunner};
use std::collections::BTreeSet;
use support::*;

/// An expression over declarations `0..n` (earlier ones only, so the
/// instantaneous graph is acyclic by construction; a `delay` may read any).
fn arb_expr(n: u64, own: u64, depth: u32) -> BoxedStrategy<Expr> {
    let constant = (-20.0f64..20.0)
        .prop_map(|v| lit((v * 4.0).round() / 4.0))
        .boxed();
    let leaf = if own == 0 {
        constant
    } else {
        prop_oneof![constant, (0..own).prop_map(|i| Expr::decl(d(i)))].boxed()
    };
    if depth == 0 {
        return leaf.boxed();
    }
    let inner = move || arb_expr(n, own, depth - 1);
    prop_oneof![
        3 => leaf,
        2 => (inner(), inner()).prop_map(|(a, b)| add(a, b)),
        1 => (inner(), inner()).prop_map(|(a, b)| sub(a, b)),
        1 => (inner(), inner()).prop_map(|(a, b)| mul(a, b)),
        1 => (inner(), inner()).prop_map(|(a, b)| div(a, b)),
        // delay: init instantaneous (earlier decls), operand any decl
        2 => (inner(), 0..n.max(1)).prop_map(move |(init, i)| Expr::delay(init, Expr::decl(d(i)))),
    ]
    .boxed()
}

#[derive(Clone, Debug)]
struct Gen {
    ir: DesignIr,
    inputs: InputTrace,
    schedule: Schedule,
    ticks: u64,
}

fn arb_design() -> impl Strategy<Value = Gen> {
    (2u64..6, any::<bool>()).prop_flat_map(|(n, two_domains)| {
        let decls: Vec<BoxedStrategy<(Option<Expr>, u64)>> = (0..n)
            .map(|i| {
                let clock = if two_domains { i % 2 } else { 0 };
                prop_oneof![
                    1 => Just((None, clock)),
                    4 => arb_expr(n, i, 2).prop_map(move |e| (Some(e), clock)),
                ]
                .boxed()
            })
            .collect();
        (decls, proptest::collection::vec(-10.0f64..10.0, 1..6)).prop_map(move |(decls, xs)| {
            let mut ir = DesignIr::default();
            for (i, (real, clock)) in decls.iter().enumerate() {
                qdecl(
                    &mut ir,
                    i as u64,
                    &format!("q{i}"),
                    real.clone(),
                    Some(*clock),
                );
            }
            let mut inputs = InputTrace::default();
            let ticks = xs.len() as u64;
            for (i, (real, _)) in decls.iter().enumerate() {
                if real.is_none() {
                    inputs.series(d(i as u64), xs.iter().map(|x| Value::scalar(*x + i as f64)));
                }
            }
            let mut schedule = Schedule::always(&ir);
            if two_domains {
                schedule.periods.insert(c(1), 2);
            }
            Gen {
                ir,
                inputs,
                schedule,
                ticks,
            }
        })
    })
}

/// With two domains, a declaration in domain 1 reading one in domain 0
/// instantaneously is ill-clocked; keep those designs out (the compiler
/// refuses them anyway).  Also keep out designs that are not causal.
fn well_formed(g: &Gen) -> bool {
    let a = bdl_compiler::analyze_design_ir(g.ir.clone());
    a.causality.valid
        && a.clocks.valid
        && a.mappings.values().all(|m| {
            !matches!(
                m.status,
                bdl_compiler::MappingStatus::Invalid | bdl_compiler::MappingStatus::Open
            )
        })
}

fn case_of(g: &Gen, name: &'static str) -> Case {
    Case {
        name,
        ir: g.ir.clone(),
        ticks: g.ticks,
        schedule: g.schedule.clone(),
        inputs: g.inputs.clone(),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(250))]

    #[test]
    fn reference_and_exec_ir_interpreter_agree(g in arb_design().prop_filter("well formed", well_formed)) {
        let art = compile_case(&case_of(&g, "prop"));
        prop_assert!(art.succeeded(), "{:?}", art.diagnostics);
        let exec = art.exec_ir.as_ref().unwrap();
        let mut rs = State::default();
        let mut es = CellState::init(exec);
        for t in 0..g.ticks {
            let active = g.schedule.active_at(t);
            let slots: Vec<_> = active.iter().filter_map(|c| exec.clock_slot(*c)).collect();
            let input = TickInput { values: g.inputs.samples.iter().filter_map(|(d, s)| s.get(&t).map(|v| (*d, v.clone()))).collect() };
            let in_slots: Vec<Option<Value>> = exec.inputs.iter().map(|i| { let id = exec.decl(i.decl).unwrap().id; input.values.get(&id).cloned() }).collect();
            match (eval::step(&g.ir, t, &active, &rs, &input), interp::step(exec, t, &slots, &es, &in_slots)) {
                (Ok(r), Ok(e)) => {
                    for dp in &exec.decls {
                        if let bdl_exec_ir::DeclKind::Computed { .. } = dp.kind {
                            prop_assert_eq!(r.values.get(&dp.id), e.values[dp.index.0 as usize].as_ref(), "tick {} decl {}", t, dp.id);
                        }
                    }
                    rs = r.next;
                    es = e.next.clone();
                }
                (Err(r), Err(e)) => {
                    let alts: BTreeSet<ErrorKind> = {
                        let due: Vec<DeclId> = eval::evaluated_this_tick(&g.ir, &active).into_iter().collect();
                        (0..due.len()).filter_map(|i| { let mut o = due[i..].to_vec(); o.extend_from_slice(&due[..i]); eval::step_in_order(&g.ir, t, &active, &rs, &input, &o).err() }).map(|e| ErrorKind::of_reference(&e)).collect()
                    };
                    let ek = ErrorKind::of_reference(&e);
                    prop_assert!(ek == ErrorKind::of_reference(&r) || alts.contains(&ek), "tick {t}: {r:?} vs {e:?}");
                    break;
                }
                (r, e) => prop_assert!(false, "tick {}: {:?} vs {:?}", t, r, e),
            }
        }
    }
}

/// A seeded batch of generated designs compiled to Rust and run.
#[test]
fn seeded_batch_of_random_designs_agrees_when_compiled() {
    let mut runner = TestRunner::new_with_rng(
        Config::default(),
        TestRng::from_seed(proptest::test_runner::RngAlgorithm::ChaCha, &[7; 32]),
    );
    let strategy = arb_design().prop_filter("well formed", well_formed);
    let names: [&'static str; 8] = [
        "rand_0", "rand_1", "rand_2", "rand_3", "rand_4", "rand_5", "rand_6", "rand_7",
    ];
    for name in names {
        let g = strategy.new_tree(&mut runner).unwrap().current();
        let case = case_of(&g, name);
        differential(&case);
    }
}
