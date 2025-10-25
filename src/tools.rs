use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    tool_box,
};


use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use sha2::{Sha256, Digest};
use crate::state::ServerState;

// Load binary file
#[mcp_tool(
    name = "load_binary",
    title = "Load Binary File",
    description = "Loads a binary file into the buffer for analysis",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct LoadBinary {
    /// Path to the binary file
    pub path: String,
    #[serde(skip)]
    pub state: Arc<RwLock<ServerState>>,
}

// Read bytes at offset
#[mcp_tool(
    name = "read_bytes",
    title = "Read Bytes",
    description = "Reads a specified number of bytes from the buffer at a given offset, returns hex dump",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ReadBytes {
    /// Starting offset in the buffer
    pub offset: u64,
    /// Number of bytes to read
    pub length: u64,
    pub state: Arc<RwLock<ServerState>>,
}

// Search for byte pattern
#[mcp_tool(
    name = "search_pattern",
    title = "Search Byte Pattern",
    description = "Searches for a hex pattern in the buffer, returns all matching offsets",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SearchPattern {
    /// Hex string pattern to search for (e.g., '4D5A' for PE header)
    pub pattern: String,
    pub state: Arc<RwLock<ServerState>>,
}

// Extract segment
#[mcp_tool(
    name = "extract_segment",
    title = "Extract Segment",
    description = "Extracts a segment of bytes and stores it for later reference",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ExtractSegment {
    /// Starting offset
    pub offset: u64,
    /// Length of segment
    pub length: u64,
    /// Optional label for the segment
    pub label: Option<String>,
    pub state: Arc<RwLock<ServerState>>,
}

// Add bookmark
#[mcp_tool(
    name = "add_bookmark",
    title = "Add Bookmark",
    description = "Creates a named bookmark at a specific offset for quick reference",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AddBookmark {
    /// Name for the bookmark
    pub name: String,
    /// Offset to bookmark
    pub offset: u64,
    pub state: Arc<RwLock<ServerState>>,
}

// Interpret as string
#[mcp_tool(
    name = "read_string",
    title = "Read String",
    description = "Attempts to read bytes as ASCII/UTF-8 string from specified offset",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ReadString {
    /// Starting offset
    pub offset: u64,
    /// Maximum length to read
    pub max_length: u64,
    pub state: Arc<RwLock<ServerState>>,
}

// Read integers
#[mcp_tool(
    name = "read_integer",
    title = "Read Integer",
    description = "Reads bytes as integer (u8, u16, u32, u64) with specified endianness",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ReadInteger {
    /// Starting offset
    pub offset: u64,
    /// Integer size: 1, 2, 4, or 8 bytes
    pub size: u8,
    /// Endianness: 'little' or 'big'
    pub endian: String,
    pub state: Arc<RwLock<ServerState>>,
}

// Calculate hash
#[mcp_tool(
    name = "calculate_hash",
    title = "Calculate Hash",
    description = "Calculates SHA-256 hash of the entire buffer or a segment",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct CalculateHash {
    /// Optional offset (if None, hash entire buffer)
    pub offset: Option<u64>,
    /// Optional length (if None, hash from offset to end)
    pub length: Option<u64>,
    pub state: Arc<RwLock<ServerState>>,
}

// Get buffer info
#[mcp_tool(
    name = "get_info",
    title = "Get Buffer Info",
    description = "Returns detailed information about the current buffer state",
    read_only_hint = true
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetInfo {
    pub state: Arc<RwLock<ServerState>>,
}

// Add analysis note
#[mcp_tool(
    name = "add_note",
    title = "Add Analysis Note",
    description = "Adds a textual analysis note to the current session",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AddNote {
    /// The analysis note text
    pub note: String,
    pub state: Arc<RwLock<ServerState>>,
}

// Set output
#[mcp_tool(
    name = "set_output",
    title = "Set Output",
    description = "Sets the final analysis output text",
    read_only_hint = false
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SetOutput {
    /// Output text
    pub text: String,
    pub state: Arc<RwLock<ServerState>>,
}

// Generate tool box
tool_box!(
    BinaryTools,
    [
        LoadBinary,
        ReadBytes,
        SearchPattern,
        ExtractSegment,
        AddBookmark,
        ReadString,
        ReadInteger,
        CalculateHash,
        GetInfo,
        AddNote,
        SetOutput
    ]
);

// ============================================================================
// Tool Implementations
// ============================================================================

impl LoadBinary {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut s = self.state.write().await;
        s.load_binary(&self.path).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Loaded binary from {}", &self.path),
        )]))
    }
}

impl ReadBytes {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let bytes = s.read_bytes(self.offset, self.length).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Read bytes: {:?}", bytes),
        )]))
    }
}

impl SearchPattern {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let results = s.search_pattern(&self.pattern).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Found pattern at addresses: {:?}", results),
        )]))
    }
}

impl ExtractSegment {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let segment = s.extract_segment(&self.label).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Extracted segment: {:?}", segment),
        )]))
    }
}

impl AddBookmark {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut s = self.state.write().await;
        s.add_bookmark(self.offset, &self.name).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Added bookmark '{}' at address {}", &self.name, self.offset),
        )]))
    }
}

impl ReadString {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let string = s.read_string(self.offset).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Read string: {}", string),
        )]))
    }
}

impl ReadInteger {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let integer = s.read_integer(self.offset, self.size).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Read integer: {}", integer),
        )]))
    }
}

impl CalculateHash {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let hash = s.calculate_hash(self.offset, self.length).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Calculated hash: {}", hash),
        )]))
    }
}

impl GetInfo {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let s = self.state.read().await;
        let info = s.get_info().await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("File info: {:?}", info),
        )]))
    }
}

impl AddNote {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut s = self.state.write().await;
        s.add_note(&self.note).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Added note: {}", self.note),
        )]))
    }
}

impl SetOutput {
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut s = self.state.write().await;
        s.set_output(&self.text).await;
        Ok(CallToolResult::text_content(vec![TextContent::from(
            format!("Set output to {}", &self.text),
        )]))
    }
}
