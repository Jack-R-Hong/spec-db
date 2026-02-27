---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
---

# lattice - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for lattice Multi-Backend Search, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

**Backend Management:**
- FR1: Operators can add a new search backend via configuration file
- FR2: Operators can add a new search backend via REST API at runtime
- FR3: Operators can add a new search backend via MCP tool at runtime
- FR4: Operators can remove an existing search backend
- FR5: Operators can list all configured search backends
- FR6: Operators can view health status of each search backend
- FR7: Operators can rebuild the index of a specific search backend

**Agent Routing:**
- FR8: Operators can define agent-to-backend routing rules in configuration
- FR9: System can route search queries to the appropriate backend based on agent context
- FR10: System can fall back to default backend when no routing rule matches
- FR11: Operators can use wildcard patterns in routing rules

**Search Capabilities:**
- FR12: Agents can search specs using full-text search (FTS) mode
- FR13: Agents can search specs using vector similarity mode
- FR14: Agents can search specs using hybrid mode (FTS + vector combined)
- FR15: System can normalize scores across different backends for comparable results
- FR16: Agents can filter search results by tags
- FR17: Agents can specify which backend to use for a search query

**Indexing & Data Management:**
- FR18: System can index spec documents into the configured search backend
- FR19: System can generate embeddings using local embedding models
- FR20: System can generate embeddings using remote API (OpenAI)
- FR21: System can remove spec documents from search backends
- FR22: System can sync spec documents from git to search backends

**Configuration:**
- FR23: Operators can configure embedding provider (local or remote) per backend
- FR24: Operators can configure embedding model and dimensions per backend
- FR25: Operators can set a default search backend
- FR26: Operators can configure backend-specific storage paths

**Migration & Compatibility:**
- FR27: Existing users can migrate configuration schema using CLI command
- FR28: System can operate with legacy configuration (Tantivy-only, no search_backends section)
- FR29: Existing MCP tools can function without breaking changes when new parameters are omitted

**Interfaces:**
- FR30: Developers can interact with backends via MCP tools (configure_backend, list_backends, extended search_specs)
- FR31: Developers can interact with backends via REST API (/api/backends endpoints)
- FR32: Developers can interact with backends via CLI (lattice backend commands)
- FR33: Developers can implement custom backends using the public VectorSearchBackend trait

### NonFunctional Requirements

**Performance:**
- NFR1: Search latency (FTS) < 50ms p95 for 1000 specs
- NFR2: Search latency (Vector) < 100ms p95 for 1000 specs
- NFR3: Search latency (Hybrid) < 150ms p95 for 1000 specs
- NFR4: Indexing throughput > 10 docs/sec batch indexing
- NFR5: Embedding generation (local) < 200ms/doc
- NFR6: Backend startup time < 5 seconds per backend

**Security:**
- NFR7: API keys stored securely — not logged or exposed in errors
- NFR8: No credentials in config examples — documentation uses placeholders
- NFR9: Input validation — all user inputs validated before processing

**Reliability:**
- NFR10: Backend failure isolation — one backend failure does not crash others
- NFR11: Graceful degradation — fall back to FTS if vector backend unavailable
- NFR12: Index corruption recovery — `lattice rebuild` can recover
- NFR13: Configuration validation — invalid config rejected with clear error

**Integration:**
- NFR14: Embedding provider abstraction — multiple providers via config
- NFR15: Backend trait stability — VectorSearchBackend API stable for external impls
- NFR16: MCP protocol compliance — all tools conform to MCP spec

**Compatibility:**
- NFR17: Rust version — supports Rust 1.85+
- NFR18: Platform support — Linux, macOS, Windows
- NFR19: Backward compatibility — existing users upgrade without data loss

### Additional Requirements

**From Architecture:**
- AR1: Brownfield project — no starter template; extend existing 7-crate workspace
- AR2: Add 2 new crates: `search-vector` (LanceDB + registry) and `embedding` (fastembed + OpenAI)
- AR3: New traits in core: `VectorSearchBackend`, `EmbeddingProvider`, `ScoredHit` type
- AR4: Extend `SpecDbError` with VectorError, EmbeddingError, BackendNotFound, RoutingError variants
- AR5: Backend Registry pattern using `HashMap<String, Box<dyn VectorSearchBackend>>`
- AR6: Reciprocal Rank Fusion (RRF) for hybrid search merge (k=60)
- AR7: All new operations sync, wrapped in `spawn_blocking` at async boundary
- AR8: Config extension: `search_backends` section in `.lattice/config.yaml`
- AR9: Extend existing `SearchEngine` trait with `search_scored()` default method
- AR10: Implementation sequence: core traits → embedding → search-vector → search extension → router → ingest → interfaces

### FR Coverage Map

FR1:  Epic 1 - Add backend via config
FR2:  Epic 4 - Add backend via REST API
FR3:  Epic 4 - Add backend via MCP tool
FR4:  Epic 4 - Remove backend
FR5:  Epic 4 - List backends
FR6:  Epic 4 - View backend health
FR7:  Epic 4 - Rebuild backend index
FR8:  Epic 3 - Define routing rules
FR9:  Epic 3 - Route by agent context
FR10: Epic 3 - Fallback to default
FR11: Epic 3 - Wildcard routing patterns
FR12: Epic 2 - FTS search mode
FR13: Epic 1 - Vector search mode
FR14: Epic 2 - Hybrid search mode
FR15: Epic 2 - Score normalization (RRF)
FR16: Epic 2 - Tag filtering
FR17: Epic 3 - Specify backend per query
FR18: Epic 1 - Index to vector backend
FR19: Epic 1 - Local embedding generation
FR20: Epic 5 - Remote embedding (OpenAI)
FR21: Epic 1 - Remove from vector backend
FR22: Epic 1 - Sync git to vector backend
FR23: Epic 5 - Configure embedding provider
FR24: Epic 5 - Configure embedding model
FR25: Epic 1 - Set default backend
FR26: Epic 1 - Configure storage paths
FR27: Epic 5 - Config migration CLI
FR28: Epic 5 - Legacy config support
FR29: Epic 5 - Backward compatible MCP tools
FR30: Epic 4 - MCP interface
FR31: Epic 4 - REST interface
FR32: Epic 4 - CLI interface
FR33: Epic 1 - Public VectorSearchBackend trait

## Epic List

### Epic 1: Embedding & Vector Search Core
Enable lattice to generate embeddings and perform vector search on specs using LanceDB. Developers can configure a vector backend, and `lattice sync` automatically generates embeddings and builds vector indexes. Agents can use vector similarity to search specs.
**FRs covered:** FR1, FR13, FR18, FR19, FR21, FR22, FR25, FR26, FR33
**ARs covered:** AR1, AR2, AR3, AR4, AR5, AR7, AR8, AR9, AR10

### Epic 2: Hybrid Search & Score Fusion
Agents get better search results by combining FTS + vector results via Reciprocal Rank Fusion. Existing FTS capability unchanged; new hybrid mode merges FTS and vector results. Agents find semantically related specs that pure keyword search misses.
**FRs covered:** FR12, FR14, FR15, FR16
**ARs covered:** AR6, AR9

### Epic 3: Agent-Scoped Routing
Different AI agents access different knowledge stores through config-driven routing rules. Operators define agent-to-backend mappings; the system routes queries automatically based on agent context. Supports wildcard patterns and fallback to default.
**FRs covered:** FR8, FR9, FR10, FR11, FR17

### Epic 4: Backend Management Interfaces
Operators can fully manage backends through MCP tools, REST API, and CLI. Includes add/remove/list/status/rebuild operations across all three interfaces. Enables runtime backend management and health monitoring.
**FRs covered:** FR2, FR3, FR4, FR5, FR6, FR7, FR30, FR31, FR32

### Epic 5: Migration & Remote Embedding
Existing lattice users upgrade seamlessly with `lattice migrate`. Legacy config (no search_backends) works unchanged. Teams can use OpenAI embeddings for higher quality vectors. All existing MCP tools maintain backward compatibility.
**FRs covered:** FR20, FR23, FR24, FR27, FR28, FR29

## Epic 1: Embedding & Vector Search Core

Enable lattice to generate embeddings and perform vector search on specs using LanceDB. Developers can configure a vector backend, and `lattice sync` automatically generates embeddings and builds vector indexes. Agents can use vector similarity to search specs.

### Story 1.1: Core Types and Traits

As a lattice developer,
I want core types (`ScoredHit`, `VectorSearchBackend`, `EmbeddingProvider`) and error variants defined in the `core` crate,
So that all downstream crates have a stable foundation to implement against.

**Acceptance Criteria:**

**Given** the `core` crate exists with `SpecDbError`, `SpecDoc`, and `SpecId` types
**When** I add the `ScoredHit` struct, `VectorSearchBackend` trait, and `EmbeddingProvider` trait to `crates/core/src/types.rs` and `crates/core/src/traits.rs`
**Then** `ScoredHit` has fields `id: SpecId` and `score: f32`
**And** `VectorSearchBackend` has methods: `index_spec(&mut self, doc: &SpecDoc, embedding: &[f32]) -> Result<(), SpecDbError>`, `remove_spec(&mut self, id: &SpecId) -> Result<(), SpecDbError>`, `search(&self, embedding: &[f32], limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>`, `search_with_tags(&self, embedding: &[f32], tags: &[String], limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>`
**And** `EmbeddingProvider` has methods: `embed(&self, text: &str) -> Result<Vec<f32>, SpecDbError>`, `embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SpecDbError>`, `dimensions(&self) -> usize`, `model_name(&self) -> &str`
**And** `SpecDbError` has new variants: `VectorError(String)`, `EmbeddingError(String)`, `BackendNotFound(String)`, `RoutingError(String)`
**And** all traits are `Send + Sync`
**And** `cargo build` succeeds with no warnings on the core crate

**FRs:** FR33 (public VectorSearchBackend trait)
**ARs:** AR3, AR4

---

### Story 1.2: Local Embedding Provider

As a lattice operator,
I want a local embedding provider using fastembed-rs,
So that I can generate embeddings without external API dependencies.

**Acceptance Criteria:**

**Given** the `EmbeddingProvider` trait is defined in core (Story 1.1)
**When** I create the `crates/embedding/` crate with a `LocalEmbedding` struct implementing `EmbeddingProvider`
**Then** `LocalEmbedding::new(model_name: &str)` loads the specified fastembed model (default: `all-MiniLM-L6-v2`)
**And** `embed(text)` returns a `Vec<f32>` of the correct dimensions (384 for MiniLM)
**And** `embed_batch(texts)` returns embeddings for multiple texts
**And** `dimensions()` returns the model's embedding dimension
**And** `model_name()` returns the configured model name
**And** all operations are sync (to be wrapped in `spawn_blocking` at async boundary)
**And** errors from fastembed are converted to `SpecDbError::EmbeddingError`
**And** integration tests pass using a temp directory for model cache

**FRs:** FR19 (local embedding generation)
**ARs:** AR2, AR7

---

### Story 1.3: LanceDB Backend Implementation

As a lattice operator,
I want a LanceDB vector search backend,
So that I can perform vector similarity search on indexed specs.

**Acceptance Criteria:**

**Given** the `VectorSearchBackend` trait is defined in core (Story 1.1)
**When** I create the `crates/search-vector/` crate with a `LanceDbBackend` struct implementing `VectorSearchBackend`
**Then** `LanceDbBackend::new(path: &Path, dimensions: usize)` opens or creates a LanceDB database at the given path
**And** `index_spec(doc, embedding)` stores the spec ID, embedding vector, title, and tags in a LanceDB table
**And** `remove_spec(id)` deletes the record with matching spec ID
**And** `search(embedding, limit)` returns the top-N closest vectors as `Vec<ScoredHit>` with cosine similarity scores
**And** `search_with_tags(embedding, tags, limit)` filters results to only those matching any of the given tags
**And** all operations are sync (LanceDB Rust SDK is sync)
**And** errors from LanceDB are converted to `SpecDbError::VectorError`
**And** integration tests pass using `tempfile::TempDir` for data isolation

**FRs:** FR13 (vector search), FR18 (index to backend), FR21 (remove from backend)
**ARs:** AR2, AR5, AR7

---

### Story 1.4: Backend Registry

As a lattice developer,
I want a `BackendRegistry` that manages multiple named vector backends,
So that the system can hold and resolve multiple backends simultaneously.

**Acceptance Criteria:**

**Given** the `LanceDbBackend` exists (Story 1.3)
**When** I create `BackendRegistry` in `crates/search-vector/src/registry.rs`
**Then** `BackendRegistry` stores backends in `HashMap<String, Box<dyn VectorSearchBackend>>`
**And** `add_backend(name, backend)` registers a new named backend
**And** `remove_backend(name)` removes and returns error `BackendNotFound` if not present
**And** `get(name)` returns `&dyn VectorSearchBackend` or `BackendNotFound` error
**And** `list()` returns all registered backend names
**And** `default_backend` field designates which backend is used when no name is specified
**And** unit tests verify add/remove/get/list operations

**FRs:** FR25 (default backend)
**ARs:** AR5

---

### Story 1.5: Backend Configuration Schema

As a lattice operator,
I want to configure vector backends via `.lattice/config.yaml`,
So that backends are initialized automatically on startup.

**Acceptance Criteria:**

**Given** the `BackendRegistry` and `LanceDbBackend` exist (Stories 1.3, 1.4)
**When** I extend the config types in `crates/core/` and add initialization logic
**Then** `.lattice/config.yaml` accepts a `search_backends` section with `default`, `backends` list
**And** each backend entry has `name`, `type`, `path` (for lancedb), and optional `embedding` subsection
**And** `embedding` subsection has `provider` (local/openai), `model`, `dimensions`
**And** config is validated at startup: unknown types rejected, missing required fields error with clear message
**And** missing `search_backends` section is valid (backward compatible, no vector backends)
**And** config deserialization uses `serde` with typed structs
**And** integration test verifies config parsing with valid and invalid YAML

**FRs:** FR1 (add backend via config), FR26 (storage paths)
**ARs:** AR8

---

### Story 1.6: Vector Indexing in Sync Pipeline

As a lattice operator,
I want `lattice sync` to automatically generate embeddings and index specs into configured vector backends,
So that vector search is populated without manual steps.

**Acceptance Criteria:**

**Given** a configured LanceDB backend with local embedding provider (Stories 1.2, 1.3, 1.5)
**When** I extend the `ingest` crate's sync pipeline
**Then** during `lattice sync`, each spec is: parsed → embedded via `EmbeddingProvider` → indexed to FTS (Tantivy) AND vector backend (LanceDB)
**And** `lattice sync --full` rebuilds vector index from scratch (drop and re-create)
**And** `lattice rebuild` also rebuilds vector backends
**And** embedding generation runs in `spawn_blocking` at the async boundary
**And** if no vector backend is configured, sync proceeds as before (FTS only)
**And** errors from embedding or vector indexing are logged but do not block FTS sync
**And** integration test verifies specs appear in both Tantivy and LanceDB after sync

**FRs:** FR22 (sync git to vector backend), FR18 (index to backend)
**ARs:** AR7, AR10

---

## Epic 2: Hybrid Search & Score Fusion

Agents get better search results by combining FTS + vector results via Reciprocal Rank Fusion. Existing FTS capability unchanged; new hybrid mode merges FTS and vector results. Agents find semantically related specs that pure keyword search misses.

### Story 2.1: Scored FTS Results

As a lattice developer,
I want the existing Tantivy `SearchEngine` to return scored results,
So that FTS scores can participate in hybrid search fusion.

**Acceptance Criteria:**

**Given** the existing `SearchEngine` trait in core with `search()` returning `Vec<SpecId>`
**When** I add `search_scored(&self, query: &str, limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>` as a default method on `SearchEngine`
**Then** the default implementation delegates to `search()` and returns `ScoredHit { id, score: 0.0 }` for each result
**And** the Tantivy `SearchIndex` in `crates/search/` overrides `search_scored()` to return actual BM25 scores from Tantivy
**And** existing `search()` and `search_with_tags()` methods are unchanged
**And** all existing tests continue to pass
**And** unit test verifies that `search_scored()` returns non-zero scores for matching documents

**FRs:** FR12 (FTS search mode)
**ARs:** AR9

---

### Story 2.2: Reciprocal Rank Fusion Implementation

As a lattice developer,
I want a Reciprocal Rank Fusion (RRF) function that merges two ranked result lists,
So that hybrid search can combine FTS and vector results into a single ranking.

**Acceptance Criteria:**

**Given** `ScoredHit` type exists in core (Story 1.1)
**When** I implement `reciprocal_rank_fusion(fts_results: &[ScoredHit], vector_results: &[ScoredHit], k: u32) -> Vec<ScoredHit>` in `crates/search-vector/src/fusion.rs`
**Then** the function computes fused score as `Σ 1/(k + rank_i)` for each unique spec ID across both lists
**And** default `k` value is 60
**And** results are sorted by fused score descending
**And** specs appearing in both lists get higher fused scores than specs in only one list
**And** the function handles empty inputs gracefully (empty FTS or empty vector list)
**And** unit tests verify: both lists contribute, single-list input works, empty inputs return empty

**FRs:** FR15 (score normalization via RRF)
**ARs:** AR6

---

### Story 2.3: Hybrid Search Mode in Router

As an AI agent,
I want to search specs using hybrid mode that combines FTS and vector results,
So that I find both keyword-matched and semantically related specs.

**Acceptance Criteria:**

**Given** scored FTS results (Story 2.1), RRF fusion (Story 2.2), and a configured vector backend (Epic 1)
**When** I extend the `QueryRouter` to handle a `Hybrid` search mode
**Then** a hybrid search: (1) embeds the query via `EmbeddingProvider`, (2) runs `search_scored()` on Tantivy, (3) runs `search()` on vector backend, (4) merges via RRF
**And** the router accepts `mode` parameter: `fts` (default, existing), `vector`, `hybrid`
**And** `mode=vector` skips FTS and queries only the vector backend
**And** `mode=hybrid` executes both and merges via RRF
**And** `mode=fts` behaves exactly as before (backward compatible)
**And** tag filtering works in all modes: FTS uses `search_with_tags`, vector uses `search_with_tags`
**And** if no vector backend is configured and mode is `vector` or `hybrid`, return `SpecDbError::BackendNotFound`
**And** integration test verifies hybrid results contain specs from both FTS and vector

**FRs:** FR14 (hybrid mode), FR16 (tag filtering)
**ARs:** AR6, AR9

---

## Epic 3: Agent-Scoped Routing

Different AI agents access different knowledge stores through config-driven routing rules. Operators define agent-to-backend mappings; the system routes queries automatically based on agent context. Supports wildcard patterns and fallback to default.

### Story 3.1: Routing Rules Configuration

As a lattice operator,
I want to define agent-to-backend routing rules in configuration,
So that different agents are automatically routed to different backends.

**Acceptance Criteria:**

**Given** the `search_backends` config section exists (Story 1.5)
**When** I add a `routing` list to the config with `agent` (glob pattern) and `backend` (name) fields
**Then** routing rules are parsed into `Vec<RoutingRule>` with `agent_pattern: String` and `backend: String`
**And** `RoutingRule` is defined in `crates/core/`
**And** wildcard patterns are supported (e.g., `doc-*` matches `doc-agent`, `doc-writer`)
**And** `*` matches any agent (used as catch-all fallback)
**And** config validation rejects rules referencing non-existent backend names
**And** if no routing rules are defined, all agents use the `default` backend
**And** unit tests verify glob matching logic and config validation

**FRs:** FR8 (routing rules), FR11 (wildcard patterns)

---

### Story 3.2: Agent Context Propagation

As a lattice developer,
I want agent identity propagated from MCP/REST interfaces to the router,
So that the router can resolve the correct backend for each agent.

**Acceptance Criteria:**

**Given** routing rules are configured (Story 3.1)
**When** I add `agent_context: Option<String>` parameter to router search methods and interface layers
**Then** MCP tools accept an optional `agent_id` parameter in `search_specs` and `query` tools
**And** REST API accepts `X-Agent-Id` header or `agent_id` query parameter
**And** the agent context is passed through: interface → router → backend resolution
**And** if `agent_id` is not provided, `agent_context` is `None`
**And** existing tool calls without `agent_id` continue to work unchanged (backward compatible)
**And** integration test verifies agent_id flows from MCP tool call to router

**FRs:** FR9 (route by agent context), FR29 (backward compatible MCP tools)

---

### Story 3.3: Backend Resolution and Dispatch

As an AI agent,
I want my search queries automatically routed to my assigned backend,
So that I only see the knowledge store I'm authorized to access.

**Acceptance Criteria:**

**Given** routing rules and agent context propagation exist (Stories 3.1, 3.2)
**When** I implement `BackendRegistry::resolve(agent_context: Option<&str>) -> Result<&dyn VectorSearchBackend, SpecDbError>`
**Then** if `agent_context` matches a routing rule, the corresponding backend is returned
**And** routing rules are evaluated in order; first match wins
**And** if no rule matches but `agent_context` is provided, the `default` backend is used
**And** if `agent_context` is `None`, the `default` backend is used
**And** an agent can explicitly override routing by specifying `backend` parameter in search (FR17)
**And** if the resolved backend name doesn't exist in registry, return `SpecDbError::BackendNotFound`
**And** integration test verifies: agent match → correct backend, no match → default, explicit override works

**FRs:** FR9 (route by agent context), FR10 (fallback to default), FR17 (specify backend per query)

---

## Epic 4: Backend Management Interfaces

Operators can fully manage backends through MCP tools, REST API, and CLI. Includes add/remove/list/status/rebuild operations across all three interfaces. Enables runtime backend management and health monitoring.

### Story 4.1: CLI Backend Commands

As a lattice operator,
I want CLI commands to manage search backends,
So that I can list, add, remove, check status, and rebuild backends from the terminal.

**Acceptance Criteria:**

**Given** `BackendRegistry` exists with add/remove/list operations (Story 1.4)
**When** I add `lattice backend` subcommands to the CLI binary
**Then** `lattice backend list` outputs all configured backends with their type and status
**And** `lattice backend add <name> <type>` adds a new backend (writes to config and initializes)
**And** `lattice backend remove <name>` removes a backend (from config and registry)
**And** `lattice backend status [name]` shows health info: backend type, doc count, index size, last sync time
**And** `lattice rebuild --backend=<name>` rebuilds only the specified backend's index
**And** all commands produce clear error messages for invalid inputs
**And** integration test verifies list/add/remove/status commands

**FRs:** FR4 (remove backend), FR5 (list backends), FR6 (health status), FR7 (rebuild), FR32 (CLI interface)

---

### Story 4.2: REST API Backend Endpoints

As a platform engineer,
I want REST API endpoints to manage backends at runtime,
So that automation scripts can provision backends without CLI access.

**Acceptance Criteria:**

**Given** `BackendRegistry` and CLI commands exist (Stories 1.4, 4.1)
**When** I add `/api/backends` routes to the `web` crate (Axum)
**Then** `GET /api/backends` returns JSON array of all backends with name, type, status
**And** `POST /api/backends` creates a new backend from JSON body `{ name, type, config }` and returns 201
**And** `DELETE /api/backends/:name` removes a backend and returns 204
**And** `GET /api/backends/:name/status` returns health info as JSON
**And** invalid requests return appropriate HTTP status codes (400, 404, 409)
**And** all endpoints are behind the existing Axum web server (enabled via config `web.enabled: true`)
**And** integration test verifies CRUD operations via HTTP

**FRs:** FR2 (add via REST), FR4 (remove), FR5 (list), FR6 (status), FR31 (REST interface)

---

### Story 4.3: MCP Backend Tools

As an AI agent developer,
I want MCP tools to manage and query backends,
So that AI agents can discover and interact with backends through the MCP protocol.

**Acceptance Criteria:**

**Given** `BackendRegistry` and REST endpoints exist (Stories 1.4, 4.2)
**When** I add MCP tools `configure_backend`, `list_backends`, and extend `search_specs` in the `mcp` crate
**Then** `configure_backend(name, type, config)` creates or updates a backend and returns confirmation
**And** `list_backends()` returns all backends with name, type, and health status
**And** `search_specs` gains optional parameters: `backend` (name), `mode` (fts/vector/hybrid), `agent_id`
**And** when `backend` is specified, it overrides routing rules
**And** when `mode` is omitted, defaults to `fts` (backward compatible)
**And** when `agent_id` is omitted, uses default backend (backward compatible)
**And** all new tools conform to MCP protocol specification (proper JSON schema)
**And** integration test verifies tools via MCP tool call simulation

**FRs:** FR3 (add via MCP), FR30 (MCP interface), FR29 (backward compatible MCP tools)

---

## Epic 5: Migration & Remote Embedding

Existing lattice users upgrade seamlessly with `lattice migrate`. Legacy config (no search_backends) works unchanged. Teams can use OpenAI embeddings for higher quality vectors. All existing MCP tools maintain backward compatibility.

### Story 5.1: Legacy Configuration Compatibility

As an existing lattice user,
I want my current config (no search_backends section) to work without changes,
So that upgrading lattice doesn't break my existing setup.

**Acceptance Criteria:**

**Given** a `.lattice/config.yaml` without any `search_backends` section (existing format)
**When** the new lattice binary starts up
**Then** the system operates in FTS-only mode (existing Tantivy behavior)
**And** no vector backends are initialized
**And** all existing MCP tools work identically to pre-upgrade behavior
**And** `search_specs` with no `mode` or `backend` parameters works as before
**And** `lattice sync` indexes only to Tantivy (no embedding generation)
**And** `lattice status` shows existing info without backend-related fields
**And** no errors, warnings, or deprecation notices for legacy config
**And** acceptance test verifies full existing workflow on legacy config

**FRs:** FR28 (legacy config support), FR29 (backward compatible MCP tools)

---

### Story 5.2: Configuration Migration Tool

As an existing lattice user,
I want a `lattice migrate` command to upgrade my config schema,
So that I can adopt the new features incrementally.

**Acceptance Criteria:**

**Given** a legacy config file without `search_backends` section
**When** I run `lattice migrate`
**Then** the command detects the current config schema version
**And** if already up-to-date, prints "Config is already current" and exits
**And** if migration needed, creates a backup of current config as `config.yaml.bak`
**And** adds a minimal `search_backends` section with `default: tantivy` and the FTS backend entry
**And** preserves all existing config values (specs_dir, data_dir, transport, telemetry)
**And** prints a summary of changes made
**And** the migrated config passes validation
**And** integration test verifies migration from legacy to new format preserves all fields

**FRs:** FR27 (migration CLI command)

---

### Story 5.3: Remote Embedding Provider (OpenAI)

As a lattice operator,
I want to use OpenAI's embedding API for higher quality vectors,
So that semantic search returns more relevant results for domain-specific content.

**Acceptance Criteria:**

**Given** the `EmbeddingProvider` trait and `LocalEmbedding` exist (Stories 1.1, 1.2)
**When** I add `RemoteEmbedding` struct in `crates/embedding/src/remote.rs` implementing `EmbeddingProvider`
**Then** `RemoteEmbedding::new(api_key, model, dimensions)` configures the OpenAI API client
**And** `embed(text)` calls OpenAI's embedding API and returns the vector
**And** `embed_batch(texts)` sends batch requests to reduce API calls
**And** API errors are converted to `SpecDbError::EmbeddingError` with descriptive messages
**And** API key is never logged or included in error messages (NFR7)
**And** network timeouts and retries are handled (3 retries with exponential backoff)
**And** the provider works with `text-embedding-3-small` and `text-embedding-ada-002` models
**And** integration test with mock HTTP server verifies API call format and error handling

**FRs:** FR20 (remote embedding via OpenAI)

---

### Story 5.4: Embedding Provider Configuration

As a lattice operator,
I want to configure which embedding provider each backend uses,
So that I can choose local or remote embedding per backend.

**Acceptance Criteria:**

**Given** `LocalEmbedding` and `RemoteEmbedding` both exist (Stories 1.2, 5.3)
**When** I extend the config `embedding` subsection and add provider initialization logic
**Then** `provider: local` with `model: all-MiniLM-L6-v2` initializes `LocalEmbedding`
**And** `provider: openai` with `model: text-embedding-3-small` initializes `RemoteEmbedding`
**And** `dimensions` is validated against the model's actual output dimensions at startup
**And** dimension mismatch between embedding provider and existing LanceDB index produces a clear error
**And** API key for OpenAI is read from `OPENAI_API_KEY` environment variable (not stored in config file)
**And** if `embedding` section is missing for a vector backend, startup fails with clear error
**And** integration test verifies both provider types initialize correctly from config

**FRs:** FR23 (configure embedding provider per backend), FR24 (configure model and dimensions)
