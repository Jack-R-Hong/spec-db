---
stepsCompleted:
  - step-01-init
  - step-02-context
  - step-03-starter
  - step-04-decisions
  - step-05-patterns
  - step-06-structure
  - step-07-validation
  - step-08-complete
  - step-04-decisions
  - step-05-patterns
  - step-06-structure
  - step-04-decisions
  - step-05-patterns
  - step-04-decisions
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - docs/project-context.md
  - docs/architecture-backend.md
  - docs/integration-architecture.md
  - docs/project-overview.md
  - docs/development-guide-backend.md
  - docs/source-tree-analysis.md
workflowType: 'architecture'
project_name: 'lattice'
user_name: 'Jack'
date: '2026-02-27'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
- 33 FRs organized into 7 capability areas
- Core capabilities: Backend management, Agent routing, Search modes, Embedding, Configuration
- All interfaces (MCP/REST/CLI) require new operations for backend management

**Non-Functional Requirements:**
- Performance: <50ms FTS, <100ms vector, <150ms hybrid (p95 for 1000 specs)
- Reliability: Backend failure isolation, graceful degradation to FTS
- Compatibility: Backward compatible, existing APIs unchanged

**Scale & Complexity:**
- Primary domain: Backend/Infrastructure (Rust)
- Complexity level: Medium-High
- Estimated new crates: 2-3 (search-vector, embedding)

### Technical Constraints & Dependencies

**From Project Context (MUST follow):**
- Git remains source of truth — embeddings are derived, rebuildable
- Async boundary pattern — vector operations use `spawn_blocking`
- Trait-based interfaces — extend with `VectorSearchBackend`
- MCP as primary API — new tools follow existing patterns
- Rust 100% — no external language dependencies

**Existing Architecture Patterns:**
- Service-oriented modular monolith (7 crates)
- Clean crate boundaries with trait abstractions
- Router dispatches to subsystems (search, causal, ingest)

### Cross-Cutting Concerns Identified

| Concern | Impact | Resolution Approach |
|---------|--------|---------------------|
| Score normalization | All backends return different ranges | Normalize to 0-1 |
| Filter translation | Unified syntax needed | Translate to backend-specific |
| Embedding lifecycle | Generation, storage, rebuild | Provider abstraction |
| Error handling | New failure modes | Extend SpecDbError enum |
| Config validation | Backend configs | Validate at startup |

## Technology Foundation

### Existing Stack (Locked)

| Layer | Technology | Version |
|-------|------------|---------|
| Language | Rust | 1.85+ |
| Search (FTS) | Tantivy | 0.22+ |
| KV Storage | Fjall | 2.x |
| Causal Engine | DeepCausality | 0.13+ |
| MCP Protocol | rmcp | 0.8+ |
| Async Runtime | Tokio | 1.x |

### New Components Required

| Component | Purpose | MVP Choice |
|-----------|---------|------------|
| Vector DB | Semantic search | LanceDB (embedded) |
| Local Embedding | Generate vectors | fastembed-rs |
| Remote Embedding | OpenAI API | async-openai |

### Architectural Patterns to Follow

- **Crate boundaries**: New functionality in new crates (`search-vector`, `embedding`)
- **Trait abstraction**: `VectorSearchBackend` trait parallel to `SearchEngine`
- **Async boundary**: Vector operations via `spawn_blocking`
- **Error handling**: Extend `SpecDbError` enum
- **Configuration**: Extend `.lattice/config.yaml` schema

## Core Architectural Decisions

### Decision 1: VectorSearchBackend Trait Design

**Decision:** New independent `VectorSearchBackend` trait + add `search_scored()` to existing `SearchEngine`

**Rationale:**
- Current `SearchEngine::search()` returns `Vec<SpecId>` (no scores)
- Hybrid search requires scores from both FTS and vector backends for merging
- Adding `search_scored()` as a default method preserves backward compatibility
- `VectorSearchBackend` is a separate concern (embeddings, vector similarity) — clean separation

**Trait Signatures:**

```rust
// New type in core
pub struct ScoredHit {
    pub id: SpecId,
    pub score: f32,
}

// New trait in core
pub trait VectorSearchBackend: Send + Sync {
    fn index_spec(&mut self, doc: &SpecDoc, embedding: &[f32]) -> Result<(), SpecDbError>;
    fn remove_spec(&mut self, id: &SpecId) -> Result<(), SpecDbError>;
    fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>;
    fn search_with_tags(
        &self,
        embedding: &[f32],
        tags: &[String],
        limit: usize,
    ) -> Result<Vec<ScoredHit>, SpecDbError>;
}

// Extension to existing trait (backward compatible)
pub trait SearchEngine {
    // ... existing methods unchanged ...
    fn search_scored(&self, query: &str, limit: usize) -> Result<Vec<ScoredHit>, SpecDbError> {
        // Default: delegates to search(), returns score 0.0
        self.search(query, limit).map(|ids| ids.into_iter().map(|id| ScoredHit { id, score: 0.0 }).collect())
    }
}
```

**Affects:** core, search, search-vector, router, composer

### Decision 2: Agent Routing Architecture

**Decision:** Backend Registry with config-driven agent routing

**Rationale:**
- PRD requires multi-store coexistence (multiple backends simultaneously)
- Journey 2 requires runtime backend addition via API
- Registry pattern supports both config-driven and API-driven backend management

**Architecture:**

```rust
pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn VectorSearchBackend>>,
    routing_rules: Vec<RoutingRule>,
    default_backend: String,
}

pub struct RoutingRule {
    pub agent_pattern: String,   // glob pattern, e.g. "doc-*"
    pub backend: String,          // backend name
}

impl BackendRegistry {
    pub fn resolve(&self, agent_context: Option<&str>) -> &dyn VectorSearchBackend { ... }
    pub fn add_backend(&mut self, name: String, backend: Box<dyn VectorSearchBackend>) { ... }
    pub fn remove_backend(&mut self, name: &str) -> Result<(), SpecDbError> { ... }
}
```

**Router Extension:**
```rust
pub struct QueryRouter<S: SearchEngine, C: CausalGraph> {
    search: S,           // FTS (always present)
    graph: C,            // Causal (always present)
    vector: Option<BackendRegistry>,  // Vector backends (optional)
}
```

**Affects:** core (types), search-vector (registry), router (dispatch), mcp (agent context)

### Decision 3: Embedding Provider Architecture

**Decision:** Independent `EmbeddingProvider` trait in core, implementations in dedicated `embedding` crate

**Rationale:**
- Embedding generation is orthogonal to vector storage
- Same embedding provider can serve multiple vector backends
- Testable independently (mock embedding for vector backend tests)

**Trait Signature:**

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, SpecDbError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SpecDbError>;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}
```

**Implementations:**
- `LocalEmbedding` — wraps fastembed-rs, sync, runs in `spawn_blocking`
- `RemoteEmbedding` — wraps async-openai, async with retry

**Affects:** core (trait), embedding (crate), ingest (uses for indexing), search-vector (receives pre-computed embeddings)

### Decision 4: Hybrid Search Merge Strategy

**Decision:** Reciprocal Rank Fusion (RRF)

**Rationale:**
- Industry standard used by Elasticsearch, Vespa, Weaviate
- Score-distribution agnostic — no need to understand BM25 vs cosine ranges
- Stable for small result sets (unlike min-max normalization)
- Simple, deterministic formula: `score = Σ 1/(k + rank_i)`, k=60

**Implementation:**

```rust
pub fn reciprocal_rank_fusion(
    fts_results: &[ScoredHit],
    vector_results: &[ScoredHit],
    k: u32,  // default 60
) -> Vec<ScoredHit> {
    // 1. Build rank maps from each result list
    // 2. For each unique id, sum 1/(k + rank) across lists
    // 3. Sort by fused score descending
    // 4. Return merged list
}
```

**Affects:** router (composer module), search-vector (score handling)

### Decision 5: Crate Organization

**Decision:** Add 2 new crates to existing workspace

```
crates/
  core/           # Existing — add VectorSearchBackend, EmbeddingProvider, ScoredHit
  search/         # Existing Tantivy — add search_scored() implementation
  search-vector/  # NEW — LanceDB impl, BackendRegistry, score fusion
  embedding/      # NEW — fastembed + openai providers
  causal/         # Existing — unchanged
  ingest/         # Existing — extend to generate embeddings during sync
  router/         # Existing — extend with agent context routing
  mcp/            # Existing — add configure_backend, list_backends tools
  web/            # Existing — add /api/backends endpoints
```

**Dependency Graph Extension:**
```
  mcp / web / router
       |
  search-vector  ←  NEW
       |
  embedding      ←  NEW
       |
     core
```

### Decision 6: Filter Strategy

**Decision:** MVP supports tag filtering only (consistent with existing SearchEngine)

**Rationale:**
- Existing `search_with_tags()` pattern is well-established
- Tags cover the primary filtering use case for spec search
- Avoids filter translation complexity in MVP
- Post-MVP: introduce `Filter` enum with `Eq`, `In`, `Range` operators

**Affects:** search-vector (tag filter translation to LanceDB SQL WHERE clause)

### Decision Impact Summary

| Decision | Implementation Sequence | Risk |
|----------|------------------------|------|
| VectorSearchBackend trait | First — blocks everything | Low |
| EmbeddingProvider trait | Second — blocks indexing | Low |
| Backend Registry | Third — blocks routing | Medium |
| LanceDB implementation | Fourth — first concrete backend | Medium |
| RRF hybrid merge | Fifth — enables hybrid search | Low |
| Router agent context | Sixth — enables agent isolation | Low |

## Implementation Patterns & Consistency Rules

### Naming Patterns

| Category | Convention | Example |
|----------|-----------|---------|
| Crate names | `spec-db-{module}` | `spec-db-search-vector`, `spec-db-embedding` |
| Module names | snake_case | `backend_registry`, `score_fusion` |
| Trait names | PascalCase, capability-based | `VectorSearchBackend`, `EmbeddingProvider` |
| Struct names | PascalCase | `BackendRegistry`, `ScoredHit` |
| Function names | snake_case, verb-first | `search_scored()`, `resolve_backend()` |
| Error variants | PascalCase(String) | `SpecDbError::VectorError(String)` |
| Config keys | snake_case YAML | `search_backends`, `default_backend` |
| Feature flags | kebab-case | `vector-search`, `embedding-local` |

### Structure Patterns

```
crates/search-vector/
  src/
    lib.rs              # Public API: re-exports
    backend.rs          # VectorSearchBackend trait impls
    registry.rs         # BackendRegistry
    lancedb.rs          # LanceDB-specific implementation
    fusion.rs           # RRF score fusion
    config.rs           # Backend config types
  tests/                # Integration tests

crates/embedding/
  src/
    lib.rs              # Public API
    provider.rs         # EmbeddingProvider trait impls
    local.rs            # fastembed wrapper
    remote.rs           # OpenAI wrapper
    config.rs           # Embedding config types
  tests/
```

**Rules:**
- Tests in `tests/` directory (follows existing lattice pattern)
- Unit tests use `#[cfg(test)] mod tests` in same file
- One public type per file (unless strongly related)
- `lib.rs` only does re-exports, no logic

### Error Handling Pattern

```rust
#[derive(thiserror::Error, Debug)]
pub enum SpecDbError {
    // ... existing variants ...
    #[error("vector error: {0}")]
    VectorError(String),
    #[error("embedding error: {0}")]
    EmbeddingError(String),
    #[error("backend not found: {0}")]
    BackendNotFound(String),
    #[error("routing error: {0}")]
    RoutingError(String),
}
```

**Rules:**
- Always use `SpecDbError` — do not introduce new error enums
- Error messages lowercase, no trailing period
- Use `map_err` to convert third-party errors (LanceDB, fastembed)

### Async Boundary Pattern

```rust
// All vector/embedding operations use spawn_blocking
// Follows existing Tantivy/Fjall pattern
let result = tokio::task::spawn_blocking(move || {
    registry.resolve(agent_ctx)?.search(&embedding, limit)
}).await??;
```

**Rules:**
- EmbeddingProvider is sync (fastembed is sync)
- VectorSearchBackend is sync (LanceDB Rust SDK is sync)
- All async conversion happens at MCP/web handler layer via `spawn_blocking`

### API Patterns

**MCP Tools:**
```rust
// Tool naming: snake_case verbs
#[tool(name = "configure_backend")]
async fn configure_backend(name: String, backend_type: String, config: String) -> Result<...>

#[tool(name = "list_backends")]
async fn list_backends() -> Result<...>
```

**REST Endpoints:**
```rust
// Endpoint naming: plural nouns, RESTful
Router::new()
    .route("/api/backends", get(list_backends).post(create_backend))
    .route("/api/backends/:name", put(update_backend).delete(delete_backend))
    .route("/api/backends/:name/status", get(backend_status))
```

**CLI Commands:**
```rust
#[derive(Subcommand)]
enum BackendCmd {
    List,
    Add { name: String, backend_type: String },
    Remove { name: String },
    Status { name: Option<String> },
}
```

### Configuration Pattern

```yaml
# Extension to .lattice/config.yaml
search_backends:
  default: tantivy
  routing:
    - agent: "doc-agent"
      backend: "lancedb-private"
    - agent: "*"
      backend: "tantivy"
  backends:
    - name: tantivy
      type: fts
    - name: lancedb-private
      type: lancedb
      path: ./data/lancedb
      embedding:
        provider: local
        model: all-MiniLM-L6-v2
```

**Rules:**
- Config uses `serde` deserialize to typed structs
- Missing `search_backends` section → behavior unchanged (backward compatible)
- Config validation at startup, not lazy

### Testing Patterns

| Test Type | Location | Convention |
|-----------|----------|------------|
| Unit tests | Same file `#[cfg(test)]` | Test with mock backends |
| Integration tests | `crates/*/tests/` | Test actual LanceDB/fastembed with temp dirs |
| Acceptance tests | `tests/` (workspace root) | End-to-end MCP tool calls |

**Rules:**
- Use `tempfile::TempDir` for test data isolation
- Mock `EmbeddingProvider` for vector backend tests (avoid model downloads)
- Integration tests use `#[ignore]` if requiring external resources

## Project Structure & Boundaries

### FR → Crate Mapping

| FR Category | Primary Crate | Supporting Crates |
|-------------|--------------|-------------------|
| Backend Management (FR1-7) | `search-vector` (registry) | `core`, `mcp`/`web`/CLI |
| Agent Routing (FR8-11) | `router` (dispatch) | `search-vector`, `core` |
| Search: FTS (FR12, 16) | `search` (existing) | `router` |
| Search: Vector (FR13, 15, 17) | `search-vector` | `embedding`, `router` |
| Search: Hybrid (FR14) | `search-vector` (fusion) | `search` + `router` |
| Indexing (FR18-22) | `ingest` (orchestrate) | `embedding`, `search-vector` |
| Configuration (FR23-26) | `core` (config types) | All crates consume config |
| Migration (FR27-29) | `src/` (CLI binary) | `core` (config migration) |
| MCP Interface (FR30) | `mcp` | All subsystems |
| REST Interface (FR31) | `web` | All subsystems |
| CLI Interface (FR32) | `src/` (binary) | All subsystems |
| Library API (FR33) | `core` (public traits) | — |

### New Crate Structure

```
crates/search-vector/          # NEW
  Cargo.toml
  src/
    lib.rs                      # Re-exports
    backend.rs                  # VectorSearchBackend trait impls
    registry.rs                 # BackendRegistry: resolve, add, remove
    lancedb.rs                  # LanceDB implementation
    fusion.rs                   # RRF score fusion
    config.rs                   # Backend config deserialization
  tests/
    lancedb_test.rs
    registry_test.rs
    fusion_test.rs

crates/embedding/              # NEW
  Cargo.toml
  src/
    lib.rs                      # Re-exports
    provider.rs                 # EmbeddingProvider dispatch
    local.rs                    # fastembed wrapper
    remote.rs                   # OpenAI API wrapper
    config.rs                   # Embedding config types
  tests/
    local_test.rs
    remote_test.rs
```

### Modified Existing Crates

| Crate | Changes |
|-------|---------|
| `core` | Add ScoredHit, RoutingRule, BackendConfig types; VectorSearchBackend, EmbeddingProvider traits; new error variants |
| `search` | Add `search_scored()` implementation exposing BM25 scores |
| `ingest` | Extend sync pipeline: parse → embed → index to FTS + vector |
| `router` | Add BackendRegistry field, agent context routing, Vector/HybridVector intents |
| `mcp` | Add `configure_backend`, `list_backends` tools; extend `search_specs` |
| `web` | Add `/api/backends` CRUD + status endpoints |
| `src/` (binary) | Add `backend` CLI subcommand |

### Architectural Boundaries

```
┌─────────────────────────────────────────────────────────┐
│                    Interface Layer                      │
│  MCP  |  Web (Axum)  |  CLI (clap)                      │
│       pass agent_context: Option<String>                 │
├─────────────────────────────────────────────────────────┤
│                   Routing Layer                          │
│  QueryRouter { search, graph, vector: BackendRegistry }  │
├─────────────────────────────────────────────────────────┤
│                  Subsystem Layer                         │
│  Search(Tantivy)  |  Causal(Fjall)  |  BackendRegistry   │
│                                        └─ LanceDB        │
│                                        └─ Qdrant (post)  │
├─────────────────────────────────────────────────────────┤
│                 Embedding Layer                          │
│  Local (fastembed)  |  Remote (OpenAI)                   │
├─────────────────────────────────────────────────────────┤
│                    Core Layer                            │
│  Types, Traits, Errors, Config                           │
└─────────────────────────────────────────────────────────┘
```

### Boundary Rules

| Boundary | Rule |
|----------|------|
| Interface → Router | Pass `agent_context: Option<String>` from MCP/REST headers |
| Router → Backend | Router resolves backend via registry, then calls trait methods |
| Backend → Embedding | Backends do NOT call embedding; ingest layer pre-computes |
| Ingest → All backends | Ingest orchestrates: parse → embed → index to FTS + vector |
| Core → Everything | Core defines traits and types; all crates depend on core |
| search-vector → core | search-vector depends on core only, NOT on search (Tantivy) |

### Crate Dependency Matrix

```
                core  search  causal  ingest  search-vector  embedding  router  mcp  web
core             -
search           ✓      -
causal           ✓             -
ingest           ✓      ✓      ✓       -        ✓             ✓
search-vector    ✓                               -             ✓
embedding        ✓                                              -
router           ✓      ✓      ✓                ✓                         -
mcp              ✓      ✓      ✓       ✓        ✓                        ✓     -
web              ✓      ✓      ✓       ✓        ✓                        ✓          -
```

## Architecture Validation Results

### Coherence Validation ✅

| Check | Result |
|-------|--------|
| VectorSearchBackend trait + BackendRegistry | ✅ trait objects via `Box<dyn VectorSearchBackend>` |
| EmbeddingProvider separate from backend | ✅ Ingest orchestrates, backends receive pre-computed embeddings |
| RRF hybrid merge + ScoredHit types | ✅ Both FTS and vector return ScoredHit, fusion uniform |
| `spawn_blocking` async pattern + sync traits | ✅ All new traits sync, matching existing pattern |
| LanceDB embedded + feature flag | ✅ `vector-search` feature makes it optional |
| No dependency cycles in crate graph | ✅ Verified |

### Requirements Coverage ✅

**Functional Requirements:** 33/33 FRs covered
- Backend Management (FR1-7): BackendRegistry + config + interfaces
- Agent Routing (FR8-11): RoutingRule + BackendRegistry.resolve()
- Search Capabilities (FR12-17): SearchEngine + VectorSearchBackend + RRF
- Indexing (FR18-22): Ingest → Embedding → VectorSearchBackend
- Configuration (FR23-26): SearchBackendsConfig in core
- Migration (FR27-29): Feature flag + config fallback
- Interfaces (FR30-33): MCP/REST/CLI/Trait patterns defined

**Non-Functional Requirements:** 19/19 NFRs covered
- Performance (NFR1-6): spawn_blocking, embedded LanceDB, batch embedding
- Security (NFR7-9): Config types, input validation at trait boundary
- Reliability (NFR10-13): Backend isolation, graceful degradation, per-backend rebuild
- Integration (NFR14-16): Trait stability, MCP compliance
- Compatibility (NFR17-19): Feature flags, backward compatible config

### Gap Analysis

**No critical gaps found.**

Minor gaps (addressable during implementation):
- LanceDB Rust SDK version: pin during first implementation story
- fastembed model list: document in config reference
- Agent context propagation: MCP tool parameter `agent_id: Option<String>`
- Graceful degradation flow: Router falls back to FTS when vector unavailable

### Architecture Completeness Checklist

- [x] Requirements Analysis — 33 FRs mapped, 19 NFRs covered
- [x] Architectural Decisions — 6 decisions with rationale and code examples
- [x] Implementation Patterns — Naming, structure, error handling, async, API, config, testing
- [x] Project Structure — Complete tree with FR→crate mapping and dependency matrix
- [x] Validation — Coherence, coverage, and readiness verified

**Overall Status:** READY FOR IMPLEMENTATION
**Confidence Level:** High

### Implementation Handoff

**AI Agent Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all components
- Respect project structure and crate boundaries
- Refer to this document for all architectural questions

**Implementation Priority Sequence:**
1. Core types and traits (ScoredHit, VectorSearchBackend, EmbeddingProvider)
2. Embedding crate (local + remote providers)
3. search-vector crate (LanceDB impl + BackendRegistry)
4. Extend search crate (search_scored for Tantivy)
5. Extend router (agent context + vector dispatch + RRF fusion)
6. Extend ingest (embed during sync pipeline)
7. Extend mcp/web/CLI (new tools + endpoints + commands)
8. Acceptance tests
