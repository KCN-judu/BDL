//! `bdld` — the BDL compiler service.
//!
//! Flutter Studio (or the CLI, or CI) is a client; this process is the
//! semantic source of truth for an opened project.  Stdout is the protocol
//! channel; all logging goes to stderr.

#![forbid(unsafe_code)]

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
    }
}
