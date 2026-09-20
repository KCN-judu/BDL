//! The Arduino family's first board, the Nano, through the same adapter
//! (docs/architecture/embedded-adapter.md § Arduino): the machine sinks
//! bound to the solver-assigned lines, the firmware generated over
//! `avr-hal` with the timers the pins need, the host bridge applying the
//! same glue, the collections refused, and — where the pinned AVR nightly
//! and `avr-gcc` are installed — the firmware cross-compiled to an AVR ELF.
//! Production evidence, not a proof of the physical effect (FVI-0022).

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::collections::CollectionsReadiness;
use bdl_compiler::{
    adapter_plan, analyze_deployment, compile, compile_for_target, CompileOptions, TargetOptions,
};
use bdl_hardware::{boards, Capability, ResourceId};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{
    Definition, Design, DeviceKind, ProjectSnapshot, Representation, Signature,
};
use bdl_model::{ClockId, DeclId, DeviceId, Dim, OutputId, OutputProfileId, SemanticId};
use bdl_runtime_host::{AdapterOp, RunRequest, TickRequest};

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
    fn value(&mut self, name: &str, output: SemanticId, formula: &str, clock: ClockId) -> DeclId {
        self.edit(EditOp::CreateMapping {
            name: name.into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output,
            },
            definition: Some(Definition::Formula {
                source: formula.into(),
            }),
            clock: Some(clock),
        })
        .created_mapping
        .unwrap()
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

struct Fixture {
    s: ProjectSnapshot,
    lamp: DeviceId,
    coil: DeviceId,
}

/// The same design as the Pico's: a climbing level on `pwm_duty8`, a
/// toggling switch on `gpio_level`.
fn fixture(name: &str) -> Fixture {
    let mut d = Surface::new(name);
    let brightness = d.concept("Brightness", Representation::Quantity { dim: Dim::ZERO });
    let state = d.concept("SwitchState", Representation::Boolean);
    let main = d.clock("main");
    let level = d.value("level", brightness, "delay(0, level + 5)", main);
    let on = d.value("on", state, "delay(false, !on)", main);
    let light = d.output("light", brightness, main, level);
    let relay = d.output("relay", state, main, on);
    let lamp = d.device("lamp", "pwm_duty8", DeviceKind::PwmChannel, light);
    let coil = d.device("coil", "gpio_level", DeviceKind::DigitalOutput, relay);
    Fixture { s: d.s, lamp, coil }
}

fn nano() -> bdl_hardware::Hardware {
    boards::arduino_nano()
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

#[test]
fn every_sink_is_bound_to_its_assigned_line_and_timer() {
    let f = fixture("nano_lamp");
    let deployment = analyze_deployment(&f.s, &nano());
    assert_eq!(deployment.status, bdl_compiler::DeploymentStatus::Feasible);
    let art = compile_for_target(&f.s, &nano(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    for file in [
        "src/adapter.rs",
        "src/bin/arduino_nano.rs",
        ".cargo/config.toml",
    ] {
        assert!(g.files.contains_key(file), "{file} missing");
    }
    assert!(!g.files.contains_key("memory.x") && !g.files.contains_key("build.rs"));
    let adapter = g.manifest.adapter.as_ref().unwrap();
    assert_eq!(
        (adapter.board.as_str(), adapter.family.as_str()),
        ("arduino_nano", "avr")
    );
    assert_eq!(adapter.triple, "avr-none");
    assert_eq!(adapter.feature, "arduino_nano");
    assert_eq!(adapter.toolchain.as_deref(), Some("nightly-2025-04-27"));
    assert!(
        adapter.build.contains("-Zbuild-std=core"),
        "{}",
        adapter.build
    );
    assert_eq!(adapter.arena_bytes, None);

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
    // the Nano's first PWM line is D3, on timer 2
    assert_eq!(lamp_pad.0, "D3");
    let lamp = &adapter.bindings[0];
    assert_eq!(
        (lamp.device_id, lamp.resource.as_str()),
        (f.lamp.raw(), "D3")
    );
    assert_eq!(lamp.peripheral, "pins.d3 on Timer2Pwm (OC2x)");
    let coil = &adapter.bindings[1];
    assert_eq!(
        (coil.device_id, coil.resource.as_str()),
        (f.coil.raw(), coil_pad.0.as_str())
    );
    assert_ne!(lamp.resource, coil.resource);

    let fw = &g.files["src/bin/arduino_nano.rs"];
    assert!(fw.contains("#![no_std]") && fw.contains("#![no_main]"));
    assert!(fw.contains("#[arduino_hal::entry]"));
    assert!(
        fw.contains("let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);"),
        "{fw}"
    );
    assert!(
        !fw.contains("Timer0Pwm::new") && !fw.contains("Timer1Pwm::new"),
        "only the timers in use"
    );
    assert!(
        fw.contains(&format!(
            "let mut command_{} = PwmLine::new(pins.d3.into_output().into_pwm(&timer2));",
            f.lamp.raw()
        )),
        "{fw}"
    );
    let field = coil_pad.0.to_lowercase();
    assert!(
        fw.contains(&format!(
            "let mut command_{} = Line::new(pins.{field}.into_output());",
            f.coil.raw()
        )),
        "{fw}"
    );
    assert!(fw.contains("Err(_) => halt()") && fw.contains("tick_wait(TICK_MICROS);"));
    assert!(!fw.contains("embassy"), "no Embassy on the AVR");
    let cargo = &g.files["Cargo.toml"];
    assert!(cargo.contains("required-features = [\"arduino_nano\"]"));
    assert!(cargo.contains("arduino-hal = { git = \"https://github.com/Rahix/avr-hal\""));
    assert!(cargo.contains("rust-version = \"1.87\""));
    assert!(!cargo.contains("embassy-rp"));
    assert!(g.files[".cargo/config.toml"].contains("target-cpu=atmega328p"));
    assert!(
        !g.files[".cargo/config.toml"].contains("[build]"),
        "no default target: the host build is untouched"
    );
}

#[test]
fn the_core_and_its_commands_are_unchanged_by_the_board() {
    let f = fixture("nano_lamp_same");
    let plain = compile(&f.s, &options());
    let pico = compile_for_target(
        &f.s,
        &boards::rp2040_pico(),
        &TargetOptions::default(),
        &options(),
    );
    let nano = compile_for_target(&f.s, &nano(), &TargetOptions::default(), &options());
    assert!(
        plain.succeeded() && pico.succeeded() && nano.succeeded(),
        "{:?}",
        nano.diagnostics
    );
    assert_eq!(plain.exec_ir, nano.exec_ir);
    assert_eq!(pico.exec_ir, nano.exec_ir);
    // the glue is the same text for both boards: the bindings differ only
    // in the resource names, the `apply` is identical
    let glue = |a: &bdl_compiler::CompileArtifact| {
        a.generated.as_ref().unwrap().files["src/adapter.rs"]
            .lines()
            .filter(|l| l.starts_with("pub fn apply") || l.starts_with("pub struct Applied"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(glue(&pico), glue(&nano));
}

#[test]
fn host_operations_on_the_nano_crate_correspond_to_the_commands() {
    let f = fixture("nano_lamp_host");
    let art = compile_for_target(&f.s, &nano(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("nano_lamp_host");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let cargo = support::cargo_for("nano_lamp_host");
    cargo.check_core().unwrap_or_else(|e| panic!("{e}"));
    let trace = cargo
        .run_host(&RunRequest {
            ticks: (0..3)
                .map(|_| TickRequest {
                    active: vec![0],
                    inputs: vec![],
                    readings: vec![],
                })
                .collect(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let duties: Vec<&AdapterOp> = trace.ticks.iter().map(|t| &t.adapter[0]).collect();
    assert_eq!(
        duties,
        [0u8, 13, 26]
            .iter()
            .map(|d| AdapterOp::Pwm {
                device_id: f.lamp.raw(),
                duty: *d
            })
            .collect::<Vec<_>>()
            .iter()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        trace.ticks[1].adapter[1],
        AdapterOp::Level {
            device_id: f.coil.raw(),
            high: true
        }
    );
}

#[test]
fn collections_are_refused_for_the_nano_and_a_foreign_family_has_no_entry() {
    let f = fixture("nano_arena");
    let art = compile(&f.s, &options());
    let deployment = analyze_deployment(&f.s, &nano());
    let mut collections = art.collections.clone().unwrap();
    collections.readiness = CollectionsReadiness::Bounded;
    collections.state_bytes_max = Some(64);
    collections.tick_bytes_max = Some(64);
    let err = adapter_plan(
        art.exec_ir.as_ref().unwrap(),
        &nano(),
        &deployment,
        &collections,
        None,
        &TargetOptions::default(),
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["adapter.collections_unsupported"]);
    // the mock board has a family no entry drives
    let big = boards::big_board();
    let art = compile_for_target(&f.s, &big, &TargetOptions::default(), &options());
    assert!(!art.succeeded());
    assert_eq!(codes(&art.diagnostics), ["adapter.target_unsupported"]);
}

#[test]
fn a_line_without_pwm_is_refused_before_the_firmware() {
    let f = fixture("nano_incompatible");
    let art = compile(&f.s, &options());
    let mut deployment = analyze_deployment(&f.s, &nano());
    let lamp_req = deployment
        .requirements
        .iter()
        .find(|r| r.id.device == f.lamp)
        .unwrap()
        .id;
    deployment
        .assignment
        .as_mut()
        .unwrap()
        .insert(lamp_req, ResourceId::new("D4"));
    let err = adapter_plan(
        art.exec_ir.as_ref().unwrap(),
        &nano(),
        &deployment,
        art.collections.as_ref().unwrap(),
        None,
        &TargetOptions::default(),
    )
    .unwrap_err();
    assert_eq!(codes(&err), ["adapter.resource_incompatible"]);
}

/// The AVR toolchain: the pinned nightly with `rust-src`, and `avr-gcc` as
/// the linker.  Absent, the cross-build is skipped — unless
/// `BDL_REQUIRE_AVR` says CI must have it.
fn avr_toolchain_available() -> bool {
    let nightly = std::process::Command::new("rustup")
        .args(["run", "nightly-2025-04-27", "rustc", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let gcc = std::process::Command::new("avr-gcc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    nightly && gcc
}

#[test]
fn the_firmware_cross_compiles_for_the_nano() {
    if !avr_toolchain_available() {
        if std::env::var("BDL_REQUIRE_AVR").is_ok() {
            panic!("the AVR toolchain is missing: `rustup toolchain install nightly-2025-04-27 --component rust-src` and avr-gcc on PATH");
        }
        eprintln!("skipping: nightly-2025-04-27 or avr-gcc not installed");
        return;
    }
    let f = fixture("nano_lamp_fw");
    let art = compile_for_target(&f.s, &nano(), &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("nano_lamp_fw");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let elf = support::cargo_for("nano_lamp_fw")
        .build_firmware_with(
            "arduino_nano",
            "avr-none",
            Some("nightly-2025-04-27"),
            &["-Zbuild-std=core"],
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let bytes = std::fs::read(&elf).unwrap();
    assert!(bytes.starts_with(b"\x7fELF"), "{}", elf.display());
    // ELF e_machine 83 = AVR
    assert_eq!(
        u16::from_le_bytes([bytes[18], bytes[19]]),
        83,
        "not an AVR ELF"
    );
}
