# OmniFocus MCP (TypeScript)

TypeScript implementation of the OmniFocus Model Context Protocol (MCP) server for OmniFocus on macOS.

## Prerequisites

- macOS with OmniFocus installed and running
- Node.js 18+
- npm
- automation permission granted for your terminal/editor to control OmniFocus

## Install

```bash
cd typescript
npm install
```

## Build

```bash
cd typescript
npm run build
```

## Run Server

From the `typescript/` directory:

```bash
node dist/index.js
```

This starts the MCP server over stdio.

## MCP Client Configuration

### Claude Desktop

Add this to your `claude_desktop_config.json` MCP servers section:

```json
{
  "mcpServers": {
    "omnifocus-ts": {
      "command": "node",
      "args": ["/Users/your-user/Projects/OmnifocusMCP/typescript/dist/index.js"]
    }
  }
}
```

### Cursor

Use a stdio MCP entry that runs the built server file:

```json
{
  "mcpServers": {
    "omnifocus-ts": {
      "command": "node",
      "args": ["dist/index.js"],
      "cwd": "/Users/your-user/Projects/OmnifocusMCP/typescript"
    }
  }
}
```

### Cline

Configure the same stdio command in Cline MCP settings:

```json
{
  "mcpServers": {
    "omnifocus-ts": {
      "command": "node",
      "args": ["dist/index.js"],
      "cwd": "/Users/your-user/Projects/OmnifocusMCP/typescript"
    }
  }
}
```

### Generic stdio clients

Any MCP client that supports stdio can use:

- command: `node`
- args: `["dist/index.js"]`
- cwd: `/path/to/OmnifocusMCP/typescript`

## Usage Examples

Once connected from your MCP client, try:

- `ping` to verify server health
- `get_inbox` to retrieve current inbox tasks
- `list_tasks` with filters such as `status: "due_soon"`
- `create_task` to add an inbox or project task
- `project_planning` prompt to generate a structured plan from project state

## Development Checks

```bash
cd typescript
npx tsc --noEmit
npm test
```
# OmniFocus MCP Server (TypeScript)

TypeScript implementation of an MCP server for OmniFocus on macOS.

## Prerequisites

- macOS with OmniFocus installed
- Node.js 18+
- OmniFocus running
- macOS Automation permission granted to your terminal/host app

## Install

```bash
npm install
```

## Build

```bash
npm run build
```

## Run over stdio

```bash
npm start
```

## MCP client configuration examples

### Claude Desktop

```json
{
  "mcpServers": {
    "omnifocus": {
      "command": "node",
      "args": ["/absolute/path/to/OmnifocusMCP/typescript/dist/index.js"]
    }
  }
}
```

### Cursor

```json
{
  "mcpServers": {
    "omnifocus": {
      "command": "node",
      "args": ["/absolute/path/to/OmnifocusMCP/typescript/dist/index.js"]
    }
  }
}
```

### Cline

```json
{
  "mcpServers": {
    "omnifocus": {
      "command": "node",
      "args": ["/absolute/path/to/OmnifocusMCP/typescript/dist/index.js"]
    }
  }
}
```

### generic stdio clients

Use a stdio command that starts the built server:

```bash
node /absolute/path/to/OmnifocusMCP/typescript/dist/index.js
```
