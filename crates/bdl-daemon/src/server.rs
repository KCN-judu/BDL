//! The imperative shell around [`Session`]: frames in, frames out.
//!
//! One reader task decodes `ClientMessage`s from stdin; one writer task
//! encodes `ServerMessage`s to stdout; the coordinator in between owns the
//! session and handles requests strictly in order.  Nothing else ever
//! touches the session.

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

        let (resp, committed) = handle(&mut session, payload);
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
        Req::InitProject(i) => match session.init(Path::new(&i.root_path), &i.name) {
            Ok(_) => (project_response(session), None),
            Err(e) => (Resp::Error(session_error(&e)), None),
        },
        Req::SaveProject(_) => match session.save() {
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
        Req::Undo(_) => commit_result(session, session_undo),
        Req::Redo(_) => commit_result(session, session_redo),
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
        Req::AnalyzeDeployment(r) => (analyze_deployment(session, &r.target_id), None),
        Req::AnalyzeDefinitionDraft(r) => (analyze_definition_draft(session, &r), None),
        Req::DiscardDefinitionDraft(r) => {
            match session.discard_draft(bdl_model::DeclId::from_raw(r.mapping_id)) {
                Ok(_) => (Resp::Ack(pb::Ack {}), None),
                Err(e) => (Resp::Error(session_error(&e)), None),
            }
        }
        Req::CompleteDefinitionDraft(r) => (complete_definition_draft(session, &r), None),
        Req::HoverDefinitionDraft(r) => (hover_definition_draft(session, &r), None),
        Req::HoverEntity(r) => (hover_entity(session, &r), None),
        Req::ListSemanticActions(r) => (list_semantic_actions(session, &r), None),
        Req::Shutdown(_) => (Resp::Ack(pb::Ack {}), None),
    }
}

/// A candidate definition checked against the current snapshot and never
/// committed: the project, its revision and its undo history are untouched.
///
/// The draft is an *overlay* on the session's IDE host: the request
/// updates it, an immutable snapshot of committed + overlays is taken, and
/// `bdl-ide` returns the stamped verdict — the same path a text editor's
/// unsaved buffer takes (`docs/IDE_SERVICE_ARCHITECTURE.md`).
fn analyze_definition_draft(session: &mut Session, r: &pb::AnalyzeDefinitionDraftRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.draft_verdict(id, &r.source) {
        Ok(v) => Resp::DefinitionDraft(pb::DefinitionDraftAnalysis {
            revision: v.stamp.revision.raw(),
            mapping_id: r.mapping_id,
            generation: r.generation,
            parse_ok: v.parse_ok,
            analysis: Some(convert::mapping_analysis_to_pb(&v.analysis)),
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
    match session.draft_completion(id, &r.source, r.offset) {
        Ok(items) => Resp::DraftCompletion(pb::DraftCompletionResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            items: items
                .iter()
                .map(|c| pb::DraftCompletionItem {
                    label: c.label.clone(),
                    kind: format!("{:?}", c.kind).to_lowercase(),
                    replace_start: c.replace.start,
                    replace_end: c.replace.end,
                    insert: c.insert.clone(),
                    resulting_type: c.resulting_type.clone().unwrap_or_default(),
                    documentation: c.documentation.clone().unwrap_or_default(),
                    relevance: u32::from(c.relevance),
                })
                .collect(),
        }),
        Err(e) => Resp::Error(session_error(&e)),
    }
}

fn hover_definition_draft(session: &mut Session, r: &pb::HoverDefinitionDraftRequest) -> Resp {
    if let Err(e) = draft_revision(session, r.revision) {
        return Resp::Error(e);
    }
    let id = bdl_model::DeclId::from_raw(r.mapping_id);
    match session.draft_hover(id, &r.source, r.offset) {
        Ok(None) => Resp::DraftHover(pb::DraftHoverResponse {
            revision: r.revision,
            mapping_id: r.mapping_id,
            found: false,
            ..Default::default()
        }),
        Ok(Some((range, h))) => Resp::DraftHover(pb::DraftHoverResponse {
            mapping_id: r.mapping_id,
            span: Some(pb::SourceSpan {
                start: range.start,
                end: range.end,
            }),
            ..hover_to_pb(h, r.revision)
        }),
        Err(e) => Resp::Error(session_error(&e)),
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

fn entity_to_pb(e: bdl_ide::EntityRef) -> pb::EntityRef {
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
        }),
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
        e => bdl_ide::DiagnosticScope::Entity(e),
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
fn analyze_deployment(session: &Session, target_id: &str) -> Resp {
    let snapshot = match session.project() {
        Ok(p) => p.current.clone(),
        Err(e) => return Resp::Error(session_error(&e)),
    };
    let Some(target) = bdl_hardware::boards::by_name(target_id) else {
        return Resp::Error(error(
            "deploy.unknown_target",
            &format!("No target named {target_id}."),
        ));
    };
    let d = bdl_compiler::analyze_deployment(&snapshot, &target);
    Resp::Deployment(pb::DeploymentResponse {
        deployment: Some(convert::deployment_to_pb(&d)),
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

fn session_undo(s: &mut Session) -> Result<Committed, SessionError> {
    s.undo()
}
fn session_redo(s: &mut Session) -> Result<Committed, SessionError> {
    s.redo()
}

fn commit_result(
    session: &mut Session,
    f: fn(&mut Session) -> Result<Committed, SessionError>,
) -> (Resp, Option<Committed>) {
    match f(session) {
        Ok(c) => (
            Resp::EditApplied(pb::EditApplied {
                project: Some(project_of(session)),
                outcome: None,
            }),
            Some(c),
        ),
        Err(e) => (Resp::Error(session_error(&e)), None),
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
            },
        ),
        Err(_) => pb::ProjectProjection::default(),
    }
}

fn session_error(e: &SessionError) -> pb::Error {
    match e {
        SessionError::Edit(edit) => convert::edit_error_to_pb(edit),
        SessionError::NoProject => error("session.no_project", &e.to_string()),
        SessionError::AlreadyOpen(_) => error("session.already_open", &e.to_string()),
        SessionError::StaleRevision { .. } => error("edit.stale_revision", &e.to_string()),
        SessionError::NothingToUndo => error("edit.nothing_to_undo", &e.to_string()),
        SessionError::NothingToRedo => error("edit.nothing_to_redo", &e.to_string()),
        SessionError::Persist(_) => error("project.persist", &e.to_string()),
        SessionError::Ide(bdl_ide::QueryError::UnknownEntity { .. }) => {
            error("draft.unknown_mapping", &e.to_string())
        }
        SessionError::Ide(_) => error("draft.unavailable", &e.to_string()),
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
        Req::SaveProject(_) => "save_project",
        Req::CloseProject(_) => "close_project",
        Req::GetProject(_) => "get_project",
        Req::ApplyEdit(_) => "apply_edit",
        Req::Undo(_) => "undo",
        Req::Redo(_) => "redo",
        Req::SetLayout(_) => "set_layout",
        Req::SubscribeProject(_) => "subscribe_project",
        Req::RunAnalysis(_) => "run_analysis",
        Req::StartSimulation(_) => "start_simulation",
        Req::StepSimulation(_) => "step_simulation",
        Req::ResetSimulation(_) => "reset_simulation",
        Req::ListTargets(_) => "list_targets",
        Req::AnalyzeDeployment(_) => "analyze_deployment",
        Req::AnalyzeDefinitionDraft(_) => "analyze_definition_draft",
        Req::DiscardDefinitionDraft(_) => "discard_definition_draft",
        Req::CompleteDefinitionDraft(_) => "complete_definition_draft",
        Req::HoverDefinitionDraft(_) => "hover_definition_draft",
        Req::HoverEntity(_) => "hover_entity",
        Req::ListSemanticActions(_) => "list_semantic_actions",
        Req::Shutdown(_) => "shutdown",
    }
}
