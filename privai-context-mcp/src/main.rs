use clap::{Parser, Subcommand};
use privai_context_mcp::{server::ServerRuntime, Config, Result};

#[derive(Debug, Parser)]
#[command(name = "privai-context-mcp")]
#[command(about = "Read-only V0 context MCP server for privAI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the stdio MCP server. Transport wiring belongs to the MCP implementation pass.
    Serve,
    /// Validate local data manifests and print the registered tool contract.
    Validate,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let config = Config::from_env();
    let runtime = ServerRuntime::new(config);

    match cli.command {
        Command::Serve => runtime.serve_stdio().await,
        Command::Validate => {
            runtime.validate_contract()?;
            Ok(())
        }
    }
}
