#![allow(clippy::unwrap_used)]
//! The Sem-block model over the real `bdld` (protocol 0.30): the Button →
//! Lamp demo's `lit` is a Sem block whose mapping block reads the Source
//! `pressed` (`MappingAnalysis.references`), and the open positions of a
//! definition arrive as `slots` — the canvas's open sockets — so that a
//! dropped Sem block goes where the compiler says, never where a client's
//! parse would put it.

use bdl_protocol::framing;
use bdl_protocol::pb::{self, client_message::Payload as Req, response::Payload as Resp};
use bytes::BytesMut;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

struct Client {
    child: Child,
    buf: BytesMut,
    next_id: u64,
    revision: u64,
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
            revision: 0,
        };
        c.call(Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "sem-e2e".into(),
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
                        Resp::Project(p) => self.revision = p.project.as_ref().unwrap().revision,
                        Resp::EditApplied(e) => {
                            self.revision = e.project.as_ref().unwrap().revision
                        }
                        _ => {}
                    }
                    return payload;
                }
                pb::server_message::Payload::Event(_) => {}
            }
        }
    }

    fn init(&mut self, root: &Path, template: &str) -> pb::ProjectProjection {
        match self.call(Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: root.file_name().unwrap().to_string_lossy().into(),
            template: Some(template.into()),
        })) {
            Resp::Project(p) => p.project.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn analysis(&mut self) -> pb::ProjectAnalysis {
        match self.call(Req::RunAnalysis(pb::RunAnalysisRequest {})) {
            Resp::Analysis(a) => a.analysis.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn edit(&mut self, op: pb::edit_op::Op) -> pb::ProjectProjection {
        let base = self.revision;
        match self.call(Req::ApplyEdit(pb::ApplyEditRequest {
            base_revision: base,
            op: Some(pb::EditOp { op: Some(op) }),
        })) {
            Resp::EditApplied(e) => e.project.unwrap(),
            other => panic!("{other:?}"),
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn named(p: &pb::ProjectProjection, mapping: &str) -> u64 {
    p.mappings.iter().find(|m| m.name == mapping).unwrap().id
}

fn of(a: &pb::ProjectAnalysis, id: u64) -> &pb::MappingAnalysis {
    a.mappings.iter().find(|m| m.id == id).unwrap()
}

#[test]
fn the_demo_is_two_sem_blocks_and_a_read_edge_and_slots_name_the_open_positions() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn();
    let root = dir.path().join("demo");
    let p = c.init(&root, "button-lamp");
    let pressed = named(&p, "pressed");
    let lit = named(&p, "lit");
    // both are Sem blocks: unit domain, no parameters; pressed a Source
    for m in &p.mappings {
        assert!(
            m.signature.as_ref().unwrap().inputs.is_empty(),
            "{}",
            m.name
        );
    }
    assert_eq!(
        p.mappings.iter().find(|m| m.id == pressed).unwrap().role(),
        pb::RelationshipRole::Source
    );
    assert_eq!(
        p.mappings.iter().find(|m| m.id == lit).unwrap().role(),
        pb::RelationshipRole::Value
    );
    let a = c.analysis();
    // lit's mapping block reads pressed — the one read edge of the canvas
    assert_eq!(of(&a, lit).references, vec![pressed]);
    assert_eq!(of(&a, pressed).applied_by, vec![lit]);
    assert!(of(&a, lit).slots.is_empty());
    assert!(of(&a, pressed).slots.is_empty());
    // a definition with open positions: the slots, in source order
    c.edit(pb::edit_op::Op::ReplaceDefinition(pb::ReplaceDefinition {
        id: lit,
        definition: Some(pb::Definition {
            kind: Some(pb::definition::Kind::Formula(
                "if ? then pressed else ?".into(),
            )),
        }),
    }));
    let a = c.analysis();
    let slots = &of(&a, lit).slots;
    assert_eq!(slots.len(), 2, "{slots:?}");
    // (a definition with holes does not elaborate: no read edges until it
    // does — `references` is the kernel's dependency, never a parse)
    assert!(of(&a, lit).references.is_empty());
    // a second Sem block of Pressed beside the first raises nothing
    let p = c.edit(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
        name: "pressedB".into(),
        description: String::new(),
        signature: Some(pb::Signature {
            inputs: vec![],
            output: p.concepts.iter().find(|x| x.name == "Pressed").unwrap().id,
        }),
        definition: None,
        clock_id: None,
    }));
    let a = c.analysis();
    let b = named(&p, "pressedB");
    assert!(of(&a, b).diagnostics.is_empty());
    assert!(of(&a, pressed).diagnostics.is_empty());
    assert!(a
        .diagnostics
        .iter()
        .all(|d| !d.message.to_lowercase().contains("producer")));
}
