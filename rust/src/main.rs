use clap::Parser;
use omnifocus_mcp::{jxa::RealJxaRunner, server::OmniFocusServer};
use rmcp::{transport::stdio, ServiceExt};

#[derive(Parser, Debug)]
#[command(name = "omnifocus-mcp", version, about = "OmniFocus MCP server")]
struct Cli {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _cli = Cli::parse();

    let server = OmniFocusServer::new(RealJxaRunner::new());
    let service = server.serve(stdio()).await?;
    let cancel_token = service.cancellation_token();
    let waiting = service.waiting();
    tokio::pin!(waiting);

    tokio::select! {
        result = &mut waiting => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            cancel_token.cancel();
            waiting.await?;
        }
    }

    Ok(())
}
