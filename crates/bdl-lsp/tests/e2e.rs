//! The adapter end to end over an in-memory connection: a client sends
//! JSON-RPC, the server answers from `bdl-ide`.  Positions are UTF-16
//! (the default) and the document has non-ASCII comments so the
//! conversion is exercised, not assumed.

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{self as lsp, Position, Uri};
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::Duration;

const SRC: &str = "// Lampe — ångström ✓\nconcept Tilt : Angle\nconcept Brightness : Scalar\n\n// 日本語のコメント\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(tilt) =\n  tilt / (90 deg)\n\nmapping broken : Tilt -> Brightness\nbroken(tilt) =\n  tilt + 1 s\n";

struct Client {
    conn: Connection,
    next: i32,
    server: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}

impl Client {
    fn start(root: Option<&std::path::Path>, pull: bool) -> Client {
        let (server_conn, client_conn) = Connection::memory();
        let server = std::thread::spawn(move || bdl_lsp::run(server_conn));
        let mut c = Client {
            conn: client_conn,
            next: 0,
            server: Some(server),
        };
        let mut caps = json!({
            "general": { "positionEncodings": ["utf-16"] },
            "textDocument": {}
        });
        if pull {
            caps["textDocument"]["diagnostic"] = json!({ "dynamicRegistration": false });
        }
        let init = c.request(
            lsp::request::Initialize::METHOD,
            json!({
                "processId": null,
                "capabilities": caps,
                "initializationOptions": { "projectRoot": root.map(|r| r.to_string_lossy().into_owned()) },
            }),
        );
        assert_eq!(init["capabilities"]["positionEncoding"], "utf-16");
        assert!(init["capabilities"]["diagnosticProvider"].is_object());
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

    fn request_err(&mut self, method: &str, params: Value) -> lsp_server::ResponseError {
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
                    return response_result.expect_err("an error");
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

    fn next_notification(&self, method: &str) -> Value {
        loop {
            match self.conn.receiver.recv_timeout(Duration::from_secs(20)) {
                Ok(Message::Notification(n)) if n.method == method => return n.params,
                Ok(_) => continue,
                Err(e) => panic!("no {method}: {e}"),
            }
        }
    }

    fn open(&self, uri: &str, text: &str) {
        self.notify(
            lsp::notification::DidOpenTextDocument::METHOD,
            json!({ "textDocument": { "uri": uri, "languageId": "bdl", "version": 1, "text": text } }),
        );
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

fn utf16_position(text: &str, needle: &str, plus: usize) -> Position {
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
        let line_text = &text[line_start..];
        let mut units = 0;
        let mut bytes = 0;
        for c in line_text.chars() {
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

#[test]
fn hover_definition_references_rename_completion_diagnostics_symbols_tokens() {
    let mut c = Client::start(None, true);
    let uri = "file:///tmp/lamp.bdl";
    c.open(uri, SRC);
    let td = json!({ "uri": uri });

    // Hover on the use of Tilt inside dimByTilt's signature (non-ASCII
    // comments above shift nothing: UTF-16 columns are per line).
    let at_tilt_use = utf16_position(SRC, "dimByTilt : Tilt", "dimByTilt : ".len() + 1);
    let h = c.request(
        lsp::request::HoverRequest::METHOD,
        json!({ "textDocument": td, "position": at_tilt_use }),
    );
    let md = h["contents"]["value"].as_str().expect("markdown");
    assert!(md.contains("concept Tilt : Angle"), "{md}");
    assert!(md.contains("an angle"));

    // Definition of that use is the declaration line.
    let d = c.request(
        lsp::request::GotoDefinition::METHOD,
        json!({ "textDocument": td, "position": at_tilt_use }),
    );
    assert_eq!(d.as_array().map(Vec::len), Some(1));
    assert_eq!(range_text(SRC, &d[0]["range"]), "Tilt");
    assert_eq!(d[0]["range"]["start"]["line"], 1);

    // References to Tilt: two signature uses and two body uses (`tilt`,
    // resolved case-insensitively by the elaborator's rule); with the
    // declaration when asked.
    let r = c.request(
        lsp::request::References::METHOD,
        json!({ "textDocument": td, "position": at_tilt_use, "context": { "includeDeclaration": true } }),
    );
    let refs: Vec<&str> = r
        .as_array()
        .expect("array")
        .iter()
        .map(|l| range_text(SRC, &l["range"]))
        .collect();
    assert_eq!(
        refs,
        vec!["Tilt", "Tilt", "tilt", "Tilt", "tilt"],
        "{refs:?}"
    );

    // Prepare + rename by identity: the plan touches every site above,
    // not the word `Tilt` in `dimByTilt`.
    let p = c.request(
        lsp::request::PrepareRenameRequest::METHOD,
        json!({ "textDocument": td, "position": at_tilt_use }),
    );
    assert_eq!(p["placeholder"], "Tilt");
    let w = c.request(
        lsp::request::Rename::METHOD,
        json!({ "textDocument": td, "position": at_tilt_use, "newName": "Lean" }),
    );
    let edits = w["changes"][uri]
        .as_array()
        .expect("edits for the document");
    assert_eq!(edits.len(), 5);
    for e in edits {
        assert_eq!(e["newText"], "Lean");
        assert!(matches!(range_text(SRC, &e["range"]), "Tilt" | "tilt"));
    }

    // Completion inside the body of dimByTilt, after `tilt / (90 d`.
    let at_unit = utf16_position(SRC, "(90 deg)", "(90 d".len());
    let comp = c.request(
        lsp::request::Completion::METHOD,
        json!({ "textDocument": td, "position": at_unit }),
    );
    let labels: Vec<&str> = comp["items"]
        .as_array()
        .or_else(|| comp.as_array())
        .expect("items")
        .iter()
        .map(|i| i["label"].as_str().expect("label"))
        .collect();
    assert!(labels.contains(&"deg"), "{labels:?}");
    let deg = comp
        .as_array()
        .expect("array")
        .iter()
        .find(|i| i["label"] == "deg")
        .expect("deg");
    assert_eq!(range_text(SRC, &deg["textEdit"]["range"]), "d");

    // Pull diagnostics: the dimension error in `broken`, and nothing about
    // `dimByTilt` (it is complete) — no error for anything legal.
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": td }),
    );
    assert_eq!(diag["kind"], "full");
    let items = diag["items"].as_array().expect("items");
    let errors: Vec<&Value> = items.iter().filter(|d| d["severity"] == 1).collect();
    assert_eq!(errors.len(), 1, "{items:?}");
    assert_eq!(errors[0]["code"], "dimension.mismatch");
    assert_eq!(range_text(SRC, &errors[0]["range"]), "tilt + 1 s");
    assert_eq!(errors[0]["source"], "bdl");

    // Symbols come from the model: two concepts, two mappings.
    let syms = c.request(
        lsp::request::DocumentSymbolRequest::METHOD,
        json!({ "textDocument": td }),
    );
    let names: Vec<&str> = syms
        .as_array()
        .expect("symbols")
        .iter()
        .map(|s| s["name"].as_str().expect("name"))
        .collect();
    assert_eq!(names, vec!["Tilt", "Brightness", "dimByTilt", "broken"]);
    assert!(syms[3]["detail"]
        .as_str()
        .expect("detail")
        .contains("does not check"));

    // Semantic tokens: delta-encoded, comments included, non-empty.
    let toks = c.request(
        lsp::request::SemanticTokensFullRequest::METHOD,
        json!({ "textDocument": td }),
    );
    let data = toks["data"].as_array().expect("data");
    assert!(
        data.len() % 5 == 0 && data.len() >= 5 * 10,
        "{}",
        data.len()
    );

    // Code actions on the dimension error: none are silent fixes, and
    // the ones that exist carry the diagnostic.
    let actions = c.request(
        lsp::request::CodeActionRequest::METHOD,
        json!({ "textDocument": td, "range": errors[0]["range"], "context": { "diagnostics": [] } }),
    );
    assert!(actions.is_array());

    // Custom: explain.
    let ex = c.request(
        bdl_lsp::ExplainEntity::METHOD,
        json!({ "textDocument": td, "position": utf16_position(SRC, "mapping broken", 10) }),
    );
    assert!(ex["markdown"].as_str().expect("md").contains("# broken"));
    assert!(ex["explanation"]["sections"].is_array());

    // Custom: invalidation preview of a model edit (a rename is a
    // refinement).
    let inv = c.request(
        bdl_lsp::InvalidationPreviewRequest::METHOD,
        json!({ "edit": { "op": "rename_concept", "id": 0, "name": "Lean" } }),
    );
    assert_eq!(inv["kind"], "refinement");

    // Edit the document: the diagnostic follows the new text; close it:
    // the document is unknown and answers are empty, not errors.
    let fixed = SRC.replace("tilt + 1 s", "tilt / (45 deg)");
    c.notify(
        lsp::notification::DidChangeTextDocument::METHOD,
        json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": fixed }] }),
    );
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": td }),
    );
    assert!(diag["items"]
        .as_array()
        .expect("items")
        .iter()
        .all(|d| d["severity"] != 1));
    c.notify(
        lsp::notification::DidCloseTextDocument::METHOD,
        json!({ "textDocument": { "uri": uri } }),
    );
    let h = c.request(
        lsp::request::HoverRequest::METHOD,
        json!({ "textDocument": td, "position": at_tilt_use }),
    );
    assert!(h.is_null());
    let unknown = c.request_err("bdl/noSuchMethod", json!({}));
    assert_eq!(unknown.code, lsp_server::ErrorCode::MethodNotFound as i32);
    c.shutdown();
}

#[test]
fn push_diagnostics_only_for_clients_without_pull_support() {
    let c = Client::start(None, false);
    let uri = "file:///tmp/lamp.bdl";
    c.open(uri, SRC);
    let published = c.next_notification(lsp::notification::PublishDiagnostics::METHOD);
    assert_eq!(published["uri"], uri);
    let items = published["diagnostics"].as_array().expect("items");
    assert!(items.iter().any(|d| d["code"] == "dimension.mismatch"));
    c.shutdown();
}

#[test]
fn opens_a_project_directory_as_committed_state() {
    // A legacy JSON project is migrated on open (ADR-0023 §6): its sources
    // become the ground, and a new source buffer sees what they declare.
    let dir = tempfile::tempdir().expect("tempdir");
    let created = bdl_model::persist::init_project(dir.path(), "lamp", "test").expect("init");
    let s = created.snapshot;
    let s = bdl_model::apply_edit(
        &s,
        &bdl_model::EditOp::CreateConcept {
            name: "Held".into(),
            description: String::new(),
            representation: Some(bdl_model::Representation::Boolean),
        },
    )
    .expect("edit")
    .snapshot;
    bdl_model::persist::save_project(dir.path(), &s, &created.layout, "test").expect("save");

    let mut c = Client::start(Some(dir.path()), true);
    assert!(
        dir.path().join("src/main.bdl").is_file(),
        "migrated on open"
    );
    assert!(!dir.path().join("design/project.bdl.json").exists());
    let uri = format!("file://{}/src/extra.bdl", dir.path().display());
    // The buffer names the committed concept without declaring it:
    // binding finds it in the project.
    c.open(&uri, "mapping isHeld : Held -> Held\n");
    let diag = c.request(
        lsp::request::DocumentDiagnosticRequest::METHOD,
        json!({ "textDocument": { "uri": uri } }),
    );
    assert!(
        diag["items"]
            .as_array()
            .expect("items")
            .iter()
            .all(|d| d["severity"] != 1),
        "{diag}"
    );
    let h = c.request(
        lsp::request::HoverRequest::METHOD,
        json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 18 } }),
    );
    assert!(h["contents"]["value"]
        .as_str()
        .expect("md")
        .contains("concept Held : Bool"));
    c.shutdown();
    let _ = Uri::from_str(&uri);
}
