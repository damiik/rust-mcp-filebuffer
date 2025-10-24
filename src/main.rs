use clap::Parser;
use rust_mcp_sdk::{McpServer, StdioTransport, TransportOptions, mcp_server::server_runtime};
use rust_mcp_sdk::schema::{
    CallToolRequest, CallToolResult, InitializeRequest, InitializeResult,
    ListToolsRequest, ListToolsResult, Implementation, ServerCapabilities,
    ServerCapabilitiesTools, TextContent, LATEST_PROTOCOL_VERSION,
    schema_utils::CallToolError,
};
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool, tool_box};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;

// ============================================================================
// STATE MANAGEMENT
// ============================================================================

#[derive(Clone, Debug)]
struct ServerState {
    buffer: String,
    output: String,
    file_loaded: Option<String>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            buffer: String::new(),
            output: String::new(),
            file_loaded: None,
        }
    }

    fn display(&self) {
        println!("\n{}", "=".repeat(60));
        println!("📊 SERVER STATE");
        println!("{}", "=".repeat(60));
        
        println!("\n📂 Loaded File: {}", 
            self.file_loaded.as_deref().unwrap_or("None"));
        
        println!("\n📝 Buffer Content ({} chars):", self.buffer.len());
        if self.buffer.is_empty() {
            println!("  [Empty]");
        } else {
            let preview = if self.buffer.len() > 200 {
                format!("{}...", &self.buffer[..200])
            } else {
                self.buffer.clone()
            };
            println!("  {}", preview);
        }
        
        println!("\n📤 Output ({} chars):", self.output.len());
        if self.output.is_empty() {
            println!("  [Empty]");
        } else {
            println!("  {}", self.output);
        }
        
        println!("\n{}", "=".repeat(60));
    }
}

// ============================================================================
// TOOLS DEFINITIONS
// ============================================================================

#[mcp_tool(
    name = "load_file",
    title = "Load File to Buffer",
    description = "Reads a text file from disk and loads its content into the buffer",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct LoadFile {
    /// Path to the file to load
    pub path: String,
}

#[mcp_tool(
    name = "get_buffer",
    title = "Get Buffer Content",
    description = "Returns the current content of the buffer for analysis",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetBuffer {}

#[mcp_tool(
    name = "analyze_buffer",
    title = "Analyze Buffer",
    description = "Returns statistics about the buffer content (word count, line count, etc.)",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AnalyzeBuffer {}

#[mcp_tool(
    name = "set_output",
    title = "Set Output Text",
    description = "Sets the output text that will be displayed in the console",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SetOutput {
    /// The text to set as output
    pub text: String,
}

#[mcp_tool(
    name = "append_buffer",
    title = "Append to Buffer",
    description = "Appends text to the current buffer content",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AppendBuffer {
    /// Text to append to the buffer
    pub text: String,
}

#[mcp_tool(
    name = "clear_buffer",
    title = "Clear Buffer",
    description = "Clears the buffer content",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ClearBuffer {}

// Generate the tool box enum
tool_box!(
    BufferTools,
    [LoadFile, GetBuffer, AnalyzeBuffer, SetOutput, AppendBuffer, ClearBuffer]
);

// ============================================================================
// TOOL IMPLEMENTATIONS
// ============================================================================

impl LoadFile {
    async fn run(self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let content = fs::read_to_string(&self.path)
            .await
            .map_err(|e| CallToolError::from_message(
                format!("Failed to read file: {}", e)
            ))?;
        
        let mut s = state.write().await;
        s.buffer = content.clone();
        s.file_loaded = Some(self.path.clone());
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!(
                "✅ Loaded {} bytes from '{}'", 
                content.len(), 
                self.path
            ))
        ]))
    }
}

impl GetBuffer {
    async fn run(self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        Ok(CallToolResult::text_content(vec![
            TextContent::from(s.buffer.clone())
        ]))
    }
}

impl AnalyzeBuffer {
    async fn run(self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        let chars = s.buffer.len();
        let words = s.buffer.split_whitespace().count();
        let lines = s.buffer.lines().count();
        
        let analysis = format!(
            "📊 Buffer Analysis:\n\
             Characters: {}\n\
             Words: {}\n\
             Lines: {}",
            chars, words, lines
        );
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(analysis)
        ]))
    }
}

impl SetOutput {
    async fn run(self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        s.output = self.text.clone();
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from("✅ Output updated")
        ]))
    }
}

impl AppendBuffer {
    async fn run(self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        s.buffer.push_str(&self.text);
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!(
                "✅ Appended {} chars. Buffer now has {} chars",
                self.text.len(),
                s.buffer.len()
            ))
        ]))
    }
}

impl ClearBuffer {
    async fn run(self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        s.buffer.clear();
        s.file_loaded = None;
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from("✅ Buffer cleared")
        ]))
    }
}

// ============================================================================
// SERVER HANDLER
// ============================================================================

struct BufferServerHandler {
    state: Arc<RwLock<ServerState>>,
}

impl BufferServerHandler {
    fn new() -> Self {
        let state = Arc::new(RwLock::new(ServerState::new()));
        
        // Display initial state
        println!("\n🚀 Simple MCP Text Buffer Server Starting...");
        let s = state.blocking_read();
        s.display();
        
        Self { state }
    }
}

#[async_trait]
impl ServerHandler for BufferServerHandler {
    async fn on_initialized(&self, runtime: Arc<dyn McpServer>) {
        let _ = runtime.stderr_message(
            "✅ Buffer Server initialized. Ready to process commands.".to_string()
        ).await;
    }

    async fn handle_initialize_request(
        &self,
        req: InitializeRequest,
        runtime: Arc<dyn McpServer>,
    ) -> Result<InitializeResult, rust_mcp_sdk::schema::RpcError> {
        runtime.set_client_details(req.params.clone()).await
            .map_err(|e| rust_mcp_sdk::schema::RpcError::internal_error()
                .with_message(format!("{}", e)))?;
        
        Ok(InitializeResult {
            server_info: Implementation {
                name: "simple-buffer-server".to_string(),
                version: "0.1.0".to_string(),
                title: Some("Simple Text Buffer MCP Server".to_string()),
            },
            capabilities: ServerCapabilities {
                tools: Some(ServerCapabilitiesTools { list_changed: None }),
                experimental: None,
                logging: None,
                prompts: None,
                resources: None,
                completions: None,
            },
            instructions: None,
            meta: None,
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        })
    }

    async fn handle_list_tools_request(
        &self,
        _: ListToolsRequest,
        _: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, rust_mcp_sdk::schema::RpcError> {
        Ok(ListToolsResult {
            tools: BufferTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        req: CallToolRequest,
        _: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let tool = BufferTools::try_from(req.params)
            .map_err(CallToolError::new)?;
        
        match tool {
            BufferTools::LoadFile(p) => p.run(&self.state).await,
            BufferTools::GetBuffer(p) => p.run(&self.state).await,
            BufferTools::AnalyzeBuffer(p) => p.run(&self.state).await,
            BufferTools::SetOutput(p) => p.run(&self.state).await,
            BufferTools::AppendBuffer(p) => p.run(&self.state).await,
            BufferTools::ClearBuffer(p) => p.run(&self.state).await,
        }
    }
}

// ============================================================================
// MAIN
// ============================================================================

#[derive(Parser)]
#[command(name = "simple-mcp-server")]
#[command(about = "A simple MCP server with text buffer manipulation")]
struct Args {}

#[tokio::main]
async fn main() {
    let _args = Args::parse();
    
    let transport = StdioTransport::new(TransportOptions::default())
        .expect("Failed to create transport");
    
    let handler = BufferServerHandler::new();
    
    let server = server_runtime::create_server(
        InitializeResult {
            server_info: Implementation {
                name: "simple-buffer-server".to_string(),
                version: "0.1.0".to_string(),
                title: Some("Simple Text Buffer MCP Server".to_string()),
            },
            capabilities: ServerCapabilities {
                tools: Some(ServerCapabilitiesTools { list_changed: None }),
                experimental: None,
                logging: None,
                prompts: None,
                resources: None,
                completions: None,
            },
            instructions: None,
            meta: None,
            protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        },
        transport,
        handler,
    );
    
    if let Err(e) = server.start().await {
        eprintln!("❌ Server error: {}", e);
    }
}
