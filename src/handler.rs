use crate::tools::BinaryTools;
use async_trait::async_trait;
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError,
    CallToolRequest,
    CallToolResult,
    ListToolsRequest,
    ListToolsResult,
    RpcError,
};
use rust_mcp_sdk::{mcp_server::ServerHandler, McpServer};
use std::sync::Arc;
use crate::state::ServerState;
use tokio::sync::RwLock;

pub struct BinaryAnalysisHandler {
    pub state: Arc<RwLock<ServerState>>,
}

impl BinaryAnalysisHandler {
    pub async fn new() -> Self {
        let state = Arc::new(RwLock::new(ServerState::new()));

        println!("\n🔬 Binary Analysis MCP Server Starting...");
        {
            let s = state.read().await;
            s.display();
        }

        Self { state }
    }
}

#[async_trait]
#[allow(unused)]
impl ServerHandler for BinaryAnalysisHandler {
    async fn handle_list_tools_request(
        &self,
        request: ListToolsRequest,
        runtime: Arc<dyn McpServer>,
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
        runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let tool_params: BinaryTools = 
            BinaryTools::try_from(request.params).map_err(CallToolError::new)?;

        match tool_params {
            BinaryTools::LoadBinary(p) => p.call_tool().await,
            BinaryTools::ReadBytes(p) => p.call_tool().await,
            BinaryTools::SearchPattern(p) => p.call_tool().await,
            BinaryTools::ExtractSegment(p) => p.call_tool().await,
            BinaryTools::AddBookmark(p) => p.call_tool().await,
            BinaryTools::ReadString(p) => p.call_tool().await,
            BinaryTools::ReadInteger(p) => p.call_tool().await,
            BinaryTools::CalculateHash(p) => p.call_tool().await,
            BinaryTools::GetInfo(p) => p.call_tool().await,
            BinaryTools::AddNote(p) => p.call_tool().await,
            BinaryTools::SetOutput(p) => p.call_tool().await,
        }
    }
}