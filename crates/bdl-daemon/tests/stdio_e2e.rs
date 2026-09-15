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
