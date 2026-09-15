#![allow(clippy::unwrap_used)]
//! The Deploy read model over the real `bdld`: what a frontend that knows
//! nothing about requirements, units or dead ends needs — a target chooser,
//! a three-way status plus `deployable`, an assignment table by name, what
//! is missing, and the blocker — through the framed protocol, with the
//! project revision on every answer.

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
    events: Vec<pb::Event>,
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
            events: Vec::new(),
        }
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
                        _ => {}
                    }
                    return payload;
                }
                pb::server_message::Payload::Event(e) => self.events.push(e),
            }
        }
    }

    fn apply(&mut self, op: pb::edit_op::Op) -> pb::EditApplied {
        let base = self.last_revision;
        match self.call(Req::ApplyEdit(pb::ApplyEditRequest {
            base_revision: base,
            op: Some(pb::EditOp { op: Some(op) }),
        })) {
            Resp::EditApplied(e) => e,
            other => panic!("edit failed: {other:?}"),
        }
    }

    fn deploy(&mut self, target: &str) -> pb::DeploymentAnalysis {
        let revision = Some(self.last_revision);
        match self.call(Req::AnalyzeDeployment(pb::AnalyzeDeploymentRequest {
            target_id: target.into(),
            revision,
        })) {
            Resp::Deployment(d) => d.deployment.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn analysis(&mut self) -> pb::ProjectAnalysis {
        match self.call(Req::RunAnalysis(pb::RunAnalysisRequest {})) {
            Resp::Analysis(a) => a.analysis.unwrap(),
            other => panic!("{other:?}"),
        }
    }
}

/// A rover: Speed concept, a nullary `cruise : () -> Speed := 0.5` in
/// domain `main`, an output `motor` accepting Speed in `main`.  Returns
/// (client, speed concept, cruise mapping, main clock, motor output).
fn rover(dir: &tempfile::TempDir) -> (Client, u64, u64, u64, u64) {
    let mut c = Client::spawn();
    c.call(Req::Handshake(pb::HandshakeRequest {
        client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
        client_name: "deploy-e2e".into(),
        client_version: "0".into(),
    }));
    c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: dir.path().join("rover").to_string_lossy().into(),
        name: "rover".into(),
    }));
    let speed = c
        .apply(pb::edit_op::Op::CreateConcept(pb::CreateConcept {
            name: "Speed".into(),
            description: String::new(),
            representation: Some(pb::Representation {
                kind: Some(pb::representation::Kind::Quantity(pb::Dim::default())),
            }),
        }))
        .outcome
        .unwrap()
        .created_concept
        .unwrap();
    let cruise = c
        .apply(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
            name: "cruise".into(),
            description: String::new(),
            signature: Some(pb::Signature {
                inputs: vec![],
                output: speed,
            }),
        }))
        .outcome
        .unwrap()
        .created_mapping
        .unwrap();
    c.apply(pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
        id: cruise,
        definition: Some(pb::Definition {
            kind: Some(pb::definition::Kind::Formula("0.5".into())),
        }),
    }));
    let main = c
        .apply(pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
            name: "main".into(),
        }))
        .outcome
        .unwrap()
        .created_clock
        .unwrap();
    c.apply(pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
        id: cruise,
        clock_id: Some(main),
    }));
    let motor = c
        .apply(pb::edit_op::Op::CreateOutput(pb::CreateOutput {
            name: "motor".into(),
            description: String::new(),
            accepts: speed,
            clock_id: Some(main),
        }))
        .outcome
        .unwrap()
        .created_output
        .unwrap();
    (c, speed, cruise, main, motor)
}

fn device(c: &mut Client, name: &str, kind: pb::DeviceKind, output: Option<u64>) -> u64 {
    c.apply(pb::edit_op::Op::CreateDevice(pb::CreateDevice {
        name: name.into(),
        kind: kind.into(),
        output_id: output,
    }))
    .outcome
    .unwrap()
    .created_device
    .unwrap()
}

fn shutdown(mut c: Client) {
    let Resp::Ack(_) = c.call(Req::Shutdown(pb::ShutdownRequest {})) else {
        panic!()
    };
    assert!(c.child.wait().unwrap().success());
}

#[test]
fn target_list_is_a_chooser_not_an_id_to_parse() {
    let dir = tempfile::tempdir().unwrap();
    let (mut c, ..) = rover(&dir);
    let Resp::Targets(t) = c.call(Req::ListTargets(pb::ListTargetsRequest {})) else {
        panic!()
    };
    let nano = t.targets.iter().find(|t| t.id == "arduino_nano").unwrap();
    assert_eq!(nano.display_name, "Arduino Nano");
    assert_eq!(nano.name, nano.display_name);
    assert_eq!(nano.family, "avr");
    assert!(nano.description.contains("6 PWM"));
    assert_eq!(nano.resource_count, 22);
    let pwm = nano
        .capabilities
        .iter()
        .find(|c| c.capability == "pwm")
        .unwrap();
    assert_eq!(
        (pwm.label.as_str(), pwm.resource_count, pwm.shareable),
        ("PWM", 6, false)
    );
    let sda = nano
        .capabilities
        .iter()
        .find(|c| c.capability == "i2c_sda")
        .unwrap();
    assert!(sda.shareable && sda.label == "I²C SDA");
    let big = t.targets.iter().find(|t| t.id == "big_board").unwrap();
    assert_eq!(big.family, "mock");
    assert_eq!(
        big.capabilities
            .iter()
            .find(|c| c.capability == "pwm")
            .unwrap()
            .resource_count,
        12
    );
    shutdown(c);
}

#[test]
fn incomplete_says_what_is_missing_then_feasible_gives_a_table_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let (mut c, _speed, cruise, _main, motor) = rover(&dir);

    // 5. incomplete configuration: nothing bound at all
    let d = c.deploy("arduino_nano");
    assert_eq!(d.revision, c.last_revision);
    assert_eq!(d.target_display_name, "Arduino Nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Incomplete);
    assert!(!d.design_ready && !d.deployable);
    let kinds: Vec<pb::MissingKind> = d.missing.iter().map(|m| m.kind()).collect();
    assert_eq!(
        kinds,
        [
            pb::MissingKind::OutputNoDriver,
            pb::MissingKind::OutputNoDevice
        ]
    );
    assert_eq!(d.missing[0].output_name, "motor");
    assert_eq!(d.missing[0].message, "motor has no final target yet.");
    assert_eq!(d.missing[1].message, "motor has no device on Arduino Nano.");
    assert!(d.rows.is_empty() && d.blocker.is_none());

    // connect the driver: design ready, deployment still incomplete
    c.apply(pb::edit_op::Op::SetMappingDrive(pb::SetMappingDrive {
        id: cruise,
        output_id: Some(motor),
    }));
    let d = c.deploy("arduino_nano");
    assert!(d.design_ready && !d.deployable);
    assert_eq!(d.status(), pb::DeploymentStatus::Incomplete);
    assert_eq!(d.missing.len(), 1);
    assert_eq!(d.missing[0].kind(), pb::MissingKind::OutputNoDevice);

    // a device bound to nothing is named too
    let spare = device(&mut c, "spare", pb::DeviceKind::DigitalOutput, None);
    let d = c.deploy("arduino_nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Incomplete);
    assert_eq!(
        d.missing.iter().map(|m| m.kind()).collect::<Vec<_>>(),
        [
            pb::MissingKind::OutputNoDevice,
            pb::MissingKind::DeviceNoOutput
        ]
    );
    assert_eq!(
        (d.missing[1].device_id, d.missing[1].device_name.as_str()),
        (Some(spare), "spare")
    );
    // the unbound device is still placed: its row has a resource but no output
    assert_eq!(d.rows.len(), 1);
    assert_eq!(
        (
            d.rows[0].output_id,
            d.rows[0].device_name.as_str(),
            d.rows[0].resource.as_deref()
        ),
        (None, "spare", Some("D0"))
    );
    c.apply(pb::edit_op::Op::DeleteDevice(pb::DeleteDevice {
        id: spare,
    }));

    // 1. feasible on the Nano: the table, by name
    let drive = device(&mut c, "drive", pb::DeviceKind::HBridgeChannel, Some(motor));
    let d = c.deploy("arduino_nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Feasible);
    assert!(d.design_ready && d.deployable && d.missing.is_empty());
    let table: Vec<(String, String, String, String, String, String, String)> = d
        .rows
        .iter()
        .map(|r| {
            (
                r.output_name.clone(),
                r.device_name.clone(),
                r.device_kind_label.clone(),
                r.requirement_label.clone(),
                r.capability_label.clone(),
                r.resource.clone().unwrap(),
                r.resource_label.clone().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        table,
        vec![
            (
                "motor".into(),
                "drive".into(),
                "H-bridge channel".into(),
                "PWM".into(),
                "PWM".into(),
                "D3".into(),
                "D3: digital in, digital out, PWM (timer 2), interrupt".into()
            ),
            (
                "motor".into(),
                "drive".into(),
                "H-bridge channel".into(),
                "direction".into(),
                "digital out".into(),
                "D0".into(),
                "D0: digital in, digital out, UART RX (UART unit 0)".into()
            ),
        ]
    );
    assert!(d
        .rows
        .iter()
        .all(|r| r.fixed.is_none() && r.device_id == drive && r.output_id == Some(motor)));
    // 7. nothing above needed the raw layer, which is still there for tools
    assert_eq!(d.requirements.len(), 2);
    assert_eq!(d.assignment.len(), 2);

    // 2. the same design on the big board
    let big = c.deploy("big_board");
    assert!(big.deployable);
    assert_eq!(big.target_display_name, "Big board (mock)");
    assert_eq!(big.rows.len(), 2);

    // 9. deterministic
    assert_eq!(c.deploy("arduino_nano"), d);

    // 10. target choice does not alter the semantic analysis
    let before = c.analysis();
    c.deploy("big_board");
    c.deploy("arduino_nano");
    assert_eq!(c.analysis(), before);
    assert!(before.output_complete);

    // 4. fixed-pin conflict: pin the PWM line to a pin that cannot do PWM
    c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
        id: drive,
        index: 0,
        resource: Some("D4".into()),
    }));
    let d = c.deploy("arduino_nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Infeasible);
    assert!(d.design_ready && !d.deployable && d.missing.is_empty());
    let b = d.blocker.unwrap();
    assert_eq!(
        (
            b.device_name.as_str(),
            b.requirement_label.as_str(),
            b.capability_label.as_str()
        ),
        ("drive", "PWM", "PWM")
    );
    assert_eq!(
        b.kind,
        Some(pb::blocker::Kind::FixedUnavailable("D4".into()))
    );
    assert_eq!(b.message, "D4 cannot carry drive PWM on arduino_nano.");
    assert!(b.explanation.contains("chosen by hand"));
    assert!(d.rows.iter().all(|r| r.resource.is_none()));
    assert_eq!(d.rows[0].fixed.as_deref(), Some("D4"));
    // still the same semantic analysis (a pin edit is a new revision, nothing more)
    let mut after = c.analysis();
    assert_eq!(after.revision, before.revision + 1);
    after.revision = before.revision;
    assert_eq!(after, before);
    c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
        id: drive,
        index: 0,
        resource: None,
    }));

    // 3. PWM exhaustion on the Nano: six more channels; each is bound to
    // nothing, so on the big board this is incomplete, on the Nano infeasible
    for n in 1..=6 {
        device(&mut c, &format!("L{n}"), pb::DeviceKind::PwmChannel, None);
    }
    let d = c.deploy("arduino_nano");
    assert_eq!(d.status(), pb::DeploymentStatus::Infeasible);
    let b = d.blocker.unwrap();
    let Some(pb::blocker::Kind::Blocked(cands)) = b.kind else {
        panic!("{:?}", b.kind)
    };
    assert_eq!(cands.candidates.len(), 6);
    assert!(cands
        .candidates
        .iter()
        .all(|x| !x.held_by_device_name.is_empty() && !x.resource_label.is_empty()));
    assert!(b
        .message
        .starts_with("No free pin on arduino_nano can carry"));
    assert!(b.explanation.contains("Counting pins is not enough"));
    let big = c.deploy("big_board");
    assert_eq!(big.status(), pb::DeploymentStatus::Incomplete);
    assert!(big.blocker.is_none() && big.rows.iter().all(|r| r.resource.is_some()));
    assert_eq!(
        big.missing
            .iter()
            .filter(|m| m.kind() == pb::MissingKind::DeviceNoOutput)
            .count(),
        6
    );

    // 8. stale revision: a request about a revision the project has left
    let stale = c.last_revision - 1;
    let Resp::Error(e) = c.call(Req::AnalyzeDeployment(pb::AnalyzeDeploymentRequest {
        target_id: "arduino_nano".into(),
        revision: Some(stale),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "deploy.stale_revision");
    // without a revision the current one is analysed and reported
    let Resp::Deployment(d) = c.call(Req::AnalyzeDeployment(pb::AnalyzeDeploymentRequest {
        target_id: "arduino_nano".into(),
        revision: None,
    })) else {
        panic!()
    };
    assert_eq!(d.deployment.unwrap().revision, c.last_revision);
    shutdown(c);
}

#[test]
fn semantic_problems_are_named_as_missing_never_as_infeasibility() {
    let dir = tempfile::tempdir().unwrap();
    let (mut c, _speed, cruise, _main, motor) = rover(&dir);
    c.apply(pb::edit_op::Op::SetMappingDrive(pb::SetMappingDrive {
        id: cruise,
        output_id: Some(motor),
    }));
    device(&mut c, "drive", pb::DeviceKind::PwmChannel, Some(motor));
    assert!(c.deploy("arduino_nano").deployable);
    // break the definition: the devices still fit, the design is not ready
    c.apply(pb::edit_op::Op::ReplaceDefinition(pb::ReplaceDefinition {
        id: cruise,
        definition: Some(pb::Definition {
            kind: Some(pb::definition::Kind::Formula("1 s".into())),
        }),
    }));
    let d = c.deploy("arduino_nano");
    assert_eq!(
        d.status(),
        pb::DeploymentStatus::Feasible,
        "target-relative status is untouched"
    );
    assert!(!d.design_ready && !d.deployable && d.blocker.is_none());
    assert_eq!(d.missing.len(), 1);
    assert_eq!(
        (
            d.missing[0].kind(),
            d.missing[0].mapping_id,
            d.missing[0].mapping_name.as_str()
        ),
        (
            pb::MissingKind::RelationshipNotChecking,
            Some(cruise),
            "cruise"
        )
    );
    assert_eq!(d.missing[0].message, "cruise does not check.");
    assert_eq!(d.rows.len(), 1);
    shutdown(c);
}
