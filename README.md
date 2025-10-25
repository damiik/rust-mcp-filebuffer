##Basic example of sse mcp server in Rust
---
Example of mcp configuration:
```json
{
  "mcpServers": {
    "binary-analysis-mcp": {
      "autoApprove": [],
      "disabled": false,
      "timeout": 60,
      "type": "sse",
      "url": "http://localhost:8080/sse"
    }
  }
}
```