#![allow(clippy::unwrap_used)]
//! The layout service over the real `bdld` (protocol 0.29): the first-open
//! policy (no positions → arranged as a whole; some → only the gaps
//! filled, nothing authored moved; every position → nothing moves), the
//! explicit arrangement answered and never applied by itself, and no
//! revision from any of it.

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
        };
        c.call(Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "layout-e2e".into(),
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
                    return r.payload.unwrap();
                }
                pb::server_message::Payload::Event(_) => {}
            }
        }
    }

    fn open(&mut self, root: &Path) -> pb::ProjectProjection {
        match self.call(Req::OpenProject(pb::OpenProjectRequest {
            root_path: root.to_string_lossy().into(),
        })) {
            Resp::Project(p) => p.project.unwrap(),
            other => panic!("{other:?}"),
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

    fn arranged(&mut self) -> pb::Layout {
        match self.call(Req::ArrangeLayout(pb::ArrangeLayoutRequest {})) {
            Resp::Layout(l) => l.layout.unwrap(),
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

fn pos(list: &[pb::NodePosition], id: u64) -> (f64, f64) {
    let p = list.iter().find(|p| p.id == id).unwrap();
    (p.x, p.y)
}

fn named(p: &pb::ProjectProjection, mapping: &str) -> u64 {
    p.mappings.iter().find(|m| m.name == mapping).unwrap().id
}

fn concept(p: &pb::ProjectProjection, name: &str) -> u64 {
    p.concepts.iter().find(|c| c.name == name).unwrap().id
}

#[test]
fn a_project_with_no_layout_opens_arranged_and_a_reopen_moves_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn();
    let root = dir.path().join("demo");
    let p = c.init(&root, "button-lamp");
    let l = p.layout.as_ref().unwrap();
    // the template has no layout: arranged as a whole, every edge forward
    let pressed = pos(&l.mappings, named(&p, "pressed"));
    let pressed_c = pos(&l.concepts, concept(&p, "Pressed"));
    let lit = pos(&l.mappings, named(&p, "lit"));
    let lit_c = pos(&l.concepts, concept(&p, "Lit"));
    let lamp = pos(&l.outputs, p.outputs[0].id);
    // `lit() = pressed` reads nothing by signature and names `pressed` in
    // its formula: the reference edge orders it after `pressed`, beside
    // the concept `pressed` produces.
    assert!(
        pressed.0 < pressed_c.0 && pressed.0 < lit.0,
        "{pressed:?} {pressed_c:?} {lit:?}"
    );
    assert!(
        lit.0 < lit_c.0 && lit.0 < lamp.0,
        "lit's produce and drive edges go right"
    );
    assert_eq!(lit_c.0, lamp.0, "the two things lit feeds share a column");
    let revision = p.revision;
    // saved: a reopen reads the same positions and arranges nothing
    let layout_file = root.join("ui/layout.json");
    assert!(layout_file.is_file());
    let text = std::fs::read_to_string(&layout_file).unwrap();
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let again = c.open(&root);
    assert_eq!(again.layout, p.layout);
    assert_eq!(std::fs::read_to_string(&layout_file).unwrap(), text);
    assert_eq!(again.revision, revision, "layout is never a revision");
}

#[test]
fn a_partial_layout_keeps_what_was_authored_and_fills_the_gaps() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn();
    let root = dir.path().join("demo");
    let p = c.init(&root, "button-lamp");
    let lit = named(&p, "lit");
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    // an authored position for lit alone (a hand-written layout file)
    std::fs::write(
        root.join("ui/layout.json"),
        format!(
            "{{\"schema_version\":1,\"layout\":{{\"mappings\":{{\"{lit}\":{{\"x\":900.0,\"y\":700.0}}}}}}}}"
        ),
    )
    .unwrap();
    let p = c.open(&root);
    let l = p.layout.as_ref().unwrap();
    assert_eq!(pos(&l.mappings, lit), (900.0, 700.0), "authored, untouched");
    // everything else was placed by place_missing, beside what it reads
    assert_eq!(l.concepts.len(), 2);
    assert_eq!(l.mappings.len(), 2);
    assert_eq!(l.outputs.len(), 1);
    let lit_c = pos(&l.concepts, concept(&p, "Lit"));
    assert!((lit_c.1 - 700.0).abs() < 200.0, "Lit near lit: {lit_c:?}");
}

#[test]
fn arrange_is_answered_not_applied_and_is_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn();
    let root = dir.path().join("demo");
    let p = c.init(&root, "button-lamp-configured");
    let opened = p.layout.clone().unwrap();
    let revision = p.revision;
    let lit = named(&p, "lit");
    // the designer moves lit far away
    let mut moved = opened.clone();
    for m in &mut moved.mappings {
        if m.id == lit {
            m.x = 1500.0;
            m.y = 900.0;
        }
    }
    match c.call(Req::SetLayout(pb::SetLayoutRequest {
        layout: Some(moved.clone()),
    })) {
        Resp::Ack(_) => {}
        other => panic!("{other:?}"),
    }
    let a = c.arranged();
    let b = c.arranged();
    assert_eq!(a, b, "deterministic");
    assert_eq!(
        a.mappings, opened.mappings,
        "the arrangement of this graph is what the first open gave"
    );
    assert_eq!(pos(&a.mappings, lit), pos(&opened.mappings, lit));
    // answered only: the project still holds the moved layout, and no
    // revision was made by any of this
    let now = match c.call(Req::GetProject(pb::GetProjectRequest {})) {
        Resp::Project(p) => p.project.unwrap(),
        other => panic!("{other:?}"),
    };
    assert_eq!(
        pos(&now.layout.as_ref().unwrap().mappings, lit),
        (1500.0, 900.0)
    );
    assert_eq!(now.revision, revision);
}
