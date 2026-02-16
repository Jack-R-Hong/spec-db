# Technical Research: spec-db Technology Stack

**Date:** 2026-02-17
**Source:** BMAD Party Mode discussion + library documentation research
**Focus:** Technology evaluation for Tantivy, Fjall, DeepCausality, and rmcp

---

## Technology Stack Overview

| Component | Library | Role | Version |
|-----------|---------|------|---------|
| Full-text search | **Tantivy** | Spec discovery and keyword search | 0.22+ |
| Key-value persistence | **Fjall** | Durable storage for DeepCausality graph | 2.x |
| Causal reasoning | **DeepCausality** | Causal graph, context-hyper-graphs, CSM | 0.13+ |
| MCP server | **rmcp** | AI agent API via Model Context Protocol | 0.8+ |
| Git integration | **git2** | Programmatic git access for spec sync | latest |
| Markdown parsing | **pulldown-cmark** | Parse spec markdown body | 0.12+ |
| YAML frontmatter | **serde_yaml** | Parse spec YAML frontmatter | 0.9+ |
| Async runtime | **Tokio** | Async runtime for MCP server | 1.x |

All components are **pure Rust** — no FFI, no external services, fully embeddable.

---

## 1. Tantivy — Full-Text Search Engine

**Repository:** [quickwit-oss/tantivy](https://github.com/quickwit-oss/tantivy)
**Crate:** `tantivy`
**Role in spec-db:** Discovery layer — full-text search for specs

### What It Is

Tantivy is a full-text search engine library written in Rust, inspired by Apache Lucene. It provides indexing, querying, and segment management. It is a **library**, not a standalone server — giving full control over the search implementation.

### Key Capabilities

- **BM25 scoring** for relevance-ranked results
- **Schema-based indexing** with typed fields (TEXT, STRING, STORED, FAST, JSON)
- **JSON field support** — index and query nested JSON structures
- **Query parser** with boolean queries, phrase queries, range queries, fuzzy search
- **Tokenizers** with stemming support for 17 languages
- **Incremental indexing** — add/update/delete documents without full rebuild
- **Multithreaded** indexing and searching
- **Sub-10ms startup time**
- **SIMD compression** for optimized performance
- **In-memory or directory-based** index storage

### Tantivy Schema for spec-db

```rust
let mut schema_builder = Schema::builder();
let id = schema_builder.add_text_field("id", STRING | STORED);
let title = schema_builder.add_text_field("title", TEXT | STORED);
let body = schema_builder.add_text_field("body", TEXT);
let tags = schema_builder.add_text_field("tags", STRING | STORED);
let meta = schema_builder.add_json_field("meta", STORED | TEXT);
let schema = schema_builder.build();
```

### Usage Pattern

```rust
// Create index
let index = Index::create_in_dir(&index_path, schema.clone())?;
let mut writer: IndexWriter = index.writer(50_000_000)?;

// Index a spec
writer.add_document(doc!(
    id => "spec::auth::jwt-validation",
    title => "JWT Token Validation Specification",
    body => "All API endpoints requiring authentication...",
    tags => "auth",
    tags => "security",
))?;
writer.commit()?;

// Search
let reader = index.reader()?;
let searcher = reader.searcher();
let query_parser = QueryParser::for_index(&index, vec![title, body]);
let query = query_parser.parse_query("authentication JWT")?;
let results = searcher.search(&query, &TopDocs::with_limit(10))?;
```

### Assessment for spec-db

- **Fit:** Excellent — purpose-built for this use case
- **Scale:** Handles millions of documents; hundreds of specs is trivial
- **Integration:** Clean Rust API, embeddable, no external dependencies
- **Recovery:** Index is rebuildable from source data (git)

---

## 2. Fjall — Embedded Key-Value Storage

**Repository:** [fjall-rs/fjall](https://github.com/fjall-rs/fjall)
**Crate:** `fjall`
**Role in spec-db:** Persistence backend for DeepCausality causal graph

### What It Is

Fjall is a log-structured, embeddable key-value storage engine written in Rust. Inspired by RocksDB, it offers a thread-safe BTreeMap-like API with LSM-tree based storage.

### Key Capabilities

- **100% safe and stable Rust** — no unsafe code
- **Multiple keyspaces** (similar to column families) with cross-keyspace atomic semantics
- **Atomic batch writes** across keyspaces
- **Serializable transactions** (single-writer or optimistic/SSI)
- **Prefix and range searching** with forward and reverse iteration
- **Built-in compression** (LZ4 default)
- **Configurable compaction strategies** (Leveled, FIFO)
- **Key-value separation** for large blobs
- **Automatic background maintenance**

### Keyspace Design for spec-db

```rust
let db = Database::builder("./data/fjall").open()?;

// Keyspace for causal graph nodes (specs, modules, interfaces)
let nodes = db.keyspace("nodes", KeyspaceCreateOptions::default)?;

// Keyspace for causal edges (relationships between nodes)
let edges = db.keyspace("edges", KeyspaceCreateOptions::default)?;

// Keyspace for context-hyper-graph metadata
let contexts = db.keyspace("contexts", KeyspaceCreateOptions::default)?;

// Keyspace for system metadata (last synced commit, config, etc.)
let meta = db.keyspace("meta", KeyspaceCreateOptions::default)?;
```

### Cross-Keyspace Atomic Writes

```rust
// Atomically add a node and its edges
let mut batch = db.batch();
batch.insert(&nodes, "spec::auth::jwt", serialized_node);
batch.insert(&edges, "spec::auth::jwt->spec::auth::token", serialized_edge);
batch.commit()?;
```

### Assessment for spec-db

- **Fit:** Excellent — lightweight embedded KV perfect for graph persistence
- **Scale:** Handles far more than hundreds of nodes; this is well within comfort zone
- **Integration:** Clean Rust API, embeddable, thread-safe
- **Recovery:** Data durability via LSM-tree; can also rebuild from git export

---

## 3. DeepCausality — Causal Reasoning Engine

**Repository:** [deepcausality/deep_causality.rs](https://github.com/deepcausality/deep_causality.rs)
**Crate:** `deep_causality`
**Role in spec-db:** In-memory causal graph for reasoning about spec relationships

### What It Is

DeepCausality is a hyper-geometric computational causality library that enables fast and deterministic context-aware causal reasoning over complex causality models. Written in Rust with production-grade safety, reliability, and performance.

### Key Capabilities

- **Recursive causal data structures** (Causaloids) for expressing complex causal structures
- **Context-hyper-graphs** for context awareness across data-like, time-like, space-like, spacetime-like entities
- **Causal State Machine (CSM)** for enforcing transition rules and validating causal edges
- **Four layers of causal reasoning:** observations, assumptions, inferences, causes
- **Fast and deterministic** — no probabilistic uncertainty, pure logical reasoning
- **Inference analysis:** How many observations are inferable? Non-inferable? Inverse-inferable?

### Architecture: Four Layers of Causal Reasoning

1. **Observations** — raw data (specs, code modules, interfaces)
2. **Assumptions** — explicit, testable premises about relationships
3. **Inference** — derived insights from observations under assumptions
4. **Causality** — established causal relationships (A causes B)

### Dependencies

- `deep_causality_ast` — abstract syntax tree for causal expressions
- `deep_causality_data_structures` — specialized data structures
- `deep_causality_haft` — hyper-graph framework
- `deep_causality_tensor` — tensor operations
- `deep_causality_uncertain` — uncertainty modeling
- `ultragraph` — underlying graph engine

### Use in spec-db

- **Nodes:** Specs, code modules, architecture decisions, interfaces
- **Edges:** Causal relationships (depends_on, constrains, implements, breaks)
- **CSM:** Validates AI-inferred edges before accepting them into the graph
- **Graph queries:** "What is causally affected if spec X changes?" → traverse downstream
- **Lifecycle:** Full graph loaded from Fjall at startup, persisted back on write/shutdown

### Assessment for spec-db

- **Fit:** Excellent for the reasoning layer — causal traversal is the core differentiator
- **Scale:** Hundreds of nodes with causal edges is well within capacity
- **Integration:** Pure Rust, works in-memory, can serialize/deserialize to Fjall
- **Unique value:** CSM validation for AI-inferred edges is a key feature

---

## 4. rmcp — Official Rust MCP SDK

**Repository:** [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
**Crate:** `rmcp`
**Role in spec-db:** MCP server for AI agent communication

### What It Is

The official Rust SDK for the Model Context Protocol (MCP). Provides async-first server and client implementation on Tokio runtime. MCP is the standardized protocol for AI assistants (Claude, GPT, etc.) to discover and call external tools.

### Key Capabilities

- **Full MCP protocol support** (version 2025-11-25)
- **`#[tool]` macro** for declarative tool definition
- **`#[tool_handler]` macro** for automatic handler generation
- **Multiple transports:** stdio, SSE, streamable-http
- **MCP Resources** — expose data for agents to read
- **MCP Prompts** — guided workflows for agents
- **Pagination** support for large result sets
- **Auth support** (JWT, OAuth)

### Tool Definition Pattern

```rust
#[tool(description = "Search specs by keyword, tag, or natural language")]
fn search_specs(
    #[tool(param, description = "Search query")] query: String,
    #[tool(param, description = "Optional tag filter")] tag: Option<String>,
) -> Result<CallToolResult, McpError> {
    // Tantivy search implementation
}

#[tool(description = "Trace causal impact of changing a spec")]
fn trace_impact(
    #[tool(param, description = "Spec ID")] spec_id: String,
    #[tool(param, description = "Max depth")] depth: Option<u32>,
) -> Result<CallToolResult, McpError> {
    // DeepCausality graph traversal
}
```

### Assessment for spec-db

- **Fit:** Perfect — MCP is the standard for AI agent tool interfaces
- **Scale:** Designed for production use
- **Integration:** Tokio-based, clean macro system, official SDK
- **Ecosystem:** Any MCP-compatible AI agent can use spec-db immediately

---

## 5. Supporting Libraries

### git2 — Git Integration
- libgit2 Rust bindings for programmatic git access
- Used for: `full_rebuild` (walk tree, index all specs) and `incremental_sync` (diff since last commit)
- Enables: git-centric workflow where git is source of truth

### pulldown-cmark — Markdown Parsing
- CommonMark-compliant Markdown parser
- Used for: parsing spec body content for Tantivy indexing

### serde_yaml — YAML Frontmatter
- YAML serialization/deserialization
- Used for: parsing spec YAML frontmatter (id, title, tags, depends_on, etc.)

---

## Crate Structure (Target Architecture)

```
spec-db/
├── Cargo.toml                    # workspace
├── crates/
│   ├── spec-db-core/             # Shared types (SpecId, SpecDoc, CausalEdge, TrustLevel)
│   ├── spec-db-store/            # Fjall persistence (GraphStore: save/load graph + specs)
│   ├── spec-db-search/           # Tantivy indexing (SpecIndex: index markdown, query)
│   ├── spec-db-causal/           # DeepCausality engine (CausalEngine: in-memory graph ops)
│   ├── spec-db-ingest/           # Markdown parsing + git sync + ingestion pipeline
│   ├── spec-db-router/           # Query classification + orchestration (intent → fan-out → compose)
│   ├── spec-db-mcp/              # MCP server (rmcp) (SpecDbServer + #[tool] handlers)
│   └── spec-db-lib/              # Public Rust library API (SpecDb facade)
```

### MVP Approach (from Barry — Quick Flow)

Start with minimal crate split for speed, then decompose:

```toml
[package]
name = "spec-db"

[dependencies]
rmcp = { version = "0.8", features = ["server", "macros", "stdio"] }
tantivy = "0.22"
fjall = "2"
deep_causality = "0.13"
git2 = "0.19"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
pulldown-cmark = "0.12"
tokio = { version = "1", features = ["full"] }
```

---

## Integration Architecture

```
┌─────────────────────────────────────────────────┐
│              spec-db                              │
│                                                   │
│  ┌────────────┐    ┌──────────────────────────┐ │
│  │  Tantivy    │    │    DeepCausality          │ │
│  │  (Search)   │    │    (In-Memory Graph)      │ │
│  │             │    │         │                  │ │
│  │  Full-text  │    │    ┌────┴─────┐           │ │
│  │  BM25       │    │    │  Fjall    │           │ │
│  │  JSON       │    │    │  (KV)    │           │ │
│  └──────┬──────┘    │    └──────────┘           │ │
│         │           └──────────┬─────────────── │ │
│         │                      │                  │
│         ▼                      ▼                  │
│  ┌──────────────────────────────────────────┐   │
│  │          Query Router                     │   │
│  │   intent classify → route → compose       │   │
│  └──────────────────────┬───────────────────┘   │
│                          │                        │
│                          ▼                        │
│  ┌──────────────────────────────────────────┐   │
│  │         MCP Server (rmcp)                 │   │
│  │   tools + resources + prompts             │   │
│  │   transport: stdio / streamable-http      │   │
│  └──────────────────────────────────────────┘   │
│                                                   │
│  ┌──────────────────────────────────────────┐   │
│  │         Git Sync (git2)                    │   │
│  │   full_rebuild / incremental_sync          │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```
