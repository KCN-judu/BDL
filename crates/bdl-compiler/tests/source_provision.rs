//! The Source side of a deployment, end to end (Phase 16,
//! docs/architecture/embedded-adapter.md § The input half): a Source
//! declared in the design, a device that provides it on a board through a
//! catalogue profile, the provision judged, the provider lowered, the
//! generated adapter reading the line before the core steps, the host
//! bridge making the same inputs from the same readings, and the firmware
//! cross-compiled.  The theorems behind the shape are
//! `BDL_FV/BDL/Surface/Assignment.lean` (`assignSource`,
//! `two_providers_same_behavior`, `assignSource_origin_irrelevant`); what
//! is checked here is that production has the shape, not that the
//! theorems transfer (FVI-0029 stays open: one reading per tick, no batch).

#![allow(clippy::unwrap_used)]

mod support;

use bdl_codegen_rust::adapter::SourceKind;
use bdl_compiler::deploy_report::MissingKind;
use bdl_compiler::{
    analyze, analyze_deployment, compile, compile_for_target, deployment_report, CompileOptions,
    DeploymentStatus, TargetOptions,
};
use bdl_diagnostics::Severity;
use bdl_exec_ir::interp::{self, CellState};
use bdl_hardware::boards;
use bdl_model::edit::{apply_edit, EditError, EditOp};
use bdl_model::surface::{
    Definition, Design, DeviceKind, ProjectSnapshot, Representation, Signature,
};
use bdl_model::{
    ClockId, DeclId, DeviceId, Dim, InputProfileId, OutputId, OutputProfileId, SemanticId,
};
use bdl_output::provision::ProvisionStatus;
use bdl_reactive::Value;
use bdl_runtime_host::{AdapterOp, DynValue, RunRequest, TickRequest};

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
    fn value(
        &mut self,
        name: &str,
        output: SemanticId,
        formula: Option<&str>,
        clock: ClockId,
    ) -> DeclId {
        let id = self
            .edit(EditOp::CreateMapping {
                name: name.into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output,
                },
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
        self.edit(EditOp::SetMappingClock {
            id,
            clock: Some(clock),
        });
        id
    }
    fn output(
        &mut self,
        name: &str,
        accepts: SemanticId,
        clock: ClockId,
        driver: DeclId,
    ) -> OutputId {
        let o = self
            .edit(EditOp::CreateOutput {
                name: name.into(),
                description: String::new(),
                accepts,
                clock: Some(clock),
            })
            .created_output
            .unwrap();
        self.edit(EditOp::SetMappingDrive {
            id: driver,
            output: Some(o),
        });
        o
    }
    fn consumer(
        &mut self,
        name: &str,
        profile: &str,
        kind: DeviceKind,
        output: OutputId,
    ) -> DeviceId {
        let id = self
            .edit(EditOp::CreateDevice {
                name: name.into(),
                kind,
                output: Some(output),
            })
            .created_device
            .unwrap();
        self.edit(EditOp::SetDeviceRealization {
            id,
            profile: Some(OutputProfileId(profile.into())),
            kind,
        });
        id
    }
    fn provider(
        &mut self,
        name: &str,
        profile: &str,
        kind: DeviceKind,
        source: DeclId,
    ) -> DeviceId {
        let id = self
            .edit(EditOp::CreateDevice {
                name: name.into(),
                kind,
                output: None,
            })
            .created_device
            .unwrap();
        self.edit(EditOp::SetDeviceSource {
            id,
            source: Some(source),
        });
        self.edit(EditOp::SetDeviceProvider {
            id,
            profile: Some(InputProfileId(profile.into())),
            kind,
        });
        id
    }
}

/// Button → rule → Lamp: `pressed` is a Source carrying a truth value,
/// `lit = !pressed` a value of it, `lamp` an output driven by `lit`.  On
/// the board a digital input provides `pressed`, a digital output
/// realises `lamp`.
struct Fixture {
    s: ProjectSnapshot,
    pressed: DeclId,
    lamp: OutputId,
    button: DeviceId,
    coil: DeviceId,
}

fn fixture(name: &str, provider: Option<&str>) -> Fixture {
    let mut d = Surface::new(name);
    let pressed_c = d.concept("Pressed", Representation::Boolean);
    let lit_c = d.concept("Lit", Representation::Boolean);
    let main = d.clock("main");
    let pressed = d.value("pressed", pressed_c, None, main);
    let lit = d.value("lit", lit_c, Some("!pressed"), main);
    let lamp = d.output("lamp", lit_c, main, lit);
    let coil = d.consumer("coil", "gpio_level", DeviceKind::DigitalOutput, lamp);
    let button = match provider {
        Some(p) => d.provider("button", p, DeviceKind::DigitalInput, pressed),
        None => DeviceId::from_raw(u64::MAX),
    };
    Fixture {
        s: d.s,
        pressed,
        lamp,
        button,
        coil,
    }
}

fn pico() -> bdl_hardware::Hardware {
    boards::rp2040_pico()
}

fn options() -> CompileOptions {
    CompileOptions {
        require_complete: true,
        ..support::options()
    }
}

fn codes(ds: &[bdl_diagnostics::Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

fn ticks(readings: &[bool]) -> RunRequest {
    RunRequest {
        ticks: readings
            .iter()
            .map(|r| TickRequest {
                active: vec![0],
                inputs: vec![],
                readings: vec![Some(DynValue::Bool { value: *r })],
            })
            .collect(),
    }
}

// ---- the chain: Source → assignment → profile → requirement → resource → adapter → input ----

#[test]
fn a_button_provides_the_source_and_the_lamp_follows_on_the_pico() {
    let f = fixture("pico_button", Some("gpio_level_in"));
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(
        deployment.status,
        DeploymentStatus::Feasible,
        "{:?}",
        deployment.diagnostics
    );
    assert!(deployment.unprovided_sources.is_empty());
    assert!(
        deployment.unbound_devices.is_empty(),
        "a provider is not an unbound device"
    );
    let p = &deployment.provisions[&f.pressed];
    assert_eq!(p.device, Some(f.button));
    assert_eq!(p.check.status, ProvisionStatus::Valid);
    assert!(
        p.check.profile_known()
            && p.check.transducer_well_formed()
            && p.check.representation_fits()
    );
    assert!(p.hardware_placed && p.backend_supported && p.admissible());
    // the provider's requirement is a digital-in line, placed on a pad
    let req = deployment
        .requirements
        .iter()
        .find(|r| r.id.device == f.button)
        .expect("the button has a requirement");
    assert_eq!(req.capability, bdl_hardware::Capability::DigitalIn);
    let pad = deployment.assignment.as_ref().unwrap()[&req.id].clone();
    assert!(pad.0.starts_with("GP"), "{}", pad.0);
    // no diagnostic about the Source: it is provided
    assert!(
        codes(&deployment.diagnostics).is_empty(),
        "{:?}",
        deployment.diagnostics
    );
    let report = deployment_report(&f.s, &analyze(&f.s), &deployment, &pico());
    assert!(report.deployable, "{:?}", report.missing);

    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let exec = art.exec_ir.as_ref().unwrap();
    assert_eq!(exec.inputs.len(), 1);
    assert_eq!(exec.providers.len(), 1);
    assert_eq!(exec.providers[0].device, f.button);
    assert_eq!(exec.providers[0].profile.as_str(), "gpio_level_in");
    let g = art.generated.as_ref().unwrap();
    let adapter = g.manifest.adapter.as_ref().unwrap();
    assert_eq!(adapter.sources.len(), 1);
    assert_eq!(adapter.sources[0].device_id, f.button.raw());
    assert_eq!(adapter.sources[0].resource, pad.0);
    assert_eq!(adapter.sources[0].capability, "digital_in");
    let glue = &g.files["src/adapter.rs"];
    assert!(glue.contains("pub fn provide("), "{glue}");
    assert!(glue.contains("pub fn read("), "{glue}");
    assert!(glue.contains("SOURCES"), "{glue}");
    let fw = &g.files["src/bin/rp2040.rs"];
    assert!(fw.contains("Sense::pull_down"), "{fw}");
    assert!(fw.contains("design::adapter::read("), "{fw}");
    // the reading happens before the step, the commands after
    let read_at = fw.find("design::adapter::read(").unwrap();
    let step_at = fw.find("design::step(").unwrap();
    let apply_at = fw.find("design::adapter::apply(").unwrap();
    assert!(read_at < step_at && step_at < apply_at);
}

#[test]
fn the_host_makes_the_same_inputs_from_the_same_readings() {
    let f = fixture("pico_button_host", Some("gpio_level_in"));
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("pico_button_host");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let cargo = support::cargo_for("pico_button_host");
    cargo.check_core().unwrap_or_else(|e| panic!("{e}"));
    let readings = [false, true, true, false, true];
    let trace = cargo
        .run_host(&ticks(&readings))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(trace.error, None);
    // the interpreter: the same provider term over the same readings
    let exec = art.exec_ir.as_ref().unwrap();
    for (t, r) in readings.iter().enumerate() {
        let raw = vec![Some(Value::Bool { value: *r })];
        let inputs = interp::provide(exec, &raw).unwrap();
        assert!(
            matches!(&inputs[0], Some(Value::Semantic { repr, .. }) if **repr == Value::Bool { value: *r })
        );
        let step = interp::step(
            exec,
            t as u64,
            &[bdl_exec_ir::ClockSlot(0)],
            &CellState { cells: vec![] },
            &inputs,
        )
        .unwrap();
        // the lamp is the negation of the button
        assert_eq!(
            step.commands[0],
            Some(Value::Bool { value: !*r }),
            "tick {t}"
        );
        let host = &trace.ticks[t];
        assert_eq!(
            host.commands[0],
            Some(DynValue::Bool { value: !*r }),
            "tick {t}"
        );
        assert_eq!(
            host.adapter[0],
            AdapterOp::Level {
                device_id: f.coil.raw(),
                high: !*r
            }
        );
    }
    // a reading not taken on a due tick is a missing input, never a default
    let missing = cargo
        .run_host(&RunRequest {
            ticks: vec![TickRequest {
                active: vec![0],
                inputs: vec![],
                readings: vec![None],
            }],
        })
        .unwrap();
    assert!(missing.error.is_some());
}

// ---- replacement invariance: two providers, one semantic trace ----------

#[test]
fn two_providers_with_the_same_semantic_trace_give_the_same_behavior() {
    // The active-high line reads `r`; the active-low line reads `!r`;
    // both provide the Source with the same value, so behavior — values,
    // outputs, commands — is the same (`two_providers_same_behavior`).
    let high = fixture("pico_button_high", Some("gpio_level_in"));
    let low = fixture("pico_button_low", Some("gpio_level_in_low"));
    let pattern = [true, false, false, true, true, false];
    let mut traces = Vec::new();
    for (name, f, invert) in [
        ("pico_button_high", &high, false),
        ("pico_button_low", &low, true),
    ] {
        let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
        assert!(art.succeeded(), "{:?}", art.diagnostics);
        let g = art.generated.as_ref().unwrap();
        // the core is target-independent and provider-independent
        let plain = compile(&f.s, &options());
        assert_eq!(
            plain.exec_ir.as_ref().unwrap().decls,
            art.exec_ir.as_ref().unwrap().decls
        );
        let dir = support::generated_dir(name);
        bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
        let readings: Vec<bool> = pattern.iter().map(|r| *r != invert).collect();
        let trace = support::cargo_for(name)
            .run_host(&ticks(&readings))
            .unwrap();
        assert_eq!(trace.error, None);
        traces.push(trace);
        // and the firmware differs only in the pull
        let fw = &g.files["src/bin/rp2040.rs"];
        assert!(fw.contains(if invert {
            "Sense::pull_up"
        } else {
            "Sense::pull_down"
        }));
    }
    let (a, b) = (&traces[0], &traces[1]);
    for (ta, tb) in a.ticks.iter().zip(&b.ticks) {
        assert_eq!(ta.values, tb.values);
        assert_eq!(ta.outputs, tb.outputs);
        assert_eq!(ta.commands, tb.commands);
        assert_eq!(ta.adapter, tb.adapter);
    }
    // the interpreter agrees, provider by provider
    for f in [&high, &low] {
        let exec = compile(&f.s, &options()).exec_ir.unwrap();
        let invert = exec.providers[0].profile.as_str() == "gpio_level_in_low";
        for r in pattern {
            let raw = vec![Some(Value::Bool { value: r != invert })];
            let inputs = interp::provide(&exec, &raw).unwrap();
            let step = interp::step(
                &exec,
                0,
                &[bdl_exec_ir::ClockSlot(0)],
                &CellState { cells: vec![] },
                &inputs,
            )
            .unwrap();
            assert_eq!(step.commands[0], Some(Value::Bool { value: !r }));
        }
    }
}

// ---- the simulation is hardware-independent -----------------------------

#[test]
fn the_simulation_never_sees_a_provider() {
    let with = fixture("sim_button_with", Some("gpio_level_in"));
    let without = fixture("sim_button_without", None);
    let a = analyze(&with.s);
    let b = analyze(&without.s);
    assert_eq!(a.ir, b.ir, "the design IR has no device in it");
    assert_eq!(a.outputs, b.outputs);
    // lowering without a target: the provider is below the inputs, the
    // program above is the same
    let ea = compile(&with.s, &support::options()).exec_ir.unwrap();
    let eb = compile(&without.s, &support::options()).exec_ir.unwrap();
    assert_eq!(ea.decls, eb.decls);
    assert_eq!(ea.inputs, eb.inputs);
    assert_eq!(ea.outputs, eb.outputs);
    assert_eq!(ea.sinks, eb.sinks);
    assert_eq!(ea.providers.len(), 1);
    assert!(eb.providers.is_empty());
    // a simulation supplies the input directly, provider or not
    let value = Some(Value::sem(
        with.s.design.mappings[&with.pressed].signature.output,
        Value::Bool { value: true },
    ));
    for e in [&ea, &eb] {
        let step = interp::step(
            e,
            0,
            &[bdl_exec_ir::ClockSlot(0)],
            &CellState { cells: vec![] },
            std::slice::from_ref(&value),
        )
        .unwrap();
        assert_eq!(step.commands[0], Some(Value::Bool { value: false }));
    }
}

// ---- backward compatibility: an old project with a Source -----------------

#[test]
fn a_source_without_a_device_is_an_incomplete_deployment_with_the_reason() {
    let f = fixture("pico_button_none", None);
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(deployment.status, DeploymentStatus::Incomplete);
    assert_eq!(
        deployment
            .unprovided_sources
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [f.pressed]
    );
    let p = &deployment.provisions[&f.pressed];
    assert_eq!(p.device, None);
    assert_eq!(p.check.status, ProvisionStatus::NotChosen);
    assert!(!p.check.is_blocking(), "nothing chosen is not an error");
    assert_eq!(codes(&deployment.diagnostics), ["deploy.source_unprovided"]);
    assert_eq!(deployment.diagnostics[0].severity, Severity::Info);
    assert!(deployment.diagnostics[0].message.contains("pressed"));
    let report = deployment_report(&f.s, &analyze(&f.s), &deployment, &pico());
    assert!(!report.deployable);
    assert!(report.design_ready, "the design itself is fine");
    let kinds: Vec<MissingKind> = report.missing.iter().map(|m| m.kind).collect();
    assert_eq!(kinds, [MissingKind::SourceNoDevice]);
    assert_eq!(report.missing[0].mapping, Some(f.pressed));
    // the design still compiles without a target, and refuses a board by name
    assert!(compile(&f.s, &options()).succeeded());
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert_eq!(codes(&art.diagnostics), ["adapter.source_unprovided"]);
    assert!(art.diagnostics[0].message.contains("pressed"));
    // the persisted form of the old project is untouched: no provider field
    let json = serde_json::to_string(&f.s.design.devices).unwrap();
    assert!(
        !json.contains("\"provider\"") && !json.contains("\"source\""),
        "{json}"
    );
}

#[test]
fn a_device_bound_to_a_source_without_a_profile_is_named() {
    let mut f = fixture("pico_button_nochoice", None);
    let id = apply_edit(
        &f.s,
        &EditOp::CreateDevice {
            name: "button".into(),
            kind: DeviceKind::DigitalInput,
            output: None,
        },
    )
    .unwrap();
    f.s = id.snapshot;
    let button = id.outcome.created_device.unwrap();
    f.s = apply_edit(
        &f.s,
        &EditOp::SetDeviceSource {
            id: button,
            source: Some(f.pressed),
        },
    )
    .unwrap()
    .snapshot;
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(
        deployment.status,
        DeploymentStatus::Feasible,
        "the device places by kind"
    );
    assert_eq!(
        codes(&deployment.diagnostics),
        ["deploy.provider_unspecified"]
    );
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert_eq!(codes(&art.diagnostics), ["adapter.provider_unspecified"]);
}

// ---- diagnostics: unknown, incompatible, kind mismatch, unsupported, contested ----

#[test]
fn an_unknown_provider_profile_blocks_with_its_id() {
    let f = fixture("pico_button_unknown", Some("gpio_from_the_future"));
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(
        deployment.provisions[&f.pressed].check.status,
        ProvisionStatus::UnknownProfile
    );
    assert!(deployment.provision_blocked());
    assert_eq!(
        codes(&deployment.diagnostics),
        ["deploy.provider_unknown_profile"]
    );
    assert!(deployment.diagnostics[0]
        .explanation
        .contains("gpio_from_the_future"));
    let report = deployment_report(&f.s, &analyze(&f.s), &deployment, &pico());
    assert!(!report.deployable);
    assert_eq!(report.missing[0].kind, MissingKind::ProviderInvalid);
    assert_eq!(report.missing[0].device, Some(f.button));
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(
        codes(&art.diagnostics).contains(&"backend.provider_invalid"),
        "{:?}",
        art.diagnostics
    );
}

#[test]
fn a_provider_that_reads_another_representation_does_not_fit() {
    let mut d = Surface::new("pico_level_source");
    let level_c = d.concept("Level", Representation::Quantity { dim: Dim::ZERO });
    let main = d.clock("main");
    let level = d.value("level", level_c, None, main);
    let out = d.value("out", level_c, Some("level"), main);
    let light = d.output("light", level_c, main, out);
    d.consumer("lamp", "pwm_duty8", DeviceKind::PwmChannel, light);
    d.provider("sensor", "gpio_level_in", DeviceKind::DigitalInput, level);
    let deployment = analyze_deployment(&d.s, &pico());
    assert!(matches!(
        deployment.provisions[&level].check.status,
        ProvisionStatus::Incompatible(_)
    ));
    assert!(deployment.provisions[&level].check.transducer_well_formed());
    assert!(!deployment.provisions[&level].check.representation_fits());
    assert_eq!(
        codes(&deployment.diagnostics),
        ["deploy.provider_incompatible"]
    );
    assert!(deployment.diagnostics[0].message.contains("sensor"));
    // every candidate says whether it fits: neither line profile does
    assert!(deployment.provisions[&level]
        .candidates
        .iter()
        .all(|(_, fit)| *fit == Some(false)));
}

#[test]
fn a_kind_chosen_by_hand_releases_the_provider_and_a_mismatch_is_named() {
    let mut f = fixture("pico_button_kind", Some("gpio_level_in"));
    // the edit keeps kind and profile together …
    f.s = apply_edit(
        &f.s,
        &EditOp::SetDeviceKind {
            id: f.button,
            kind: DeviceKind::PwmChannel,
        },
    )
    .unwrap()
    .snapshot;
    assert_eq!(f.s.design.devices[&f.button].provider, None);
    // … so a mismatch only arises from a hand-edited file
    let mut design = f.s.design.clone();
    design.devices.get_mut(&f.button).unwrap().provider =
        Some(InputProfileId("gpio_level_in".into()));
    let s = ProjectSnapshot::new(design);
    let deployment = analyze_deployment(&s, &pico());
    assert!(matches!(
        deployment.provisions[&f.pressed].check.status,
        ProvisionStatus::KindMismatch { .. }
    ));
    assert_eq!(
        codes(&deployment.diagnostics),
        ["deploy.provider_kind_mismatch"]
    );
}

#[test]
fn a_board_without_a_reader_is_named_and_the_placement_is_untouched() {
    let f = fixture("nano_button", Some("gpio_level_in"));
    let nano = boards::arduino_nano();
    let deployment = analyze_deployment(&f.s, &nano);
    assert_eq!(
        deployment.status,
        DeploymentStatus::Feasible,
        "{:?}",
        deployment.diagnostics
    );
    let p = &deployment.provisions[&f.pressed];
    assert!(p.check.is_valid() && p.hardware_placed);
    assert!(!p.backend_supported);
    assert!(!p.admissible());
    assert_eq!(
        codes(&deployment.diagnostics),
        ["deploy.provider_unsupported"]
    );
    assert_eq!(deployment.diagnostics[0].severity, Severity::Warning);
    let art = compile_for_target(&f.s, &nano, &TargetOptions::default(), &options());
    assert_eq!(codes(&art.diagnostics), ["adapter.provider_unsupported"]);
    assert!(art.diagnostics[0].message.contains("Arduino Nano"));
}

#[test]
fn a_source_provided_twice_is_contested() {
    let mut f = fixture("pico_button_twice", Some("gpio_level_in"));
    let mut d = Surface { s: f.s.clone() };
    d.provider(
        "button2",
        "gpio_level_in_low",
        DeviceKind::DigitalInput,
        f.pressed,
    );
    f.s = d.s;
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(codes(&deployment.diagnostics), ["deploy.source_contested"]);
    assert!(deployment.provision_blocked());
    assert_eq!(
        deployment.provisions[&f.pressed].device,
        Some(f.button),
        "the first device provides"
    );
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert_eq!(codes(&art.diagnostics), ["adapter.provider_invalid"]);
}

#[test]
fn a_line_the_board_cannot_give_is_the_placement_s_refusal() {
    let mut f = fixture("pico_button_pinned", Some("gpio_level_in"));
    f.s = apply_edit(
        &f.s,
        &EditOp::SetDevicePin {
            id: f.button,
            index: 0,
            resource: Some("GP99".into()),
        },
    )
    .unwrap()
    .snapshot;
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(deployment.status, DeploymentStatus::Infeasible);
    assert!(!deployment.provisions[&f.pressed].hardware_placed);
    assert!(
        deployment.provisions[&f.pressed].check.is_valid(),
        "the contract still holds"
    );
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(codes(&art.diagnostics).contains(&"adapter.deployment_not_feasible"));
}

// ---- the model keeps consume and provide apart -----------------------------

#[test]
fn a_device_consumes_or_provides_never_both() {
    let mut f = fixture("model_button", Some("gpio_level_in"));
    let button = f.button;
    // binding an output releases the Source and its provider
    f.s = apply_edit(
        &f.s,
        &EditOp::SetDeviceOutput {
            id: button,
            output: Some(f.lamp),
        },
    )
    .unwrap()
    .snapshot;
    let d = &f.s.design.devices[&button];
    assert_eq!(
        (d.output, d.source, d.provider.clone()),
        (Some(f.lamp), None, None)
    );
    // binding a Source releases the output and its realization
    f.s = apply_edit(
        &f.s,
        &EditOp::SetDeviceSource {
            id: button,
            source: Some(f.pressed),
        },
    )
    .unwrap()
    .snapshot;
    let d = &f.s.design.devices[&button];
    assert_eq!(
        (d.output, d.source, d.realization.clone()),
        (None, Some(f.pressed), None)
    );
    // only a Source can be provided
    let lit =
        f.s.design
            .mappings
            .values()
            .find(|m| m.name == "lit")
            .unwrap()
            .id;
    let err = apply_edit(
        &f.s,
        &EditOp::SetDeviceSource {
            id: button,
            source: Some(lit),
        },
    )
    .unwrap_err();
    assert_eq!(err, EditError::NotASource { id: lit });
    // deleting the Source releases the device
    f.s = apply_edit(&f.s, &EditOp::DeleteMapping { id: f.pressed })
        .unwrap()
        .snapshot;
    assert_eq!(f.s.design.devices[&button].source, None);
}

#[test]
fn source_kinds_follow_the_profile() {
    assert_eq!(
        bdl_compiler::target::source_kind(&InputProfileId("gpio_level_in".into())),
        Some(SourceKind::LevelPullDown)
    );
    assert_eq!(
        bdl_compiler::target::source_kind(&InputProfileId("gpio_level_in_low".into())),
        Some(SourceKind::LevelPullUp)
    );
    assert_eq!(
        bdl_compiler::target::source_kind(&InputProfileId("pwm_duty8".into())),
        None
    );
}

// ---- the firmware cross-compiles with a Source ----------------------------

#[test]
fn the_firmware_with_a_button_cross_compiles_for_the_pico() {
    let installed = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("thumbv6m-none-eabi"))
        .unwrap_or(false);
    if !installed {
        if std::env::var("BDL_REQUIRE_CROSS").is_ok() {
            panic!("thumbv6m-none-eabi is not installed (rust-toolchain.toml lists it: `rustup target add thumbv6m-none-eabi`)");
        }
        eprintln!("skipping: thumbv6m-none-eabi not installed");
        return;
    }
    let f = fixture("pico_button_fw", Some("gpio_level_in_low"));
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("pico_button_fw");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let elf = support::cargo_for("pico_button_fw")
        .build_firmware("rp2040", "thumbv6m-none-eabi")
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(std::fs::read(&elf).unwrap().starts_with(b"\x7fELF"));
}
