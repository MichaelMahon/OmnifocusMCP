# Contributing to OmniFocus MCP

Thanks for contributing. Keep changes small, focused, and test-backed.

## Development setup

Clone once, then set up each implementation independently.

```bash
git clone https://github.com/vitalyrodnenko/OmnifocusMCP.git
cd OmnifocusMCP
```

Python (`uv`, 3.11+):

```bash
cd python
uv sync --extra dev
```

TypeScript (`npm`, Node.js 20+):

```bash
cd typescript
npm install
```

Rust (`cargo`):

```bash
cd rust
cargo check
```

Detailed install guides:
- `docs/install-python.md`
- `docs/install-typescript.md`
- `docs/install-rust.md`

## Running tests

Python:

```bash
cd python && ruff check src/ && mypy src/ --strict && pytest tests/ -v
```

TypeScript:

```bash
cd typescript && npx tsc --noEmit && npm test
```

Rust:

```bash
cd rust && cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

Integration tests require OmniFocus running and macOS Automation permission.

## Key rules

- all 3 implementations must expose identical tool names, input schemas, and response shapes
- JXA scripts must be character-identical across implementations
- write the JXA script in Python first, then copy it to TypeScript and Rust
- all user input must be escaped through the implementation escape helper
- never use `shell=True` in Python or `exec` in Node for `osascript` calls

## Pull request guidelines

- one concern per PR, small diff preferred
- include a short rationale and testing notes
- lint, typecheck, and tests must pass before review
