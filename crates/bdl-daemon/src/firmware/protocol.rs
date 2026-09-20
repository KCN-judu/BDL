//! The firmware requests of the protocol (0.26) over [`crate::firmware`]:
//! a build or a flash runs on its own thread against a snapshot taken
//! when it started, reports every stage as an event, and leaves its
//! outcome here for `GetBuildStatus`.  One build and one flash at a time;
//! the coordinator never waits on either.

use crate::firmware::{self as build, flash, readiness, Plan};
use crate::session::Session;
use bdl_protocol::convert;
use bdl_protocol::pb::{self, response::Payload as Resp};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// What is known in memory about one target's builds: the running one,
/// the last failure, the last output.
#[derive(Default)]
struct TargetState {
    running: Option<Running>,
    last_stage: Option<build::Stage>,
    failure: Option<build::Failure>,
    output: Vec<String>,
    command: String,
    triple: String,
    generated_dir: String,
}

struct Running {
    cancel: Arc<AtomicBool>,
    stage: build::Stage,
}

#[derive(Default)]
pub struct Firmware {
    targets: Arc<Mutex<HashMap<String, TargetState>>>,
    flashing: Arc<AtomicBool>,
    next_id: u64,
}

fn error(code: &str, message: &str) -> pb::Error {
    pb::Error {
        code: code.to_owned(),
        message: message.to_owned(),
        details_json: String::new(),
    }
}

fn event(payload: pb::event::Payload) -> pb::ServerMessage {
    pb::ServerMessage {
        payload: Some(pb::server_message::Payload::Event(pb::Event {
            payload: Some(payload),
        })),
    }
}

pub fn stage_to_pb(s: build::Stage) -> pb::BuildStage {
    match s {
        build::Stage::Checking => pb::BuildStage::Checking,
        build::Stage::Generating => pb::BuildStage::Generating,
        build::Stage::Preparing => pb::BuildStage::Preparing,
        build::Stage::Compiling => pb::BuildStage::Compiling,
        build::Stage::Packaging => pb::BuildStage::Packaging,
        build::Stage::Completed => pb::BuildStage::Completed,
        build::Stage::Failed => pb::BuildStage::Failed,
        build::Stage::Cancelled => pb::BuildStage::Cancelled,
    }
}

fn flash_stage_to_pb(s: flash::Stage) -> pb::FlashStage {
    match s {
        flash::Stage::Preparing => pb::FlashStage::Preparing,
        flash::Stage::Writing => pb::FlashStage::Writing,
        flash::Stage::Restarting => pb::FlashStage::Restarting,
        flash::Stage::Completed => pb::FlashStage::Completed,
    }
}

fn method_to_pb(m: flash::Method) -> pb::FlashMethod {
    match m {
        flash::Method::Uf2Volume => pb::FlashMethod::Uf2Volume,
        flash::Method::Probe => pb::FlashMethod::Probe,
    }
}

pub fn artifact_to_pb(a: &build::Artifact) -> pb::BuildArtifact {
    pb::BuildArtifact {
        path: a.path.display().to_string(),
        kind: a.kind.clone(),
        elf_path: a.elf_path.display().to_string(),
        identity: a.identity.clone(),
        built_at: a.built_at,
        revision: a.revision,
        compiler_version: a.compiler_version.clone(),
        size_bytes: a.size_bytes,
    }
}

fn failure_to_pb(f: &build::Failure) -> pb::BuildFailure {
    pb::BuildFailure {
        stage: stage_to_pb(f.stage).into(),
        code: f.code.to_owned(),
        message: f.message.clone(),
        explanation: f.explanation.clone(),
        diagnostics: f
            .diagnostics
            .iter()
            .map(convert::diagnostic_to_pb)
            .collect(),
        command: f.command.clone(),
        output: f.output.clone(),
    }
}

fn device_to_pb(d: &flash::Device) -> pb::FlashDevice {
    pb::FlashDevice {
        id: d.id.clone(),
        method: method_to_pb(d.method).into(),
        label: d.label.clone(),
        detail: d.detail.clone(),
    }
}

/// The readiness fields of a deployment analysis (0.26).
pub fn readiness_to_pb(
    session: &Session,
    snapshot: &bdl_model::surface::ProjectSnapshot,
    target_id: &str,
    report: &bdl_compiler::DeploymentReport,
) -> (bool, Vec<pb::BuildBlocker>) {
    let plan = session
        .project()
        .ok()
        .and_then(|p| Plan::for_target(&p.root, target_id, session.compiler_version()));
    let r = readiness::readiness(plan.as_ref(), snapshot, report);
    (
        r.ready,
        r.blockers
            .into_iter()
            .map(|b| pb::BuildBlocker {
                code: b.code,
                message: b.message,
                explanation: b.explanation,
                output_id: b.output,
                device_id: b.device,
                mapping_id: b.mapping,
            })
            .collect(),
    )
}

impl Firmware {
    /// The status of one target's firmware, composed from the running or
    /// last build, the record on disk and what the project would build
    /// now.
    pub fn status(&self, session: &Session, target_id: &str) -> Result<pb::BuildStatus, pb::Error> {
        let project = session
            .project()
            .map_err(|_| error("project.none_open", "No project is open."))?;
        let plan = Plan::for_target(&project.root, target_id, session.compiler_version());
        let mut s = pb::BuildStatus {
            target_id: target_id.to_owned(),
            ..Default::default()
        };
        let record = plan.as_ref().and_then(|p| p.record());
        if let (Some(p), Some(r)) = (&plan, &record) {
            s.artifact = Some(artifact_to_pb(&r.artifact));
            s.artifact_fresh = p
                .current_identity(&project.current)
                .map(|id| id == r.artifact.identity)
                .unwrap_or(false);
            s.generated_dir = r.generated_dir.display().to_string();
            s.command = r.command.clone();
            s.triple = r.triple.clone();
        } else if let Some(p) = &plan {
            s.generated_dir = p.out_dir.display().to_string();
            s.triple = p.entry.triple().to_owned();
        }
        let targets = self.targets.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(t) = targets.get(target_id) {
            if let Some(r) = &t.running {
                s.running = true;
                s.stage = stage_to_pb(r.stage).into();
            } else if let Some(st) = t.last_stage {
                s.stage = stage_to_pb(st).into();
            }
            if let Some(f) = &t.failure {
                s.failure = Some(failure_to_pb(f));
            }
            if !t.output.is_empty() {
                s.output = t.output.clone();
            }
            if !t.command.is_empty() {
                s.command = t.command.clone();
            }
            if !t.triple.is_empty() {
                s.triple = t.triple.clone();
            }
            if !t.generated_dir.is_empty() {
                s.generated_dir = t.generated_dir.clone();
            }
        } else if record.is_some() {
            s.stage = pb::BuildStage::Completed.into();
        }
        Ok(s)
    }

    /// Start a build; the response is immediate, the stages are events.
    pub fn start_build(
        &mut self,
        session: &Session,
        r: &pb::BuildFirmwareRequest,
        tx: &mpsc::Sender<pb::ServerMessage>,
    ) -> Resp {
        let project = match session.project() {
            Ok(p) => p,
            Err(_) => return Resp::Error(error("project.none_open", "No project is open.")),
        };
        if let Some(rev) = r.revision {
            if rev != project.current.revision.raw() {
                return Resp::Error(error(
                    "build.stale_revision",
                    "The project has moved on since the build was asked for.",
                ));
            }
        }
        let Some(plan) = Plan::for_target(&project.root, &r.target_id, session.compiler_version())
        else {
            return Resp::Error(error(
                "build.unknown_target",
                &format!("No firmware can be built for {}.", r.target_id),
            ));
        };
        let mut targets = self.targets.lock().unwrap_or_else(|e| e.into_inner());
        let state = targets.entry(r.target_id.clone()).or_default();
        if state.running.is_some() {
            return Resp::Error(error(
                "build.busy",
                "A build for this board is already running.",
            ));
        }
        self.next_id += 1;
        let id = self.next_id;
        let cancel = Arc::new(AtomicBool::new(false));
        state.running = Some(Running {
            cancel: cancel.clone(),
            stage: build::Stage::Checking,
        });
        state.failure = None;
        state.output.clear();
        state.triple = plan.entry.triple().to_owned();
        state.generated_dir = plan.out_dir.display().to_string();
        drop(targets);

        let snapshot = project.current.clone();
        let targets = self.targets.clone();
        let tx = tx.clone();
        let target_id = r.target_id.clone();
        std::thread::spawn(move || {
            let mut progress = |p: build::Progress| {
                {
                    let mut t = targets.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(state) = t.get_mut(&target_id) {
                        if let Some(run) = &mut state.running {
                            run.stage = p.stage;
                        }
                        if !p.detail.is_empty() {
                            state.output.extend(p.detail.lines().map(str::to_owned));
                            let skip = state.output.len().saturating_sub(build::OUTPUT_TAIL);
                            state.output.drain(..skip);
                        }
                    }
                }
                let _ = tx.blocking_send(event(pb::event::Payload::BuildProgress(
                    pb::BuildProgress {
                        build_id: id,
                        target_id: target_id.clone(),
                        stage: stage_to_pb(p.stage).into(),
                        message: p.message,
                        done: p.done,
                        detail: p.detail,
                        status: None,
                    },
                )));
            };
            let outcome = build::run(&plan, &snapshot, &cancel, &mut progress);
            let (stage, message, status) = {
                let mut t = targets.lock().unwrap_or_else(|e| e.into_inner());
                let state = t.entry(target_id.clone()).or_default();
                state.running = None;
                let (stage, message) = match &outcome {
                    Ok(record) => {
                        state.command = record.command.clone();
                        state.output = Vec::new();
                        (
                            build::Stage::Completed,
                            format!("Firmware built: {}", record.artifact.path.display()),
                        )
                    }
                    Err(f) => {
                        let stage = if f.stage == build::Stage::Cancelled {
                            build::Stage::Cancelled
                        } else {
                            build::Stage::Failed
                        };
                        state.failure = Some((**f).clone());
                        if !f.command.is_empty() {
                            state.command = f.command.clone();
                        }
                        if !f.output.is_empty() {
                            state.output = f.output.clone();
                        }
                        (stage, f.message.clone())
                    }
                };
                state.last_stage = Some(stage);
                let mut status = pb::BuildStatus {
                    target_id: target_id.clone(),
                    running: false,
                    stage: stage_to_pb(stage).into(),
                    artifact: None,
                    artifact_fresh: false,
                    failure: state.failure.as_ref().map(failure_to_pb),
                    generated_dir: state.generated_dir.clone(),
                    command: state.command.clone(),
                    triple: state.triple.clone(),
                    output: state.output.clone(),
                };
                if let Ok(record) = &outcome {
                    status.artifact = Some(artifact_to_pb(&record.artifact));
                    // Built from the snapshot the build started with: fresh
                    // for that snapshot by construction; the client asks
                    // again after any later revision.
                    status.artifact_fresh = true;
                }
                (stage, message, status)
            };
            let _ = tx.blocking_send(event(pb::event::Payload::BuildProgress(
                pb::BuildProgress {
                    build_id: id,
                    target_id,
                    stage: stage_to_pb(stage).into(),
                    message,
                    done: None,
                    detail: String::new(),
                    status: Some(status),
                },
            )));
        });
        Resp::Ack(pb::Ack {})
    }

    pub fn cancel(&self) -> Resp {
        let targets = self.targets.lock().unwrap_or_else(|e| e.into_inner());
        for t in targets.values() {
            if let Some(r) = &t.running {
                r.cancel.store(true, Ordering::Relaxed);
            }
        }
        Resp::Ack(pb::Ack {})
    }

    pub fn devices(&self, session: &Session, target_id: &str) -> Resp {
        let project = match session.project() {
            Ok(p) => p,
            Err(_) => return Resp::Error(error("project.none_open", "No project is open.")),
        };
        let Some(plan) = Plan::for_target(&project.root, target_id, session.compiler_version())
        else {
            return Resp::Error(error(
                "build.unknown_target",
                &format!("No firmware can be built for {target_id}."),
            ));
        };
        let (devices, methods) = flash::discover(&plan.entry.flash());
        Resp::FlashDevices(pb::FlashDevicesResponse {
            devices: devices.iter().map(device_to_pb).collect(),
            methods: methods
                .into_iter()
                .map(|m| pb::FlashMethodView {
                    method: method_to_pb(m.method).into(),
                    available: m.available,
                    label: m.label,
                    hint: m.hint,
                })
                .collect(),
        })
    }

    /// Start a flash of the last artifact to one device; refused, never
    /// guessed, when the device is ambiguous or the artifact stale.
    pub fn start_flash(
        &mut self,
        session: &Session,
        r: &pb::FlashFirmwareRequest,
        tx: &mpsc::Sender<pb::ServerMessage>,
    ) -> Resp {
        let project = match session.project() {
            Ok(p) => p,
            Err(_) => return Resp::Error(error("project.none_open", "No project is open.")),
        };
        let Some(plan) = Plan::for_target(&project.root, &r.target_id, session.compiler_version())
        else {
            return Resp::Error(error(
                "build.unknown_target",
                &format!("No firmware can be built for {}.", r.target_id),
            ));
        };
        {
            let targets = self.targets.lock().unwrap_or_else(|e| e.into_inner());
            if targets
                .get(&r.target_id)
                .is_some_and(|t| t.running.is_some())
            {
                return Resp::Error(error(
                    "flash.busy",
                    "A build is running; flash when it completes.",
                ));
            }
        }
        if self.flashing.load(Ordering::Relaxed) {
            return Resp::Error(error("flash.busy", "A flash is already running."));
        }
        let Some(record) = plan.record() else {
            return Resp::Error(error(
                "flash.no_artifact",
                "Nothing has been built for this board yet.",
            ));
        };
        let fresh = plan
            .current_identity(&project.current)
            .map(|id| id == record.artifact.identity)
            .unwrap_or(false);
        if !fresh {
            return Resp::Error(error(
                "flash.artifact_stale",
                "The last firmware was built from an earlier design or deployment; build again first.",
            ));
        }
        let (devices, _) = flash::discover(&plan.entry.flash());
        let device = match &r.device_id {
            Some(id) => match devices.iter().find(|d| &d.id == id) {
                Some(d) => d.clone(),
                None => {
                    return Resp::Error(error(
                        "flash.unknown_device",
                        "That device is no longer reachable; look again.",
                    ))
                }
            },
            None => match devices.as_slice() {
                [] => return Resp::Error(error(
                    "flash.no_device",
                    "No board is reachable. Hold BOOTSEL while plugging the Pico in, then flash.",
                )),
                [one] => one.clone(),
                _ => {
                    return Resp::Error(error(
                        "flash.ambiguous_device",
                        "Several devices are reachable; choose the one to flash.",
                    ))
                }
            },
        };
        self.flashing.store(true, Ordering::Relaxed);
        self.next_id += 1;
        let id = self.next_id;
        let flashing = self.flashing.clone();
        let tx = tx.clone();
        let target_id = r.target_id.clone();
        let spec = plan.entry.flash();
        std::thread::spawn(move || {
            let artifact = record.artifact.clone();
            let device_pb = device_to_pb(&device);
            let send =
                |stage: pb::FlashStage, message: String, failure: Option<pb::FlashFailure>| {
                    let _ = tx.blocking_send(event(pb::event::Payload::FlashProgress(
                        pb::FlashProgress {
                            flash_id: id,
                            target_id: target_id.clone(),
                            stage: stage.into(),
                            message,
                            device: Some(device_pb.clone()),
                            artifact: Some(artifact_to_pb(&artifact)),
                            failure,
                        },
                    )));
                };
            let mut progress =
                |p: flash::Progress| send(flash_stage_to_pb(p.stage), p.message, None);
            let outcome = flash::flash(&device, &artifact, &spec, &mut progress);
            if let Err(f) = outcome {
                send(
                    pb::FlashStage::Failed,
                    f.message.clone(),
                    Some(pb::FlashFailure {
                        stage: flash_stage_to_pb(f.stage).into(),
                        code: f.code.to_owned(),
                        message: f.message,
                        explanation: f.explanation,
                        command: f.command,
                        output: f.output,
                    }),
                );
            }
            flashing.store(false, Ordering::Relaxed);
        });
        Resp::Ack(pb::Ack {})
    }
}

pub fn templates_response() -> Resp {
    Resp::Templates(pb::TemplatesResponse {
        templates: crate::templates::templates()
            .into_iter()
            .map(|t| pb::TemplateView {
                id: t.id.to_owned(),
                display_name: t.display_name.to_owned(),
                description: t.description.to_owned(),
                target_id: t.target_id.to_owned(),
                configured: t.configured,
            })
            .collect(),
    })
}
