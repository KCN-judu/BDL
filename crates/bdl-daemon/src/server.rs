//! The imperative shell around [`Session`]: frames in, frames out.
//!
//! One reader task decodes `ClientMessage`s from stdin; one writer task
//! encodes `ServerMessage`s to stdout; the coordinator in between owns the
//! session and handles requests strictly in order.  Nothing else ever
//! touches the session.

use crate::firmware::protocol::Firmware;
use crate::formula;
use crate::session::{Committed, Session, SessionError, SimulationRun};
use crate::COMPILER_VERSION;
use bdl_protocol::convert::{self, SessionInfo};
use bdl_protocol::framing;
use bdl_protocol::pb::{self, client_message::Payload as Req, response::Payload as Resp};
use bytes::BytesMut;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

const DAEMON_NAME: &str = "bdld";

pub async fn serve_stdio() -> anyhow::Result<()> {
    info!(compiler = COMPILER_VERSION, "bdld serving on stdio");
    let (in_tx, in_rx) = mpsc::channel::<pb::ClientMessage>(64);
    let (out_tx, out_rx) = mpsc::channel::<pb::ServerMessage>(64);

    let reader = tokio::spawn(read_loop(tokio::io::stdin(), in_tx));
    let writer = tokio::spawn(write_loop(tokio::io::stdout(), out_rx));

    let outcome = coordinate(in_rx, out_tx).await;

    // Let the writer drain, then stop the reader.
    let _ = writer.await;
    reader.abort();
    outcome
}

async fn read_loop<R: tokio::io::AsyncRead + Unpin>(
    mut input: R,
    tx: mpsc::Sender<pb::ClientMessage>,
) {
    let mut buf = BytesMut::with_capacity(8 * 1024);
    let mut chunk = [0u8; 8 * 1024];
    loop {
        match input.read(&mut chunk).await {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) => {
                warn!(error = %e, "stdin read failed");
                break;
            }
        }
        loop {
            match framing::decode::<pb::ClientMessage>(&mut buf) {
                Ok(Some(msg)) => {
                    if tx.send(msg).await.is_err() {
                        return;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    warn!(error = %e, "malformed frame; closing input");
                    return;
                }
            }
        }
    }
}

async fn write_loop<W: tokio::io::AsyncWrite + Unpin>(
    mut output: W,
    mut rx: mpsc::Receiver<pb::ServerMessage>,
) {
    let mut buf = BytesMut::with_capacity(8 * 1024);
    while let Some(msg) = rx.recv().await {
        buf.clear();
        if let Err(e) = framing::encode(&msg, &mut buf) {
            warn!(error = %e, "could not encode frame");
            continue;
        }
        if output.write_all(&buf).await.is_err() || output.flush().await.is_err() {
            warn!("stdout write failed");
            return;
        }
    }
}

/// The single coordinator: serial request handling over one session.
async fn coordinate(
    mut rx: mpsc::Receiver<pb::ClientMessage>,
    tx: mpsc::Sender<pb::ServerMessage>,
) -> anyhow::Result<()> {
    let mut session = Session::new(COMPILER_VERSION);
    let mut firmware = Firmware::default();
    let mut subscribed = false;

    while let Some(msg) = rx.recv().await {
        let request_id = msg.request_id;
        let Some(payload) = msg.payload else {
            send_response(
                &tx,
                request_id,
                Resp::Error(error("protocol.empty_request", "empty request")),
            )
            .await;
            continue;
        };
        debug!(request_id, kind = payload_name(&payload), "request");

        if matches!(payload, Req::Shutdown(_)) {
            send_response(&tx, request_id, Resp::Ack(pb::Ack {})).await;
            break;
        }
        if matches!(payload, Req::SubscribeProject(_)) {
            subscribed = true;
        }

        // A build or a flash runs on its own thread and reports as
        // events; it needs the sender, which the pure dispatch does not.
        let (resp, committed) = match payload {
            Req::BuildFirmware(r) => (firmware.start_build(&session, &r, &tx), None),
            Req::FlashFirmware(r) => (firmware.start_flash(&session, &r, &tx), None),
            Req::GetBuildStatus(r) => (
                match firmware.status(&session, &r.target_id) {
                    Ok(status) => Resp::BuildStatus(pb::BuildStatusResponse {
                        status: Some(status),
                    }),
                    Err(e) => Resp::Error(e),
                },
                None,
            ),
            Req::CancelBuild(_) => (firmware.cancel(), None),
            Req::ListFlashDevices(r) => (firmware.devices(&session, &r.target_id), None),
            other => handle(&mut session, other),
        };
        send_response(&tx, request_id, resp).await;
        if subscribed {
            if let Some(c) = committed {
                let event = pb::ServerMessage {
                    payload: Some(pb::server_message::Payload::Event(pb::Event {
                        payload: Some(pb::event::Payload::ProjectChanged(pb::ProjectChanged {
                            project: Some(projection_of(&session, &c.snapshot)),
                            outcome: c.outcome.as_ref().map(convert::outcome_to_pb),
                        })),
                    })),
                };
                let _ = tx.send(event).await;
                // The analysis for the very revision just committed, from
                // the IDE host so `RunAnalysis` shares it.  Cheap for now;
                // moves to a worker task when it is not.
                let analysis = match session.ide() {
                    Ok(host) => host.committed_analysis(),
                    Err(_) => std::sync::Arc::new(bdl_compiler::analyze(&c.snapshot)),
                };
                let ready = pb::ServerMessage {
                    payload: Some(pb::server_message::Payload::Event(pb::Event {
                        payload: Some(pb::event::Payload::AnalysisReady(pb::AnalysisReady {
                            analysis: Some(convert::analysis_to_pb(&analysis)),
                        })),
                    })),
                };
                let _ = tx.send(ready).await;
            }
        }
    }
    info!("bdld stopping");
    Ok(())
}

/// Pure dispatch: request in, response (and what was committed) out.
fn handle(session: &mut Session, req: Req) -> (Resp, Option<Committed>) {
    match req {
        Req::Handshake(h) => {
            let client = h.client_protocol_version.unwrap_or_default();
            let compatible = bdl_protocol::compatible(&client, &bdl_protocol::PROTOCOL_VERSION);
            info!(client = %h.client_name, client_version = %h.client_version, compatible, "handshake");
            (
                Resp::Handshake(pb::HandshakeResponse {
                    protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
                    compiler_version: COMPILER_VERSION.to_owned(),
                    daemon_name: DAEMON_NAME.to_owned(),
                    compatible,
                }),
                None,
            )
        }
        Req::OpenProject(o) => match session.open(Path::new(&o.root_path)) {
            Ok(_) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::InitProject(i) => {
            let template = match i.template.as_deref() {
                None => None,
                Some(id) => match crate::templates::template(id) {
                    Some(t) => Some(t),
                    None => {
                        return (
                            Resp::Error(error(
                                "project.unknown_template",
                                &format!("No template named {id}."),
                            )),
                            None,
                        )
                    }
                },
            };
            match session.init_with_source(
                Path::new(&i.root_path),
                &i.name,
                template.as_ref().map(|t| t.source.as_str()),
            ) {
                Ok(_) => (project_response(session), None),
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::ListTemplates(_) => (crate::firmware::protocol::templates_response(), None),
        // Handled by the coordinator (they need the event sender); never
        // reach the pure dispatch.
        Req::BuildFirmware(_)
        | Req::FlashFirmware(_)
        | Req::GetBuildStatus(_)
        | Req::CancelBuild(_)
        | Req::ListFlashDevices(_) => (
            Resp::Error(error(
                "protocol.internal",
                "firmware requests are coordinated",
            )),
            None,
        ),
        Req::InitSystemProject(i) => match session.init_system(Path::new(&i.root_path), &i.name) {
            Ok(_) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::InitTextProject(i) => match session.init_text(Path::new(&i.root_path), &i.name) {
            Ok(_) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::ReloadProject(_) => match session.reload_text() {
            Ok(_) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::GetSources(_) => match session.sources() {
            Ok(sources) => (
                Resp::Sources(pb::SourcesResponse {
                    sources: Some(sources_to_pb(&sources)),
                }),
                None,
            ),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::ApplySourceEdit(a) => {
            let base = bdl_model::Revision::from_raw(a.base_revision);
            match session.apply_source_edit(base, &a.path, &a.text) {
                Ok(edit) => {
                    let sources = session.sources().map(|s| sources_to_pb(&s)).ok();
                    let resp = Resp::SourceEditApplied(pb::SourceEditApplied {
                        accepted: edit.accepted,
                        project: Some(project_of(session)),
                        sources,
                    });
                    // An accepted text edit is a commit like any other:
                    // subscribers see the design move.
                    let committed = edit.accepted.then_some(Committed {
                        snapshot: edit.snapshot,
                        outcome: None,
                    });
                    (resp, committed)
                }
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::GetSystem(_) => match system_view(session) {
            Ok(v) => (Resp::System(pb::SystemResponse { system: Some(v) }), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::ApplySystemEdit(a) => {
            let Some(op) = a.op.as_ref() else {
                return (
                    Resp::Error(error(
                        "protocol.missing_field",
                        "apply_system_edit.op is required",
                    )),
                    None,
                );
            };
            let op = match convert::system::system_edit_op_from_pb(op) {
                Ok(op) => op,
                Err(e) => {
                    return (
                        Resp::Error(error("protocol.invalid_edit", &e.to_string())),
                        None,
                    )
                }
            };
            match session.apply_system(bdl_model::Revision::from_raw(a.base_revision), &op) {
                Ok(c) => {
                    let view = system_view(session).ok();
                    let resp = Resp::SystemEditApplied(pb::SystemEditApplied {
                        system: view,
                        project: Some(project_of(session)),
                        outcome: Some(convert::system::system_outcome_to_pb(&c.outcome)),
                    });
                    // Subscribers see the derived flat design move as after
                    // any commit; the system-level outcome is in the response.
                    (
                        resp,
                        Some(Committed {
                            snapshot: c.snapshot,
                            outcome: None,
                        }),
                    )
                }
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::RunSystemAnalysis(_) => match session.system_analysis() {
            Ok(a) => (
                Resp::SystemAnalysis(pb::SystemAnalysisResponse {
                    analysis: Some(convert::system::system_analysis_to_pb(&a)),
                }),
                None,
            ),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        // Group edits never commit: no ProjectChanged, no AnalysisReady —
        // the flat design is the same value (Phase 8b, Theorem A).
        Req::ApplyGroupEdit(a) => {
            let Some(op) = a.op.as_ref() else {
                return (
                    Resp::Error(error(
                        "protocol.missing_field",
                        "apply_group_edit.op is required",
                    )),
                    None,
                );
            };
            let op = match convert::system::group_edit_op_from_pb(op) {
                Ok(op) => op,
                Err(e) => {
                    return (
                        Resp::Error(error("protocol.invalid_edit", &e.to_string())),
                        None,
                    )
                }
            };
            match session.apply_group(a.base_generation, &op) {
                Ok(()) => match system_view(session) {
                    Ok(v) => (Resp::System(pb::SystemResponse { system: Some(v) }), None),
                    Err(e) => (Resp::Error(session_error(&e)), None),
                },
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::PreviewComponentExtraction(r) => {
            let choices = convert::system::choices_from_pb(r.choices.as_ref());
            match session
                .preview_extraction(bdl_system::BehaviorGroupId::from_raw(r.group), &choices)
            {
                Ok(p) => (
                    Resp::ExtractionPreview(pb::ExtractionPreviewResponse {
                        preview: Some(convert::system::preview_to_pb(&p)),
                    }),
                    None,
                ),
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::SaveProject(r) => match session.save_with(r.force) {
            Ok(()) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::CloseProject(_) => match session.close() {
            Ok(()) => (Resp::Ack(pb::Ack {}), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::GetProject(_) => match session.project() {
            Ok(_) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::SubscribeProject(_) => (Resp::Ack(pb::Ack {}), None),
        Req::ApplyEdit(a) => {
            let Some(op) = a.op.as_ref() else {
                return (
                    Resp::Error(error("protocol.missing_field", "apply_edit.op is required")),
                    None,
                );
            };
            let op = match convert::edit_op_from_pb(op) {
                Ok(op) => op,
                Err(e) => {
                    return (
                        Resp::Error(error("protocol.invalid_edit", &e.to_string())),
                        None,
                    )
                }
            };
            match session.apply(bdl_model::Revision::from_raw(a.base_revision), &op) {
                Ok(c) => {
                    let resp = Resp::EditApplied(pb::EditApplied {
                        project: Some(project_of(session)),
                        outcome: c.outcome.as_ref().map(convert::outcome_to_pb),
                    });
                    (resp, Some(c))
                }
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::Undo(_) => step_result(session, true),
        Req::Redo(_) => step_result(session, false),
        Req::ArrangeLayout(_) => match session.arranged_layout() {
            Ok(layout) => (
                Resp::Layout(pb::LayoutResponse {
                    layout: Some(convert::layout_to_pb(&layout)),
                }),
                None,
            ),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::SetLayout(l) => {
            let layout = l
                .layout
                .as_ref()
                .map(convert::layout_from_pb)
                .unwrap_or_default();
            match session.set_layout(layout) {
                Ok(()) => (Resp::Ack(pb::Ack {}), None),
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::RunAnalysis(_) => match session.ide() {
            Ok(host) => {
                // The committed snapshot's analysis, cached by the IDE host
                // per revision and tagged with it; the client discards
                // stale ones.
                let analysis = host.committed_analysis();
                (
                    Resp::Analysis(pb::AnalysisResponse {
                        analysis: Some(convert::analysis_to_pb(&analysis)),
                    }),
                    None,
                )
            }
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::StartSimulation(r) => (start_simulation(session, &r), None),
        Req::StepSimulation(r) => (step_simulation(session, r.ticks), None),
        Req::ResetSimulation(_) => (reset_simulation(session), None),
        Req::ListTargets(_) => (
            Resp::Targets(pb::TargetsResponse {
                targets: bdl_hardware::boards::registry()
                    .iter()
                    .map(|(id, hw)| convert::target_view(id, hw))
                    .collect(),
            }),
            None,
        ),
        Req::AnalyzeDeployment(r) => (analyze_deployment(session, &r), None),
        Req::AnalyzeDefinitionDraft(r) => (analyze_definition_draft(session, &r), None),
        Req::DiscardDefinitionDraft(r) => {
            match session.discard_draft(
                component_scope(r.component),
                bdl_model::DeclId::from_raw(r.mapping_id),
            ) {
                Ok(_) => (Resp::Ack(pb::Ack {}), None),
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::CompleteDefinitionDraft(r) => (complete_definition_draft(session, &r), None),
        Req::HoverDefinitionDraft(r) => (hover_definition_draft(session, &r), None),
        Req::SemanticTokens(r) => (semantic_tokens(session, &r), None),
        Req::SourceCompletion(r) => (source_completion(session, &r), None),
        Req::SourceHover(r) => (source_hover(session, &r), None),
        Req::SourceDefinition(r) => (source_definition(session, &r), None),
        Req::SourceReferences(r) => (source_references(session, &r), None),
        Req::FormatSource(r) => (format_source(session, &r), None),
        Req::HoverEntity(r) => (hover_entity(session, &r), None),
        Req::ListSemanticActions(r) => (list_semantic_actions(session, &r), None),
        Req::GetFormulaProjection(r) => (get_formula_projection(session, &r), None),
        Req::GetFormulaSlot(r) => (get_formula_slot(session, &r), None),
        Req::ComposeFormula(r) => (compose_formula(session, &r), None),
        Req::ListConceptTemplates(_) => (
            Resp::ConceptTemplates(convert::concept_templates_response(libraries())),
            None,
        ),
        Req::ListValueCategories(_) => (
            Resp::ValueCategories(convert::value_categories_response()),
            None,
        ),
        Req::NavigateFormula(r) => (navigate_formula(session, &r), None),
        Req::CompleteFormulaCaret(r) => (complete_formula_caret(session, &r), None),
        Req::GetFormulaSignature(r) => (get_formula_signature(session, &r), None),
        Req::GetFormulaRender(r) => (get_formula_render(session, &r), None),
        Req::InstantiateConceptTemplate(r) => instantiate_concept_template(session, &r),
        Req::ListLibraryItems(_) => (
            Resp::LibraryItems(convert::library_items_response(libraries())),
            None,
        ),
        Req::InstantiateLibraryItem(r) => instantiate_library_item(session, &r),
        Req::CreateSource(r) => create_source(session, &r),
        Req::ListSourceCandidates(r) => (list_source_candidates(session, &r), None),
        Req::Shutdown(_) => (Resp::Ack(pb::Ack {}), None),
    }
}

/// A candidate definition checked against the current snapshot and never
/// committed: the project, its revision and its undo history are untouched.
///
/// The draft is an *overlay* on the session's IDE host: the request
/// updates it, an immutable snapshot of committed + overlays is taken, and
/// `bdl-ide` returns the stamped verdict — the same path a text editor's
/// unsaved buffer takes (`docs/architecture/ide-service.md`).
fn analyze_definition_draft(session: &mut Session, r: &pb::AnalyzeDefinitionDraftRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.draft_verdict(component_scope(r.component), id, &r.source) {
        Ok(v) => {
            // the Composer's view of the same world: a projection failure
            // (a definition by reference) leaves the field unset
            let projection = session
                .formula_projection(component_scope(r.component), id)
                .ok()
                .map(|p| formula::projection_to_pb(&p));
            Resp::DefinitionDraft(pb::DefinitionDraftAnalysis {
                revision: v.stamp.revision.raw(),
                mapping_id: r.mapping_id,
                generation: r.generation,
                parse_ok: v.parse_ok,
                analysis: Some(convert::mapping_analysis_to_pb(&v.analysis)),
                projection,
            })
        }
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn get_formula_projection(session: &mut Session, r: &pb::GetFormulaProjectionRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.formula_projection(component_scope(r.component), id) {
        Ok(p) => Resp::FormulaProjection(pb::FormulaProjectionResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            projection: Some(formula::projection_to_pb(&p)),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn get_formula_slot(session: &mut Session, r: &pb::GetFormulaSlotRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.formula_slot(component_scope(r.component), id, &r.source, &r.node_id) {
        Ok(slot) => Resp::FormulaSlot(formula::slot_to_pb(&slot, r.revision, r.mapping_id)),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn compose_formula(session: &mut Session, r: &pb::ComposeFormulaRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    let op = match r.action.as_ref().and_then(formula::action_from_pb) {
        Some(op) => op,
        None => {
            return Resp::Error(error(
                "protocol.missing_field",
                "a compose action with a node is required",
            ))
        }
    };
    match session.compose_formula(component_scope(r.component), id, &r.source, &op) {
        Ok(c) => Resp::ComposeFormula(pb::ComposeFormulaResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            source: c.source,
            edits: c
                .edits
                .iter()
                .map(|e| pb::DraftTextEdit {
                    start: e.range.start,
                    end: e.range.end,
                    new_text: e.new_text.clone(),
                })
                .collect(),
            select: c.select.unwrap_or_default(),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

/// The current revision, or the stale-revision refusal every draft query
/// shares: a draft is judged against the world the client believes it is
/// in, and the projection that follows a change re-asks.
fn draft_revision(session: &Session, requested: u64) -> Result<(), pb::Error> {
    let revision = match session.project() {
        Ok(p) => p.current.revision.raw(),
        Err(e) => return Err(session_error(&e)),
    };
    if revision != requested {
        return Err(error(
            "draft.stale_revision",
            &format!("draft targets revision {requested} but the project is at {revision}"),
        ));
    }
    Ok(())
}

fn complete_definition_draft(
    session: &mut Session,
    r: &pb::CompleteDefinitionDraftRequest,
) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.draft_completion(component_scope(r.component), id, &r.source, r.offset) {
        Ok(items) => Resp::DraftCompletion(pb::DraftCompletionResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            items: items.iter().map(completion_to_pb).collect(),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn completion_to_pb(c: &bdl_ide::SemanticCompletion) -> pb::DraftCompletionItem {
    pb::DraftCompletionItem {
        label: c.label.clone(),
        kind: format!("{:?}", c.kind).to_lowercase(),
        replace_start: c.replace.start,
        replace_end: c.replace.end,
        insert: c.insert.clone(),
        resulting_type: c.resulting_type.clone().unwrap_or_default(),
        documentation: c.documentation.clone().unwrap_or_default(),
        relevance: u32::from(c.relevance),
        structured_insert: c.structured_insert.clone().unwrap_or_default(),
    }
}

fn navigate_formula(session: &mut Session, r: &pb::NavigateFormulaRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    let (Some(side), Some(motion)) = (
        formula::side_from_pb(r.side()),
        formula::motion_from_pb(r.motion()),
    ) else {
        return Resp::Error(error(
            "protocol.missing_field",
            "a caret side and a motion are required",
        ));
    };
    match session.navigate_formula(
        component_scope(r.component),
        id,
        &r.source,
        &r.node_id,
        side,
        motion,
    ) {
        Ok(c) => Resp::NavigateFormula(pb::NavigateFormulaResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            node_id: c.node,
            side: formula::side_to_pb(c.side).into(),
            offset: c.offset,
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn complete_formula_caret(session: &mut Session, r: &pb::CompleteFormulaCaretRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    let Some(side) = formula::side_from_pb(r.side()) else {
        return Resp::Error(error("protocol.missing_field", "a caret side is required"));
    };
    match session.caret_completion(
        component_scope(r.component),
        id,
        &r.source,
        &r.node_id,
        side,
        &r.prefix,
    ) {
        Ok(items) => Resp::DraftCompletion(pb::DraftCompletionResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            items: items.iter().map(completion_to_pb).collect(),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn get_formula_signature(session: &mut Session, r: &pb::GetFormulaSignatureRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.formula_signature(component_scope(r.component), id, &r.source, &r.node_id) {
        Ok(s) => Resp::FormulaSignature(formula::signature_to_pb(
            s.as_ref(),
            r.revision,
            r.mapping_id,
        )),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn get_formula_render(session: &mut Session, r: &pb::GetFormulaRenderRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.formula_render(component_scope(r.component), id) {
        Ok(render) => Resp::FormulaRender(formula::render_to_pb(&render, r.revision, r.mapping_id)),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn hover_definition_draft(session: &mut Session, r: &pb::HoverDefinitionDraftRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.draft_hover(component_scope(r.component), id, &r.source, r.offset) {
        Ok(None) => Resp::DraftHover(pb::DraftHoverResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            found: false,
            ..Default::default()
        }),
        Ok(Some(h)) => Resp::DraftHover(pb::DraftHoverResponse {
            mapping_id: r.mapping_id,
            ..hover_at_to_pb(h, r.revision)
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

/// Tokens over a text as typed (protocol 0.21).  A stale `revision` is
/// not an error: tokens are presentation, and the daemon answers over
/// its current project and states the revision it classified at — the
/// client compares.  The legend travels with every answer.
fn semantic_tokens(session: &mut Session, r: &pb::SemanticTokensRequest) -> Resp {
    use pb::semantic_tokens_request::Document;
    let tokens = match r.document.as_ref() {
        Some(Document::Path(path)) => session.source_tokens(path, &r.text),
        Some(Document::Formula(f)) => session.draft_tokens(
            component_scope(f.component),
            bdl_model::DeclId::from_raw(f.mapping_id),
            &r.text,
        ),
        None => {
            return Resp::Error(error(
                "protocol.missing_field",
                "semantic tokens need a document: a source path or a formula",
            ))
        }
    };
    let revision = session
        .project()
        .map(|p| p.current.revision.raw())
        .unwrap_or_default();
    match tokens {
        Ok(tokens) => Resp::SemanticTokens(pb::SemanticTokensResponse {
            revision,
            generation: r.generation,
            legend: Some(legend_to_pb(&bdl_ide::legend())),
            text_len: r.text.len() as u32,
            tokens: tokens
                .iter()
                .map(|t| pb::SemanticToken {
                    start: t.range.start,
                    end: t.range.end,
                    token_type: t.ty.index(),
                    token_modifiers: t.modifiers.bits(),
                })
                .collect(),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn current_revision(session: &Session) -> u64 {
    session
        .project()
        .map(|p| p.current.revision.raw())
        .unwrap_or_default()
}

/// The Code view's IDE queries (protocol 0.22): each over the text as
/// typed, answered at the daemon's current revision with the client's
/// generation echoed; a stale `revision` is not an error.
fn source_completion(session: &mut Session, r: &pb::SourceCompletionRequest) -> Resp {
    match session.source_completion(&r.path, &r.text, r.offset) {
        Ok(items) => Resp::SourceCompletion(pb::SourceCompletionResponse {
            revision: current_revision(session),
            generation: r.generation,
            items: items.iter().map(completion_to_pb).collect(),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn source_hover(session: &mut Session, r: &pb::SourceHoverRequest) -> Resp {
    match session.source_hover(&r.path, &r.text, r.offset) {
        Ok(None) => Resp::DraftHover(pb::DraftHoverResponse {
            revision: current_revision(session),
            found: false,
            ..Default::default()
        }),
        Ok(Some(h)) => Resp::DraftHover(hover_at_to_pb(h, current_revision(session))),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn locations_to_pb(
    revision: u64,
    generation: u64,
    title: String,
    locations: Vec<crate::session::SourceLocation>,
) -> Resp {
    Resp::SourceLocations(pb::SourceLocationsResponse {
        revision,
        generation,
        title,
        locations: locations
            .into_iter()
            .map(|l| pb::SourceLocation {
                path: l.path,
                start: l.range.start,
                end: l.range.end,
            })
            .collect(),
    })
}

fn source_definition(session: &mut Session, r: &pb::SourceDefinitionRequest) -> Resp {
    match session.source_definition(&r.path, &r.text, r.offset) {
        Ok((title, locations)) => {
            locations_to_pb(current_revision(session), r.generation, title, locations)
        }
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn source_references(session: &mut Session, r: &pb::SourceReferencesRequest) -> Resp {
    match session.source_references(&r.path, &r.text, r.offset, r.include_declaration) {
        Ok((title, locations)) => {
            locations_to_pb(current_revision(session), r.generation, title, locations)
        }
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn format_source(session: &mut Session, r: &pb::FormatSourceRequest) -> Resp {
    match session.format_source(&r.path, &r.text) {
        Ok(formatted) => Resp::FormatSource(pb::FormatSourceResponse {
            revision: current_revision(session),
            generation: r.generation,
            formatted: formatted.is_some(),
            text: formatted.unwrap_or_else(|| r.text.clone()),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn legend_to_pb(l: &bdl_ide::Legend) -> pb::SemanticTokenLegend {
    pb::SemanticTokenLegend {
        version: l.version,
        types: l.types.clone(),
        modifiers: l.modifiers.clone(),
    }
}

fn entity_from_pb(e: Option<&pb::EntityRef>) -> Result<bdl_ide::EntityRef, pb::Error> {
    use pb::entity_ref::Kind;
    let kind = e
        .and_then(|e| e.kind.as_ref())
        .ok_or_else(|| error("protocol.missing_field", "entity is required"))?;
    Ok(match kind {
        Kind::Project(_) => bdl_ide::EntityRef::Project,
        Kind::ConceptId(id) => bdl_ide::EntityRef::Concept(bdl_model::SemanticId::from_raw(*id)),
        Kind::MappingId(id) => bdl_ide::EntityRef::Mapping(bdl_model::DeclId::from_raw(*id)),
        Kind::ClockId(id) => bdl_ide::EntityRef::Clock(bdl_model::ClockId::from_raw(*id)),
        Kind::OutputId(id) => bdl_ide::EntityRef::Output(bdl_model::OutputId::from_raw(*id)),
        Kind::DeviceId(id) => bdl_ide::EntityRef::Device(bdl_model::DeviceId::from_raw(*id)),
    })
}

pub(crate) fn entity_to_pb(e: bdl_ide::EntityRef) -> pb::EntityRef {
    use pb::entity_ref::Kind;
    pb::EntityRef {
        kind: Some(match e {
            bdl_ide::EntityRef::Project => Kind::Project(pb::Unit {}),
            bdl_ide::EntityRef::Concept(c) => Kind::ConceptId(c.raw()),
            bdl_ide::EntityRef::Mapping(m) => Kind::MappingId(m.raw()),
            bdl_ide::EntityRef::Clock(c) => Kind::ClockId(c.raw()),
            bdl_ide::EntityRef::Output(o) => Kind::OutputId(o.raw()),
            bdl_ide::EntityRef::Device(d) => Kind::DeviceId(d.raw()),
            bdl_ide::EntityRef::Requirement { device, .. } => Kind::DeviceId(device.raw()),
            // System entities are text-workspace entities the wire has no
            // reference for yet; Studio names them by its own views.
            bdl_ide::EntityRef::Component(_)
            | bdl_ide::EntityRef::Port { .. }
            | bdl_ide::EntityRef::Instance(_)
            | bdl_ide::EntityRef::Binding(_)
            | bdl_ide::EntityRef::Export(_) => Kind::Project(pb::Unit {}),
        }),
    }
}

/// A hover at a position: the entity's card, or an equation's words,
/// with the range of the name.
fn hover_at_to_pb(h: bdl_ide::HoverAt, revision: u64) -> pb::DraftHoverResponse {
    let span = Some(pb::SourceSpan {
        start: h.range.start,
        end: h.range.end,
    });
    match h.content {
        bdl_ide::HoverContent::Entity(e) => pb::DraftHoverResponse {
            span,
            ..hover_to_pb(e, revision)
        },
        bdl_ide::HoverContent::Equation {
            name,
            shape,
            documentation,
        } => pb::DraftHoverResponse {
            revision,
            found: true,
            span,
            title: shape,
            explanation: documentation,
            equation: name,
            ..Default::default()
        },
    }
}

fn hover_to_pb(h: bdl_ide::SemanticHover, revision: u64) -> pb::DraftHoverResponse {
    pb::DraftHoverResponse {
        revision,
        found: true,
        concept_id: match h.entity {
            bdl_ide::EntityRef::Concept(c) => Some(c.raw()),
            _ => None,
        },
        mapping_id: match h.entity {
            bdl_ide::EntityRef::Mapping(m) => m.raw(),
            _ => 0,
        },
        entity: Some(entity_to_pb(h.entity)),
        title: h.title,
        signature: h.signature.unwrap_or_default(),
        representation: h.representation.unwrap_or_default(),
        status: h.status.label().to_owned(),
        open: h.status.is_open(),
        details: h
            .details
            .iter()
            .map(|d| pb::HoverDetail {
                label: d.label.clone(),
                value: d.value.clone(),
            })
            .collect(),
        explanation: h.explanation.unwrap_or_default(),
        ..Default::default()
    }
}

/// The everyday card for a canvas node or a library row.
/// The libraries this daemon serves.  The Standard Library
/// is embedded; team/project/package libraries are a loader away
/// (`docs/spec/concept-library.md`).
fn libraries() -> &'static bdl_library::LibrarySet {
    bdl_library::LibrarySet::shared()
}

/// The legacy instantiation request (0.6–0.16): a concept template by id,
/// with an optional name — the Concept item of the same id, planned and
/// applied like any item.  `source_name` (0.16) is deprecated and ignored:
/// a Source is an item of its own category, instantiated with
/// `InstantiateLibraryItem`.
#[allow(deprecated)]
fn instantiate_concept_template(
    session: &mut Session,
    r: &pb::InstantiateConceptTemplateRequest,
) -> (Resp, Option<Committed>) {
    if libraries().get(&r.template_id).is_none() {
        return (
            Resp::Error(error(
                "library.unknown_template",
                &format!("no concept template `{}` is served", r.template_id),
            )),
            None,
        );
    }
    let mut names = std::collections::BTreeMap::new();
    if let Some(n) = r.name.as_deref().filter(|n| !n.trim().is_empty()) {
        names.insert("concept".to_string(), n.to_string());
    }
    instantiate_item(session, r.base_revision, &r.template_id, names, r.component)
}

/// One library item into the project, atomically: the fragment is planned
/// against the target design (the system's own, or a component body) and
/// every step applied in one transaction — `SystemEditApplied` with the
/// outcome naming the created concept and relationship, or an error and
/// nothing applied.  Nothing about the item is recorded; a Source is a
/// Source because of the objects' shape (ADR-0032).
fn instantiate_library_item(
    session: &mut Session,
    r: &pb::InstantiateLibraryItemRequest,
) -> (Resp, Option<Committed>) {
    if libraries().item(&r.item_id).is_none() {
        return (
            Resp::Error(error(
                "library.unknown_item",
                &format!("no library item `{}` is served", r.item_id),
            )),
            None,
        );
    }
    let names = r
        .names
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    instantiate_item(session, r.base_revision, &r.item_id, names, r.component)
}

fn instantiate_item(
    session: &mut Session,
    base_revision: u64,
    item_id: &str,
    names: std::collections::BTreeMap<String, String>,
    component: Option<u64>,
) -> (Resp, Option<Committed>) {
    let Some(item) = libraries().item(item_id) else {
        return (
            Resp::Error(error(
                "library.unknown_item",
                &format!("no library item `{item_id}` is served"),
            )),
            None,
        );
    };
    let revision = bdl_model::Revision::from_raw(base_revision);
    let p = match session.project() {
        Ok(p) => p,
        Err(e) => return (Resp::Error(session_error(&e)), None),
    };
    // A system project inserts into its own design or a component body;
    // the item's default names are made free against that design.
    let Some(sys) = p.system.as_ref() else {
        return (Resp::Error(session_error(&SessionError::NotASystem)), None);
    };
    let scope = component_scope(component);
    let design = match scope {
        None => &sys.current.system.base,
        Some(c) => match sys.current.system.components.get(&c) {
            Some(comp) => &comp.body,
            None => {
                return (
                    Resp::Error(session_error(&SessionError::UnknownComponent(c))),
                    None,
                )
            }
        },
    };
    let steps = match bdl_library::plan(design, item, &names) {
        Ok(steps) => steps,
        Err(e) => {
            return (
                Resp::Error(error("library.invalid_plan", &e.to_string())),
                None,
            )
        }
    };
    match session.apply_library_item(revision, scope, &steps) {
        Ok(c) => {
            let view = system_view(session).ok();
            let resp = Resp::SystemEditApplied(pb::SystemEditApplied {
                system: view,
                project: Some(project_of(session)),
                outcome: Some(convert::system::system_outcome_to_pb(&c.outcome)),
            });
            (
                resp,
                Some(Committed {
                    snapshot: c.snapshot,
                    outcome: None,
                }),
            )
        }
        Err(e) => (Resp::Error(session_error(&e)), None),
    }
}

/// The design a request scoped by `component` addresses: the system's own
/// base, or the component's body.
fn scoped_design(
    p: &crate::session::OpenProject,
    component: Option<u64>,
) -> Result<(&bdl_model::surface::Design, Option<bdl_system::ComponentId>), pb::Error> {
    let Some(sys) = p.system.as_ref() else {
        return Err(session_error(&SessionError::NotASystem));
    };
    let scope = component_scope(component);
    let design = match scope {
        None => &sys.current.system.base,
        Some(c) => match sys.current.system.components.get(&c) {
            Some(comp) => &comp.body,
            None => return Err(session_error(&SessionError::UnknownComponent(c))),
        },
    };
    Ok((design, scope))
}

/// A Source over a chosen concept: one `CreateMapping` for an existing
/// concept, `CreateConcept` then `CreateMapping` in one transaction for a
/// new one — the same planned steps a library item runs through, so the
/// keys resolve inside the transaction and nothing is applied when either
/// edit is refused.  Names are used as given: a taken or unspellable one
/// is the ordinary edit refusal.
fn create_source(session: &mut Session, r: &pb::CreateSourceRequest) -> (Resp, Option<Committed>) {
    use bdl_library::PlannedStep;
    let revision = bdl_model::Revision::from_raw(r.base_revision);
    let p = match session.project() {
        Ok(p) => p,
        Err(e) => return (Resp::Error(session_error(&e)), None),
    };
    let (design, scope) = match scoped_design(p, r.component) {
        Ok(x) => x,
        Err(e) => return (Resp::Error(e), None),
    };
    let source_name = r.source_name.trim().to_owned();
    let committed = match &r.concept {
        // an existing concept, by identity: one ordinary edit
        Some(pb::create_source_request::Concept::ExistingConcept(id)) => {
            let id = bdl_model::SemanticId::from_raw(*id);
            if !design.concepts.contains_key(&id) {
                return (
                    Resp::Error(error(
                        "edit.unknown_concept",
                        &format!("no concept {id} in this design"),
                    )),
                    None,
                );
            }
            let op = bdl_model::EditOp::CreateMapping {
                name: source_name,
                description: r.source_description.clone(),
                signature: bdl_model::surface::Signature {
                    inputs: Vec::new(),
                    output: id,
                },
                definition: None,
                clock: None,
            };
            let sop = match scope {
                None => bdl_system::SystemEditOp::Base { op },
                Some(c) => bdl_system::SystemEditOp::EditComponentBody { component: c, op },
            };
            session.apply_system(revision, &sop)
        }
        // a new concept and the Source over it: two planned steps in one
        // transaction, the key resolved inside it
        Some(pb::create_source_request::Concept::NewConcept(c)) => {
            let representation = match (c.representation.as_ref(), c.category_id.as_deref()) {
                (Some(rep), _) => match convert::representation_from_pb(rep) {
                    Ok(r) => Some(r),
                    Err(e) => {
                        return (
                            Resp::Error(error("edit.invalid_representation", &e.to_string())),
                            None,
                        )
                    }
                },
                (None, Some(id)) => match convert::representation_of_category(id) {
                    Some(r) => Some(r),
                    None => {
                        return (
                            Resp::Error(error(
                                "library.unknown_category",
                                &format!("no value category `{id}`"),
                            )),
                            None,
                        )
                    }
                },
                (None, None) => None,
            };
            let steps = [
                PlannedStep::Concept {
                    key: "value".into(),
                    op: bdl_model::EditOp::CreateConcept {
                        name: c.name.trim().to_owned(),
                        description: c.description.clone(),
                        representation,
                    },
                },
                PlannedStep::Mapping {
                    key: "source".into(),
                    name: source_name,
                    description: r.source_description.clone(),
                    inputs: Vec::new(),
                    output: "value".into(),
                },
            ];
            session.apply_library_item(revision, scope, &steps)
        }
        None => return (
            Resp::Error(error(
                "edit.invalid_request",
                "a Source is created over a concept: choose an existing one or describe a new one",
            )),
            None,
        ),
    };
    match committed {
        Ok(c) => {
            let view = system_view(session).ok();
            let resp = Resp::SystemEditApplied(pb::SystemEditApplied {
                system: view,
                project: Some(project_of(session)),
                outcome: Some(convert::system::system_outcome_to_pb(&c.outcome)),
            });
            (
                resp,
                Some(Committed {
                    snapshot: c.snapshot,
                    outcome: None,
                }),
            )
        }
        Err(e) => (Resp::Error(session_error(&e)), None),
    }
}

/// The concepts a Source may be created over, ranked for a preset, with
/// the preset's names made free in the design in scope.
fn list_source_candidates(session: &mut Session, r: &pb::ListSourceCandidatesRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let p = match session.project() {
        Ok(p) => p,
        Err(e) => return Resp::Error(session_error(&e)),
    };
    let (design, _) = match scoped_design(p, r.component) {
        Ok(x) => x,
        Err(e) => return Resp::Error(e),
    };
    let preset = if r.item_id.is_empty() {
        None
    } else {
        match libraries().item(&r.item_id).and_then(|i| i.preset()) {
            Some(preset) => Some(preset),
            None => {
                return Resp::Error(error(
                    "library.unknown_item",
                    &format!("no Source item `{}` is served", r.item_id),
                ))
            }
        }
    };
    let candidates = bdl_library::rank_concepts(
        design,
        preset.as_ref().and_then(|p| p.representation.as_ref()),
    );
    Resp::SourceCandidates(pb::SourceCandidatesResponse {
        revision: r.revision,
        candidates: candidates
            .iter()
            .map(|c| pb::SourceCandidateView {
                concept_id: c.concept.raw(),
                preferred: c.preferred,
            })
            .collect(),
        suggested_concept_name: preset
            .as_ref()
            .map(|p| bdl_library::free_name(design, &p.concept_name))
            .unwrap_or_default(),
        suggested_source_name: preset
            .as_ref()
            .map(|p| bdl_library::free_name(design, &p.source_name))
            .unwrap_or_default(),
        preset: preset.as_ref().map(convert::source_preset_view),
    })
}

fn hover_entity(session: &mut Session, r: &pb::HoverEntityRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let entity = match entity_from_pb(r.entity.as_ref()) {
        Ok(e) => e,
        Err(e) => return Resp::Error(e),
    };
    let snapshot = match session.ide_snapshot() {
        Ok(s) => s,
        Err(e) => return Resp::Error(session_error(&e)),
    };
    match bdl_ide::hover(&snapshot, entity) {
        Some(h) => Resp::DraftHover(hover_to_pb(h, r.revision)),
        None => Resp::DraftHover(pb::DraftHoverResponse {
            revision: r.revision,
            found: false,
            entity: Some(entity_to_pb(entity)),
            ..Default::default()
        }),
    }
}

/// The fixes for an entity's diagnostics plus its context actions, as
/// bdl-ide offers them.  Ready plans carry their model edits; the client
/// applies them as ordinary revisioned edits.
fn list_semantic_actions(session: &mut Session, r: &pb::ListSemanticActionsRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let entity = match entity_from_pb(r.entity.as_ref()) {
        Ok(e) => e,
        Err(e) => return Resp::Error(e),
    };
    let snapshot = match session.ide_snapshot() {
        Ok(s) => s,
        Err(e) => return Resp::Error(session_error(&e)),
    };
    let mut actions = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let scope = match entity {
        bdl_ide::EntityRef::Project => bdl_ide::DiagnosticScope::Project,
        e => bdl_ide::DiagnosticScope::Concerning(e),
    };
    for d in &bdl_ide::diagnostics(&snapshot, scope).items {
        for a in bdl_ide::actions_for(&snapshot, d) {
            if seen.insert(a.id.clone()) {
                actions.push(a);
            }
        }
    }
    for a in bdl_ide::actions_at(&snapshot, entity) {
        if seen.insert(a.id.clone()) {
            actions.push(a);
        }
    }
    Resp::SemanticActions(pb::SemanticActionsResponse {
        revision: r.revision,
        entity: Some(entity_to_pb(entity)),
        actions: actions.iter().map(action_to_pb).collect(),
    })
}

fn action_to_pb(a: &bdl_ide::SemanticAction) -> pb::SemanticActionView {
    use bdl_ide::{ActionKind, Applicability, SemanticOperation};
    let (applicability, reason, options) = match &a.applicability {
        Applicability::Ready => (pb::ActionApplicability::Ready, String::new(), Vec::new()),
        Applicability::NeedsChoice { options } => (
            pb::ActionApplicability::NeedsChoice,
            String::new(),
            options
                .iter()
                .map(|c| pb::ActionChoiceView {
                    label: c.label.clone(),
                    edit: Some(convert::edit_op_to_pb(&c.edit)),
                })
                .collect(),
        ),
        Applicability::Blocked { reason } => {
            (pb::ActionApplicability::Blocked, reason.clone(), Vec::new())
        }
    };
    let edits = a
        .plan
        .iter()
        .flat_map(|p| p.operations.iter())
        .filter_map(|op| match op {
            SemanticOperation::Model { edit } => Some(convert::edit_op_to_pb(edit)),
            _ => None,
        })
        .collect();
    let invalidation = a
        .plan
        .as_ref()
        .map(|p| {
            if p.invalidation.is_refinement() {
                "a refinement: nothing established elsewhere is reopened".to_owned()
            } else {
                let facts: Vec<String> = p
                    .invalidation
                    .categories
                    .iter()
                    .map(|f| format!("{f:?}").to_lowercase())
                    .collect();
                format!("an edit: reopens {}", facts.join(", "))
            }
        })
        .unwrap_or_default();
    pb::SemanticActionView {
        id: a.id.to_string(),
        title: a.title.clone(),
        kind: match a.kind {
            ActionKind::QuickFix => "quick_fix".into(),
            ActionKind::Refactor => "refactor".into(),
        },
        applicability: applicability.into(),
        reason,
        options,
        explanation: a.explanation.clone(),
        edits,
        addresses: a.addresses.clone(),
        invalidation,
    }
}

/// Deployment is target-relative and never cached with the project: the
/// same revision is analysed afresh for every target asked for.
fn analyze_deployment(session: &Session, r: &pb::AnalyzeDeploymentRequest) -> Resp {
    let snapshot = match session.project() {
        Ok(p) => p.current.clone(),
        Err(e) => return Resp::Error(session_error(&e)),
    };
    if let Some(rev) = r.revision {
        if rev != snapshot.revision.raw() {
            return Resp::Error(error(
                "deploy.stale_revision",
                &format!(
                    "The project has moved on (asked about revision {rev}, now {}).",
                    snapshot.revision.raw()
                ),
            ));
        }
    }
    let Some(target) = bdl_hardware::boards::by_name(&r.target_id) else {
        return Resp::Error(error(
            "deploy.unknown_target",
            &format!("No target named {}.", r.target_id),
        ));
    };
    // The semantic analysis is target-independent and computed as for
    // RunAnalysis; the deployment analysis is target-relative; the read
    // model joins them without feeding either back.
    let analysis = bdl_compiler::analyze(&snapshot);
    let d = bdl_compiler::analyze_deployment(&snapshot, &target);
    let report = bdl_compiler::deployment_report(&snapshot, &analysis, &d, &target);
    let mut deployment = convert::deployment_with_report_to_pb(&d, &report, &snapshot.design);
    // Whether the firmware can be built now, and what stops it (0.26):
    // the report's items and the firmware pass's own refusals, in one
    // ordered list, so *Build* is offered exactly when it would pass.
    let (ready, blockers) =
        crate::firmware::protocol::readiness_to_pb(session, &snapshot, &r.target_id, &report);
    deployment.build_ready = ready;
    deployment.build_blockers = blockers;
    Resp::Deployment(pb::DeploymentResponse {
        deployment: Some(deployment),
    })
}

fn start_simulation(session: &mut Session, r: &pb::StartSimulationRequest) -> Resp {
    let snapshot = match session.project() {
        Ok(p) => p.current.clone(),
        Err(e) => return Resp::Error(session_error(&e)),
    };
    let analysis = bdl_compiler::analyze(&snapshot);
    if !analysis.causality.valid {
        return Resp::Error(error(
            "simulation.not_causal",
            "The design has relationships that depend on each other in the same instant; fix them before simulating.",
        ));
    }
    let mut inputs = bdl_reactive::InputTrace::default();
    for i in &r.inputs {
        let Some(v) = &i.value else { continue };
        match convert::value_from_pb(v) {
            Ok(v) => inputs.set(bdl_model::DeclId::from_raw(i.mapping_id), i.tick, v),
            Err(e) => return Resp::Error(error("protocol.invalid_value", &e.to_string())),
        }
    }
    let schedule = if r.schedule.is_empty() {
        bdl_reactive::Schedule::always(&analysis.ir)
    } else {
        bdl_reactive::Schedule {
            periods: r
                .schedule
                .iter()
                .map(|p| (bdl_model::ClockId::from_raw(p.clock_id), p.period))
                .collect(),
        }
    };
    let concept_names = analysis
        .ir
        .concepts
        .values()
        .map(|c| (c.id, c.name.clone()))
        .collect();
    match bdl_reactive::Simulation::new(analysis.ir, &analysis.causality, schedule, inputs) {
        Ok(simulation) => {
            let run = SimulationRun {
                revision: snapshot.revision,
                simulation,
                concept_names,
            };
            let resp = simulation_response(&run, &[], None);
            if let Ok(slot) = session.simulation_mut() {
                *slot = Some(run);
            }
            resp
        }
        Err(e) => Resp::Error(error("simulation.start", &e.to_string())),
    }
}

fn step_simulation(session: &mut Session, ticks: u64) -> Resp {
    let Ok(slot) = session.simulation_mut() else {
        return Resp::Error(error("session.no_project", "no project is open"));
    };
    let Some(run) = slot.as_mut() else {
        return Resp::Error(error("simulation.not_started", "start a simulation first"));
    };
    let from = run.simulation.trace().ticks.len();
    let mut failure = None;
    for _ in 0..ticks.max(1) {
        if let Err(e) = run.simulation.step() {
            failure = Some(e);
            break;
        }
    }
    let samples: Vec<bdl_reactive::TickSample> = run.simulation.trace().ticks[from..].to_vec();
    let error = failure.map(|e| {
        let (code, decl) = match &e {
            bdl_reactive::simulate::SimulationError::Runtime(
                bdl_reactive::eval::RuntimeError::MissingInput { decl, .. },
            ) => ("simulation.missing_input", Some(*decl)),
            bdl_reactive::simulate::SimulationError::Runtime(
                bdl_reactive::eval::RuntimeError::DivisionByZero { decl, .. },
            ) => ("simulation.division_by_zero", Some(*decl)),
            bdl_reactive::simulate::SimulationError::Runtime(
                bdl_reactive::eval::RuntimeError::NonFinite { decl, .. },
            ) => ("simulation.non_finite", Some(*decl)),
            _ => ("simulation.runtime", None),
        };
        let d = bdl_diagnostics::Diagnostic::error(
            code,
            decl.map(|d| bdl_diagnostics::Entity::Mapping { id: d })
                .unwrap_or(bdl_diagnostics::Entity::Project),
            e.to_string(),
        );
        convert::diagnostic_to_pb(&d)
    });
    simulation_response(run, &samples, error)
}

fn reset_simulation(session: &mut Session) -> Resp {
    let Ok(slot) = session.simulation_mut() else {
        return Resp::Error(error("session.no_project", "no project is open"));
    };
    let Some(run) = slot.as_mut() else {
        return Resp::Error(error("simulation.not_started", "start a simulation first"));
    };
    run.simulation.reset();
    simulation_response(run, &[], None)
}

fn simulation_response(
    run: &SimulationRun,
    samples: &[bdl_reactive::TickSample],
    error: Option<pb::Diagnostic>,
) -> Resp {
    let names = run.concept_names.clone();
    let name =
        move |id: bdl_model::SemanticId| names.get(&id).cloned().unwrap_or_else(|| id.to_string());
    Resp::Simulation(pb::SimulationResponse {
        revision: run.revision.raw(),
        next_tick: run.simulation.tick(),
        samples: samples
            .iter()
            .map(|t| convert::tick_sample_to_pb(t, &name))
            .collect(),
        error,
    })
}

/// Undo / redo.  A flat project answers `EditApplied`; a system project
/// answers `SystemEditApplied` (the system alongside) and pushes a
/// `ProjectChanged` only for a semantic step — an authoring step (a group
/// edit undone) moves no revision.
fn step_result(session: &mut Session, undo: bool) -> (Resp, Option<Committed>) {
    let is_system = session.project().map(|p| p.is_system()).unwrap_or(false);
    if !is_system {
        let r = if undo { session.undo() } else { session.redo() };
        return match r {
            Ok(c) => (
                Resp::EditApplied(pb::EditApplied {
                    project: Some(project_of(session)),
                    outcome: None,
                }),
                Some(c),
            ),
            Err(e) => (Resp::Error(session_error(&e)), None),
        };
    }
    match session.system_step(undo) {
        Ok(stepped) => {
            let view = system_view(session).ok();
            let resp = Resp::SystemEditApplied(pb::SystemEditApplied {
                system: view,
                project: Some(project_of(session)),
                outcome: None,
            });
            match stepped {
                crate::session::Stepped::Semantic(c) => (resp, Some(*c)),
                crate::session::Stepped::Authoring => (resp, None),
            }
        }
        Err(e) => (Resp::Error(session_error(&e)), None),
    }
}

fn anchor_to_pb(a: &crate::session::SourceAnchor) -> Option<pb::SourceAnchor> {
    use bdl_text::TextEntity as E;
    use pb::source_anchor::Entity;
    let (component, entity) = match a.entity {
        E::Concept(id) => (None, Entity::ConceptId(id.raw())),
        E::Mapping(id) => (None, Entity::MappingId(id.raw())),
        E::Clock(id) => (None, Entity::ClockId(id.raw())),
        E::Output(id) => (None, Entity::OutputId(id.raw())),
        E::Device(id) => (None, Entity::DeviceId(id.raw())),
        E::Component(c) => (None, Entity::ComponentId(c.raw())),
        E::BodyConcept(c, id) => (Some(c.raw()), Entity::ConceptId(id.raw())),
        E::BodyMapping(c, id) => (Some(c.raw()), Entity::MappingId(id.raw())),
        E::BodyClock(c, id) => (Some(c.raw()), Entity::ClockId(id.raw())),
        E::BodyOutput(c, id) => (Some(c.raw()), Entity::OutputId(id.raw())),
        E::BodyDevice(c, id) => (Some(c.raw()), Entity::DeviceId(id.raw())),
        E::Port(c, id) => (Some(c.raw()), Entity::PortId(id.raw())),
        E::Instance(id) => (None, Entity::InstanceId(id.raw())),
        E::Binding(_) | E::Export(_) => return None,
    };
    Some(pb::SourceAnchor {
        start: a.start,
        end: a.end,
        component,
        entity: Some(entity),
    })
}

fn sources_to_pb(s: &crate::session::Sources) -> pb::SourcesView {
    pb::SourcesView {
        revision: s.revision.raw(),
        files: s
            .files
            .iter()
            .map(|f| pb::SourceFileView {
                path: f.path.clone(),
                text: f.text.clone(),
                draft: f.draft,
                anchors: f.anchors.iter().filter_map(anchor_to_pb).collect(),
            })
            .collect(),
        diagnostics: s
            .diagnostics
            .iter()
            .map(|d| pb::SourceDiagnostic {
                path: d.path.clone(),
                code: d.code.clone(),
                message: d.message.clone(),
                start: d.start,
                end: d.end,
                open: d.open,
            })
            .collect(),
    }
}

fn project_response(session: &Session) -> Resp {
    Resp::Project(pb::ProjectResponse {
        project: Some(project_of(session)),
    })
}

/// Projection of the open project's current snapshot.  Callers only invoke
/// this with a project open; an absent project renders as an empty
/// projection rather than a panic.
fn project_of(session: &Session) -> pb::ProjectProjection {
    match session.project() {
        Ok(p) => projection_of(session, &p.current),
        Err(_) => pb::ProjectProjection::default(),
    }
}

/// Projection of a specific snapshot (e.g. the one a commit produced) with
/// the session's layout and undo/dirty facts.
fn projection_of(
    session: &Session,
    snapshot: &bdl_model::ProjectSnapshot,
) -> pb::ProjectProjection {
    match session.project() {
        Ok(p) => convert::projection(
            snapshot,
            &p.layout,
            &SessionInfo {
                root_path: p.root.to_string_lossy().into_owned(),
                can_undo: p.can_undo(),
                can_redo: p.can_redo(),
                dirty: p.dirty(),
                derived: p.is_system(),
                textual: p.is_text(),
            },
        ),
        Err(_) => pb::ProjectProjection::default(),
    }
}

/// The system view of the open system project.
fn system_view(session: &mut Session) -> Result<pb::SystemView, SessionError> {
    let boundaries = session.group_boundaries()?;
    let p = session.project()?;
    let sys = p.system.as_ref().ok_or(SessionError::NotASystem)?;
    Ok(convert::system::system_view(
        &sys.current,
        &sys.flattened.origins,
        sys.authoring_generation,
        &boundaries,
        p.dirty(),
        &p.definition_drafts(),
    ))
}

fn component_scope(component: Option<u64>) -> Option<bdl_system::ComponentId> {
    component.map(bdl_system::ComponentId::from_raw)
}

fn session_error(e: &SessionError) -> pb::Error {
    match e {
        SessionError::Edit(edit) => convert::edit_error_to_pb(edit),
        // A base edit that fails inside a system edit is that edit's
        // refusal (`edit.<reason>`): every project is a system (ADR-0023)
        // and the reasons speak the design's language.
        SessionError::SystemEdit(bdl_system::SystemEditError::Base(inner)) => {
            convert::edit_error_to_pb(inner)
        }
        SessionError::LibraryPlan(message) => pb::Error {
            code: "library.invalid_plan".into(),
            message: message.clone(),
            details_json: String::new(),
        },
        SessionError::SystemEdit(edit) => pb::Error {
            code: format!("system_edit.{}", system_edit_code(edit)),
            message: edit.to_string(),
            details_json: serde_json::to_string(edit).unwrap_or_default(),
        },
        SessionError::GroupEdit(edit) => pb::Error {
            code: format!("group_edit.{}", group_edit_code(edit)),
            message: edit.to_string(),
            details_json: serde_json::to_string(edit).unwrap_or_default(),
        },
        SessionError::Extraction(x) => pb::Error {
            code: format!("extract.{}", extract_code(x)),
            message: x.to_string(),
            details_json: serde_json::to_string(x).unwrap_or_default(),
        },
        SessionError::UnknownComponent(_) => error("system.unknown_component", &e.to_string()),
        SessionError::StaleGeneration { .. } => {
            error("group_edit.stale_generation", &e.to_string())
        }
        SessionError::NotASystem => error("system.not_a_system", &e.to_string()),
        SessionError::NoProject => error("session.no_project", &e.to_string()),
        SessionError::AlreadyOpen(_) => error("session.already_open", &e.to_string()),
        SessionError::StaleRevision { .. } => error("edit.stale_revision", &e.to_string()),
        SessionError::NothingToUndo => error("edit.nothing_to_undo", &e.to_string()),
        SessionError::NothingToRedo => error("edit.nothing_to_redo", &e.to_string()),
        SessionError::Persist(_) => error("project.persist", &e.to_string()),
        SessionError::Text(_) => error("project.text", &e.to_string()),
        SessionError::InvalidName { reason, .. } => error("edit.invalid_name", reason),
        SessionError::InvalidSourcePath { .. } => error("source.invalid_path", &e.to_string()),
        SessionError::ChangedOnDisk { .. } => error("project.changed_on_disk", &e.to_string()),
        SessionError::Ide(bdl_ide::QueryError::UnknownEntity { .. }) => {
            error("draft.unknown_mapping", &e.to_string())
        }
        SessionError::Ide(bdl_ide::QueryError::NotApplicable { reason }) => {
            error("formula.not_applicable", reason)
        }
        SessionError::Ide(_) => error("draft.unavailable", &e.to_string()),
    }
}

/// A stable snake_case code per refusal kind.
fn system_edit_code(e: &bdl_system::SystemEditError) -> String {
    use bdl_system::SystemEditError as E;
    match e {
        E::EmptyName => "empty_name",
        E::DuplicateComponentName { .. } => "duplicate_component_name",
        E::UnknownComponent { .. } => "unknown_component",
        E::ComponentInUse { .. } => "component_in_use",
        E::UnknownInstance { .. } => "unknown_instance",
        E::DuplicateInstanceName { .. } => "duplicate_instance_name",
        E::InstanceInUse { .. } => "instance_in_use",
        E::UnknownPort { .. } => "unknown_port",
        E::DuplicatePortName { .. } => "duplicate_port_name",
        E::NotABodyDeclaration { .. } => "not_a_body_declaration",
        E::DeclAlreadyExposed { .. } => "decl_already_exposed",
        E::PortInUse { .. } => "port_in_use",
        E::PortBacked { .. } => "port_backed",
        E::NotABodyClock { .. } => "not_a_body_clock",
        E::ClockParamInUse { .. } => "clock_param_in_use",
        E::NotABodyConcept { .. } => "not_a_body_concept",
        E::NotABodyOutput { .. } => "not_a_body_output",
        E::UnknownSystemConcept { .. } => "unknown_system_concept",
        E::UnknownSystemOutput { .. } => "unknown_system_output",
        E::UnknownSystemClock { .. } => "unknown_system_clock",
        E::NotAClockParameter { .. } => "not_a_clock_parameter",
        E::NotAParameter { .. } => "not_a_parameter",
        E::SourceNotProvided { .. } => "source_not_provided",
        E::DestinationNotRequired { .. } => "destination_not_required",
        E::DestinationBound { .. } => "destination_bound",
        E::PortExported { .. } => "port_exported",
        E::PortHasValue => "port_has_value",
        E::UnknownBinding { .. } => "unknown_binding",
        E::UnknownExport { .. } => "unknown_export",
        E::DuplicateExportName { .. } => "duplicate_export_name",
        E::ExportNotRequired { .. } => "export_not_required",
        E::NotSubstitutable { .. } => "not_substitutable",
        E::NotABaseDeclaration { .. } => "not_a_base_declaration",
        E::BaseNotOpen { .. } => "base_not_open",
        E::UnknownGroup { .. } => "unknown_group",
        E::Extraction(x) => return format!("extract.{}", extract_code(x)),
        E::Base(_) => "base",
        E::Body { .. } => "body",
    }
    .to_owned()
}

fn group_edit_code(e: &bdl_system::GroupEditError) -> &'static str {
    use bdl_system::GroupEditError as E;
    match e {
        E::EmptyName => "empty_name",
        E::DuplicateGroupName { .. } => "duplicate_group_name",
        E::UnknownGroup { .. } => "unknown_group",
        E::NotABaseDeclaration { .. } => "not_a_base_declaration",
        E::AlreadyGrouped { .. } => "already_grouped",
        E::NotAMember { .. } => "not_a_member",
        E::SameGroup => "same_group",
        E::ScopeMismatch => "scope_mismatch",
        E::UnknownComponent { .. } => "unknown_component",
    }
}

fn extract_code(e: &bdl_system::ExtractError) -> &'static str {
    use bdl_system::ExtractError as E;
    match e {
        E::UnknownGroup { .. } => "unknown_group",
        E::EmptyGroup { .. } => "empty_group",
        E::EmptyName => "empty_name",
        E::DuplicateComponentName { .. } => "duplicate_component_name",
        E::DuplicateInstanceName { .. } => "duplicate_instance_name",
        E::NotAnOpenMember { .. } => "not_an_open_member",
        E::NotADrivenSink { .. } => "not_a_driven_sink",
        E::NotABaseGroup { .. } => "not_a_base_group",
    }
}

fn error(code: &str, message: &str) -> pb::Error {
    pb::Error {
        code: code.to_owned(),
        message: message.to_owned(),
        details_json: String::new(),
    }
}

async fn send_response(tx: &mpsc::Sender<pb::ServerMessage>, request_id: u64, payload: Resp) {
    let msg = pb::ServerMessage {
        payload: Some(pb::server_message::Payload::Response(pb::Response {
            request_id,
            payload: Some(payload),
        })),
    };
    let _ = tx.send(msg).await;
}

fn payload_name(p: &Req) -> &'static str {
    match p {
        Req::Handshake(_) => "handshake",
        Req::OpenProject(_) => "open_project",
        Req::InitProject(_) => "init_project",
        Req::InitSystemProject(_) => "init_system_project",
        Req::InitTextProject(_) => "init_text_project",
        Req::ReloadProject(_) => "reload_project",
        Req::GetSources(_) => "get_sources",
        Req::ApplySourceEdit(_) => "apply_source_edit",
        Req::ApplyGroupEdit(_) => "apply_group_edit",
        Req::PreviewComponentExtraction(_) => "preview_component_extraction",
        Req::GetSystem(_) => "get_system",
        Req::ApplySystemEdit(_) => "apply_system_edit",
        Req::RunSystemAnalysis(_) => "run_system_analysis",
        Req::SaveProject(_) => "save_project",
        Req::CloseProject(_) => "close_project",
        Req::GetProject(_) => "get_project",
        Req::ApplyEdit(_) => "apply_edit",
        Req::Undo(_) => "undo",
        Req::Redo(_) => "redo",
        Req::SetLayout(_) => "set_layout",
        Req::ArrangeLayout(_) => "arrange_layout",
        Req::SubscribeProject(_) => "subscribe_project",
        Req::RunAnalysis(_) => "run_analysis",
        Req::StartSimulation(_) => "start_simulation",
        Req::StepSimulation(_) => "step_simulation",
        Req::ResetSimulation(_) => "reset_simulation",
        Req::ListTargets(_) => "list_targets",
        Req::AnalyzeDeployment(_) => "analyze_deployment",
        Req::BuildFirmware(_) => "build_firmware",
        Req::GetBuildStatus(_) => "get_build_status",
        Req::CancelBuild(_) => "cancel_build",
        Req::ListFlashDevices(_) => "list_flash_devices",
        Req::FlashFirmware(_) => "flash_firmware",
        Req::ListTemplates(_) => "list_templates",
        Req::AnalyzeDefinitionDraft(_) => "analyze_definition_draft",
        Req::DiscardDefinitionDraft(_) => "discard_definition_draft",
        Req::CompleteDefinitionDraft(_) => "complete_definition_draft",
        Req::HoverDefinitionDraft(_) => "hover_definition_draft",
        Req::SemanticTokens(_) => "semantic_tokens",
        Req::SourceCompletion(_) => "source_completion",
        Req::SourceHover(_) => "source_hover",
        Req::SourceDefinition(_) => "source_definition",
        Req::SourceReferences(_) => "source_references",
        Req::FormatSource(_) => "format_source",
        Req::HoverEntity(_) => "hover_entity",
        Req::ListSemanticActions(_) => "list_semantic_actions",
        Req::GetFormulaProjection(_) => "get_formula_projection",
        Req::GetFormulaSlot(_) => "get_formula_slot",
        Req::ComposeFormula(_) => "compose_formula",
        Req::ListConceptTemplates(_) => "list_concept_templates",
        Req::ListValueCategories(_) => "list_value_categories",
        Req::NavigateFormula(_) => "navigate_formula",
        Req::CompleteFormulaCaret(_) => "complete_formula_caret",
        Req::GetFormulaSignature(_) => "get_formula_signature",
        Req::GetFormulaRender(_) => "get_formula_render",
        Req::InstantiateConceptTemplate(_) => "instantiate_concept_template",
        Req::ListLibraryItems(_) => "list_library_items",
        Req::InstantiateLibraryItem(_) => "instantiate_library_item",
        Req::CreateSource(_) => "create_source",
        Req::ListSourceCandidates(_) => "list_source_candidates",
        Req::Shutdown(_) => "shutdown",
    }
}
