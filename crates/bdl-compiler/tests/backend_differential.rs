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
