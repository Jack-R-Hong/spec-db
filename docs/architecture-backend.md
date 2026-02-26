# Backend Architecture (Rust)

## Overview

The Lattice backend is a Rust workspace with 7 crates following a clean dependency hierarchy. It implements a causal specification database exposed via MCP (Model Context Protocol) and REST API.

## Architecture Pattern

**Service-Oriented Modular Monolith** with:
- Clean crate boundaries enforcing separation of concerns
- Trait-based interfaces for subsystem abstraction
- Async boundary at the edge (MCP/web handlers)
- Sync implementations for data subsystems

## Crate Dependency Graph

```
                    ┌─────────────┐
                    │   lattice   │  (binary)
                    │   src/      │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
        ┌─────────┐  ┌─────────┐  ┌─────────┐
        │   mcp   │  │   web   │  │ router  │
        └────┬────┘  └────┬────┘  └────┬────┘
             │            │            │
             └────────────┼────────────┘
                          │
              ┌───────────┼───────────┐
              ▼           ▼           ▼
        ┌─────────┐ ┌─────────┐ ┌─────────┐
        │ search  │ │ causal  │ │ ingest  │
        └────┬────┘ └────┬────┘ └────┬────┘
             │           │           │
             └───────────┼───────────┘
                         ▼
                   ┌─────────┐
                   │  core   │
                   └─────────┘
```

## Crate Responsibilities

### spec-db-core
**Purpose**: Shared domain types, traits, and configuration

| Component | Description |
|-----------|-------------|
| `SpecId` | Validated newtype: `spec::{segment}::{segment}` pattern |
| `SpecDoc` | Full spec document with metadata and body |
| `TrustLevel` | Clamped `[0.0, 1.0]` confidence score |
| `CausalEdge` | Edge with type, trust, origin (Human/AI) |
| `SpecDbError` | Unified error enum (thiserror) |
| `SearchEngine` | Trait for search operations |
| `CausalGraph` | Trait for graph operations |

### spec-db-search
**Purpose**: Full-text search via Tantivy

- Index schema: id, title, body, tags, version
- BM25 ranking with field boosting
- Tag filtering support
- Snippet generation for results

### spec-db-causal
**Purpose**: Causal graph storage and reasoning via DeepCausality + Fjall

- Node storage: `SpecId → SpecNode` (id, title, version)
- Edge storage: `{from}\x00{to} → CausalEdge`
- Edge types: DependsOn, Constrains, Implements
- Impact tracing with depth limits
- Dependency resolution

### spec-db-ingest
**Purpose**: Git-based synchronization and markdown parsing

- Git diff-based incremental sync
- Full tree walk for complete rebuilds
- YAML frontmatter parsing
- SpecId validation at ingestion boundary
- Consistency verification (SHA + doc count)

### spec-db-router
**Purpose**: Query classification and routing

- Natural language query analysis
- Routes to search vs causal subsystem
- Hybrid query support

### spec-db-mcp
**Purpose**: MCP protocol implementation

| Component | Description |
|-----------|-------------|
| Tool handlers | `search_specs`, `get_spec`, `trace_impact`, etc. |
| Resource handlers | `spec://`, `graph://` URIs |
| Transport | stdio (always-on) + optional HTTP |

### spec-db-web
**Purpose**: REST API and embedded UI serving

- Axum-based HTTP server
- REST endpoints for all MCP tool equivalents
- Embedded SvelteKit static files (rust-embed)
- CORS configuration

## Data Architecture

### Storage Engines

| Store | Engine | Location | Purpose |
|-------|--------|----------|---------|
| Search Index | Tantivy | `data/tantivy/` | Full-text indexing |
| Graph Store | Fjall | `data/fjall/` | KV for nodes, edges, metadata |
| Source of Truth | Git | `specs/` | Markdown spec files |

### Key Formats (Fjall)

| Key Pattern | Value | Description |
|-------------|-------|-------------|
| `{spec_id}` | `SpecNode` (bincode) | Node data |
| `{from_id}\x00{to_id}` | `CausalEdge` (bincode) | Edge data |
| `"last_sync_sha"` | String | Git commit SHA |
| `"doc_count"` | u64 | Consistency check |

## Async Architecture

```
┌────────────────────────────────────────────────────────┐
│                    Async Boundary                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │ MCP Handler │  │ Web Handler │  │ CLI Command │    │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘    │
│         │                │                │            │
│         └────────────────┼────────────────┘            │
│                          ▼                             │
│                  spawn_blocking()                      │
└────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────┼──────────────────────────────┐
│                    Sync Subsystems                       │
│                          ▼                               │
│  ┌─────────┐      ┌─────────┐      ┌─────────┐         │
│  │ Search  │      │ Causal  │      │ Ingest  │         │
│  │(Tantivy)│      │ (Fjall) │      │ (git2)  │         │
│  └─────────┘      └─────────┘      └─────────┘         │
└──────────────────────────────────────────────────────────┘
```

**Rule**: Never call Tantivy/Fjall/git2 from async context without `spawn_blocking`.

## Error Handling

```rust
// Library crates: thiserror
#[derive(thiserror::Error, Debug)]
pub enum SpecDbError {
    #[error("search error: {0}")]
    SearchError(String),
    #[error("graph error: {0}")]
    GraphError(String),
    // ... etc
}

// Binary: anyhow for ergonomic error reporting
fn main() -> anyhow::Result<()> { ... }
```

## Configuration

`.lattice/config.yaml`:
```yaml
specs_dir: specs          # Spec markdown location
data_dir: data            # Index storage location
transport:
  stdio: true             # Always-on MCP stdio
  http: null              # Optional HTTP transport
web:
  enabled: true           # Enable web UI
  host: '127.0.0.1'
  port: 3000
telemetry:
  enabled: false          # OpenTelemetry export
```

## Build Configuration

| File | Setting |
|------|---------|
| `Cargo.toml` | Edition 2024, MSRV 1.85, LTO release |
| `rustfmt.toml` | max_width=100, use_small_heuristics=Max |
| `clippy.toml` | allow-unwrap-in-tests=true |

---

*Generated: 2026-02-27 | Scan Level: Quick*
