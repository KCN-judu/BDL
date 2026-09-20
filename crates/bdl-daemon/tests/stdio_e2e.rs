#![allow(clippy::unwrap_used)]
//! End-to-end: spawn the real `bdld` binary, speak the framed protocol over
//! its stdio, and walk the first vertical-slice steps — handshake, init,
//! create concepts, create an unresolved mapping, save, reopen.

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
        Client {
            child,
            buf: BytesMut::new(),
            next_id: 1,
            last_revision: 0,
        }
    }

    fn send(&mut self, payload: Req) -> u64 {
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
        id
    }

    fn next_message(&mut self) -> pb::ServerMessage {
        loop {
            if let Some(m) = framing::decode::<pb::ServerMessage>(&mut self.buf).unwrap() {
                return m;
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
        }
    }

    /// Send a request and return its response, collecting events on the way.
    fn call(&mut self, payload: Req, events: &mut Vec<pb::Event>) -> Resp {
        let id = self.send(payload);
        loop {
            match self.next_message().payload.unwrap() {
                pb::server_message::Payload::Response(r) => {
                    assert_eq!(r.request_id, id);
                    let payload = r.payload.unwrap();
                    match &payload {
                        Resp::Project(p) => {
                            self.last_revision = p.project.as_ref().unwrap().revision
                        }
                        Resp::SystemEditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        Resp::EditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        _ => {}
                    }
                    return payload;
                }
                pb::server_message::Payload::Event(e) => events.push(e),
            }
        }
    }
}

fn project(resp: Resp) -> pb::ProjectProjection {
    match resp {
        Resp::Project(p) => p.project.unwrap(),
        Resp::EditApplied(e) => e.project.unwrap(),
        other => panic!("expected a project, got {other:?}"),
    }
}

fn edit(base: u64, op: pb::edit_op::Op) -> Req {
    Req::ApplyEdit(pb::ApplyEditRequest {
        base_revision: base,
        op: Some(pb::EditOp { op: Some(op) }),
    })
}

#[test]
fn vertical_slice_steps_1_to_12() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut events = Vec::new();
    let mut c = Client::spawn();

    // handshake
    let hs = c.call(
        Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "e2e".into(),
            client_version: "0".into(),
        }),
        &mut events,
    );
    let Resp::Handshake(h) = hs else {
        panic!("no handshake")
    };
    assert!(h.compatible);
    assert_eq!(h.protocol_version.unwrap(), bdl_protocol::PROTOCOL_VERSION);
    assert!(!h.compiler_version.is_empty());

    // editing without a project is a structured error, not a crash
    let Resp::Error(e) = c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events) else {
        panic!()
    };
    assert_eq!(e.code, "session.no_project");

    // init + subscribe
    let p = project(c.call(
        Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: "lamp".into(),
            template: None,
        }),
        &mut events,
    ));
    assert_eq!(p.revision, 0);
    c.call(
        Req::SubscribeProject(pb::SubscribeProjectRequest {}),
        &mut events,
    );

    // create Tilt, Brightness
    let p = project(c.call(
        edit(
            0,
            pb::edit_op::Op::CreateConcept(pb::CreateConcept {
                name: "Tilt".into(),
                ..Default::default()
            }),
        ),
        &mut events,
    ));
    let tilt = p.concepts[0].id;
    let p = project(c.call(
        edit(
            1,
            pb::edit_op::Op::CreateConcept(pb::CreateConcept {
                name: "Brightness".into(),
                ..Default::default()
            }),
        ),
        &mut events,
    ));
    let bright = p
        .concepts
        .iter()
        .find(|c| c.name == "Brightness")
        .unwrap()
        .id;

    // stale revision is refused
    let Resp::Error(e) = c.call(
        edit(
            0,
            pb::edit_op::Op::CreateConcept(pb::CreateConcept {
                name: "Held".into(),
                ..Default::default()
            }),
        ),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(e.code, "edit.stale_revision");

    // dimByTilt : Tilt -> Brightness, unresolved
    let resp = c.call(
        edit(
            2,
            pb::edit_op::Op::CreateMapping(pb::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Some(pb::Signature {
                    inputs: vec![tilt],
                    output: bright,
                }),
                ..Default::default()
            }),
        ),
        &mut events,
    );
    let Resp::EditApplied(applied) = resp else {
        panic!()
    };
    assert_eq!(
        applied.outcome.as_ref().unwrap().kind(),
        pb::EditKind::Refinement
    );
    let p = applied.project.unwrap();
    assert_eq!(p.revision, 3);
    assert!(p.dirty);
    let m = &p.mappings[0];
    assert_eq!(m.state(), pb::AcceptanceState::Declared);
    assert!(m.definition.is_none());

    // attach a dimension-correct formula: analysed as type-valid, with the
    // Core term visible for the explanation view
    let resp = c.call(
        edit(
            3,
            pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
                id: m.id,
                definition: Some(pb::Definition {
                    kind: Some(pb::definition::Kind::Formula("Tilt / 90 deg".into())),
                }),
            }),
        ),
        &mut events,
    );
    let Resp::EditApplied(_) = resp else {
        panic!("{resp:?}")
    };
    // Tilt/Brightness have no representation yet → open, not invalid
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    assert_eq!(a.revision, 4);
    assert_eq!(a.mappings[0].status(), pb::MappingStatus::Open);
    let open = a
        .diagnostics
        .iter()
        .find(|d| d.code == "semantic.unbound_representation")
        .expect("the open note");
    assert_eq!(open.severity(), pb::DiagnosticSeverity::Info);
    // the rule is applied by nothing yet: a note beside it, never an error
    let unapplied = a
        .diagnostics
        .iter()
        .find(|d| d.code == "reactive.rule_unapplied")
        .expect("the unapplied note");
    assert_eq!(unapplied.severity(), pb::DiagnosticSeverity::Info);
    assert_eq!(
        unapplied.message,
        "dimByTilt is a rule nothing applies yet."
    );
    // bind representations → type-valid
    for (id, rep) in [
        (
            tilt,
            pb::Representation {
                kind: Some(pb::representation::Kind::Quantity(pb::Dim {
                    angle: 1,
                    ..Default::default()
                })),
            },
        ),
        (
            bright,
            pb::Representation {
                kind: Some(pb::representation::Kind::Quantity(pb::Dim::default())),
            },
        ),
    ] {
        let base = c.last_revision;
        c.call(
            edit(
                base,
                pb::edit_op::Op::SetConceptRepresentation(pb::SetConceptRepresentation {
                    id,
                    representation: Some(rep),
                }),
            ),
            &mut events,
        );
    }
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    assert_eq!(a.revision, 6);
    // a well-typed pure mapping climbs the whole ladder: it is on no loop and
    // reads across no domain
    assert_eq!(a.mappings[0].status(), pb::MappingStatus::ClockConsistent);
    assert!(a.causal && a.clock_consistent);
    assert_eq!(a.evaluation_order, vec![m.id]);
    // nothing wrong; the one note is that nothing applies the rule yet
    assert!(a
        .diagnostics
        .iter()
        .all(|d| d.code == "reactive.rule_unapplied"));
    assert!(
        a.mappings[0].core_expr.contains("(rep #0)"),
        "{}",
        a.mappings[0].core_expr
    );
    assert!(
        a.mappings[0].core_expr.contains("mk sem#1"),
        "{}",
        a.mappings[0].core_expr
    );
    // simulate: dimByTilt is a function, so it is simply a closure at every
    // tick; the reference evaluator runs it once the design is causal
    let Resp::Simulation(sim) = c.call(
        Req::StartSimulation(pb::StartSimulationRequest {
            inputs: vec![],
            schedule: vec![],
        }),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(sim.revision, 6);
    assert_eq!(sim.next_tick, 0);
    let Resp::Simulation(sim) = c.call(
        Req::StepSimulation(pb::StepSimulationRequest { ticks: 3 }),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(sim.next_tick, 3);
    assert_eq!(sim.samples.len(), 3);
    assert_eq!(sim.samples[2].tick, 2);
    assert_eq!(sim.samples[0].values[0].rendered, "<function>");
    assert!(sim.error.is_none());
    let Resp::Simulation(sim) = c.call(
        Req::ResetSimulation(pb::ResetSimulationRequest {}),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(sim.next_tick, 0);

    // a bad formula → invalid with a spanned, product-language diagnostic
    c.call(
        edit(
            6,
            pb::edit_op::Op::ReplaceDefinition(pb::ReplaceDefinition {
                id: m.id,
                definition: Some(pb::Definition {
                    kind: Some(pb::definition::Kind::Formula("Tilt + 1 s".into())),
                }),
            }),
        ),
        &mut events,
    );
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    assert_eq!(a.mappings[0].status(), pb::MappingStatus::Invalid);
    let d = a
        .diagnostics
        .iter()
        .find(|d| d.code == "dimension.mismatch")
        .expect("the dimension error");
    // an edit discards the run: stepping needs a fresh start
    let Resp::Error(e) = c.call(
        Req::StepSimulation(pb::StepSimulationRequest { ticks: 1 }),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(e.code, "simulation.not_started");
    assert!(d.message.contains("an angle and a time"));
    assert_eq!(d.span.as_ref().map(|s| (s.start, s.end)), Some((0, 10)));
    // subscribers received an AnalysisReady per committed revision
    let ready: Vec<u64> = events
        .iter()
        .filter_map(|e| match &e.payload {
            Some(pb::event::Payload::AnalysisReady(r)) => {
                Some(r.analysis.as_ref().unwrap().revision)
            }
            _ => None,
        })
        .collect();
    assert_eq!(ready, vec![1, 2, 3, 4, 5, 6, 7]);
    // restore a valid formula for the persistence checks below
    c.call(
        edit(
            7,
            pb::edit_op::Op::ReplaceDefinition(pb::ReplaceDefinition {
                id: m.id,
                definition: Some(pb::Definition {
                    kind: Some(pb::definition::Kind::Formula("Tilt / 90 deg".into())),
                }),
            }),
        ),
        &mut events,
    );
    let rev_after_formula = c.last_revision;
    assert_eq!(
        rev_after_formula, 8,
        "revisions: init 0, two concepts, mapping, attach, two representations, replace, restore"
    );

    // duplicate name → designer-readable error with a stable code
    let Resp::Error(e) = c.call(
        edit(
            rev_after_formula,
            pb::edit_op::Op::CreateConcept(pb::CreateConcept {
                name: "Tilt".into(),
                ..Default::default()
            }),
        ),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(e.code, "edit.duplicate_concept_name", "{}", e.message);
    assert!(e.message.contains("Tilt"));

    // layout is not a revision
    c.call(
        Req::SetLayout(pb::SetLayoutRequest {
            layout: Some(pb::Layout {
                concepts: vec![pb::NodePosition {
                    id: tilt,
                    x: 1.0,
                    y: 2.0,
                }],
                mappings: vec![pb::NodePosition {
                    id: m.id,
                    x: 3.0,
                    y: 4.0,
                }],
                outputs: vec![],
                ..Default::default()
            }),
        }),
        &mut events,
    );
    let p = project(c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events));
    assert_eq!(p.revision, rev_after_formula);

    // save, close, reopen: the unresolved mapping and layout survive
    let p = project(c.call(
        Req::SaveProject(pb::SaveProjectRequest { force: false }),
        &mut events,
    ));
    assert!(!p.dirty);
    c.call(Req::CloseProject(pb::CloseProjectRequest {}), &mut events);
    let p = project(c.call(
        Req::OpenProject(pb::OpenProjectRequest {
            root_path: root.to_string_lossy().into(),
        }),
        &mut events,
    ));
    assert_eq!(p.revision, 0);
    assert_eq!(p.concepts.len(), 2);
    assert_eq!(p.mappings.len(), 1);
    assert_eq!(p.mappings[0].name, "dimByTilt");
    assert_eq!(p.mappings[0].state(), pb::AcceptanceState::Defined);
    assert_eq!(p.mappings[0].id, m.id);
    assert_eq!(p.layout.unwrap().mappings[0].x, 3.0);

    // subscribers saw every committed revision, in order
    let seen: Vec<u64> = events
        .iter()
        .filter_map(|e| match &e.payload {
            Some(pb::event::Payload::ProjectChanged(pc)) => {
                Some(pc.project.as_ref().unwrap().revision)
            }
            _ => None,
        })
        .collect();
    assert_eq!(seen, vec![1, 2, 3, 4, 5, 6, 7, 8]);

    let Resp::Ack(_) = c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events) else {
        panic!()
    };
    assert!(c.child.wait().unwrap().success());
}

/// Outputs, devices and target-relative deployment over the wire.
#[test]
fn outputs_and_deployment_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rover");
    let mut events = Vec::new();
    let mut c = Client::spawn();
    c.call(
        Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "e2e".into(),
            client_version: "0".into(),
        }),
        &mut events,
    );
    project(c.call(
        Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: "rover".into(),
            template: None,
        }),
        &mut events,
    ));
    fn apply(c: &mut Client, events: &mut Vec<pb::Event>, op: pb::edit_op::Op) -> pb::EditApplied {
        let base = c.last_revision;
        match c.call(edit(base, op), events) {
            Resp::EditApplied(e) => e,
            other => panic!("edit failed: {other:?}"),
        }
    }
    let speed = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Speed".into(),
            description: String::new(),
            representation: Some(pb::Representation {
                kind: Some(pb::representation::Kind::Quantity(pb::Dim::default())),
            }),
        }),
    )
    .outcome
    .unwrap()
    .created_concept
    .unwrap();
    let level = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "cruise".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![],
                output: speed,
            }),
            ..Default::default()
        }),
    )
    .outcome
    .unwrap()
    .created_mapping
    .unwrap();
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
            id: level,
            definition: Some(pb::Definition {
                kind: Some(pb::definition::Kind::Formula("0.5".into())),
            }),
        }),
    );
    let main = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
            name: "main".into(),
        }),
    )
    .outcome
    .unwrap()
    .created_clock
    .unwrap();
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id: level,
            clock_id: Some(main),
        }),
    );
    let motor = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateOutput(pb::CreateOutput {
            name: "motor".into(),
            description: String::new(),
            accepts: speed,
            clock_id: Some(main),
        }),
    )
    .outcome
    .unwrap()
    .created_output
    .unwrap();

    // undriven: the design is not complete, and says so without an error
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    assert!(!a.output_complete);
    assert_eq!(a.outputs.len(), 1);
    assert_eq!(a.outputs[0].state(), pb::OutputState::Undriven);
    assert!(
        a.diagnostics
            .iter()
            .any(|d| d.code == "output.missing_driver"
                && d.severity() == pb::DiagnosticSeverity::Info)
    );

    // the everyday card for an entity, and the fixes bdl-ide offers for it
    let entity = |kind: pb::entity_ref::Kind| pb::EntityRef { kind: Some(kind) };
    let Resp::DraftHover(h) = c.call(
        Req::HoverEntity(pb::HoverEntityRequest {
            revision: c.last_revision,
            entity: Some(entity(pb::entity_ref::Kind::MappingId(level))),
        }),
        &mut events,
    ) else {
        panic!("expected a hover")
    };
    assert!(h.found);
    assert_eq!(h.title, "cruise");
    assert_eq!(h.signature, "mapping cruise : () -> Speed");
    assert_eq!(h.mapping_id, level);
    assert!(!h.status.is_empty());
    let Resp::DraftHover(none) = c.call(
        Req::HoverEntity(pb::HoverEntityRequest {
            revision: c.last_revision,
            entity: Some(entity(pb::entity_ref::Kind::MappingId(999))),
        }),
        &mut events,
    ) else {
        panic!("expected a hover")
    };
    assert!(!none.found);
    let Resp::SemanticActions(acts) = c.call(
        Req::ListSemanticActions(pb::ListSemanticActionsRequest {
            revision: c.last_revision,
            entity: Some(entity(pb::entity_ref::Kind::OutputId(motor))),
        }),
        &mut events,
    ) else {
        panic!("expected actions")
    };
    // the undriven sink offers "connect a driver" as a choice among the
    // mappings that could drive it, never a guess
    let connect = acts
        .actions
        .iter()
        .find(|a| a.addresses.iter().any(|c| c == "output.missing_driver"))
        .expect("a fix for the missing driver");
    assert_eq!(
        connect.applicability(),
        pb::ActionApplicability::NeedsChoice
    );
    assert!(connect
        .options
        .iter()
        .any(|o| o
            .edit
            .as_ref()
            .unwrap()
            .op
            .as_ref()
            .is_some_and(|op| matches!(
                op,
                pb::edit_op::Op::SetMappingDrive(d) if d.id == level && d.output_id == Some(motor)
            ))));
    // a stale revision is refused like every other read of the world
    let Resp::Error(e) = c.call(
        Req::ListSemanticActions(pb::ListSemanticActionsRequest {
            revision: c.last_revision - 1,
            entity: Some(entity(pb::entity_ref::Kind::OutputId(motor))),
        }),
        &mut events,
    ) else {
        panic!("expected a refusal")
    };
    assert_eq!(e.code, "draft.stale_revision");

    let applied = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::SetMappingDrive(pb::SetMappingDrive {
            id: level,
            output_id: Some(motor),
        }),
    );
    let p = applied.project.unwrap();
    assert_eq!(p.outputs.len(), 1);
    assert_eq!(p.outputs[0].name, "motor");
    assert_eq!(p.clocks[0].name, "main");
    assert_eq!(p.mappings[0].drives_output_id, Some(motor));
    assert_eq!(p.mappings[0].clock_id, Some(main));
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    assert!(a.output_complete, "{:?}", a.diagnostics);
    assert_eq!(a.outputs[0].state(), pb::OutputState::Driven);
    assert_eq!(a.outputs[0].driver, Some(level));

    // deleting a driven output is refused with a structured error
    let base = c.last_revision;
    let Resp::Error(e) = c.call(
        edit(
            base,
            pb::edit_op::Op::DeleteOutput(pb::DeleteOutput { id: motor }),
        ),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(e.code, "edit.output_in_use");

    // a device realises the output; its pin table comes from the kind
    let dev = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateDevice(pb::CreateDevice {
            name: "driver".into(),
            kind: pb::DeviceKind::HBridgeChannel.into(),
            output_id: Some(motor),
        }),
    );
    let device = dev.outcome.unwrap().created_device.unwrap();
    let dv = &dev.project.unwrap().devices[0];
    assert_eq!(dv.kind(), pb::DeviceKind::HBridgeChannel);
    assert_eq!(dv.requirements.len(), 2);
    assert_eq!(dv.requirements[0].capability, "pwm");
    assert_eq!(dv.requirements[1].capability, "digital_out");

    let Resp::Targets(t) = c.call(Req::ListTargets(pb::ListTargetsRequest {}), &mut events) else {
        panic!()
    };
    let ids: Vec<&str> = t.targets.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["arduino_nano", "big_board", "rp2040_pico"]);

    let deploy = |c: &mut Client, events: &mut Vec<pb::Event>, target: &str| match c.call(
        Req::AnalyzeDeployment(pb::AnalyzeDeploymentRequest {
            target_id: target.into(),
            revision: None,
        }),
        events,
    ) {
        Resp::Deployment(d) => d.deployment.unwrap(),
        other => panic!("{other:?}"),
    };
    let d = deploy(&mut c, &mut events, "arduino_nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Feasible);
    assert_eq!(d.revision, c.last_revision);
    assert_eq!(d.assignment.len(), 2);
    assert_eq!(d.assignment[0].resource, "D3");
    // no realization chosen yet: placed by kind, told so, not blocked
    let codes: Vec<&str> = d.diagnostics.iter().map(|x| x.code.as_str()).collect();
    assert_eq!(codes, ["deploy.realization_unspecified"]);
    assert!(d.deployable);
    let r = &d.realizations[0];
    assert_eq!(r.status(), pb::RealizationStatus::NotChosen);
    assert_eq!(
        (r.device_name.as_str(), r.output_name.as_str()),
        ("driver", "motor")
    );
    assert!(r.hardware_placed && !r.encoder_well_formed);
    let fits: Vec<(&str, bool)> = r
        .candidates
        .iter()
        .map(|p| (p.id.as_str(), p.compatible))
        .collect();
    assert!(fits.contains(&("hbridge_signed", true)) && fits.contains(&("gpio_level", false)));

    // choosing a realization is one deployment edit: the profile and the
    // kind it needs land together, the view says the encoding is valid,
    // and the semantic analysis is what it was
    let before = match c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) {
        Resp::Analysis(a) => a.analysis.unwrap(),
        other => panic!("{other:?}"),
    };
    let applied = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::SetDeviceRealization(pb::SetDeviceRealization {
            id: device,
            profile_id: Some("hbridge_signed".into()),
            kind: pb::DeviceKind::HBridgeChannel.into(),
        }),
    );
    let dv = &applied.project.unwrap().devices[0];
    assert_eq!(dv.realization.as_deref(), Some("hbridge_signed"));
    assert_eq!(dv.kind(), pb::DeviceKind::HBridgeChannel);
    let d = deploy(&mut c, &mut events, "arduino_nano");
    assert!(d.diagnostics.is_empty(), "{:?}", d.diagnostics);
    let r = &d.realizations[0];
    assert_eq!(r.status(), pb::RealizationStatus::EncodingValid);
    assert!(r.encoder_well_formed && r.representation_fits && r.hardware_placed);
    let after = match c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) {
        Resp::Analysis(a) => a.analysis.unwrap(),
        other => panic!("{other:?}"),
    };
    assert_eq!(after.diagnostics, before.diagnostics);
    assert_eq!(after.mappings, before.mappings);
    assert_eq!(after.outputs, before.outputs);
    // a profile that does not fit is refused as such, and blocks deployment
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::SetDeviceRealization(pb::SetDeviceRealization {
            id: device,
            profile_id: Some("gpio_level".into()),
            kind: pb::DeviceKind::DigitalOutput.into(),
        }),
    );
    let d = deploy(&mut c, &mut events, "arduino_nano");
    assert_eq!(
        d.realizations[0].status(),
        pb::RealizationStatus::Incompatible
    );
    assert!(!d.deployable);
    assert_eq!(d.missing[0].kind(), pb::MissingKind::RealizationInvalid);
    assert_eq!(d.diagnostics[0].code, "deploy.realization_incompatible");
    // back to the H-bridge kind, no profile: as at the start
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::SetDeviceRealization(pb::SetDeviceRealization {
            id: device,
            profile_id: None,
            kind: pb::DeviceKind::HBridgeChannel.into(),
        }),
    );
    let Resp::Error(e) = c.call(
        Req::AnalyzeDeployment(pb::AnalyzeDeploymentRequest {
            target_id: "toaster".into(),
            revision: None,
        }),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(e.code, "deploy.unknown_target");

    // pin the PWM line by hand to a pin that cannot do PWM: infeasible on
    // the Nano, with the reason, while the semantic analysis is unchanged
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
            id: device,
            index: 0,
            resource: Some("D4".into()),
        }),
    );
    let d = deploy(&mut c, &mut events, "arduino_nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Infeasible);
    let dead = d.dead_end.unwrap();
    assert_eq!(
        dead.reason,
        Some(pb::dead_end::Reason::FixedUnavailable("D4".into()))
    );
    assert_eq!(d.diagnostics[0].code, "deploy.infeasible");
    assert!(d.diagnostics[0].message.contains("D4 cannot carry driver"));
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    assert!(a.analysis.unwrap().output_complete);

    // the whole thing survives a save/reopen
    project(c.call(
        Req::SaveProject(pb::SaveProjectRequest { force: false }),
        &mut events,
    ));
    c.call(Req::CloseProject(pb::CloseProjectRequest {}), &mut events);
    let p = project(c.call(
        Req::OpenProject(pb::OpenProjectRequest {
            root_path: root.to_string_lossy().into(),
        }),
        &mut events,
    ));
    assert_eq!(p.devices[0].fixed_pins[0].resource, "D4");
    assert_eq!(p.outputs[0].clock_id, Some(main));
    let Resp::Ack(_) = c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events) else {
        panic!()
    };
    assert!(c.child.wait().unwrap().success());
}

/// A definition draft is checked by the same pipeline as a commit, tagged
/// with the revision it was checked against, and never touches the project.
#[test]
fn definition_drafts_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut events = Vec::new();
    let mut c = Client::spawn();
    c.call(
        Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "e2e".into(),
            client_version: "0".into(),
        }),
        &mut events,
    );
    project(c.call(
        Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: "lamp".into(),
            template: None,
        }),
        &mut events,
    ));
    fn apply(c: &mut Client, events: &mut Vec<pb::Event>, op: pb::edit_op::Op) -> pb::EditApplied {
        let base = c.last_revision;
        match c.call(edit(base, op), events) {
            Resp::EditApplied(e) => e,
            other => panic!("edit failed: {other:?}"),
        }
    }
    fn quantity(dim: pb::Dim) -> Option<pb::Representation> {
        Some(pb::Representation {
            kind: Some(pb::representation::Kind::Quantity(dim)),
        })
    }
    let tilt = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Tilt".into(),
            description: String::new(),
            representation: quantity(pb::Dim {
                angle: 1,
                ..Default::default()
            }),
        }),
    )
    .outcome
    .unwrap()
    .created_concept
    .unwrap();
    let brightness = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Brightness".into(),
            description: String::new(),
            representation: quantity(pb::Dim::default()),
        }),
    )
    .outcome
    .unwrap()
    .created_concept
    .unwrap();
    let mapping = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![tilt],
                output: brightness,
            }),
            ..Default::default()
        }),
    )
    .outcome
    .unwrap()
    .created_mapping
    .unwrap();
    let revision = c.last_revision;

    let draft =
        |c: &mut Client, events: &mut Vec<pb::Event>, rev: u64, id: u64, gen: u64, src: &str| {
            c.call(
                Req::AnalyzeDefinitionDraft(pb::AnalyzeDefinitionDraftRequest {
                    component: None,
                    revision: rev,
                    mapping_id: id,
                    generation: gen,
                    source: src.into(),
                }),
                events,
            )
        };

    // valid draft: the full ladder, echoed generation and revision
    let Resp::DefinitionDraft(d) =
        draft(&mut c, &mut events, revision, mapping, 7, "Tilt / 90 deg")
    else {
        panic!("expected a draft analysis")
    };
    assert_eq!(d.revision, revision);
    assert_eq!(d.mapping_id, mapping);
    assert_eq!(d.generation, 7);
    assert!(d.parse_ok);
    let a = d.analysis.unwrap();
    assert_eq!(a.status(), pb::MappingStatus::ClockConsistent);
    // the draft's own verdict is clean; the mapping's list keeps the note
    // that nothing applies the rule, the same on every surface
    assert!(a
        .diagnostics
        .iter()
        .all(|d| d.code == "reactive.rule_unapplied"));
    assert!(!a.core_expr.is_empty());

    // invalid draft: the dimension diagnostic with a span into the draft source
    let src = "Tilt + 1 s";
    let Resp::DefinitionDraft(d) = draft(&mut c, &mut events, revision, mapping, 8, src) else {
        panic!("expected a draft analysis")
    };
    let a = d.analysis.unwrap();
    assert_eq!(a.status(), pb::MappingStatus::Invalid);
    let dim = a
        .diagnostics
        .iter()
        .find(|d| d.code == "dimension.mismatch")
        .expect("dimension diagnostic");
    let span = dim.span.unwrap();
    assert_eq!(&src[span.start as usize..span.end as usize], src);

    // syntax error: parse_ok false, still a normal response
    let Resp::DefinitionDraft(d) = draft(&mut c, &mut events, revision, mapping, 9, "Tilt /")
    else {
        panic!("expected a draft analysis")
    };
    assert!(!d.parse_ok);
    assert_eq!(d.analysis.unwrap().status(), pb::MappingStatus::Invalid);

    // completion and hover run over the same overlay: inputs and units are
    // offered, and the concept under the cursor is explained by bdl-ide
    let Resp::DraftCompletion(comp) = c.call(
        Req::CompleteDefinitionDraft(pb::CompleteDefinitionDraftRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "Ti".into(),
            offset: 2,
        }),
        &mut events,
    ) else {
        panic!("expected completions")
    };
    assert_eq!(comp.revision, revision);
    let tilt_item = comp
        .items
        .iter()
        .find(|i| i.label == "Tilt")
        .expect("the input Tilt is offered");
    assert_eq!(tilt_item.kind, "input");
    assert_eq!((tilt_item.replace_start, tilt_item.replace_end), (0, 2));
    let Resp::DraftHover(h) = c.call(
        Req::HoverDefinitionDraft(pb::HoverDefinitionDraftRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "Tilt / 90 deg".into(),
            offset: 1,
        }),
        &mut events,
    ) else {
        panic!("expected a hover")
    };
    assert!(h.found);
    assert_eq!(h.concept_id, Some(tilt));
    assert_eq!(h.title, "Tilt");
    assert_eq!(h.span.unwrap(), pb::SourceSpan { start: 0, end: 4 });
    let Resp::DraftHover(none) = c.call(
        Req::HoverDefinitionDraft(pb::HoverDefinitionDraftRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "Tilt / 90 deg".into(),
            offset: 8,
        }),
        &mut events,
    ) else {
        panic!("expected a hover")
    };
    assert!(!none.found, "a number is not an entity");

    // the Formula Composer (0.12): the draft analysis carries the
    // projection of the same world, a slot says what it expects and what
    // fits, and a structured action answers with the text it makes
    let Resp::DefinitionDraft(d) = draft(&mut c, &mut events, revision, mapping, 10, "Tilt / ?")
    else {
        panic!("expected a draft analysis")
    };
    let p = d.projection.expect("projection with the draft");
    assert!(!p.complete && p.parse_ok);
    assert_eq!(p.slots, vec!["r.1".to_string()]);
    let tree = p.root.as_ref().expect("tree");
    assert_eq!((tree.kind.as_str(), tree.name.as_str()), ("binary", "/"));
    assert_eq!(tree.children[0].kind, "reference");
    assert_eq!(tree.children[1].kind, "slot");
    assert_eq!(
        tree.children[1]
            .expected
            .as_ref()
            .map(|t| t.description.as_str()),
        Some("an angle")
    );
    assert!(tree.children[1]
        .diagnostics
        .iter()
        .any(|d| d.code == "formula.slot.empty"));
    let Resp::FormulaSlot(slot) = c.call(
        Req::GetFormulaSlot(pb::GetFormulaSlotRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "Tilt / ?".into(),
            node_id: "r.1".into(),
        }),
        &mut events,
    ) else {
        panic!("expected a slot")
    };
    let symbols: Vec<&str> = slot.units.iter().map(|u| u.symbol.as_str()).collect();
    assert_eq!(symbols, vec!["rad", "deg", "turn"]);
    assert!(slot.explanation.starts_with("Expected: an angle"));
    assert!(slot.references.iter().any(|r| r.label == "Tilt"));
    assert!(slot.equations.iter().any(|e| e.name == "clamp"));
    let Resp::ComposeFormula(composed) = c.call(
        Req::ComposeFormula(pb::ComposeFormulaRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "Tilt / 180 deg".into(),
            action: Some(pb::ComposeAction {
                node_id: "r.1".into(),
                action: Some(pb::compose_action::Action::SetUnit(pb::ComposeSetUnit {
                    unit_id: "angle.rad".into(),
                    preserve_value: true,
                })),
            }),
        }),
        &mut events,
    ) else {
        panic!("expected a composed formula")
    };
    assert_eq!(composed.source, "Tilt / 3.141592653589793 rad");
    assert_eq!(composed.edits.len(), 1);
    assert_eq!((composed.edits[0].start, composed.edits[0].end), (7, 14));
    // the committed definition projects without a draft; a stale revision is refused
    let Resp::FormulaProjection(fp) = c.call(
        Req::GetFormulaProjection(pb::GetFormulaProjectionRequest {
            component: None,
            revision,
            mapping_id: mapping,
        }),
        &mut events,
    ) else {
        panic!("expected a projection")
    };
    assert_eq!(fp.projection.unwrap().source, "Tilt / 180 deg");
    let Resp::Error(e) = c.call(
        Req::GetFormulaProjection(pb::GetFormulaProjectionRequest {
            component: None,
            revision: revision + 99,
            mapping_id: mapping,
        }),
        &mut events,
    ) else {
        panic!("expected a refusal")
    };
    assert_eq!(e.code, "draft.stale_revision");

    // 0.26: the structure on the wire — a composite unit literal with its
    // canonical spelling and display, roles, a caret moved, keyboard
    // insertion, completion at a caret, signature help, the render, the
    // value categories with their unit candidates
    let Resp::DefinitionDraft(d) = draft(
        &mut c,
        &mut events,
        revision,
        mapping,
        11,
        "clamp(Tilt / (180 deg per s * 2 s), ?, 1)",
    ) else {
        panic!("expected a draft analysis")
    };
    let tree = d.projection.unwrap().root.unwrap();
    assert_eq!(tree.children[0].role, "argument 1");
    assert_eq!(tree.children[0].children[1].role, "denominator");
    let lit = &tree.children[0].children[1].children[0];
    assert_eq!(
        (lit.kind.as_str(), lit.unit.as_str()),
        ("quantity", "deg per s")
    );
    assert_eq!(
        (lit.unit_source.as_str(), lit.unit_display.as_str()),
        ("deg per s", "deg/s")
    );
    assert!(lit.unit_id.is_empty());
    assert_eq!(tree.append_at, Some(tree.range.as_ref().unwrap().end - 1));
    let Resp::NavigateFormula(nav) = c.call(
        Req::NavigateFormula(pb::NavigateFormulaRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "clamp(Tilt / (180 deg per s * 2 s), ?, 1)".into(),
            node_id: "r".into(),
            side: pb::CaretSide::Before.into(),
            motion: pb::CaretMotion::NextSlot.into(),
        }),
        &mut events,
    ) else {
        panic!("expected a caret")
    };
    assert_eq!(
        (nav.node_id.as_str(), nav.side(), nav.offset),
        ("r.1", pb::CaretSide::Before, 36)
    );
    let Resp::ComposeFormula(inserted) = c.call(
        Req::ComposeFormula(pb::ComposeFormulaRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "?".into(),
            action: Some(pb::ComposeAction {
                node_id: "r".into(),
                action: Some(pb::compose_action::Action::Insert(pb::ComposeInsert {
                    side: pb::CaretSide::Before.into(),
                    text: "clamp".into(),
                })),
            }),
        }),
        &mut events,
    ) else {
        panic!("expected a composed formula")
    };
    assert_eq!(inserted.source, "clamp(?, ?, ?)");
    assert_eq!(inserted.select, "r.0");
    let Resp::DraftCompletion(caret) = c.call(
        Req::CompleteFormulaCaret(pb::CompleteFormulaCaretRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "?".into(),
            node_id: "r".into(),
            side: pb::CaretSide::Before.into(),
            prefix: "cl".into(),
        }),
        &mut events,
    ) else {
        panic!("expected completions")
    };
    let clamp = caret
        .items
        .iter()
        .find(|i| i.label.starts_with("clamp"))
        .expect("clamp at the caret");
    assert_eq!(clamp.structured_insert, "clamp(?, ?, ?)");
    assert_eq!((clamp.replace_start, clamp.replace_end), (0, 0));
    let Resp::FormulaSignature(sig) = c.call(
        Req::GetFormulaSignature(pb::GetFormulaSignatureRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "clamp(Tilt, ?, 1)".into(),
            node_id: "r.1".into(),
        }),
        &mut events,
    ) else {
        panic!("expected signature help")
    };
    assert!(sig.found);
    assert_eq!((sig.name.as_str(), sig.active), ("clamp", Some(1)));
    assert_eq!(sig.parameters.len(), 3);
    // the render is of the saved formula — nothing is saved yet, so it is
    // empty while a draft is open; after a commit it reads the committed
    // text, not the draft
    let Resp::FormulaRender(rendered) = c.call(
        Req::GetFormulaRender(pb::GetFormulaRenderRequest {
            component: None,
            revision,
            mapping_id: mapping,
        }),
        &mut events,
    ) else {
        panic!("expected a render")
    };
    assert!(rendered.fragments.is_empty());
    let saved = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "saved".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![tilt],
                output: brightness,
            }),
            definition: Some(pb::Definition {
                kind: Some(pb::definition::Kind::Formula("Tilt / 90 deg".into())),
            }),
            clock_id: None,
        }),
    )
    .outcome
    .unwrap()
    .created_mapping
    .unwrap();
    let Resp::FormulaRender(rendered) = c.call(
        Req::GetFormulaRender(pb::GetFormulaRenderRequest {
            component: None,
            revision: c.last_revision,
            mapping_id: saved,
        }),
        &mut events,
    ) else {
        panic!("expected a render")
    };
    let kinds: Vec<&str> = rendered.fragments.iter().map(|f| f.kind.as_str()).collect();
    assert_eq!(kinds, ["reference", "operator", "number", "unit"]);
    assert_eq!(rendered.references, vec!["Tilt"]);
    assert_eq!(rendered.compact, "Tilt / 90 deg");
    // back to the world the rest of this test expects
    let undone = c.call(Req::Undo(pb::UndoRequest {}), &mut events);
    assert!(
        matches!(undone, Resp::EditApplied(_) | Resp::SystemEditApplied(_)),
        "{undone:?}"
    );
    assert_eq!(c.last_revision, revision + 2, "one edit, one undo");
    let Resp::ValueCategories(cats) = c.call(
        Req::ListValueCategories(pb::ListValueCategoriesRequest {}),
        &mut events,
    ) else {
        panic!("expected the value categories")
    };
    let angular = cats
        .categories
        .iter()
        .find(|k| k.id == "angular_velocity")
        .expect("angular velocity");
    assert_eq!(angular.display_name, "angular velocity");
    assert_eq!(angular.type_name, "AngularVelocity");
    let preferred = angular.preferred_unit.as_ref().unwrap();
    assert_eq!(
        (preferred.source.as_str(), preferred.display.as_str()),
        ("rad per s", "rad/s")
    );
    let offered: Vec<&str> = angular.units.iter().map(|u| u.source.as_str()).collect();
    assert_eq!(
        offered,
        ["rad per s", "deg per s", "turn per s", "deg per min"]
    );
    assert!(cats
        .categories
        .iter()
        .any(|k| k.id == "boolean" && k.type_name == "Bool"));
    assert!(cats.categories.iter().any(|k| k.id == "count"));
    let Resp::ConceptTemplates(ct) = c.call(
        Req::ListConceptTemplates(pb::ListConceptTemplatesRequest {}),
        &mut events,
    ) else {
        panic!("expected the templates")
    };
    let speed = ct.quantities.iter().find(|q| q.id == "speed").unwrap();
    assert_eq!(speed.unit, "m/s", "the display, as 0.25 clients read it");
    assert_eq!(speed.preferred_unit.as_ref().unwrap().source, "m per s");

    // an open draft: a concept without a representation is Open, not an error
    let warmth = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Warmth".into(),
            description: String::new(),
            representation: None,
        }),
    )
    .outcome
    .unwrap()
    .created_concept
    .unwrap();
    let open_mapping = apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "byWarmth".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![warmth],
                output: brightness,
            }),
            ..Default::default()
        }),
    )
    .outcome
    .unwrap()
    .created_mapping
    .unwrap();
    let revision = c.last_revision;
    let Resp::DefinitionDraft(d) = draft(
        &mut c,
        &mut events,
        revision,
        open_mapping,
        12,
        "Warmth / 2",
    ) else {
        panic!("expected a draft analysis")
    };
    assert!(d.parse_ok);
    let a = d.analysis.unwrap();
    assert_eq!(a.status(), pb::MappingStatus::Open);
    assert!(a
        .diagnostics
        .iter()
        .all(|d| d.severity() != pb::DiagnosticSeverity::Error));
    assert!(a
        .diagnostics
        .iter()
        .any(|d| d.code == "semantic.unbound_representation"));

    // discarding the draft: the mapping is judged by its committed
    // definition again (none: Declared) — and discarding twice is fine
    c.call(
        Req::DiscardDefinitionDraft(pb::DiscardDefinitionDraftRequest {
            component: None,
            mapping_id: open_mapping,
        }),
        &mut events,
    );
    let Resp::Ack(_) = c.call(
        Req::DiscardDefinitionDraft(pb::DiscardDefinitionDraftRequest {
            component: None,
            mapping_id: open_mapping,
        }),
        &mut events,
    ) else {
        panic!("discard is idempotent")
    };
    let Resp::DraftHover(h) = c.call(
        Req::HoverDefinitionDraft(pb::HoverDefinitionDraftRequest {
            component: None,
            revision,
            mapping_id: mapping,
            source: "Tilt".into(),
            offset: 0,
        }),
        &mut events,
    ) else {
        panic!("expected a hover")
    };
    assert!(h.found);

    // deleting the mapping prunes its overlay: a later draft for it is refused
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::DeleteMapping(pb::DeleteMapping { id: open_mapping }),
    );
    let revision = c.last_revision;
    let Resp::Error(e) = draft(&mut c, &mut events, revision, open_mapping, 13, "1") else {
        panic!("expected an error")
    };
    assert_eq!(e.code, "draft.unknown_mapping");

    // stale revision and unknown mapping are refused with stable codes
    let Resp::Error(e) = draft(&mut c, &mut events, revision - 1, mapping, 10, "1") else {
        panic!("expected an error")
    };
    assert_eq!(e.code, "draft.stale_revision");
    let Resp::Error(e) = draft(&mut c, &mut events, revision, 999, 11, "1") else {
        panic!("expected an error")
    };
    assert_eq!(e.code, "draft.unknown_mapping");

    // nothing was committed: same revision, still unresolved, nothing to undo
    let p = project(c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events));
    assert_eq!(p.revision, revision);
    assert!(p.mappings[0].definition.is_none());
    assert!(events.iter().all(|e| !matches!(
        e.payload,
        Some(pb::event::Payload::ProjectChanged(_)) | Some(pb::event::Payload::AnalysisReady(_))
    )));

    // committing the draft, saving and reopening keeps the source exactly
    apply(
        &mut c,
        &mut events,
        pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
            id: mapping,
            definition: Some(pb::Definition {
                kind: Some(pb::definition::Kind::Formula("Tilt / 90 deg".into())),
            }),
        }),
    );
    project(c.call(
        Req::SaveProject(pb::SaveProjectRequest { force: false }),
        &mut events,
    ));
    c.call(Req::CloseProject(pb::CloseProjectRequest {}), &mut events);
    let p = project(c.call(
        Req::OpenProject(pb::OpenProjectRequest {
            root_path: root.to_string_lossy().into(),
        }),
        &mut events,
    ));
    assert_eq!(
        p.mappings[0].definition.as_ref().unwrap().kind,
        Some(pb::definition::Kind::Formula("Tilt / 90 deg".into()))
    );
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!("expected an analysis")
    };
    assert_eq!(
        a.analysis.unwrap().mappings[0].status(),
        pb::MappingStatus::ClockConsistent
    );
    // no phantom overlay survives a reopen: hovering the committed text
    // without setting a draft would need one, and there is none — a fresh
    // draft is what the request sets, and it is judged from scratch
    let Resp::DefinitionDraft(d) = draft(&mut c, &mut events, p.revision, mapping, 1, "Tilt +")
    else {
        panic!("expected a draft analysis")
    };
    assert!(!d.parse_ok);
    c.call(
        Req::DiscardDefinitionDraft(pb::DiscardDefinitionDraftRequest {
            component: None,
            mapping_id: mapping,
        }),
        &mut events,
    );
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!("expected an analysis")
    };
    assert_eq!(
        a.analysis.unwrap().mappings[0].status(),
        pb::MappingStatus::ClockConsistent,
        "the committed analysis never sees drafts"
    );

    c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events);
    assert!(c.child.wait().unwrap().success());
}

#[test]
fn concept_templates_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut events = Vec::new();
    let mut c = Client::spawn();
    c.call(
        Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "e2e".into(),
            client_version: "0".into(),
        }),
        &mut events,
    );

    // The libraries are served before any project is open: the embedded
    // standard library, template for template, and the fixture library
    // `BDL_LIBRARIES` names beside it.
    let listed = match c.call(
        Req::ListConceptTemplates(pb::ListConceptTemplatesRequest {}),
        &mut events,
    ) {
        Resp::ConceptTemplates(t) => t,
        other => panic!("expected templates, got {other:?}"),
    };
    let std = bdl_library::Library::standard();
    assert_eq!(listed.libraries.len(), 2);
    assert_eq!(listed.libraries[0].id, "std");
    assert_eq!(listed.libraries[1].id, "fx");
    assert_eq!(listed.libraries[0].version, std.info.version);
    let served: Vec<&str> = listed.libraries[0]
        .templates
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    let embedded: Vec<&str> = std.templates().iter().map(|t| t.id.as_str()).collect();
    assert_eq!(served, embedded);
    let light = listed.libraries[1]
        .templates
        .iter()
        .find(|t| t.id == "fx.ambient_light")
        .unwrap();
    assert_eq!(light.type_name, "Illuminance");
    assert_eq!(light.unit, "lx");
    assert_eq!(light.role_hint(), pb::RoleHint::Input);
    assert!(listed
        .quantities
        .iter()
        .any(|q| q.type_name == "Illuminance"));

    // Instantiating twice yields two concepts with distinct ids and free
    // names; both carry the template's defaults independently.
    project(c.call(
        Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: "lamp".into(),
            template: None,
        }),
        &mut events,
    ));
    let mut ids = Vec::new();
    for _ in 0..2 {
        let base = c.last_revision;
        let applied = match c.call(
            Req::InstantiateConceptTemplate(pb::InstantiateConceptTemplateRequest {
                component: None,
                base_revision: base,
                template_id: "fx.temperature".into(),
                name: None,
                ..Default::default()
            }),
            &mut events,
        ) {
            // every project is a behaviour system (ADR-0023): the answer
            // is the system's, with the flat outcome inside
            Resp::SystemEditApplied(e) => e,
            other => panic!("instantiate failed: {other:?}"),
        };
        ids.push(
            applied
                .outcome
                .unwrap()
                .inner
                .unwrap()
                .created_concept
                .unwrap(),
        );
    }
    assert_ne!(ids[0], ids[1]);
    let p = project(c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events));
    let names: Vec<&str> = p.concepts.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(names, vec!["Temperature", "Temperature2"]);
    for x in &p.concepts {
        match &x.representation.as_ref().unwrap().kind {
            Some(pb::representation::Kind::Quantity(d)) => assert_eq!(d.temperature, 1),
            other => panic!("expected a temperature, got {other:?}"),
        }
    }

    // A chosen name and an unknown template.
    let base = c.last_revision;
    let named = match c.call(
        Req::InstantiateConceptTemplate(pb::InstantiateConceptTemplateRequest {
            component: None,
            base_revision: base,
            template_id: "fx.temperature".into(),
            name: Some("OvenTemperature".into()),
            ..Default::default()
        }),
        &mut events,
    ) {
        Resp::SystemEditApplied(e) => e.project.unwrap(),
        other => panic!("instantiate failed: {other:?}"),
    };
    assert!(named.concepts.iter().any(|x| x.name == "OvenTemperature"));
    match c.call(
        Req::InstantiateConceptTemplate(pb::InstantiateConceptTemplateRequest {
            component: None,
            base_revision: c.last_revision,
            template_id: "std.nope".into(),
            name: None,
            ..Default::default()
        }),
        &mut events,
    ) {
        Resp::Error(e) => assert_eq!(e.code, "library.unknown_template"),
        other => panic!("expected an error, got {other:?}"),
    }
    c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events);
}

/// The Standard Library as items (0.17): every concept template is a
/// Concept item and the eight Source items sit beside them.  A Source
/// item is one transaction: a concept and an unresolved `() -> Value`
/// relationship with fresh identities, one undo removes both, a redo
/// brings them back with the same identities, a collision moves to the
/// next free names, an unknown id is refused, and the project saved and
/// reopened holds the ordinary objects — the relationship a simulation
/// input, the source text in the preferred spelling.
#[test]
fn library_items_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rig");
    let mut events = Vec::new();
    let mut c = Client::spawn();
    c.call(
        Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "e2e".into(),
            client_version: "0".into(),
        }),
        &mut events,
    );
    let listed = match c.call(
        Req::ListLibraryItems(pb::ListLibraryItemsRequest {}),
        &mut events,
    ) {
        Resp::LibraryItems(l) => l,
        other => panic!("expected items, got {other:?}"),
    };
    let std = bdl_library::Library::standard();
    assert_eq!(listed.libraries.len(), 2);
    assert_eq!(listed.libraries[0].items.len(), std.items().len());
    assert!(
        listed.libraries[0]
            .items
            .iter()
            .all(|i| i.category == "concept"),
        "the standard library is value categories only"
    );
    let items = &listed.libraries[1].items;
    let concepts = items.iter().filter(|i| i.category == "concept").count();
    let sources: Vec<&pb::LibraryItemView> =
        items.iter().filter(|i| i.category == "source").collect();
    assert_eq!(concepts, 3);
    assert_eq!(sources.len(), 5);
    // a Concept item carries its template view; a Source item what it creates
    let temp = items.iter().find(|i| i.id == "fx.temperature").unwrap();
    assert_eq!(temp.concept.as_ref().unwrap().type_name, "Temperature");
    assert_eq!(temp.creates.len(), 1);
    let sensor = items
        .iter()
        .find(|i| i.id == "fx.source.temperature")
        .unwrap();
    assert_eq!(sensor.display_name, "Temperature Input");
    assert_eq!(sensor.creates.len(), 2);
    assert_eq!(
        (
            sensor.creates[0].kind.as_str(),
            sensor.creates[0].name.as_str(),
            sensor.creates[0].type_name.as_str()
        ),
        ("concept", "Temperature", "Temperature")
    );
    assert_eq!(
        (
            sensor.creates[1].kind.as_str(),
            sensor.creates[1].name.as_str(),
            sensor.creates[1].signature.as_str()
        ),
        ("mapping", "temperatureInput", "() -> Temperature")
    );
    for s in &sources {
        assert!(s.creates[1].signature.starts_with("() -> "), "{}", s.id);
    }

    project(c.call(
        Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: "rig".into(),
            template: None,
        }),
        &mut events,
    ));
    let insert =
        |c: &mut Client, events: &mut Vec<pb::Event>, id: &str, names: Vec<(&str, &str)>| {
            let base = c.last_revision;
            match c.call(
                Req::InstantiateLibraryItem(pb::InstantiateLibraryItemRequest {
                    base_revision: base,
                    item_id: id.into(),
                    names: names
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    component: None,
                }),
                events,
            ) {
                Resp::SystemEditApplied(e) => e,
                other => panic!("instantiate failed: {other:?}"),
            }
        };
    // every Source item, each one revision with both objects created
    let before = c.last_revision;
    let mut created = Vec::new();
    for s in &sources {
        let e = insert(&mut c, &mut events, &s.id, vec![]);
        let inner = e.outcome.unwrap().inner.unwrap();
        created.push((
            s.id.clone(),
            inner.created_concept.unwrap(),
            inner.created_mapping.unwrap(),
        ));
    }
    assert!(c.last_revision > before);
    let p = project(c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events));
    assert_eq!(p.concepts.len(), 5);
    assert_eq!(p.mappings.len(), 5);
    let mapping_names: Vec<&str> = p.mappings.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(
        mapping_names,
        vec![
            "temperatureInput",
            "tiltInput",
            "distanceInput",
            "analogInput",
            "externalInput"
        ]
    );
    for m in &p.mappings {
        assert!(
            m.signature.as_ref().unwrap().inputs.is_empty(),
            "{}",
            m.name
        );
        assert!(
            m.definition.is_none(),
            "{}: a source stays unresolved",
            m.name
        );
    }
    // the relationship produces the concept created with it
    for (_, concept, mapping) in &created {
        let m = p.mappings.iter().find(|m| m.id == *mapping).unwrap();
        assert_eq!(m.signature.as_ref().unwrap().output, *concept);
        assert!(p.concepts.iter().any(|x| x.id == *concept));
    }
    // Analog Input and External Value leave the value form open
    for name in ["AnalogValue", "ExternalValue"] {
        let x = p.concepts.iter().find(|x| x.name == name).unwrap();
        assert!(x.representation.is_none(), "{name}");
    }
    // a collision: the next free names, on both kinds; a chosen name wins
    let e = insert(&mut c, &mut events, "fx.source.temperature", vec![]);
    let p = e.project.unwrap();
    assert!(p.concepts.iter().any(|x| x.name == "Temperature2"));
    assert!(p.mappings.iter().any(|m| m.name == "temperatureInput2"));
    let e = insert(
        &mut c,
        &mut events,
        "fx.source.temperature",
        vec![("value", "OvenTemp"), ("source", "ovenInput")],
    );
    let p = e.project.unwrap();
    assert!(p.concepts.iter().any(|x| x.name == "OvenTemp"));
    assert!(p.mappings.iter().any(|m| m.name == "ovenInput"));
    // one undo removes the whole item; a redo brings it back, same ids
    let oven = p
        .mappings
        .iter()
        .find(|m| m.name == "ovenInput")
        .unwrap()
        .id;
    let oven_temp = p.concepts.iter().find(|x| x.name == "OvenTemp").unwrap().id;
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {}), &mut events) else {
        panic!("undo")
    };
    let p = u.project.unwrap();
    assert!(!p.mappings.iter().any(|m| m.name == "ovenInput"));
    assert!(!p.concepts.iter().any(|x| x.name == "OvenTemp"));
    let Resp::SystemEditApplied(r) = c.call(Req::Redo(pb::RedoRequest {}), &mut events) else {
        panic!("redo")
    };
    let p = r.project.unwrap();
    assert_eq!(
        p.mappings
            .iter()
            .find(|m| m.name == "ovenInput")
            .map(|m| m.id),
        Some(oven)
    );
    assert_eq!(
        p.concepts
            .iter()
            .find(|x| x.name == "OvenTemp")
            .map(|x| x.id),
        Some(oven_temp)
    );
    // an unknown item is refused and nothing changes
    let rev = c.last_revision;
    match c.call(
        Req::InstantiateLibraryItem(pb::InstantiateLibraryItemRequest {
            base_revision: rev,
            item_id: "fx.source.nope".into(),
            names: Default::default(),
            component: None,
        }),
        &mut events,
    ) {
        Resp::Error(e) => assert_eq!(e.code, "library.unknown_item"),
        other => panic!("expected an error, got {other:?}"),
    }
    assert_eq!(c.last_revision, rev);
    // atomic: a second step that is refused (a relationship name the
    // language cannot spell) leaves no first step behind
    let p_before = project(c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events));
    match c.call(
        Req::InstantiateLibraryItem(pb::InstantiateLibraryItemRequest {
            base_revision: rev,
            item_id: "fx.source.tilt".into(),
            names: [
                ("value".to_string(), "LeanAngle".to_string()),
                ("source".to_string(), "lean sensor".to_string()),
            ]
            .into_iter()
            .collect(),
            component: None,
        }),
        &mut events,
    ) {
        Resp::Error(_) => {}
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(c.last_revision, rev);
    let p_after = project(c.call(Req::GetProject(pb::GetProjectRequest {}), &mut events));
    assert_eq!(p_after.concepts.len(), p_before.concepts.len());
    assert!(!p_after.concepts.iter().any(|x| x.name == "LeanAngle"));
    // a Concept item through the same request
    let e = insert(&mut c, &mut events, "fx.humidity", vec![]);
    assert!(e
        .project
        .unwrap()
        .concepts
        .iter()
        .any(|x| x.name == "Humidity"));
    // the source text: the preferred spelling, never the shorthand
    let Resp::Sources(s) = c.call(Req::GetSources(pb::GetSourcesRequest {}), &mut events) else {
        panic!("sources")
    };
    let text: String = s
        .sources
        .unwrap()
        .files
        .iter()
        .map(|f| f.text.clone())
        .collect();
    assert!(
        text.contains("mapping temperatureInput : () -> Temperature"),
        "{text}"
    );
    assert!(text.contains("concept Temperature : Temperature"), "{text}");
    assert!(
        !text.contains("mapping temperatureInput : Temperature"),
        "{text}"
    );
    // the unresolved source is a simulation input: a value per tick reaches it
    let temp_sensor = created[0].2;
    let room_temp = created[0].1;
    let Resp::Simulation(sim) = c.call(
        Req::StartSimulation(pb::StartSimulationRequest {
            inputs: vec![pb::SimulationInput {
                mapping_id: temp_sensor,
                tick: 0,
                value: Some(pb::Value {
                    kind: Some(pb::value::Kind::Semantic(Box::new(pb::SemanticValue {
                        concept_id: room_temp,
                        repr: Some(Box::new(pb::Value {
                            kind: Some(pb::value::Kind::Quantity(pb::Quantity {
                                dim: Some(pb::Dim {
                                    temperature: 1,
                                    ..Default::default()
                                }),
                                value: 293.0,
                            })),
                        })),
                    }))),
                }),
            }],
            schedule: vec![],
        }),
        &mut events,
    ) else {
        panic!("simulation")
    };
    assert!(sim.error.is_none(), "{:?}", sim.error);
    // saved and reopened: the same objects, no library needed
    let Resp::Project(_) = c.call(
        Req::SaveProject(pb::SaveProjectRequest { force: false }),
        &mut events,
    ) else {
        panic!("save")
    };
    c.call(Req::CloseProject(pb::CloseProjectRequest {}), &mut events);
    let reopened = project(c.call(
        Req::OpenProject(pb::OpenProjectRequest {
            root_path: root.to_string_lossy().into(),
        }),
        &mut events,
    ));
    assert_eq!(
        reopened
            .mappings
            .iter()
            .find(|m| m.name == "temperatureInput")
            .map(|m| m.id),
        Some(temp_sensor)
    );
    assert_eq!(
        reopened
            .concepts
            .iter()
            .find(|x| x.name == "Temperature")
            .map(|x| x.id),
        Some(room_temp)
    );
    assert_eq!(reopened.mappings.len(), 7);
    c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events);
}

/// What one library transaction guarantees (docs/spec/concept-library.md):
/// a Source item is exactly one revision and one history entry; a refused
/// step leaves nothing — no object, no layout, no history entry, no
/// dirtiness, and no identity consumed; a request naming a key the item
/// does not create, or choosing a name the project holds, is refused
/// whole; and inside a component body the fragment keys resolve to the
/// body's own identities, never to a system concept that happens to share
/// the local number.
#[test]
fn a_library_transaction_is_all_or_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rig");
    let mut events = Vec::new();
    let mut c = Client::spawn();
    c.call(
        Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "e2e".into(),
            client_version: "0".into(),
        }),
        &mut events,
    );
    project(c.call(
        Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: "rig".into(),
            template: None,
        }),
        &mut events,
    ));
    let instantiate = |c: &mut Client,
                       events: &mut Vec<pb::Event>,
                       id: &str,
                       names: Vec<(&str, &str)>,
                       component: Option<u64>| {
        let base = c.last_revision;
        c.call(
            Req::InstantiateLibraryItem(pb::InstantiateLibraryItemRequest {
                base_revision: base,
                item_id: id.into(),
                names: names
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                component,
            }),
            events,
        )
    };
    let get = |c: &mut Client, events: &mut Vec<pb::Event>| {
        project(c.call(Req::GetProject(pb::GetProjectRequest {}), events))
    };

    // exactly one revision, one history entry, both objects placed
    let before = c.last_revision;
    let Resp::SystemEditApplied(e) =
        instantiate(&mut c, &mut events, "fx.source.temperature", vec![], None)
    else {
        panic!("instantiate")
    };
    let p = e.project.unwrap();
    assert_eq!(p.revision, before + 1, "one transaction, one revision");
    let layout = p.layout.as_ref().unwrap();
    assert_eq!(layout.concepts.len(), 1);
    assert_eq!(layout.mappings.len(), 1);
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {}), &mut events) else {
        panic!("undo")
    };
    let p = u.project.unwrap();
    assert!(
        p.concepts.is_empty() && p.mappings.is_empty(),
        "one undo, both gone"
    );
    assert!(!p.can_undo, "one history entry: nothing older to undo");
    let Resp::SystemEditApplied(r) = c.call(Req::Redo(pb::RedoRequest {}), &mut events) else {
        panic!("redo")
    };
    let p = r.project.unwrap();
    assert_eq!((p.concepts.len(), p.mappings.len()), (1, 1));
    let room_temp = p.concepts[0].id;
    let temp_sensor = p.mappings[0].id;

    // the item's category forces no role: the relationship is a Source
    // because it is unresolved at the boundary (ADR-0032), an ordinary
    // Mapping once it has a definition, and a Source again without one
    let role = |c: &mut Client, events: &mut Vec<pb::Event>| -> Option<String> {
        let Resp::DraftHover(h) = c.call(
            Req::HoverEntity(pb::HoverEntityRequest {
                revision: c.last_revision,
                entity: Some(pb::EntityRef {
                    kind: Some(pb::entity_ref::Kind::MappingId(temp_sensor)),
                }),
            }),
            events,
        ) else {
            panic!("hover")
        };
        assert!(h.found);
        h.details
            .iter()
            .find(|d| d.label == "role")
            .map(|d| d.value.clone())
    };
    assert_eq!(role(&mut c, &mut events).as_deref(), Some("Source"));
    let base = c.last_revision;
    let Resp::EditApplied(_) = c.call(
        edit(
            base,
            pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
                id: temp_sensor,
                definition: Some(pb::Definition {
                    kind: Some(pb::definition::Kind::Formula("293 K".into())),
                }),
            }),
        ),
        &mut events,
    ) else {
        panic!("attach")
    };
    assert_eq!(role(&mut c, &mut events).as_deref(), Some("Value"));
    let Resp::SystemEditApplied(_) = c.call(Req::Undo(pb::UndoRequest {}), &mut events) else {
        panic!("undo")
    };
    assert_eq!(role(&mut c, &mut events).as_deref(), Some("Source"));

    // a clean slate to measure nothing against
    let Resp::Project(_) = c.call(
        Req::SaveProject(pb::SaveProjectRequest { force: false }),
        &mut events,
    ) else {
        panic!("save")
    };
    let clean = get(&mut c, &mut events);
    assert!(!clean.dirty);
    let rev = c.last_revision;

    // a refused second step: nothing of the first step survives
    let refusals = [
        // the language cannot spell the relationship's name
        (
            "fx.source.tilt",
            vec![("source", "lean sensor")],
            "edit.invalid_name",
        ),
        // the chosen concept name is taken (never a silent `Temperature2`)
        ("fx.source.tilt", vec![("value", "Temperature")], "edit."),
        // the chosen relationship name is taken, after a good first step
        (
            "fx.source.tilt",
            vec![("source", "temperatureInput")],
            "edit.",
        ),
        // a key the item does not create
        (
            "fx.source.tilt",
            vec![("sensor", "X")],
            "library.invalid_plan",
        ),
    ];
    for (id, names, code) in refusals {
        match instantiate(&mut c, &mut events, id, names.clone(), None) {
            Resp::Error(e) => assert!(
                e.code.starts_with(code),
                "{id} {names:?}: expected {code}, got {} ({})",
                e.code,
                e.message
            ),
            other => panic!("{id} {names:?}: expected a refusal, got {other:?}"),
        }
        assert_eq!(c.last_revision, rev);
        let after = get(&mut c, &mut events);
        assert_eq!(after.revision, rev);
        assert_eq!(after.concepts, clean.concepts, "{id} {names:?}");
        assert_eq!(after.mappings, clean.mappings, "{id} {names:?}");
        assert_eq!(after.layout, clean.layout, "no layout half-state");
        assert_eq!(after.can_undo, clean.can_undo, "no history entry");
        assert!(!after.dirty, "{id} {names:?}: nothing to save");
    }
    // no identity was consumed by the refused first steps: the next
    // concept gets the id it would have got anyway
    let Resp::SystemEditApplied(e) = instantiate(&mut c, &mut events, "fx.humidity", vec![], None)
    else {
        panic!("instantiate")
    };
    let humidity = e.outcome.unwrap().inner.unwrap().created_concept.unwrap();
    assert_eq!(humidity, room_temp + 1, "sequential ids, none leaked");
    // …and undoing now removes the humidity, not the refused attempts
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {}), &mut events) else {
        panic!("undo")
    };
    let p = u.project.unwrap();
    assert_eq!(p.concepts, clean.concepts);
    assert_eq!(p.mappings, clean.mappings);

    // inside a component body: the keys resolve to the body's identities
    let base = c.last_revision;
    let Resp::SystemEditApplied(e) = c.call(
        Req::ApplySystemEdit(pb::ApplySystemEditRequest {
            base_revision: base,
            op: Some(pb::SystemEditOp {
                op: Some(pb::system_edit_op::Op::CreateComponent(
                    pb::CreateComponent {
                        name: "Probe".into(),
                        description: String::new(),
                    },
                )),
            }),
        }),
        &mut events,
    ) else {
        panic!("create component")
    };
    let probe = e.outcome.unwrap().created_component.unwrap();
    let base_before = get(&mut c, &mut events);
    let Resp::SystemEditApplied(e) = instantiate(
        &mut c,
        &mut events,
        "fx.source.distance",
        vec![],
        Some(probe),
    ) else {
        panic!("instantiate in body")
    };
    let inner = e.outcome.unwrap().inner.unwrap();
    let (local_concept, local_mapping) = (
        inner.created_concept.unwrap(),
        inner.created_mapping.unwrap(),
    );
    let Resp::System(sv) = c.call(Req::GetSystem(pb::GetSystemRequest {}), &mut events) else {
        panic!("system")
    };
    let system = sv.system.unwrap();
    let body = system
        .components
        .iter()
        .find(|x| x.id == probe)
        .unwrap()
        .body
        .as_ref()
        .unwrap();
    let distance = body
        .concepts
        .iter()
        .find(|x| x.id == local_concept)
        .unwrap();
    assert_eq!(distance.name, "Distance");
    let sensor = body
        .mappings
        .iter()
        .find(|m| m.id == local_mapping)
        .unwrap();
    assert_eq!(sensor.name, "distanceInput");
    assert_eq!(
        sensor.signature.as_ref().unwrap().output,
        local_concept,
        "the body's own concept, not the system concept with the same local number"
    );
    // the system's base design is untouched: the same concepts, and its
    // concept 0 (Temperature) is still Temperature, not bound by the body
    let base_after = get(&mut c, &mut events);
    assert!(base_after.concepts.iter().all(|x| base_before
        .concepts
        .iter()
        .any(|b| b.id == x.id && b.name == x.name)));
    assert!(base_after
        .concepts
        .iter()
        .any(|x| x.id == room_temp && x.name == "Temperature"));
    assert_eq!(
        base_after.mappings.len(),
        base_before.mappings.len(),
        "a body without an instance adds nothing to the flat design"
    );
    // an unknown component is refused whole
    match instantiate(&mut c, &mut events, "fx.source.tilt", vec![], Some(999)) {
        Resp::Error(e) => assert_eq!(e.code, "system.unknown_component"),
        other => panic!("expected a refusal, got {other:?}"),
    }
    c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events);
}
