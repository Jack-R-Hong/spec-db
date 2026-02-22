---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-02-23'
inputDocuments:
  - prd.md
  - product-brief-spec-db-2026-02-17.md
  - research-technical-spec-db.md
  - brainstorming-spec-db.md
  - docs/project-context.md
workflowType: 'architecture'
project_name: 'spec-db'
user_name: 'Jack'
date: '2026-02-23'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
46 functional requirements across 9 categories: Spec Discovery (5), Causal Reasoning (5), Hybrid Intelligence (3), Spec Lifecycle (6), Git Integration (6), Agent Integration/MCP (7), System Administration/CLI (6), Data Integrity (5), Observability (3). The requirements reveal two primary subsystems (search + causal reasoning) unified by a query router, with git sync and MCP serving as the integration boundaries.

**Non-Functional Requirements:**
- Performance: <10ms search, <50ms graph traversal, <5ms query routing, <1s startup, <5s full rebuild, <100ms per-spec ingestion, <100ms MCP end-to-end, <50MB memory, <30MB binary
- Reliability: Idempotent rebuilds, Fjall crash recovery, zero data lock-in (git-derivable), graceful degradation (search-only mode if graph fails), atomic rebuilds, no silent failures
- Integration: MCP protocol compliance (2025-11-25), git2 compatibility, cross-platform (Linux/macOS/Windows), stdio + streamable-http transport, OTLP export
- Security: Local-first (stdio = no network surface), token auth for HTTP, no telemetry home, filesystem scoping, no code execution
- Scalability: Hundreds of specs (100–500 target), thousands without architectural changes, single-process

**Scale & Complexity:**

- Primary domain: Developer infrastructure — CLI tool + MCP server
- Complexity level: Medium
- Estimated architectural components: 8 (core types, store, search, causal engine, ingestion/sync, query router, MCP server, CLI)

### Technical Constraints & Dependencies

- **Language:** 100% Rust — no FFI, no external services, single binary
- **Technology stack is locked:** Tantivy 0.22+, Fjall 2.x, DeepCausality 0.13+, rmcp 0.8+, git2, pulldown-cmark 0.12+, serde_yaml 0.9+, Tokio 1.x
- **Git is source of truth:** All indexes are derived and fully rebuildable — runtime data (tantivy/, fjall/) is not versioned
- **DeepCausality is highest-risk integration:** Build order places it second; petgraph is the identified fallback
- **MVP may start monolithic:** Target crate structure is 8 crates, but initial implementation could be a single crate for velocity

### Cross-Cutting Concerns Identified

- **Cross-store consistency:** Tantivy and Fjall must agree on commit SHA + doc count; startup checks, post-sync verification, auto-escalation to full rebuild on drift
- **SpecId as universal key:** Single identifier scheme used as Tantivy stored field, Fjall key, and DeepCausality node ID — must be enforced consistently across all subsystems
- **OpenTelemetry instrumentation:** Every operation (search, traversal, sync, MCP calls) must emit traces and metrics — this is woven through every component
- **Atomic operations:** Fjall cross-keyspace batches for node+edge writes; temp-dir-then-swap for full rebuilds; both sync modes must leave stores in a consistent state or not modify them at all
- **Error propagation and graceful degradation:** Errors surface clearly (no silent failures); if causal graph fails to load, system degrades to search-only with a warning
- **Git sync robustness:** Incremental sync must handle renames (git -M flag), deletions, and edge cases; doc count sanity check after every incremental sync

## Starter Template Evaluation

### Primary Technology Domain

Rust CLI tool + MCP server — developer infrastructure. No web/frontend starter templates apply. The relevant decisions are workspace structure, dependency versions, and tooling configuration.

### Version Audit (Feb 2026)

Planned versions from project-context.md have drifted. Updated locked versions:

| Crate | Version | Notes |
|-------|---------|-------|
| tantivy | 0.25.0 | Breaking from 0.22: columnar storage redesign |
| fjall | 3.0.x | Breaking from 2.x: new transaction API |
| deep_causality | 0.13.4 | Stable, matches plan |
| rmcp | 0.16.0 | Rapid evolution; pin exact version |
| git2 | 0.20.4 | Stable, security patch applied |
| pulldown-cmark | 0.13.0 | Minor breaking: event model update |
| serde_yml | latest | Replaces deprecated serde_yaml |
| tokio | 1.49.0 | Stable 1.x series |
| clap | 4.5.x | CLI framework (derive macros) |
| opentelemetry | 0.31.0 | Traces + metrics; multi-crate ecosystem |

**Migration note:** serde_yaml is deprecated by its maintainer. Replace with `serde_yml` (maintained community fork) or `yaml_serde` (YAML.org maintained). This affects spec ingestion and config parsing.

### Starter Options Considered

| Option | Description | Verdict |
|--------|-------------|---------|
| Mono-crate MVP | Single crate, refactor later | Rejected — too flat for 8 planned subsystems |
| Virtual workspace (lean start) | 3 crates initially, expand per build order | **Selected** |
| Full 8-crate workspace | All crates scaffolded day 1 | Rejected — empty crates add overhead |
| cargo-generate template | MCP server template (belyak/mcp-gen-rs-tpl-tool) | Reviewed — too simple, doesn't match workspace needs |

### Selected Starter: Virtual Workspace (Lean Start)

**Rationale:** Matches the risk-first build order from the PRD. Start with the riskiest integration (DeepCausality + Fjall), expand the workspace as each subsystem is proven. Follows the ripgrep pattern: workspace with `crates/` directory, single binary from root.

**Initialization Command:**

```bash
mkdir spec-db && cd spec-db
cargo init --name spec-db
mkdir -p crates/{spec-db-core,spec-db-causal}
cargo new --lib crates/spec-db-core
cargo new --lib crates/spec-db-causal
```

**Architectural Decisions Provided by Starter:**

**Language & Runtime:**
- Rust 2024 edition, MSRV 1.85+
- Tokio 1.x async runtime for MCP server
- No FFI, no unsafe unless absolutely necessary

**Build Tooling:**
- Cargo workspace with `workspace.dependencies` for DRY version management
- Release profile: `lto = true`, `strip = true`, `codegen-units = 1` for minimal binary
- Single binary output via `cargo install`

**Testing Framework:**
- Built-in `#[cfg(test)]` + `cargo test --workspace`
- Integration tests in `tests/` directory per crate

**Code Organization:**
```
spec-db/
├── Cargo.toml              # Workspace root + binary crate
├── Cargo.lock
├── src/
│   └── main.rs             # CLI entry point (clap)
├── crates/
│   ├── spec-db-core/       # Shared types: SpecId, SpecDoc, CausalEdge
│   └── spec-db-causal/     # DeepCausality + Fjall integration
├── specs/                  # Example spec files (shipped via init)
├── .spec-db/               # Runtime config
├── rustfmt.toml
├── clippy.toml
└── .github/workflows/ci.yml
```

**Tooling Configuration:**
- rustfmt: `max_width = 100`, `edition = "2024"`
- clippy: `warn` on `all` + `pedantic`, allow `module_name_repetitions`
- CI: GitHub Actions matrix — MSRV + stable, Linux/macOS/Windows, fmt + clippy + test

**Development Experience:**
- `cargo run -- serve` for MCP server mode
- `cargo run -- sync` for manual sync
- `cargo test --workspace` for all tests
- `cargo clippy --workspace -- -D warnings` for lint

**Note:** Project initialization using this scaffolding should be the first implementation story. The workspace expands as subsystems are built: spec-db-search (build order #3), spec-db-ingest (#4), spec-db-router (#6), spec-db-mcp (#7).

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Fjall serialization format (bincode) — needed before any persistence code
- SpecId format and validation (validated newtype) — used by every subsystem
- serde_yaml replacement (serde_yml) — needed for spec ingestion and config
- Error handling strategy (thiserror in libs, anyhow at binary) — shapes every crate's API
- DeepCausality graph model (specs-only nodes, depends_on-only edges for MVP) — defines the core data model

**Important Decisions (Shape Architecture):**
- Query router approach (keyword heuristics + explicit tools as primary)
- Logging/tracing (tracing crate + tracing-opentelemetry)
- HTTP auth (bearer token from config)
- Configuration format (YAML as specified in PRD)

**Deferred Decisions (Post-MVP):**
- Binary distribution via GitHub Releases (post-MVP when adoption warrants it)
- Additional node types (modules, interfaces) — P2
- Additional edge types (constrains, implements) — P2
- TOML config alternative — not planned

### Data Architecture

**Serialization:**
- Fjall key-value encoding: `bincode` — fast, compact, standard Rust choice for KV stores
- YAML parsing: `serde_yml` — drop-in replacement for deprecated `serde_yaml`, API-compatible
- Spec content: Markdown + YAML frontmatter parsed via `pulldown-cmark` 0.13 + `serde_yml`

**SpecId Format:**
- Newtype: `struct SpecId(String)` with validation on construction
- Pattern: `spec::{segment}::{segment}` (e.g., `spec::auth::jwt-validation`)
- Segments: lowercase alphanumeric + hyphens, no empty segments
- Enforced at ingestion boundary — invalid IDs rejected with clear error
- Used as: Tantivy STRING stored field, Fjall key, DeepCausality node identifier

**Graph Model (MVP):**
- Node types: Specs only (no modules/interfaces until P2)
- Edge types: `depends_on` only (no `constrains`/`implements` until P2)
- Edge direction: A→B means "A depends on B"
- `trace_impact(B)` traverses incoming edges — finds everything that depends on B
- `find_dependencies(A)` traverses outgoing edges — finds everything A depends on
- Trust level: All MVP edges are human-curated (trust=1.0 from frontmatter)

**Fjall Keyspace Design:**
- `nodes` keyspace: key=SpecId, value=bincode-serialized SpecNode (id, title, version, tags, owner, created)
- `edges` keyspace: key=`{from_id}:{to_id}`, value=bincode-serialized CausalEdge (type, trust)
- `meta` keyspace: key=string constants, value=system metadata (last_sync_sha, doc_count, config)
- Cross-keyspace atomic writes via Fjall batch for node+edge operations

**Tantivy Schema:**
- `id`: STRING | STORED — SpecId for retrieval
- `title`: TEXT | STORED — title-boosted search (higher relevance weight)
- `body`: TEXT — full-text searchable, not stored (retrieve from git if needed)
- `tags`: STRING | STORED — exact-match filtering
- `meta`: JSON | STORED — additional frontmatter fields (version, owner, created, depends_on)

### Authentication & Security

**Streamable-HTTP Auth:**
- Bearer token authentication when HTTP transport is enabled
- Token configured in `.spec-db/config.yaml` under `http.auth_token`
- No auth required for stdio transport (default, local-only)
- Decision rationale: Simplest viable option for an optional transport on a local dev tool

**File System Scoping:**
- Reads only from configured spec directories (default: `specs/`)
- Writes only to configured data directories (default: `data/`)
- Paths resolved relative to project root, no symlink following outside project
- No code execution — parses markdown and YAML only

### API & Communication Patterns

**Error Handling:**
- Library crates (`spec-db-core`, `spec-db-causal`, etc.): `thiserror` for typed, matchable errors
- Binary entry point (`src/main.rs`): `anyhow` for ergonomic error reporting
- MCP tool errors: Map library errors to `McpError` with human-readable messages
- No silent failures — all errors propagate with context

**Error Type Hierarchy:**
```
SpecDbError (thiserror)
├── SearchError (Tantivy failures)
├── GraphError (DeepCausality/Fjall failures)
├── SyncError (git2 failures)
├── IngestError (parsing/validation failures)
├── ConsistencyError (cross-store drift)
└── ConfigError (config loading failures)
```

**Query Router:**
- Keyword heuristic classification for the `query()` MCP tool
- Causal signals: "impact", "depends", "breaks", "affects", "upstream", "downstream"
- Search signals: everything else defaults to Tantivy full-text search
- Hybrid: If search returns results with `depends_on` edges, append causal context
- Primary path: Agents call `search_specs()` or `trace_impact()` directly — router is a convenience

**MCP Transport:**
- Default: stdio (zero network configuration, local-only)
- Optional: streamable-http (configured in `.spec-db/config.yaml`)
- Both use the same `ServerHandler` implementation — transport is a wiring concern

### Infrastructure & Deployment

**Observability:**
- `tracing` crate for structured logging and span instrumentation
- `tracing-opentelemetry` bridge for OTLP export
- `tracing-subscriber` for local console output (human-readable by default)
- OpenTelemetry export is opt-in via config — no telemetry unless configured
- Key spans: search queries, graph traversals, sync operations, MCP tool calls, consistency checks

**Binary Distribution:**
- MVP: `cargo install spec-db` — single command, no platform-specific packaging
- Post-MVP: GitHub Releases with pre-built binaries via `cargo-dist` for Linux/macOS/Windows
- Cross-platform: No platform-specific code — pure Rust, no FFI

**Configuration:**
- Format: YAML (`.spec-db/config.yaml`) as specified in PRD
- Parsed via `serde_yml`
- Sensible defaults — config file is optional for basic usage
- `spec-db init` generates config with documented defaults

### Decision Impact Analysis

**Implementation Sequence:**
1. SpecId newtype + validation (spec-db-core) — foundation for everything
2. Fjall keyspace setup + bincode serialization (spec-db-core/spec-db-causal)
3. DeepCausality graph model with depends_on edges (spec-db-causal)
4. Tantivy schema + indexing (spec-db-search, added to workspace)
5. Spec ingestion pipeline with serde_yml (spec-db-ingest, added to workspace)
6. Git sync engine (spec-db-ingest)
7. Query router with keyword heuristics (spec-db-router, added to workspace)
8. MCP server wiring with rmcp (spec-db-mcp, added to workspace)
9. CLI with clap (root binary crate)
10. OpenTelemetry instrumentation (woven through all crates)
11. Cross-store consistency checks (after both stores exist)

**Cross-Component Dependencies:**
- SpecId validation affects every subsystem — must be decided first
- bincode serialization format is shared between spec-db-core and spec-db-causal
- Error types defined in spec-db-core are used by all downstream crates
- tracing instrumentation is added incrementally as each crate is built
- Query router depends on both Tantivy and DeepCausality being functional

## Implementation Patterns & Consistency Rules

### Pattern Categories Defined

**18 potential conflict points identified** across naming (6), structure (5), format (4), and process (3) categories where AI agents could make incompatible choices.

### Naming Patterns

**N1. Module file style:** Modern style — `foo.rs` + `foo/bar.rs`. No `mod.rs` files except crate roots.

**N2. Crate naming:** Hyphens in Cargo.toml (`spec-db-core`), underscores in Rust imports (`use spec_db_core::...`). Standard Cargo behavior.

**N3. Fjall key format:** All keys are UTF-8 strings.
- Node keys: raw SpecId string (e.g., `spec::auth::jwt-validation`)
- Edge keys: `{from_id}\x00{to_id}` (null-byte separator)
- Meta keys: plain string constants (`"last_sync_sha"`, `"doc_count"`)

**N4. MCP tool/resource naming:** snake_case for tools (`search_specs`, `trace_impact`). Colon-separated scheme for resource URIs (`spec://{id}`, `graph://overview`).

**N5. Tracing span naming:** Dot-separated hierarchical — `spec_db.search.query`, `spec_db.graph.traverse`, `spec_db.sync.incremental`, `spec_db.mcp.tool_call`.

**N6. Config field naming:** snake_case for all YAML config fields. Matches Rust struct field naming via serde defaults.

### Structure Patterns

**S1. Test location:** Unit tests inline (`#[cfg(test)] mod tests`). Integration tests in `tests/` at crate root. No workspace-level `tests/` — each crate owns its tests.

**S2. Public API surface:** Each crate's `lib.rs` defines public API via explicit `pub use` re-exports. Internal modules are `pub(crate)`. No wildcard re-exports (`pub use foo::*`).

**S3. Trait-based crate interfaces:** `spec-db-core` defines traits (`SearchEngine`, `CausalGraph`, `SpecStore`). Implementation crates provide concrete types. Root binary wires them together. Enables DeepCausality→petgraph fallback without changing downstream code.

**S4. Shared type location:** All domain types (`SpecId`, `SpecDoc`, `CausalEdge`, `SpecNode`, error types) live in `spec-db-core`. No other crate defines domain types. Implementation-specific types stay private to their crate.

**S5. Module depth:** Maximum 2 levels within a crate (`crate::module::submodule`). Deeper nesting signals the crate needs splitting.

### Format Patterns

**F1. MCP tool response format:** All tools return JSON-serialized content within `CallToolResult`.
- Search results: `[{id, title, score, snippet}]`
- Graph results: `{node, edges: [{from, to, type}]}`
- Admin results: `{status, message, details}`

**F2. MCP error format:** Consistent shape across all tools: `{error_type: "SearchError|GraphError|...", message: "human-readable", context: {optional_detail}}`.

**F3. Spec frontmatter handling:** Only defined fields are indexed: `id`, `title`, `version`, `tags`, `depends_on`, `owner`, `created`. Unknown fields preserved in `meta` JSON but not given special treatment.

**F4. Log output format:** Human-readable by default (`tracing-subscriber` fmt layer). Structured JSON when OpenTelemetry export is configured. Config selects one mode — never both.

### Process Patterns

**P1. Initialization failure handling:** Fail-fast on critical failures (config not found, data directory not writable). Degrade gracefully on non-critical failures (graph load fails → search-only mode with warning on stderr). Never silently swallow errors.

**P2. Sync operation atomicity:** Full rebuild uses temp-dir-then-swap (build complete index, verify, atomically rename). Incremental sync uses Fjall batches. If incremental fails partway, partial state is rolled back and full rebuild auto-triggers.

**P3. Async vs sync boundaries:** MCP server layer is async (Tokio). Everything below is synchronous — Tantivy search, Fjall operations, DeepCausality graph ops. Async boundary is at MCP tool handler level: `tokio::task::spawn_blocking` wraps sync operations.

### Enforcement Guidelines

**All AI Agents MUST:**
- Import domain types from `spec-db-core` only — never redefine `SpecId`, `SpecDoc`, etc.
- Use trait interfaces from `spec-db-core` when referencing other subsystems — never depend on concrete implementations across crate boundaries
- Return JSON-serialized content from MCP tools — never plain text
- Instrument all public functions with `tracing` spans following the dot-separated naming convention
- Handle errors via `thiserror` in library code and propagate with `?` — never `unwrap()` or `expect()` in library code (test code is exempt)
- Keep async boundary at the MCP handler level — never make Tantivy/Fjall/DeepCausality calls async

**Pattern Enforcement:**
- `cargo clippy --workspace -- -D warnings` catches naming and style violations
- CI runs `cargo fmt --all -- --check` to enforce formatting
- Crate-level `pub` API review: `lib.rs` must be the sole public surface
- Integration tests verify MCP tool responses match the defined JSON schemas

### Anti-Patterns

**NEVER:**
- `unwrap()` or `expect()` in library crate code (use `?` with typed errors)
- `pub use foo::*` wildcard re-exports (explicit re-exports only)
- Define domain types outside `spec-db-core`
- Make Tantivy/Fjall/DeepCausality calls from async context without `spawn_blocking`
- Return unstructured text from MCP tools (always JSON)
- Use `mod.rs` for module files (modern file naming only)
- Nest modules deeper than 2 levels within a crate
- Add `#[allow(clippy::...)]` without a comment explaining why

## Project Structure & Boundaries

### Complete Project Directory Structure

```
spec-db/
├── Cargo.toml                        # Workspace root + binary crate
├── Cargo.lock
├── LICENSE
├── README.md
├── rustfmt.toml
├── clippy.toml
├── .gitignore
├── .github/
│   └── workflows/
│       └── ci.yml
│
├── src/
│   └── main.rs                       # CLI entry: init | serve | sync | rebuild | status
│
├── crates/
│   ├── core/                         # [Build Order #1] Shared types, traits, errors
│   │   ├── Cargo.toml                # name = "spec-db-core"
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs              # SpecId, SpecDoc, SpecNode, CausalEdge, TrustLevel
│   │       ├── traits.rs             # SearchEngine, CausalGraph, SpecStore traits
│   │       └── error.rs              # SpecDbError hierarchy (thiserror)
│   │
│   ├── causal/                       # [Build Order #2] DeepCausality + Fjall
│   │   ├── Cargo.toml                # name = "spec-db-causal"
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── engine.rs             # In-memory graph: load, add_node, add_edge
│   │   │   ├── store.rs              # Fjall keyspaces: nodes, edges, meta + bincode serde
│   │   │   └── traversal.rs          # trace_impact (downstream), find_dependencies (upstream)
│   │   └── tests/
│   │       └── integration.rs
│   │
│   ├── search/                       # [Build Order #3] Tantivy indexing + search
│   │   ├── Cargo.toml                # name = "spec-db-search"
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs             # Tantivy schema: id, title, body, tags, meta
│   │   │   ├── indexer.rs            # add_doc, remove_doc, commit
│   │   │   └── query.rs              # BM25 search, title boost, tag filter
│   │   └── tests/
│   │       └── integration.rs
│   │
│   ├── ingest/                       # [Build Order #4-5] Parsing + git sync
│   │   ├── Cargo.toml                # name = "spec-db-ingest"
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs             # Markdown + YAML frontmatter parsing
│   │   │   ├── validate.rs           # SpecId format validation, frontmatter completeness
│   │   │   ├── sync.rs               # full_rebuild (tree walk) + incremental (git diff -M)
│   │   │   └── consistency.rs        # Cross-store SHA + doc count checks
│   │   └── tests/
│   │       ├── fixtures/
│   │       │   ├── valid_spec.md
│   │       │   ├── invalid_id.md
│   │       │   ├── missing_fields.md
│   │       │   └── multi_depends.md
│   │       └── integration.rs
│   │
│   ├── router/                       # [Build Order #6] Query classification + composition
│   │   ├── Cargo.toml                # name = "spec-db-router"
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── classifier.rs         # Keyword heuristic intent classification
│   │   │   └── composer.rs           # Fan-out to search/causal, compose response
│   │   └── tests/
│   │       └── integration.rs
│   │
│   └── mcp/                          # [Build Order #7] MCP server (rmcp)
│       ├── Cargo.toml                # name = "spec-db-mcp"
│       ├── src/
│       │   ├── lib.rs
│       │   ├── server.rs             # ServerHandler impl, tool_router
│       │   ├── tools.rs              # #[tool] macros: search_specs, get_spec, trace_impact, etc.
│       │   └── resources.rs          # Resource handlers: spec://{id}, graph://overview
│       └── tests/
│           └── integration.rs
│
├── specs/                            # Example specs shipped via `spec-db init`
│   └── example/
│       ├── hello-world.md
│       └── getting-started.md
│
├── data/                             # Runtime data — NOT in git
│   ├── tantivy/
│   └── fjall/
│
├── .spec-db/
│   └── config.yaml
│
└── docs/
    └── project-context.md
```

### Architectural Boundaries

**Crate Dependency Graph (unidirectional — no cycles):**
```
spec-db (binary)
├── mcp            → tools/resources, ServerHandler
│   ├── router     → query classification, result composition
│   │   ├── search → Tantivy search execution
│   │   │   └── core
│   │   └── causal → DeepCausality graph operations
│   │       └── core
│   └── ingest     → sync trigger from MCP
│       ├── search
│       ├── causal
│       └── core
└── core           → shared types (everyone depends on this)
```

**Trait Boundaries (defined in core):**

| Trait | Implemented By | Purpose |
|-------|---------------|---------|
| `SearchEngine` | `search::SearchIndex` | Full-text search operations |
| `CausalGraph` | `causal::CausalEngine` | Graph traversal operations |
| `SpecStore` | `causal::FjallStore` | Persistent key-value operations |

**Data Boundaries:**

| Boundary | Owns | Accessed By |
|----------|------|-------------|
| Tantivy index (`data/tantivy/`) | `search` crate | Only `search` reads/writes |
| Fjall database (`data/fjall/`) | `causal` crate | Only `causal` reads/writes |
| Git repository | `ingest` crate | Only `ingest` reads via git2 |
| Config (`.spec-db/config.yaml`) | Root binary | Parsed once at startup, passed as config struct |

**Async Boundary:**

| Layer | Runtime | Pattern |
|-------|---------|---------|
| `mcp` (MCP handlers) | Async (Tokio) | `#[tool]` async handlers |
| `router` | Sync | Called via `spawn_blocking` |
| `search` | Sync | Tantivy is synchronous |
| `causal` | Sync | DeepCausality + Fjall are synchronous |
| `ingest` | Sync | git2 + parsing are synchronous |

### Requirements to Structure Mapping

| FR Category | Crate | Key Files |
|-------------|-------|-----------|
| Spec Discovery (FR1-5) | `search` | `query.rs`, `schema.rs` |
| Causal Reasoning (FR6-10) | `causal` | `traversal.rs`, `engine.rs` |
| Hybrid Intelligence (FR11-13) | `router` | `classifier.rs`, `composer.rs` |
| Spec Lifecycle (FR14-19) | `ingest` | `parser.rs`, `validate.rs` |
| Git Integration (FR20-25) | `ingest` | `sync.rs` |
| Agent Integration (FR26-32) | `mcp` | `tools.rs`, `resources.rs` |
| System Administration (FR33-38) | Root binary + `ingest` | `main.rs`, `sync.rs` |
| Data Integrity (FR39-43) | `ingest` | `consistency.rs` |
| Observability (FR44-46) | All crates | `tracing` spans in every public fn |

**Cross-Cutting Concerns:**

| Concern | Location | Access Pattern |
|---------|----------|---------------|
| SpecId validation | `core::types` | All crates import from core |
| Error types | `core::error` | All crates use thiserror types |
| Trait interfaces | `core::traits` | Impl crates implement; consumers use traits |
| Tracing/OTel | Each crate instruments own spans | Config wired at binary level |
| Cross-store consistency | `ingest::consistency` | Called after sync, at startup |

### Data Flow

**Search:** Agent → `mcp::tools` → `spawn_blocking` → `search::SearchIndex::search()` → Tantivy → JSON → Agent

**Causal:** Agent → `mcp::tools` → `spawn_blocking` → `causal::CausalEngine::trace_impact()` → in-memory graph → JSON → Agent

**Hybrid:** Agent → `mcp::tools` → `spawn_blocking` → `router::QueryRouter::route()` → classifier → fan-out → composer → JSON → Agent

**Sync:** CLI/Agent → `ingest::GitSync` → git2 diff/walk → `parser` → `validate` → update `search` + `causal` → `consistency` check → report

**Startup:** `main.rs` → parse config → open Fjall + load graph (degrade if fails) → open Tantivy → consistency check (auto-rebuild if drift) → start MCP server → ready

### Workspace Expansion Plan

| Phase | Crates in Workspace | What's New |
|-------|-------------------|------------|
| 1 | `core`, `causal` | Shared types + riskiest integration |
| 2 | + `search` | Tantivy indexing |
| 3 | + `ingest` | Spec parsing + git sync |
| 4 | + `router` | Query classification |
| 5 | + `mcp` | MCP server wiring |
| 6 | Root binary complete | CLI + full startup flow |

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:** All technology choices (Tantivy 0.25, Fjall 3.0, DeepCausality 0.13.4, rmcp 0.16, Tokio 1.x) are pure Rust with no known conflicts. bincode serialization works with serde derive on all domain types. serde_yml is a drop-in replacement for deprecated serde_yaml. No contradictory decisions found.

**Pattern Consistency:** snake_case naming is used consistently across Rust code, YAML config, MCP tool names, and tracing spans. Modern module file style is uniform. Error handling pattern (thiserror in libs, anyhow at binary) is coherent. Async boundary is cleanly defined at the MCP handler layer.

**Structure Alignment:** Crate dependency graph is unidirectional with no cycles. Each data store (Tantivy, Fjall, git) has a single owner crate. Trait boundaries in `core` cleanly separate interface from implementation, enabling the documented DeepCausality→petgraph fallback path.

### Requirements Coverage Validation ✅

**Functional Requirements:** All 46 FRs across 9 categories are architecturally supported and mapped to specific crates and files. No orphaned requirements.

**Non-Functional Requirements:**
- Performance: Tantivy provides <10ms search at this scale. In-memory DeepCausality graph provides <50ms traversal. Fjall startup is <1s for hundreds of nodes. Full rebuild <5s is achievable with git tree walk + parallel indexing.
- Reliability: Fjall LSM-tree crash recovery, atomic rebuilds via temp-dir-then-swap, graceful degradation to search-only mode, full rebuild from git as ultimate recovery.
- Integration: MCP protocol compliance via rmcp, git2 compatibility, cross-platform pure Rust, stdio + streamable-http transports, OTLP export via tracing-opentelemetry.
- Security: Local-first (stdio = no network surface), bearer token for optional HTTP, filesystem scoping, no code execution.
- Scalability: Hundreds of specs (in-memory graph is single-digit MB), single-process.

### Implementation Readiness Validation ✅

**Decision Completeness:** All critical decisions are documented with specific crate versions and rationale. Implementation sequence is defined (11 steps). Cross-component dependencies are mapped.

**Structure Completeness:** Complete project directory tree with every file defined and annotated. All crates have explicit source files mapped to FR categories.

**Pattern Completeness:** 18 potential conflict points identified and resolved. Naming, structure, format, and process patterns are comprehensive. Enforcement guidelines and anti-patterns are documented.

### Gap Analysis Results

**Critical Gaps:** None identified.

**Important Gaps (implementation-phase risks):**
1. rmcp 0.16 API — rapid evolution from 0.8 means macro API may differ from documented examples. Pin exact version, verify in first MCP story. Trait boundaries isolate blast radius.
2. DeepCausality graph mapping — exact mapping from context-hyper-graph to simple spec graph needs verification during build order #2. petgraph fallback is documented.
3. Fjall v3 batch API — new transaction model in v3 needs syntax verification. First integration test validates roundtrip.

**Nice-to-Have Gaps (deferred):**
- Benchmarking strategy for performance NFR verification
- P2 migration story (AI-inferred edges, trust scoring)
- Documentation strategy beyond README

### Architecture Completeness Checklist

**✅ Requirements Analysis**
- [x] Project context thoroughly analyzed (46 FRs, 5 NFR categories)
- [x] Scale and complexity assessed (medium, developer infrastructure)
- [x] Technical constraints identified (100% Rust, locked stack, git source of truth)
- [x] Cross-cutting concerns mapped (consistency, SpecId, OTel, atomicity, errors, sync)

**✅ Architectural Decisions**
- [x] Critical decisions documented with versions (11 decisions)
- [x] Technology stack fully specified (10 crates with exact versions)
- [x] Integration patterns defined (trait boundaries, data flow, async boundary)
- [x] Performance considerations addressed (in-memory graph, Tantivy BM25, Fjall LSM)

**✅ Implementation Patterns**
- [x] Naming conventions established (6 patterns)
- [x] Structure patterns defined (5 patterns)
- [x] Format patterns specified (4 patterns)
- [x] Process patterns documented (3 patterns)

**✅ Project Structure**
- [x] Complete directory structure defined (7 crates, all files annotated)
- [x] Component boundaries established (trait-based, unidirectional deps)
- [x] Integration points mapped (data flow for 5 primary operations)
- [x] Requirements to structure mapping complete (46 FRs → crate + file)

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High — all requirements covered, no critical gaps, trait boundaries contain risk.

**Key Strengths:**
- Trait-based crate interfaces enable the DeepCausality→petgraph fallback without changing downstream code
- Risk-first build order tackles the hardest integration (DeepCausality + Fjall) first
- Git as source of truth means all derived state is fully rebuildable — zero lock-in
- 18 consistency patterns prevent AI agent implementation conflicts
- Clean async boundary (MCP = async, everything else = sync) avoids colored function problems

**Areas for Future Enhancement:**
- Benchmarking harness for NFR verification (can be added as a story)
- P2 architecture decisions (AI-inferred edges, trust scoring, CSM validation)
- Multi-repo federation architecture (P3)
- Embeddable library API design (P3)

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all crates
- Respect crate boundaries — never import concrete types across crate boundaries, use traits
- Import all domain types from `spec-db-core` — never redefine
- Refer to this document for all architectural questions

**First Implementation Priority:**
1. Scaffold workspace: `cargo init` + `crates/core/` + `crates/causal/`
2. Define `SpecId`, `SpecDoc`, `CausalEdge` types in `core`
3. Define `SearchEngine`, `CausalGraph`, `SpecStore` traits in `core`
4. Define `SpecDbError` hierarchy in `core`
5. Begin DeepCausality + Fjall integration in `causal` (build order #2 — riskiest piece)
