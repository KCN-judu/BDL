//! Rough timings for the IDE queries on representative designs, to decide
//! whether finer incremental caching is needed (`docs/IDE_SERVICE_ARCHITECTURE.md`
//! §Performance baseline).  Run with `cargo run --release -p bdl-ide
//! --example perf_baseline`.

use bdl_ide::*;
use bdl_ide_db::textual::render_module;
use bdl_ide_db::DocumentUri;
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, Dim, SemanticId};
use std::time::{Duration, Instant};

fn design(concepts: usize, mappings: usize, outputs: usize) -> (ProjectSnapshot, DeclId) {
    let mut s = ProjectSnapshot::new(Design::empty("perf"));
    let mut ids: Vec<SemanticId> = Vec::new();
    let dims = [Dim::ANGLE, Dim::ZERO, Dim::TIME, Dim::LENGTH];
    for i in 0..concepts {
        let a = apply_edit(
            &s,
            &EditOp::CreateConcept {
                name: format!("Concept{i}"),
                description: String::new(),
                representation: Some(Representation::Quantity {
                    dim: dims[i % dims.len()],
                }),
            },
        )
        .expect("concept");
        ids.push(a.outcome.created_concept.expect("id"));
        s = a.snapshot;
    }
    let clock = {
        let a = apply_edit(
            &s,
            &EditOp::CreateClockDomain {
                name: "main".into(),
            },
        )
        .expect("clock");
        s = a.snapshot;
        a.outcome.created_clock.expect("clock")
    };
    let mut first = None;
    for i in 0..mappings {
        let input = ids[i % ids.len()];
        let output = ids[(i * 7 + 1) % ids.len()];
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: format!("mapping{i}"),
                description: String::new(),
                signature: Signature {
                    inputs: vec![input],
                    output,
                },
            },
        )
        .expect("mapping");
        let id = a.outcome.created_mapping.expect("id");
        first.get_or_insert(id);
        s = a.snapshot;
        // Half the mappings get a (deliberately dimension-wrong or right)
        // formula so the checker has work to do.
        if i % 2 == 0 {
            s = apply_edit(
                &s,
                &EditOp::AttachDefinition {
                    id,
                    definition: Definition::Formula {
                        source: format!("Concept{} * 2", i % ids.len()),
                    },
                },
            )
            .expect("formula")
            .snapshot;
        }
    }
    for i in 0..outputs {
        let a = apply_edit(
            &s,
            &EditOp::CreateOutput {
                name: format!("sink{i}"),
                description: String::new(),
                accepts: ids[(i * 3) % ids.len()],
                clock: Some(clock),
            },
        )
        .expect("output");
        s = a.snapshot;
    }
    (s, first.expect("at least one mapping"))
}

fn time<T>(label: &str, runs: u32, mut f: impl FnMut() -> T) -> T {
    let mut best = Duration::MAX;
    let mut out = None;
    for _ in 0..runs {
        let t = Instant::now();
        out = Some(f());
        best = best.min(t.elapsed());
    }
    println!(
        "  {label:<44} {:>9.3} ms (best of {runs})",
        best.as_secs_f64() * 1e3
    );
    out.expect("ran")
}

fn bench(name: &str, concepts: usize, mappings: usize, outputs: usize) {
    let (s, m) = design(concepts, mappings, outputs);
    println!("{name}: {concepts} concepts, {mappings} mappings, {outputs} outputs");
    let mut host = IdeHost::new(s.clone());
    time("committed snapshot (compile + index)", 5, || {
        host.set_committed(s.clone());
        host.snapshot()
    });
    time("formula overlay → snapshot + draft verdict", 5, || {
        host.set_definition_draft(m, "Concept0 / (90 deg)");
        let snap = host.snapshot();
        draft_verdict(&snap, m).map(|v| v.status).ok()
    });
    let snap = host.snapshot();
    time("diagnostics (project)", 5, || {
        diagnostics(&snap, DiagnosticScope::Project).items.len()
    });
    time("completion (formula)", 20, || {
        completion(
            &snap,
            &CompletionContext::Formula {
                mapping: m,
                offset: 3,
            },
        )
        .len()
    });
    time("hover (mapping)", 20, || {
        hover(&snap, EntityRef::Mapping(m)).is_some()
    });
    let concept = s.design.concepts.keys().next().copied().expect("concept");
    time("references (concept)", 20, || {
        references(&snap, EntityRef::Concept(concept))
            .references
            .len()
    });
    let _ = time("rename plan (concept)", 20, || {
        plan_rename(&snap, EntityRef::Concept(concept), "Renamed").map(|p| p.operations.len())
    });
    let uri = DocumentUri::new("file:///perf.bdl");
    let text = render_module(&s.design);
    time("text overlay → snapshot (bind + compile)", 5, || {
        host.set_text_document(&uri, text.clone());
        host.snapshot()
    });
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("doc");
    time("document diagnostics (placed)", 5, || {
        diagnostics(&snap, DiagnosticScope::Document(doc))
            .items
            .iter()
            .filter_map(|d| project_to_document(&snap, d, doc))
            .count()
    });
    time("semantic tokens", 5, || semantic_tokens(&snap, doc).len());
    println!();
}

fn main() {
    bench("small (lamp-sized)", 3, 3, 1);
    bench("medium", 60, 80, 10);
    bench("large", 300, 400, 40);
}
