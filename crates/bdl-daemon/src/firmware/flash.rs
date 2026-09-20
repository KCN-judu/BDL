//! Getting the built image onto the board.  Two ways, both owned here:
//!
//! * the board's own bootloader as a volume — the Pico held with BOOTSEL
//!   while it is plugged in mounts as `RPI-RP2`; the UF2 is copied onto
//!   it and the board restarts into the new firmware.  No tool, no probe,
//!   no second board: the path the demo takes;
//! * a debug probe over `probe-rs`, when the tool is installed and a probe
//!   is wired to the board's SWD pins: `probe-rs download` then `reset`.
//!
//! Discovery lists every device either way reaches now; a flash names one
//! of them.  With several and none named, nothing is written: a client
//! must choose, never the daemon.

use super::Artifact;
use bdl_codegen_rust::targets::FlashSpec;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Uf2Volume,
    Probe,
}

/// A device a flash can reach now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    /// `uf2:<volume path>` or `probe:<vid:pid:serial>`; opaque to clients.
    pub id: String,
    pub method: Method,
    pub label: String,
    pub detail: String,
}

/// Whether this host can use a method for the target, and what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodView {
    pub method: Method,
    pub available: bool,
    pub label: String,
    pub hint: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Preparing,
    Writing,
    Restarting,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub stage: Stage,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub stage: Stage,
    /// `flash.write_failed`, `flash.device_gone`, `flash.tool_failed`,
    /// `flash.unsupported`.
    pub code: &'static str,
    pub message: String,
    pub explanation: String,
    pub command: String,
    pub output: Vec<String>,
}

/// The volume roots a bootloader drive appears under: `BDL_UF2_ROOTS`
/// (a `PATH`-style list, for tests and unusual hosts), else the
/// platform's mount points.
pub fn uf2_roots() -> Vec<PathBuf> {
    if let Some(list) = std::env::var_os("BDL_UF2_ROOTS") {
        return std::env::split_paths(&list).collect();
    }
    let mut roots = Vec::new();
    if cfg!(target_os = "macos") {
        roots.push(PathBuf::from("/Volumes"));
    } else if cfg!(target_os = "windows") {
        for letter in b'D'..=b'Z' {
            roots.push(PathBuf::from(format!("{}:\\", letter as char)));
        }
    } else {
        let user = std::env::var("USER").unwrap_or_default();
        roots.push(PathBuf::from("/media").join(&user));
        roots.push(PathBuf::from("/run/media").join(&user));
        roots.push(PathBuf::from("/media"));
        roots.push(PathBuf::from("/mnt"));
    }
    roots
}

/// The bootloader volumes of `label` under the roots: a directory whose
/// `INFO_UF2.TXT` names the board.
pub fn uf2_volumes(label: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for root in uf2_roots() {
        // A root that is itself the volume (a drive letter on Windows).
        if is_uf2_volume(&root, label) {
            found.push(root);
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if is_uf2_volume(&p, label) {
                found.push(p);
            }
        }
    }
    found.sort();
    found
}

fn is_uf2_volume(dir: &Path, label: &str) -> bool {
    std::fs::read_to_string(dir.join("INFO_UF2.TXT"))
        .map(|t| t.contains(label))
        .unwrap_or(false)
}

/// The `probe-rs` to run: `BDL_PROBE_RS`, else the one on `PATH`, else
/// cargo's bin directory.
pub fn probe_rs_binary() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("BDL_PROBE_RS") {
        let p = PathBuf::from(p);
        return p.is_file().then_some(p);
    }
    if let Some(p) = super::find_on_path("probe-rs") {
        return Some(p);
    }
    let p = super::home_dir()?
        .join(".cargo")
        .join("bin")
        .join(format!("probe-rs{}", std::env::consts::EXE_SUFFIX));
    p.is_file().then_some(p)
}

/// The probes `probe-rs list` reports: one line each,
/// `[0]: Debug Probe (CMSIS-DAP) -- 2e8a:000c:E66… (CMSIS-DAP)`.
pub fn parse_probe_list(text: &str) -> Vec<Device> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix('[') else {
            continue;
        };
        let Some((_, rest)) = rest.split_once("]:") else {
            continue;
        };
        let Some((name, ident)) = rest.split_once(" -- ") else {
            continue;
        };
        let ident = ident.trim();
        let (selector, kind) = match ident.split_once(' ') {
            Some((s, k)) => (s.trim(), k.trim().trim_matches(|c| c == '(' || c == ')')),
            None => (ident, ""),
        };
        out.push(Device {
            id: format!("probe:{selector}"),
            method: Method::Probe,
            label: name.trim().to_owned(),
            detail: if kind.is_empty() {
                selector.to_owned()
            } else {
                format!("{selector} ({kind})")
            },
        });
    }
    out
}

/// Every device a flash of a target could reach now, and the methods.
pub fn discover(spec: &FlashSpec) -> (Vec<Device>, Vec<MethodView>) {
    let mut devices = Vec::new();
    let mut methods = Vec::new();
    match spec.uf2 {
        Some(fam) => {
            for v in uf2_volumes(fam.volume_label) {
                devices.push(Device {
                    id: format!("uf2:{}", v.display()),
                    method: Method::Uf2Volume,
                    label: format!("Raspberry Pi Pico in BOOTSEL mode ({})", fam.volume_label),
                    detail: v.display().to_string(),
                });
            }
            methods.push(MethodView {
                method: Method::Uf2Volume,
                available: true,
                label: "USB, the board's own bootloader".to_owned(),
                hint: format!(
                    "Hold BOOTSEL while plugging the board in over USB; it appears as a drive named {}.",
                    fam.volume_label
                ),
            });
        }
        None => methods.push(MethodView {
            method: Method::Uf2Volume,
            available: false,
            label: "USB, the board's own bootloader".to_owned(),
            hint: "This board's bootloader does not take an image over USB.".to_owned(),
        }),
    }
    match (spec.probe_chip, probe_rs_binary()) {
        (Some(_), Some(bin)) => {
            let out = Command::new(&bin).arg("list").output();
            if let Ok(o) = out {
                let text = String::from_utf8_lossy(&o.stdout);
                devices.extend(parse_probe_list(&text));
            }
            methods.push(MethodView {
                method: Method::Probe,
                available: true,
                label: "A debug probe (probe-rs)".to_owned(),
                hint: "Connect a debug probe to the board's SWD pins.".to_owned(),
            });
        }
        (Some(_), None) => methods.push(MethodView {
            method: Method::Probe,
            available: false,
            label: "A debug probe (probe-rs)".to_owned(),
            hint: "probe-rs is not installed; the USB bootloader needs no tool.".to_owned(),
        }),
        (None, _) => methods.push(MethodView {
            method: Method::Probe,
            available: false,
            label: "A debug probe (probe-rs)".to_owned(),
            hint: "No probe can program this board yet.".to_owned(),
        }),
    }
    (devices, methods)
}

/// Write the artifact to one device.  Progress hears each stage.
pub fn flash(
    device: &Device,
    artifact: &Artifact,
    spec: &FlashSpec,
    progress: &mut dyn FnMut(Progress),
) -> Result<(), Failure> {
    match device.method {
        Method::Uf2Volume => flash_uf2(device, artifact, progress),
        Method::Probe => flash_probe(device, artifact, spec, progress),
    }
}

/// How long the bootloader volume is given to disappear after the copy
/// (the board restarts and unmounts itself); `BDL_FLASH_RESTART_WAIT_MS`
/// shortens it for a test's pretend volume.
fn restart_wait() -> Duration {
    std::env::var("BDL_FLASH_RESTART_WAIT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_secs(10))
}

fn flash_uf2(
    device: &Device,
    artifact: &Artifact,
    progress: &mut dyn FnMut(Progress),
) -> Result<(), Failure> {
    let volume = PathBuf::from(device.id.trim_start_matches("uf2:"));
    progress(Progress {
        stage: Stage::Preparing,
        message: format!("Checking {}", volume.display()),
    });
    if artifact.kind != "uf2" {
        return Err(Failure {
            stage: Stage::Preparing,
            code: "flash.unsupported",
            message: "The built firmware is not an image the bootloader takes.".into(),
            explanation: "Build again; the image for the board is written beside the firmware."
                .into(),
            command: String::new(),
            output: Vec::new(),
        });
    }
    if !volume.join("INFO_UF2.TXT").is_file() {
        return Err(Failure {
            stage: Stage::Preparing,
            code: "flash.device_gone",
            message: "The board is no longer in bootloader mode.".into(),
            explanation: "Hold BOOTSEL while plugging it in again, then flash.".into(),
            command: String::new(),
            output: Vec::new(),
        });
    }
    let name = artifact
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "firmware.uf2".into());
    let dest = volume.join(&name);
    progress(Progress {
        stage: Stage::Writing,
        message: format!("Writing {name} ({} bytes)", artifact.size_bytes),
    });
    let bytes = std::fs::read(&artifact.path).map_err(|e| Failure {
        stage: Stage::Writing,
        code: "flash.write_failed",
        message: "The image could not be read.".into(),
        explanation: e.to_string(),
        command: String::new(),
        output: Vec::new(),
    })?;
    // One write; the bootloader restarts the board as soon as the last
    // block lands, which can fail the close — that is the success path.
    let written = (|| -> std::io::Result<()> {
        use std::io::Write;
        let mut f = std::fs::File::create(&dest)?;
        f.write_all(&bytes)?;
        let _ = f.sync_all();
        Ok(())
    })();
    if let Err(e) = written {
        if volume.join("INFO_UF2.TXT").is_file() {
            return Err(Failure {
                stage: Stage::Writing,
                code: "flash.write_failed",
                message: format!("The image could not be written to {}.", volume.display()),
                explanation: e.to_string(),
                command: String::new(),
                output: Vec::new(),
            });
        }
    }
    progress(Progress {
        stage: Stage::Restarting,
        message: "Waiting for the board to restart".into(),
    });
    let start = Instant::now();
    let mut restarted = false;
    let wait = restart_wait();
    while start.elapsed() < wait {
        if !volume.join("INFO_UF2.TXT").is_file() {
            restarted = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    progress(Progress {
        stage: Stage::Completed,
        message: if restarted {
            "The board restarted into the new firmware.".into()
        } else {
            "The image was written; the board did not leave bootloader mode by itself — unplug it and plug it in again.".into()
        },
    });
    Ok(())
}

fn flash_probe(
    device: &Device,
    artifact: &Artifact,
    spec: &FlashSpec,
    progress: &mut dyn FnMut(Progress),
) -> Result<(), Failure> {
    let Some(chip) = spec.probe_chip else {
        return Err(Failure {
            stage: Stage::Preparing,
            code: "flash.unsupported",
            message: "No probe can program this board yet.".into(),
            explanation: String::new(),
            command: String::new(),
            output: Vec::new(),
        });
    };
    let Some(bin) = probe_rs_binary() else {
        return Err(Failure {
            stage: Stage::Preparing,
            code: "flash.tool_failed",
            message: "probe-rs is not installed.".into(),
            explanation: "Install probe-rs, or flash over the USB bootloader.".into(),
            command: String::new(),
            output: Vec::new(),
        });
    };
    let selector = device.id.trim_start_matches("probe:").to_owned();
    progress(Progress {
        stage: Stage::Preparing,
        message: format!("Using {}", device.label),
    });
    let run = |args: &[&str], stage: Stage| -> Result<(), Failure> {
        let command = format!("probe-rs {}", args.join(" "));
        let out = Command::new(&bin)
            .args(args)
            .output()
            .map_err(|e| Failure {
                stage,
                code: "flash.tool_failed",
                message: "probe-rs could not be started.".into(),
                explanation: e.to_string(),
                command: command.clone(),
                output: Vec::new(),
            })?;
        if out.status.success() {
            Ok(())
        } else {
            let mut output: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_owned)
                .collect();
            output.extend(
                String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .map(str::to_owned),
            );
            Err(Failure {
                stage,
                code: "flash.tool_failed",
                message: format!("probe-rs could not program the board through {}.", device.label),
                explanation: "The probe's output is in the details; check the SWD wiring and that the board is powered.".into(),
                command,
                output,
            })
        }
    };
    progress(Progress {
        stage: Stage::Writing,
        message: format!("Writing {}", artifact.elf_path.display()),
    });
    let elf = artifact.elf_path.display().to_string();
    run(
        &["download", "--chip", chip, "--probe", &selector, &elf],
        Stage::Writing,
    )?;
    progress(Progress {
        stage: Stage::Restarting,
        message: "Restarting the board".into(),
    });
    run(
        &["reset", "--chip", chip, "--probe", &selector],
        Stage::Restarting,
    )?;
    progress(Progress {
        stage: Stage::Completed,
        message: "The board restarted into the new firmware.".into(),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_list_lines_become_devices() {
        let text = "The following debug probes were found:\n\
                    [0]: Debug Probe (CMSIS-DAP) -- 2e8a:000c:E6614C311B3E3F35 (CMSIS-DAP)\n\
                    [1]: J-Link -- 1366:0101:000123456789 (J-Link)\n\
                    not a probe line\n";
        let d = parse_probe_list(text);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].id, "probe:2e8a:000c:E6614C311B3E3F35");
        assert_eq!(d[0].label, "Debug Probe (CMSIS-DAP)");
        assert_eq!(d[0].detail, "2e8a:000c:E6614C311B3E3F35 (CMSIS-DAP)");
        assert_eq!(d[0].method, Method::Probe);
        assert_eq!(d[1].id, "probe:1366:0101:000123456789");
        assert!(parse_probe_list("No debug probes were found.\n").is_empty());
    }
}
