mod handler;
mod tools;
mod state;

use clap::Parser;
use handler::BinaryAnalysisHandler;
use rust_mcp_sdk::event_store::InMemoryEventStore;
use rust_mcp_sdk::mcp_server::{hyper_server, HyperServerOptions};
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
    LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::{error::SdkResult, mcp_server::ServerHandler};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "binary-analysis-mcp")]
#[command(about = "MCP server for binary file analysis and reverse engineering")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "binary-analysis-server".to_string(),
            version: "0.1.0".to_string(),
            title: Some("Binary Analysis MCP Server".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some("server instructions...".to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    let handler = BinaryAnalysisHandler::new().await;

    let server = hyper_server::create_server(
        server_details,
        handler,
        HyperServerOptions {
            host: "127.0.0.1".to_string(),
            port: args.port,
            ping_interval: Duration::from_secs(5),
            event_store: Some(Arc::new(InMemoryEventStore::default())), 
            ..Default::default()
        },
    );

    server.start().await?;

    Ok(())
}
