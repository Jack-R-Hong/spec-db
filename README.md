# Lattice

A causal specification database for AI agents. Combines full-text search (Tantivy) with causal knowledge graph reasoning (DeepCausality + Fjall), exposed as MCP tools over stdio.

## Quick Start

```bash
cargo install --path .

# Initialize a project
lattice init

# Build search index and causal graph from specs
lattice sync

# Start MCP server (stdio)
lattice serve
```

## Spec Format

Specs are Markdown files with YAML frontmatter in `specs/`:

```markdown
---
id: "spec::auth::jwt-validation"
title: "JWT Validation"
version: 1
tags: ["auth", "security"]
depends_on: ["spec::auth::token-format"]
created: "2026-01-15"
---

# JWT Validation

Your spec content here.
```

## CLI Commands

| Command | Description |
|---|---|
| `lattice init` | Scaffold project structure with example specs and config |
| `lattice serve` | Start MCP server over stdio |
| `lattice sync` | Incremental sync from git |
| `lattice sync --full` | Full rebuild from git |
| `lattice rebuild` | Destructive full index rebuild |
| `lattice status` | Show doc count, last sync SHA, consistency state |

## MCP Tools

Once `lattice serve` is running, agents can call these tools:

| Tool | Description |
|---|---|
| `search_specs(query, limit?, tags?)` | Full-text search across indexed specs |
| `get_spec(id)` | Retrieve a spec by its ID |
| `trace_impact(id, depth?)` | Trace downstream impact from a spec node |
| `find_dependencies(id)` | Find upstream dependencies of a spec |
| `query(natural_language)` | Natural-language query with automatic routing |
| `add_spec(markdown)` | Ingest a new spec document |
| `sync(mode?)` | Trigger incremental or full sync |

### MCP Resources

| URI | Description |
|---|---|
| `spec://{id}` | Read individual spec content |
| `graph://overview` | Graph summary statistics |
| `graph://node/{id}` | Spec node with all inbound/outbound edges |

## Configuration

`.lattice/config.yaml`:

```yaml
specs_dir: specs          # Where spec markdown files live
data_dir: data            # Where Tantivy/Fjall indexes are stored
transport:
  stdio: true             # Always-on stdio transport
  http:                    # Optional HTTP transport
    host: 127.0.0.1
    port: 8080
telemetry:
  enabled: false          # Opt-in OpenTelemetry export
  endpoint: http://localhost:4317
  protocol: grpc
```

## MCP Client Config

Add to your MCP client (e.g. Claude Desktop `claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "lattice": {
      "command": "lattice",
      "args": ["serve"],
      "cwd": "/path/to/your/spec/project"
    }
  }
}
```

## Getting Started (Development)

### Prerequisites

- [Rust 1.85+](https://rustup.rs/)
- Git
- [Node.js 20+](https://nodejs.org/) (for web UI only)
- [Buck2](https://buck2.build/) (optional, alternative build system)

### Build from Source

```bash
# Clone
git clone <repo-url> && cd spec-db

# Build (debug)
cargo build

# Build (release)
cargo build --release

# Install locally
cargo install --path .
```

### Run Tests

```bash
# All tests (unit + integration)
cargo test

# Integration / acceptance tests only
cargo test --test '*'

# Single test file
cargo test --test acceptance_story_1_1
```

### Development Workflow

```bash
# 1. Create a test project somewhere
mkdir /tmp/my-specs && cd /tmp/my-specs && git init

# 2. Initialize lattice project
lattice init

# 3. Commit the scaffolded specs (lattice syncs from git)
git add -A && git commit -m "init"

# 4. Build index and causal graph
lattice sync

# 5. Start MCP server (stdio)
lattice serve

# 6. Check project status
lattice status
```

### Web UI

The web UI is a SvelteKit app embedded into the binary at build time.

```bash
cd web-ui
npm install
npm run dev       # Dev server with HMR
npm run build     # Production build (output: web-ui/dist/)
npm run check     # Type-check
```

Enable the web UI in `.lattice/config.yaml`:

```yaml
web:
  enabled: true
  host: 127.0.0.1
  port: 8080
```

Then `lattice serve` will start both the MCP stdio server and the web UI.

### Build with Buck2 (Alternative)

```bash
# Build the binary
buck2 build //:lattice

# Run directly
buck2 run //:lattice -- serve
```

Third-party dependencies are managed via [reindeer](https://github.com/facebookincubator/reindeer). After updating `Cargo.toml`:

```bash
reindeer buckify
```

### Project Structure

```
src/              Main binary (CLI entry point)
crates/
  core/           Config, spec model, shared types
  causal/         Causal graph (DeepCausality + Fjall)
  search/         Full-text search (Tantivy)
  ingest/         Git sync, markdown parsing
  router/         Query routing logic
  mcp/            MCP server + tool handlers
  web/            Web UI server (Axum + embedded SvelteKit)
web-ui/           SvelteKit frontend source
tests/            Integration / acceptance tests
```
