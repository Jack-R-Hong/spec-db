# Source Tree Analysis

## Repository Structure

```
lattice/
├── src/                    # Binary entry point
│   ├── main.rs            # CLI dispatcher (clap)
│   └── telemetry.rs       # Tracing setup
│
├── crates/                 # Workspace crates (7 total)
│   ├── core/              # Shared types, config, traits
│   │   └── src/
│   │       ├── config.rs      # Config loading (.lattice/config.yaml)
│   │       ├── error.rs       # SpecDbError (thiserror)
│   │       ├── traits.rs      # SearchEngine, CausalGraph, SpecStore
│   │       └── types.rs       # SpecId, SpecDoc, TrustLevel, CausalEdge
│   │
│   ├── search/            # Tantivy full-text search
│   │   └── src/           # Index management, query execution
│   │
│   ├── causal/            # DeepCausality + Fjall graph
│   │   └── src/           # Graph storage, traversal, impact analysis
│   │
│   ├── ingest/            # Git sync + markdown parsing
│   │   └── src/           # git2 integration, pulldown-cmark
│   │
│   ├── router/            # Query classification
│   │   └── src/           # Routes queries to search/causal
│   │
│   ├── mcp/               # MCP server (rmcp)
│   │   └── src/           # Tool handlers, resource handlers
│   │
│   └── web/               # Web UI server (Axum)
│       └── src/           # REST API, static file serving
│
├── web-ui/                 # SvelteKit frontend (Part: web-ui)
│   ├── src/
│   │   ├── routes/        # SvelteKit routes (+page.svelte, +layout.svelte)
│   │   └── lib/           # Shared components and utilities
│   ├── package.json       # Node dependencies
│   ├── svelte.config.js   # SvelteKit config
│   ├── vite.config.ts     # Vite build config
│   └── tsconfig.json      # TypeScript config
│
├── tests/                  # Integration & acceptance tests
│   ├── acceptance_story_*.rs  # Story-based acceptance tests (18 files)
│   └── integration.rs     # Cross-crate integration tests
│
├── specs/                  # Example spec documents (user content)
│
├── data/                   # Runtime data (NOT in git)
│   ├── tantivy/           # Search index files
│   └── fjall/             # KV store files
│
├── .lattice/               # Project configuration
│   └── config.yaml        # Runtime config
│
├── docs/                   # Documentation output
│   └── project-context.md # AI agent implementation rules
│
├── _bmad-output/           # BMAD planning artifacts
│   └── planning-artifacts/
│
├── Cargo.toml             # Workspace manifest
├── Cargo.lock             # Locked dependencies
├── rustfmt.toml           # Code formatting (edition 2024, max_width 100)
├── clippy.toml            # Linting (allow unwrap in tests)
├── LICENSE                # Project license
└── README.md              # Project documentation
```

## Critical Directories

### Backend Crates

| Directory | Owner | Purpose |
|-----------|-------|---------|
| `crates/core/` | All | Domain types, traits, config - everyone depends on this |
| `crates/search/` | Tantivy | Full-text indexing and search execution |
| `crates/causal/` | DeepCausality + Fjall | Causal graph storage and traversal |
| `crates/ingest/` | git2 | Git sync, markdown parsing, spec validation |
| `crates/router/` | Query routing | Classifies queries → search or causal subsystem |
| `crates/mcp/` | rmcp | MCP tool/resource handlers, stdio/http transport |
| `crates/web/` | Axum | REST API, embedded SvelteKit static serving |

### Web UI Structure

| Directory | Purpose |
|-----------|---------|
| `web-ui/src/routes/` | SvelteKit page routes |
| `web-ui/src/lib/` | Shared components, stores, utilities |
| `web-ui/dist/` | Production build output (embedded into binary) |

### Data Directories (Runtime, NOT in Git)

| Directory | Owner Crate | Contents |
|-----------|-------------|----------|
| `data/tantivy/` | search | Tantivy index segments |
| `data/fjall/` | causal | Fjall LSM-tree files |

## Entry Points

| Entry Point | Location | Purpose |
|-------------|----------|---------|
| **CLI** | `src/main.rs` | Command dispatcher (init, serve, sync, rebuild, status) |
| **MCP Server** | `crates/mcp/src/` | Tool handlers activated via `lattice serve` |
| **Web Server** | `crates/web/src/` | REST API + static files via `lattice serve` |
| **Web UI** | `web-ui/src/routes/+page.svelte` | SvelteKit app entry |

## Test Organization

| Location | Type | Count |
|----------|------|-------|
| `tests/acceptance_story_*.rs` | Acceptance tests | 18 files |
| `tests/integration.rs` | Integration tests | 1 file |
| `crates/*/src/*.rs` | Unit tests (inline) | Per-module |

## Configuration Files

| File | Purpose |
|------|---------|
| `.lattice/config.yaml` | Runtime configuration (specs_dir, data_dir, transports) |
| `Cargo.toml` | Rust workspace manifest with pinned versions |
| `rustfmt.toml` | Formatting: Edition 2024, max_width 100 |
| `clippy.toml` | Linting: allow unwrap in tests |
| `web-ui/package.json` | Node dependencies for Svelte app |
| `web-ui/svelte.config.js` | SvelteKit adapter configuration |
| `web-ui/vite.config.ts` | Vite build settings |

---

*Generated: 2026-02-27 | Scan Level: Quick*
