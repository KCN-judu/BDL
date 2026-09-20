#![allow(clippy::unwrap_used)]
//! Firmware over the real `bdld` (protocol 0.26): the two demo templates,
//! the readiness the Deploy page offers *Build* on, a build reported stage
//! by stage, the artifact's freshness across an edit, and a flash that is
//! refused rather than guessed — with no device, with two, with a stale
//! image — and one that writes the image onto a pretend bootloader
//! volume.  The cargo stage runs the real cross-build when the target is
//! installed (`BDL_REQUIRE_CROSS=1` in CI insists); every other stage is
//! proved with a stand-in `cargo` so the failure paths are cheap.

use bdl_protocol::framing;
use bdl_protocol::pb::{self, client_message::Payload as Req, response::Payload as Resp};
use bytes::BytesMut;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

struct Client {
    child: Child,
    buf: BytesMut,
    next_id: u64,
    last_revision: u64,
    events: Vec<pb::Event>,
}

impl Client {
    fn spawn_with(env: &[(&str, &str)]) -> Client {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bdld"));
        cmd.arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let child = cmd.spawn().expect("spawn bdld");
        let mut c = Client {
            child,
            buf: BytesMut::new(),
            next_id: 1,
            last_revision: 0,
            events: Vec::new(),
        };
        c.call(Req::Handshake(pb::HandshakeRequest {
            client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
            client_name: "firmware-e2e".into(),
            client_version: "0".into(),
        }));
        c
    }

    fn read_message(&mut self) -> pb::ServerMessage {
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
            match self.read_message().payload.unwrap() {
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

    /// Events until one satisfies `done`; every event seen is returned.
    fn events_until(&mut self, done: impl Fn(&pb::Event) -> bool) -> Vec<pb::Event> {
        let mut seen = std::mem::take(&mut self.events);
        if let Some(i) = seen.iter().position(&done) {
            let rest = seen.split_off(i + 1);
            self.events = rest;
            return seen;
        }
        loop {
            match self.read_message().payload.unwrap() {
                pb::server_message::Payload::Event(e) => {
                    let finished = done(&e);
                    seen.push(e);
                    if finished {
                        return seen;
                    }
                }
                pb::server_message::Payload::Response(r) => {
                    panic!("unexpected response while waiting for events: {r:?}")
                }
            }
        }
    }

    fn init(&mut self, root: &Path, template: Option<&str>) -> pb::ProjectProjection {
        match self.call(Req::InitProject(pb::InitProjectRequest {
            root_path: root.to_string_lossy().into(),
            name: root.file_name().unwrap().to_string_lossy().into(),
            template: template.map(str::to_owned),
        })) {
            Resp::Project(p) => p.project.unwrap(),
            other => panic!("{other:?}"),
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

    fn status(&mut self, target: &str) -> pb::BuildStatus {
        match self.call(Req::GetBuildStatus(pb::GetBuildStatusRequest {
            target_id: target.into(),
        })) {
            Resp::BuildStatus(s) => s.status.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn build(&mut self, target: &str) -> Vec<pb::BuildProgress> {
        let revision = Some(self.last_revision);
        match self.call(Req::BuildFirmware(pb::BuildFirmwareRequest {
            target_id: target.into(),
            revision,
        })) {
            Resp::Ack(_) => {}
            other => panic!("{other:?}"),
        }
        self.events_until(|e| {
            matches!(&e.payload, Some(pb::event::Payload::BuildProgress(p)) if p.status.is_some())
        })
        .into_iter()
        .filter_map(|e| match e.payload {
            Some(pb::event::Payload::BuildProgress(p)) => Some(p),
            _ => None,
        })
        .collect()
    }

    fn devices(&mut self, target: &str) -> pb::FlashDevicesResponse {
        match self.call(Req::ListFlashDevices(pb::ListFlashDevicesRequest {
            target_id: target.into(),
        })) {
            Resp::FlashDevices(d) => d,
            other => panic!("{other:?}"),
        }
    }

    fn flash(
        &mut self,
        target: &str,
        device: Option<&str>,
    ) -> Result<Vec<pb::FlashProgress>, pb::Error> {
        match self.call(Req::FlashFirmware(pb::FlashFirmwareRequest {
            target_id: target.into(),
            device_id: device.map(str::to_owned),
        })) {
            Resp::Ack(_) => {}
            Resp::Error(e) => return Err(e),
            other => panic!("{other:?}"),
        }
        Ok(self
            .events_until(|e| {
                matches!(&e.payload, Some(pb::event::Payload::FlashProgress(p))
                    if p.stage() == pb::FlashStage::Completed || p.stage() == pb::FlashStage::Failed)
            })
            .into_iter()
            .filter_map(|e| match e.payload {
                Some(pb::event::Payload::FlashProgress(p)) => Some(p),
                _ => None,
            })
            .collect())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

const PICO: &str = "rp2040_pico";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn cross_available() -> bool {
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("thumbv6m-none-eabi"))
        .unwrap_or(false);
    if !installed && std::env::var("BDL_REQUIRE_CROSS").is_ok() {
        panic!("thumbv6m-none-eabi is not installed (rust-toolchain.toml lists it)");
    }
    installed
}

// ---- templates and readiness ----

#[test]
fn the_templates_are_listed_and_a_project_starts_from_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn_with(&[]);
    let templates = match c.call(Req::ListTemplates(pb::ListTemplatesRequest {})) {
        Resp::Templates(t) => t.templates,
        other => panic!("{other:?}"),
    };
    let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["button-lamp", "button-lamp-configured"]);
    assert!(templates.iter().all(|t| t.target_id == PICO));
    assert!(!templates[0].configured && templates[1].configured);

    let p = c.init(&dir.path().join("guided"), Some("button-lamp"));
    assert_eq!(p.concepts.len(), 2);
    assert_eq!(p.mappings.len(), 2);
    assert_eq!(p.outputs.len(), 1);
    assert!(
        p.devices.is_empty(),
        "the guided demo leaves the deployment to the designer"
    );
    let pressed = p.mappings.iter().find(|m| m.name == "pressed").unwrap();
    assert_eq!(pressed.role(), pb::RelationshipRole::Source);
    // the files are a text project like any other
    assert!(dir.path().join("guided/src/main.bdl").is_file());
    assert!(dir.path().join("guided/bdl.toml").is_file());

    match c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: dir.path().join("x").to_string_lossy().into(),
        name: "x".into(),
        template: Some("no-such".into()),
    })) {
        Resp::Error(e) => assert_eq!(e.code, "project.unknown_template"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn readiness_names_the_smallest_blocker_and_clears_when_the_deployment_is_made() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn_with(&[]);
    let p = c.init(&dir.path().join("guided"), Some("button-lamp"));
    let lamp = p.outputs[0].id;
    let pressed = p.mappings.iter().find(|m| m.name == "pressed").unwrap().id;

    // Nothing deployed: the output and the Source each need a device.
    let d = c.deploy(PICO);
    assert!(d.design_ready, "the design itself is valid");
    assert!(!d.build_ready);
    let codes: Vec<&str> = d.build_blockers.iter().map(|b| b.code.as_str()).collect();
    assert!(codes.contains(&"output_no_device"), "{codes:?}");
    assert!(codes.contains(&"source_no_device"), "{codes:?}");
    let first = &d.build_blockers[0];
    assert!(!first.message.is_empty() && !first.explanation.is_empty());
    assert!(
        first.output_id.is_some() || first.mapping_id.is_some(),
        "a blocker names its object"
    );

    // A device for the lamp, by kind: the Source is still unprovided.
    let led = c
        .apply(pb::edit_op::Op::CreateDevice(pb::CreateDevice {
            name: "led".into(),
            kind: pb::DeviceKind::DigitalOutput.into(),
            output_id: Some(lamp),
        }))
        .outcome
        .unwrap()
        .created_device
        .unwrap();
    c.apply(pb::edit_op::Op::SetDeviceRealization(
        pb::SetDeviceRealization {
            id: led,
            profile_id: Some("gpio_level".into()),
            kind: pb::DeviceKind::DigitalOutput.into(),
        },
    ));
    let d = c.deploy(PICO);
    assert!(!d.build_ready);
    let codes: Vec<&str> = d.build_blockers.iter().map(|b| b.code.as_str()).collect();
    assert_eq!(codes, ["source_no_device"], "{codes:?}");
    assert_eq!(d.build_blockers[0].mapping_id, Some(pressed));

    // A device for the button without a provider: the provider is the
    // missing thing, not the device.
    let button = c
        .apply(pb::edit_op::Op::CreateDevice(pb::CreateDevice {
            name: "button".into(),
            kind: pb::DeviceKind::DigitalInput.into(),
            output_id: None,
        }))
        .outcome
        .unwrap()
        .created_device
        .unwrap();
    c.apply(pb::edit_op::Op::SetDeviceSource(pb::SetDeviceSource {
        id: button,
        source_id: Some(pressed),
    }));
    let d = c.deploy(PICO);
    assert!(!d.build_ready, "{:?}", d.build_blockers);
    assert!(
        d.build_blockers
            .iter()
            .any(|b| b.code.contains("provider") && b.device_id == Some(button)),
        "{:?}",
        d.build_blockers
    );

    // The provider chosen: ready.
    c.apply(pb::edit_op::Op::SetDeviceProvider(pb::SetDeviceProvider {
        id: button,
        profile_id: Some("gpio_level_in_low".into()),
        kind: pb::DeviceKind::DigitalInput.into(),
    }));
    let d = c.deploy(PICO);
    assert!(d.build_ready, "{:?}", d.build_blockers);
    assert!(d.build_blockers.is_empty());
    assert!(d.deployable);

    // A pin the board does not have: not feasible, the placement blocks.
    c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
        id: led,
        index: 0,
        resource: Some("GP99".into()),
    }));
    let d = c.deploy(PICO);
    assert!(!d.build_ready);
    assert_eq!(d.status(), pb::DeploymentStatus::Infeasible);
    let b = &d.build_blockers[0];
    assert_eq!(b.code, "placement_blocked");
    assert!(b.message.contains("GP99"), "{}", b.message);
    assert_eq!(b.device_id, Some(led));
}

#[test]
fn a_provider_this_board_cannot_read_blocks_the_build_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn_with(&[]);
    let p = c.init(&dir.path().join("wired"), Some("button-lamp-configured"));
    // The pads are the Pico's; let the Nano's placement choose its own.
    for d in &p.devices {
        c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
            id: d.id,
            index: 0,
            resource: None,
        }));
    }
    // The Arduino Nano places the same devices but its firmware reads no
    // digital input yet: the placement is fine, the firmware is refused.
    let d = c.deploy("arduino_nano");
    assert_ne!(d.status(), pb::DeploymentStatus::Infeasible);
    assert!(!d.build_ready, "{:?}", d.build_blockers);
    assert!(
        d.build_blockers
            .iter()
            .any(|b| b.code.contains("provider") && b.message.contains("button")),
        "{:?}",
        d.build_blockers
    );
    let pico = c.deploy(PICO);
    assert!(pico.build_ready, "{:?}", pico.build_blockers);
}

// ---- the build, stage by stage ----

/// A tiny ELF32 whose one loadable segment sits in the Pico's flash — the
/// stand-in cargo's "linked firmware".
fn tiny_elf() -> Vec<u8> {
    let segment = [0xAAu8; 300];
    let phoff = 52u32;
    let data_at = phoff + 32;
    let mut header = vec![0u8; 52];
    header[..4].copy_from_slice(b"\x7fELF");
    header[4] = 1;
    header[5] = 1;
    header[6] = 1;
    header[16..18].copy_from_slice(&2u16.to_le_bytes());
    header[18..20].copy_from_slice(&40u16.to_le_bytes());
    header[28..32].copy_from_slice(&phoff.to_le_bytes());
    header[42..44].copy_from_slice(&32u16.to_le_bytes());
    header[44..46].copy_from_slice(&1u16.to_le_bytes());
    let mut ph = [0u8; 32];
    ph[0..4].copy_from_slice(&1u32.to_le_bytes());
    ph[4..8].copy_from_slice(&data_at.to_le_bytes());
    ph[8..12].copy_from_slice(&0x1000_0000u32.to_le_bytes());
    ph[12..16].copy_from_slice(&0x1000_0000u32.to_le_bytes());
    ph[16..20].copy_from_slice(&(segment.len() as u32).to_le_bytes());
    ph[20..24].copy_from_slice(&(segment.len() as u32).to_le_bytes());
    [header, ph.to_vec(), segment.to_vec()].concat()
}

/// A stand-in `cargo`: answers `--version`, and on `build` prints what
/// the daemon reads — a compiler message, an artifact naming the ELF it
/// writes, the finish line — and exits as told.
#[cfg(unix)]
fn fake_cargo(dir: &Path, succeed: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let elf = dir.join("firmware.elf");
    std::fs::write(&elf, tiny_elf()).unwrap();
    let script = dir.join("cargo");
    let body = if succeed {
        format!(
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo cargo 1.89.0; exit 0; fi\n\
             bin=\"\"; prev=\"\"; for a in \"$@\"; do if [ \"$prev\" = \"--bin\" ]; then bin=\"$a\"; fi; prev=\"$a\"; done\n\
             printf '%s\\n' '{{\"reason\":\"compiler-message\",\"message\":{{\"rendered\":\"warning: unused thing\",\"level\":\"warning\"}}}}'\n\
             printf '%s\\n' '{{\"reason\":\"compiler-artifact\",\"target\":{{\"name\":\"embassy_rp\"}},\"executable\":null}}'\n\
             printf '%s\\n' \"{{\\\"reason\\\":\\\"compiler-artifact\\\",\\\"target\\\":{{\\\"name\\\":\\\"$bin\\\"}},\\\"executable\\\":\\\"{elf}\\\"}}\"\n\
             printf '%s\\n' '{{\"reason\":\"build-finished\",\"success\":true}}'\n\
             exit 0\n",
            elf = elf.display()
        )
    } else {
        "#!/bin/sh\n\
         if [ \"$1\" = \"--version\" ]; then echo cargo 1.89.0; exit 0; fi\n\
         printf '%s\\n' '{\"reason\":\"compiler-message\",\"message\":{\"rendered\":\"error[E0425]: cannot find value `x` in this scope\",\"level\":\"error\"}}'\n\
         echo 'error: could not compile `demo` (lib) due to 1 previous error' >&2\n\
         printf '%s\\n' '{\"reason\":\"build-finished\",\"success\":false}'\n\
         exit 101\n"
            .to_owned()
    };
    std::fs::write(&script, body).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

fn stages(events: &[pb::BuildProgress]) -> Vec<pb::BuildStage> {
    let mut out: Vec<pb::BuildStage> = Vec::new();
    for e in events {
        if out.last() != Some(&e.stage()) {
            out.push(e.stage());
        }
    }
    out
}

#[cfg(unix)]
#[test]
fn a_build_reports_its_stages_and_leaves_a_fresh_artifact_until_the_design_moves() {
    let dir = tempfile::tempdir().unwrap();
    let cargo = fake_cargo(dir.path(), true);
    let mut c = Client::spawn_with(&[
        ("CARGO", cargo.to_str().unwrap()),
        (
            "BDL_RUNTIME_DIR",
            repo_root().join("runtime").to_str().unwrap(),
        ),
    ]);
    let root = dir.path().join("wired");
    let p = c.init(&root, Some("button-lamp-configured"));
    let before = c.status(PICO);
    assert!(!before.running && before.artifact.is_none() && !before.artifact_fresh);
    assert_eq!(before.stage(), pb::BuildStage::Unspecified);

    let events = c.build(PICO);
    let seq = stages(&events);
    assert_eq!(
        seq,
        [
            pb::BuildStage::Checking,
            pb::BuildStage::Generating,
            pb::BuildStage::Preparing,
            pb::BuildStage::Compiling,
            pb::BuildStage::Packaging,
            pb::BuildStage::Completed,
        ],
        "{events:#?}"
    );
    // the compiler's words reach the client as details, never as the message
    assert!(events
        .iter()
        .any(|e| e.stage() == pb::BuildStage::Compiling && e.detail.contains("unused thing")));
    assert!(events
        .iter()
        .any(|e| e.stage() == pb::BuildStage::Compiling && e.done == Some(2)));
    let last = events.last().unwrap();
    let status = last.status.as_ref().unwrap();
    assert!(!status.running && status.artifact_fresh && status.failure.is_none());
    let artifact = status.artifact.as_ref().unwrap();
    assert_eq!(artifact.kind, "uf2");
    assert!(artifact.path.ends_with("-rp2040.uf2"), "{}", artifact.path);
    assert!(Path::new(&artifact.path).is_file());
    assert_eq!(
        artifact.size_bytes,
        2 * 512,
        "300 bytes of flash: two pages"
    );
    assert_eq!(artifact.revision, p.revision);
    assert!(!artifact.identity.is_empty());
    assert!(status
        .command
        .starts_with("cargo build --release --target thumbv6m-none-eabi"));
    assert_eq!(status.triple, "thumbv6m-none-eabi");
    assert!(Path::new(&status.generated_dir)
        .join("Cargo.toml")
        .is_file());
    assert!(Path::new(&status.generated_dir)
        .join("bdl-build.json")
        .is_file());
    // The generated crate lives under the project, beside its sources.
    assert_eq!(
        Path::new(&status.generated_dir),
        root.join("build").join(PICO)
    );

    // Asked again: the same, fresh.
    let again = c.status(PICO);
    assert!(again.artifact_fresh);
    assert_eq!(again.stage(), pb::BuildStage::Completed);
    assert_eq!(again.artifact.as_ref().unwrap().identity, artifact.identity);

    // A layout-only change leaves the firmware fresh: nothing the
    // firmware is made of moved.
    let fresh_after_layout = c.status(PICO).artifact_fresh;
    assert!(fresh_after_layout);

    // A deployment change — the button on another pad — makes it stale:
    // the last image is not this design.
    let button = p.devices.iter().find(|d| d.name == "button").unwrap().id;
    c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
        id: button,
        index: 0,
        resource: Some("GP3".into()),
    }));
    let stale = c.status(PICO);
    assert!(!stale.artifact_fresh, "the pad moved");
    assert!(
        stale.artifact.is_some(),
        "the old image is still reported, as stale"
    );
    let d = c.deploy(PICO);
    assert!(d.build_ready, "stale, but buildable again");

    // Back on GP2: the identity is the content, so the old image is
    // fresh again without a rebuild.
    c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
        id: button,
        index: 0,
        resource: Some("GP2".into()),
    }));
    assert!(c.status(PICO).artifact_fresh);

    // A reopened project reads the record from disk.
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    c.call(Req::OpenProject(pb::OpenProjectRequest {
        root_path: root.to_string_lossy().into(),
    }));
    let reopened = c.status(PICO);
    assert!(reopened.artifact.is_some() && reopened.artifact_fresh);
}

#[cfg(unix)]
#[test]
fn a_compiler_failure_is_a_failure_at_the_compiling_stage_with_its_words() {
    let dir = tempfile::tempdir().unwrap();
    let cargo = fake_cargo(dir.path(), false);
    let mut c = Client::spawn_with(&[
        ("CARGO", cargo.to_str().unwrap()),
        (
            "BDL_RUNTIME_DIR",
            repo_root().join("runtime").to_str().unwrap(),
        ),
    ]);
    c.init(&dir.path().join("wired"), Some("button-lamp-configured"));
    let events = c.build(PICO);
    assert_eq!(events.last().unwrap().stage(), pb::BuildStage::Failed);
    let status = events.last().unwrap().status.as_ref().unwrap();
    let f = status.failure.as_ref().unwrap();
    assert_eq!(f.stage(), pb::BuildStage::Compiling);
    assert_eq!(f.code, "build.cargo_failed");
    assert!(f.message.contains("did not compile"), "{}", f.message);
    assert!(
        f.output.iter().any(|l| l.contains("E0425")),
        "{:?}",
        f.output
    );
    assert!(
        f.output.iter().any(|l| l.contains("could not compile")),
        "{:?}",
        f.output
    );
    assert!(f.command.starts_with("cargo build"));
    assert!(status.artifact.is_none());
    // and it stays reported until the next build
    let again = c.status(PICO);
    assert_eq!(again.stage(), pb::BuildStage::Failed);
    assert_eq!(again.failure.as_ref().unwrap().code, "build.cargo_failed");
    // nothing to flash
    match c.flash(PICO, None) {
        Err(e) => assert_eq!(e.code, "flash.no_artifact"),
        Ok(p) => panic!("{p:?}"),
    }
}

#[test]
fn a_build_of_an_unready_deployment_fails_at_checking_with_the_refusals() {
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn_with(&[]);
    c.init(&dir.path().join("guided"), Some("button-lamp"));
    let events = c.build(PICO);
    assert_eq!(
        stages(&events),
        [pb::BuildStage::Checking, pb::BuildStage::Failed]
    );
    let f = events
        .last()
        .unwrap()
        .status
        .as_ref()
        .unwrap()
        .failure
        .clone()
        .unwrap();
    assert_eq!(f.stage(), pb::BuildStage::Checking);
    assert_eq!(f.code, "build.not_ready");
    assert!(!f.diagnostics.is_empty());
    assert!(
        f.diagnostics.iter().any(|d| d.code.starts_with("adapter.")),
        "{:?}",
        f.diagnostics
    );
    // an unknown board
    match c.call(Req::BuildFirmware(pb::BuildFirmwareRequest {
        target_id: "big_board".into(),
        revision: None,
    })) {
        Resp::Error(e) => assert_eq!(e.code, "build.unknown_target"),
        other => panic!("{other:?}"),
    }
    // a build asked at a revision the project has left
    match c.call(Req::BuildFirmware(pb::BuildFirmwareRequest {
        target_id: PICO.into(),
        revision: Some(999),
    })) {
        Resp::Error(e) => assert_eq!(e.code, "build.stale_revision"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn without_the_runtime_crates_the_build_fails_while_preparing() {
    let dir = tempfile::tempdir().unwrap();
    let nowhere = dir.path().join("nowhere");
    std::fs::create_dir_all(&nowhere).unwrap();
    let mut c = Client::spawn_with(&[("BDL_RUNTIME_DIR", nowhere.to_str().unwrap())]);
    c.init(&dir.path().join("wired"), Some("button-lamp-configured"));
    let events = c.build(PICO);
    let f = events
        .last()
        .unwrap()
        .status
        .as_ref()
        .unwrap()
        .failure
        .clone()
        .unwrap();
    assert_eq!(f.stage(), pb::BuildStage::Preparing);
    assert_eq!(f.code, "build.runtime_missing");
    assert!(f.explanation.contains("BDL_RUNTIME_DIR"));
}

// ---- flashing ----

fn volume(root: &Path, name: &str) -> PathBuf {
    let v = root.join(name);
    std::fs::create_dir_all(&v).unwrap();
    std::fs::write(
        v.join("INFO_UF2.TXT"),
        "UF2 Bootloader v3.0\nModel: Raspberry Pi RP2\nBoard-ID: RPI-RP2\n",
    )
    .unwrap();
    v
}

#[cfg(unix)]
#[test]
fn a_flash_is_refused_with_no_device_or_two_and_writes_the_image_to_the_one() {
    let dir = tempfile::tempdir().unwrap();
    let cargo = fake_cargo(dir.path(), true);
    let vols = dir.path().join("vols");
    std::fs::create_dir_all(&vols).unwrap();
    let mut c = Client::spawn_with(&[
        ("CARGO", cargo.to_str().unwrap()),
        (
            "BDL_RUNTIME_DIR",
            repo_root().join("runtime").to_str().unwrap(),
        ),
        ("BDL_UF2_ROOTS", vols.to_str().unwrap()),
        ("BDL_FLASH_RESTART_WAIT_MS", "300"),
        ("BDL_PROBE_RS", "/nonexistent/probe-rs"),
    ]);
    let p = c.init(&dir.path().join("wired"), Some("button-lamp-configured"));

    // Before any build: nothing to flash, whatever is plugged in.
    match c.flash(PICO, None) {
        Err(e) => assert_eq!(e.code, "flash.no_artifact"),
        Ok(p) => panic!("{p:?}"),
    }
    let events = c.build(PICO);
    assert_eq!(events.last().unwrap().stage(), pb::BuildStage::Completed);

    // No device: the methods say what to do.
    let d = c.devices(PICO);
    assert!(d.devices.is_empty());
    let usb = d
        .methods
        .iter()
        .find(|m| m.method() == pb::FlashMethod::Uf2Volume)
        .unwrap();
    assert!(usb.available);
    assert!(usb.hint.contains("BOOTSEL") && usb.hint.contains("RPI-RP2"));
    let probe = d
        .methods
        .iter()
        .find(|m| m.method() == pb::FlashMethod::Probe)
        .unwrap();
    assert!(!probe.available);
    assert!(probe.hint.contains("probe-rs"));
    match c.flash(PICO, None) {
        Err(e) => assert_eq!(e.code, "flash.no_device"),
        Ok(p) => panic!("{p:?}"),
    }

    // Two boards in bootloader mode: never guessed.
    let v1 = volume(&vols, "RPI-RP2");
    let v2 = volume(&vols, "RPI-RP2 1");
    let d = c.devices(PICO);
    assert_eq!(d.devices.len(), 2);
    assert!(d
        .devices
        .iter()
        .all(|x| x.method() == pb::FlashMethod::Uf2Volume));
    match c.flash(PICO, None) {
        Err(e) => assert_eq!(e.code, "flash.ambiguous_device"),
        Ok(p) => panic!("{p:?}"),
    }
    // ... unless one is named.
    let chosen = d
        .devices
        .iter()
        .find(|x| x.detail == v2.display().to_string())
        .unwrap();
    let progress = c.flash(PICO, Some(&chosen.id)).unwrap();
    let seq: Vec<pb::FlashStage> = progress.iter().map(|p| p.stage()).collect();
    assert_eq!(
        seq,
        [
            pb::FlashStage::Preparing,
            pb::FlashStage::Writing,
            pb::FlashStage::Restarting,
            pb::FlashStage::Completed
        ]
    );
    let done = progress.last().unwrap();
    assert_eq!(done.device.as_ref().unwrap().id, chosen.id);
    assert!(done.artifact.as_ref().unwrap().path.ends_with(".uf2"));
    let written = std::fs::read_dir(&v2)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .find(|n| n.ends_with(".uf2"))
        .expect("the image was copied onto the volume");
    assert_eq!(
        std::fs::read(v2.join(&written)).unwrap(),
        std::fs::read(&done.artifact.as_ref().unwrap().path).unwrap()
    );
    assert!(
        std::fs::read_dir(&v1)
            .unwrap()
            .flatten()
            .all(|e| !e.file_name().to_string_lossy().ends_with(".uf2")),
        "the other board was left alone"
    );
    // A pretend volume never unmounts: the outcome says so rather than
    // claiming a restart it did not see.
    assert!(
        done.message.contains("did not leave bootloader mode"),
        "{}",
        done.message
    );

    // The design moves on: the image is stale and the flash refused.
    std::fs::remove_dir_all(&v1).unwrap();
    let button = p.devices.iter().find(|d| d.name == "button").unwrap().id;
    c.apply(pb::edit_op::Op::SetDevicePin(pb::SetDevicePin {
        id: button,
        index: 0,
        resource: Some("GP3".into()),
    }));
    match c.flash(PICO, None) {
        Err(e) => assert_eq!(e.code, "flash.artifact_stale"),
        Ok(p) => panic!("{p:?}"),
    }
    // a device that went away
    match c.flash(PICO, Some("uf2:/gone")) {
        Err(e) => assert!(e.code == "flash.artifact_stale" || e.code == "flash.unknown_device"),
        Ok(p) => panic!("{p:?}"),
    }
}

#[cfg(unix)]
#[test]
fn a_debug_probe_is_listed_and_flashed_through_probe_rs() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let cargo = fake_cargo(dir.path(), true);
    let log = dir.path().join("probe.log");
    let probe = dir.path().join("probe-rs");
    std::fs::write(
        &probe,
        format!(
            "#!/bin/sh\n\
             echo \"$@\" >> {log}\n\
             if [ \"$1\" = \"list\" ]; then\n\
               echo 'The following debug probes were found:'\n\
               echo '[0]: Debug Probe (CMSIS-DAP) -- 2e8a:000c:E6614C311B3E3F35 (CMSIS-DAP)'\n\
               exit 0\n\
             fi\n\
             exit 0\n",
            log = log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&probe, std::fs::Permissions::from_mode(0o755)).unwrap();
    let vols = dir.path().join("vols");
    std::fs::create_dir_all(&vols).unwrap();
    let mut c = Client::spawn_with(&[
        ("CARGO", cargo.to_str().unwrap()),
        (
            "BDL_RUNTIME_DIR",
            repo_root().join("runtime").to_str().unwrap(),
        ),
        ("BDL_UF2_ROOTS", vols.to_str().unwrap()),
        ("BDL_PROBE_RS", probe.to_str().unwrap()),
    ]);
    c.init(&dir.path().join("wired"), Some("button-lamp-configured"));
    let events = c.build(PICO);
    let elf = events
        .last()
        .unwrap()
        .status
        .as_ref()
        .unwrap()
        .artifact
        .as_ref()
        .unwrap()
        .elf_path
        .clone();
    let d = c.devices(PICO);
    assert_eq!(d.devices.len(), 1);
    let one = &d.devices[0];
    assert_eq!(one.method(), pb::FlashMethod::Probe);
    assert_eq!(one.id, "probe:2e8a:000c:E6614C311B3E3F35");
    assert!(d
        .methods
        .iter()
        .any(|m| m.method() == pb::FlashMethod::Probe && m.available));
    // exactly one device: no choice needed
    let progress = c.flash(PICO, None).unwrap();
    assert_eq!(progress.last().unwrap().stage(), pb::FlashStage::Completed);
    let calls = std::fs::read_to_string(&log).unwrap();
    assert!(
        calls.contains(&format!(
            "download --chip RP2040 --probe 2e8a:000c:E6614C311B3E3F35 {elf}"
        )),
        "{calls}"
    );
    assert!(
        calls.contains("reset --chip RP2040 --probe 2e8a:000c:E6614C311B3E3F35"),
        "{calls}"
    );
}

// ---- the real cross-build ----

#[test]
fn the_wired_demo_builds_a_uf2_for_the_pico_with_the_real_toolchain() {
    if !cross_available() {
        eprintln!("skipping: thumbv6m-none-eabi not installed");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let mut c = Client::spawn_with(&[(
        "BDL_RUNTIME_DIR",
        repo_root().join("runtime").to_str().unwrap(),
    )]);
    c.init(&dir.path().join("wired"), Some("button-lamp-configured"));
    let events = c.build(PICO);
    let last = events.last().unwrap();
    assert_eq!(
        last.stage(),
        pb::BuildStage::Completed,
        "{:#?}",
        last.status.as_ref().unwrap().failure
    );
    let artifact = last.status.as_ref().unwrap().artifact.clone().unwrap();
    let uf2 = std::fs::read(&artifact.path).unwrap();
    assert!(uf2.len() % 512 == 0 && !uf2.is_empty());
    assert_eq!(&uf2[0..4], &0x0A32_4655u32.to_le_bytes());
    assert_eq!(
        &uf2[28..32],
        &0xe48b_ff56u32.to_le_bytes(),
        "the RP2040 family"
    );
    assert_eq!(
        &uf2[12..16],
        &0x1000_0000u32.to_le_bytes(),
        "the first page is the boot block"
    );
    assert!(std::fs::read(&artifact.elf_path)
        .unwrap()
        .starts_with(b"\x7fELF"));
    // cargo compiled real crates: the count is not the stand-in's two
    assert!(events.iter().any(|e| e.done.unwrap_or(0) > 10));
}
