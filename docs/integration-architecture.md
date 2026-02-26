# Integration Architecture

## Overview

Lattice is a monorepo with two integrated parts:
1. **Backend** (Rust) - MCP server, CLI, REST API
2. **Web UI** (Svelte) - Embedded SPA served by the backend

## Integration Points

### Build-Time Integration

```
┌────────────────────────────────────────────────────────────┐
│                     Build Process                           │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐                      ┌─────────────────┐  │
│  │   web-ui/   │  npm run build       │   web-ui/dist/  │  │
│  │   src/      │ ──────────────────►  │   (static)      │  │
│  └─────────────┘                      └────────┬────────┘  │
│                                                │            │
│                                     rust-embed │            │
│                                                ▼            │
│  ┌─────────────┐                      ┌─────────────────┐  │
│  │   crates/   │  cargo build         │    lattice      │  │
│  │   src/      │ ──────────────────►  │   (binary)      │  │
│  └─────────────┘                      └─────────────────┘  │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

The SvelteKit app is built to static files (`dist/`) which are embedded into the Rust binary using `rust-embed`. This creates a single self-contained executable.

### Runtime Integration

```
┌─────────────────────────────────────────────────────────────┐
│                    lattice serve                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │   MCP Server    │         │      Web Server         │   │
│  │   (rmcp stdio)  │         │      (Axum HTTP)        │   │
│  └────────┬────────┘         └────────────┬────────────┘   │
│           │                               │                 │
│           │                    ┌──────────┴──────────┐     │
│           │                    │                     │     │
│           │              ┌─────┴─────┐        ┌──────┴───┐ │
│           │              │ REST API  │        │ Static   │ │
│           │              │ /api/*    │        │ Files    │ │
│           │              └─────┬─────┘        └──────────┘ │
│           │                    │                           │
│           └────────────────────┼───────────────────────────┤
│                                │                           │
│                        ┌───────┴───────┐                   │
│                        │    Router     │                   │
│                        └───────┬───────┘                   │
│                                │                           │
│              ┌─────────────────┼─────────────────┐        │
│              ▼                 ▼                 ▼        │
│        ┌──────────┐     ┌──────────┐     ┌──────────┐    │
│        │  Search  │     │  Causal  │     │  Ingest  │    │
│        │ (Tantivy)│     │ (Fjall)  │     │  (git2)  │    │
│        └──────────┘     └──────────┘     └──────────┘    │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

## Communication Protocols

### Web UI → Backend

| Protocol | Endpoint | Purpose |
|----------|----------|---------|
| REST/JSON | `GET /api/search?q=...` | Full-text search |
| REST/JSON | `GET /api/spec/:id` | Get spec by ID |
| REST/JSON | `GET /api/graph/overview` | Graph statistics |
| REST/JSON | `GET /api/graph/node/:id` | Node with edges |
| REST/JSON | `GET /api/impact/:id` | Trace impact |
| REST/JSON | `GET /api/dependencies/:id` | Find dependencies |

### MCP Client → Backend

| Protocol | Transport | Purpose |
|----------|-----------|---------|
| MCP/JSON-RPC | stdio | AI agent tool calls |
| MCP/JSON-RPC | HTTP (optional) | Remote agent access |

## Data Flow

### Search Query Flow

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Web UI  │───►│ REST API │───►│  Router  │───►│  Search  │
│          │◄───│          │◄───│          │◄───│ (Tantivy)│
└──────────┘    └──────────┘    └──────────┘    └──────────┘
   JSON           JSON          SearchQuery      IndexReader
```

### Graph Query Flow

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Web UI  │───►│ REST API │───►│  Router  │───►│  Causal  │
│          │◄───│          │◄───│          │◄───│  (Fjall) │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
   JSON           JSON          GraphQuery       KV Lookup
```

### Sync Flow (MCP or CLI)

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│  Ingest  │───►│  Search  │    │  Causal  │
│  (git2)  │    │ (Tantivy)│    │  (Fjall) │
└────┬─────┘    └────┬─────┘    └────┬─────┘
     │               │               │
     │  SpecDoc      │  Index        │  Store
     └───────────────┴───────────────┘
```

## Shared Data

### Domain Types (spec-db-core)

Both parts share these types via the REST API:

| Type | Fields | JSON |
|------|--------|------|
| `SpecId` | String (validated) | `"spec::auth::jwt"` |
| `SpecDoc` | id, title, version, tags, depends_on, body | Full object |
| `SpecNode` | id, title, version | Subset for graph |
| `CausalEdge` | source, target, edge_type, trust, origin | Full object |

### API Response Formats

```typescript
// Search result
interface SearchResult {
  id: string;
  title: string;
  score: number;
  snippet: string;
}

// Graph node with edges
interface GraphNode {
  node: SpecNode;
  inbound: CausalEdge[];
  outbound: CausalEdge[];
}

// Graph overview
interface GraphOverview {
  node_count: number;
  edge_count: number;
  last_sync_sha: string;
}
```

## Configuration Sharing

Both parts read from `.lattice/config.yaml`:

| Setting | Backend | Web UI |
|---------|---------|--------|
| `web.enabled` | Starts HTTP server | N/A |
| `web.host` | Bind address | API base URL |
| `web.port` | Listen port | API base URL |
| `specs_dir` | Sync source | N/A |
| `data_dir` | Index location | N/A |

## Development Workflow

### Full Stack Development

```bash
# Terminal 1: Backend with auto-rebuild
cargo watch -x 'run -- serve'

# Terminal 2: Web UI with HMR
cd web-ui && npm run dev
```

### Production Build

```bash
# 1. Build web UI
cd web-ui && npm run build && cd ..

# 2. Build Rust binary (embeds web-ui/dist/)
cargo build --release

# 3. Single binary contains everything
./target/release/lattice serve
```

---

*Generated: 2026-02-27 | Scan Level: Quick*
