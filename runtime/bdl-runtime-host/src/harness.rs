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
        let mut c = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        c.args(args)
            .current_dir(&self.crate_dir)
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
