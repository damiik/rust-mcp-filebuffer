// ============================================================================
// src/state.rs
// ============================================================================
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct BinarySegment {
    pub offset: usize,
    pub data: Vec<u8>,
    pub label: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ServerState {
    pub buffer: Vec<u8>,
    pub file_loaded: Option<String>,
    pub bookmarks: HashMap<String, usize>,
    pub segments: Vec<BinarySegment>,
    pub analysis_notes: Vec<String>,
    pub output: String,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            file_loaded: None,
            bookmarks: HashMap::new(),
            segments: Vec::new(),
            analysis_notes: Vec::new(),
            output: String::new(),
        }
    }

    pub fn display(&self) {
        println!("\n{}", "=".repeat(70));
        println!("🔬 BINARY ANALYSIS SERVER STATE");
        println!("{}", "=".repeat(70));
        
        println!("\n📂 Loaded File: {}", 
            self.file_loaded.as_deref().unwrap_or("None"));
        
        println!("\n📊 Buffer: {} bytes", self.buffer.len());
        if !self.buffer.is_empty() {
            let preview_len = self.buffer.len().min(64);
            println!("  First {} bytes (hex):", preview_len);
            println!("  {}", hex::encode(&self.buffer[..preview_len]));
            if self.buffer.len() > 64 {
                println!("  ... ({} more bytes)", self.buffer.len() - 64);
            }
        }
        
        println!("\n🔖 Bookmarks: {}", self.bookmarks.len());
        for (name, offset) in &self.bookmarks {
            println!("  {} -> 0x{:08X}", name, offset);
        }
        
        println!("\n📦 Segments: {}", self.segments.len());
        for (i, seg) in self.segments.iter().enumerate() {
            println!("  [{}] 0x{:08X}: {} bytes{}", 
                i, 
                seg.offset, 
                seg.data.len(),
                seg.label.as_ref().map(|l| format!(" ({})", l)).unwrap_or_default()
            );
        }
        
        println!("\n📝 Analysis Notes: {}", self.analysis_notes.len());
        for (i, note) in self.analysis_notes.iter().enumerate() {
            let preview = if note.len() > 60 {
                format!("{}...", &note[..60])
            } else {
                note.clone()
            };
            println!("  [{}] {}", i, preview);
        }
        
        println!("\n📤 Output:");
        if self.output.is_empty() {
            println!("  [Empty]");
        } else {
            println!("  {}", self.output);
        }
        
        println!("\n{}", "=".repeat(70));
    }
}
