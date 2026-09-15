//! `bdl-lsp` over stdio.  Logs go to stderr (stdout is the protocol).

#![forbid(unsafe_code)]

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let (connection, io_threads) = lsp_server::Connection::stdio();
    bdl_lsp::run(connection)?;
    io_threads.join()?;
    Ok(())
}
