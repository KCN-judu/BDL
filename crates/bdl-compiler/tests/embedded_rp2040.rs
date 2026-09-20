//! The first embedded platform adapter, end to end
//! (docs/architecture/embedded-adapter.md): a design that defines only
//! logical outputs, realised as raw commands, compiled for the Raspberry
//! Pi Pico — the machine sinks bound to the solver-assigned pads, the
//! firmware generated over Embassy, the host bridge applying the same
//! glue to recording sinks, and the firmware cross-compiled for
//! `thumbv6m-none-eabi`.  Nothing here is a proof of the adapter's
//! physical correspondence (FVI-0022, ISS-0017): it is production
//! evidence.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_codegen_rust::adapter::SinkKind;
use bdl_compiler::collections::{CollectionsReadiness, CollectionsReport};
use bdl_compiler::{
    adapter_plan, analyze_deployment, compile, compile_for_target, CompileOptions, TargetOptions,
};
use bdl_hardware::{boards, Capability, ResourceId};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{
    Definition, Design, DeviceKind, ProjectSnapshot, Representation, Signature,
};
use bdl_model::{ClockId, DeclId, DeviceId, Dim, OutputId, OutputProfileId, SemanticId};
use bdl_runtime_adapter::{duty8, CommandFault};
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
    fn device(
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
}

/// A design with no Source: a level that climbs 5 % per tick from 0 drives
/// `light` as `pwm_duty8`, and a switch that toggles every tick drives
/// `relay` as `gpio_level`.  Past 100 % the raw duty exceeds 255 — what
/// the adapter's policy refuses, on purpose.
struct Fixture {
    s: ProjectSnapshot,
    lamp: DeviceId,
    coil: DeviceId,
}

fn fixture(name: &str) -> Fixture {
    let mut d = Surface::new(name);
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let state = d.concept("SwitchState", Representation::Boolean);
    let main = d.clock("main");
    let level = d.value("level", brightness, Some("delay(0, level + 5)"), main);
    let on = d.value("on", state, Some("delay(false, !on)"), main);
    let light = d.output("light", brightness, main, level);
    let relay = d.output("relay", state, main, on);
    let lamp = d.device("lamp", "pwm_duty8", DeviceKind::PwmChannel, light);
    let coil = d.device("coil", "gpio_level", DeviceKind::DigitalOutput, relay);
    Fixture { s: d.s, lamp, coil }
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

// ---- C, D, G, O: the sink ↔ resource ↔ peripheral chain ----------------

#[test]
fn every_sink_is_bound_to_its_assigned_pad_and_nothing_else() {
    let f = fixture("pico_lamp");
    let deployment = analyze_deployment(&f.s, &pico());
    assert_eq!(deployment.status, bdl_compiler::DeploymentStatus::Feasible);
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    for file in [
        "src/adapter.rs",
        "src/bin/rp2040.rs",
        "memory.x",
        "build.rs",
        ".cargo/config.toml",
    ] {
        assert!(g.files.contains_key(file), "{file} missing");
    }
    let adapter = g.manifest.adapter.as_ref().expect("adapter entry");
    assert_eq!(
        (adapter.board.as_str(), adapter.family.as_str()),
        ("rp2040_pico", "rp2040")
    );
    assert_eq!(adapter.triple, "thumbv6m-none-eabi");
    assert_eq!(adapter.tick_micros, 10_000);
    assert_eq!(adapter.periods, vec![1]);
    assert_eq!(adapter.arena_bytes, None);
    assert_eq!(adapter.bindings.len(), 2);

    // Each binding names the pad the placement assigned — by device id and
    // resource id — and that pad carries the capability; the peripheral
    // follows the datasheet from the pad number alone.
    let placed = |device: DeviceId, cap: Capability| -> ResourceId {
        let req = deployment
            .requirements
            .iter()
            .find(|r| r.id.device == device && r.capability == cap)
            .unwrap();
        deployment.assignment.as_ref().unwrap()[&req.id].clone()
    };
    let lamp_pad = placed(f.lamp, Capability::Pwm);
    let coil_pad = placed(f.coil, Capability::DigitalOut);
    let lamp = &adapter.bindings[0];
    assert_eq!(
        (lamp.device_id, lamp.kind.as_str(), lamp.profile.as_str()),
        (f.lamp.raw(), "pwm_duty8", "pwm_duty8")
    );
    assert_eq!(lamp.resource, lamp_pad.0);
    assert_eq!(lamp.capability, "pwm");
    assert!(pico().supports(&lamp_pad, Capability::Pwm));
    let n: u32 = lamp_pad.0.strip_prefix("GP").unwrap().parse().unwrap();
    assert_eq!(
        lamp.peripheral,
        format!(
            "PIN_{n} on PWM_SLICE{} channel {}",
            (n / 2) % 8,
            if n % 2 == 0 { "A" } else { "B" }
        )
    );
    let coil = &adapter.bindings[1];
    assert_eq!(
        (coil.device_id, coil.kind.as_str()),
        (f.coil.raw(), "level")
    );
    assert_eq!(coil.resource, coil_pad.0);
    assert_ne!(lamp.resource, coil.resource);

    // The generated glue and firmware bind exactly those symbols and pads;
    // no display name is a symbol anywhere.
    let glue = &g.files["src/adapter.rs"];
    assert!(glue.contains(&format!("symbol: \"command_{}\"", f.lamp.raw())));
    assert!(glue.contains(&format!("resource: \"{}\"", lamp_pad.0)));
    assert!(glue.contains(&format!("pub fn apply(tick: &crate::Tick, command_{}: &mut dyn PwmDuty8, command_{}: &mut dyn Level) -> Applied", f.lamp.raw(), f.coil.raw())));
    let fw = &g.files["src/bin/rp2040.rs"];
    let m: u32 = coil_pad.0.strip_prefix("GP").unwrap().parse().unwrap();
    assert!(
        fw.contains(&format!(
            "let mut command_{} = Pwm{}::new(p.PWM_SLICE{}, p.PIN_{n}, DEFAULT_PWM_DIVIDER);",
            f.lamp.raw(),
            if n % 2 == 0 { "A" } else { "B" },
            (n / 2) % 8
        )),
        "{fw}"
    );
    assert!(
        fw.contains(&format!(
            "let mut command_{} = Line::new(p.PIN_{m});",
            f.coil.raw()
        )),
        "{fw}"
    );
    assert!(fw.contains("#![no_std]") && fw.contains("#![no_main]"));
    assert!(fw.contains("Err(_) => halt()"));
    assert!(
        !fw.contains("\"lamp\"") && !fw.contains("\"coil\""),
        "names are not identity"
    );
    assert!(g.files["Cargo.toml"].contains("required-features = [\"rp2040\"]"));
}

// ---- J, K, M, N: the core and its trace are what they were --------------

#[test]
fn the_core_and_its_commands_are_unchanged_by_the_target() {
    let f = fixture("pico_lamp_same");
    let plain = compile(&f.s, &options());
    let target = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(
        plain.succeeded() && target.succeeded(),
        "{:?}",
        target.diagnostics
    );
    // The lowered program — declarations, cells, outputs, sinks — is
    // identical: the adapter enters nothing upstream of the commands.
    assert_eq!(plain.exec_ir, target.exec_ir);
    assert_eq!(plain.analysis, target.analysis);
    // The core source is the same up to the adapter module's declaration.
    let core_plain = plain.generated.as_ref().unwrap().core_source().to_string();
    let core_target = target.generated.as_ref().unwrap().core_source().to_string();
    let core_target_stripped = core_target
        .replace("#[cfg(feature = \"adapter\")]\npub mod adapter;\n", "")
        .replace("\n// The platform adapter's glue, generated beside the core (docs/architecture/embedded-adapter.md).\n", "\n");
    assert_eq!(core_plain.trim_end(), core_target_stripped.trim_end());
    let m = &target.generated.as_ref().unwrap().manifest;
    assert_eq!(m.sinks, plain.generated.as_ref().unwrap().manifest.sinks);
}

// ---- L, E, F, A: the host applies the same glue to recording sinks -------

#[test]
fn host_adapter_operations_correspond_to_the_commands() {
    let f = fixture("pico_lamp_host");
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("pico_lamp_host");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let cargo = support::cargo_for("pico_lamp_host");
    // A: the core alone, no features, still builds as no_std.
    cargo.check_core().unwrap_or_else(|e| panic!("{e}"));
    let ticks = 24u64;
    let req = RunRequest {
        ticks: (0..ticks)
            .map(|_| TickRequest {
                active: vec![0],
                inputs: vec![],
                readings: vec![],
            })
            .collect(),
    };
    let trace = cargo.run_host(&req).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(trace.error, None);
    assert_eq!(trace.ticks.len(), ticks as usize);
    let mut refused = 0;
    for (t, tick) in trace.ticks.iter().enumerate() {
        assert_eq!(tick.commands.len(), 2);
        assert_eq!(tick.adapter.len(), 2, "one operation per sink");
        // the PWM sink: the raw command through the numeric policy
        let raw = match &tick.commands[0] {
            Some(DynValue::Quantity { value }) => *value,
            other => panic!("tick {t}: {other:?}"),
        };
        let expected_raw = 5.0 * t as f64 * 255.0 / 100.0;
        assert!((raw - expected_raw).abs() < 1e-9, "tick {t}: {raw}");
        let want = match duty8(raw) {
            Ok(duty) => AdapterOp::Pwm {
                device_id: f.lamp.raw(),
                duty,
            },
            Err(fault) => {
                refused += 1;
                AdapterOp::Refused {
                    device_id: f.lamp.raw(),
                    fault: format!("{fault:?}"),
                }
            }
        };
        assert_eq!(tick.adapter[0], want, "tick {t}");
        // the GPIO sink: the truth value as written
        let high = match &tick.commands[1] {
            Some(DynValue::Bool { value }) => *value,
            other => panic!("tick {t}: {other:?}"),
        };
        assert_eq!(high, t % 2 == 1);
        assert_eq!(
            tick.adapter[1],
            AdapterOp::Level {
                device_id: f.coil.raw(),
                high
            }
        );
    }
    // exact points of the policy on the way up: 0 %, 5 % (12.75 → 13),
    // 50 % (127.5 → 128), 100 % (255), then out of range
    let duty_at = |t: usize| match &trace.ticks[t].adapter[0] {
        AdapterOp::Pwm { duty, .. } => Some(*duty),
        _ => None,
    };
    assert_eq!(duty_at(0), Some(0));
    assert_eq!(duty_at(1), Some(13));
    assert_eq!(duty_at(10), Some(128));
    assert_eq!(duty_at(20), Some(255));
    assert_eq!(duty_at(21), None);
    assert_eq!(
        trace.ticks[21].adapter[0],
        AdapterOp::Refused {
            device_id: f.lamp.raw(),
            fault: format!("{:?}", CommandFault::OutOfRange { min: 0, max: 255 })
        }
    );
    assert_eq!(refused, 3);
    // a domain that is not due holds every line
    let idle = cargo
        .run_host(&RunRequest {
            ticks: vec![TickRequest {
                active: vec![],
                inputs: vec![],
                readings: vec![],
            }],
        })
        .unwrap();
    assert_eq!(
        idle.ticks[0].adapter,
        vec![
            AdapterOp::Held {
                device_id: f.lamp.raw()
            },
            AdapterOp::Held {
                device_id: f.coil.raw()
            }
        ]
    );
}

/// The quantized profile realises the same output on the same board.
#[test]
fn the_quantized_profile_binds_and_applies_like_the_eight_bit_one() {
    let mut d = Surface::new("pico_quantized");
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let main = d.clock("main");
    let level = d.value("level", brightness, Some("delay(0, level + 30)"), main);
    let light = d.output("light", brightness, main, level);
    let lamp = d.device("lamp", "pwm_duty4", DeviceKind::PwmChannel, light);
    let art = compile_for_target(&d.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let b = &art
        .generated
        .as_ref()
        .unwrap()
        .manifest
        .adapter
        .as_ref()
        .unwrap()
        .bindings[0];
    assert_eq!(
        (b.device_id, b.profile.as_str(), b.kind.as_str()),
        (lamp.raw(), "pwm_duty4", "pwm_duty8")
    );
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("pico_quantized");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let trace = support::cargo_for("pico_quantized")
        .run_host(&RunRequest {
            ticks: (0..4)
                .map(|_| TickRequest {
                    active: vec![0],
                    inputs: vec![],
                    readings: vec![],
                })
                .collect(),
        })
        .unwrap();
    // levels 0, 30, 60, 90 → duties 0, 85, 170, 255: exact, no rounding needed
    let duties: Vec<&AdapterOp> = trace.ticks.iter().map(|t| &t.adapter[0]).collect();
    assert_eq!(
        duties,
        [0u8, 85, 170, 255]
            .iter()
            .map(|d| AdapterOp::Pwm {
                device_id: lamp.raw(),
                duty: *d
            })
            .collect::<Vec<_>>()
            .iter()
            .collect::<Vec<_>>()
    );
}

// ---- B, Q: the firmware cross-compiles, no_std -------------------------

#[test]
fn the_firmware_cross_compiles_for_the_pico() {
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
    let f = fixture("pico_lamp_fw");
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("pico_lamp_fw");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let elf = support::cargo_for("pico_lamp_fw")
        .build_firmware("rp2040", "thumbv6m-none-eabi")
        .unwrap_or_else(|e| panic!("{e}"));
    let bytes = std::fs::read(&elf).unwrap();
    assert!(bytes.starts_with(b"\x7fELF"), "{}", elf.display());
    // Q: the crate's core and adapter carry no std; the firmware's only
    // std-dependent file is build.rs, which runs on the host.
    for (path, text) in &g.files {
        if path.ends_with(".rs") && path != "build.rs" && path != "src/bin/host.rs" {
            assert!(
                !text.contains("use std::") && !text.contains("std::"),
                "{path} names std"
            );
        }
    }
    assert!(g.files["src/lib.rs"].contains("#![no_std]"));
    assert!(g.files["src/bin/rp2040.rs"].contains("#![no_std]"));
}

// ---- H, I, and the other refusals ----------------------------------------

fn plan_inputs(
    f: &Fixture,
) -> (
    bdl_exec_ir::ExecIr,
    bdl_compiler::DeploymentAnalysis,
    CollectionsReport,
) {
    let art = compile(&f.s, &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let deployment = analyze_deployment(&f.s, &pico());
    (art.exec_ir.unwrap(), deployment, art.collections.unwrap())
}

fn codes(ds: &[bdl_diagnostics::Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

#[test]
fn a_sink_without_a_placed_resource_is_refused_not_defaulted() {
    let f = fixture("pico_unbound");
    let (exec, mut deployment, collections) = plan_inputs(&f);
    let lamp_req = deployment
        .requirements
        .iter()
        .find(|r| r.id.device == f.lamp)
        .unwrap()
        .id;
    deployment.assignment.as_mut().unwrap().remove(&lamp_req);
    let err = adapter_plan(
        &exec,
        &pico(),
        &deployment,
        &collections,
        None,
        &TargetOptions::default(),
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["adapter.sink_unbound"]);
    assert!(err[0].message.contains("lamp"));
}

#[test]
fn a_resource_without_the_capability_is_an_inconsistency_not_a_fallback() {
    let f = fixture("pico_incompatible");
    let (exec, mut deployment, collections) = plan_inputs(&f);
    let lamp_req = deployment
        .requirements
        .iter()
        .find(|r| r.id.device == f.lamp)
        .unwrap()
        .id;
    // GP25 (the LED) has no PWM pad
    deployment
        .assignment
        .as_mut()
        .unwrap()
        .insert(lamp_req, ResourceId::new("GP25"));
    let err = adapter_plan(
        &exec,
        &pico(),
        &deployment,
        &collections,
        None,
        &TargetOptions::default(),
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["adapter.resource_incompatible"]);
    assert!(err[0].message.contains("GP25"));
    // and a pad the target does not have is refused by the target entry
    let mut plan = adapter_plan(
        &exec,
        &pico(),
        &analyze_deployment(&f.s, &pico()),
        &collections,
        None,
        &TargetOptions::default(),
    )
    .unwrap();
    plan.sinks[0].resource = "D3".into();
    assert!(
        bdl_codegen_rust::generate_with_adapter(&exec, &options().codegen, Some(&plan)).is_err()
    );
}

#[test]
fn a_placement_the_solver_refuses_never_reaches_the_adapter() {
    let mut f = fixture("pico_pinned");
    // pin the lamp's PWM line by hand to the LED pad, which cannot do PWM
    let a = apply_edit(
        &f.s,
        &EditOp::SetDevicePin {
            id: f.lamp,
            index: 0,
            resource: Some("GP25".into()),
        },
    )
    .unwrap();
    f.s = a.snapshot;
    let art = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert!(!art.succeeded());
    assert_eq!(codes(&art.diagnostics), ["adapter.deployment_not_feasible"]);
    assert!(art.exec_ir.is_some(), "the lowering itself is untouched");
}

#[test]
fn a_source_without_a_device_is_refused_by_name() {
    let mut d = Surface::new("pico_source");
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let main = d.clock("main");
    let level = d.value("level", brightness, None, main);
    let light = d.output("light", brightness, main, level);
    d.device("lamp", "pwm_duty8", DeviceKind::PwmChannel, light);
    let art = compile_for_target(&d.s, &pico(), &TargetOptions::default(), &options());
    assert!(!art.succeeded());
    assert_eq!(codes(&art.diagnostics), ["adapter.source_unprovided"]);
    assert!(art.diagnostics[0].message.contains("level"));
}

#[test]
fn a_profile_the_adapter_cannot_drive_is_named() {
    let mut d = Surface::new("pico_i2c");
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let main = d.clock("main");
    let level = d.value("level", brightness, Some("delay(0, level + 5)"), main);
    let light = d.output("light", brightness, main, level);
    d.device("bus", "i2c_level8", DeviceKind::I2cSensor, light);
    let art = compile_for_target(&d.s, &pico(), &TargetOptions::default(), &options());
    assert!(!art.succeeded());
    assert_eq!(codes(&art.diagnostics), ["adapter.profile_unsupported"]);
    assert!(art.diagnostics[0].message.contains("i2c_level8"));
}

// ---- P: the arena follows the manifest's bounds --------------------------

#[test]
fn the_arena_is_sized_from_the_bounds_and_an_unbounded_design_is_refused() {
    let f = fixture("pico_arena");
    let (exec, deployment, mut collections) = plan_inputs(&f);
    assert_eq!(collections.readiness, CollectionsReadiness::ScalarOnly);
    let plan = adapter_plan(
        &exec,
        &pico(),
        &deployment,
        &collections,
        None,
        &TargetOptions::default(),
    )
    .unwrap();
    assert_eq!(plan.arena_bytes, None);
    assert_eq!(
        plan.sinks.iter().map(|s| s.kind).collect::<Vec<_>>(),
        [SinkKind::PwmDuty8, SinkKind::Level]
    );

    collections.readiness = CollectionsReadiness::Bounded;
    collections.state_bytes_max = Some(1000);
    collections.tick_bytes_max = Some(500);
    let plan = adapter_plan(
        &exec,
        &pico(),
        &deployment,
        &collections,
        None,
        &TargetOptions::default(),
    )
    .unwrap();
    assert_eq!(plan.arena_bytes, Some(2048));
    let g =
        bdl_codegen_rust::generate_with_adapter(&exec, &options().codegen, Some(&plan)).unwrap();
    assert_eq!(g.manifest.adapter.as_ref().unwrap().arena_bytes, Some(2048));
    let fw = &g.files["src/bin/rp2040.rs"];
    assert!(fw.contains("#[global_allocator]"));
    assert!(fw.contains("static ARENA: StaticCell<[u8; 2048]>"));
    assert!(fw.contains("ARENA.init([0_u8; 2048])"));
    assert!(g.files["Cargo.toml"].contains("static-cell"));

    for readiness in [
        CollectionsReadiness::InputBounded,
        CollectionsReadiness::Unbounded,
    ] {
        collections.readiness = readiness;
        let err = adapter_plan(
            &exec,
            &pico(),
            &deployment,
            &collections,
            None,
            &TargetOptions::default(),
        )
        .unwrap_err();
        assert_eq!(codes(&err), ["adapter.collections_unbounded"]);
    }
}

// ---- the schedule reaches the firmware as periods -------------------------

#[test]
fn the_compiled_schedule_becomes_the_firmwares_periods() {
    let mut d = Surface::new("pico_periods");
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let fast = d.clock("fast");
    let slow = d.clock("slow");
    let level = d.value("level", brightness, Some("delay(0, level + 5)"), slow);
    let light = d.output("light", brightness, slow, level);
    d.device("lamp", "pwm_duty8", DeviceKind::PwmChannel, light);
    let _ = fast;
    let mut opts = options();
    opts.schedule = Some(bdl_reactive::Schedule {
        periods: [(fast, 1), (slow, 4)].into_iter().collect(),
    });
    let art = compile_for_target(&d.s, &pico(), &TargetOptions { tick_micros: 2_500 }, &opts);
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let adapter = art
        .generated
        .as_ref()
        .unwrap()
        .manifest
        .adapter
        .as_ref()
        .unwrap();
    assert_eq!(adapter.tick_micros, 2_500);
    // one clock slot: only `slow` is used by the program (`fast` has no
    // declaration and no output, so the plan has no slot for it)
    assert_eq!(adapter.periods, vec![4]);
    let fw = &art.generated.as_ref().unwrap().files["src/bin/rp2040.rs"];
    assert!(
        fw.contains("pub const PERIODS: [u64; 1] = [4_u64];"),
        "{fw}"
    );
    assert!(fw.contains("pub const TICK_MICROS: u64 = 2500_u64;"));
    let art = compile_for_target(&d.s, &pico(), &TargetOptions { tick_micros: 0 }, &opts);
    assert_eq!(codes(&art.diagnostics), ["adapter.tick_invalid"]);
}

// ---- determinism ----------------------------------------------------------

#[test]
fn firmware_generation_is_deterministic() {
    let f = fixture("pico_det");
    let a = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    let b = compile_for_target(&f.s, &pico(), &TargetOptions::default(), &options());
    assert_eq!(a.generated, b.generated);
    assert_eq!(a.diagnostics, b.diagnostics);
}
