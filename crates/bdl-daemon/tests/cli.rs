//! `bdld check | compile | simulate` over a text project: the headless
//! front end reads the same sources through the same loader as the
//! editors, and its exit code tells a script what it found.

use std::path::Path;
use std::process::Command;

const CONCEPTS: &str = "concept Tilt : Angle\nconcept Brightness : Scalar\nclock interaction\n";
const MAIN: &str = "mapping tilt : Tilt @interaction\nmapping tiltValue : Tilt @interaction\ntiltValue() = tilt\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) = t / (90 deg)\n\nmapping brightness : Brightness @interaction\nbrightness() = dimByTilt(tiltValue)\n\noutput light : Brightness @interaction\ndrive light = brightness\n";

fn project(root: &Path, main: &str) {
    std::fs::create_dir_all(root.join("src")).expect("mkdir");
    std::fs::write(
        root.join("bdl.toml"),
        "schema_version = 1\nname = \"lamp\"\ncompiler_version = \"test\"\nkind = \"text\"\n",
    )
    .expect("manifest");
    std::fs::write(root.join("src/concepts.bdl"), CONCEPTS).expect("write");
    std::fs::write(root.join("src/main.bdl"), main).expect("write");
}

fn bdld(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bdld"))
        .args(args)
        .output()
        .expect("run bdld");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn check_compile_and_simulate_a_text_project() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("lamp");
    project(&root, MAIN);
    let r = root.to_string_lossy().into_owned();

    let (code, out, err) = bdld(&["check", &r]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("checks, outputs complete"), "{out}");
    assert!(!out.contains("error["), "{out}");
    // Opening allocated the identities: the sidecar is there for the
    // next tool.
    assert!(root.join(".bdl/identities.json").is_file());

    let (code, out, _) = bdld(&["check", &r, "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value =
        serde_json::from_str(out.lines().next().expect("line")).expect("json");
    assert!(v["diagnostics"]
        .as_array()
        .is_some_and(|d| d.iter().all(|x| x["severity"] != "error")));

    let (code, out, _) = bdld(&[
        "simulate",
        &r,
        "--ticks",
        "2",
        "--input",
        "tilt=0.7853981633974483",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("tick 1:"), "{out}");
    assert!(
        out.contains("brightness = ") && out.contains("0.5"),
        "{out}"
    );

    let out_dir = dir.path().join("gen");
    let (code, out, err) = bdld(&["compile", &r, "--out", out_dir.to_str().expect("utf8")]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out_dir.join("src/lib.rs").is_file());
    assert!(out_dir.join("Cargo.toml").is_file());
}

#[test]
fn a_fault_in_the_sources_fails_check_with_its_position() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("lamp");
    project(
        &root,
        &MAIN.replace(
            "mapping brightness : Brightness",
            "mapping brightness : Glow",
        ),
    );
    let r = root.to_string_lossy().into_owned();
    let (code, out, _) = bdld(&["check", &r]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("src/main.bdl:8:"), "{out}");
    assert!(out.contains("Glow"), "{out}");

    let (code, _, err) = bdld(&["check", dir.path().join("nowhere").to_str().expect("utf8")]);
    assert_eq!(code, 2);
    assert!(err.starts_with("error:"), "{err}");
}

/// The deployable window in text (docs/spec/deployment-capacity.md):
/// `compile --period` reports what the schedule requires of each
/// cross-domain window, and `--bounded-memory` refuses a collection the
/// design grows without bound instead of generating a core that drops
/// values.
#[test]
fn compile_reports_collection_bounds_and_refuses_unbounded_state_on_bounded_memory() {
    const WINDOW: &str = "\
concept Reading : Scalar
concept Readings : List<Scalar>
concept Count : Scalar
clock fast
clock slow
mapping x : Reading @fast
mapping count : Count @fast
count() = 1 + delay(0, count)
mapping log : Readings @fast
log() = take(3, cons(x, delay([], log)))
mapping logD : Readings @slow
logD() = sync(fast, [], log)
mapping seen : Count @slow
seen() = sync(fast, 0, count)
mapping cursor : Count @slow
cursor() = delay(0, seen)
mapping window : Readings @slow
window() = reverse(take(seen - cursor, logD))
";
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("window");
    std::fs::create_dir_all(root.join("src")).expect("mkdir");
    std::fs::write(
        root.join("bdl.toml"),
        "schema_version = 1\nname = \"window\"\ncompiler_version = \"test\"\nkind = \"text\"\n",
    )
    .expect("manifest");
    std::fs::write(root.join("src/main.bdl"), WINDOW).expect("write");
    let r = root.to_string_lossy().into_owned();
    let out_dir = dir.path().join("gen");
    let o = out_dir.to_str().expect("utf8");

    // sufficient: the design keeps 3, slow sees at most 3 per activation
    let (code, out, err) = bdld(&[
        "compile", &r, "--out", o, "--period", "fast=1", "--period", "slow=3",
    ]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains("collections: bounded by the design"), "{out}");
    assert!(
        out.contains(
            "window fast → slow: up to 3 value(s) between two activations of slow; logD keeps 3"
        ),
        "{out}"
    );
    assert!(!out.contains("warning["), "{out}");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out_dir.join("bdl-manifest.json")).expect("manifest"),
    )
    .expect("json");
    assert_eq!(manifest["collections"]["unbounded"], false);
    assert!(manifest["collections"]["state_bytes_max"]
        .as_u64()
        .is_some());

    // insufficient: said, not silent
    let (code, out, _) = bdld(&[
        "compile", &r, "--out", o, "--period", "fast=1", "--period", "slow=5",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("warning[deployment.window_capacity] mapping logD: Between two activations of slow, fast produces up to 5 values, but logD keeps 3."),
        "{out}"
    );

    // an unbounded log: a host warns, a bounded-memory target refuses
    let grown =
        format!("{WINDOW}mapping grown : Readings @fast\ngrown() = cons(x, delay([], grown))\n");
    std::fs::write(root.join("src/main.bdl"), grown).expect("write");
    let (code, out, _) = bdld(&["compile", &r, "--out", o]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("warning[deployment.unbounded_list_state] mapping grown: grown keeps every value it has ever received."),
        "{out}"
    );
    let (code, out, _) = bdld(&["compile", &r, "--out", o, "--bounded-memory"]);
    assert_ne!(code, 0, "{out}");
    assert!(
        out.contains("error[deployment.unbounded_list_state]"),
        "{out}"
    );
    assert!(out.contains("This target has finite memory"), "{out}");
    // a period for an unknown domain is refused up front
    let (code, _, err) = bdld(&["compile", &r, "--out", o, "--period", "nope=2"]);
    assert_ne!(code, 0);
    assert!(err.contains("no timing domain named `nope`"), "{err}");
}

/// `bdld migrate-unit-domain`: the legacy `mapping f : A` becomes
/// `mapping f : () -> A` everywhere, and nothing else — comments, blank
/// lines, definitions, the explicit form — while the identities the
/// project already allocated stay what they were and the design is the
/// same design; `--dry-run` writes nothing; a second run has nothing to do.
#[test]
fn migrate_unit_domain_rewrites_only_the_legacy_signatures_and_keeps_identities() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("lamp");
    let main = "// the lamp\nmapping tilt : Tilt @interaction   // a source\nmapping tiltValue : Tilt @interaction\ntiltValue() = tilt\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) = t / (90 deg)\n\nmapping brightness : () -> Brightness @interaction\nbrightness() = dimByTilt(tiltValue)\n\noutput light : Brightness @interaction\ndrive light = brightness\n";
    project(&root, main);
    let r = root.to_string_lossy().into_owned();
    // open once so the identities exist
    let (code, _, err) = bdld(&["check", &r]);
    assert_eq!(code, 0, "{err}");
    let ids_before = std::fs::read_to_string(root.join(".bdl/identities.json")).expect("ids");

    let (code, out, err) = bdld(&["migrate-unit-domain", &r, "--dry-run"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        out.contains("2 signature(s) in 1 file(s) would be rewritten"),
        "{out}"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.bdl")).expect("main"),
        main
    );

    let (code, out, err) = bdld(&["migrate-unit-domain", &r, "--json"]);
    assert_eq!(code, 0, "{out}{err}");
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["signatures"], 2);
    assert_eq!(v["files"][0]["path"], "src/main.bdl");
    let migrated = std::fs::read_to_string(root.join("src/main.bdl")).expect("main");
    assert_eq!(
        migrated,
        "// the lamp\nmapping tilt : () -> Tilt @interaction   // a source\nmapping tiltValue : () -> Tilt @interaction\ntiltValue() = tilt\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) = t / (90 deg)\n\nmapping brightness : () -> Brightness @interaction\nbrightness() = dimByTilt(tiltValue)\n\noutput light : Brightness @interaction\ndrive light = brightness\n"
    );
    // concepts.bdl has no mapping: untouched
    assert_eq!(
        std::fs::read_to_string(root.join("src/concepts.bdl")).expect("concepts"),
        CONCEPTS
    );
    // the same identities, the same design
    let (code, out, _) = bdld(&["check", &r]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("checks, outputs complete"), "{out}");
    let ids_after = std::fs::read_to_string(root.join(".bdl/identities.json")).expect("ids");
    assert_eq!(ids_before, ids_after);
    let (code, out, _) = bdld(&["migrate-unit-domain", &r]);
    assert_eq!(code, 0);
    assert!(out.contains("nothing to migrate"), "{out}");
}

/// `compile --target rp2040_pico` writes the firmware beside the core from
/// the placement; a design with a Source, or an unknown board, is refused
/// before anything is written.
#[test]
fn compile_for_a_board_writes_the_adapter_from_the_placement() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("lamp");
    // no Source: a level that climbs; a PWM device realising the output
    let main = "mapping level : () -> Brightness @interaction\nlevel() = delay(0, level + 5)\n\noutput light : Brightness @interaction\ndrive light = level\n\ndevice lamp : pwm_channel for light { realization pwm_duty8 }\n";
    project(&root, main);
    let r = root.to_string_lossy().into_owned();
    let out_dir = dir.path().join("gen");
    let o = out_dir.to_str().expect("utf8");
    let (code, out, err) = bdld(&[
        "compile",
        &r,
        "--out",
        o,
        "--target",
        "rp2040_pico",
        "--tick-micros",
        "20000",
    ]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out_dir.join("src/bin/rp2040.rs").is_file());
    assert!(out_dir.join("src/adapter.rs").is_file());
    assert!(out_dir.join("memory.x").is_file());
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_dir.join("bdl-manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["adapter"]["board"], "rp2040_pico");
    assert_eq!(manifest["adapter"]["tick_micros"], 20000);
    assert_eq!(manifest["adapter"]["bindings"][0]["capability"], "pwm");
    assert_eq!(manifest["adapter"]["bindings"][0]["resource"], "GP0");

    let (code, _, err) = bdld(&["compile", &r, "--out", o, "--target", "toaster"]);
    assert_eq!(code, 2);
    assert!(err.contains("no target named `toaster`"), "{err}");

    // a Source has no device to provide it: refused with the reason
    let root2 = dir.path().join("lamp2");
    project(
        &root2,
        &format!("{MAIN}\ndevice lamp : pwm_channel for light {{ realization pwm_duty8 }}\n"),
    );
    let r = root2.to_string_lossy().into_owned();
    let (code, out, _) = bdld(&["compile", &r, "--out", o, "--target", "rp2040_pico"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("adapter.inputs_unbound"), "{out}");
    assert!(out.contains("No device provides tilt"), "{out}");
    // and without a device at all the placement is not finished
    let root3 = dir.path().join("lamp3");
    project(&root3, MAIN);
    let r = root3.to_string_lossy().into_owned();
    let (code, out, _) = bdld(&["compile", &r, "--out", o, "--target", "rp2040_pico"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("adapter.deployment_not_feasible"), "{out}");
}
