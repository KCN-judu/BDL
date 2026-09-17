//! The adapter over a text project (ADR-0020): the sources on disk are
//! the ground, open buffers substitute, and every query answers with
//! `file://` locations across files — the acceptance chain of the
//! textual milestone at the LSP boundary.

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{self as lsp, Position};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

const CONCEPTS: &str = "// Shared vocabulary.\nconcept Tilt : Angle\nconcept Brightness : Scalar\nconcept Gain : Scalar\n\nclock interaction\n";

const LAMP: &str = "component AdaptiveLamp {\n  use concept Tilt\n  use concept Brightness\n  use concept Gain\n  param clock main\n\n  requires tiltValue : Tilt @main\n  param gain : Gain\n\n  mapping dimByTilt : Tilt -> Brightness\n  dimByTilt(t) = t / (90 deg)\n\n  provides brightness : Brightness @main\n  brightness() = dimByTilt(tiltValue) * gain\n}\n";

const MAIN: &str = "mapping tilt : Tilt @interaction\nmapping tiltValue : Tilt @interaction\ntiltValue() = tilt\n\ninstance lampA : AdaptiveLamp { main = interaction, gain = 2 }\ninstance lampB : AdaptiveLamp { main = interaction, gain = 1 }\n\nmapping brightness : Brightness @interaction\nmapping mirror : Brightness @interaction\n\nbind lampA.tiltValue = tiltValue\nbind lampB.tiltValue = tiltValue\nbind brightness = lampA.brightness\nbind mirror = lampB.brightness\n\noutput light : Brightness @interaction\ndrive light = brightness\n";

struct Client {
    conn: Connection,
    next: i32,
    server: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}

impl Client {
    fn start(root: &Path) -> Client {
        let (server_conn, client_conn) = Connection::memory();
        let server = std::thread::spawn(move || bdl_lsp::run(server_conn));
        let mut c = Client {
            conn: client_conn,
            next: 0,
            server: Some(server),
        };
        let init = c.request(
            lsp::request::Initialize::METHOD,
            json!({
                "processId": null,
                "capabilities": {
                    "general": { "positionEncodings": ["utf-16"] },
                    "textDocument": { "diagnostic": { "dynamicRegistration": false } }
                },
                "initializationOptions": { "projectRoot": root.to_string_lossy() },
            }),
        );
        assert!(init["capabilities"]["renameProvider"].is_object());
        c.notify(lsp::notification::Initialized::METHOD, json!({}));
        c
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = RequestId::from(self.next);
        self.next += 1;
        self.conn
            .sender
            .send(Message::Request(Request::new(
                id.clone(),
                method.into(),
                params,
            )))
            .expect("send");
        loop {
            match self.conn.receiver.recv_timeout(Duration::from_secs(20)) {
                Ok(Message::Response(Response {
                    id: rid,
                    response_result,
                })) if rid == id => {
                    return match response_result {
                        Ok(v) => v,
                        Err(e) => panic!("{method} failed: {e:?}"),
                    };
                }
                Ok(_) => continue,
                Err(e) => panic!("no response to {method}: {e}"),
            }
        }
    }

    fn notify(&self, method: &str, params: Value) {
        self.conn
            .sender
            .send(Message::Notification(Notification::new(
                method.into(),
                params,
            )))
            .expect("send");
    }

    fn shutdown(mut self) {
        self.request(lsp::request::Shutdown::METHOD, Value::Null);
        self.notify(lsp::notification::Exit::METHOD, Value::Null);
        self.server
            .take()
            .expect("server")
            .join()
            .expect("join")
            .expect("server exits cleanly");
    }
}

fn position(text: &str, needle: &str, plus: usize) -> Position {
    let byte = text.find(needle).expect("needle") + plus;
    let before = &text[..byte];
    let line = before.matches('\n').count() as u32;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = before[line_start..].encode_utf16().count() as u32;
    Position { line, character }
}

fn range_text<'a>(text: &'a str, range: &Value) -> &'a str {
    let to_byte = |p: &Value| {
        let line = p["line"].as_u64().expect("line") as usize;
        let ch = p["character"].as_u64().expect("char") as usize;
        let line_start = text
            .split_inclusive('\n')
            .take(line)
            .map(str::len)
            .sum::<usize>();
        let mut units = 0;
        let mut bytes = 0;
        for c in text[line_start..].chars() {
            if units >= ch {
                break;
            }
            units += c.len_utf16();
            bytes += c.len_utf8();
        }
        line_start + bytes
    };
    &text[to_byte(&range["start"])..to_byte(&range["end"])]
}

struct Project {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Project {
    fn create() -> Project {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().to_path_buf();
        bdl_text::init_text_project(&root, "lamp", "test").expect("init");
        std::fs::write(root.join("src/concepts.bdl"), CONCEPTS).expect("write");
        std::fs::write(root.join("src/lamp.bdl"), LAMP).expect("write");
        std::fs::write(root.join("src/main.bdl"), MAIN).expect("write");
        Project { _dir: dir, root }
    }

    fn uri(&self, rel: &str) -> String {
        format!("file://{}", self.root.join(rel).to_string_lossy())
    }

    fn text(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).expect("read")
    }
}

fn locations_by_file(v: &Value) -> Vec<(String, String)> {
    v.as_array()
        .expect("locations")
        .iter()
        .map(|l| {
            (
                l["uri"].as_str().expect("uri").to_owned(),
                l["range"]["start"]["line"].to_string(),
            )
        })
        .collect()
}

#[test]
fn navigation_and_rename_cross_files_without_opening_them() {
    let p = Project::create();
    let mut c = Client::start(&p.root);
    let lamp = json!({ "uri": p.uri("src/lamp.bdl") });

    // Definition of `Tilt` in the component's `requires` is the
    // declaration in concepts.bdl — a file no buffer is open for.
    let at_tilt = position(LAMP, "tiltValue : Tilt", "tiltValue : ".len() + 1);
    let d = c.request(
        lsp::request::GotoDefinition::METHOD,
        json!({ "textDocument": lamp, "position": at_tilt }),
    );
    assert_eq!(d.as_array().map(Vec::len), Some(1), "{d}");
    assert_eq!(d[0]["uri"], p.uri("src/concepts.bdl"));
    assert_eq!(range_text(CONCEPTS, &d[0]["range"]), "Tilt");

    // References to Tilt reach every file: the declaration, the `use`,
    // the port and mapping signatures, and the two mappings in main.bdl.
    let r = c.request(
        lsp::request::References::METHOD,
        json!({ "textDocument": lamp, "position": at_tilt, "context": { "includeDeclaration": true } }),
    );
    let locs = locations_by_file(&r);
    let files: std::collections::BTreeSet<&str> = locs.iter().map(|(u, _)| u.as_str()).collect();
    assert_eq!(
        files,
        [
            p.uri("src/concepts.bdl"),
            p.uri("src/lamp.bdl"),
            p.uri("src/main.bdl")
        ]
        .iter()
        .map(String::as_str)
        .collect(),
        "{locs:?}"
    );
    assert!(locs.len() >= 6, "{locs:?}");

    // Rename by identity: edits land in all three files, each on the word.
    let w = c.request(
        lsp::request::Rename::METHOD,
        json!({ "textDocument": lamp, "position": at_tilt, "newName": "Lean" }),
    );
    let changes = w["changes"].as_object().expect("changes");
    assert_eq!(changes.len(), 3, "{w}");
    for (uri, edits) in changes {
        let text = if *uri == p.uri("src/concepts.bdl") {
            CONCEPTS
        } else if *uri == p.uri("src/lamp.bdl") {
            LAMP
        } else {
            assert_eq!(*uri, p.uri("src/main.bdl"));
            MAIN
        };
        for e in edits.as_array().expect("edits") {
            assert_eq!(e["newText"], "Lean");
            assert_eq!(range_text(text, &e["range"]), "Tilt", "{uri}");
        }
    }

    // Hover on the instance names its component; on the component, its
    // interface.
    let main = json!({ "uri": p.uri("src/main.bdl") });
    let h = c.request(
        lsp::request::HoverRequest::METHOD,
        json!({ "textDocument": main, "position": position(MAIN, "instance lampA", 10) }),
    );
    let md = h["contents"]["value"].as_str().expect("md");
    assert!(md.contains("lampA") && md.contains("AdaptiveLamp"), "{md}");
    let h = c.request(
        lsp::request::HoverRequest::METHOD,
        json!({ "textDocument": lamp, "position": position(LAMP, "component AdaptiveLamp", 12) }),
    );
    let md = h["contents"]["value"].as_str().expect("md");
    assert!(
        md.contains("AdaptiveLamp") && md.contains("tiltValue"),
        "{md}"
    );

    // Definition of the component from its instance crosses into lamp.bdl.
    let d = c.request(
        lsp::request::GotoDefinition::METHOD,
        json!({ "textDocument": main, "position": position(MAIN, ": AdaptiveLamp {", 3) }),
    );
    assert_eq!(d[0]["uri"], p.uri("src/lamp.bdl"), "{d}");
    assert_eq!(range_text(LAMP, &d[0]["range"]), "AdaptiveLamp");

    // Rename the component, a port and an instance: each plan lands on
    // the declaration and every reference, across files.
    let rename = |c: &mut Client, td: &Value, pos: Position, new: &str| -> Vec<(String, String)> {
        let w = c.request(
            lsp::request::Rename::METHOD,
            json!({ "textDocument": td, "position": pos, "newName": new }),
        );
        let mut out = Vec::new();
        for (uri, edits) in w["changes"].as_object().expect("changes") {
            let text = if *uri == p.uri("src/concepts.bdl") {
                CONCEPTS
            } else if *uri == p.uri("src/lamp.bdl") {
                LAMP
            } else {
                MAIN
            };
            for e in edits.as_array().expect("edits") {
                assert_eq!(e["newText"], new);
                out.push((
                    uri.rsplit('/').next().expect("file").to_owned(),
                    range_text(text, &e["range"]).to_owned(),
                ));
            }
        }
        out.sort();
        out
    };
    let edits = rename(&mut c, &main, position(MAIN, ": AdaptiveLamp {", 3), "Lamp");
    assert_eq!(
        edits,
        vec![
            ("lamp.bdl".to_owned(), "AdaptiveLamp".to_owned()),
            ("main.bdl".to_owned(), "AdaptiveLamp".to_owned()),
            ("main.bdl".to_owned(), "AdaptiveLamp".to_owned()),
        ],
        "{edits:?}"
    );
    let edits = rename(
        &mut c,
        &lamp,
        position(LAMP, "requires tiltValue", 10),
        "tiltIn",
    );
    assert_eq!(
        edits,
        vec![
            ("lamp.bdl".to_owned(), "tiltValue".to_owned()),
            ("lamp.bdl".to_owned(), "tiltValue".to_owned()),
            ("main.bdl".to_owned(), "tiltValue".to_owned()),
            ("main.bdl".to_owned(), "tiltValue".to_owned()),
        ],
        "{edits:?}"
    );
    let edits = rename(&mut c, &main, position(MAIN, "instance lampA", 10), "first");
    assert_eq!(
        edits,
        vec![
            ("main.bdl".to_owned(), "lampA".to_owned()),
            ("main.bdl".to_owned(), "lampA".to_owned()),
            ("main.bdl".to_owned(), "lampA".to_owned()),
        ],
        "{edits:?}"
    );

    let edits = rename(
        &mut c,
        &main,
        position(MAIN, "mapping tiltValue", 8),
        "tiltNow",
    );
    assert_eq!(
        edits,
        vec![
            ("main.bdl".to_owned(), "tiltValue".to_owned()),
            ("main.bdl".to_owned(), "tiltValue".to_owned()),
            ("main.bdl".to_owned(), "tiltValue".to_owned()),
            ("main.bdl".to_owned(), "tiltValue".to_owned()),
        ],
        "{edits:?}"
    );
    // Inside a formula body, a name that calls a mapping of the component
    // hovers as that mapping and defines to its declaration.
    let at_call = position(LAMP, "= dimByTilt(tiltValue)", 2);
    let h = c.request(
        lsp::request::HoverRequest::METHOD,
        json!({ "textDocument": lamp, "position": at_call }),
    );
    let md = h["contents"]["value"].as_str().expect("md");
    assert!(md.contains("dimByTilt"), "{md}");
    let d = c.request(
        lsp::request::GotoDefinition::METHOD,
        json!({ "textDocument": lamp, "position": at_call }),
    );
    assert_eq!(d[0]["uri"], p.uri("src/lamp.bdl"), "{d}");
    assert_eq!(range_text(LAMP, &d[0]["range"]), "dimByTilt");

    // Semantic tokens for a file that was never opened.
    let toks = c.request(
        lsp::request::SemanticTokensFullRequest::METHOD,
        json!({ "textDocument": lamp }),
    );
    let data = toks["data"].as_array().expect("data");
    assert!(
        data.len() % 5 == 0 && data.len() >= 5 * 20,
        "{}",
        data.len()
    );

    // Diagnostics: the project is complete, so nothing is an error.
    for rel in ["src/concepts.bdl", "src/lamp.bdl", "src/main.bdl"] {
        let diag = c.request(
            lsp::request::DocumentDiagnosticRequest::METHOD,
            json!({ "textDocument": { "uri": p.uri(rel) } }),
        );
        let items = diag["items"].as_array().expect("items");
        assert!(items.iter().all(|d| d["severity"] != 1), "{rel}: {items:?}");
    }
    c.shutdown();
}

#[test]
fn open_buffers_substitute_and_saves_reload_the_ground() {
    let p = Project::create();
    let mut c = Client::start(&p.root);
    let main_uri = p.uri("src/main.bdl");
    let main = json!({ "uri": main_uri });

    // A buffer that names a concept nobody declares is a load fault in
    // that file only, and the other files still analyse.
    let broken = MAIN.replace("mapping mirror : Brightness", "mapping mirror : Glow");
    c.notify(
        lsp::notification::DidOpenTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "languageId": "bdl", "version": 1, "text": broken } }),
    );
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": main }),
    );
    let items = diag["items"].as_array().expect("items");
    let errors: Vec<&Value> = items.iter().filter(|d| d["severity"] == 1).collect();
    assert!(!errors.is_empty(), "{items:?}");
    assert!(
        errors
            .iter()
            .any(|e| range_text(&broken, &e["range"]).contains("Glow")),
        "{errors:?}"
    );
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": { "uri": p.uri("src/lamp.bdl") } }),
    );
    assert!(diag["items"]
        .as_array()
        .expect("items")
        .iter()
        .all(|d| d["severity"] != 1));

    // Completion after `mapping mirror : ` offers the concepts of every
    // file, not only this one.
    let comp = c.request(
        lsp::request::Completion::METHOD,
        json!({ "textDocument": main, "position": position(&broken, "mirror : Glow", "mirror : ".len()) }),
    );
    let labels: Vec<&str> = comp
        .as_array()
        .or_else(|| comp["items"].as_array())
        .expect("items")
        .iter()
        .filter_map(|i| i["label"].as_str())
        .collect();
    assert!(
        labels.contains(&"Gain") && labels.contains(&"Brightness"),
        "{labels:?}"
    );

    // Completion knows the scope of the position from the authored
    // system, not from the file it is in.
    let labels_at = |c: &mut Client,
                     uri: &str,
                     text: &str,
                     needle: &str,
                     plus: usize|
     -> Vec<String> {
        c.notify(
            lsp::notification::DidChangeTextDocument::METHOD,
            json!({ "textDocument": { "uri": uri, "version": 9 }, "contentChanges": [{ "text": text }] }),
        );
        let comp = c.request(
            lsp::request::Completion::METHOD,
            json!({ "textDocument": { "uri": uri }, "position": position(text, needle, plus) }),
        );
        comp.as_array()
            .or_else(|| comp["items"].as_array())
            .expect("items")
            .iter()
            .filter_map(|i| i["label"].as_str().map(str::to_owned))
            .collect()
    };
    // Module level, at an item start: the project items.
    let with_blank = format!("{MAIN}\n\n");
    let labels = labels_at(
        &mut c,
        &main_uri,
        &with_blank,
        "drive light = brightness\n\n",
        "drive light = brightness\n\n".len(),
    );
    assert!(
        labels.contains(&"instance".into()) && labels.contains(&"bind".into()),
        "{labels:?}"
    );
    assert!(!labels.contains(&"requires".into()), "{labels:?}");
    // `bind lampA.|` → that instance's ports.
    let text = format!("{MAIN}bind lampA.\n");
    let labels = labels_at(
        &mut c,
        &main_uri,
        &text,
        "bind lampA.\n",
        "bind lampA.".len(),
    );
    // Parameters are given at instantiation, not bound.
    assert_eq!(labels, ["brightness", "tiltValue"], "{labels:?}");
    // `instance x : |` → components.
    let text = format!("{MAIN}instance lampC : \n");
    let labels = labels_at(
        &mut c,
        &main_uri,
        &text,
        "instance lampC : \n",
        "instance lampC : ".len(),
    );
    assert_eq!(labels, vec!["AdaptiveLamp".to_owned()], "{labels:?}");
    // `instance lampC : AdaptiveLamp { |` → its arguments.
    let text = format!("{MAIN}instance lampC : AdaptiveLamp {{ \n");
    let labels = labels_at(
        &mut c,
        &main_uri,
        &text,
        "AdaptiveLamp { \n",
        "AdaptiveLamp { ".len(),
    );
    assert_eq!(
        labels,
        vec!["gain".to_owned(), "main".to_owned()],
        "{labels:?}"
    );
    // Inside the component: an item start offers body items, `@` the
    // body's clocks, a type position the concepts the body knows.
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "version": 10 }, "contentChanges": [{ "text": MAIN }] }),
    );
    let lamp_uri = p.uri("src/lamp.bdl");
    c.notify(
        lsp::notification::DidOpenTextDocument::METHOD,
        json!({ "textDocument": { "uri": lamp_uri, "languageId": "bdl", "version": 1, "text": LAMP } }),
    );
    let labels = labels_at(
        &mut c,
        &lamp_uri,
        LAMP,
        "param clock main\n\n",
        "param clock main\n\n".len(),
    );
    assert!(
        labels.contains(&"requires".into()) && labels.contains(&"use".into()),
        "{labels:?}"
    );
    assert!(!labels.contains(&"instance".into()), "{labels:?}");
    let text = LAMP.replace(
        "provides brightness : Brightness @main",
        "provides brightness : Brightness @",
    );
    let labels = labels_at(
        &mut c,
        &lamp_uri,
        &text,
        "Brightness @\n",
        "Brightness @".len(),
    );
    assert_eq!(labels, vec!["main".to_owned()], "{labels:?}");
    let text = LAMP.replace(
        "provides brightness : Brightness @main",
        "provides brightness : ",
    );
    let labels = labels_at(
        &mut c,
        &lamp_uri,
        &text,
        "provides brightness : \n",
        "provides brightness : ".len(),
    );
    assert_eq!(labels, ["Brightness", "Gain", "Tilt"], "{labels:?}");
    // In a body formula: the body's relationships by the component's
    // names — one `dimByTilt`, not one per instance — and its inputs.
    let text = LAMP.replace("dimByTilt(tiltValue) * gain", "dimByTilt(tiltValue) * ");
    let labels = labels_at(
        &mut c,
        &lamp_uri,
        &text,
        "dimByTilt(tiltValue) * \n",
        "dimByTilt(tiltValue) * ".len(),
    );
    assert!(
        labels.contains(&"gain".into()) && labels.contains(&"tiltValue".into()),
        "{labels:?}"
    );
    assert_eq!(
        labels.iter().filter(|l| l.starts_with("dimByTilt")).count(),
        1,
        "{labels:?}"
    );
    assert!(!labels.iter().any(|l| l.contains("lampA")), "{labels:?}");
    c.notify(
        lsp::notification::DidCloseTextDocument::METHOD,
        json!({ "textDocument": { "uri": lamp_uri } }),
    );

    // Formatting: canonical spacing, comments kept; a canonical file
    // gets no edit; a file with a syntax error is left alone.
    let messy = "mapping   tilt:Tilt @interaction // outside\n".to_owned()
        + &MAIN["mapping tilt : Tilt @interaction\n".len()..];
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "version": 20 }, "contentChanges": [{ "text": messy }] }),
    );
    let fmt = c.request(
        lsp::request::Formatting::METHOD,
        json!({ "textDocument": main, "options": { "tabSize": 2, "insertSpaces": true } }),
    );
    let edits = fmt.as_array().expect("edits");
    assert_eq!(edits.len(), 1, "{fmt}");
    let formatted = edits[0]["newText"].as_str().expect("text");
    assert!(
        formatted.starts_with("mapping tilt : Tilt @interaction  // outside\n"),
        "{formatted}"
    );
    assert_eq!(
        &formatted["mapping tilt : Tilt @interaction  // outside\n".len()..],
        &MAIN["mapping tilt : Tilt @interaction\n".len()..]
    );
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "version": 21 }, "contentChanges": [{ "text": MAIN }] }),
    );
    let fmt = c.request(
        lsp::request::Formatting::METHOD,
        json!({ "textDocument": main, "options": { "tabSize": 2, "insertSpaces": true } }),
    );
    assert_eq!(fmt.as_array().map(Vec::len), Some(0), "{fmt}");

    // Inlay hints: the concept a parameter reads (`t: Tilt` in the
    // component), the transport of a binding that crosses domains.
    let lamp_uri = p.uri("src/lamp.bdl");
    let hints = c.request(
        lsp::request::InlayHintRequest::METHOD,
        json!({ "textDocument": { "uri": lamp_uri }, "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 40, "character": 0 } } }),
    );
    let labels: Vec<(u64, String)> = hints
        .as_array()
        .expect("hints")
        .iter()
        .map(|h| {
            (
                h["position"]["line"].as_u64().expect("line"),
                h["label"].as_str().expect("label").to_owned(),
            )
        })
        .collect();
    let t_line = position(LAMP, "dimByTilt(t) =", 0).line as u64;
    assert_eq!(labels, vec![(t_line, ": Tilt".to_owned())], "{labels:?}");
    let with_sync = format!("{MAIN}clock display\nmapping slow : Brightness @display\nbind slow = lampA.brightness init 0\n");
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "version": 22 }, "contentChanges": [{ "text": with_sync }] }),
    );
    let hints = c.request(
        lsp::request::InlayHintRequest::METHOD,
        json!({ "textDocument": main, "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 40, "character": 0 } } }),
    );
    let labels: Vec<String> = hints
        .as_array()
        .expect("hints")
        .iter()
        .map(|h| h["label"].as_str().expect("label").to_owned())
        .collect();
    assert_eq!(
        labels,
        vec!["sync interaction → display, init 0".to_owned()],
        "{labels:?}"
    );

    // Virtual documents: explain, Core, Rust.
    let core = c.request(
        bdl_lsp::VirtualDocumentRequest::METHOD,
        json!({ "kind": "core" }),
    );
    assert_eq!(core["language"], "bdl-core");
    let text = core["text"].as_str().expect("text");
    assert!(
        text.contains("sem Tilt")
            && text.contains("decl tiltValue")
            && text.contains("@ interaction"),
        "{text}"
    );
    let rust = c.request(
        bdl_lsp::VirtualDocumentRequest::METHOD,
        json!({ "kind": "rust" }),
    );
    assert_eq!(rust["language"], "rust");
    assert!(
        rust["text"]
            .as_str()
            .expect("text")
            .contains("bdl_runtime_core"),
        "{}",
        rust["text"]
    );
    let ex = c.request(
        bdl_lsp::VirtualDocumentRequest::METHOD,
        json!({ "kind": "explain", "textDocument": main, "position": position(&with_sync, "mapping tiltValue", 8) }),
    );
    assert_eq!(ex["language"], "markdown");
    assert!(
        ex["text"]
            .as_str()
            .expect("text")
            .starts_with("# tiltValue"),
        "{}",
        ex["text"]
    );
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "version": 23 }, "contentChanges": [{ "text": MAIN }] }),
    );

    // Fix the buffer, save it to disk, tell the server: the ground is
    // re-read and the identity table written for the next tool.
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri, "version": 2 }, "contentChanges": [{ "text": MAIN }] }),
    );
    std::fs::write(p.root.join("src/main.bdl"), MAIN).expect("write");
    c.notify(
        lsp::notification::DidSaveTextDocument::METHOD,
        json!({ "textDocument": { "uri": main_uri } }),
    );
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": main }),
    );
    assert!(diag["items"]
        .as_array()
        .expect("items")
        .iter()
        .all(|d| d["severity"] != 1));
    let table: bdl_text::IdentityTable =
        serde_json::from_str(&p.text(".bdl/identities.json")).expect("table");
    assert!(
        table.keys.contains_key("concept:Tilt"),
        "{:?}",
        table.keys.keys()
    );
    let tilt_id = table.keys["concept:Tilt"].id;

    // Another tool edits concepts.bdl on disk (a renamed concept); the
    // watched-files notification reloads it and the identity survives
    // the rename because the rest of the file is unchanged.
    std::fs::write(
        p.root.join("src/concepts.bdl"),
        CONCEPTS.replace("concept Tilt", "concept Lean"),
    )
    .expect("write");
    c.notify(
        lsp::notification::DidChangeWatchedFiles::METHOD,
        json!({ "changes": [{ "uri": p.uri("src/concepts.bdl"), "type": 2 }] }),
    );
    // The saved buffer of main.bdl is stale on its own: `Tilt` is gone.
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": main }),
    );
    let items = diag["items"].as_array().expect("items");
    assert!(items.iter().any(|d| d["severity"] == 1), "{items:?}");
    let table: bdl_text::IdentityTable =
        serde_json::from_str(&p.text(".bdl/identities.json")).expect("table");
    assert!(!table.keys.contains_key("concept:Tilt"));
    assert_eq!(table.keys["concept:Lean"].id, tilt_id, "{:?}", table.keys);
    c.shutdown();
}
