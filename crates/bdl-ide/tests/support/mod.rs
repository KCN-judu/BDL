//! Test designs shared by the query tests: the lamp (one unresolved
//! mapping over two concepts) and its extensions.
#![allow(dead_code)]

use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ClockId, ConceptId, DeclId, Dim, OutputId};

pub struct Lamp {
    pub snapshot: ProjectSnapshot,
    pub tilt: ConceptId,
    pub brightness: ConceptId,
    pub dim_by_tilt: DeclId,
}

pub fn edit(s: &ProjectSnapshot, op: EditOp) -> ProjectSnapshot {
    apply_edit(s, &op).expect("fixture edit applies").snapshot
}

fn created_concept(s: &ProjectSnapshot, op: EditOp) -> (ProjectSnapshot, ConceptId) {
    let a = apply_edit(s, &op).expect("fixture edit applies");
    let id = a.outcome.created_concept.expect("a concept was created");
    (a.snapshot, id)
}

fn created_mapping(s: &ProjectSnapshot, op: EditOp) -> (ProjectSnapshot, DeclId) {
    let a = apply_edit(s, &op).expect("fixture edit applies");
    let id = a.outcome.created_mapping.expect("a mapping was created");
    (a.snapshot, id)
}

pub fn concept(name: &str, rep: Option<Representation>) -> EditOp {
    EditOp::CreateConcept {
        name: name.into(),
        description: String::new(),
        representation: rep,
    }
}

pub fn mapping(name: &str, inputs: Vec<ConceptId>, output: ConceptId) -> EditOp {
    EditOp::CreateMapping {
        name: name.into(),
        description: String::new(),
        signature: Signature { inputs, output },
        definition: None,
        clock: None,
    }
}

pub fn formula(id: DeclId, source: &str) -> EditOp {
    EditOp::AttachDefinition {
        id,
        definition: Definition::Formula {
            source: source.into(),
        },
    }
}

/// `Tilt : Angle`, `Brightness : Scalar`, `dimByTilt : Tilt -> Brightness`
/// unresolved.
pub fn lamp() -> Lamp {
    let s = ProjectSnapshot::new(Design::empty("lamp"));
    let (s, tilt) = created_concept(
        &s,
        concept("Tilt", Some(Representation::Quantity { dim: Dim::ANGLE })),
    );
    let (s, brightness) = created_concept(
        &s,
        concept(
            "Brightness",
            Some(Representation::Quantity { dim: Dim::ZERO }),
        ),
    );
    let (snapshot, dim_by_tilt) = created_mapping(&s, mapping("dimByTilt", vec![tilt], brightness));
    Lamp {
        snapshot,
        tilt,
        brightness,
        dim_by_tilt,
    }
}

pub struct Contested {
    pub snapshot: ProjectSnapshot,
    pub brightness: ConceptId,
    pub light: OutputId,
    pub clock: ClockId,
    pub level_a: DeclId,
    pub level_b: DeclId,
}

/// Two nullary mappings both driving one required sink in one domain:
/// `output.multiple_drivers`.
pub fn contested_output() -> Contested {
    let s = ProjectSnapshot::new(Design::empty("lamp"));
    let (s, brightness) = created_concept(
        &s,
        concept(
            "Brightness",
            Some(Representation::Quantity { dim: Dim::ZERO }),
        ),
    );
    let a = apply_edit(
        &s,
        &EditOp::CreateClockDomain {
            name: "interaction".into(),
        },
    )
    .expect("clock");
    let clock = a.outcome.created_clock.expect("clock id");
    let s = a.snapshot;
    let a = apply_edit(
        &s,
        &EditOp::CreateOutput {
            name: "light".into(),
            description: String::new(),
            accepts: brightness,
            clock: Some(clock),
        },
    )
    .expect("output");
    let light = a.outcome.created_output.expect("output id");
    let s = a.snapshot;
    let (s, level_a) = created_mapping(&s, mapping("levelA", vec![], brightness));
    let (s, level_b) = created_mapping(&s, mapping("levelB", vec![], brightness));
    let mut s = s;
    for m in [level_a, level_b] {
        s = edit(&s, formula(m, "0.5"));
        s = edit(
            &s,
            EditOp::SetMappingClock {
                id: m,
                clock: Some(clock),
            },
        );
        s = edit(
            &s,
            EditOp::SetMappingDrive {
                id: m,
                output: Some(light),
            },
        );
    }
    Contested {
        snapshot: s,
        brightness,
        light,
        clock,
        level_a,
        level_b,
    }
}
