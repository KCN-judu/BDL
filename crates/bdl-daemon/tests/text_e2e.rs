//! Text projects over the wire (ADR-0020): created and edited through the
//! ordinary system requests, saved as source, reopened with the same
//! identities; a text project written by hand opens, analyses and
//! simulates like a Studio-authored one; a save over sources that
//! changed on disk is refused unless forced.

#![allow(clippy::unwrap_used)]

use bdl_protocol::framing;
use bdl_protocol::pb::{self, client_message::Payload as Req, response::Payload as Resp};
use bytes::BytesMut;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

struct Client {
    child: Child,
    buf: BytesMut,
    next_id: u64,
    last_revision: u64,
}

impl Client {
    fn spawn() -> Client {
        let child = Command::new(env!("CARGO_BIN_EXE_bdld"))
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn bdld");
        let mut c = Client {
            child,
            buf: BytesMut::new(),
            next_id: 1,
            last_revision: 0,
        };
        c.call(Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "text-e2e".into(),
            client_version: "0".into(),
        }));
        c
    }

    fn call(&mut self, payload: Req) -> Resp {
        let id = self.next_id;
        self.next_id += 1;
        let mut out = BytesMut::new();
        framing::encode(
            &pb::ClientMessage {
                request_id: id,
                payload: Some(payload),
            },
            &mut out,
        )
        .unwrap();
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(&out).unwrap();
        stdin.flush().unwrap();
        loop {
            let msg = loop {
                if let Some(m) = framing::decode::<pb::ServerMessage>(&mut self.buf).unwrap() {
                    break m;
                }
                let mut chunk = [0u8; 4096];
                let n = self
                    .child
                    .stdout
                    .as_mut()
                    .unwrap()
                    .read(&mut chunk)
                    .unwrap();
                assert!(n > 0, "daemon closed stdout");
                self.buf.extend_from_slice(&chunk[..n]);
            };
            match msg.payload.unwrap() {
                pb::server_message::Payload::Response(r) => {
                    assert_eq!(r.request_id, id);
                    let payload = r.payload.unwrap();
                    match &payload {
                        Resp::Project(p) => {
                            self.last_revision = p.project.as_ref().unwrap().revision
                        }
                        Resp::EditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        Resp::SystemEditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        Resp::SourceEditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        _ => {}
                    }
                    return payload;
                }
                pb::server_message::Payload::Event(_) => {}
            }
        }
    }

    fn base(&mut self, op: pb::edit_op::Op) -> pb::EditOutcome {
        let base = self.last_revision;
        match self.call(Req::ApplySystemEdit(pb::ApplySystemEditRequest {
            base_revision: base,
            op: Some(pb::SystemEditOp {
                op: Some(pb::system_edit_op::Op::Base(pb::EditOp { op: Some(op) })),
            }),
        })) {
            Resp::SystemEditApplied(e) => e.outcome.unwrap().inner.unwrap(),
            other => panic!("edit failed: {other:?}"),
        }
    }

    fn project(&mut self) -> pb::ProjectProjection {
        match self.call(Req::GetProject(pb::GetProjectRequest {})) {
            Resp::Project(p) => p.project.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn open(&mut self, root: &std::path::Path) -> pb::ProjectProjection {
        match self.call(Req::OpenProject(pb::OpenProjectRequest {
            root_path: root.to_string_lossy().into(),
        })) {
            Resp::Project(p) => p.project.unwrap(),
            other => panic!("open failed: {other:?}"),
        }
    }

    fn save(&mut self, force: bool) -> Resp {
        self.call(Req::SaveProject(pb::SaveProjectRequest { force }))
    }

    fn sources(&mut self) -> pb::SourcesView {
        match self.call(Req::GetSources(pb::GetSourcesRequest {})) {
            Resp::Sources(s) => s.sources.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn source_edit(&mut self, path: &str, text: &str) -> pb::SourceEditApplied {
        let base = self.last_revision;
        match self.call(Req::ApplySourceEdit(pb::ApplySourceEditRequest {
            base_revision: base,
            path: path.into(),
            text: text.into(),
        })) {
            Resp::SourceEditApplied(e) => e,
            other => panic!("source edit failed: {other:?}"),
        }
    }
}

fn quantity(dim: pb::Dim) -> Option<pb::Representation> {
    Some(pb::Representation {
        kind: Some(pb::representation::Kind::Quantity(dim)),
    })
}

fn angle() -> pb::Dim {
    pb::Dim {
        angle: 1,
        ..Default::default()
    }
}

#[test]
fn a_text_project_is_edited_over_the_wire_saved_as_source_and_reopened() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut c = Client::spawn();
    let Resp::Project(p) = c.call(Req::InitTextProject(pb::InitTextProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "lamp".into(),
    })) else {
        panic!()
    };
    let p = p.project.unwrap();
    assert_eq!(p.kind(), pb::ProjectKind::Text);
    assert!(root.join("bdl.toml").is_file());
    assert!(root.join("src/main.bdl").is_file());
    assert!(root.join(".bdl/identities.json").is_file());
    assert!(
        !root.join("design").exists(),
        "a text project has no JSON design"
    );

    // Studio-style edits through the system request.
    let tilt = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Tilt".into(),
            description: String::new(),
            representation: quantity(angle()),
        }))
        .created_concept
        .unwrap();
    let bright = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Brightness".into(),
            description: String::new(),
            representation: quantity(pb::Dim::default()),
        }))
        .created_concept
        .unwrap();
    let main = c
        .base(pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
            name: "main".into(),
        }))
        .created_clock
        .unwrap();
    let raw = c
        .base(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "raw".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![],
                output: tilt,
            }),
        }))
        .created_mapping
        .unwrap();
    let dim = c
        .base(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![tilt],
                output: bright,
            }),
        }))
        .created_mapping
        .unwrap();
    c.base(pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
        id: dim,
        definition: Some(pb::Definition {
            kind: Some(pb::definition::Kind::Formula("Tilt / (90 deg)".into())),
        }),
    }));
    let brightness = c
        .base(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "brightness".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![],
                output: bright,
            }),
        }))
        .created_mapping
        .unwrap();
    c.base(pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
        id: brightness,
        definition: Some(pb::Definition {
            kind: Some(pb::definition::Kind::Formula("dimByTilt(raw)".into())),
        }),
    }));
    for m in [raw, brightness] {
        c.base(pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id: m,
            clock_id: Some(main),
        }));
    }
    let light = c
        .base(pb::edit_op::Op::CreateOutput(pb::CreateOutput {
            name: "light".into(),
            description: String::new(),
            accepts: bright,
            clock_id: Some(main),
        }))
        .created_output
        .unwrap();
    c.base(pb::edit_op::Op::SetMappingDrive(pb::SetMappingDrive {
        id: brightness,
        output_id: Some(light),
    }));
    assert!(c.project().dirty);

    // Save: the sources now declare everything.
    let Resp::Project(saved) = c.save(false) else {
        panic!("save refused")
    };
    assert!(!saved.project.unwrap().dirty);
    let text = std::fs::read_to_string(root.join("src/main.bdl")).unwrap();
    assert!(text.contains("concept Tilt : Angle\n"), "{text}");
    assert!(
        text.contains(
            "mapping dimByTilt : Tilt -> Brightness\ndimByTilt(Tilt) =\n  Tilt / (90 deg)\n"
        ),
        "{text}"
    );
    // a relationship without inputs is written in the preferred spelling
    // from the start: never the output-only shorthand
    assert!(
        text.contains("mapping brightness : () -> Brightness @main\n"),
        "{text}"
    );
    assert!(text.contains("mapping raw : () -> Tilt @main\n"), "{text}");
    assert!(!text.contains("mapping raw : Tilt"), "{text}");
    assert!(text.contains("output light : Brightness @main\n"), "{text}");
    assert!(text.contains("drive light = brightness\n"), "{text}");
    let ids = std::fs::read_to_string(root.join(".bdl/identities.json")).unwrap();
    assert!(ids.contains("\"concept:Tilt\""), "{ids}");

    // Reopen: the same identities, the same design, executable.
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let p = c.open(&root);
    assert_eq!(p.kind(), pb::ProjectKind::Text);
    let concept_ids: Vec<(u64, String)> =
        p.concepts.iter().map(|x| (x.id, x.name.clone())).collect();
    assert!(
        concept_ids.contains(&(tilt, "Tilt".into())),
        "{concept_ids:?}"
    );
    assert!(concept_ids.contains(&(bright, "Brightness".into())));
    let m: Vec<(u64, String)> = p.mappings.iter().map(|x| (x.id, x.name.clone())).collect();
    assert!(m.contains(&(dim, "dimByTilt".into())), "{m:?}");
    assert!(m.contains(&(brightness, "brightness".into())));
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {})) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    assert!(a.causal && a.clock_consistent && a.output_complete, "{a:?}");

    // An external edit (a rename in the file): saving refuses, reload
    // takes it in with the identity kept, then saving works.
    let edited = text
        .replace("concept Tilt : Angle", "concept HeadTilt : Angle")
        .replace("mapping raw : Tilt", "mapping raw : HeadTilt")
        .replace(
            "dimByTilt : Tilt -> Brightness\ndimByTilt(Tilt) =\n  Tilt / (90 deg)",
            "dimByTilt : HeadTilt -> Brightness\ndimByTilt(t) =\n  t / (90 deg)",
        );
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(root.join("src/main.bdl"), &edited).unwrap();
    c.base(pb::edit_op::Op::SetConceptDescription(
        pb::SetConceptDescription {
            id: bright,
            description: "0 to 1".into(),
        },
    ));
    let Resp::Error(e) = c.save(false) else {
        panic!("a save over changed sources must be refused")
    };
    assert_eq!(e.code, "project.changed_on_disk");
    let Resp::Project(p) = c.call(Req::ReloadProject(pb::ReloadProjectRequest {})) else {
        panic!()
    };
    let p = p.project.unwrap();
    let renamed = p.concepts.iter().find(|x| x.name == "HeadTilt").unwrap();
    assert_eq!(
        renamed.id, tilt,
        "a unique same-file rename keeps the identity"
    );
    let Resp::Project(_) = c.save(false) else {
        panic!("save after reload")
    };
}

#[test]
fn a_hand_written_text_system_opens_analyses_and_simulates() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("system");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("bdl.toml"),
        "schema_version = 1\nname = \"system\"\nkind = \"text\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/system.bdl"),
        include_str!("../../bdl-syntax/test_data/valid/system.bdl"),
    )
    .unwrap();
    let mut c = Client::spawn();
    let p = c.open(&root);
    assert_eq!(p.kind(), pb::ProjectKind::Text);
    let Resp::SystemAnalysis(sa) = c.call(Req::RunSystemAnalysis(pb::RunSystemAnalysisRequest {}))
    else {
        panic!()
    };
    let sa = sa.analysis.unwrap();
    assert_eq!(
        sa.acceptance(),
        pb::SystemAcceptance::Executable,
        "{:?}",
        sa.composition
    );
    let Resp::System(s) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    let s = s.system.unwrap();
    assert_eq!(s.components.len(), 1);
    assert_eq!(s.instances.len(), 2);
    assert_eq!(s.bindings.len(), 5);

    // Simulate: tilt 90° → lampA.brightness = 1 × gain 2 = 2, mirror (lampB, gain 1) = 1.
    let tilt = p.concepts.iter().find(|x| x.name == "Tilt").unwrap().id;
    let interaction = p
        .clocks
        .iter()
        .find(|k| k.name == "interaction")
        .unwrap()
        .id;
    let display = p.clocks.iter().find(|k| k.name == "display").unwrap().id;
    let raw = p.mappings.iter().find(|m| m.name == "tilt").unwrap().id;
    let mirror = p.mappings.iter().find(|m| m.name == "mirror").unwrap().id;
    let bright = p
        .mappings
        .iter()
        .find(|m| m.name == "brightness")
        .unwrap()
        .id;
    let Resp::Simulation(sim) = c.call(Req::StartSimulation(pb::StartSimulationRequest {
        inputs: (0..3)
            .map(|t| pb::SimulationInput {
                mapping_id: raw,
                tick: t,
                value: Some(pb::Value {
                    kind: Some(pb::value::Kind::Semantic(Box::new(pb::SemanticValue {
                        concept_id: tilt,
                        repr: Some(Box::new(pb::Value {
                            kind: Some(pb::value::Kind::Quantity(pb::Quantity {
                                dim: Some(angle()),
                                value: 90.0f64.to_radians(),
                            })),
                        })),
                    }))),
                }),
            })
            .collect(),
        schedule: vec![
            pb::SchedulePeriod {
                clock_id: interaction,
                period: 1,
            },
            pb::SchedulePeriod {
                clock_id: display,
                period: 2,
            },
        ],
    })) else {
        panic!()
    };
    assert!(sim.error.is_none(), "{:?}", sim.error);
    let Resp::Simulation(sim) = c.call(Req::StepSimulation(pb::StepSimulationRequest { ticks: 3 }))
    else {
        panic!()
    };
    assert!(sim.error.is_none(), "{:?}", sim.error);
    let last = &sim.samples[2];
    let b = last.values.iter().find(|v| v.mapping_id == bright).unwrap();
    assert_eq!(b.rendered, "Brightness(2)", "lampA with gain 2");
    let m = last.values.iter().find(|v| v.mapping_id == mirror).unwrap();
    assert_eq!(m.rendered, "Brightness(1)", "lampB with gain 1");
}

/// The layout service (ADR-0023 §7): a project written by hand has no
/// layout; opening it places every entity and persists the placement;
/// reopening changes nothing; a commit places what it created and never
/// moves what has a position.
#[test]
fn a_hand_written_project_is_placed_on_open_and_new_items_on_commit() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("system");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("bdl.toml"),
        "schema_version = 2\nname = \"system\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/system.bdl"),
        include_str!("../../bdl-syntax/test_data/valid/system.bdl"),
    )
    .unwrap();
    assert!(!root.join("ui/layout.json").exists());
    let mut c = Client::spawn();
    let p = c.open(&root);
    let layout = p.layout.clone().unwrap();
    // the system canvas shows the system's own design (base ids) and its
    // instances; the flat projection's per-instance copies are not nodes
    let Resp::System(s) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    let s = s.system.unwrap();
    let base = s.base.as_ref().unwrap();
    for concept in &base.concepts {
        assert!(
            layout.concepts.iter().any(|n| n.id == concept.id),
            "{} unplaced",
            concept.name
        );
    }
    for m in &base.mappings {
        assert!(
            layout.mappings.iter().any(|n| n.id == m.id),
            "{} unplaced",
            m.name
        );
    }
    for o in &base.outputs {
        assert!(
            layout.outputs.iter().any(|n| n.id == o.id),
            "{} unplaced",
            o.name
        );
    }
    for i in &s.instances {
        assert!(
            layout.instances.iter().any(|n| n.id == i.id),
            "{} unplaced",
            i.name
        );
    }
    assert_eq!(
        layout.components.len(),
        s.components.len(),
        "every body canvas"
    );
    assert!(!p.dirty, "a placement is not an unsaved change");
    assert!(root.join("ui/layout.json").is_file(), "persisted on open");

    // reopening: the same layout, nothing placed again
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let p2 = c.open(&root);
    assert_eq!(p2.layout, p.layout);

    // a commit places what it created and leaves everything else alone
    let tilt = base.concepts.iter().find(|x| x.name == "Tilt").unwrap().id;
    let before = p2.layout.clone().unwrap();
    let tilt_at = *before.concepts.iter().find(|n| n.id == tilt).unwrap();
    let created = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Warmth".into(),
            description: String::new(),
            representation: quantity(pb::Dim::default()),
        }))
        .created_concept
        .unwrap();
    let after = c.project().layout.unwrap();
    assert!(
        after.concepts.iter().any(|n| n.id == created),
        "placed on commit"
    );
    assert_eq!(
        after.concepts.iter().find(|n| n.id == tilt).unwrap(),
        &tilt_at,
        "a positioned node never moves"
    );
    assert_eq!(after.mappings, before.mappings);
    assert_eq!(after.outputs, before.outputs);
    assert_eq!(after.instances, before.instances);
}

/// The Code view (ADR-0023 §3–§5): the sources are the project with every
/// graph edit written back; a text edit that builds is a new revision
/// bound to the same identities; one that does not keeps the last good
/// project and the draft; comments survive graph edits; a save writes
/// the accepted text; undo is one history.
#[test]
fn code_view_edits_flow_through_the_model_and_keep_identities() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut c = Client::spawn();
    let Resp::Project(p) = c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "lamp".into(),
    })) else {
        panic!()
    };
    c.last_revision = p.project.unwrap().revision;
    let tilt = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Tilt".into(),
            description: "how far the head is tilted".into(),
            representation: quantity(angle()),
        }))
        .created_concept
        .unwrap();
    let bright = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Brightness".into(),
            description: String::new(),
            representation: quantity(pb::Dim::default()),
        }))
        .created_concept
        .unwrap();

    // graph → text: the sources carry what the canvas made
    let sources = c.sources();
    assert_eq!(sources.revision, c.last_revision);
    let main = sources
        .files
        .iter()
        .find(|f| f.path == "src/main.bdl")
        .expect("main.bdl");
    assert!(!main.draft);
    assert!(main.text.contains("concept Tilt : Angle"), "{}", main.text);
    assert!(
        main.text.contains("/// how far the head is tilted"),
        "{}",
        main.text
    );
    assert!(sources.diagnostics.is_empty());
    // anchors say where each entity's item is in the text as shown
    let at = main
        .anchors
        .iter()
        .find(|a| a.entity == Some(pb::source_anchor::Entity::ConceptId(tilt)))
        .expect("Tilt anchored");
    let item = &main.text[at.start as usize..at.end as usize];
    assert!(item.ends_with("concept Tilt : Angle"), "{item:?}");
    assert!(
        item.starts_with("/// how far"),
        "the item is doc comment to end"
    );

    // text → graph: a relationship typed in the Code view, with a comment
    let typed = format!(
        "{}\n// a note the model does not keep\n/// dims with the tilt\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(Tilt) =\n  Tilt / 90 deg\n",
        main.text.trim_end()
    );
    let applied = c.source_edit("src/main.bdl", &typed);
    assert!(applied.accepted, "{:?}", applied.sources);
    let p = applied.project.unwrap();
    assert_eq!(p.revision, c.last_revision);
    let dim = p
        .mappings
        .iter()
        .find(|m| m.name == "dimByTilt")
        .expect("bound");
    assert_eq!(dim.signature.as_ref().unwrap().inputs, vec![tilt]);
    assert_eq!(dim.signature.as_ref().unwrap().output, bright);
    assert!(matches!(
        dim.definition.as_ref().and_then(|d| d.kind.as_ref()),
        Some(pb::definition::Kind::Formula(f)) if f.contains("90 deg")
    ));
    assert_eq!(
        p.concepts.iter().find(|x| x.name == "Tilt").unwrap().id,
        tilt,
        "identities survive a text edit"
    );
    assert!(
        p.layout
            .as_ref()
            .unwrap()
            .mappings
            .iter()
            .any(|n| n.id == dim.id),
        "placed by the layout service on commit"
    );
    let dim_id = dim.id;

    // invalid text keeps the last known good project and the draft
    let broken = typed.replace("Tilt -> Brightness", "Tilt -> ");
    let refused = c.source_edit("src/main.bdl", &broken);
    assert!(!refused.accepted);
    let p = refused.project.unwrap();
    assert_eq!(p.revision, c.last_revision, "no revision");
    assert!(
        p.mappings.iter().any(|m| m.id == dim_id),
        "nothing discarded"
    );
    let sources = refused.sources.unwrap();
    let main = sources
        .files
        .iter()
        .find(|f| f.path == "src/main.bdl")
        .unwrap();
    assert!(main.draft);
    assert_eq!(main.text, broken, "the draft exactly as typed");
    let d = sources
        .diagnostics
        .iter()
        .find(|d| !d.open)
        .expect("a fault");
    assert_eq!(d.path, "src/main.bdl");
    assert!(d.end >= d.start && (d.end as usize) <= broken.len());
    assert_eq!(
        c.sources()
            .files
            .iter()
            .find(|f| f.path == "src/main.bdl")
            .unwrap()
            .text,
        broken
    );

    // a rename in the text keeps the identity (reconciliation, ADR-0020 §4)
    let renamed = typed
        .replace("concept Tilt", "concept HeadTilt")
        .replace(": Tilt ->", ": HeadTilt ->")
        .replace("dimByTilt(Tilt)", "dimByTilt(HeadTilt)")
        .replace("  Tilt / 90", "  HeadTilt / 90");
    let applied = c.source_edit("src/main.bdl", &renamed);
    assert!(applied.accepted, "{:?}", applied.sources);
    let p = applied.project.unwrap();
    assert_eq!(
        p.concepts.iter().find(|x| x.name == "HeadTilt").unwrap().id,
        tilt
    );
    assert!(
        !applied.sources.unwrap().files[0].draft,
        "the draft is gone"
    );

    // graph → text again: a rename from the canvas patches the text and
    // leaves the comments where they were
    c.base(pb::edit_op::Op::RenameConcept(pb::RenameConcept {
        id: bright,
        name: "Level".into(),
    }));
    let main = c
        .sources()
        .files
        .into_iter()
        .find(|f| f.path == "src/main.bdl")
        .unwrap();
    assert!(
        main.text.contains("mapping dimByTilt : HeadTilt -> Level"),
        "{}",
        main.text
    );
    assert!(
        main.text.contains("// a note the model does not keep"),
        "{}",
        main.text
    );
    assert!(
        main.text.contains("/// dims with the tilt"),
        "{}",
        main.text
    );
    assert!(
        main.text.contains("/// how far the head is tilted"),
        "{}",
        main.text
    );

    // stale base is refused
    let Resp::Error(e) = c.call(Req::ApplySourceEdit(pb::ApplySourceEditRequest {
        base_revision: 0,
        path: "src/main.bdl".into(),
        text: main.text.clone(),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "edit.stale_revision");
    let Resp::Error(e) = c.call(Req::ApplySourceEdit(pb::ApplySourceEditRequest {
        base_revision: c.last_revision,
        path: "../outside.bdl".into(),
        text: String::new(),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "source.invalid_path");

    // save: the text on disk is the Code view's; reopen keeps identities
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    let on_disk = std::fs::read_to_string(root.join("src/main.bdl")).unwrap();
    assert_eq!(on_disk, main.text);
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let p = c.open(&root);
    assert_eq!(
        p.concepts.iter().find(|x| x.name == "HeadTilt").unwrap().id,
        tilt
    );
    assert_eq!(
        p.mappings
            .iter()
            .find(|m| m.name == "dimByTilt")
            .unwrap()
            .id,
        dim_id
    );

    // one history: a text edit that adds a sink undoes like a graph edit
    let with_sink = format!("{}\noutput light : Level\n", on_disk.trim_end());
    let applied = c.source_edit("src/main.bdl", &with_sink);
    assert!(applied.accepted, "{:?}", applied.sources);
    assert_eq!(applied.project.as_ref().unwrap().outputs.len(), 1);
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    let p = u.project.unwrap();
    c.last_revision = p.revision;
    assert!(p.outputs.is_empty());
    let main = c
        .sources()
        .files
        .into_iter()
        .find(|f| f.path == "src/main.bdl")
        .unwrap();
    assert!(!main.text.contains("output light"), "{}", main.text);
    assert!(main.text.contains("// a note the model does not keep"));
}

/// Saving saves the whole authoring state: a definition draft (valid,
/// invalid or empty) and text that does not build survive save, close and
/// reopen exactly as typed; the graph keeps its identities; dirty is one
/// question — does the persistent state differ from what is saved.
#[test]
fn unfinished_edits_survive_save_close_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut c = Client::spawn();
    let Resp::Project(p) = c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "lamp".into(),
    })) else {
        panic!()
    };
    c.last_revision = p.project.unwrap().revision;
    let tilt = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Tilt".into(),
            description: String::new(),
            representation: quantity(angle()),
        }))
        .created_concept
        .unwrap();
    let bright = c
        .base(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Brightness".into(),
            description: String::new(),
            representation: quantity(pb::Dim::default()),
        }))
        .created_concept
        .unwrap();
    let dim = c
        .base(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![tilt],
                output: bright,
            }),
        }))
        .created_mapping
        .unwrap();
    let level = c
        .base(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "level".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![],
                output: bright,
            }),
        }))
        .created_mapping
        .unwrap();
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);

    // a valid draft, an invalid one, an empty one: each dirties the project
    for (id, text) in [(dim, "Tilt / 90 deg"), (level, "Tilt +"), (level, "")] {
        let Resp::DefinitionDraft(_) = c.call(Req::AnalyzeDefinitionDraft(
            pb::AnalyzeDefinitionDraftRequest {
                revision: c.last_revision,
                mapping_id: id,
                generation: 1,
                source: text.into(),
                component: None,
            },
        )) else {
            panic!("draft {text:?}")
        };
    }
    assert!(c.project().dirty, "a draft is unsaved work");

    // text that does not build, in the Code view
    let typed = format!(
        "{}\n// a note\noutput light : Brightness\ndrive light =\n",
        c.sources().files[0].text.trim_end()
    );
    let refused = c.source_edit("src/main.bdl", &typed);
    assert!(!refused.accepted);
    assert!(c.project().dirty);

    // save: everything as typed
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty, "saved: nothing differs");
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.bdl")).unwrap(),
        typed,
        "the typed text is the file"
    );
    let Resp::System(s) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    let drafts = s.system.unwrap().definition_drafts;
    assert_eq!(drafts.len(), 2);
    assert!(drafts
        .iter()
        .any(|d| d.mapping_id == dim && d.source == "Tilt / 90 deg"));
    assert!(drafts
        .iter()
        .any(|d| d.mapping_id == level && d.source.is_empty()));

    // close, reopen: the same state, the same ids, still clean
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let p = c.open(&root);
    assert!(!p.dirty);
    assert_eq!(
        p.mappings
            .iter()
            .find(|m| m.name == "dimByTilt")
            .unwrap()
            .id,
        dim
    );
    assert!(
        p.mappings.iter().all(|m| m.definition.is_none()),
        "nothing committed"
    );
    let sources = c.sources();
    let main = sources
        .files
        .iter()
        .find(|f| f.path == "src/main.bdl")
        .unwrap();
    assert!(main.draft);
    assert_eq!(main.text, typed, "exactly the typed text");
    assert!(
        sources.diagnostics.iter().any(|d| !d.open),
        "with its reasons"
    );
    let Resp::System(s) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    let mut again = s.system.unwrap().definition_drafts;
    again.sort_by_key(|d| d.mapping_id);
    assert_eq!(again.len(), 2);
    assert_eq!(again[0].mapping_id, dim);
    assert_eq!(again[0].source, "Tilt / 90 deg");
    assert_eq!(again[1].mapping_id, level);
    assert_eq!(again[1].source, "");

    // discarding a draft, or fixing the text, is unsaved work again
    c.call(Req::DiscardDefinitionDraft(
        pb::DiscardDefinitionDraftRequest {
            mapping_id: level,
            component: None,
        },
    ));
    assert!(c.project().dirty);
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    let fixed = typed.replace("drive light =\n", "drive light = level\n");
    let applied = c.source_edit("src/main.bdl", &fixed);
    assert!(applied.accepted, "{:?}", applied.sources);
    assert!(c.project().dirty);
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.bdl")).unwrap(),
        fixed
    );
    let sidecar = std::fs::read_to_string(root.join(".bdl/authoring.json")).unwrap();
    assert!(!sidecar.contains("last_good"), "no draft left: {sidecar}");
    assert!(
        sidecar.contains("Tilt / 90 deg"),
        "the definition draft stays: {sidecar}"
    );
}

/// A standard Source item is an ordinary library mechanism (ADR-0032):
/// one commit creates a concept and an unresolved `() -> concept`
/// relationship, the source is written in the preferred spelling, never
/// the shorthand, and reopening re-derives the same shape from the text.
#[test]
fn a_source_item_is_two_ordinary_edits_in_one_commit_and_writes_the_unit_domain() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut c = Client::spawn();
    let Resp::Project(p) = c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "lamp".into(),
    })) else {
        panic!()
    };
    c.last_revision = p.project.unwrap().revision;

    // the library serves the Sources as items: what each creates, in the
    // canonical English (Studio localizes by id); the concept templates of
    // the legacy request carry nothing of it
    let Resp::LibraryItems(t) = c.call(Req::ListLibraryItems(pb::ListLibraryItemsRequest {}))
    else {
        panic!()
    };
    let std = &t.libraries[0];
    let temp = std
        .items
        .iter()
        .find(|x| x.id == "std.source.temperature")
        .expect("served");
    assert_eq!(temp.category, "source");
    assert_eq!(temp.display_name, "Temperature Sensor");
    assert_eq!(temp.creates.len(), 2);
    assert_eq!(temp.creates[1].name, "TempSensor");
    assert_eq!(temp.creates[1].signature, "() -> RoomTemp");
    assert!(std
        .items
        .iter()
        .filter(|x| x.category == "source")
        .all(|x| x.creates.len() == 2 && x.creates[1].signature.starts_with("() -> ")));
    assert!(std
        .items
        .iter()
        .filter(|x| x.category == "concept")
        .all(|x| x.creates.len() == 1 && x.concept.is_some()));
    #[allow(deprecated)]
    {
        let Resp::ConceptTemplates(legacy) = c.call(Req::ListConceptTemplates(
            pb::ListConceptTemplatesRequest {},
        )) else {
            panic!()
        };
        assert!(legacy.libraries[0]
            .templates
            .iter()
            .all(|x| x.source_default_name.is_empty() && x.display_names.is_empty()));
        assert!(!legacy.libraries[0]
            .templates
            .iter()
            .any(|x| x.id.starts_with("std.source.")));
        // the legacy request knows no Source: a 0.16 `source_name` changes
        // nothing, and a Source id is not a template
        let Resp::Error(e) = c.call(Req::InstantiateConceptTemplate(
            pb::InstantiateConceptTemplateRequest {
                base_revision: c.last_revision,
                template_id: "std.source.temperature".into(),
                name: None,
                component: None,
                source_name: Some("TempSensor".into()),
            },
        )) else {
            panic!("a Source is not a template")
        };
        assert_eq!(e.code, "library.unknown_template");
        let Resp::SystemEditApplied(e) = c.call(Req::InstantiateConceptTemplate(
            pb::InstantiateConceptTemplateRequest {
                base_revision: c.last_revision,
                template_id: "std.environment.humidity".into(),
                name: None,
                component: None,
                source_name: Some("Hygrometer".into()),
            },
        )) else {
            panic!("a Concept item through the legacy request")
        };
        let p = e.project.unwrap();
        c.last_revision = p.revision;
        assert!(
            p.mappings.is_empty(),
            "`source_name` is ignored: no relationship"
        );
        assert_eq!(p.concepts.len(), 1);
        let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
            panic!()
        };
        c.last_revision = u.project.unwrap().revision;
    }

    // instantiate, naming both as a designer would
    let before = c.last_revision;
    let Resp::SystemEditApplied(e) = c.call(Req::InstantiateLibraryItem(
        pb::InstantiateLibraryItemRequest {
            base_revision: before,
            item_id: "std.source.temperature".into(),
            names: [
                ("value".to_string(), "RoomTemp".to_string()),
                ("source".to_string(), "TempSensor".to_string()),
            ]
            .into_iter()
            .collect(),
            component: None,
        },
    )) else {
        panic!()
    };
    let p = e.project.unwrap();
    c.last_revision = p.revision;
    assert!(p.revision > before);
    let outcome = e.outcome.unwrap().inner.unwrap();
    let concept = outcome.created_concept.expect("the concept");
    let source = outcome.created_mapping.expect("the relationship");
    let m = p.mappings.iter().find(|m| m.id == source).unwrap();
    assert_eq!(m.name, "TempSensor");
    assert!(
        m.signature.as_ref().unwrap().inputs.is_empty(),
        "unit domain"
    );
    assert_eq!(m.signature.as_ref().unwrap().output, concept);
    assert!(
        m.definition.is_none(),
        "unresolved: the environment provides it"
    );
    assert_eq!(
        p.concepts.iter().find(|x| x.id == concept).unwrap().name,
        "RoomTemp"
    );
    // one undo removes both: they are one history entry
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    let p = u.project.unwrap();
    c.last_revision = p.revision;
    assert!(p.mappings.is_empty() && p.concepts.is_empty());
    let Resp::SystemEditApplied(r) = c.call(Req::Redo(pb::RedoRequest {})) else {
        panic!()
    };
    c.last_revision = r.project.unwrap().revision;

    // the text: the preferred spelling, never the shorthand; a save keeps it
    let main = c
        .sources()
        .files
        .into_iter()
        .find(|f| f.path == "src/main.bdl")
        .unwrap();
    assert!(
        main.text.contains("mapping TempSensor : () -> RoomTemp"),
        "{}",
        main.text
    );
    assert!(
        !main.text.contains("mapping TempSensor : RoomTemp\n"),
        "{}",
        main.text
    );
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    let on_disk = std::fs::read_to_string(root.join("src/main.bdl")).unwrap();
    assert!(
        on_disk.contains("mapping TempSensor : () -> RoomTemp"),
        "{on_disk}"
    );

    // reopen: the same shape, the same ids — the role is re-derived, nothing
    // about the template or the role is in the project
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let p = c.open(&root);
    let m = p.mappings.iter().find(|m| m.name == "TempSensor").unwrap();
    assert_eq!(m.id, source);
    assert!(m.signature.as_ref().unwrap().inputs.is_empty());
    assert!(m.definition.is_none());
    let sidecar = std::fs::read_to_string(root.join(".bdl/authoring.json")).unwrap();
    assert!(!sidecar.to_lowercase().contains("source"), "{sidecar}");
    assert!(!sidecar.contains("template"), "{sidecar}");
}
