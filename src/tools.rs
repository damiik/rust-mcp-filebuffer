// ============================================================================
// src/tools.rs
// ============================================================================
use rust_mcp_sdk::schema::{schema_utils::CallToolError, CallToolResult, TextContent};

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use sha2::{Sha256, Digest};
use crate::state::ServerState;

use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};

//****************//
//  LoadBinary    //
//****************//
#[mcp_tool(
    name = "load_binary",
    description = "Loads a binary file into the buffer for analysis"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct LoadBinary {
    /// Path to the binary file
    pub path: String,
}

impl LoadBinary {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let data = fs::read(&self.path).await
            .map_err(|e| CallToolError::from_message(format!("Failed to read file: {}", e)))?;
        
        let mut s = state.write().await;
        s.buffer = data.clone();
        s.file_loaded = Some(self.path.clone());
        s.bookmarks.clear();
        s.segments.clear();
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!("✅ Loaded {} bytes from '{}'", data.len(), self.path))
        ]))
    }
}

//****************//
//  ReadBytes     //
//****************//
#[mcp_tool(
    name = "read_bytes",
    description = "Reads a specified number of bytes from the buffer at a given offset, returns hex dump"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ReadBytes {
    /// Starting offset in the buffer
    pub offset: usize,
    /// Number of bytes to read
    pub length: usize,
}

impl ReadBytes {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        
        if self.offset + self.length > s.buffer.len() {
            return Err(CallToolError::from_message("Read exceeds buffer bounds"));
        }
        
        let bytes = &s.buffer[self.offset..self.offset + self.length];
        let hex_dump = hex::encode(bytes);
        
        let ascii: String = bytes.iter()
            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
            .collect();
        
        let output = format!(
            "Offset 0x{:08X} ({} bytes):\nHex: {}\nASCII: {}",
            self.offset, self.length, hex_dump, ascii
        );
        
        Ok(CallToolResult::text_content(vec![TextContent::from(output)]))
    }
}

//*******************//
//  SearchPattern    //
//*******************//
#[mcp_tool(
    name = "search_pattern",
    description = "Searches for a hex pattern in the buffer, returns all matching offsets"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SearchPattern {
    /// Hex string pattern to search for (e.g., '4D5A' for PE header)
    pub pattern: String,
}

impl SearchPattern {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        
        let pattern = hex::decode(&self.pattern)
            .map_err(|e| CallToolError::from_message(format!("Invalid hex pattern: {}", e)))?;
        
        let mut matches = Vec::new();
        for i in 0..=s.buffer.len().saturating_sub(pattern.len()) {
            if &s.buffer[i..i + pattern.len()] == pattern.as_slice() {
                matches.push(i);
            }
        }
        
        let output = if matches.is_empty() {
            "No matches found".to_string()
        } else {
            format!("Found {} matches at offsets:\n{}", 
                matches.len(),
                matches.iter()
                    .map(|&off| format!("  0x{:08X}", off))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        
        Ok(CallToolResult::text_content(vec![TextContent::from(output)]))
    }
}

//*******************//
//  ExtractSegment   //
//*******************//
#[mcp_tool(
    name = "extract_segment",
    description = "Extracts a segment of bytes and stores it for later reference"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ExtractSegment {
    /// Starting offset
    pub offset: usize,
    /// Length of segment
    pub length: usize,
    /// Optional label for the segment
    pub label: Option<String>,
}

impl ExtractSegment {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        
        if self.offset + self.length > s.buffer.len() {
            return Err(CallToolError::from_message("Segment exceeds buffer bounds"));
        }
        
        let data = s.buffer[self.offset..self.offset + self.length].to_vec();
        let segment = crate::state::BinarySegment {
            offset: self.offset,
            data: data.clone(),
            label: self.label.clone(),
        };
        
        s.segments.push(segment);
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!(
                "✅ Extracted segment: {} bytes at 0x{:08X}{}",
                self.length,
                self.offset,
                self.label.as_ref().map(|l| format!(" ({})", l)).unwrap_or_default()
            ))
        ]))
    }
}

//****************//
//  AddBookmark   //
//****************//
#[mcp_tool(
    name = "add_bookmark",
    description = "Creates a named bookmark at a specific offset for quick reference"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AddBookmark {
    /// Name for the bookmark
    pub name: String,
    /// Offset to bookmark
    pub offset: usize,
}

impl AddBookmark {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        
        if self.offset > s.buffer.len() {
            return Err(CallToolError::from_message("Offset exceeds buffer size"));
        }
        
        s.bookmarks.insert(self.name.clone(), self.offset);
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!("✅ Bookmark '{}' added at 0x{:08X}", self.name, self.offset))
        ]))
    }
}

//****************//
//  ReadString    //
//****************//
#[mcp_tool(
    name = "read_string",
    description = "Attempts to read bytes as ASCII/UTF-8 string from specified offset"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ReadString {
    /// Starting offset
    pub offset: usize,
    /// Maximum length to read
    pub max_length: usize,
}

impl ReadString {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        
        let end = (self.offset + self.max_length).min(s.buffer.len());
        let bytes = &s.buffer[self.offset..end];
        
        let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let str_bytes = &bytes[..null_pos];
        
        let text = String::from_utf8_lossy(str_bytes);
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!("String at 0x{:08X} ({} bytes):\n{}", 
                self.offset, str_bytes.len(), text))
        ]))
    }
}

//****************//
//  ReadInteger   //
//****************//
#[mcp_tool(
    name = "read_integer",
    description = "Reads bytes as integer (u8, u16, u32, u64) with specified endianness"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct ReadInteger {
    /// Starting offset
    pub offset: usize,
    /// Integer size: 1, 2, 4, or 8 bytes
    pub size: u8,
    /// Endianness: 'little' or 'big'
    pub endian: String,
}

impl ReadInteger {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        
        if self.offset + self.size as usize > s.buffer.len() {
            return Err(CallToolError::from_message("Read exceeds buffer bounds"));
        }
        
        let bytes = &s.buffer[self.offset..self.offset + self.size as usize];
        
        let value = match (self.size, self.endian.as_str()) {
            (1, _) => bytes[0] as u64,
            (2, "little") => u16::from_le_bytes([bytes[0], bytes[1]]) as u64,
            (2, "big") => u16::from_be_bytes([bytes[0], bytes[1]]) as u64,
            (4, "little") => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64,
            (4, "big") => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64,
            (8, "little") => u64::from_le_bytes(bytes.try_into().unwrap()),
            (8, "big") => u64::from_be_bytes(bytes.try_into().unwrap()),
            _ => return Err(CallToolError::from_message("Invalid size or endianness")),
        };
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!(
                "u{} at 0x{:08X} ({} endian): {} (0x{:X})",
                self.size * 8, self.offset, self.endian, value, value
            ))
        ]))
    }
}

//******************//
//  CalculateHash   //
//******************//
#[mcp_tool(
    name = "calculate_hash",
    description = "Calculates SHA-256 hash of the entire buffer or a segment"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct CalculateHash {
    /// Optional offset (if None, hash entire buffer)
    pub offset: Option<usize>,
    /// Optional length (if None, hash from offset to end)
    pub length: Option<usize>,
}

impl CalculateHash {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        
        let offset = self.offset.unwrap_or(0);
        let end = self.length
            .map(|len| offset + len)
            .unwrap_or(s.buffer.len());
        
        if end > s.buffer.len() {
            return Err(CallToolError::from_message("Range exceeds buffer size"));
        }
        
        let data = &s.buffer[offset..end];
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from(format!(
                "SHA-256 (0x{:08X} - 0x{:08X}, {} bytes):\n{}",
                offset, end, data.len(), hex::encode(hash)
            ))
        ]))
    }
}

//************//
//  GetInfo   //
//************//
#[mcp_tool(
    name = "get_info",
    description = "Returns detailed information about the current buffer state"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetInfo {}

impl GetInfo {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let s = state.read().await;
        
        let info = format!(
            "📊 Buffer Information:\n\
             File: {}\n\
             Size: {} bytes (0x{:X})\n\
             Bookmarks: {}\n\
             Segments: {}\n\
             Notes: {}",
            s.file_loaded.as_deref().unwrap_or("None"),
            s.buffer.len(),
            s.buffer.len(),
            s.bookmarks.len(),
            s.segments.len(),
            s.analysis_notes.len()
        );
        
        Ok(CallToolResult::text_content(vec![TextContent::from(info)]))
    }
}

//************//
//  AddNote   //
//************//
#[mcp_tool(
    name = "add_note",
    description = "Adds a textual analysis note to the current session"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct AddNote {
    /// The analysis note text
    pub note: String,
}

impl AddNote {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        s.analysis_notes.push(self.note.clone());
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from("✅ Note added")
        ]))
    }
}

//**************//
//  SetOutput   //
//**************//
#[mcp_tool(
    name = "set_output",
    description = "Sets the final analysis output text"
)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SetOutput {
    /// Output text
    pub text: String,
}

impl SetOutput {
    pub async fn call_tool(&self, state: &Arc<RwLock<ServerState>>) 
        -> Result<CallToolResult, CallToolError> 
    {
        let mut s = state.write().await;
        s.output = self.text.clone();
        s.display();
        
        Ok(CallToolResult::text_content(vec![
            TextContent::from("✅ Output set")
        ]))
    }
}

//*****************//
//  BinaryTools    //
//*****************//
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
