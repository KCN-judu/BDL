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
    assert_eq!(a.diagnostics[0].code, "semantic.unbound_representation");
    assert_eq!(a.diagnostics[0].severity(), pb::DiagnosticSeverity::Info);
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
    assert!(a.diagnostics.is_empty());
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
    let d = &a.diagnostics[0];
    assert_eq!(d.code, "dimension.mismatch");
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
    let p = project(c.call(Req::SaveProject(pb::SaveProjectRequest {}), &mut events));
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
    assert_eq!(h.signature, "mapping cruise : Speed");
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
            name: "drive".into(),
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
    assert_eq!(ids, ["arduino_nano", "big_board"]);

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
    assert!(d.diagnostics.is_empty());
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
    assert!(d.diagnostics[0].message.contains("D4 cannot carry drive"));
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {}), &mut events) else {
        panic!()
    };
    assert!(a.analysis.unwrap().output_complete);

    // the whole thing survives a save/reopen
    project(c.call(Req::SaveProject(pb::SaveProjectRequest {}), &mut events));
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
    assert!(a.diagnostics.is_empty());
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
    project(c.call(Req::SaveProject(pb::SaveProjectRequest {}), &mut events));
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

    // The library is served before any project is open, and it is the
    // embedded standard library, template for template.
    let listed = match c.call(
        Req::ListConceptTemplates(pb::ListConceptTemplatesRequest {}),
        &mut events,
    ) {
        Resp::ConceptTemplates(t) => t,
        other => panic!("expected templates, got {other:?}"),
    };
    let std = bdl_library::Library::standard();
    assert_eq!(listed.libraries.len(), 1);
    assert_eq!(listed.libraries[0].id, "std");
    assert_eq!(listed.libraries[0].version, std.info.version);
    let served: Vec<&str> = listed.libraries[0]
        .templates
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    let embedded: Vec<&str> = std.templates().iter().map(|t| t.id.as_str()).collect();
    assert_eq!(served, embedded);
    let light = listed.libraries[0]
        .templates
        .iter()
        .find(|t| t.id == "std.environment.ambient_light")
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
        }),
        &mut events,
    ));
    let mut ids = Vec::new();
    for _ in 0..2 {
        let base = c.last_revision;
        let applied = match c.call(
            Req::InstantiateConceptTemplate(pb::InstantiateConceptTemplateRequest {
                base_revision: base,
                template_id: "std.environment.temperature".into(),
                name: None,
            }),
            &mut events,
        ) {
            Resp::EditApplied(e) => e,
            other => panic!("instantiate failed: {other:?}"),
        };
        ids.push(applied.outcome.unwrap().created_concept.unwrap());
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
            base_revision: base,
            template_id: "std.environment.temperature".into(),
            name: Some("OvenTemperature".into()),
        }),
        &mut events,
    ) {
        Resp::EditApplied(e) => e.project.unwrap(),
        other => panic!("instantiate failed: {other:?}"),
    };
    assert!(named.concepts.iter().any(|x| x.name == "OvenTemperature"));
    match c.call(
        Req::InstantiateConceptTemplate(pb::InstantiateConceptTemplateRequest {
            base_revision: c.last_revision,
            template_id: "std.nope".into(),
            name: None,
        }),
        &mut events,
    ) {
        Resp::Error(e) => assert_eq!(e.code, "library.unknown_template"),
        other => panic!("expected an error, got {other:?}"),
    }
    c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events);
}
