# OmniFocus MCP

OmniFocus MCP is a Model Context Protocol server that lets MCP-compatible clients interact with OmniFocus on macOS via JXA (`osascript` + `evaluateJavaScript`).

This repository contains two implementations:

- Python implementation (primary): `python/`
- TypeScript implementation (port): `typescript/`

## prerequisites

- macOS
- OmniFocus 3+
- Python 3.10+ (for the Python implementation)
- Node.js 18+ (for the TypeScript implementation)

## compatible clients

Any MCP client that supports stdio transport, including:

- Claude Desktop
- Cursor
- Cline
- Zed
- custom MCP clients

## implementation docs

- Python: `python/README.md`
- TypeScript: `typescript/README.md`
