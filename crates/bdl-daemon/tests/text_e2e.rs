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
            // the preset fixture beside the Standard Library: the Source
            // item mechanics stay covered though std ships none (0.3)
            .env(
                "BDL_LIBRARIES",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/presets.toml"),
            )
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
            ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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
    assert!(text.contains("drive light by brightness\n"), "{text}");
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
    // the nodes are the ladder's (ADR-0044): every Sem block (unit-domain
    // declaration) and, when defined, its mapping block; a concept and a
    // rule are templates and get no position
    assert!(layout.concepts.is_empty());
    for m in &base.mappings {
        let sig = m.signature.as_ref().unwrap();
        if !sig.inputs.is_empty() {
            assert!(
                !layout.mappings.iter().any(|n| n.id == m.id),
                "{} is a rule, not a node",
                m.name
            );
            continue;
        }
        assert!(
            layout.mappings.iter().any(|n| n.id == m.id),
            "{} unplaced",
            m.name
        );
        assert_eq!(
            layout.definitions.iter().any(|n| n.id == m.id),
            m.definition.is_some(),
            "{}'s mapping block",
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

    // a commit places what it created and leaves everything else alone:
    // a new Sem block of an existing concept
    let tilt = base.concepts.iter().find(|x| x.name == "Tilt").unwrap().id;
    let before = p2.layout.clone().unwrap();
    let created = c
        .base(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "tiltAgain".into(),
            signature: Some(pb::Signature {
                inputs: vec![],
                output: tilt,
            }),
            ..Default::default()
        }))
        .created_mapping
        .unwrap();
    let after = c.project().layout.unwrap();
    assert!(
        after.mappings.iter().any(|n| n.id == created),
        "placed on commit"
    );
    for n in &before.mappings {
        assert_eq!(
            after.mappings.iter().find(|x| x.id == n.id).unwrap(),
            n,
            "a positioned node never moves"
        );
    }
    assert_eq!(after.definitions, before.definitions);
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
        template: None,
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
    // a rule is a template, not a node (ADR-0044): nothing is placed for it
    assert!(
        !p.layout
            .as_ref()
            .unwrap()
            .mappings
            .iter()
            .any(|n| n.id == dim.id),
        "a rule gets no position"
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
        template: None,
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
            ..Default::default()
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
            ..Default::default()
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
        "{}\n// a note\noutput light : Brightness\ndrive light by\n",
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
    let fixed = typed.replace("drive light by\n", "drive light by level\n");
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
        template: None,
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
    let std = t
        .libraries
        .iter()
        .find(|l| l.id == "fx")
        .expect("the fixture library beside std");
    let temp = std
        .items
        .iter()
        .find(|x| x.id == "fx.source.temperature")
        .expect("served");
    assert_eq!(temp.category, "source");
    assert_eq!(temp.display_name, "Temperature Input");
    assert_eq!(temp.creates.len(), 2);
    assert_eq!(temp.creates[1].name, "temperatureInput");
    assert_eq!(temp.creates[1].signature, "() -> Temperature");
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
            .any(|x| x.id.starts_with("fx.source.")));
        // the legacy request knows no Source: a 0.16 `source_name` changes
        // nothing, and a Source id is not a template
        let Resp::Error(e) = c.call(Req::InstantiateConceptTemplate(
            pb::InstantiateConceptTemplateRequest {
                base_revision: c.last_revision,
                template_id: "fx.source.temperature".into(),
                name: None,
                component: None,
                source_name: Some("temperatureInput".into()),
            },
        )) else {
            panic!("a Source is not a template")
        };
        assert_eq!(e.code, "library.unknown_template");
        let Resp::SystemEditApplied(e) = c.call(Req::InstantiateConceptTemplate(
            pb::InstantiateConceptTemplateRequest {
                base_revision: c.last_revision,
                template_id: "fx.humidity".into(),
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
            item_id: "fx.source.temperature".into(),
            names: [
                ("value".to_string(), "Temperature".to_string()),
                ("source".to_string(), "temperatureInput".to_string()),
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
    assert_eq!(m.name, "temperatureInput");
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
        "Temperature"
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
        main.text
            .contains("mapping temperatureInput : () -> Temperature"),
        "{}",
        main.text
    );
    assert!(
        !main
            .text
            .contains("mapping temperatureInput : Temperature\n"),
        "{}",
        main.text
    );
    let Resp::Project(saved) = c.save(false) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    let on_disk = std::fs::read_to_string(root.join("src/main.bdl")).unwrap();
    assert!(
        on_disk.contains("mapping temperatureInput : () -> Temperature"),
        "{on_disk}"
    );

    // reopen: the same shape, the same ids — the role is re-derived, nothing
    // about the template or the role is in the project
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let p = c.open(&root);
    let m = p
        .mappings
        .iter()
        .find(|m| m.name == "temperatureInput")
        .unwrap();
    assert_eq!(m.id, source);
    assert!(m.signature.as_ref().unwrap().inputs.is_empty());
    assert!(m.definition.is_none());
    let sidecar = std::fs::read_to_string(root.join(".bdl/authoring.json")).unwrap();
    assert!(!sidecar.to_lowercase().contains("source"), "{sidecar}");
    assert!(!sidecar.contains("template"), "{sidecar}");
}

/// Semantic tokens over the wire (protocol 0.21): the Code view's text and
/// a formula draft are classified by the one classifier, on the LSP
/// vocabulary, with the legend in every answer; a text that does not
/// build keeps its lexical classes; a component's body relationship is
/// classified in its own scope; the client's generation comes back.
#[test]
fn semantic_tokens_are_served_for_sources_and_formula_drafts() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("system");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("bdl.toml"),
        "schema_version = 1\nname = \"system\"\nkind = \"text\"\n",
    )
    .unwrap();
    let text = include_str!("../../bdl-syntax/test_data/valid/system.bdl");
    std::fs::write(root.join("src/system.bdl"), text).unwrap();
    let mut c = Client::spawn();
    let p = c.open(&root);
    let sources = c.sources();
    let shown = &sources.files[0];
    assert_eq!(shown.path, "src/system.bdl");

    let tokens =
        |c: &mut Client, document: pb::semantic_tokens_request::Document, text: &str| match c.call(
            Req::SemanticTokens(pb::SemanticTokensRequest {
                revision: c.last_revision,
                generation: 7,
                text: text.into(),
                document: Some(document),
            }),
        ) {
            Resp::SemanticTokens(t) => t,
            other => panic!("{other:?}"),
        };
    let classes =
        |t: &pb::SemanticTokensResponse, text: &str| -> Vec<(String, String, Vec<String>)> {
            let legend = t.legend.as_ref().unwrap();
            t.tokens
                .iter()
                .map(|tok| {
                    let mods = legend
                        .modifiers
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| tok.token_modifiers & (1 << i) != 0)
                        .map(|(_, m)| m.clone())
                        .collect();
                    (
                        text[tok.start as usize..tok.end as usize].to_owned(),
                        legend.types[tok.token_type as usize].clone(),
                        mods,
                    )
                })
                .collect()
        };
    let has = |rows: &[(String, String, Vec<String>)], text: &str, ty: &str, mods: &[&str]| {
        rows.iter().any(|(t, k, m)| {
            t == text && k == ty && m.iter().map(String::as_str).eq(mods.iter().copied())
        })
    };

    // the file as shown
    let path = pb::semantic_tokens_request::Document::Path(shown.path.clone());
    let t = tokens(&mut c, path.clone(), &shown.text);
    assert_eq!(t.generation, 7);
    assert_eq!(t.revision, c.last_revision);
    assert_eq!(t.text_len, shown.text.len() as u32);
    let legend = t.legend.as_ref().unwrap();
    assert_eq!(legend.version, bdl_ide::LEGEND_VERSION);
    assert_eq!(legend.types, bdl_ide::legend().types);
    assert_eq!(legend.modifiers, bdl_ide::legend().modifiers);
    for w in t.tokens.windows(2) {
        assert!(w[0].end <= w[1].start, "{:?} {:?}", w[0], w[1]);
    }
    let rows = classes(&t, &shown.text);
    assert!(has(&rows, "concept", "keyword", &[]), "{rows:?}");
    assert!(has(&rows, "Tilt", "type", &["declaration"]));
    assert!(
        has(&rows, "tilt", "variable", &["declaration", "source"]),
        "{rows:?}"
    );
    assert!(has(&rows, "tiltValue", "variable", &["declaration"]));
    assert!(has(&rows, "interaction", "namespace", &["declaration"]));
    assert!(has(&rows, "interaction", "namespace", &[]));
    assert!(has(&rows, "light", "variable", &["declaration", "output"]));
    assert!(has(
        &rows,
        "pwmLight",
        "variable",
        &["declaration", "device"]
    ));
    assert!(has(&rows, "AdaptiveLamp", "class", &["declaration"]));
    assert!(has(&rows, "AdaptiveLamp", "class", &[]));
    assert!(has(
        &rows,
        "lampA",
        "variable",
        &["declaration", "instance"]
    ));
    assert!(has(&rows, "dimByTilt", "function", &["declaration"]));
    assert!(has(&rows, "t", "parameter", &["declaration"]));
    assert!(has(&rows, "90", "number", &[]));
    assert!(has(&rows, "deg", "unit", &[]));
    assert!(has(&rows, "/", "operator", &[]));
    assert!(has(&rows, "// supplied from outside", "comment", &[]));
    assert!(has(&rows, "gain", "property", &["declaration"]));
    assert!(!rows
        .iter()
        .any(|(t, _, _)| t == ":" || t == "{" || t == "}"));

    // text that does not build: the lexical layer stays, and the project
    // is untouched (no revision moved, the committed sources unchanged)
    let broken = shown
        .text
        .replace("dimByTilt(t) = t / (90 deg)", "dimByTilt(t) = t / (90 deg");
    let rev = c.last_revision;
    let t = tokens(&mut c, path.clone(), &broken);
    assert_eq!(t.revision, rev);
    assert_eq!(t.text_len, broken.len() as u32);
    let rows = classes(&t, &broken);
    assert!(has(&rows, "concept", "keyword", &[]));
    assert!(has(&rows, "90", "number", &[]));
    assert!(has(&rows, "Tilt", "type", &["declaration"]));
    assert!(c.sources().files[0].text == shown.text);
    assert_eq!(c.last_revision, rev);
    // and the file's tokens after it are over the committed text again
    let t = tokens(&mut c, path.clone(), &shown.text);
    assert!(has(&classes(&t, &shown.text), "deg", "unit", &[]));

    // a formula draft in a component's body, relative to its own text
    let Resp::System(s) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    let s = s.system.unwrap();
    let lamp = &s.components[0];
    let dim = lamp
        .body
        .as_ref()
        .unwrap()
        .mappings
        .iter()
        .find(|m| m.name == "dimByTilt")
        .unwrap()
        .id;
    let draft = "clamp(t / (90 deg), 0, 1)";
    let t = tokens(
        &mut c,
        pb::semantic_tokens_request::Document::Formula(pb::FormulaDocument {
            mapping_id: dim,
            component: Some(lamp.id),
        }),
        draft,
    );
    assert_eq!(t.text_len, draft.len() as u32);
    let rows = classes(&t, draft);
    assert!(
        has(&rows, "clamp", "function", &["defaultLibrary"]),
        "{rows:?}"
    );
    assert!(has(&rows, "t", "parameter", &[]));
    assert!(has(&rows, "90", "number", &[]));
    assert!(has(&rows, "deg", "unit", &[]));
    assert!(has(&rows, "0", "number", &[]));

    // a system relationship's draft names a Source and a Rule
    let brightness = p
        .mappings
        .iter()
        .find(|m| m.name == "brightness")
        .unwrap()
        .id;
    let draft = "if tilt > 0 deg then mirror else slow";
    let t = tokens(
        &mut c,
        pb::semantic_tokens_request::Document::Formula(pb::FormulaDocument {
            mapping_id: brightness,
            component: None,
        }),
        draft,
    );
    let rows = classes(&t, draft);
    assert!(has(&rows, "if", "keyword", &[]));
    assert!(has(&rows, "tilt", "variable", &["source"]), "{rows:?}");
    assert!(has(&rows, "mirror", "variable", &[]));
    assert!(has(&rows, "slow", "variable", &[]));

    // a path the Code view may not write is refused
    let Resp::Error(e) = c.call(Req::SemanticTokens(pb::SemanticTokensRequest {
        revision: c.last_revision,
        generation: 8,
        text: String::new(),
        document: Some(pb::semantic_tokens_request::Document::Path(
            "../x.bdl".into(),
        )),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "source.invalid_path");
}

/// The Code view's IDE queries over the wire (protocol 0.22): completion,
/// hover, definition and references over the text as typed, across two
/// files and inside a component's source, and formatting as one authored
/// edit after which the tokens are over the new text.  Every answer echoes
/// the client's generation; a stale revision is answered, not refused.
#[test]
fn source_queries_answer_over_the_text_as_typed() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("ws");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("bdl.toml"),
        "schema_version = 1\nname = \"ws\"\nkind = \"text\"\n",
    )
    .unwrap();
    let a = "\
concept Tilt : Angle
concept Brightness : Scalar
clock main
mapping tilt : () -> Tilt @main
mapping dimByTilt : Tilt -> Brightness
dimByTilt(t) = clamp(t / (90 deg), 0, 1)
";
    let b = "\
mapping brightness : () -> Brightness @main
brightness() = dimByTilt(tilt)
";
    std::fs::write(root.join("src/a.bdl"), a).unwrap();
    std::fs::write(root.join("src/b.bdl"), b).unwrap();
    let mut c = Client::spawn();
    c.open(&root);
    let rev = c.last_revision;
    let at = |text: &str, needle: &str| text.find(needle).expect("needle") as u32;

    // completion in b's formula, typed further than the daemon holds
    let typed = format!("{b}mapping level : () -> Brightness @main\nlevel() = ");
    let Resp::SourceCompletion(r) = c.call(Req::SourceCompletion(pb::SourceCompletionRequest {
        revision: rev,
        generation: 3,
        path: "src/b.bdl".into(),
        text: typed.clone(),
        offset: typed.len() as u32,
    })) else {
        panic!("completion")
    };
    assert_eq!(r.generation, 3);
    assert_eq!(r.revision, rev);
    let labels: Vec<&str> = r.items.iter().map(|i| i.label.as_str()).collect();
    assert!(labels.contains(&"tilt"), "{labels:?}");
    assert!(labels.contains(&"brightness"), "{labels:?}");
    let dim = r
        .items
        .iter()
        .find(|i| i.label == "dimByTilt(Tilt)")
        .expect("rule");
    assert_eq!(dim.insert, "dimByTilt(");
    assert_eq!(dim.kind, "mapping");
    assert!(r
        .items
        .iter()
        .any(|i| i.kind == "equation" && i.insert == "clamp("));
    // the level's own name is not a candidate; the replace range is at the end
    assert!(!labels.contains(&"level"));
    assert_eq!(dim.replace_start, typed.len() as u32);

    // hover in b on the rule declared in a: the card with its role
    let Resp::DraftHover(h) = c.call(Req::SourceHover(pb::SourceHoverRequest {
        revision: rev,
        generation: 4,
        path: "src/b.bdl".into(),
        text: b.into(),
        offset: at(b, "dimByTilt") + 3,
    })) else {
        panic!("hover")
    };
    assert!(h.found);
    assert_eq!(h.title, "dimByTilt");
    assert_eq!(
        h.span.as_ref().map(|s| (s.start, s.end)),
        Some((at(b, "dimByTilt"), at(b, "dimByTilt") + 9))
    );
    assert!(
        h.details
            .iter()
            .any(|d| d.label == "role" && d.value == "Rule"),
        "{:?}",
        h.details
    );
    // hover on an equation: the library's words, no entity
    let Resp::DraftHover(h) = c.call(Req::SourceHover(pb::SourceHoverRequest {
        revision: rev,
        generation: 5,
        path: "src/a.bdl".into(),
        text: a.into(),
        offset: at(a, "clamp") + 1,
    })) else {
        panic!("hover")
    };
    assert!(h.found);
    assert_eq!(h.equation, "clamp");
    assert!(h.title.starts_with("clamp("));
    assert!(h.entity.is_none());
    // hover on nothing: not found, cleanly
    let Resp::DraftHover(h) = c.call(Req::SourceHover(pb::SourceHoverRequest {
        revision: rev,
        generation: 6,
        path: "src/a.bdl".into(),
        text: a.into(),
        offset: at(a, "90"),
    })) else {
        panic!("hover")
    };
    assert!(!h.found);

    // definition across files: `tilt` in b → its declaration in a
    let Resp::SourceLocations(d) = c.call(Req::SourceDefinition(pb::SourceDefinitionRequest {
        revision: rev,
        generation: 7,
        path: "src/b.bdl".into(),
        text: b.into(),
        offset: at(b, "tilt)"),
    })) else {
        panic!("definition")
    };
    assert_eq!(d.generation, 7);
    assert_eq!(d.title, "tilt");
    assert_eq!(d.locations.len(), 1);
    assert_eq!(d.locations[0].path, "src/a.bdl");
    assert_eq!(
        &a[d.locations[0].start as usize..d.locations[0].end as usize],
        "tilt"
    );
    // references across files: `Brightness` from a → the signatures in b
    let Resp::SourceLocations(r) = c.call(Req::SourceReferences(pb::SourceReferencesRequest {
        revision: rev,
        generation: 8,
        path: "src/a.bdl".into(),
        text: a.into(),
        offset: at(a, "Brightness"),
        include_declaration: true,
    })) else {
        panic!("references")
    };
    assert_eq!(r.title, "Brightness");
    let in_b = r.locations.iter().filter(|l| l.path == "src/b.bdl").count();
    assert_eq!(in_b, 1, "{:?}", r.locations);
    assert!(r
        .locations
        .iter()
        .any(|l| l.path == "src/a.bdl" && l.start == at(a, "Brightness")));
    // the overlay of the edited file coexists: a reference typed in b
    // that the project does not hold yet is found from a
    let Resp::SourceLocations(r) = c.call(Req::SourceReferences(pb::SourceReferencesRequest {
        revision: rev,
        generation: 9,
        path: "src/b.bdl".into(),
        text: format!("{b}mapping level : () -> Brightness @main\n"),
        offset: at(b, "Brightness"),
        include_declaration: false,
    })) else {
        panic!("references")
    };
    assert_eq!(
        r.locations.iter().filter(|l| l.path == "src/b.bdl").count(),
        2,
        "{:?}",
        r.locations
    );

    // a stale revision is answered at the current one
    let Resp::SourceLocations(d) = c.call(Req::SourceDefinition(pb::SourceDefinitionRequest {
        revision: rev + 40,
        generation: 10,
        path: "src/b.bdl".into(),
        text: b.into(),
        offset: at(b, "tilt)"),
    })) else {
        panic!("definition")
    };
    assert_eq!(d.revision, rev);
    assert_eq!(d.locations.len(), 1);

    // format: the canonical layout, then the tokens are over that text
    let messy = a
        .replace("concept Tilt : Angle", "concept  Tilt:Angle")
        .replace("mapping tilt :", "mapping   tilt  :");
    let Resp::FormatSource(f) = c.call(Req::FormatSource(pb::FormatSourceRequest {
        revision: rev,
        generation: 11,
        path: "src/a.bdl".into(),
        text: messy.clone(),
    })) else {
        panic!("format")
    };
    assert_eq!(f.generation, 11);
    assert!(f.formatted);
    assert_ne!(f.text, messy);
    assert_eq!(f.text, a);
    let Resp::SemanticTokens(t) = c.call(Req::SemanticTokens(pb::SemanticTokensRequest {
        revision: rev,
        generation: 12,
        text: f.text.clone(),
        document: Some(pb::semantic_tokens_request::Document::Path(
            "src/a.bdl".into(),
        )),
    })) else {
        panic!("tokens")
    };
    assert_eq!(t.text_len, f.text.len() as u32);
    assert!(!t.tokens.is_empty());
    // already canonical, or not parseable: unchanged, and nothing moved
    for text in [f.text.as_str(), "mapping x : (\n"] {
        let Resp::FormatSource(f2) = c.call(Req::FormatSource(pb::FormatSourceRequest {
            revision: rev,
            generation: 13,
            path: "src/a.bdl".into(),
            text: text.into(),
        })) else {
            panic!("format")
        };
        assert!(!f2.formatted);
        assert_eq!(f2.text, text);
    }
    assert_eq!(c.last_revision, rev);
    assert_eq!(c.sources().files[0].text, a);
    // the whole formatted file is one authored edit
    let applied = c.source_edit("src/a.bdl", &f.text);
    assert!(applied.accepted);
    assert!(c.last_revision > rev);

    // inside a component's source: the port, never a flattened copy
    let sys = include_str!("../../bdl-syntax/test_data/valid/system.bdl");
    std::fs::write(root.join("src/system.bdl"), sys).unwrap();
    c.call(Req::ReloadProject(pb::ReloadProjectRequest {}));
    let use_ = at(sys, "dimByTilt(tiltValue)") + "dimByTilt(".len() as u32;
    let Resp::SourceLocations(d) = c.call(Req::SourceDefinition(pb::SourceDefinitionRequest {
        revision: c.last_revision,
        generation: 14,
        path: "src/system.bdl".into(),
        text: sys.into(),
        offset: use_,
    })) else {
        panic!("definition")
    };
    assert!(!d.locations.is_empty());
    let requires = at(sys, "requires tiltValue");
    assert!(
        d.locations
            .iter()
            .all(|l| l.path == "src/system.bdl" && l.start > requires && l.start < requires + 20),
        "{:?}",
        d.locations
    );
    let Resp::DraftHover(h) = c.call(Req::SourceHover(pb::SourceHoverRequest {
        revision: c.last_revision,
        generation: 15,
        path: "src/system.bdl".into(),
        text: sys.into(),
        offset: at(sys, "dimByTilt(tiltValue)"),
    })) else {
        panic!("hover")
    };
    assert_eq!(h.title, "dimByTilt", "{h:?}");
    assert!(!h.signature.contains("lampA"), "{}", h.signature);
}
