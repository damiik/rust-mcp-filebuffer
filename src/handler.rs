// ============================================================================
// src/handler.rs
// ============================================================================
use crate::tools::BinaryTools;
use crate::state::ServerState;
use async_trait::async_trait;
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError, CallToolRequest, CallToolResult, 
    ListToolsRequest, ListToolsResult, RpcError,
};
use rust_mcp_sdk::{mcp_server::ServerHandler, McpServer};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BinaryAnalysisHandler {
    pub state: Arc<RwLock<ServerState>>,
}

impl BinaryAnalysisHandler {
    pub async fn new() -> Self {
        let state = Arc::new(RwLock::new(ServerState::new()));
        
        println!("\n🔬 Binary Analysis MCP Server Starting...");
        {
            // let s = state.blocking_read();
            let s = state.read().await;
            s.display();
        } // Drop the read lock here before moving state
        
        Self { state }
    }
}

#[async_trait]
impl ServerHandler for BinaryAnalysisHandler {
    async fn on_initialized(&self, runtime: Arc<dyn McpServer>) {
        let _ = runtime.stderr_message(
            "✅ Binary Analysis Server initialized. Ready for reverse engineering.".to_string()
        ).await;
    }

    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: BinaryTools::tools(),
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_params: BinaryTools =
            BinaryTools::try_from(request.params).map_err(CallToolError::new)?;
        
        match tool_params {
            BinaryTools::LoadBinary(tool) => tool.call_tool(&self.state).await,
            BinaryTools::ReadBytes(tool) => tool.call_tool(&self.state).await,
            BinaryTools::SearchPattern(tool) => tool.call_tool(&self.state).await,
            BinaryTools::ExtractSegment(tool) => tool.call_tool(&self.state).await,
            BinaryTools::AddBookmark(tool) => tool.call_tool(&self.state).await,
            BinaryTools::ReadString(tool) => tool.call_tool(&self.state).await,
            BinaryTools::ReadInteger(tool) => tool.call_tool(&self.state).await,
            BinaryTools::CalculateHash(tool) => tool.call_tool(&self.state).await,
            BinaryTools::GetInfo(tool) => tool.call_tool(&self.state).await,
            BinaryTools::AddNote(tool) => tool.call_tool(&self.state).await,
            BinaryTools::SetOutput(tool) => tool.call_tool(&self.state).await,
        }
    }
}