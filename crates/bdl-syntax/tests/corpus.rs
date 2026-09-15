//! Golden corpus: every `test_data/{valid,invalid}/*.bdl` must round-trip
//! losslessly and produce exactly the tree and diagnostics recorded in the
//! sibling `.snap` file.  Regenerate with `UPDATE_GOLDEN=1 cargo test -p
//! bdl-syntax --test corpus`.

use bdl_syntax::syntax::debug_tree;
use bdl_syntax::{lower_module, parse_module};
use std::path::{Path, PathBuf};

fn corpus(dir: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_data")
        .join(dir);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("{}: {e}", root.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "bdl"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "no corpus in {}", root.display());
    files
}

fn render(src: &str) -> String {
    let parse = parse_module(src);
    assert_eq!(parse.text(), src, "lossless");
    let mut out = debug_tree(&parse.syntax_node());
    for e in parse.errors() {
        out.push_str(&format!("error {e}\n  {}\n", e.technical()));
        if let Some(h) = &e.hint {
            out.push_str(&format!("  hint: {h}\n"));
        }
    }
    let (module, lower_errors) = lower_module(&parse);
    out.push_str(&format!(
        "lowered: {} concept(s), {} mapping(s), {} enum(s)\n",
        module.concepts().count(),
        module.mappings().count(),
        module.enums().count()
    ));
    for e in lower_errors.iter().filter(|e| !parse.errors().contains(e)) {
        out.push_str(&format!("lower error {e}\n"));
    }
    out
}

fn check(dir: &str, expect_errors: bool) {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut failures = Vec::new();
    for path in corpus(dir) {
        let src = std::fs::read_to_string(&path).expect("readable");
        let parse = parse_module(&src);
        assert_eq!(
            !parse.errors().is_empty(),
            expect_errors,
            "{}: errors {:?}",
            path.display(),
            parse.errors()
        );
        let actual = render(&src);
        let snap = path.with_extension("snap");
        if update {
            std::fs::write(&snap, &actual).expect("write snapshot");
            continue;
        }
        let expected = std::fs::read_to_string(&snap).unwrap_or_default();
        if expected != actual {
            failures.push(path.display().to_string());
            eprintln!(
                "--- {} differs from {} ---\n{actual}",
                path.display(),
                snap.display()
            );
        }
    }
    assert!(
        failures.is_empty(),
        "golden mismatch: {failures:?} (UPDATE_GOLDEN=1 to accept)"
    );
}

#[test]
fn valid_corpus() {
    check("valid", false);
}

#[test]
fn invalid_corpus() {
    check("invalid", true);
}

#[test]
fn every_corpus_file_is_deterministic() {
    for dir in ["valid", "invalid"] {
        for path in corpus(dir) {
            let src = std::fs::read_to_string(&path).expect("readable");
            assert_eq!(render(&src), render(&src), "{}", path.display());
        }
    }
}
