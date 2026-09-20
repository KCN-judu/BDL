//! Output realization end to end (docs/architecture/output-realization.md,
//! FV Phase 14): a logical output keeps its meaning; a device binding's
//! realization profile is deployment data; the profile's pure encoder
//! becomes a machine sink below the behavior plan; the hardware judgment
//! stays with the solver.  Every test here runs the surface design a
//! designer authors in Studio through `compile` / `analyze_deployment`.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::{analyze_deployment, compile, deployment_report, CompileOptions};
use bdl_exec_ir::interp::{self, CellState};
use bdl_exec_ir::ExecIr;
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{
    Definition, Design, DeviceKind, ProjectSnapshot, Representation, Signature,
};
use bdl_model::{ClockId, DeclId, DeviceId, Dim, OutputId, OutputProfileId, SemanticId};
use bdl_reactive::Value;
use bdl_runtime_host::{DynValue, RunRequest, TickRequest};
use std::collections::BTreeMap;

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
    fn concept(&mut self, name: &str, representation: Representation) -> SemanticId {
        self.edit(EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: Some(representation),
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
    fn output(&mut self, name: &str, accepts: SemanticId, clock: ClockId) -> OutputId {
        self.edit(EditOp::CreateOutput {
            name: name.into(),
            description: String::new(),
            accepts,
            clock: Some(clock),
        })
        .created_output
        .unwrap()
    }
    fn drive(&mut self, id: DeclId, output: OutputId) {
        self.edit(EditOp::SetMappingDrive {
            id,
            output: Some(output),
        });
    }
    fn device(&mut self, name: &str, kind: DeviceKind, output: OutputId) -> DeviceId {
        self.edit(EditOp::CreateDevice {
            name: name.into(),
            kind,
            output: Some(output),
        })
        .created_device
        .unwrap()
    }
    fn realize(&mut self, id: DeviceId, profile: &str, kind: DeviceKind) {
        self.edit(EditOp::SetDeviceRealization {
            id,
            profile: Some(OutputProfileId(profile.into())),
            kind,
        });
    }
}

/// The lamp: `level : () -> Brightness` (an input) in `main` drives
/// `light : Brightness`; one PWM device `lamp` realises `light`.
struct Lamp {
    d: Surface,
    brightness: SemanticId,
    level: DeclId,
    light: OutputId,
    lamp: DeviceId,
}

fn lamp() -> Lamp {
    let mut d = Surface::new("lamp_realized");
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let main = d.clock("main");
    let level = d.mapping("level", vec![], brightness, None, Some(main));
    let light = d.output("light", brightness, main);
    d.drive(level, light);
    let lamp = d.device("lamp", DeviceKind::PwmChannel, light);
    Lamp {
        d,
        brightness,
        level,
        light,
        lamp,
    }
}

fn compiled(s: &ProjectSnapshot) -> ExecIr {
    let art = compile(s, &support::options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    art.exec_ir.unwrap()
}

/// Run the exec IR on `levels` fed to the single input, every domain
/// active every tick.
fn run(exec: &ExecIr, brightness: SemanticId, levels: &[f64]) -> Vec<interp::TickResult> {
    let active: Vec<_> = exec.clocks.iter().map(|c| c.slot).collect();
    let mut state = CellState::init(exec);
    let mut out = Vec::new();
    for (t, level) in levels.iter().enumerate() {
        let inputs = vec![Some(Value::sem(brightness, Value::q(Dim::ZERO, *level)))];
        let r = interp::step(exec, t as u64, &active, &state, &inputs).unwrap();
        state = r.next.clone();
        out.push(r);
    }
    out
}

fn commands(ticks: &[interp::TickResult]) -> Vec<Option<Value>> {
    ticks.iter().map(|t| t.commands[0].clone()).collect()
}

fn q(v: f64) -> Option<Value> {
    Some(Value::q(Dim::ZERO, v))
}

const LEVELS: [f64; 5] = [0.0, 50.0, 100.0, 12.5, 74.9];

// ---- A, B, C: behavior is the same under every realization ----------------

#[test]
fn changing_the_realization_changes_no_behavior_and_only_the_raw_trace() {
    let mut l = lamp();
    let none = compiled(&l.d.s);
    l.d.realize(l.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    let a = compiled(&l.d.s);
    l.d.realize(l.lamp, "pwm_duty4", DeviceKind::PwmChannel);
    let b = compiled(&l.d.s);

    // The behavior plan is identical: same declarations, cells, outputs,
    // clocks — the sinks are the only difference (`lower_transparent`).
    for (x, y) in [(&none, &a), (&a, &b)] {
        assert_eq!(x.decls, y.decls);
        assert_eq!(x.cells, y.cells);
        assert_eq!(x.outputs, y.outputs);
        assert_eq!(x.clocks, y.clocks);
        assert_eq!(x.inputs, y.inputs);
    }
    assert!(none.sinks.is_empty());
    assert_eq!(a.sinks.len(), 1);
    assert_eq!(b.sinks.len(), 1);
    assert_eq!(a.sinks[0].profile.as_str(), "pwm_duty8");
    assert_eq!(b.sinks[0].profile.as_str(), "pwm_duty4");
    assert_eq!(a.sinks[0].output, l.light);
    assert_eq!(a.sinks[0].device, l.lamp);

    // The behavior trace is identical tick for tick; the raw traces differ.
    let tn = run(&none, l.brightness, &LEVELS);
    let ta = run(&a, l.brightness, &LEVELS);
    let tb = run(&b, l.brightness, &LEVELS);
    for t in 0..LEVELS.len() {
        assert_eq!(tn[t].values, ta[t].values);
        assert_eq!(ta[t].values, tb[t].values);
        assert_eq!(tn[t].outputs, ta[t].outputs);
        assert_eq!(ta[t].outputs, tb[t].outputs);
        assert_eq!(
            ta[t].outputs[0],
            Some(Value::sem(l.brightness, Value::q(Dim::ZERO, LEVELS[t])))
        );
    }
    assert!(tn.iter().all(|t| t.commands.is_empty()));
    assert_ne!(commands(&ta), commands(&tb));
}

// ---- D, E: exact raw encodings ------------------------------------------

#[test]
fn pwm_duty8_encodes_min_max_and_intermediate_levels_exactly() {
    let mut l = lamp();
    l.d.realize(l.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    let exec = compiled(&l.d.s);
    assert_eq!(exec.sinks[0].raw, bdl_ir::Ty::q(Dim::ZERO));
    let t = run(&exec, l.brightness, &LEVELS);
    assert_eq!(
        commands(&t),
        vec![
            q(0.0),
            q(127.5),
            q(255.0),
            q(31.875),
            q(74.9 * 255.0 / 100.0)
        ]
    );
}

#[test]
fn pwm_duty4_quantizes_many_levels_to_one_duty() {
    let mut l = lamp();
    l.d.realize(l.lamp, "pwm_duty4", DeviceKind::PwmChannel);
    let exec = compiled(&l.d.s);
    let t = run(
        &exec,
        l.brightness,
        &[0.0, 24.9, 25.0, 49.0, 50.0, 74.9, 75.0, 100.0],
    );
    assert_eq!(
        commands(&t),
        vec![
            q(0.0),
            q(0.0),
            q(85.0),
            q(85.0),
            q(170.0),
            q(170.0),
            q(255.0),
            q(255.0)
        ]
    );
}

// ---- F, G, H: the other witnesses ------------------------------------------

#[test]
fn i2c_profile_realises_the_same_output_as_a_register_value_pair() {
    let mut l = lamp();
    l.d.realize(l.lamp, "i2c_level8", DeviceKind::I2cSensor);
    assert_eq!(l.d.s.design.devices[&l.lamp].kind, DeviceKind::I2cSensor);
    let exec = compiled(&l.d.s);
    let t = run(&exec, l.brightness, &[0.0, 100.0]);
    let pair = |v: f64| {
        Some(Value::pair(
            Value::q(Dim::ZERO, 42.0),
            Value::q(Dim::ZERO, v),
        ))
    };
    assert_eq!(commands(&t), vec![pair(0.0), pair(255.0)]);
    // The behavior trace is the PWM lamp's.
    let mut p = lamp();
    p.d.realize(p.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    let tp = run(&compiled(&p.d.s), p.brightness, &[0.0, 100.0]);
    assert_eq!(
        tp.iter().map(|x| &x.outputs).collect::<Vec<_>>(),
        t.iter().map(|x| &x.outputs).collect::<Vec<_>>()
    );
}

#[test]
fn gpio_profile_passes_a_switch_state_through_as_a_truth_value() {
    let mut d = Surface::new("relay");
    let state = d.concept("SwitchState", Representation::Boolean);
    let main = d.clock("main");
    let on = d.mapping("on", vec![], state, None, Some(main));
    let relay = d.output("relay", state, main);
    d.drive(on, relay);
    let dev = d.device("coil", DeviceKind::DigitalOutput, relay);
    d.realize(dev, "gpio_level", DeviceKind::DigitalOutput);
    let exec = compiled(&d.s);
    assert_eq!(exec.sinks[0].raw, bdl_ir::Ty::Bool);
    let active: Vec<_> = exec.clocks.iter().map(|c| c.slot).collect();
    let cs = CellState::init(&exec);
    for b in [true, false] {
        let r = interp::step(
            &exec,
            0,
            &active,
            &cs,
            &[Some(Value::sem(state, Value::Bool { value: b }))],
        )
        .unwrap();
        assert_eq!(r.commands, vec![Some(Value::Bool { value: b })]);
    }
}

#[test]
fn hbridge_profile_splits_a_signed_level_into_direction_and_duty() {
    let mut l = lamp();
    l.d.realize(l.lamp, "hbridge_signed", DeviceKind::HBridgeChannel);
    let exec = compiled(&l.d.s);
    let t = run(&exec, l.brightness, &[100.0, -100.0, 0.0, -50.0]);
    let pair = |fwd: bool, duty: f64| {
        Some(Value::pair(
            Value::Bool { value: fwd },
            Value::q(Dim::ZERO, duty),
        ))
    };
    assert_eq!(
        commands(&t),
        vec![
            pair(true, 255.0),
            pair(false, 255.0),
            pair(true, 0.0),
            pair(false, 127.5)
        ]
    );
}

// ---- N, clocks: the logical layer is untouched ----------------------------

#[test]
fn single_driver_and_drive_edges_are_unchanged_by_realization() {
    let mut l = lamp();
    let before = bdl_compiler::analyze(&l.d.s);
    l.d.realize(l.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    let after = bdl_compiler::analyze(&l.d.s);
    assert_eq!(before.outputs, after.outputs);
    assert_eq!(before.ir, after.ir);
    assert_eq!(before.diagnostics, after.diagnostics);
    assert_eq!(
        after.outputs.valid_bindings,
        [(l.level, l.light)].into_iter().collect()
    );
    // The sink is due exactly when the driver is: no domain of its own.
    let exec = compiled(&l.d.s);
    assert_eq!(exec.clocks.len(), 1);
    let cs = CellState::init(&exec);
    let idle = interp::step(&exec, 0, &[], &cs, &[None]).unwrap();
    assert_eq!(idle.outputs, vec![None]);
    assert_eq!(idle.commands, vec![None]);
}

// ---- O, P, Q, R: the judgments, named -------------------------------------

fn nano() -> bdl_hardware::Hardware {
    bdl_hardware::boards::arduino_nano()
}

fn codes(d: &bdl_compiler::DeploymentAnalysis) -> Vec<&str> {
    d.diagnostics.iter().map(|x| x.code.as_str()).collect()
}

#[test]
fn a_binding_without_a_profile_places_by_kind_and_is_told_so() {
    let l = lamp();
    let d = analyze_deployment(&l.d.s, &nano());
    assert_eq!(d.status, bdl_compiler::DeploymentStatus::Feasible);
    assert_eq!(codes(&d), ["deploy.realization_unspecified"]);
    assert!(d.diagnostics.iter().all(|x| !x.is_error()));
    let r = &d.realizations[&l.lamp];
    assert_eq!(
        r.check.status,
        bdl_output::realization::RealizationStatus::NotChosen
    );
    assert!(r.hardware_placed && !r.admissible());
    assert!(!d.realization_blocked());
    let report = deployment_report(&l.d.s, &bdl_compiler::analyze(&l.d.s), &d, &nano());
    assert!(report.deployable);
    // ... and the artefact is the one there always was.
    assert!(compiled(&l.d.s).sinks.is_empty());
}

#[test]
fn an_unknown_profile_is_named_and_refuses_the_artefact() {
    let mut l = lamp();
    l.d.realize(l.lamp, "pwm_duty16", DeviceKind::PwmChannel);
    let d = analyze_deployment(&l.d.s, &nano());
    assert_eq!(codes(&d), ["deploy.realization_unknown_profile"]);
    assert!(d.realization_blocked());
    assert!(d.diagnostics[0].message.contains("lamp"));
    let report = deployment_report(&l.d.s, &bdl_compiler::analyze(&l.d.s), &d, &nano());
    assert!(!report.deployable);
    assert_eq!(
        report.missing[0].kind,
        bdl_compiler::MissingKind::RealizationInvalid
    );
    let art = compile(&l.d.s, &support::options());
    assert!(!art.succeeded());
    assert_eq!(
        art.diagnostics[0].code.as_str(),
        "backend.realization_invalid"
    );
}

#[test]
fn an_incompatible_profile_names_the_representations() {
    let mut l = lamp();
    l.d.realize(l.lamp, "gpio_level", DeviceKind::DigitalOutput);
    let d = analyze_deployment(&l.d.s, &nano());
    assert_eq!(codes(&d), ["deploy.realization_incompatible"]);
    let r = &d.realizations[&l.lamp];
    assert!(r.check.encoder_well_formed() && !r.check.representation_fits());
    assert!(d.diagnostics[0].message.contains("q[1]"));
    assert!(d.diagnostics[0].message.contains("bool"));
    assert!(!compile(&l.d.s, &support::options()).succeeded());
    // The candidate list says which profiles would fit.
    let fits: Vec<(&str, Option<bool>)> = r
        .candidates
        .iter()
        .map(|(p, f)| (p.id.as_str(), *f))
        .collect();
    assert!(fits.contains(&("pwm_duty8", Some(true))));
    assert!(fits.contains(&("gpio_level", Some(false))));
}

#[test]
fn a_kind_that_disagrees_with_the_profile_is_reported() {
    let mut l = lamp();
    l.d.realize(l.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    l.d.edit(EditOp::SetDeviceKind {
        id: l.lamp,
        kind: DeviceKind::DigitalOutput,
    });
    // SetDeviceKind releases the profile: kind and profile never disagree
    // through the edit layer ...
    assert_eq!(l.d.s.design.devices[&l.lamp].realization, None);
    // ... but a persisted file can say anything.
    let mut s = l.d.s.clone();
    s.design.devices.get_mut(&l.lamp).unwrap().realization =
        Some(OutputProfileId("pwm_duty8".into()));
    let d = analyze_deployment(&s, &nano());
    assert_eq!(codes(&d), ["deploy.realization_kind_mismatch"]);
    assert!(d.realization_blocked());
}

#[test]
fn hardware_feasibility_is_judged_apart_from_the_encoding() {
    let mut l = lamp();
    l.d.realize(l.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    // A board without PWM: the encoding is valid, the hardware is not.
    let gpio_only = bdl_hardware::Hardware {
        name: "gpio_only".into(),
        resources: vec![bdl_hardware::Resource {
            id: bdl_hardware::ResourceId("D1".into()),
            capabilities: [bdl_hardware::Capability::DigitalOut].into_iter().collect(),
            units: BTreeMap::new(),
        }],
        shareable: std::collections::BTreeSet::new(),
        display_name: "GPIO only".into(),
        description: String::new(),
        family: String::new(),
    };
    let d = analyze_deployment(&l.d.s, &gpio_only);
    assert_eq!(d.status, bdl_compiler::DeploymentStatus::Infeasible);
    let r = &d.realizations[&l.lamp];
    assert!(r.check.is_valid() && !r.hardware_placed && !r.admissible());
    // And on the Nano all three hold.
    let d = analyze_deployment(&l.d.s, &nano());
    assert!(d.realizations[&l.lamp].admissible());
    assert!(codes(&d).is_empty());
}

// ---- V: determinism and the generated core ---------------------------------

#[test]
fn realized_artefacts_are_deterministic() {
    let mut l = lamp();
    l.d.realize(l.lamp, "i2c_level8", DeviceKind::I2cSensor);
    let a = compile(&l.d.s, &CompileOptions::default());
    let b = compile(&l.d.s, &CompileOptions::default());
    assert_eq!(a.exec_ir, b.exec_ir);
    assert_eq!(a.generated, b.generated);
    let m = &a.generated.unwrap().manifest;
    assert_eq!(m.sinks.len(), 1);
    assert_eq!(m.sinks[0].profile, "i2c_level8");
    assert_eq!(m.sinks[0].symbol, format!("command_{}", l.lamp.raw()));
}

/// The generated core's `Commands` agree with the exec IR interpreter's
/// commands tick for tick, and its `Values`/`Outputs` are the unrealised
/// design's.
#[test]
fn generated_raw_commands_agree_with_the_interpreter() {
    let mut l = lamp();
    l.d.realize(l.lamp, "pwm_duty8", DeviceKind::PwmChannel);
    let art = compile(&l.d.s, &support::options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let exec = art.exec_ir.as_ref().unwrap();
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("lamp_realized");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let req = RunRequest {
        ticks: LEVELS
            .iter()
            .map(|v| TickRequest {
                active: vec![0],
                inputs: vec![Some(DynValue::sem(
                    l.brightness.raw(),
                    DynValue::Quantity { value: *v },
                ))],
            })
            .collect(),
    };
    let trace = support::cargo_for("lamp_realized")
        .run_host(&req)
        .unwrap_or_else(|e| panic!("{e}"));
    let interp = run(exec, l.brightness, &LEVELS);
    assert_eq!(trace.error, None);
    assert_eq!(trace.ticks.len(), LEVELS.len());
    for (t, g) in trace.ticks.iter().enumerate() {
        let want: Vec<Option<DynValue>> = interp[t]
            .commands
            .iter()
            .map(|c| c.as_ref().map(support::dyn_of))
            .collect();
        assert_eq!(g.commands, want, "tick {t}");
        let outs: Vec<Option<DynValue>> = interp[t]
            .outputs
            .iter()
            .map(|c| c.as_ref().map(support::dyn_of))
            .collect();
        assert_eq!(g.outputs, outs, "tick {t}");
    }
    assert_eq!(
        trace.ticks[1].commands,
        vec![Some(DynValue::Quantity { value: 127.5 })]
    );
}
