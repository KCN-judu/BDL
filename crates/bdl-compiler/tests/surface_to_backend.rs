//! DI-17 closed: a formula may name another relationship, apply one, and
//! remember with `delay` / `sync`.  These cases go the whole way — surface
//! model → elaboration → typing → causality and clocks → reference
//! evaluator → generated Rust core — and the two executions must agree
//! trace for trace (`differential`).  The designs are the ones a designer
//! can author in Studio, nothing at Core level.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::{analyze, MappingStatus};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ClockId, DeclId, Dim, SemanticId};
use bdl_reactive::{InputTrace, Schedule, Value};
use support::*;

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
    fn concept(&mut self, name: &str, dim: Dim) -> SemanticId {
        self.edit(EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: Some(Representation::Quantity { dim }),
        })
        .created_concept
        .unwrap()
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

fn sem_scalar(concept: SemanticId, v: f64) -> Value {
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

#[test]
fn a_relationship_applied_to_an_input_lights_the_lamp() {
    let mut d = Surface::new("lamp");
    let tilt = d.concept("Tilt", Dim::ANGLE);
    let brightness = d.concept("Brightness", Dim::ZERO);
    let interaction = d.clock("interaction");
    let tilt_in = d.mapping("tilt", vec![], tilt, None, Some(interaction));
    d.mapping(
        "dimByTilt",
        vec![tilt],
        brightness,
        Some("Tilt / 90 deg"),
        None,
    );
    let bright = d.mapping(
        "brightness",
        vec![],
        brightness,
        Some("dimByTilt(tilt)"),
        Some(interaction),
    );

    // Core: brightness := mk Brightness (rep (app (declRef dimByTilt) (declRef tilt)))
    let a = analyze(&d.s);
    let core = bdl_check::pretty::expr(a.mappings[&bright].realization.as_ref().unwrap());
    assert!(core.contains("(rep (decl#1 decl#0))"), "{core}");
    assert_eq!(
        a.causality.order,
        vec![tilt_in, DeclId::from_raw(1), bright]
    );

    let mut case = case("surface_lamp", &d.s, 4);
    case.inputs.series(
        tilt_in,
        [0.0f64, 30.0, 60.0, 90.0]
            .iter()
            .map(|deg| Value::sem(tilt, Value::q(Dim::ANGLE, deg.to_radians()))),
    );
    let (art, trace) = differential(&case);
    let b = quantities(&value_of(&trace, &art, bright));
    assert_eq!(b[0], Some(0.0));
    assert!((b[1].unwrap() - 1.0 / 3.0).abs() < 1e-15);
    assert!((b[2].unwrap() - 2.0 / 3.0).abs() < 1e-15);
    assert_eq!(b[3], Some(1.0));
}

#[test]
fn delay_remembers_the_previous_activation_from_the_surface() {
    let mut d = Surface::new("delay");
    let level = d.concept("Level", Dim::ZERO);
    let main = d.clock("main");
    let x = d.mapping("x", vec![], level, None, Some(main));
    let acc = d.mapping("acc", vec![], level, Some("delay(0, acc + x)"), Some(main));
    let twice = d.mapping("twice", vec![], level, Some("acc * 2"), Some(main));

    let a = analyze(&d.s);
    assert!(
        a.mappings[&acc].diagnostics.is_empty(),
        "{:?}",
        a.mappings[&acc].diagnostics
    );
    assert!(a.dependencies.depends_on(acc, x));
    assert!(
        !a.dependencies.inst_depends_on(acc, acc) && !a.dependencies.inst_depends_on(acc, x),
        "delay is not instantaneous"
    );

    let mut case = case("surface_delay", &d.s, 5);
    case.inputs
        .series(x, (1..=5).map(|n| sem_scalar(level, n as f64)));
    let (art, trace) = differential(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, acc)),
        vec![Some(0.0), Some(1.0), Some(3.0), Some(6.0), Some(10.0)]
    );
    assert_eq!(
        quantities(&value_of(&trace, &art, twice)),
        vec![Some(0.0), Some(2.0), Some(6.0), Some(12.0), Some(20.0)]
    );
}

#[test]
fn sync_observes_only_prior_source_activations_from_the_surface() {
    let mut d = Surface::new("sync");
    let level = d.concept("Level", Dim::ZERO);
    let fast = d.clock("fast");
    let slow = d.clock("slow");
    let x = d.mapping("x", vec![], level, None, Some(fast));
    let y = d.mapping("y", vec![], level, Some("sync(fast, -1, x)"), Some(slow));
    let z = d.mapping("z", vec![], level, Some("sync(slow, 0, y)"), Some(fast));

    let a = analyze(&d.s);
    assert!(a.clocks.valid, "{:?}", a.clocks.diagnostics);

    let mut case = case("surface_sync", &d.s, 7);
    case.schedule.periods.insert(slow, 2);
    case.inputs
        .series(x, (0..7).map(|n| sem_scalar(level, n as f64 * 10.0)));
    let (art, trace) = differential(&case);
    assert_eq!(
        quantities(&value_of(&trace, &art, y)),
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
    // z in fast sees y from the previous slow activation, never the same tick's
    assert_eq!(
        quantities(&value_of(&trace, &art, z)),
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
}

#[test]
fn reading_across_domains_without_sync_is_refused_and_sync_offers_the_way() {
    let mut d = Surface::new("cross");
    let level = d.concept("Level", Dim::ZERO);
    let fast = d.clock("fast");
    let slow = d.clock("slow");
    d.mapping("x", vec![], level, None, Some(fast));
    let y = d.mapping("y", vec![], level, Some("x + 1"), Some(slow));
    let a = analyze(&d.s);
    assert_eq!(a.mappings[&y].status, MappingStatus::TemporallyValid);
    assert!(a.mappings[&y]
        .diagnostics
        .iter()
        .any(|d| d.code.as_str() == "clock.cross_domain_reference"));
}

/// Attach `src` to a fresh probe mapping, collect its codes, remove it.
fn probe(d: &mut Surface, output: SemanticId, clock: ClockId, src: &str) -> Vec<String> {
    let m = d.mapping("probe", vec![], output, Some(src), Some(clock));
    let a = analyze(&d.s);
    let codes = a.mappings[&m]
        .diagnostics
        .iter()
        .map(|x| x.code.as_str().to_owned())
        .collect();
    d.edit(EditOp::DeleteMapping { id: m });
    codes
}

#[test]
fn the_surface_refuses_what_the_kernel_cannot_mean() {
    let mut d = Surface::new("bad");
    let tilt = d.concept("Tilt", Dim::ANGLE);
    let brightness = d.concept("Brightness", Dim::ZERO);
    let speed = d.concept("Speed", Dim::ZERO);
    let main = d.clock("main");
    d.mapping("tilt", vec![], tilt, None, Some(main));
    d.mapping(
        "dimByTilt",
        vec![tilt],
        brightness,
        Some("Tilt / 90 deg"),
        None,
    );
    d.mapping("speed", vec![], speed, None, Some(main));
    let has = |codes: &[String], code: &str| codes.iter().any(|c| c == code);

    // a relationship with inputs named without arguments
    let c = probe(&mut d, brightness, main, "dimByTilt");
    assert!(has(&c, "formula.mapping.needs_arguments"), "{c:?}");
    // wrong arity
    let c = probe(&mut d, brightness, main, "dimByTilt(tilt, tilt)");
    assert!(has(&c, "formula.call.arity"), "{c:?}");
    // a value of the wrong concept
    let c = probe(&mut d, brightness, main, "dimByTilt(speed)");
    assert!(has(&c, "formula.call.argument_type"), "{c:?}");
    // a computed number as an argument: no grant makes it a Tilt
    let c = probe(&mut d, brightness, main, "dimByTilt(tilt + 1 deg)");
    assert!(has(&c, "formula.call.argument"), "{c:?}");
    // a concept applied
    let c = probe(&mut d, brightness, main, "Tilt(tilt)");
    assert!(has(&c, "formula.call.not_a_relationship"), "{c:?}");
    // sync on an unknown domain
    let c = probe(&mut d, brightness, main, "sync(ambient, 0, speed)");
    assert!(has(&c, "formula.sync.unknown_domain"), "{c:?}");
    // delay with mismatched kinds
    let c = probe(&mut d, brightness, main, "delay(true, speed)");
    assert!(has(&c, "type.temporal_mismatch"), "{c:?}");
    // memory under inputs is refused before the checker sees it
    let f = d.mapping(
        "f",
        vec![tilt],
        brightness,
        Some("delay(0, Tilt / 90 deg)"),
        None,
    );
    let a = analyze(&d.s);
    assert!(a.mappings[&f]
        .diagnostics
        .iter()
        .any(|x| x.code.as_str() == "formula.temporal.under_inputs"));
}
