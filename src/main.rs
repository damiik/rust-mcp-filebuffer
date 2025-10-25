// ============================================================================
// src/main.rs
// ============================================================================
mod state;
mod tools;
mod handler;

use clap::Parser;
use rust_mcp_sdk::{StdioTransport, TransportOptions, mcp_server::server_runtime};
use rust_mcp_sdk::schema::{
    Implementation, InitializeResult, ServerCapabilities,
    ServerCapabilitiesTools, LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::McpServer;
use handler::BinaryAnalysisHandler;

#[derive(Parser)]
#[command(name = "binary-analysis-mcp")]
#[command(about = "MCP server for binary file analysis and reverse engineering")]
struct Args {}

#[tokio::main]
async fn main() {
    let _args = Args::parse();
    
    let transport = StdioTransport::new(TransportOptions::default())
        .expect("Failed to create transport");
    
    let handler = BinaryAnalysisHandler::new();
    
    let server_info = InitializeResult {
        server_info: Implementation {
            name: "binary-analysis-server".to_string(),
            version: "0.1.0".to_string(),
            title: Some("Binary Analysis MCP Server for Reverse Engineering".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            experimental: None,
            logging: None,
            prompts: None,
            resources: None,
            completions: None,
        },
        instructions: Some("Binary analysis server for reverse engineering tasks".to_string()),
        meta: None,
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };
    
    let server = server_runtime::create_server(server_info, transport, handler);
    
    if let Err(e) = server.start().await {
        eprintln!("❌ Server error: {}", e);
    }
}