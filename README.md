# OmniFocus MCP

OmniFocus MCP is a Model Context Protocol server that lets MCP-compatible clients interact with OmniFocus on macOS via JXA (`osascript` + `evaluateJavaScript`).

This monorepo ships two full implementations with matching tool names and response shapes:

- Python implementation: `python/`
- TypeScript implementation: `typescript/`

## Prerequisites

- macOS with OmniFocus 3+ installed
- OmniFocus running when tools are invoked
- macOS Automation permission granted to your terminal/editor
- Python 3.10+ (Python implementation)
- Node.js 18+ and npm (TypeScript implementation)

## Feature Comparison

| Capability | Python (`python/`) | TypeScript (`typescript/`) |
| --- | --- | --- |
| MCP transport | stdio | stdio |
| JXA bridge (`evaluateJavaScript`) | yes | yes |
| Read tools | yes (10) | yes (10) |
| Write tools | yes (9) | yes (9) |
| Resources | yes (3) | yes (3) |
| Prompts | yes (4) | yes (4) |
| Primary run command | `python -m omnifocus_mcp` or `omnifocus-mcp` | `node dist/index.js` |
| Packaging entrypoint | `pyproject.toml` script | `package.json` bin |

## Install

### Python implementation

```bash
cd python
uv pip install -e ".[dev]"
```

Alternative:

```bash
cd python
python -m pip install -e ".[dev]"
```

### TypeScript implementation

```bash
cd typescript
npm install
npm run build
```

## MCP client config examples

Any MCP client with stdio support can use either implementation.

### Claude Desktop

Python:

```json
{
  "mcpServers": {
    "omnifocus-python": {
      "command": "uv",
      "args": ["run", "omnifocus-mcp"],
      "cwd": "/absolute/path/to/OmnifocusMCP/python"
    }
  }
}
```

TypeScript:

```json
{
  "mcpServers": {
    "omnifocus-typescript": {
      "command": "node",
      "args": ["/absolute/path/to/OmnifocusMCP/typescript/dist/index.js"]
    }
  }
}
```

### Cursor

Python:

```json
{
  "mcpServers": {
    "omnifocus-python": {
      "command": "python",
      "args": ["-m", "omnifocus_mcp"],
      "cwd": "/absolute/path/to/OmnifocusMCP/python"
    }
  }
}
```

TypeScript:

```json
{
  "mcpServers": {
    "omnifocus-typescript": {
      "command": "node",
      "args": ["dist/index.js"],
      "cwd": "/absolute/path/to/OmnifocusMCP/typescript"
    }
  }
}
```

### Cline

Python:

```json
{
  "mcpServers": {
    "omnifocus-python": {
      "command": "python",
      "args": ["-m", "omnifocus_mcp"],
      "cwd": "/absolute/path/to/OmnifocusMCP/python"
    }
  }
}
```

TypeScript:

```json
{
  "mcpServers": {
    "omnifocus-typescript": {
      "command": "node",
      "args": ["dist/index.js"],
      "cwd": "/absolute/path/to/OmnifocusMCP/typescript"
    }
  }
}
```

### switching between Python and TypeScript

- use the Python config when you want `uv`/`python` execution from `python/`
- use the TypeScript config when you want `node` execution from `typescript/dist/index.js`
- keep only one OmniFocus server entry enabled in your client config to avoid duplicate tool sets

## switching implementations

1. choose either the Python or TypeScript command block for your MCP client
2. replace the existing OmniFocus server entry with the other implementation command
3. restart the MCP client so it reloads the server command

## Docs

- Python docs: `python/README.md`
- TypeScript docs: `typescript/README.md`
