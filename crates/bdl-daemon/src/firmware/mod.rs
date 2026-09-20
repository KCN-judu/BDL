//! Building a board's firmware from an open project, and flashing it: the
//! daemon owns the whole path, a client watches
//! (docs/architecture/firmware-build.md).  This module is the build;
//! `flash` writes the image to the board, `readiness` says whether a
//! build would pass, `protocol` serves the requests, `uf2` makes the
//! image.
//!
//! ```text
//! run(plan, snapshot)
//!   = Checking    compile_for_target — every refusal the firmware would meet
//!   → Generating  the crate written under <root>/build/<target>/
//!   → Preparing   cargo, the board's Rust target, the runtime crates located
//!   → Compiling   cargo build, its messages read as they come
//!   → Packaging   the image the board takes (a UF2) from the ELF
//!   → Completed   a record on disk beside the crate: the artifact and the
//!                 identity of what it was built from
//! ```
//!
//! Every judgment stays where it was — `compile_for_target` decides what
//! can be built, the target entry says how — and every failure is a
//! value with the stage it happened in, a code, product language and the
//! tool's own words for the advanced view.  Nothing here reads the
//! design; the generated crate is the design's whole contribution, which
//! is why its content hash is the artifact's identity: the same crate,
//! settings and compiler give the same firmware, and anything else makes
//! the last image stale.

pub mod flash;
pub mod protocol;
pub mod readiness;
pub mod uf2;

use bdl_codegen_rust::targets::Entry;
use bdl_codegen_rust::GeneratedCrate;
use bdl_compiler::{CompileArtifact, CompileOptions, MemoryPolicy, TargetOptions};
use bdl_diagnostics::Diagnostic;
use bdl_hardware::Hardware;
use bdl_model::surface::ProjectSnapshot;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Where a build stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Checking,
    Generating,
    Preparing,
    Compiling,
    Packaging,
    Completed,
    Failed,
    Cancelled,
}

/// One step reported while a build runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub stage: Stage,
    /// Product language: "Compiling embassy-rp".
    pub message: String,
    /// Crates compiled so far, while compiling.
    pub done: Option<u32>,
    /// A line for the advanced view; may be empty.
    pub detail: String,
}

/// Why a build did not complete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub stage: Stage,
    /// `build.not_ready`, `build.toolchain_missing`, `build.target_missing`,
    /// `build.runtime_missing`, `build.cargo_failed`, `build.package_failed`,
    /// `build.io`, `build.cancelled`.
    pub code: &'static str,
    pub message: String,
    pub explanation: String,
    /// The compiler's refusals when the deployment is not ready.
    pub diagnostics: Vec<Diagnostic>,
    pub command: String,
    /// What the tool printed (the tail).
    pub output: Vec<String>,
}

impl Failure {
    fn new(
        stage: Stage,
        code: &'static str,
        message: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Failure {
        Failure {
            stage,
            code,
            message: message.into(),
            explanation: explanation.into(),
            diagnostics: Vec::new(),
            command: String::new(),
            output: Vec::new(),
        }
    }
}

/// The firmware built: the image to write, the ELF it came from, and the
/// identity that says what it was built from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub path: PathBuf,
    /// "uf2" or "elf".
    pub kind: String,
    pub elf_path: PathBuf,
    pub identity: String,
    /// Seconds since the Unix epoch.
    pub built_at: u64,
    pub revision: u64,
    pub compiler_version: String,
    pub size_bytes: u64,
}

/// What a completed build leaves on disk beside the crate
/// (`bdl-build.json`), so a reopened project knows its last firmware.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub target_id: String,
    pub package: String,
    pub triple: String,
    pub command: String,
    pub generated_dir: PathBuf,
    pub artifact: Artifact,
}

pub const RECORD_FILE: &str = "bdl-build.json";

/// How many lines of a tool's output a failure or a status keeps.
pub const OUTPUT_TAIL: usize = 200;

/// Everything a build is told; nothing in it comes from the design.
#[derive(Clone, Debug)]
pub struct Plan {
    pub target_id: String,
    pub hardware: Hardware,
    pub entry: Entry,
    pub tick_micros: u64,
    /// The generated crate's directory.
    pub out_dir: PathBuf,
    /// cargo's target directory for it.
    pub target_dir: PathBuf,
    /// The repository's `runtime/` directory the crate depends on, when
    /// one was found.
    pub runtime_dir: Option<PathBuf>,
    pub compiler_version: String,
}

impl Plan {
    /// A plan for a board, or `None` when the board has no target entry
    /// (it is known to the placement but no firmware exists for it).
    pub fn for_target(root: &Path, target_id: &str, compiler_version: &str) -> Option<Plan> {
        let hardware = bdl_hardware::boards::by_name(target_id)?;
        let entry = Entry::for_board(&hardware.family, &hardware.name)?;
        let out_dir = out_dir(root, target_id);
        Some(Plan {
            target_id: target_id.to_owned(),
            hardware,
            entry,
            tick_micros: TargetOptions::default().tick_micros,
            target_dir: out_dir.join("target"),
            out_dir,
            runtime_dir: runtime_dir(),
            compiler_version: compiler_version.to_owned(),
        })
    }

    /// The compiler options the build (and the identity) use: bounded
    /// memory, the runtime crates by absolute path.
    pub fn compile_options(&self) -> CompileOptions {
        let mut codegen = bdl_codegen_rust::CodegenOptions::default();
        if let Some(rt) = &self.runtime_dir {
            let p = |name: &str| rt.join(name).display().to_string();
            codegen.runtime_core_path = p("bdl-runtime-core");
            codegen.runtime_host_path = p("bdl-runtime-host");
            codegen.runtime_adapter_path = p("bdl-runtime-adapter");
            codegen.runtime_embassy_rp_path = p("bdl-runtime-embassy-rp");
            codegen.runtime_arduino_path = p("bdl-runtime-arduino");
        }
        CompileOptions {
            require_complete: true,
            codegen,
            memory: MemoryPolicy::Bounded,
            schedule: None,
        }
    }

    pub fn target_options(&self) -> TargetOptions {
        TargetOptions {
            tick_micros: self.tick_micros,
        }
    }

    /// The compiler's judgment of the project for this board, with the
    /// crate it would generate.
    pub fn compile(&self, snapshot: &ProjectSnapshot) -> CompileArtifact {
        bdl_compiler::compile_for_target(
            snapshot,
            &self.hardware,
            &self.target_options(),
            &self.compile_options(),
        )
    }

    /// The identity of the firmware this project would build now, or
    /// `None` when it cannot be built.
    pub fn current_identity(&self, snapshot: &ProjectSnapshot) -> Option<String> {
        let art = self.compile(snapshot);
        art.generated.as_ref().map(|g| self.identity(g))
    }

    /// The content identity: the generated crate, the settings and the
    /// compiler.  Layout, names of things outside the crate and the time
    /// do not enter it.
    pub fn identity(&self, generated: &GeneratedCrate) -> String {
        let mut h = Sha256::new();
        h.update(b"bdl-firmware-identity/1\n");
        h.update(self.compiler_version.as_bytes());
        h.update(b"\n");
        h.update(self.target_id.as_bytes());
        h.update(b"\n");
        h.update(self.tick_micros.to_string().as_bytes());
        h.update(b"\n");
        h.update(self.entry.triple().as_bytes());
        h.update(b"\n");
        for (path, text) in &generated.files {
            h.update(path.as_bytes());
            h.update(b"\0");
            h.update(text.as_bytes());
            h.update(b"\0");
        }
        let digest = h.finalize();
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn command(&self, package: &str) -> String {
        self.entry.build_command(package)
    }

    /// The record of the last completed build for this target, if any.
    pub fn record(&self) -> Option<Record> {
        let text = std::fs::read_to_string(self.out_dir.join(RECORD_FILE)).ok()?;
        let r: Record = serde_json::from_str(&text).ok()?;
        (r.target_id == self.target_id && r.artifact.path.is_file()).then_some(r)
    }
}

/// `<root>/build/<target>/`: the generated crate, its cargo target
/// directory and the build record.
pub fn out_dir(root: &Path, target_id: &str) -> PathBuf {
    root.join("build").join(target_id)
}

/// The repository's `runtime/` directory: `BDL_RUNTIME_DIR`, else found
/// above the running `bdld`, else above the working directory.
pub fn runtime_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("BDL_RUNTIME_DIR") {
        let p = PathBuf::from(dir);
        return is_runtime_dir(&p).then_some(p);
    }
    let mut starts = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        starts.push(exe);
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    for start in starts {
        for dir in start.ancestors() {
            let candidate = dir.join("runtime");
            if is_runtime_dir(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn is_runtime_dir(p: &Path) -> bool {
    p.join("bdl-runtime-core").join("Cargo.toml").is_file()
}

/// The `cargo` to run: `$CARGO`, else the one on `PATH`, else rustup's
/// in the home directory (Studio is launched without a shell's `PATH`).
pub fn cargo_binary() -> PathBuf {
    if let Ok(c) = std::env::var("CARGO") {
        return PathBuf::from(c);
    }
    if let Some(p) = find_on_path("cargo") {
        return p;
    }
    if let Some(home) = home_dir() {
        let p = home
            .join(".cargo")
            .join("bin")
            .join(format!("cargo{}", std::env::consts::EXE_SUFFIX));
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from("cargo")
}

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn find_on_path(name: &str) -> Option<PathBuf> {
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|d| d.join(&file))
            .find(|p| p.is_file())
    })
}

/// Write `files` under `dir`; a file whose contents are unchanged is left
/// alone so cargo's fingerprints stay valid.
fn write_crate(
    dir: &Path,
    files: &std::collections::BTreeMap<String, String>,
) -> std::io::Result<()> {
    for (rel, contents) in files {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if std::fs::read_to_string(&path)
            .map(|s| s == *contents)
            .unwrap_or(false)
        {
            continue;
        }
        std::fs::write(&path, contents)?;
    }
    Ok(())
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn tail(lines: &[String]) -> Vec<String> {
    let skip = lines.len().saturating_sub(OUTPUT_TAIL);
    lines[skip..].to_vec()
}

/// The whole build, stage by stage.  `progress` hears every stage change
/// and every crate cargo finishes; `cancel` set from another thread ends
/// the build at the next line cargo prints.
pub fn run(
    plan: &Plan,
    snapshot: &ProjectSnapshot,
    cancel: &Arc<AtomicBool>,
    progress: &mut dyn FnMut(Progress),
) -> Result<Record, Box<Failure>> {
    let stage = |s: Stage, m: &str| Progress {
        stage: s,
        message: m.to_owned(),
        done: None,
        detail: String::new(),
    };
    let board = plan.hardware.display();

    // ---- Checking ----
    progress(stage(
        Stage::Checking,
        &format!("Checking the deployment on {board}"),
    ));
    let art = plan.compile(snapshot);
    let Some(generated) = art.generated.as_ref() else {
        let mut diagnostics: Vec<Diagnostic> = art
            .analysis
            .diagnostics
            .iter()
            .filter(|d| d.severity == bdl_diagnostics::Severity::Error)
            .cloned()
            .collect();
        diagnostics.extend(art.diagnostics.iter().cloned());
        let first = diagnostics
            .iter()
            .find(|d| d.severity == bdl_diagnostics::Severity::Error)
            .or(diagnostics.first());
        let mut f = Failure::new(
            Stage::Checking,
            "build.not_ready",
            first
                .map(|d| d.message.clone())
                .unwrap_or_else(|| format!("The design cannot be built for {board} yet.")),
            first
                .map(|d| d.explanation.clone())
                .unwrap_or_else(|| "The Deploy page names what is missing.".to_owned()),
        );
        f.diagnostics = diagnostics;
        return Err(Box::new(f));
    };
    let identity = plan.identity(generated);
    let package = generated.package.clone();
    let binary = plan.entry.binary(&package);
    let command = plan.command(&package);

    // ---- Generating ----
    progress(stage(Stage::Generating, "Writing the generated crate"));
    if let Err(e) = write_crate(&plan.out_dir, &generated.files) {
        return Err(Box::new(Failure::new(
            Stage::Generating,
            "build.io",
            format!(
                "The crate could not be written under {}.",
                plan.out_dir.display()
            ),
            e.to_string(),
        )));
    }

    // ---- Preparing ----
    progress(stage(Stage::Preparing, "Locating the toolchain"));
    let Some(runtime) = &plan.runtime_dir else {
        return Err(Box::new(Failure::new(
            Stage::Preparing,
            "build.runtime_missing",
            "The BDL runtime crates were not found.",
            "The generated crate depends on the `runtime/` directory of the BDL repository. Set BDL_RUNTIME_DIR to it, or run bdld from a checkout.",
        )));
    };
    let cargo = cargo_binary();
    let version = Command::new(&cargo).arg("--version").output();
    match version {
        Ok(o) if o.status.success() => {}
        _ => {
            return Err(Box::new(Failure::new(
                Stage::Preparing,
                "build.toolchain_missing",
                "The Rust toolchain is not installed.",
                "Building firmware needs cargo. Install Rust from https://rustup.rs and open Studio again.",
            )));
        }
    }
    let triple = plan.entry.triple();
    if !target_installed(&cargo, triple, &plan.out_dir) {
        return Err(Box::new(Failure::new(
            Stage::Preparing,
            "build.target_missing",
            format!("The Rust target for {board} is not installed."),
            format!("Run `rustup target add {triple}` once, then build again."),
        )));
    }
    let _ = runtime; // located: the crate's manifest already names it

    // ---- Compiling ----
    progress(stage(
        Stage::Compiling,
        &format!("Compiling the firmware for {board}"),
    ));
    let mut args: Vec<String> = Vec::new();
    let toolchain = plan.entry.toolchain();
    let mut cmd = match toolchain {
        Some(t) => {
            let mut c = Command::new("rustup");
            c.args(["run", t, "cargo"]);
            c
        }
        None => Command::new(&cargo),
    };
    args.extend(
        [
            "build",
            "--release",
            "--target",
            triple,
            "--features",
            &plan.entry.feature(),
            "--bin",
            &binary,
            "--message-format=json-render-diagnostics",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    if toolchain.is_some() {
        args.push("-Zbuild-std=core".into());
    }
    cmd.args(&args)
        .current_dir(&plan.out_dir)
        .env("CARGO_TARGET_DIR", &plan.target_dir)
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let mut f = Failure::new(
                Stage::Compiling,
                "build.toolchain_missing",
                "cargo could not be started.",
                e.to_string(),
            );
            f.command = command;
            return Err(Box::new(f));
        }
    };
    let stderr = child.stderr.take();
    let stderr_lines = std::thread::spawn(move || {
        let mut lines = Vec::new();
        if let Some(err) = stderr {
            for line in BufReader::new(err).lines().map_while(Result::ok) {
                lines.push(line);
            }
        }
        lines
    });
    let mut output: Vec<String> = Vec::new();
    let mut done: u32 = 0;
    let mut executable: Option<PathBuf> = None;
    let mut success: Option<bool> = None;
    if let Some(out) = child.stdout.take() {
        for line in BufReader::new(out).lines().map_while(Result::ok) {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                let mut f = Failure::new(
                    Stage::Cancelled,
                    "build.cancelled",
                    "The build was stopped.",
                    "Nothing was written to the board; build again when ready.",
                );
                f.command = command;
                return Err(Box::new(f));
            }
            let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
                output.push(line);
                continue;
            };
            match msg.get("reason").and_then(|r| r.as_str()) {
                Some("compiler-artifact") => {
                    done += 1;
                    let name = msg
                        .pointer("/target/name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    if let Some(exe) = msg.get("executable").and_then(|e| e.as_str()) {
                        if name == binary {
                            executable = Some(PathBuf::from(exe));
                        }
                    }
                    progress(Progress {
                        stage: Stage::Compiling,
                        message: format!("Compiled {name}"),
                        done: Some(done),
                        detail: String::new(),
                    });
                }
                Some("compiler-message") => {
                    if let Some(r) = msg.pointer("/message/rendered").and_then(|r| r.as_str()) {
                        for l in r.lines() {
                            output.push(l.to_owned());
                        }
                        progress(Progress {
                            stage: Stage::Compiling,
                            message: String::new(),
                            done: Some(done),
                            detail: r.trim_end().to_owned(),
                        });
                    }
                }
                Some("build-finished") => {
                    success = msg.get("success").and_then(|s| s.as_bool());
                }
                _ => {}
            }
        }
    }
    let status = child.wait();
    let err_lines = stderr_lines.join().unwrap_or_default();
    output.extend(err_lines.iter().cloned());
    let ok = status.map(|s| s.success()).unwrap_or(false) && success.unwrap_or(true);
    if !ok {
        let code = if err_lines.iter().any(|l| {
            l.contains("target may not be installed") || l.contains("can't find crate for `core`")
        }) {
            "build.target_missing"
        } else {
            "build.cargo_failed"
        };
        let mut f = Failure::new(
            Stage::Compiling,
            code,
            format!("The firmware for {board} did not compile."),
            if code == "build.target_missing" {
                format!("Run `rustup target add {triple}` once, then build again.")
            } else {
                "The compiler's output is in the details. A generated crate that does not compile is a BDL defect: keep the output and report it.".to_owned()
            },
        );
        f.command = command;
        f.output = tail(&output);
        return Err(Box::new(f));
    }
    let elf = executable.unwrap_or_else(|| {
        let dir = plan.target_dir.join(triple).join("release");
        let plain = dir.join(&binary);
        if plain.is_file() {
            plain
        } else {
            dir.join(format!("{binary}.elf"))
        }
    });

    // ---- Packaging ----
    progress(stage(Stage::Packaging, "Writing the image for the board"));
    let elf_bytes = match std::fs::read(&elf) {
        Ok(b) => b,
        Err(e) => {
            let mut f = Failure::new(
                Stage::Packaging,
                "build.io",
                format!("The linked firmware was not found at {}.", elf.display()),
                e.to_string(),
            );
            f.command = command;
            f.output = tail(&output);
            return Err(Box::new(f));
        }
    };
    let (path, kind) = match plan.entry.flash().uf2 {
        Some(fam) => {
            let family = uf2::Family {
                id: fam.family_id,
                flash_start: fam.flash_start,
                flash_len: fam.flash_len,
            };
            let image = match uf2::from_elf(&elf_bytes, family) {
                Ok(i) => i,
                Err(e) => {
                    let mut f = Failure::new(
                        Stage::Packaging,
                        "build.package_failed",
                        "The image for the board could not be made from the firmware.".to_owned(),
                        e.to_string(),
                    );
                    f.command = command;
                    return Err(Box::new(f));
                }
            };
            let path = plan.out_dir.join(format!("{binary}.uf2"));
            if let Err(e) = std::fs::write(&path, &image) {
                return Err(Box::new(Failure::new(
                    Stage::Packaging,
                    "build.io",
                    format!("The image could not be written to {}.", path.display()),
                    e.to_string(),
                )));
            }
            (path, "uf2")
        }
        None => (elf.clone(), "elf"),
    };
    let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let record = Record {
        target_id: plan.target_id.clone(),
        package,
        triple: triple.to_owned(),
        command,
        generated_dir: plan.out_dir.clone(),
        artifact: Artifact {
            path,
            kind: kind.to_owned(),
            elf_path: elf,
            identity,
            built_at: now_unix(),
            revision: snapshot.revision.raw(),
            compiler_version: plan.compiler_version.clone(),
            size_bytes,
        },
    };
    let text = serde_json::to_string_pretty(&record).unwrap_or_default();
    if let Err(e) = std::fs::write(plan.out_dir.join(RECORD_FILE), text) {
        return Err(Box::new(Failure::new(
            Stage::Packaging,
            "build.io",
            "The build record could not be written.",
            e.to_string(),
        )));
    }
    progress(stage(
        Stage::Completed,
        &format!("Firmware for {board} built"),
    ));
    Ok(record)
}

/// Whether the Rust target is installed for the toolchain `cargo` runs in
/// `dir` (the generated crate's own `rust-toolchain.toml` applies, and
/// rustup installs what it lists on this very call): the sysroot has its
/// `rustlib/<triple>` directory.  Unknown sysroot: assume yes and let
/// cargo say otherwise.
fn target_installed(cargo: &Path, triple: &str, dir: &Path) -> bool {
    let rustc = cargo
        .parent()
        .map(|d| d.join(format!("rustc{}", std::env::consts::EXE_SUFFIX)))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("rustc"));
    let Ok(out) = Command::new(rustc)
        .args(["--print", "sysroot"])
        .current_dir(dir)
        .output()
    else {
        return true;
    };
    if !out.status.success() {
        return true;
    }
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    Path::new(&sysroot)
        .join("lib")
        .join("rustlib")
        .join(triple)
        .is_dir()
}
