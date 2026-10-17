#![allow(clippy::unwrap_used)]
//! The Breathing Beacon: the analog-dynamics demonstration under
//! `examples/breathing_beacon` — a self-driving design (no Source) whose
//! state lives in `delay` cells: a triangle wave, a gamma curve, a
//! first-order lag and a Schmitt trigger, out to two PWM pins and a
//! digital pin of the Arduino Nano.  Checked here from the text on disk:
//! the ladder, the reference simulation over one 400-tick breath, the
//! deployment (`Feasible`, the pins the file names), the firmware core
//! run on the host (the same duties the glue would write) and — where the
//! AVR toolchain is installed — the cross-build to an AVR ELF.

mod support;

use bdl_compiler::{analyze, analyze_deployment, compile_for_target, DeploymentStatus};
use bdl_compiler::{CompileOptions, TargetOptions};
use bdl_hardware::boards;
use bdl_model::surface::ProjectSnapshot;
use bdl_reactive::{InputTrace, Schedule, Simulation, Value};
use bdl_runtime_host::{AdapterOp, RunRequest, TickRequest};
use std::path::{Path, PathBuf};

/// One full breath: 200 ticks up, a tick at the top, 200 down (0.5 % per tick).
const BREATH: usize = 402;

fn example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/breathing_beacon")
}

/// The project opens read-write (identities, layout); open a copy.
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

fn number(v: &Value) -> f64 {
    match v {
        Value::Semantic { repr, .. } => number(repr),
        Value::Quantity { value, .. } => *value,
        Value::Bool { value } => f64::from(u8::from(*value)),
        other => panic!("not a number: {other:?}"),
    }
}

/// The reference evaluator's series for the relationship called `name`.
fn series(snapshot: &ProjectSnapshot, ticks: usize, name: &str) -> Vec<f64> {
    let a = analyze(snapshot);
    let id = snapshot
        .design
        .mappings
        .values()
        .find(|m| m.name == name)
        .unwrap()
        .id;
    let mut sim = Simulation::new(
        a.ir.clone(),
        &a.causality,
        Schedule::always(&a.ir),
        InputTrace::default(),
    )
    .unwrap();
    sim.run(ticks as u64)
        .unwrap()
        .series(id)
        .iter()
        .map(|(_, v)| number(v))
        .collect()
}

fn options() -> CompileOptions {
    CompileOptions {
        require_complete: true,
        ..support::options()
    }
}

#[test]
fn the_beacon_is_complete_and_its_breath_is_a_triangle_with_a_lagging_tail() {
    let s = load();
    let a = analyze(&s);
    assert!(
        a.causality.valid && a.clocks.valid && a.output_complete,
        "{:?}",
        a.mappings
            .values()
            .map(|m| &m.diagnostics)
            .collect::<Vec<_>>()
    );

    let breath = series(&s, BREATH, "breath");
    let ring = series(&s, BREATH, "ring");
    let glow = series(&s, BREATH, "afterglow");
    let alert = series(&s, BREATH, "alert");

    // a triangle in percent: dark at the start, full at tick 200, a tick
    // of plateau at the top, dark again at tick 401 — one slope each way,
    // never leaving [0, 100]
    assert_eq!(breath[0], 0.0);
    assert_eq!(breath[200], 100.0);
    assert_eq!(breath[201], 100.0);
    assert_eq!(breath[BREATH - 1], 0.0);
    assert!(breath[..=200].windows(2).all(|w| w[1] > w[0]));
    assert!(breath[201..BREATH].windows(2).all(|w| w[1] < w[0]));
    assert!(breath.iter().all(|b| (0.0..=100.0).contains(b)));

    // the gamma curve is the breath squared, so never above it
    for (b, r) in breath.iter().zip(&ring) {
        assert!((r - b * b / 100.0).abs() < 1e-9);
    }

    // the tail lags the ring: below it on the way up, above it on the
    // way down, and it still glows after the ring is back near dark
    assert!(glow[100] < ring[100] && glow[300] > ring[300]);
    assert!(glow[BREATH - 1] > ring[BREATH - 1]);

    // the Schmitt trigger: on once the tail passes 0.6, off only when it
    // falls under 0.2 — one pulse per breath, not a flicker at either edge
    let edges: Vec<usize> = (1..alert.len())
        .filter(|i| alert[*i] != alert[i - 1])
        .collect();
    assert_eq!(edges.len(), 2, "{edges:?}");
    // (the trigger reads the tail of the tick before: it is a `delay`)
    let (on, off) = (edges[0], edges[1]);
    assert!(glow[on - 2] <= 60.0 && glow[on - 1] > 60.0 && alert[on] == 1.0);
    assert!(glow[off - 2] > 20.0 && glow[off - 1] <= 20.0 && alert[off] == 0.0);
}

#[test]
fn it_is_feasible_on_the_nano_on_the_pins_the_file_names() {
    let s = load();
    let nano = boards::arduino_nano();
    let d = analyze_deployment(&s, &nano);
    assert_eq!(d.status, DeploymentStatus::Feasible, "{:?}", d.diagnostics);
    assert!(d.unprovided_sources.is_empty());
    let art = compile_for_target(&s, &nano, &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let adapter = g.manifest.adapter.as_ref().unwrap();
    let mut pins: Vec<&str> = adapter
        .bindings
        .iter()
        .map(|b| b.resource.as_str())
        .collect();
    pins.sort();
    assert_eq!(pins, ["D10", "D13", "D9"]);
    assert!(g.files.contains_key("src/bin/arduino_nano.rs"));
}

#[test]
fn the_firmware_core_drives_the_pins_through_a_whole_breath() {
    let s = load();
    let nano = boards::arduino_nano();
    let art = compile_for_target(&s, &nano, &TargetOptions::default(), &options());
    assert!(art.succeeded(), "{:?}", art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = support::generated_dir("breathing_beacon_host");
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let cargo = support::cargo_for("breathing_beacon_host");
    cargo.check_core().unwrap_or_else(|e| panic!("{e}"));
    let trace = cargo
        .run_host(&RunRequest {
            ticks: (0..BREATH)
                .map(|_| TickRequest {
                    active: vec![0],
                    inputs: vec![],
                    readings: vec![],
                })
                .collect(),
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let pwm_of = |slot: usize| -> Vec<u8> {
        trace
            .ticks
            .iter()
            .map(|t| match &t.adapter[slot] {
                AdapterOp::Pwm { duty, .. } => *duty,
                other => panic!("not a duty: {other:?}"),
            })
            .collect()
    };
    let ring = pwm_of(0);
    let glow = pwm_of(1);
    assert_eq!(ring[0], 0);
    assert_eq!(ring[200], 255, "the ring reaches full duty");
    assert!(ring[100] > 0 && ring[100] < 255 && ring[100] < ring[150]);
    assert_eq!(ring[BREATH - 1], 0);
    assert!(glow[BREATH - 1] > ring[BREATH - 1], "the tail still glows");
    // the digital pin follows the Schmitt trigger: high at the top of the
    // breath, low at both ends
    let level = |i: usize| {
        matches!(
            trace.ticks[i].adapter[2],
            AdapterOp::Level { high: true, .. }
        )
    };
    assert!(!level(0) && level(200) && !level(BREATH - 1));
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
fn the_beacon_cross_compiles_to_an_avr_image() {
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
    let dir = support::generated_dir("breathing_beacon_fw");
    bdl_runtime_host::harness::write_crate(&dir, &art.generated.as_ref().unwrap().files).unwrap();
    let elf = support::cargo_for("breathing_beacon_fw")
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
