//! `bdld` — the BDL compiler service.
//!
//! Flutter Studio (or the CLI, or CI) is a client; this process is the
//! semantic source of truth for an opened project.  Stdout is the protocol
//! channel; all logging goes to stderr.

#![forbid(unsafe_code)]

mod cli;
mod rename;
mod server;
mod session;

use clap::{Parser, Subcommand};

/// Version of the compiler implementation, reported in the handshake and
/// recorded in every artifact it writes.
pub const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "bdld", version = COMPILER_VERSION, about = "BDL compiler service")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the protocol over stdin/stdout (length-prefixed protobuf frames).
    Serve,
    /// Print compiler and protocol versions as JSON.
    Version,
    /// Open a project (flat, system or text) and report its diagnostics.
    Check {
        /// The project directory (the one with `bdl.toml`).
        root: std::path::PathBuf,
        /// Machine-readable output.
        #[arg(long)]
        json: bool,
    },
    /// Generate the Rust crate of a project.
    Compile {
        root: std::path::PathBuf,
        /// Where the generated crate goes.
        #[arg(long, default_value = "target/bdl")]
        out: std::path::PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Run the reference evaluator over a project for a number of ticks.
    Simulate {
        root: std::path::PathBuf,
        /// Activations to run.
        #[arg(long, default_value_t = 1)]
        ticks: u64,
        /// A constant input, `relationship=value`; repeatable.
        #[arg(long = "input")]
        inputs: Vec<String>,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    match Cli::parse().command {
        Command::Version => {
            let v = bdl_protocol::PROTOCOL_VERSION;
            println!(
                "{}",
                serde_json::json!({
                    "compiler_version": COMPILER_VERSION,
                    "protocol_version": format!("{}.{}.{}", v.major, v.minor, v.patch),
                    "project_schema_version": bdl_model::PROJECT_SCHEMA_VERSION,
                })
            );
            Ok(())
        }
        Command::Serve => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server::serve_stdio())
        }
        Command::Check { root, json } => exit_with(cli::check(&root, COMPILER_VERSION, json)),
        Command::Compile { root, out, json } => {
            exit_with(cli::compile(&root, COMPILER_VERSION, &out, json))
        }
        Command::Simulate {
            root,
            ticks,
            inputs,
            json,
        } => exit_with(cli::simulate(&root, COMPILER_VERSION, ticks, &inputs, json)),
    }
}

/// A headless command's outcome as the process exit code: 0 checks,
/// 1 the project has errors, 2 it could not be opened.
fn exit_with(result: Result<(), cli::Failure>) -> anyhow::Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(cli::Failure::Errors(_)) => std::process::exit(1),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2)
        }
    }
}
