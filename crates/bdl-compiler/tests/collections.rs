//! Collections at deployment (docs/spec/deployment-capacity.md): the
//! static list bounds are sound against the runs, the report classifies
//! the corpus as documented, the deployment diagnostics say what a
//! designer can act on, and a bounded-memory target refuses an unbounded
//! design instead of dropping values.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::{
    collections_report, compile_design_ir, CollectionsReadiness, CompileOptions, MemoryPolicy,
};
use bdl_exec_ir::bounds::{Bound, Shape};
use bdl_reactive::eval::{self, State, TickInput};
use bdl_reactive::Value;
use support::*;

/// Every list in `v` is within the bound its shape states.
fn within(shape: &Shape, v: &Value) -> bool {
    match (shape, v) {
        (Shape::List { bound, elem }, Value::List { items }) => {
            let n = items.len() as u64;
            let ok = match bound {
                Bound::Finite { elements } => n <= *elements,
                Bound::Input | Bound::Unbounded => true,
            };
            ok && items.iter().all(|x| within(elem, x))
        }
        (Shape::Pair { fst, snd }, Value::Pair { fst: a, snd: b }) => {
            within(fst, a) && within(snd, b)
        }
        (Shape::Opt { inner }, Value::Some { value }) => within(inner, value),
        (s, Value::Semantic { repr, .. }) => within(s, repr),
        (Shape::Bottom, Value::List { items }) => items.is_empty(),
        _ => true,
    }
}

#[test]
fn every_bound_holds_on_every_tick_of_every_corpus_case() {
    for case in corpus() {
        let art = compile_case(&case);
        let exec = art.exec_ir.as_ref().unwrap();
        let report = art.collections.as_ref().unwrap();
        let bounds = bdl_exec_ir::bounds::analyse(exec);
        let mut state = State::default();
        for t in 0..case.ticks {
            let active = case.schedule.active_at(t);
            let input = TickInput {
                values: case
                    .inputs
                    .samples
                    .iter()
                    .filter_map(|(d, s)| s.get(&t).map(|v| (*d, v.clone())))
                    .collect(),
            };
            let Ok(out) = eval::step(&case.ir, t, &active, &state, &input) else {
                break;
            };
            for (dp, shape) in exec.decls.iter().zip(&bounds.decls) {
                if let Some(v) = out.values.get(&dp.id) {
                    assert!(
                        within(shape, v),
                        "{}: tick {t}, {} = {v:?} exceeds {shape:?}",
                        case.name,
                        dp.name
                    );
                }
            }
            for (cp, shape) in exec.cells.iter().zip(&bounds.cells) {
                if let Some(v) = out.next.cells.get(&cp.cell) {
                    assert!(
                        within(shape, v),
                        "{}: tick {t}, cell {:?} = {v:?} exceeds {shape:?}",
                        case.name,
                        cp.cell
                    );
                }
            }
            state = out.next;
        }
        let _ = report;
    }
}

fn readiness_of(name: &str) -> CollectionsReadiness {
    let case = corpus().into_iter().find(|c| c.name == name).unwrap();
    compile_case(&case).collections.unwrap().readiness
}

#[test]
fn the_corpus_is_classified_as_documented() {
    use CollectionsReadiness::*;
    for (name, want) in [
        ("lamp", ScalarOnly),
        ("delay", ScalarOnly),
        ("collections", InputBounded),
        ("list_ops", InputBounded),
        ("nested", InputBounded),
        ("list_state", InputBounded),
        ("concepts_ordered", ScalarOnly),
        ("buffer", Unbounded),
        ("bounded_buffer", Bounded),
        ("overflowing_buffer", Bounded),
    ] {
        assert_eq!(readiness_of(name), want, "{name}");
    }
}

#[test]
fn the_bounded_buffer_is_bounded_by_its_take_and_the_formal_log_is_not() {
    let case = corpus()
        .into_iter()
        .find(|c| c.name == "bounded_buffer")
        .unwrap();
    let art = compile_case(&case);
    let r = art.collections.unwrap();
    let log = r.cells.iter().find(|c| c.name == "log").unwrap();
    assert_eq!(
        log.shape,
        Shape::List {
            bound: Bound::Finite { elements: 3 },
            elem: Box::new(Shape::Scalar)
        }
    );
    // five cells: two lists of at most 3 (24 + 3·8 each), three counts
    assert_eq!(r.state_bytes_max, Some(48 * 2 + 8 * 3));
    assert!(r.tick_bytes_max.is_some());
    // the manifest carries the same
    let m = &art.generated.unwrap().manifest;
    let entry = m.collections.as_ref().unwrap();
    assert!(!entry.unbounded);
    assert_eq!(entry.state_bytes_max, r.state_bytes_max);
    assert!(entry
        .cells
        .iter()
        .any(|c| c.bound == Bound::Finite { elements: 3 }));

    let case = corpus().into_iter().find(|c| c.name == "buffer").unwrap();
    let art = compile_case(&case);
    let r = art.collections.unwrap();
    let log = r.cells.iter().find(|c| c.name == "log").unwrap();
    assert!(log.shape.is_unbounded());
    assert_eq!(r.state_bytes_max, None);
    assert!(
        art.generated
            .unwrap()
            .manifest
            .collections
            .unwrap()
            .unbounded
    );
    // a scalar design has no collections entry at all
    let lamp = corpus().into_iter().find(|c| c.name == "lamp").unwrap();
    assert!(compile_case(&lamp)
        .generated
        .unwrap()
        .manifest
        .collections
        .is_none());
}

#[test]
fn a_bounded_memory_target_refuses_an_unbounded_design_and_warns_about_inputs() {
    let case = corpus().into_iter().find(|c| c.name == "buffer").unwrap();
    // a host: warned, still generated
    let art = compile_case(&case);
    assert!(art.succeeded());
    let d = art
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "deployment.unbounded_list_state")
        .unwrap();
    assert!(!d.is_error());
    assert_eq!(d.message, "log keeps every value it has ever received.");
    assert!(d.explanation.contains("take(N, …)"));
    // a target with finite memory: refused
    let opts = CompileOptions {
        memory: MemoryPolicy::Bounded,
        ..options()
    };
    let art = compile_design_ir(case.ir.clone(), case.name, &opts);
    assert!(!art.succeeded());
    assert!(
        art.exec_ir.is_some(),
        "lowering happened; generation did not"
    );
    let d = art
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "deployment.unbounded_list_state")
        .unwrap();
    assert!(d.is_error());
    assert!(d.explanation.contains("finite memory"));
    // an input-sized list is the platform's to bound: a warning there only
    let case = corpus().into_iter().find(|c| c.name == "list_ops").unwrap();
    let art = compile_design_ir(case.ir.clone(), case.name, &opts);
    assert!(art.succeeded());
    let d = art
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "deployment.list_input_unbounded")
        .unwrap();
    assert_eq!(
        d.message,
        "xs is a collection supplied from outside; its size is the platform's to bound."
    );
    assert!(compile_case(&case)
        .diagnostics
        .iter()
        .all(|d| d.code.as_str() != "deployment.list_input_unbounded"));
}

#[test]
fn the_window_capacity_comes_from_the_schedule_and_is_compared_with_the_bound() {
    let sufficient = corpus()
        .into_iter()
        .find(|c| c.name == "bounded_buffer")
        .unwrap();
    let opts = CompileOptions {
        schedule: Some(sufficient.schedule.clone()),
        ..options()
    };
    let art = compile_design_ir(sufficient.ir.clone(), sufficient.name, &opts);
    let r = art.collections.as_ref().unwrap();
    assert_eq!(r.windows.len(), 1);
    let w = &r.windows[0];
    assert_eq!(
        (w.source_name.as_str(), w.destination_name.as_str()),
        ("fast", "slow")
    );
    assert_eq!(w.required, Some(3));
    assert_eq!(w.carried.len(), 1);
    assert_eq!(w.carried[0].1, "logD");
    assert_eq!(w.carried[0].2, Bound::Finite { elements: 3 });
    assert!(art.diagnostics.is_empty(), "{:?}", art.diagnostics);

    let overflowing = corpus()
        .into_iter()
        .find(|c| c.name == "overflowing_buffer")
        .unwrap();
    let art = compile_design_ir(overflowing.ir.clone(), overflowing.name, &opts);
    let d = art
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "deployment.window_capacity")
        .unwrap();
    assert_eq!(
        d.message,
        "Between two activations of slow, fast produces up to 3 values, but logD keeps 2."
    );
    assert!(!d.is_error());
    // without a schedule the requirement is unknown and nothing is said
    let art = compile_case(&overflowing);
    assert_eq!(art.collections.as_ref().unwrap().windows[0].required, None);
    assert!(art.diagnostics.is_empty());
    // a slower source needs less
    let mut s = overflowing.schedule.clone();
    s.periods.insert(c(0), 2);
    let r = collections_report(art.exec_ir.as_ref().unwrap(), Some(&s));
    assert_eq!(r.windows[0].required, Some(2));
}
