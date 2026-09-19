//! The textual expression forms — `let`, `if`, calls, `Some`/`None`,
//! `match`, memory placed in a `let` or a scrutinee or a branch — from a
//! surface design all the way to generated Rust, trace for trace against
//! the reference evaluator (`differential`).  Nothing here is Core-level:
//! every formula is text a designer can type in Studio or in a `.bdl` file.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::{analyze, CompileArtifact, MappingStatus};
use bdl_exec_ir::interp::{self, CellState};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ClockId, DeclId, Dim, SemanticId};
use bdl_reactive::eval::{self, State, TickInput};
use bdl_reactive::{InputTrace, Schedule, Value};
use bdl_runtime_host::RunTrace;
use support::*;

/// The three engines on one case: reference vs generated Rust (the
/// harness's `differential`), and reference vs the exec-IR interpreter.
fn three_way(case: &Case) -> (CompileArtifact, RunTrace) {
    let (art, trace) = differential(case);
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
        let in_slots: Vec<Option<Value>> = exec
            .inputs
            .iter()
            .map(|i| input.values.get(&exec.decl(i.decl).unwrap().id).cloned())
            .collect();
        let r = eval::step(&case.ir, t, &active, &rs, &input).unwrap();
        let e = interp::step(exec, t, &slots, &es, &in_slots).unwrap();
        for dp in &exec.decls {
            if let bdl_exec_ir::DeclKind::Computed { .. } = dp.kind {
                assert_eq!(
                    r.values.get(&dp.id),
                    e.values[dp.index.0 as usize].as_ref(),
                    "{}: tick {t} decl {}",
                    case.name,
                    dp.id
                );
            }
        }
        rs = r.next;
        es = e.next.clone();
    }
    (art, trace)
}

struct Surface {
    s: ProjectSnapshot,
}

impl Surface {
    fn new(name: &str) -> Surface {
        Surface {
            s: ProjectSnapshot::new(Design::empty(name)),
        }
    }
    fn edit(&mut self, op: EditOp) -> bdl_model::edit::EditOutcome {
        let a = apply_edit(&self.s, &op).expect("surface edit applies");
        self.s = a.snapshot;
        a.outcome
    }
    fn concept(&mut self, name: &str, rep: Representation) -> SemanticId {
        self.edit(EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: Some(rep),
        })
        .created_concept
        .unwrap()
    }
    fn level(&mut self, name: &str) -> SemanticId {
        self.concept(name, Representation::Quantity { dim: Dim::ZERO })
    }
    fn clock(&mut self, name: &str) -> ClockId {
        self.edit(EditOp::CreateClockDomain { name: name.into() })
            .created_clock
            .unwrap()
    }
    fn mapping(
        &mut self,
        name: &str,
        inputs: Vec<SemanticId>,
        output: SemanticId,
        formula: Option<&str>,
        clock: Option<ClockId>,
    ) -> DeclId {
        let id = self
            .edit(EditOp::CreateMapping {
                name: name.into(),
                description: String::new(),
                signature: Signature { inputs, output },
                definition: None,
                clock: None,
            })
            .created_mapping
            .unwrap();
        if let Some(f) = formula {
            self.edit(EditOp::AttachDefinition {
                id,
                definition: Definition::Formula { source: f.into() },
            });
        }
        if let Some(c) = clock {
            self.edit(EditOp::SetMappingClock { id, clock: Some(c) });
        }
        id
    }
}

fn scalar(concept: SemanticId, v: f64) -> Value {
    Value::sem(concept, Value::q(Dim::ZERO, v))
}

/// Analyse, insist on a clean ladder, and package for the differential run.
fn case(name: &'static str, s: &ProjectSnapshot, ticks: u64) -> Case {
    let a = analyze(s);
    for m in a.mappings.values() {
        assert!(
            m.status == MappingStatus::ClockConsistent || m.status == MappingStatus::Declared,
            "{name}: {} is {:?}: {:?}",
            m.id,
            m.status,
            m.diagnostics
        );
    }
    assert!(
        a.causality.valid && a.clocks.valid,
        "{name}: {:?}",
        a.diagnostics
    );
    Case {
        name,
        schedule: Schedule::always(&a.ir),
        ir: a.ir,
        ticks,
        inputs: InputTrace::default(),
    }
}

/// One-domain design with an input `x : Level` and `y : Level := formula`.
fn xy(name: &str, formula: &str) -> (Surface, SemanticId, DeclId, DeclId) {
    let mut d = Surface::new(name);
    let level = d.level("Level");
    let main = d.clock("main");
    let x = d.mapping("x", vec![], level, None, Some(main));
    let y = d.mapping("y", vec![], level, Some(formula), Some(main));
    (d, level, x, y)
}

#[test]
fn let_and_nested_let() {
    let (d, level, x, y) = xy(
        "expr_let",
        "{ let a = x + 1; let b = { let c = a * 2; c + 1 }; a + b }",
    );
    let mut case = case("expr_let", &d.s, 3);
    case.inputs
        .series(x, [0.0, 1.0, 4.0].map(|v| scalar(level, v)));
    let (art, trace) = three_way(&case);
    // a = x+1, c = 2a, b = c+1 = 2a+1, y = a + b = 3a + 1
    assert_eq!(
        quantities(&value_of(&trace, &art, y)),
        vec![Some(4.0), Some(7.0), Some(16.0)]
    );
}

#[test]
fn if_on_a_condition_and_on_literals() {
    let (mut d, level, x, y) = xy("expr_if", "if x < 3 then x * 10 else x");
    let main = d.s.design.clocks.keys().next().copied().unwrap();
    let t = d.mapping(
        "t",
        vec![],
        level,
        Some("if true then x else 0"),
        Some(main),
    );
    let f = d.mapping(
        "f",
        vec![],
        level,
        Some("if false then x else 0"),
        Some(main),
    );
    let mut case = case("expr_if", &d.s, 3);
    case.inputs
        .series(x, [1.0, 3.0, 5.0].map(|v| scalar(level, v)));
    let (art, trace) = three_way(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, y)),
        vec![Some(10.0), Some(3.0), Some(5.0)]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, t)),
        vec![Some(1.0), Some(3.0), Some(5.0)]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, f)),
        vec![Some(0.0), Some(0.0), Some(0.0)]
    );
}

#[test]
fn calls_and_nested_calls_are_inlined_relationships() {
    let mut d = Surface::new("expr_calls");
    let tilt = d.concept("Tilt", Representation::Quantity { dim: Dim::ANGLE });
    let brightness = d.level("Brightness");
    let main = d.clock("main");
    let tilt_in = d.mapping("tilt", vec![], tilt, None, Some(main));
    d.mapping(
        "dimByTilt",
        vec![tilt],
        brightness,
        Some("Tilt / 90 deg"),
        None,
    );
    d.mapping(
        "twice",
        vec![brightness],
        brightness,
        Some("Brightness * 2"),
        None,
    );
    let b = d.mapping("b", vec![], brightness, Some("dimByTilt(tilt)"), Some(main));
    let b2 = d.mapping(
        "b2",
        vec![],
        brightness,
        Some("twice(dimByTilt(tilt)) + b"),
        Some(main),
    );
    let mut case = case("expr_calls", &d.s, 3);
    case.inputs.series(
        tilt_in,
        [0.0f64, 45.0, 90.0]
            .iter()
            .map(|deg| Value::sem(tilt, Value::q(Dim::ANGLE, deg.to_radians()))),
    );
    let (art, trace) = three_way(&case);
    let bv = quantities(&value_of(&trace, &art, b));
    assert_eq!(bv[0], Some(0.0));
    assert!((bv[1].unwrap() - 0.5).abs() < 1e-15);
    assert_eq!(bv[2], Some(1.0));
    let b2v = quantities(&value_of(&trace, &art, b2));
    assert_eq!(b2v[0], Some(0.0));
    assert!((b2v[1].unwrap() - 1.5).abs() < 1e-15);
    assert_eq!(b2v[2], Some(3.0));
    // both relationships were inlined; no closure exists at runtime
    assert_eq!(art.exec_ir.as_ref().unwrap().functions.len(), 2);
}

#[test]
fn option_match_and_nested_match() {
    let (mut d, level, x, y) = xy(
        "expr_match",
        "match (if x < 3 then Some(x * 10) else None) { Some(v) => v, None => -1 }",
    );
    let main = d.s.design.clocks.keys().next().copied().unwrap();
    let z = d.mapping(
        "z",
        vec![],
        level,
        Some("match Some(if x < 2 then Some(x) else None) { Some(Some(v)) => v + 100, Some(None) => 100, None => 200 }"),
        Some(main),
    );
    let b = d.mapping(
        "b",
        vec![],
        level,
        Some("match x < 2 { true => 1, false => 0 }"),
        Some(main),
    );
    let mut case = case("expr_match", &d.s, 3);
    case.inputs
        .series(x, [1.0, 2.0, 3.0].map(|v| scalar(level, v)));
    let (art, trace) = three_way(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, y)),
        vec![Some(10.0), Some(20.0), Some(-1.0)]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, z)),
        vec![Some(101.0), Some(100.0), Some(100.0)]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, b)),
        vec![Some(1.0), Some(0.0), Some(0.0)]
    );
}

#[test]
fn match_on_a_delayed_option_keeps_one_state_cell() {
    let (d, level, x, acc) = xy(
        "expr_match_delay",
        "match delay(None, Some(y + x)) { Some(v) => v, None => 0 }",
    );
    let mut case = case("expr_match_delay", &d.s, 5);
    case.inputs
        .series(x, (1..=5).map(|n| scalar(level, n as f64)));
    let (art, trace) = three_way(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, acc)),
        vec![Some(0.0), Some(1.0), Some(3.0), Some(6.0), Some(10.0)]
    );
    // the scrutinee is bound once: one cell, at the scrutinee's path
    let exec = art.exec_ir.as_ref().unwrap();
    assert_eq!(exec.cells.len(), 1);
    assert_eq!(exec.cells[0].cell.decl, acc);
}

#[test]
fn memory_in_a_let_value_and_in_an_if_branch() {
    let (d, level, x, acc) = xy(
        "expr_let_delay",
        "{ let previous = delay(0, y); previous + x }",
    );
    let mut case = case("expr_let_delay", &d.s, 4);
    case.inputs
        .series(x, (1..=4).map(|n| scalar(level, n as f64)));
    let (art, trace) = three_way(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, acc)),
        vec![Some(1.0), Some(3.0), Some(6.0), Some(10.0)]
    );
    assert_eq!(art.exec_ir.as_ref().unwrap().cells.len(), 1);

    // a branch holding a sync: two domains, the slow one gated
    let mut d = Surface::new("expr_branch_sync");
    let level = d.level("Level");
    let gate_c = d.concept("Gate", Representation::Boolean);
    let fast = d.clock("fast");
    let slow = d.clock("slow");
    let x = d.mapping("x", vec![], level, None, Some(fast));
    let gate = d.mapping("gate", vec![], gate_c, None, Some(slow));
    let z = d.mapping(
        "z",
        vec![],
        level,
        Some("if gate then sync(fast, -1, x) else 0"),
        Some(slow),
    );
    let mut case2 = self::case("expr_branch_sync", &d.s, 6);
    case2.schedule.periods.insert(slow, 2);
    case2
        .inputs
        .series(x, (0..6).map(|n| scalar(level, n as f64 * 10.0)));
    // the gate is read on slow ticks only: 0, 2, 4
    for (t, g) in [(0u64, true), (2, true), (4, false)] {
        case2
            .inputs
            .samples
            .entry(gate)
            .or_default()
            .insert(t, Value::sem(gate_c, Value::boolean(g)));
    }
    let (art, trace) = three_way(&case2);
    // tick 0: nothing before → -1; tick 2: last fast activation strictly
    // before is tick 1 → 10; tick 4: gate false → 0
    assert_eq!(
        quantities(&value_of(&trace, &art, z)),
        vec![Some(-1.0), None, Some(10.0), None, Some(0.0), None]
    );
}

#[test]
fn branching_syntax_does_not_hide_instantaneous_dependencies() {
    let mut d = Surface::new("deps");
    let level = d.level("Level");
    let main = d.clock("main");
    let x = d.mapping("x", vec![], level, None, Some(main));
    let y = d.mapping("y", vec![], level, Some("{ let a = x; a }"), Some(main));
    let z = d.mapping(
        "z",
        vec![],
        level,
        Some("match Some(1) { Some(_) => y, None => 0 }"),
        Some(main),
    );
    let w = d.mapping(
        "w",
        vec![],
        level,
        Some("if true then 0 else x"),
        Some(main),
    );
    let a = analyze(&d.s);
    assert!(a.dependencies.inst_depends_on(y, x));
    assert!(a.dependencies.inst_depends_on(z, y));
    assert!(a.dependencies.inst_depends_on(w, x));
    assert!(a.causality.valid);

    // a self-reference inside a match arm is an instantaneous cycle …
    let p = d.mapping(
        "p",
        vec![],
        level,
        Some("match Some(p) { Some(v) => v, None => 0 }"),
        Some(main),
    );
    let a = analyze(&d.s);
    assert_eq!(a.mappings[&p].status, MappingStatus::Invalid);
    assert!(a.mappings[&p]
        .diagnostics
        .iter()
        .any(|d| d.code.as_str() == "reactive.instantaneous_cycle"));
    d.edit(EditOp::DeleteMapping { id: p });
    // … and the same through a delayed scrutinee is not
    let q = d.mapping(
        "q",
        vec![],
        level,
        Some("match delay(None, Some(q)) { Some(v) => v, None => 0 }"),
        Some(main),
    );
    let a = analyze(&d.s);
    assert_eq!(a.mappings[&q].status, MappingStatus::ClockConsistent);
    assert!(!a.dependencies.inst_depends_on(q, q) && a.dependencies.depends_on(q, q));
}

#[test]
fn the_higher_order_boundary_is_explicit_at_the_surface() {
    let mut d = Surface::new("ho");
    let tilt = d.concept("Tilt", Representation::Quantity { dim: Dim::ANGLE });
    let brightness = d.level("Brightness");
    let main = d.clock("main");
    d.mapping("tilt", vec![], tilt, None, Some(main));
    d.mapping(
        "dimByTilt",
        vec![tilt],
        brightness,
        Some("Tilt / 90 deg"),
        None,
    );
    // a relationship passed to a relationship, or used as a value
    let bad = d.mapping(
        "bad",
        vec![],
        brightness,
        Some("dimByTilt(dimByTilt)"),
        Some(main),
    );
    let a = analyze(&d.s);
    let codes: Vec<&str> = a.mappings[&bad]
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert!(
        codes.contains(&"formula.mapping.needs_arguments"),
        "{codes:?}"
    );
    assert!(
        !codes.iter().any(|c| c.starts_with("backend.")),
        "refused by the surface, never reaching the backend: {codes:?}"
    );
}
