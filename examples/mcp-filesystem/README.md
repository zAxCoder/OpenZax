# MCP Filesystem Example

Demonstrates OpenZax MCP client connecting to the filesystem MCP server.

## Prerequisites

```bash
# Install Node.js (if not already installed)
# Then install the filesystem MCP server
npm install -g @modelcontextprotocol/server-filesystem
```

## Running

```bash
# From the examples/mcp-filesystem directory
cargo run

# Or from workspace root
cargo run --example mcp-filesystem
```

## What It Does

1. Spawns the filesystem MCP server via stdio
2. Initializes MCP connection
3. Lists available tools
4. Reads README.md using the `read_file` tool
5. Displays file content preview

## Expected Output

```
OpenZax MCP Filesystem Example

Starting filesystem MCP server...
Initializing MCP connection...
✓ Connected to: @modelcontextprotocol/server-filesystem v0.1.0

Available tools:
  - read_file: Read complete contents of a file
  - write_file: Write content to a file
  - list_directory: List contents of a directory
  - create_directory: Create a new directory
  - move_file: Move or rename a file
  - search_files: Search for files

Reading README.md...
✓ File read successfully
Content preview: # OpenZax

> Secure AI Development Assistant built with Rust

OpenZax is a next-generation autonomous desktop AI operating system...

Example completed!
```

## Troubleshooting

### "Failed to spawn process"

Make sure the MCP server is installed:
```bash
npm install -g @modelcontextprotocol/server-filesystem
```

### "Connection closed"

The server may have crashed. Check that:
- Node.js is installed and in PATH
- The server package is correctly installed
- You have read permissions for the current directory

## Next Steps

- Try other MCP servers (GitHub, PostgreSQL, etc.)
- Integrate MCP tools into OpenZax agent workflows
- Build custom MCP servers for your use cases
