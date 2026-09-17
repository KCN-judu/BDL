//! Generated output is deterministic and reviewed: every corpus case's
//! core, host bridge and manifest are compared with the checked-in golden
//! files (`BDL_UPDATE_GOLDEN=1` rewrites them), and generating twice gives
//! identical bytes.

#![allow(clippy::unwrap_used)]

mod support;

use std::path::PathBuf;
use support::*;

fn golden_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name)
}

#[test]
fn generated_files_match_golden_and_are_deterministic() {
    let update = std::env::var("BDL_UPDATE_GOLDEN").is_ok();
    // Golden files use the portable default runtime paths.
    let opts = bdl_compiler::CompileOptions::default();
    for case in corpus() {
        let a = bdl_compiler::compile_design_ir(case.ir.clone(), case.name, &opts);
        let b = bdl_compiler::compile_design_ir(case.ir.clone(), case.name, &opts);
        assert!(a.succeeded(), "{}: {:?}", case.name, a.diagnostics);
        assert_eq!(a.generated, b.generated, "{}: not deterministic", case.name);
        assert_eq!(a.exec_ir, b.exec_ir);
        let g = a.generated.unwrap();
        for (rel, contents) in &g.files {
            let path = golden_dir(case.name).join(rel);
            if update {
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, contents).unwrap();
                continue;
            }
            let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!("{}: {e} (BDL_UPDATE_GOLDEN=1 to create)", path.display())
            });
            assert_eq!(
                &on_disk,
                contents,
                "{} differs from golden {} (BDL_UPDATE_GOLDEN=1 to accept)",
                case.name,
                path.display()
            );
        }
    }
}

#[test]
fn manifest_maps_every_generated_entity_back_to_its_id() {
    let case = corpus().into_iter().find(|c| c.name == "sync").unwrap();
    let art = compile_case(&case);
    let g = art.generated.unwrap();
    let m = &g.manifest;
    assert_eq!(m.manifest_version, 1);
    assert_eq!(
        m.clocks
            .iter()
            .map(|c| (c.slot, c.clock_id, c.name.as_str()))
            .collect::<Vec<_>>(),
        vec![(0, 0, "fast"), (1, 1, "slow")]
    );
    assert_eq!(m.inputs[0].decl_id, 0);
    assert_eq!(m.cells.len(), 2);
    assert_eq!(
        (
            m.cells[0].decl_id,
            m.cells[0].path.clone(),
            m.cells[0].writer_clock_slot
        ),
        (1, vec![], 0)
    );
    assert_eq!(
        (
            m.cells[1].decl_id,
            m.cells[1].path.clone(),
            m.cells[1].writer_clock_slot
        ),
        (2, vec![], 1)
    );
    assert_eq!(
        m.decls
            .iter()
            .map(|d| d.symbol.as_str())
            .collect::<Vec<_>>(),
        vec!["decl_0", "decl_1", "decl_2"]
    );
    let json: serde_json::Value = serde_json::from_str(&g.files["bdl-manifest.json"]).unwrap();
    assert_eq!(json["manifest_version"], 1);
    // symbols never come from display names
    assert!(!g.core_source().contains("fn fast") && g.core_source().contains("CLOCK_0"));
}

#[test]
fn baseline_metrics_are_recorded() {
    let mut rows = Vec::new();
    for case in corpus() {
        let art = compile_case(&case);
        let g = art.generated.unwrap();
        rows.push((case.name, g.core_lines(), g.total_bytes()));
    }
    // Printed for the record (docs/architecture/codegen-rust.md keeps a snapshot).
    for (n, l, b) in &rows {
        eprintln!("{n}: core {l} lines, crate {b} bytes");
    }
    assert!(rows.iter().all(|(_, l, _)| *l > 20));
}
