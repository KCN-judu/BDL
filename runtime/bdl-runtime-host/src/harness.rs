//! Driving `cargo` over a generated crate: write it, check it, build and
//! run its `host` binary with a request.  Used by the differential tests;
//! usable by tooling.  Every failure is a value, never a panic.

use crate::{RunRequest, RunTrace};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum HarnessError {
    Io(std::io::Error),
    /// `cargo` exited non-zero; the captured stderr.
    Cargo {
        command: String,
        stderr: String,
    },
    /// The host binary wrote something that is not a trace.
    Trace(String),
}

impl std::fmt::Display for HarnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HarnessError::Io(e) => write!(f, "io: {e}"),
            HarnessError::Cargo { command, stderr } => write!(f, "`{command}` failed:\n{stderr}"),
            HarnessError::Trace(s) => write!(f, "malformed trace: {s}"),
        }
    }
}

impl From<std::io::Error> for HarnessError {
    fn from(e: std::io::Error) -> Self {
        HarnessError::Io(e)
    }
}

/// Write `files` (relative path → contents) under `dir`, creating
/// directories; existing files with identical contents are left untouched
/// so cargo's fingerprints stay valid.
pub fn write_crate(dir: &Path, files: &BTreeMap<String, String>) -> Result<(), HarnessError> {
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

/// A cargo invocation in `dir` with its own target directory (so it never
/// contends with the workspace build that may be running the tests).
pub struct Cargo {
    pub crate_dir: PathBuf,
    pub target_dir: PathBuf,
}

impl Cargo {
    pub fn new(crate_dir: impl Into<PathBuf>, target_dir: impl Into<PathBuf>) -> Cargo {
        Cargo {
            crate_dir: crate_dir.into(),
            target_dir: target_dir.into(),
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        // `+<toolchain>` is rustup's, not cargo's: route through `rustup run`
        let mut c = match args.first().and_then(|a| a.strip_prefix('+')) {
            Some(toolchain) => {
                let mut c = Command::new("rustup");
                c.args(["run", toolchain, "cargo"]).args(&args[1..]);
                c
            }
            None => {
                let mut c = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
                c.args(args);
                c
            }
        };
        c.current_dir(&self.crate_dir)
            .env("CARGO_TARGET_DIR", &self.target_dir)
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS");
        c
    }

    fn run_ok(&self, args: &[&str]) -> Result<(), HarnessError> {
        let out = self.command(args).output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(HarnessError::Cargo {
                command: format!("cargo {}", args.join(" ")),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        }
    }

    /// `cargo check` of the core alone (no features): the `no_std` crate
    /// must compile by itself.
    pub fn check_core(&self) -> Result<(), HarnessError> {
        self.run_ok(&["check", "--quiet", "--lib"])
    }

    /// The package name from the crate's `Cargo.toml` (`name = "…"` in
    /// `[package]`), which also names its host binary `<package>-host`.
    pub fn package_name(&self) -> Result<String, HarnessError> {
        let manifest = std::fs::read_to_string(self.crate_dir.join("Cargo.toml"))?;
        manifest
            .lines()
            .map(str::trim)
            .find_map(|l| {
                l.strip_prefix("name = \"")
                    .and_then(|r| r.strip_suffix('"'))
            })
            .map(str::to_owned)
            .ok_or_else(|| HarnessError::Trace("Cargo.toml has no package name".into()))
    }

    /// Build the `<package>-host` binary.
    pub fn build_host(&self) -> Result<PathBuf, HarnessError> {
        let bin = format!("{}-host", self.package_name()?);
        self.run_ok(&["build", "--quiet", "--features", "host", "--bin", &bin])?;
        let suffix = std::env::consts::EXE_SUFFIX;
        Ok(self.target_dir.join("debug").join(format!("{bin}{suffix}")))
    }

    /// Cross-build the firmware `<package>-<target>` (feature `<target>`,
    /// release profile) for `triple`; the path of the ELF.
    pub fn build_firmware(&self, target: &str, triple: &str) -> Result<PathBuf, HarnessError> {
        self.build_firmware_with(target, triple, None, &[])
    }

    /// [`build_firmware`](Self::build_firmware) on another toolchain
    /// (`cargo +<toolchain>`) with extra cargo arguments (`-Zbuild-std=core`
    /// for a tier-3 target); the ELF's path.
    pub fn build_firmware_with(
        &self,
        target: &str,
        triple: &str,
        toolchain: Option<&str>,
        extra: &[&str],
    ) -> Result<PathBuf, HarnessError> {
        let bin = format!("{}-{target}", self.package_name()?);
        let plus = toolchain.map(|t| format!("+{t}"));
        let mut args: Vec<&str> = Vec::new();
        if let Some(p) = &plus {
            args.push(p);
        }
        args.extend([
            "build",
            "--quiet",
            "--release",
            "--target",
            triple,
            "--features",
            target,
            "--bin",
            &bin,
        ]);
        args.extend_from_slice(extra);
        self.run_ok(&args)?;
        // cargo names an AVR binary `<bin>.elf`
        let dir = self.target_dir.join(triple).join("release");
        let plain = dir.join(&bin);
        if plain.is_file() {
            Ok(plain)
        } else {
            Ok(dir.join(format!("{bin}.elf")))
        }
    }

    /// Build if needed and run one request through the host binary.
    pub fn run_host(&self, req: &RunRequest) -> Result<RunTrace, HarnessError> {
        let bin = self.build_host()?;
        let mut child = Command::new(&bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let body = serde_json::to_string(req).map_err(|e| HarnessError::Trace(e.to_string()))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(body.as_bytes())?;
        }
        let out = child.wait_with_output()?;
        if !out.status.success() {
            return Err(HarnessError::Cargo {
                command: bin.display().to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        serde_json::from_slice(&out.stdout).map_err(|e| {
            HarnessError::Trace(format!("{e}: {}", String::from_utf8_lossy(&out.stdout)))
        })
    }
}
