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
                    return r.payload.unwrap();
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

    // duplicate name → designer-readable error with a stable code
    let Resp::Error(e) = c.call(
        edit(
            3,
            pb::edit_op::Op::CreateConcept(pb::CreateConcept {
                name: "Tilt".into(),
                ..Default::default()
            }),
        ),
        &mut events,
    ) else {
        panic!()
    };
    assert_eq!(e.code, "edit.duplicate_concept_name");
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
    assert_eq!(p.revision, 3);

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
    assert_eq!(p.mappings[0].state(), pb::AcceptanceState::Declared);
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
    assert_eq!(seen, vec![1, 2, 3]);

    let Resp::Ack(_) = c.call(Req::Shutdown(pb::ShutdownRequest {}), &mut events) else {
        panic!()
    };
    assert!(c.child.wait().unwrap().success());
}
