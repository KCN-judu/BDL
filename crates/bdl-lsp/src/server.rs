//! The LSP message loop: JSON-RPC in, `bdl-ide` queries, JSON-RPC out.
//!
//! One thread reads messages (`lsp-server`), mutates the [`IdeHost`] on
//! notifications (`didOpen`, `didChange`, `didClose` are overlay updates;
//! `$/cancelRequest` cancels), and hands every request to a worker thread
//! that takes a snapshot, runs the query, and answers — unless the
//! request was cancelled meanwhile, in which case it answers
//! `RequestCancelled`.  Snapshots are immutable, so a worker never sees a
//! half-updated host; the host is locked only while a snapshot is taken
//! or an overlay changes.
//!
//! Diagnostics are *pulled* (`textDocument/diagnostic`).  Push
//! (`publishDiagnostics`) is used only for clients that do not advertise
//! pull support, and only from the notification path, so the pull path
//! stays the model.

use crate::convert;
use crate::position::{LineIndex, PositionEncoding};
use bdl_ide::{
    actions_at, actions_for, completion, definition_at, diagnostics, document_symbols, entity_at,
    explain, format_document, hover_at, inlay_hints, plan_rename, preview_change,
    project_to_document, references_at, semantic_tokens, virtual_document, CompletionContext,
    DiagnosticScope, EntityRole, IdeHost,
};
use bdl_ide_db::{CancelScope, DocumentId, DocumentUri, OverlayKey};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{
    self as lsp, CodeActionProviderCapability, DiagnosticOptions, DiagnosticServerCapabilities,
    GotoDefinitionResponse, InitializeParams, OneOf, RenameOptions, SemanticTokensFullOptions,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Uri, WorkDoneProgressOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

pub const SERVER_NAME: &str = "bdl-lsp";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// LSP-level `RequestCancelled` (JSON-RPC reserved range).
const REQUEST_CANCELLED: i32 = -32800;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializationOptions {
    /// The BDL project directory (holds `bdl.toml`).  Defaults to the
    /// workspace root.
    pub project_root: Option<PathBuf>,
}

// ---- custom requests (kept minimal; see docs/architecture/ide-service.md)

/// `bdl/explainEntity`: the explanation of the entity at a position.
pub enum ExplainEntity {}
impl lsp::request::Request for ExplainEntity {
    type Params = lsp::TextDocumentPositionParams;
    type Result = Option<ExplainResult>;
    const METHOD: &'static str = "bdl/explainEntity";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplainResult {
    pub explanation: bdl_ide::Explanation,
    pub markdown: String,
}

/// `bdl/invalidationPreview`: what a model edit would reopen.
pub enum InvalidationPreviewRequest {}
impl lsp::request::Request for InvalidationPreviewRequest {
    type Params = InvalidationPreviewParams;
    type Result = bdl_ide::InvalidationPreview;
    const METHOD: &'static str = "bdl/invalidationPreview";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvalidationPreviewParams {
    pub edit: bdl_model::EditOp,
}

/// `bdl/previewEdit`: the full semantic plan of a rename at a position
/// (model operations included, which a `WorkspaceEdit` cannot carry).
pub enum PreviewEdit {}
impl lsp::request::Request for PreviewEdit {
    type Params = lsp::RenameParams;
    type Result = Option<bdl_ide::SemanticEditPlan>;
    const METHOD: &'static str = "bdl/previewEdit";
}

/// `bdl/virtualDocument`: a read-only rendering — the explanation of
/// the entity at a position (or of the whole project), the kernel Core,
/// or the generated Rust.
pub enum VirtualDocumentRequest {}
impl lsp::request::Request for VirtualDocumentRequest {
    type Params = VirtualDocumentParams;
    type Result = VirtualDocumentResult;
    const METHOD: &'static str = "bdl/virtualDocument";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualDocumentParams {
    pub kind: bdl_ide::VirtualKind,
    /// Narrows an explanation to the entity at this position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_document: Option<lsp::TextDocumentIdentifier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<lsp::Position>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualDocumentResult {
    pub uri: String,
    pub language: String,
    pub text: String,
}

// ---- server ---------------------------------------------------------------------

struct Shared {
    host: Mutex<IdeHost>,
    encoding: PositionEncoding,
    /// The text project's root when the workspace is one (ADR-0020):
    /// `file://` URIs under `root/src` are the workspace's documents.
    text_root: Option<PathBuf>,
}

pub struct Server {
    connection: Connection,
    shared: Arc<Shared>,
    pull_diagnostics: bool,
    /// LSP request id → the host's request id, for `$/cancelRequest`.
    inflight: Arc<Mutex<HashMap<RequestId, bdl_ide_db::RequestId>>>,
}

/// Run the server over a connection until `shutdown`/`exit`.  Performs the
/// initialize handshake itself.
pub fn run(connection: Connection) -> anyhow::Result<()> {
    let (id, params) = connection.initialize_start()?;
    let params: InitializeParams = serde_json::from_value(params)?;
    let encoding = PositionEncoding::negotiate(
        params
            .capabilities
            .general
            .as_ref()
            .and_then(|g| g.position_encodings.as_deref()),
    );
    let pull_diagnostics = params
        .capabilities
        .text_document
        .as_ref()
        .and_then(|t| t.diagnostic.as_ref())
        .is_some();
    let options: InitializationOptions = params
        .initialization_options
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .unwrap_or_default()
        .unwrap_or_default();
    // Workspace folders first; `rootUri` only for older clients.
    #[allow(deprecated)]
    let root = options.project_root.or_else(|| {
        params
            .workspace_folders
            .as_ref()
            .and_then(|f| f.first())
            .and_then(|f| uri_to_path(&f.uri))
            .or_else(|| params.root_uri.as_ref().and_then(uri_to_path))
    });
    let host = open_host(root.as_deref());
    let text_root = if host.is_text_workspace() {
        root.clone()
    } else {
        None
    };

    let capabilities = ServerCapabilities {
        position_encoding: Some(encoding.kind()),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            lsp::TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: Some(lsp::TextDocumentSyncSaveOptions::SaveOptions(
                    lsp::SaveOptions {
                        include_text: Some(false),
                    },
                )),
            },
        )),
        hover_provider: Some(lsp::HoverProviderCapability::Simple(true)),
        completion_provider: Some(lsp::CompletionOptions {
            trigger_characters: Some(vec![":".into(), ">".into(), " ".into()]),
            ..Default::default()
        }),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        document_symbol_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        inlay_hint_provider: Some(OneOf::Left(true)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: Some(convert::SOURCE.into()),
            inter_file_dependencies: true,
            workspace_diagnostics: false,
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                legend: convert::legend(),
                range: Some(false),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        ..Default::default()
    };
    let result = lsp::InitializeResult {
        capabilities,
        server_info: Some(lsp::ServerInfo {
            name: SERVER_NAME.into(),
            version: Some(SERVER_VERSION.into()),
        }),
    };
    connection.initialize_finish(id, serde_json::to_value(result)?)?;
    info!(?encoding, pull_diagnostics, "initialized");

    let server = Server {
        connection,
        shared: Arc::new(Shared {
            host: Mutex::new(host),
            encoding,
            text_root,
        }),
        pull_diagnostics,
        inflight: Arc::new(Mutex::new(HashMap::new())),
    };
    server.main_loop()
}

/// The committed state: the project at `root` when it holds one, else an
/// empty design that open documents populate.
fn open_host(root: Option<&Path>) -> IdeHost {
    if let Some(root) = root {
        if root.join("bdl.toml").is_file() {
            // One kind of project (ADR-0023): the sources are the truth; a
            // legacy JSON project is migrated in place by the loader.
            match text_ground(root) {
                Ok((name, files, table)) => {
                    info!(root = %root.display(), files = files.len(), "opened project");
                    return IdeHost::text_workspace(&name, files, table);
                }
                Err(e) => {
                    warn!(root = %root.display(), error = %e, "could not load the project; starting empty")
                }
            }
        }
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "workspace".into());
        return IdeHost::empty(&name);
    }
    IdeHost::empty("workspace")
}

/// The sources and identity table of a project on disk, migrating a
/// legacy JSON project first (ADR-0023 §6).
fn text_ground(
    root: &Path,
) -> Result<(String, Vec<bdl_text::SourceFile>, bdl_text::IdentityTable), bdl_text::TextError> {
    bdl_text::migrate_legacy(root, env!("CARGO_PKG_VERSION"))?;
    let manifest = bdl_model::persist::read_manifest(root)?;
    let files = bdl_text::discover_sources(root)?;
    let table = bdl_text::workspace::load_identities(root)?;
    Ok((manifest.name, files, table))
}

/// A `file://` URI for an absolute path (spaces and `%` escaped).
/// A `file://` URI for an absolute path.  A Windows path `C:\a\b` is
/// `file:///C:/a/b`: forward slashes, one leading slash before the drive.
fn path_to_uri(path: &Path) -> Option<Uri> {
    let s = path.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !s.starts_with('/') {
        out.push('/');
    }
    for ch in s.chars() {
        match ch {
            ' ' => out.push_str("%20"),
            '%' => out.push_str("%25"),
            '#' => out.push_str("%23"),
            '?' => out.push_str("%3F"),
            c => out.push(c),
        }
    }
    Uri::from_str(&out).ok()
}

fn uri_to_path(uri: &Uri) -> Option<PathBuf> {
    if uri.scheme().map(|s| s.as_str()) != Some("file") {
        return None;
    }
    let path = uri.path().as_str();
    let decoded = percent_decode(path);
    // `/C:/a/b` is the URI form of a Windows drive path.
    let decoded = match decoded.as_bytes() {
        [b'/', drive, b':', b'/', ..] if drive.is_ascii_alphabetic() && cfg!(windows) => {
            decoded[1..].replace('/', "\\")
        }
        _ => decoded,
    };
    Some(PathBuf::from(decoded))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if let Some(h) = s
                .get(i + 1..i + 3)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                out.push(h);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The host's document for a client URI: a workspace source file is
/// `bdl-file:<relative path>` (so it is the same document whether or not
/// a buffer is open); anything else is the URI itself.
fn document_uri(text_root: Option<&Path>, uri: &Uri) -> DocumentUri {
    if let (Some(root), Some(path)) = (text_root, uri_to_path(uri)) {
        if let Ok(rel) = path.strip_prefix(root) {
            let rel: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            if rel.first().map(String::as_str) == Some(bdl_text::SOURCE_DIR)
                && path.extension().and_then(|e| e.to_str()) == Some("bdl")
            {
                return bdl_ide_db::workspace::file_uri(&rel.join("/"));
            }
        }
    }
    DocumentUri::new(uri.as_str())
}

impl Server {
    fn main_loop(self) -> anyhow::Result<()> {
        for msg in &self.connection.receiver {
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        info!("shutdown");
                        return Ok(());
                    }
                    self.dispatch_request(req);
                }
                Message::Notification(n) => self.handle_notification(n),
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    // ---- notifications: overlay updates and cancellation -------------------------

    fn handle_notification(&self, n: Notification) {
        match n.method.as_str() {
            lsp::notification::DidOpenTextDocument::METHOD => {
                if let Ok(p) = n.extract::<lsp::DidOpenTextDocumentParams>(
                    lsp::notification::DidOpenTextDocument::METHOD,
                ) {
                    self.document_changed(&p.text_document.uri, p.text_document.text);
                }
            }
            lsp::notification::DidChangeTextDocument::METHOD => {
                if let Ok(p) = n.extract::<lsp::DidChangeTextDocumentParams>(
                    lsp::notification::DidChangeTextDocument::METHOD,
                ) {
                    // Full sync: the last change carries the whole text.
                    if let Some(change) = p.content_changes.into_iter().last() {
                        self.document_changed(&p.text_document.uri, change.text);
                    }
                }
            }
            lsp::notification::DidSaveTextDocument::METHOD => {
                // The sources on disk moved: reload the ground and commit
                // the identities the build decided (ADR-0020 §4).
                if self.shared.text_root.is_some() {
                    self.reload_text_ground();
                }
            }
            lsp::notification::DidChangeWatchedFiles::METHOD => {
                if self.shared.text_root.is_some() {
                    self.reload_text_ground();
                }
            }
            lsp::notification::DidCloseTextDocument::METHOD => {
                if let Ok(p) = n.extract::<lsp::DidCloseTextDocumentParams>(
                    lsp::notification::DidCloseTextDocument::METHOD,
                ) {
                    let uri = document_uri(self.shared.text_root.as_deref(), &p.text_document.uri);
                    self.lock_host().close_text_document(&uri);
                    if !self.pull_diagnostics {
                        self.publish(&p.text_document.uri, Vec::new(), None);
                    }
                }
            }
            lsp::notification::Cancel::METHOD => {
                if let Ok(p) = n.extract::<lsp::CancelParams>(lsp::notification::Cancel::METHOD) {
                    let id = match p.id {
                        lsp::NumberOrString::Number(n) => RequestId::from(n),
                        lsp::NumberOrString::String(s) => RequestId::from(s),
                    };
                    let host_id = self.inflight.lock().ok().and_then(|m| m.get(&id).copied());
                    if let Some(host_id) = host_id {
                        self.lock_host().cancel_request(host_id);
                        debug!(?id, "cancelled");
                    }
                }
            }
            _ => {}
        }
    }

    /// Re-read a text project's sources and identity table from disk and
    /// write the reconciled table back, so every tool that opens the
    /// project next agrees on the identities the editor session decided.
    fn reload_text_ground(&self) {
        let Some(root) = self.shared.text_root.as_deref() else {
            return;
        };
        match text_ground(root) {
            Ok((_, files, table)) => {
                let mut host = self.lock_host();
                host.set_text_ground(files, table);
                let snapshot = host.snapshot();
                if let Some(world) = snapshot.text() {
                    let path = root.join(bdl_text::IDENTITIES_FILE);
                    match serde_json::to_string_pretty(&world.table) {
                        Ok(text) => {
                            if let Err(e) = bdl_model::persist::write_atomic(&path, text.as_bytes())
                            {
                                warn!(error = %e, "could not write the identity table");
                            }
                        }
                        Err(e) => warn!(error = %e, "could not render the identity table"),
                    }
                }
            }
            Err(e) => warn!(error = %e, "could not reload the text project"),
        }
    }

    fn document_changed(&self, uri: &Uri, text: String) {
        let doc = document_uri(self.shared.text_root.as_deref(), uri);
        let (id, _) = self.lock_host().set_text_document(&doc, text);
        if !self.pull_diagnostics {
            // Push fallback, isolated here: compute and publish right away.
            let snapshot = self.lock_host().snapshot();
            let index = snapshot
                .document(id)
                .map(|d| LineIndex::new(&d.source, self.shared.encoding));
            if let Some(index) = index {
                let resolve = |d: DocumentId| self.resolve_document(&snapshot, d);
                let items: Vec<lsp::Diagnostic> =
                    diagnostics(&snapshot, DiagnosticScope::Document(id))
                        .items
                        .iter()
                        .filter_map(|d| project_to_document(&snapshot, d, id))
                        .map(|d| convert::diagnostic(&d, &index, &resolve))
                        .collect();
                self.publish(uri, items, None);
            }
        }
    }

    fn publish(&self, uri: &Uri, diagnostics: Vec<lsp::Diagnostic>, version: Option<i32>) {
        let params = lsp::PublishDiagnosticsParams {
            uri: uri.clone(),
            diagnostics,
            version,
        };
        let _ = self
            .connection
            .sender
            .send(Message::Notification(Notification::new(
                lsp::notification::PublishDiagnostics::METHOD.into(),
                params,
            )));
    }

    fn lock_host(&self) -> std::sync::MutexGuard<'_, IdeHost> {
        match self.shared.host.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn resolve_document(
        &self,
        snapshot: &bdl_ide::AnalysisSnapshot,
        id: DocumentId,
    ) -> Option<(Uri, LineIndex)> {
        resolve_document(
            snapshot,
            self.shared.encoding,
            self.shared.text_root.as_deref(),
            id,
        )
    }

    // ---- requests: one worker per request over one snapshot ------------------------

    fn dispatch_request(&self, req: Request) {
        let id = req.id.clone();
        let method = req.method.clone();
        let scope = request_scope(&req, &self.lock_host(), self.shared.text_root.as_deref());
        let (host_req, token) = self.lock_host().begin_request(scope);
        if let Ok(mut m) = self.inflight.lock() {
            m.insert(id.clone(), host_req);
        }
        let shared = self.shared.clone();
        let sender = self.connection.sender.clone();
        let inflight = self.inflight.clone();
        std::thread::spawn(move || {
            let response = {
                let snapshot = {
                    let mut host = match shared.host.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    host.snapshot_cancellable(&token)
                };
                match snapshot {
                    Err(_) => {
                        Response::new_err(id.clone(), REQUEST_CANCELLED, "request cancelled".into())
                    }
                    Ok(snapshot) => {
                        let ctx = Ctx {
                            snapshot: &snapshot,
                            encoding: shared.encoding,
                            text_root: shared.text_root.as_deref(),
                        };
                        let r = handle_request(&ctx, req);
                        if token.is_cancelled() {
                            Response::new_err(
                                id.clone(),
                                REQUEST_CANCELLED,
                                "request cancelled".into(),
                            )
                        } else {
                            match r {
                                Ok(value) => Response::new_ok(id.clone(), value),
                                Err(e) => Response::new_err(id.clone(), e.code, e.message),
                            }
                        }
                    }
                }
            };
            if let Ok(mut host) = shared.host.lock() {
                host.end_request(host_req);
            }
            if let Ok(mut m) = inflight.lock() {
                m.remove(&id);
            }
            debug!(%method, "answered");
            let _ = sender.send(Message::Response(response));
        });
    }
}

/// What a request depends on, for cancellation: a request on a document
/// is obsolete when that document's overlay changes.
fn request_scope(req: &Request, host: &IdeHost, text_root: Option<&Path>) -> CancelScope {
    let uri = req
        .params
        .get("textDocument")
        .and_then(|t| t.get("uri"))
        .and_then(|u| u.as_str())
        .and_then(|u| Uri::from_str(u).ok())
        .map(|u| document_uri(text_root, &u));
    match uri.and_then(|u| host.known_document(&u)) {
        Some(document) => CancelScope::Overlay(OverlayKey::TextDocument { document }),
        None => CancelScope::Project,
    }
}

fn resolve_document(
    snapshot: &bdl_ide::AnalysisSnapshot,
    encoding: PositionEncoding,
    text_root: Option<&Path>,
    id: DocumentId,
) -> Option<(Uri, LineIndex)> {
    let uri = snapshot.document_uri(id)?;
    let uri = match (bdl_ide_db::workspace::file_path(uri), text_root) {
        (Some(rel), Some(root)) => path_to_uri(&root.join(rel))?,
        _ => Uri::from_str(uri.as_str()).ok()?,
    };
    let doc = snapshot.document(id)?;
    Some((uri, LineIndex::new(&doc.source, encoding)))
}

struct Ctx<'a> {
    snapshot: &'a bdl_ide::AnalysisSnapshot,
    encoding: PositionEncoding,
    text_root: Option<&'a Path>,
}

struct RequestError {
    code: i32,
    message: String,
}

impl RequestError {
    fn invalid(message: impl Into<String>) -> RequestError {
        RequestError {
            code: ErrorCode::InvalidParams as i32,
            message: message.into(),
        }
    }
    fn method_not_found(method: &str) -> RequestError {
        RequestError {
            code: ErrorCode::MethodNotFound as i32,
            message: format!("unknown method {method}"),
        }
    }
}

impl Ctx<'_> {
    /// The document and its index; `None` when the document is not open.
    fn document(&self, uri: &Uri) -> Option<(DocumentId, LineIndex)> {
        let id = self
            .snapshot
            .document_by_uri(&document_uri(self.text_root, uri))?;
        let doc = self.snapshot.document(id)?;
        Some((id, LineIndex::new(&doc.source, self.encoding)))
    }

    fn resolve(&self, id: DocumentId) -> Option<(Uri, LineIndex)> {
        resolve_document(self.snapshot, self.encoding, self.text_root, id)
    }

    /// Text anchors as LSP locations, in the documents they are in.
    fn locations(&self, anchors: Vec<bdl_ide::ProjectionAnchor>) -> Vec<lsp::Location> {
        anchors
            .into_iter()
            .filter_map(|a| {
                let (uri, index) = self.resolve(a.document()?)?;
                Some(lsp::Location {
                    uri,
                    range: index.range(a.text_range()?),
                })
            })
            .collect()
    }

    fn entity_at(
        &self,
        uri: &Uri,
        pos: lsp::Position,
    ) -> Option<(DocumentId, LineIndex, bdl_ide::EntityRef, EntityRole)> {
        let (doc, index) = self.document(uri)?;
        let offset = index.offset(pos);
        let (e, role) = entity_at(self.snapshot, doc, offset)?;
        Some((doc, index, e, role))
    }
}

fn params<P: serde::de::DeserializeOwned>(req: &Request) -> Result<P, RequestError> {
    serde_json::from_value(req.params.clone()).map_err(|e| RequestError::invalid(e.to_string()))
}

fn ok<T: Serialize>(value: T) -> Result<serde_json::Value, RequestError> {
    serde_json::to_value(value).map_err(|e| RequestError::invalid(e.to_string()))
}

fn handle_request(ctx: &Ctx<'_>, req: Request) -> Result<serde_json::Value, RequestError> {
    use lsp::request as r;
    match req.method.as_str() {
        r::HoverRequest::METHOD => {
            let p: lsp::HoverParams = params(&req)?;
            let pos = p.text_document_position_params;
            let h = ctx
                .document(&pos.text_document.uri)
                .and_then(|(doc, index)| {
                    let offset = index.offset(pos.position);
                    hover_at(ctx.snapshot, doc, offset).map(|h| convert::hover_at(&h, &index))
                });
            ok(h)
        }
        r::GotoDefinition::METHOD => {
            let p: lsp::GotoDefinitionParams = params(&req)?;
            let pos = p.text_document_position_params;
            let locations: Vec<lsp::Location> = ctx
                .document(&pos.text_document.uri)
                .map(|(doc, index)| {
                    let offset = index.offset(pos.position);
                    ctx.locations(definition_at(ctx.snapshot, doc, offset))
                })
                .unwrap_or_default();
            ok(if locations.is_empty() {
                None
            } else {
                Some(GotoDefinitionResponse::Array(locations))
            })
        }
        r::References::METHOD => {
            let p: lsp::ReferenceParams = params(&req)?;
            let pos = p.text_document_position;
            let include_decl = p.context.include_declaration;
            let locations: Vec<lsp::Location> = ctx
                .document(&pos.text_document.uri)
                .map(|(doc, index)| {
                    let offset = index.offset(pos.position);
                    ctx.locations(references_at(ctx.snapshot, doc, offset, include_decl))
                })
                .unwrap_or_default();
            ok(locations)
        }
        r::PrepareRenameRequest::METHOD => {
            let p: lsp::TextDocumentPositionParams = params(&req)?;
            let (doc, index) = match ctx.document(&p.text_document.uri) {
                Some(d) => d,
                None => return ok(None::<lsp::PrepareRenameResponse>),
            };
            let offset = index.offset(p.position);
            let anchor = ctx
                .snapshot
                .projections()
                .anchor_at(doc, offset)
                .filter(|a| matches!(a.role, EntityRole::Name | EntityRole::Reference));
            ok(anchor.and_then(|a| {
                Some(lsp::PrepareRenameResponse::RangeWithPlaceholder {
                    range: index.range(a.text_range()?),
                    placeholder: ctx.snapshot.name_of(a.entity)?.to_owned(),
                })
            }))
        }
        r::Rename::METHOD => {
            let p: lsp::RenameParams = params(&req)?;
            let pos = p.text_document_position;
            let Some((_, _, e, _)) = ctx.entity_at(&pos.text_document.uri, pos.position) else {
                return ok(None::<lsp::WorkspaceEdit>);
            };
            let plan = plan_rename(ctx.snapshot, e, &p.new_name)
                .map_err(|e| RequestError::invalid(e.to_string()))?;
            let (edit, _) = convert::workspace_edit(&plan, &|d| ctx.resolve(d));
            ok(Some(edit))
        }
        r::Completion::METHOD => {
            let p: lsp::CompletionParams = params(&req)?;
            let pos = p.text_document_position;
            let Some((doc, index)) = ctx.document(&pos.text_document.uri) else {
                return ok(None::<lsp::CompletionResponse>);
            };
            let offset = index.offset(pos.position);
            let items: Vec<lsp::CompletionItem> = completion(
                ctx.snapshot,
                &CompletionContext::Document {
                    document: doc,
                    offset,
                },
            )
            .iter()
            .map(|c| convert::completion_item(c, &index))
            .collect();
            ok(Some(lsp::CompletionResponse::Array(items)))
        }
        r::DocumentDiagnosticRequest::METHOD => {
            let p: lsp::DocumentDiagnosticParams = params(&req)?;
            let items = match ctx.document(&p.text_document.uri) {
                Some((doc, index)) => diagnostics(ctx.snapshot, DiagnosticScope::Document(doc))
                    .items
                    .iter()
                    .filter_map(|d| project_to_document(ctx.snapshot, d, doc))
                    .map(|d| convert::diagnostic(&d, &index, &|id| ctx.resolve(id)))
                    .collect(),
                None => Vec::new(),
            };
            let report = lsp::DocumentDiagnosticReportResult::Report(
                lsp::DocumentDiagnosticReport::Full(lsp::RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: lsp::FullDocumentDiagnosticReport {
                        result_id: Some(ctx.snapshot.stamp().to_string()),
                        items,
                    },
                }),
            );
            ok(report)
        }
        r::DocumentSymbolRequest::METHOD => {
            let p: lsp::DocumentSymbolParams = params(&req)?;
            let symbols: Vec<lsp::DocumentSymbol> = match ctx.document(&p.text_document.uri) {
                Some((doc, index)) => document_symbols(ctx.snapshot, doc)
                    .iter()
                    .filter_map(|s| convert::document_symbol(s, doc, &index))
                    .collect(),
                None => Vec::new(),
            };
            ok(Some(lsp::DocumentSymbolResponse::Nested(symbols)))
        }
        r::SemanticTokensFullRequest::METHOD => {
            let p: lsp::SemanticTokensParams = params(&req)?;
            let Some((doc, index)) = ctx.document(&p.text_document.uri) else {
                return ok(None::<lsp::SemanticTokensResult>);
            };
            let text = ctx
                .snapshot
                .document(doc)
                .map(|d| d.source.clone())
                .unwrap_or_default();
            let data = convert::semantic_tokens(
                &semantic_tokens(ctx.snapshot, doc),
                index.encoding(),
                &text,
            );
            ok(Some(lsp::SemanticTokensResult::Tokens(
                lsp::SemanticTokens {
                    result_id: Some(ctx.snapshot.stamp().to_string()),
                    data,
                },
            )))
        }
        r::CodeActionRequest::METHOD => {
            let p: lsp::CodeActionParams = params(&req)?;
            let Some((doc, index)) = ctx.document(&p.text_document.uri) else {
                return ok(None::<lsp::CodeActionResponse>);
            };
            let range = index.text_range(p.range);
            let resolve = |id| ctx.resolve(id);
            let mut out: Vec<lsp::CodeActionOrCommand> = Vec::new();
            let set = diagnostics(ctx.snapshot, DiagnosticScope::Document(doc));
            for d in &set.items {
                let Some(placed) = project_to_document(ctx.snapshot, d, doc) else {
                    continue;
                };
                if placed.range.end < range.start || range.end < placed.range.start {
                    continue;
                }
                let lsp_diag = convert::diagnostic(&placed, &index, &resolve);
                for a in actions_for(ctx.snapshot, d) {
                    out.push(lsp::CodeActionOrCommand::CodeAction(convert::code_action(
                        &a,
                        vec![lsp_diag.clone()],
                        &resolve,
                    )));
                }
            }
            if let Some((e, _)) = entity_at(ctx.snapshot, doc, range.start) {
                for a in actions_at(ctx.snapshot, e) {
                    out.push(lsp::CodeActionOrCommand::CodeAction(convert::code_action(
                        &a,
                        Vec::new(),
                        &resolve,
                    )));
                }
            }
            ok(Some(out))
        }
        r::Formatting::METHOD => {
            let p: lsp::DocumentFormattingParams = params(&req)?;
            let Some((doc, index)) = ctx.document(&p.text_document.uri) else {
                return ok(None::<Vec<lsp::TextEdit>>);
            };
            let edits: Vec<lsp::TextEdit> = format_document(ctx.snapshot, doc)
                .into_iter()
                .map(|e| lsp::TextEdit {
                    range: index.range(e.range),
                    new_text: e.new_text,
                })
                .collect();
            ok(Some(edits))
        }
        r::InlayHintRequest::METHOD => {
            let p: lsp::InlayHintParams = params(&req)?;
            let Some((doc, index)) = ctx.document(&p.text_document.uri) else {
                return ok(None::<Vec<lsp::InlayHint>>);
            };
            let range = index.text_range(p.range);
            let hints: Vec<lsp::InlayHint> = inlay_hints(ctx.snapshot, doc, Some(range))
                .into_iter()
                .map(|h| lsp::InlayHint {
                    position: index.position(h.offset),
                    label: lsp::InlayHintLabel::String(h.label),
                    kind: Some(match h.kind {
                        bdl_ide::InlayKind::Type => lsp::InlayHintKind::TYPE,
                        bdl_ide::InlayKind::Transport => lsp::InlayHintKind::PARAMETER,
                    }),
                    text_edits: None,
                    tooltip: None,
                    padding_left: Some(h.kind == bdl_ide::InlayKind::Transport),
                    padding_right: None,
                    data: None,
                })
                .collect();
            ok(Some(hints))
        }
        VirtualDocumentRequest::METHOD => {
            let p: VirtualDocumentParams = params(&req)?;
            let entity = match (&p.text_document, p.position) {
                (Some(td), Some(pos)) => ctx.entity_at(&td.uri, pos).map(|(_, _, e, _)| e),
                _ => None,
            };
            let doc = virtual_document(ctx.snapshot, p.kind, entity);
            ok(VirtualDocumentResult {
                uri: doc.uri.as_str().to_owned(),
                language: doc.language,
                text: doc.text,
            })
        }
        ExplainEntity::METHOD => {
            let p: lsp::TextDocumentPositionParams = params(&req)?;
            let result = ctx
                .entity_at(&p.text_document.uri, p.position)
                .and_then(|(_, _, e, _)| explain(ctx.snapshot, e))
                .map(|explanation| ExplainResult {
                    markdown: explanation.markdown(),
                    explanation,
                });
            ok(result)
        }
        InvalidationPreviewRequest::METHOD => {
            let p: InvalidationPreviewParams = params(&req)?;
            ok(preview_change(ctx.snapshot, &p.edit))
        }
        PreviewEdit::METHOD => {
            let p: lsp::RenameParams = params(&req)?;
            let pos = p.text_document_position;
            let plan = ctx
                .entity_at(&pos.text_document.uri, pos.position)
                .map(|(_, _, e, _)| plan_rename(ctx.snapshot, e, &p.new_name))
                .transpose()
                .map_err(|e| RequestError::invalid(e.to_string()))?;
            ok(plan)
        }
        other => Err(RequestError::method_not_found(other)),
    }
}
