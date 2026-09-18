#![allow(clippy::unwrap_used)]
//! The Smart Lamp: the demonstration project under `examples/smart_lamp`,
//! authored entirely with surface edits (as Studio would), checked here
//! against everything the compiler knows about it — the ladder, the
//! output pass, four simulated ticks, deployment on the Nano — and kept
//! byte-identical to what this test writes.
//!
//! `BDL_WRITE_EXAMPLES=1 cargo test -p bdl-daemon --test examples`
//! rewrites the project directory.

use bdl_compiler::{analyze, analyze_deployment, DeploymentStatus, MappingStatus};
use bdl_model::edit::apply_edit;
use bdl_model::edit::EditOp;
use bdl_model::layout::{Layout, Point};
use bdl_model::persist;
use bdl_model::surface::ProjectSnapshot;
use bdl_model::surface::{Definition, DeviceKind, Representation, Signature};
use bdl_model::Dim;
use bdl_reactive::{InputTrace, Schedule, Simulation, Value};
use std::path::PathBuf;

const COMPILER_VERSION: &str = "example";

fn example_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/smart_lamp")
}

struct Authored {
    snapshot: ProjectSnapshot,
    layout: Layout,
}

/// Author the lamp through the same edits Studio sends, into `root`.
fn author(root: &std::path::Path) -> Authored {
    let created = persist::init_project(root, "smart_lamp", COMPILER_VERSION).unwrap();
    let mut s = created.snapshot;
    let apply = |s: &mut ProjectSnapshot, op: EditOp| {
        let a = apply_edit(s, &op).unwrap();
        *s = a.snapshot;
        a.outcome
    };
    let concept = |name: &str, dim: Dim, description: &str| EditOp::CreateConcept {
        name: name.into(),
        description: description.into(),
        representation: Some(Representation::Quantity { dim }),
    };
    let tilt = apply(
        &mut s,
        concept(
            "Tilt",
            Dim::ANGLE,
            "How far the lamp head is tilted from upright.",
        ),
    )
    .created_concept
    .unwrap();
    let ambient = apply(
        &mut s,
        concept(
            "AmbientLight",
            Dim {
                luminous: 1,
                angle: 2,
                length: -2,
                ..Dim::ZERO
            },
            "Illuminance of the room, as the light sensor reports it.",
        ),
    )
    .created_concept
    .unwrap();
    let brightness = apply(
        &mut s,
        concept(
            "Brightness",
            Dim::ZERO,
            "The lamp's output level, 0 (off) to 1 (full).",
        ),
    )
    .created_concept
    .unwrap();
    let interaction = apply(
        &mut s,
        EditOp::CreateClockDomain {
            name: "interaction".into(),
        },
    )
    .created_clock
    .unwrap();
    let mapping = |name: &str, inputs: Vec<bdl_model::SemanticId>, output, description: &str| {
        EditOp::CreateMapping {
            name: name.into(),
            description: description.into(),
            signature: Signature { inputs, output },
        }
    };
    let tilt_in = apply(
        &mut s,
        mapping(
            "tilt",
            vec![],
            tilt,
            "The tilt sensor: a value per activation, from outside.",
        ),
    )
    .created_mapping
    .unwrap();
    apply(
        &mut s,
        EditOp::SetMappingClock {
            id: tilt_in,
            clock: Some(interaction),
        },
    );
    let ambient_in = apply(
        &mut s,
        mapping(
            "ambient",
            vec![],
            ambient,
            "The light sensor: a value per activation.",
        ),
    )
    .created_mapping
    .unwrap();
    apply(
        &mut s,
        EditOp::SetMappingClock {
            id: ambient_in,
            clock: Some(interaction),
        },
    );
    let dim_by_tilt = apply(
        &mut s,
        mapping(
            "dimByTilt",
            vec![tilt],
            brightness,
            "Upright is off, flat is full: brightness grows with the tilt.",
        ),
    )
    .created_mapping
    .unwrap();
    apply(
        &mut s,
        EditOp::AttachDefinition {
            id: dim_by_tilt,
            definition: Definition::Formula {
                source: "Tilt / 90 deg".into(),
            },
        },
    );
    let adapt = apply(
        &mut s,
        mapping(
            "adaptBrightness",
            vec![brightness, ambient],
            brightness,
            "In a bright room the lamp need only be half as bright.",
        ),
    )
    .created_mapping
    .unwrap();
    apply(
        &mut s,
        EditOp::AttachDefinition {
            id: adapt,
            definition: Definition::Formula {
                source: "if AmbientLight > 300 lx then Brightness / 2 else Brightness".into(),
            },
        },
    );
    let bright = apply(
        &mut s,
        mapping(
            "brightness",
            vec![],
            brightness,
            "What the lamp shows: the tilt's brightness, adapted to the room.",
        ),
    )
    .created_mapping
    .unwrap();
    apply(
        &mut s,
        EditOp::AttachDefinition {
            id: bright,
            definition: Definition::Formula {
                source: "adaptBrightness(dimByTilt(tilt), ambient)".into(),
            },
        },
    );
    apply(
        &mut s,
        EditOp::SetMappingClock {
            id: bright,
            clock: Some(interaction),
        },
    );
    let light = apply(
        &mut s,
        EditOp::CreateOutput {
            name: "light".into(),
            description: "The lamp itself.".into(),
            accepts: brightness,
            clock: Some(interaction),
        },
    )
    .created_output
    .unwrap();
    apply(
        &mut s,
        EditOp::SetOutputRequired {
            id: light,
            required: true,
        },
    );
    apply(
        &mut s,
        EditOp::SetMappingDrive {
            id: bright,
            output: Some(light),
        },
    );
    apply(
        &mut s,
        EditOp::CreateDevice {
            name: "pwmLight".into(),
            kind: DeviceKind::PwmChannel,
            output: Some(light),
        },
    );
    let mut layout = Layout::default();
    for (i, c) in [tilt, ambient, brightness].iter().enumerate() {
        layout.concepts.insert(
            *c,
            Point {
                x: 48.0,
                y: 48.0 + 120.0 * i as f64,
            },
        );
    }
    for (i, m) in [tilt_in, ambient_in, dim_by_tilt, adapt, bright]
        .iter()
        .enumerate()
    {
        layout.mappings.insert(
            *m,
            Point {
                x: 368.0 + 320.0 * (i / 3) as f64,
                y: 48.0 + 130.0 * (i % 3) as f64,
            },
        );
    }
    layout.outputs.insert(
        light,
        Point {
            x: 1008.0,
            y: 178.0,
        },
    );
    persist::save_project(root, &s, &layout, COMPILER_VERSION).unwrap();
    Authored {
        snapshot: s,
        layout,
    }
}

#[test]
fn smart_lamp_is_authored_from_the_surface_and_does_everything_the_compiler_knows() {
    let dir = tempfile::tempdir().unwrap();
    let fresh = dir.path().join("smart_lamp");
    let authored = author(&fresh);
    let snapshot = authored.snapshot.clone();

    // the ladder: every defined relationship clock-consistent, the two
    // inputs declared; the sink driven; the design complete
    let a = analyze(&snapshot);
    for m in a.mappings.values() {
        let expected = if snapshot.design.mappings[&m.id].definition.is_none() {
            MappingStatus::Declared
        } else {
            MappingStatus::ClockConsistent
        };
        assert_eq!(m.status, expected, "{}: {:?}", m.id, m.diagnostics);
    }
    assert!(a.causality.valid && a.clocks.valid && a.output_complete);

    // four ticks: upright→flat in a dim room, then flat in a bright room
    let by_name = |n: &str| {
        snapshot
            .design
            .mappings
            .values()
            .find(|m| m.name == n)
            .unwrap()
            .id
    };
    let (tilt, ambient, bright) = (by_name("tilt"), by_name("ambient"), by_name("brightness"));
    let concept = |n: &str| {
        snapshot
            .design
            .concepts
            .values()
            .find(|c| c.name == n)
            .unwrap()
    };
    let mut inputs = InputTrace::default();
    inputs.series(
        tilt,
        [0.0f64, 45.0, 90.0, 90.0]
            .iter()
            .map(|deg| Value::sem(concept("Tilt").id, Value::q(Dim::ANGLE, deg.to_radians()))),
    );
    let lux = concept("AmbientLight");
    inputs.series(
        ambient,
        [50.0, 50.0, 50.0, 800.0].iter().map(|lx| {
            Value::sem(
                lux.id,
                Value::q(
                    match &lux.representation {
                        Some(Representation::Quantity { dim }) => *dim,
                        _ => unreachable!(),
                    },
                    *lx,
                ),
            )
        }),
    );
    let mut sim =
        Simulation::new(a.ir.clone(), &a.causality, Schedule::always(&a.ir), inputs).unwrap();
    let trace = sim.run(4).unwrap();
    let b: Vec<f64> = trace
        .series(bright)
        .iter()
        .map(|(_, v)| match v {
            Value::Semantic { repr, .. } => match **repr {
                Value::Quantity { value, .. } => value,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(b, vec![0.0, 0.5, 1.0, 0.5]);

    // deployment: one PWM line, placed on the Nano
    let nano = bdl_hardware::boards::by_name("arduino_nano").unwrap();
    let d = analyze_deployment(&snapshot, &nano);
    assert_eq!(d.status, DeploymentStatus::Feasible, "{:?}", d.diagnostics);
    assert_eq!(d.assignment.as_ref().unwrap().len(), 1);

    // the checked-in example is exactly this project in the unified
    // layout (ADR-0023): the legacy form just written is migrated in
    // place — sources, identities and layout — and compared file by
    // file (or written now).
    bdl_text::migrate_legacy(&fresh, COMPILER_VERSION)
        .unwrap()
        .expect("a legacy project migrates");
    let example = example_dir();
    if std::env::var_os("BDL_WRITE_EXAMPLES").is_some() {
        if example.exists() {
            std::fs::remove_dir_all(&example).unwrap();
        }
        copy_dir(&fresh, &example);
        // the JSON the migration renamed is not part of the example
        let _ = std::fs::remove_dir_all(example.join("design"));
    }
    let loaded = bdl_text::load_project(&example).unwrap();
    assert!(loaded.migrated.is_none(), "the example is already unified");
    let mut base = loaded.build.system.base.clone();
    for m in base.mappings.values_mut() {
        m.parameters.clear();
    }
    assert_eq!(base, snapshot.design);
    assert_eq!(loaded.layout, authored.layout);
    for f in [
        "src/main.bdl",
        bdl_text::IDENTITIES_FILE,
        bdl_text::AUTHORING_FILE,
        persist::LAYOUT_FILE,
        persist::MANIFEST_FILE,
    ] {
        assert_eq!(
            std::fs::read_to_string(example.join(f)).unwrap(),
            std::fs::read_to_string(fresh.join(f)).unwrap(),
            "{f}: examples/smart_lamp is stale — rerun with BDL_WRITE_EXAMPLES=1"
        );
    }
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}
