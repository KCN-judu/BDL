//! The central acceptance criterion of the backend: for the same design,
//! schedule and inputs the generated Rust program and the reference
//! evaluator produce the same trace — values, temporal results, outputs,
//! errors.  Every case is generated, `cargo check`ed as a `no_std`
//! library, built with its host bridge, and run.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::{CompileOptions, MappingStatus};
use bdl_ir::{DesignIr, Expr};
use bdl_runtime_host::DynValue;
use support::*;

#[test]
fn lamp_brightness_is_zero_third_two_thirds_one() {
    let case = corpus().into_iter().find(|c| c.name == "lamp").unwrap();
    let (art, trace) = differential(&case);
    let b = quantities(&value_of(&trace, &art, d(2)));
    let expected = [
        0.0,
        30f64.to_radians() / std::f64::consts::FRAC_PI_2,
        60f64.to_radians() / std::f64::consts::FRAC_PI_2,
        1.0,
    ];
    for (got, want) in b.iter().zip(expected) {
        assert_eq!(
            got.unwrap(),
            want,
            "exact: the same IEEE operations on both sides"
        );
    }
    assert!((b[1].unwrap() - 1.0 / 3.0).abs() < 1e-15 && (b[2].unwrap() - 2.0 / 3.0).abs() < 1e-15);
    // the function declaration is inlined, not a runtime value
    assert_eq!(art.exec_ir.as_ref().unwrap().functions.len(), 1);
    assert_eq!(
        art.generated.as_ref().unwrap().manifest.functions[0].name,
        "dimByTilt"
    );
}

#[test]
fn delay_reads_init_then_the_previous_tick_and_commits_after_the_tick() {
    let case = corpus().into_iter().find(|c| c.name == "delay").unwrap();
    let (art, trace) = differential(&case);
    let acc = quantities(&value_of(&trace, &art, d(1)));
    assert_eq!(
        acc,
        vec![Some(0.0), Some(1.0), Some(3.0), Some(6.0), Some(10.0)]
    );
    // state bytes: two Option<f64> cells is what `State` holds
    assert!(trace.metrics.state_bytes >= 16);
}

#[test]
fn sync_observes_only_prior_source_activations_whatever_the_order() {
    let case = corpus().into_iter().find(|c| c.name == "sync").unwrap();
    let (art, trace) = differential(&case);
    // slow (period 2) y := sync fast x: at tick 0 nothing before → -1; at
    // tick 2 the last fast activation strictly before is tick 1 → 10; …
    let y = quantities(&value_of(&trace, &art, d(1)));
    assert_eq!(
        y,
        vec![
            Some(-1.0),
            None,
            Some(10.0),
            None,
            Some(30.0),
            None,
            Some(50.0)
        ]
    );
    // fast z := sync slow y: sees y from the previous slow activation
    let z = quantities(&value_of(&trace, &art, d(2)));
    assert_eq!(
        z,
        vec![
            Some(0.0),
            Some(-1.0),
            Some(-1.0),
            Some(10.0),
            Some(10.0),
            Some(30.0),
            Some(30.0)
        ]
    );
    // the same request with the domains listed in the opposite order
    let mut req = run_request(&case, &art);
    for t in &mut req.ticks {
        t.active.reverse();
    }
    let again = cargo_for(case.name).run_host(&req).unwrap();
    assert_eq!(again.ticks, trace.ticks);
}

#[test]
fn collections_and_grouped_values_agree_and_stay_first_order() {
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "collections")
        .unwrap();
    let (art, trace) = differential(&case);
    let exec = art.exec_ir.as_ref().unwrap();
    assert!(exec.uses_lists());
    assert!(art.generated.as_ref().unwrap().manifest.requires_allocator);
    // no closure anywhere: every combinator lowered to first-order code
    let src = art.generated.as_ref().unwrap().core_source();
    assert!(src.contains("list::fold("), "{src}");
    assert!(src.contains("extern crate alloc;"));
    // tick 3: xs = [0, 1, 2]
    let at = |decl: u64| value_of(&trace, &art, d(decl))[3].clone().unwrap();
    assert_eq!(at(1), DynValue::Bool { value: false }); // any(x > 2)
    assert_eq!(at(2), DynValue::Quantity { value: 3.0 }); // sum
    assert_eq!(
        at(3),
        DynValue::List {
            items: vec![
                DynValue::Quantity { value: 5.0 },
                DynValue::Quantity { value: 6.0 },
                DynValue::Quantity { value: 7.0 }
            ]
        }
    );
    assert_eq!(
        at(4),
        DynValue::pair_of(
            DynValue::Quantity { value: 0.0 },
            DynValue::Quantity { value: 3.0 }
        )
    );
    assert_eq!(at(6), DynValue::Bool { value: true }); // xs == reverse(reverse(xs))
                                                       // remembered: the previous tick's list, [] at tick 0
    assert_eq!(
        value_of(&trace, &art, d(7))[0].clone().unwrap(),
        DynValue::List { items: vec![] }
    );
    assert_eq!(
        value_of(&trace, &art, d(7))[3].clone().unwrap(),
        DynValue::List {
            items: vec![
                DynValue::Quantity { value: 0.0 },
                DynValue::Quantity { value: 1.0 }
            ]
        }
    );
    assert_eq!(at(9), DynValue::Bool { value: true }); // contains 2
    assert_eq!(at(10), DynValue::Quantity { value: 3.0 }); // clamp(3, 1, 4)
}

#[test]
fn the_lossless_buffer_window_is_the_source_activations_since_the_last_slow_tick() {
    // Phase 9a, Theorem M: with `slow` every 3 ticks and `fast` every tick,
    // the window at a slow activation holds the fast values *strictly
    // before* it since the previous slow activation, oldest first.
    let case = corpus().into_iter().find(|c| c.name == "buffer").unwrap();
    let (art, trace) = differential(&case);
    let window = value_of(&trace, &art, d(5));
    let list = |xs: &[f64]| DynValue::List {
        items: xs
            .iter()
            .map(|v| DynValue::Quantity { value: *v })
            .collect(),
    };
    // x = 1, 2, 3, …; slow active at ticks 0, 3, 6
    assert_eq!(window[0], Some(list(&[])));
    assert_eq!(window[1], None);
    assert_eq!(window[3], Some(list(&[1.0, 2.0, 3.0])));
    assert_eq!(window[6], Some(list(&[4.0, 5.0, 6.0])));
}

/// The deployable buffer (log bounded by `take cap`, count kept apart):
/// with a sufficient capacity its window is the unbounded window at every
/// tick (the production reading of FV `bounded_buffer_agrees`); with an
/// insufficient one the oldest values of a window are the ones missing
/// (`dropOldest`), explicitly and observably.
#[test]
fn a_bounded_buffer_keeps_the_window_under_sufficient_capacity_and_drops_the_oldest_otherwise() {
    let list = |xs: &[f64]| DynValue::List {
        items: xs
            .iter()
            .map(|v| DynValue::Quantity { value: *v })
            .collect(),
    };
    let unbounded = corpus().into_iter().find(|c| c.name == "buffer").unwrap();
    let (art, trace) = differential(&unbounded);
    let reference = value_of(&trace, &art, d(5));

    let bounded = corpus()
        .into_iter()
        .find(|c| c.name == "bounded_buffer")
        .unwrap();
    let (art, trace) = differential(&bounded);
    let window = value_of(&trace, &art, d(6));
    assert_eq!(window, reference, "cap 3 ≥ required 3: the same window");
    // the log itself never holds more than `cap` values
    for v in value_of(&trace, &art, d(2)).into_iter().flatten() {
        let DynValue::List { items } = v else {
            panic!()
        };
        assert!(items.len() <= 3);
    }

    let overflowing = corpus()
        .into_iter()
        .find(|c| c.name == "overflowing_buffer")
        .unwrap();
    let (art, trace) = differential(&overflowing);
    let window = value_of(&trace, &art, d(6));
    assert_eq!(window[0], Some(list(&[])));
    assert_eq!(
        window[3],
        Some(list(&[2.0, 3.0])),
        "1 was the oldest: dropped"
    );
    assert_eq!(
        window[6],
        Some(list(&[5.0, 6.0])),
        "4 was the oldest: dropped"
    );
}

/// DI-26: `ite` is strict — a branch that can fail fails the tick even
/// when not chosen; a conditional whose branches are total gives the
/// same value whether both are built (reference) or one (generated).
#[test]
fn the_strict_conditional_fails_on_an_unchosen_failing_branch_and_lazily_otherwise() {
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "ite_strictness")
        .unwrap();
    let (art, trace) = differential(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, d(1))),
        vec![Some(1.0), Some(-1.0)]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, d(2))),
        vec![Some(0.5), Some(-0.25)]
    );
    let err = trace.error.as_ref().unwrap();
    assert_eq!(err.tick, 2, "x = 0: the unchosen 1/x still fails the tick");
    let core = art.generated.as_ref().unwrap().core_source();
    assert!(
        core.contains("if (") && core.contains("prim::ite("),
        "sign is lazy, inverse strict"
    );
}

/// Long run (§19 of the hardening brief): thousands of ticks over list
/// state, a bounded buffer and two rates, through all three engines.
/// Every list the design keeps stays within its bound, the window holds
/// exactly the source values since the previous slow activation, and two
/// runs give one trace.  The unbounded buffer's log, by contrast, grows by
/// one per tick — the deployment issue capacity validation reports.
#[test]
fn a_long_run_keeps_list_state_bounded_and_the_window_exact() {
    const TICKS: u64 = 3000;
    let mut case = corpus()
        .into_iter()
        .find(|c| c.name == "bounded_buffer")
        .unwrap();
    case.name = "bounded_buffer_long";
    case.ticks = TICKS;
    case.inputs = bdl_reactive::InputTrace::default();
    case.inputs.series(
        d(0),
        (0..TICKS).map(|n| bdl_reactive::Value::scalar(n as f64 + 1.0)),
    );
    let (art, trace) = differential(&case);
    let again = cargo_for(case.name)
        .run_host(&run_request(&case, &art))
        .unwrap();
    assert!(
        trace.ticks == again.ticks && trace.error == again.error,
        "deterministic"
    );
    let log = value_of(&trace, &art, d(2));
    let window = value_of(&trace, &art, d(6));
    for t in 0..TICKS as usize {
        let DynValue::List { items } = log[t].as_ref().unwrap() else {
            panic!()
        };
        assert!(
            items.len() <= 3,
            "tick {t}: the log holds {} values",
            items.len()
        );
        if t % 3 == 0 {
            let want: Vec<f64> = (t.saturating_sub(3)..t).map(|u| u as f64 + 1.0).collect();
            assert_eq!(quantities_of(window[t].as_ref().unwrap()), want, "tick {t}");
        } else {
            assert_eq!(window[t], None);
        }
    }
    // the same design's list state also bounded in the list_state case
    let mut case = corpus()
        .into_iter()
        .find(|c| c.name == "list_state")
        .unwrap();
    case.name = "list_state_long";
    case.ticks = TICKS;
    case.inputs = bdl_reactive::InputTrace::default();
    case.inputs.series(
        d(0),
        (0..TICKS).map(|t| {
            bdl_reactive::Value::list(
                (0..(t % 3)).map(|i| bdl_reactive::Value::scalar((t * 10 + i) as f64)),
            )
        }),
    );
    let (art, trace) = differential(&case);
    for v in value_of(&trace, &art, d(3)).into_iter().flatten() {
        let DynValue::List { items } = v else {
            panic!()
        };
        assert!(items.len() <= 3);
    }
    // the formal construction keeps everything: its log is the tick count
    let mut case = corpus().into_iter().find(|c| c.name == "buffer").unwrap();
    case.name = "buffer_long";
    case.ticks = 300;
    case.inputs = bdl_reactive::InputTrace::default();
    case.inputs.series(
        d(0),
        (0..300).map(|n| bdl_reactive::Value::scalar(n as f64)),
    );
    let (art, trace) = differential(&case);
    let log = value_of(&trace, &art, d(1));
    let DynValue::List { items } = log[299].as_ref().unwrap() else {
        panic!()
    };
    assert_eq!(items.len(), 300);
}

fn quantities_of(v: &DynValue) -> Vec<f64> {
    let DynValue::List { items } = v else {
        panic!()
    };
    items
        .iter()
        .map(|v| match v {
            DynValue::Quantity { value } => *value,
            _ => panic!(),
        })
        .collect()
}

/// Cost evidence, not a CI threshold: `cargo test -p bdl-compiler --test
/// backend_differential collections_cost -- --ignored --nocapture` prints
/// the generated core's step time per operation at three list sizes.  The
/// numbers recorded in docs/evidence/testing.md come from here.
#[test]
#[ignore]
fn collections_cost_measurement() {
    use bdl_ir::{Prim, Ty};
    use bdl_model::Dim;
    let q = Ty::q(Dim::ZERO);
    let lq = Ty::list(q.clone());
    let mut inst = bdl_equations::Instance::default();
    inst.subst.tys.insert(0, q.clone());
    inst.subst.tys.insert(1, q.clone());
    inst.subst.dims.insert(0, Dim::ZERO);
    let lib = |name: &str| (bdl_equations::lookup(name).unwrap().build)(&inst);
    let ops: Vec<(&str, Ty, Expr)> = vec![
        (
            "map",
            lq.clone(),
            Expr::apps(
                lib("map"),
                [
                    Expr::decl(d(0)),
                    Expr::lam(q.clone(), add(Expr::var(0), lit(1.0))),
                ],
            ),
        ),
        (
            "filter",
            lq.clone(),
            Expr::apps(
                lib("filter"),
                [
                    Expr::decl(d(0)),
                    Expr::lam(
                        q.clone(),
                        Expr::apps(
                            Expr::prim(Prim::Lt { dim: Dim::ZERO }),
                            [Expr::var(0), lit(1e9)],
                        ),
                    ),
                ],
            ),
        ),
        (
            "append",
            lq.clone(),
            Expr::apps(lib("append"), [Expr::decl(d(0)), Expr::decl(d(0))]),
        ),
        ("sum", q.clone(), Expr::app(lib("sum"), Expr::decl(d(0)))),
        (
            "length",
            q.clone(),
            Expr::app(lib("length"), Expr::decl(d(0))),
        ),
        (
            "zip",
            Ty::list(Ty::prod(q.clone(), q.clone())),
            Expr::apps(lib("zip"), [Expr::decl(d(0)), Expr::decl(d(0))]),
        ),
    ];
    for (op, ty, body) in ops {
        for n in [2000u64, 8000, 32000] {
            let mut ir = DesignIr::default();
            support::decl(&mut ir, 0, "xs", lq.clone(), None, Some(0));
            support::decl(&mut ir, 1, op, ty.clone(), Some(body.clone()), Some(0));
            ir.clock_names.insert(c(0), "main".into());
            let name: &'static str = Box::leak(format!("cost_{op}_{n}").into_boxed_str());
            let mut case = Case::new_named(name, ir, 10);
            case.inputs.series(
                d(0),
                (0..10).map(|_| {
                    bdl_reactive::Value::list((0..n).map(|i| bdl_reactive::Value::scalar(i as f64)))
                }),
            );
            if op == "zip" && n > 8000 {
                continue;
            }
            let (_, trace) = differential(&case);
            eprintln!(
                "cost {op} n={n}: {} ns per tick (generated core, debug build)",
                trace.metrics.step_ns / u128::from(trace.metrics.steps)
            );
        }
    }
}

#[test]
fn every_corpus_case_agrees_with_the_reference() {
    for case in corpus() {
        let (art, trace) = differential(&case);
        let g = art.generated.as_ref().unwrap();
        // Baseline metrics (docs/architecture/codegen-rust.md keeps a snapshot); `--nocapture` to see them.
        eprintln!(
            "metrics {}: core {} lines, crate {} bytes, state {} bytes, {} steps in {} ns",
            case.name,
            g.core_lines(),
            g.total_bytes(),
            trace.metrics.state_bytes,
            trace.metrics.steps,
            trace.metrics.step_ns
        );
    }
}

/// The third engine: the executable-IR interpreter agrees with the
/// reference on every corpus case, tick by tick, value by value (the
/// generated core is compared by `differential`).
#[test]
fn every_corpus_case_agrees_with_the_exec_ir_interpreter() {
    use bdl_exec_ir::interp::{self, CellState};
    use bdl_reactive::eval::{self, State, TickInput};
    for case in corpus() {
        let art = compile_case(&case);
        assert!(art.succeeded(), "{}: {:?}", case.name, art.diagnostics);
        let exec = art.exec_ir.as_ref().unwrap();
        let mut rs = State::default();
        let mut es = CellState::init(exec);
        for t in 0..case.ticks {
            let active = case.schedule.active_at(t);
            let slots: Vec<_> = active.iter().filter_map(|c| exec.clock_slot(*c)).collect();
            let input = TickInput {
                values: case
                    .inputs
                    .samples
                    .iter()
                    .filter_map(|(d, s)| s.get(&t).map(|v| (*d, v.clone())))
                    .collect(),
            };
            let in_slots: Vec<Option<bdl_reactive::Value>> = exec
                .inputs
                .iter()
                .map(|i| {
                    let id = exec.decl(i.decl).unwrap().id;
                    input.values.get(&id).cloned()
                })
                .collect();
            match (
                eval::step(&case.ir, t, &active, &rs, &input),
                interp::step(exec, t, &slots, &es, &in_slots),
            ) {
                (Ok(r), Ok(e)) => {
                    for dp in &exec.decls {
                        if !matches!(dp.kind, bdl_exec_ir::DeclKind::Computed { .. }) {
                            continue;
                        }
                        assert_eq!(
                            r.values.get(&dp.id),
                            e.values[dp.index.0 as usize].as_ref(),
                            "{}: tick {t} decl {}",
                            case.name,
                            dp.name
                        );
                    }
                    rs = r.next;
                    es = e.next.clone();
                }
                (Err(r), Err(e)) => {
                    assert_eq!(
                        ErrorKind::of_reference(&r),
                        ErrorKind::of_reference(&e),
                        "{}: tick {t}",
                        case.name
                    );
                    break;
                }
                (r, e) => panic!("{}: tick {t}: {r:?} vs {e:?}", case.name),
            }
        }
    }
}

#[test]
fn errors_are_structured_and_stop_at_the_same_tick() {
    for (name, kind, tick) in [
        ("div_by_zero", "division_by_zero", 2),
        ("non_finite", "non_finite", 1),
        ("missing_input", "missing_input", 1),
    ] {
        let case = corpus().into_iter().find(|c| c.name == name).unwrap();
        let (_, trace) = differential(&case);
        let e = trace.error.as_ref().unwrap();
        assert_eq!(e.tick, tick, "{name}");
        assert!(
            serde_json::to_string(&e.error).unwrap().contains(kind),
            "{name}: {:?}",
            e.error
        );
        assert_eq!(trace.ticks.len() as u64, tick);
    }
}

#[test]
fn outputs_are_projected_after_evaluation_and_match_output_values() {
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "lamp_output")
        .unwrap();
    let (art, trace) = differential(&case);
    let g = art.generated.as_ref().unwrap();
    assert_eq!(g.manifest.outputs.len(), 1);
    assert_eq!(g.manifest.outputs[0].symbol, "output_4");
    assert_eq!(g.manifest.outputs[0].driver_decl_id, 2);
    let light: Vec<Option<f64>> = quantities(
        &trace
            .ticks
            .iter()
            .map(|t| t.outputs[0].clone())
            .collect::<Vec<_>>(),
    );
    assert_eq!(light[3], Some(1.0));
    assert!(matches!(
        trace.ticks[0].outputs[0],
        Some(DynValue::Semantic { id: 1, .. })
    ));
    // require_complete: the lamp without an output has none to complete; with one, it is complete
    let mut opts = options();
    opts.require_complete = true;
    assert!(bdl_compiler::compile_design_ir(lamp(true), "lamp", &opts).succeeded());
}

#[test]
fn instantaneous_cycle_is_refused_before_codegen_and_delayed_cycle_runs() {
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "a", Some(Expr::decl(d(1))), Some(0));
    qdecl(&mut ir, 1, "b", Some(Expr::decl(d(0))), Some(0));
    let art = bdl_compiler::compile_design_ir(ir, "cycle", &CompileOptions::default());
    assert!(!art.succeeded());
    assert_eq!(art.diagnostics[0].code.as_str(), "backend.not_ready");
    assert!(art.diagnostics[0]
        .explanation
        .contains("instantaneous loop"));
    assert!(art
        .analysis
        .mappings
        .values()
        .all(|m| m.status == MappingStatus::Invalid));
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "delayed_cycle")
        .unwrap();
    let (art, trace) = differential(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, d(1))),
        vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)]
    );
}

#[test]
fn a_pure_declaration_serves_two_domains_identically() {
    let case = corpus().into_iter().find(|c| c.name == "agnostic").unwrap();
    let (art, trace) = differential(&case);
    let k = quantities(&value_of(&trace, &art, d(0)));
    // k is evaluated whenever anything is active: ticks 0,2,3,4,6 (periods 2 and 3)
    assert_eq!(
        k,
        vec![Some(6.0), None, Some(6.0), Some(6.0), Some(6.0), None]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, d(1))),
        vec![Some(7.0), None, Some(7.0), None, Some(7.0), None]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, d(2))),
        vec![Some(5.0), None, None, Some(5.0), None, None]
    );
}

#[test]
fn every_corpus_core_checks_as_a_no_std_library_on_its_own() {
    for case in corpus() {
        let art = compile_case(&case);
        let g = art.generated.as_ref().unwrap();
        assert!(g.core_source().starts_with("//! Generated by"));
        assert!(
            g.core_source().contains("#![no_std]")
                && g.core_source().contains("#![forbid(unsafe_code)]")
        );
        // no std, no target crate; `alloc` and `Vec` only in a program
        // that carries lists, and then declared in the manifest
        let lists = art.exec_ir.as_ref().unwrap().uses_lists();
        assert_eq!(g.manifest.requires_allocator, lists, "{}", case.name);
        let mut forbidden = vec!["std::", "BTreeMap", "HashMap", "embassy", "hal", "Box<"];
        if !lists {
            forbidden.extend(["alloc::", "Vec<", "list::"]);
        }
        for forbidden in forbidden {
            assert!(
                !g.core_source().contains(forbidden),
                "{}: core mentions {forbidden}",
                case.name
            );
        }
        let dir = generated_dir(case.name);
        bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
        cargo_for(case.name)
            .check_core()
            .unwrap_or_else(|e| panic!("{}: {e}", case.name));
    }
}
