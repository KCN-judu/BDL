//! `bdld` — the BDL compiler service.
//!
//! Flutter Studio (or the CLI, or CI) is a client; this process is the
//! semantic source of truth for an opened project.  Stdout is the protocol
//! channel; all logging goes to stderr.

#![forbid(unsafe_code)]

mod cli;
mod firmware;
mod formula;
mod rename;
mod server;
mod session;
mod templates;

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
        /// The target has finite memory: a collection the design grows
        /// without bound refuses the artefact instead of being reported.
        #[arg(long)]
        bounded_memory: bool,
        /// The deployment schedule, `domain=period` (1 = every tick);
        /// repeatable.  Decides what each cross-domain window needs.
        #[arg(long = "period")]
        periods: Vec<String>,
        /// Also generate the platform adapter for this board (`rp2040_pico`):
        /// the placement must be feasible and every realization admissible;
        /// the crate gains `src/bin/rp2040.rs` and builds with
        /// `cargo build --release --target thumbv6m-none-eabi --features rp2040`.
        #[arg(long)]
        target: Option<String>,
        /// The firmware's base tick in microseconds (with `--target`).
        #[arg(long, default_value_t = 10_000)]
        tick_micros: u64,
        #[arg(long)]
        json: bool,
    },
    /// Rewrite legacy zero-input signatures (`mapping f : A`) into the
    /// preferred explicit spelling (`mapping f : () -> A`), project-wide
    /// and losslessly: one insertion per signature, comments, spacing and
    /// identities untouched, refused if the design would change.  Opt-in;
    /// nothing else rewrites a source.
    MigrateUnitDomain {
        root: std::path::PathBuf,
        /// Report what would change without writing.
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Rewrite legacy drive declarations (`drive o = m`) into the preferred
    /// spelling (`drive o by m`), project-wide and losslessly: the `=`
    /// becomes `by`, nothing else moves, refused if the design would
    /// change.  Opt-in; nothing else rewrites a source.
    MigrateDriveBy {
        root: std::path::PathBuf,
        /// Report what would change without writing.
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Build a board's firmware: generate the crate under
    /// `<project>/build/<target>/`, compile it with cargo, write the image
    /// the board takes (a UF2 for the Raspberry Pi Pico) and a build
    /// record beside it.  Every stage is printed; a failure names its
    /// stage and code.
    Build {
        root: std::path::PathBuf,
        /// The board (`rp2040_pico`).
        #[arg(long)]
        target: String,
        #[arg(long)]
        json: bool,
    },
    /// Write the last built firmware to the board.  With no device named,
    /// the one reachable device is used; with several, `--list` shows
    /// them and `--device` chooses.  A firmware older than the project as
    /// it is now is refused: build again first.
    Flash {
        root: std::path::PathBuf,
        #[arg(long)]
        target: String,
        /// A device id from `--list`.
        #[arg(long)]
        device: Option<String>,
        /// Only list the reachable devices.
        #[arg(long)]
        list: bool,
        #[arg(long)]
        json: bool,
    },
    /// Create a project, empty or from a template (`--template`, see
    /// `templates`).
    Init {
        root: std::path::PathBuf,
        /// The project's name (the directory's name when absent).
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        template: Option<String>,
    },
    /// List the templates `init --template` accepts.
    Templates,
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
        Command::Compile {
            root,
            out,
            bounded_memory,
            periods,
            target,
            tick_micros,
            json,
        } => {
            let options = cli::CompileCli {
                bounded_memory,
                periods,
                target,
                tick_micros,
            };
            exit_with(cli::compile(&root, COMPILER_VERSION, &out, &options, json))
        }
        Command::Simulate {
            root,
            ticks,
            inputs,
            json,
        } => exit_with(cli::simulate(&root, COMPILER_VERSION, ticks, &inputs, json)),
        Command::Build { root, target, json } => {
            exit_with(cli::build(&root, COMPILER_VERSION, &target, json))
        }
        Command::Flash {
            root,
            target,
            device,
            list,
            json,
        } => exit_with(cli::flash(
            &root,
            COMPILER_VERSION,
            &target,
            device.as_deref(),
            list,
            json,
        )),
        Command::Init {
            root,
            name,
            template,
        } => exit_with(cli::init(
            &root,
            COMPILER_VERSION,
            name.as_deref(),
            template.as_deref(),
        )),
        Command::Templates => {
            for t in templates::templates() {
                println!(
                    "{}\t{}{}\n    {}",
                    t.id,
                    t.display_name,
                    if t.configured {
                        " (deployment included)"
                    } else {
                        ""
                    },
                    t.description
                );
            }
            Ok(())
        }
        Command::MigrateUnitDomain {
            root,
            dry_run,
            json,
        } => exit_with(cli::migrate_unit_domain(&root, dry_run, json)),
        Command::MigrateDriveBy {
            root,
            dry_run,
            json,
        } => exit_with(cli::migrate_drive_by(&root, dry_run, json)),
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
