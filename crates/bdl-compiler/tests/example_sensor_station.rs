#![allow(clippy::unwrap_used)]
//! The Sensor Station: the demonstration under `examples/sensor_station`
//! — three analog Sources (a 10 kΩ thermistor, a photoresistor and a trim
//! potentiometer on A1, A0, A2 of an Arduino Nano) read through the
//! catalogue's curves, three outputs (a gamma-corrected night light and a
//! fan on PWM, an overheat lamp on D13).  Checked from the text on disk:
//! the ladder, the deployment on the Nano, the firmware core run on the
//! host over ADC counts a real divider would give at known temperatures
//! and illuminances, and — where the AVR toolchain is installed — the
//! cross-build to an AVR ELF.

mod support;

use bdl_compiler::{analyze, analyze_deployment, compile_for_target, DeploymentStatus};
use bdl_compiler::{CompileOptions, TargetOptions};
use bdl_hardware::boards;
use bdl_model::surface::ProjectSnapshot;
use bdl_runtime_host::{AdapterOp, DynValue, RunRequest, TickRequest};
use std::path::{Path, PathBuf};

fn example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/sensor_station")
}

fn load() -> ProjectSnapshot {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&example_dir(), dir.path());
    let loaded = bdl_text::load_project(dir.path()).unwrap();
    assert!(loaded.migrated.is_none(), "the example is already unified");
    let mut design = loaded.build.system.base.clone();
    for m in design.mappings.values_mut() {
        m.parameters.clear();
    }
    ProjectSnapshot::new(design)
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn options() -> CompileOptions {
    CompileOptions {
        require_complete: true,
        ..support::options()
    }
}

// What the divider puts on the pin, from the part's own physics (not from
// the catalogue's table): the 10-bit count of a 5 V reference.

/// 10 kΩ NTC, β 3950, to ground under a 10 kΩ pull-up.
fn ntc_counts(celsius: f64) -> f64 {
    let t = celsius + 273.15;
    let r = 10_000.0 * (3950.0 * (1.0 / t - 1.0 / 298.15)).exp();
    (1023.0 * r / (r + 10_000.0)).round()
}

/// GL5528 (15 kΩ at 10 lx, γ 0.7) from 5 V over 10 kΩ to ground.
fn ldr_counts(lux: f64) -> f64 {
    let r = 15_000.0 * (lux / 10.0).powf(-0.7);
    (1023.0 * 10_000.0 / (r + 10_000.0)).round()
}

/// A potentiometer between 5 V and ground.
fn knob_counts(percent: f64) -> f64 {
    (percent * 10.23).round()
}

#[test]
fn the_station_is_complete_and_feasible_on_the_nano_on_the_pins_the_file_names() {
    let s = load();
    let a = analyze(&s);
    assert!(a.causality.valid && a.clocks.valid && a.output_complete);
    let nano = boards::arduino_nano();
    let d = analyze_deployment(&s, &nano);
    assert_eq!(d.status, DeploymentStatus::Feasible, "{:?}", d.diagnostics);
    assert!(d.unprovided_sources.is_empty());
    let art = compile_for_target(&s, &nano, &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let adapter = g.manifest.adapter.as_ref().unwrap();
    let mut sinks: Vec<&str> = adapter
        .bindings
        .iter()
        .map(|b| b.resource.as_str())
        .collect();
    sinks.sort();
    assert_eq!(sinks, ["D10", "D13", "D9"]);
    let sources: Vec<(&str, &str, &str)> = adapter
        .sources
        .iter()
        .map(|s| (s.profile.as_str(), s.resource.as_str(), s.kind.as_str()))
        .collect();
    assert_eq!(
        sources,
        [
            ("adc_ntc10k_b3950", "A1", "adc"),
            ("adc_ldr_gl5528", "A0", "adc"),
            ("adc_percent", "A2", "adc"),
        ]
    );
    let fw = &g.files["src/bin/arduino_nano.rs"];
    assert!(fw.contains("arduino_hal::Adc::new") && fw.contains("read_blocking"));
    // a board whose firmware has no ADC reader refuses the analog profiles by name
    let pico = compile_for_target(
        &s,
        &boards::rp2040_pico(),
        &TargetOptions::default(),
        &options(),
    );
    assert!(!pico.succeeded());
}

fn reading(counts: f64) -> Option<DynValue> {
    Some(DynValue::Quantity { value: counts })
}

#[test]
fn the_core_follows_a_room_through_four_conditions() {
    let s = load();
    let nano = boards::arduino_nano();
    let art = compile_for_target(&s, &nano, &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let dir = support::generated_dir("sensor_station_host");
    bdl_runtime_host::harness::write_crate(&dir, &art.generated.as_ref().unwrap().files).unwrap();
    let cargo = support::cargo_for("sensor_station_host");
    cargo.check_core().unwrap_or_else(|e| panic!("{e}"));

    // (°C, lux, knob %), each held 300 ticks (3 s at the base tick)
    let phases: [(f64, f64, f64); 4] = [
        (25.0, 1.0, 0.0), // warm and dark: setpoint 20 °C, the fan half on, the light full
        (45.0, 1000.0, 0.0), // hot and bright: the fan full, the light off, overheat
        (30.0, 1000.0, 0.0), // 10 K over: overheat holds (it clears only below 8 K)
        (26.0, 1000.0, 0.0), // 6 K over: overheat clears, the fan at 60 %
    ];
    let mut ticks = Vec::new();
    for (celsius, lux, knob) in phases {
        for _ in 0..300 {
            ticks.push(TickRequest {
                active: vec![0],
                inputs: vec![],
                readings: vec![
                    reading(ntc_counts(celsius)),
                    reading(ldr_counts(lux)),
                    reading(knob_counts(knob)),
                ],
            });
        }
    }
    let trace = cargo
        .run_host(&RunRequest { ticks })
        .unwrap_or_else(|e| panic!("{e}"));
    let at = |phase: usize| {
        let ops = &trace.ticks[phase * 300 + 299].adapter;
        let duty = |i: usize| match &ops[i] {
            AdapterOp::Pwm { duty, .. } => i32::from(*duty),
            other => panic!("not a duty: {other:?}"),
        };
        let alarm = match &ops[2] {
            AdapterOp::Level { high, .. } => *high,
            other => panic!("not a level: {other:?}"),
        };
        (duty(0), duty(1), alarm)
    };

    let (light, fan, alarm) = at(0);
    assert_eq!(light, 255, "dark: the night light is full");
    assert!(
        (105..=150).contains(&fan),
        "5 K over: about half, got {fan}"
    );
    assert!(!alarm);

    let (light, fan, alarm) = at(1);
    assert_eq!(light, 0, "bright: the night light has faded out");
    assert_eq!(fan, 255, "25 K over: the fan is full");
    assert!(alarm);

    let (_, fan, alarm) = at(2);
    assert!(fan >= 225, "10 K over: the fan is (nearly) full, got {fan}");
    assert!(alarm, "overheat holds between its thresholds");

    let (_, fan, alarm) = at(3);
    assert!(
        (125..=180).contains(&fan),
        "6 K over: about 60 %, got {fan}"
    );
    assert!(!alarm, "overheat cleared below eight kelvin");
}

#[test]
fn a_disconnected_or_shorted_sensor_saturates_and_never_faults_the_core() {
    let s = load();
    let nano = boards::arduino_nano();
    let art = compile_for_target(&s, &nano, &TargetOptions::default(), &options());
    let dir = support::generated_dir("sensor_station_fault");
    bdl_runtime_host::harness::write_crate(&dir, &art.generated.as_ref().unwrap().files).unwrap();
    let cargo = support::cargo_for("sensor_station_fault");
    // the extremes of the 10-bit range on every pin
    let ticks: Vec<TickRequest> = [0.0, 1023.0, 0.0]
        .iter()
        .flat_map(|c| {
            (0..5).map(move |_| TickRequest {
                active: vec![0],
                inputs: vec![],
                readings: vec![reading(*c), reading(*c), reading(*c)],
            })
        })
        .collect();
    let trace = cargo
        .run_host(&RunRequest { ticks })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(trace.ticks.len(), 15);
}

/// The AVR toolchain, as `embedded_arduino_nano.rs` asks for it: absent,
/// the cross-build is skipped unless `BDL_REQUIRE_AVR` says CI must have it.
fn avr_toolchain_available() -> bool {
    let ok = |cmd: &str, args: &[&str]| {
        std::process::Command::new(cmd)
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    ok(
        "rustup",
        &["run", "nightly-2025-04-27", "rustc", "--version"],
    ) && ok("avr-gcc", &["--version"])
}

#[test]
fn the_station_cross_compiles_to_an_avr_image() {
    if !avr_toolchain_available() {
        if std::env::var("BDL_REQUIRE_AVR").is_ok() {
            panic!("the AVR toolchain is missing: `rustup toolchain install nightly-2025-04-27 --component rust-src` and avr-gcc on PATH");
        }
        eprintln!("skipping: nightly-2025-04-27 or avr-gcc not installed");
        return;
    }
    let art = compile_for_target(
        &load(),
        &boards::arduino_nano(),
        &TargetOptions::default(),
        &options(),
    );
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let dir = support::generated_dir("sensor_station_fw");
    bdl_runtime_host::harness::write_crate(&dir, &art.generated.as_ref().unwrap().files).unwrap();
    let elf = support::cargo_for("sensor_station_fw")
        .build_firmware_with(
            "arduino_nano",
            "avr-none",
            Some("nightly-2025-04-27"),
            &["-Zbuild-std=core"],
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let bytes = std::fs::read(&elf).unwrap();
    assert!(bytes.starts_with(b"\x7fELF"));
    // ELF e_machine 83 = AVR
    assert_eq!(u16::from_le_bytes([bytes[18], bytes[19]]), 83);
}
