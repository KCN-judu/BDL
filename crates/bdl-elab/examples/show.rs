//! `cargo run -p bdl-elab --example show -- '<formula>'`: elaborate a
//! formula over a fixed lamp design and print the Core term / diagnostics.
#![allow(clippy::unwrap_used)]
use bdl_elab::elaborate_design;
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::Dim;

fn main() {
    let src = std::env::args().nth(1).unwrap_or_default();
    let nullary = std::env::args().nth(2).is_some();
    let mut s = ProjectSnapshot::new(Design::empty("lamp"));
    let mut ids = Vec::new();
    for (name, rep) in [
        ("Tilt", Some(Representation::Quantity { dim: Dim::ANGLE })),
        (
            "Brightness",
            Some(Representation::Quantity { dim: Dim::ZERO }),
        ),
        ("Held", Some(Representation::Boolean)),
        ("Level", Some(Representation::Quantity { dim: Dim::ZERO })),
    ] {
        let a = apply_edit(
            &s,
            &EditOp::CreateConcept {
                name: name.into(),
                description: String::new(),
                representation: rep,
            },
        )
        .unwrap();
        ids.push(a.outcome.created_concept.unwrap());
        s = a.snapshot;
    }
    let a = apply_edit(
        &s,
        &EditOp::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![ids[0]],
                output: ids[1],
            },
            definition: None,
            clock: None,
        },
    )
    .unwrap();
    let dim = a.outcome.created_mapping.unwrap();
    s = a.snapshot;
    s = apply_edit(
        &s,
        &EditOp::AttachDefinition {
            id: dim,
            definition: Definition::Formula {
                source: "Tilt / 90 deg".into(),
            },
        },
    )
    .unwrap()
    .snapshot;
    let a = apply_edit(
        &s,
        &EditOp::CreateMapping {
            name: "level".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![],
                output: ids[3],
            },
            definition: None,
            clock: None,
        },
    )
    .unwrap();
    s = a.snapshot;
    let a = apply_edit(
        &s,
        &EditOp::CreateMapping {
            name: "f".into(),
            description: String::new(),
            signature: Signature {
                inputs: if nullary {
                    vec![]
                } else {
                    vec![ids[0], ids[2]]
                },
                output: ids[1],
            },
            definition: None,
            clock: None,
        },
    )
    .unwrap();
    let f = a.outcome.created_mapping.unwrap();
    s = a.snapshot;
    s = apply_edit(
        &s,
        &EditOp::AttachDefinition {
            id: f,
            definition: Definition::Formula {
                source: src.clone(),
            },
        },
    )
    .unwrap()
    .snapshot;
    let e = elaborate_design(&s.design);
    let m = &e.mappings[&f];
    match &m.outcome {
        bdl_elab::RealizationOutcome::Elaborated(r) => {
            println!("{}", bdl_check::pretty::expr(&r.expr));
            match bdl_check::check_realization(&e.ir, f) {
                Ok(t) => println!("checks: {:?}", t.map(|t| bdl_check::pretty::kernel(&t))),
                Err(err) => println!("CHECK ERROR: {err:?}"),
            }
        }
        o => println!("{o:?}"),
    }
    for d in &m.diagnostics {
        println!(
            "{:?} {} @{:?}: {}{}",
            d.severity,
            d.code.as_str(),
            d.span,
            d.message,
            d.fixes
                .first()
                .map(|f| format!("  [fix: {f}]"))
                .unwrap_or_default()
        );
    }
}
