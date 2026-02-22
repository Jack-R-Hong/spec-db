---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
status: complete
completedAt: '2026-02-23'
inputDocuments:
  - prd.md
  - architecture.md
---

# spec-db - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for spec-db, decomposing the requirements from the PRD and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

**Spec Discovery (5):**

FR1: Agents can search specs by keyword with BM25 relevance-ranked results
FR2: Agents can filter search results by tag
FR3: Agents can retrieve the full content of a spec by its ID
FR4: Agents can receive title-boosted search results (title matches rank higher than body matches)
FR5: Agents can receive search results that include spec ID, title, tags, and relevance score

**Causal Reasoning (5):**

FR6: Agents can trace the downstream causal impact of a spec ("if I change this, what breaks?")
FR7: Agents can discover the upstream dependencies of a spec ("what does this depend on?")
FR8: Agents can specify traversal depth when tracing impact or dependencies
FR9: The system automatically creates causal edges from `depends_on` fields in spec frontmatter
FR10: Agents can view a spec's node with all its inbound and outbound causal edges

**Hybrid Intelligence (3):**

FR11: Agents can submit natural-language queries that are automatically routed to search, causal reasoning, or both
FR12: The query router provides causal context when search returns zero results
FR13: Agents can receive composed results that combine search results with causal context

**Spec Lifecycle (6):**

FR14: Agents can ingest a new spec document (markdown + YAML frontmatter)
FR15: The system parses spec frontmatter to extract ID, title, version, tags, depends_on, owner, and created date
FR16: The system indexes spec content for full-text search on ingestion
FR17: The system creates causal graph nodes and edges on ingestion
FR18: The system validates spec IDs for uniqueness during ingestion
FR19: Spec authors can write specs in markdown with YAML frontmatter following a defined format

**Git Integration (6):**

FR20: The system can perform a full rebuild of all indexes from a git repository tree walk
FR21: The system can perform incremental sync by processing only changed files via `git diff`
FR22: The system detects renamed files during incremental sync using git rename detection
FR23: The system removes specs from indexes and graph when deleted from git
FR24: Full rebuild produces identical indexes regardless of when it is run (idempotent)
FR25: The system tracks the last-synced git commit SHA for both stores

**Agent Integration / MCP (7):**

FR26: The system exposes all capabilities as MCP tools over stdio transport
FR27: The system optionally exposes MCP tools over streamable-http transport
FR28: Agents can discover available spec-db tools through MCP protocol
FR29: Agents can read individual spec content via `spec://{id}` resource
FR30: Agents can read causal graph summary statistics via `graph://overview` resource
FR31: Agents can read a specific node with all edges via `graph://node/{id}` resource
FR32: The `graph://overview` resource exposes disconnected clusters (specs with no causal edges)

**System Administration / CLI (6):**

FR33: Users can initialize a new spec-db project with scaffolded directory structure and example specs
FR34: Users can start the MCP server from a configuration file
FR35: Users can manually trigger sync (full or incremental) via CLI
FR36: Users can perform a destructive full index rebuild via CLI
FR37: Users can view index health status (document count, last sync commit, consistency check result)
FR38: The system reads all configuration from `.spec-db/config.yaml`

**Data Integrity (5):**

FR39: The system verifies cross-store consistency (Tantivy vs. Fjall) on startup
FR40: The system verifies cross-store consistency after every sync operation
FR41: The system compares git commit SHA and document count across both stores to detect drift
FR42: The system warns and offers auto-rebuild when drift is detected
FR43: The system auto-escalates to full rebuild when incremental sync doc counts diverge

**Observability (3):**

FR44: The system emits OpenTelemetry traces for search queries, graph traversals, sync operations, and MCP tool calls
FR45: The system emits OpenTelemetry metrics for search latency, sync duration, consistency check results, and document counts
FR46: The system emits drift detection metrics when cross-store inconsistency is found

### NonFunctional Requirements

**Performance (10):**

NFR1: Full-text search latency < 10ms across 100+ specs
NFR2: Causal graph traversal < 50ms at depth ≤ 5
NFR3: Query router classification < 5ms overhead
NFR4: Startup time < 1 second (graph load from Fjall)
NFR5: Full rebuild < 5 seconds for 100+ specs
NFR6: Incremental sync < 2 seconds (changed files only)
NFR7: Spec ingestion < 100ms per spec
NFR8: MCP tool response < 100ms end-to-end
NFR9: Memory footprint < 50MB (100+ specs + full graph)
NFR10: Binary size < 30MB (single statically-linked binary)

**Reliability (6):**

NFR11: Rebuild idempotency — `git clone` + `spec-db rebuild` produces bit-identical indexes
NFR12: Crash recovery — Fjall LSM-tree durability, no data loss on unexpected shutdown
NFR13: Zero data lock-in — all state derived from git
NFR14: Graceful degradation — search-only mode if causal graph fails to load
NFR15: Atomic rebuilds — temp directories + swap prevents serving partial indexes
NFR16: Error propagation — all errors surface clearly, no silent failures

**Integration (6):**

NFR17: MCP protocol compliance (version 2025-11-25)
NFR18: Git compatibility via libgit2 / git2 crate
NFR19: Cross-platform — Linux, macOS, Windows
NFR20: Stdio transport default for local MCP
NFR21: Streamable-http transport optional for remote access
NFR22: OpenTelemetry export via standard OTLP protocol

**Security (5):**

NFR23: Local-first by default (stdio = no network surface)
NFR24: Token-based auth required when HTTP transport enabled
NFR25: No telemetry home — OpenTelemetry export is opt-in
NFR26: File system scoping — reads/writes only configured directories
NFR27: No code execution — parses markdown + YAML only

**Scalability (4):**

NFR28: Target: hundreds of specs (100-500) per team
NFR29: Upper bound: thousands without architectural changes
NFR30: Not designed for millions/multi-tenant/cross-org
NFR31: Single MCP server process; concurrent agent access deferred

### Additional Requirements

- **Starter Template:** Virtual workspace (lean start) — 3 initial crates (`core`, `causal`), expanding per build order. Impacts Epic 1, Story 1 (project scaffolding).
- **Version Lock:** Tantivy 0.25.0, Fjall 3.0.x, DeepCausality 0.13.4, rmcp 0.16.0, git2 0.20.4, pulldown-cmark 0.13.0, serde_yml (replaces deprecated serde_yaml), Tokio 1.49.0, clap 4.5.x, opentelemetry 0.31.0
- **serde_yaml deprecated** — must use `serde_yml` for spec ingestion and config parsing
- **Fjall serialization:** bincode for key-value encoding
- **SpecId newtype:** `struct SpecId(String)` with validation pattern `spec::{segment}::{segment}`
- **Graph model (MVP):** Specs-only nodes, depends_on-only edges
- **Error handling strategy:** `thiserror` in library crates, `anyhow` at binary entry point
- **Query router:** Keyword heuristic classification (not ML-based)
- **Observability stack:** `tracing` + `tracing-opentelemetry` + `tracing-subscriber`
- **Async boundary:** MCP handlers = async (Tokio), everything below = sync via `spawn_blocking`
- **Anti-patterns enforced:** No `unwrap()` in lib code, no wildcard re-exports, no `mod.rs`, no domain types outside `core`
- **18 implementation consistency patterns** defined (6 naming, 5 structure, 4 format, 3 process)
- **Workspace expansion:** 6 phases matching risk-first build order
- **Implementation sequence:** 11 ordered steps

### FR Coverage Map

FR1: Epic 2 - BM25 keyword search
FR2: Epic 2 - Tag-filtered search
FR3: Epic 2 - Retrieve spec by ID
FR4: Epic 2 - Title-boosted results
FR5: Epic 2 - Search result fields (ID, title, tags, score)
FR6: Epic 1 - Trace downstream causal impact
FR7: Epic 1 - Discover upstream dependencies
FR8: Epic 1 - Configurable traversal depth
FR9: Epic 1 - Auto-create edges from `depends_on`
FR10: Epic 1 - View node with all edges
FR11: Epic 5 - Natural-language query routing
FR12: Epic 5 - Causal context on zero search results
FR13: Epic 5 - Composed search + causal results
FR14: Epic 3 - Ingest new spec document
FR15: Epic 3 - Parse frontmatter fields
FR16: Epic 3 - Index content for search on ingestion
FR17: Epic 3 - Create graph nodes/edges on ingestion
FR18: Epic 3 - Validate spec ID uniqueness
FR19: Epic 3 - Markdown + YAML frontmatter spec format
FR20: Epic 4 - Full rebuild from git tree walk
FR21: Epic 4 - Incremental sync via git diff
FR22: Epic 4 - Rename detection during sync
FR23: Epic 4 - Remove deleted specs from stores
FR24: Epic 4 - Idempotent full rebuild
FR25: Epic 4 - Track last-synced git SHA
FR26: Epic 6 - MCP tools over stdio
FR27: Epic 6 - MCP tools over streamable-http
FR28: Epic 6 - MCP tool discovery
FR29: Epic 6 - `spec://{id}` resource
FR30: Epic 6 - `graph://overview` resource
FR31: Epic 6 - `graph://node/{id}` resource
FR32: Epic 6 - Disconnected cluster detection in overview
FR33: Epic 6 - Project init with scaffolding
FR34: Epic 6 - Start MCP server from config
FR35: Epic 6 - CLI sync trigger
FR36: Epic 6 - CLI rebuild command
FR37: Epic 6 - CLI status (health check)
FR38: Epic 6 - Config from `.spec-db/config.yaml`
FR39: Epic 7 - Cross-store consistency on startup
FR40: Epic 7 - Cross-store consistency after sync
FR41: Epic 7 - SHA + doc count comparison
FR42: Epic 7 - Warn + auto-rebuild on drift
FR43: Epic 7 - Auto-escalate to full rebuild on count divergence
FR44: Epic 7 - OpenTelemetry traces
FR45: Epic 7 - OpenTelemetry metrics
FR46: Epic 7 - Drift detection metrics

## Epic List

### Epic 1: Foundation & Causal Knowledge Graph
The spec-db workspace is scaffolded with core domain types, trait interfaces, and error hierarchy. The riskiest integration — DeepCausality + Fjall — is proven. Specs can be stored as causal graph nodes with `depends_on` edges, and agents can traverse impact chains and dependency paths.
**FRs covered:** FR6, FR7, FR8, FR9, FR10

### Epic 2: Spec Discovery & Search
Full-text BM25 search over spec documents is operational. Agents get relevance-ranked results with title boosting over body, tag-based filtering, and results that include spec ID, title, tags, and score.
**FRs covered:** FR1, FR2, FR3, FR4, FR5

### Epic 3: Spec Authoring & Ingestion
Spec authors write markdown documents with YAML frontmatter in the defined format. The system parses frontmatter, validates spec IDs for uniqueness, indexes content for full-text search, and creates causal graph nodes and edges — all in a single ingestion pipeline.
**FRs covered:** FR14, FR15, FR16, FR17, FR18, FR19

### Epic 4: Git-Centric Sync
Git is the single source of truth. The system performs full rebuilds from git tree walks and incremental sync via `git diff` with rename detection. Deleted specs are removed from indexes and graph. Rebuilds are idempotent. Both stores track the last-synced commit SHA.
**FRs covered:** FR20, FR21, FR22, FR23, FR24, FR25

### Epic 5: Intelligent Query Routing
Agents submit natural-language queries that are automatically classified and routed to search, causal reasoning, or both. When search returns zero results, the router still provides causal context. Composed results combine search hits with causal relationships.
**FRs covered:** FR11, FR12, FR13

### Epic 6: MCP Server & CLI
AI agents discover and call spec-db tools/resources via MCP over stdio (default) and optional streamable-http transport. Users manage spec-db via CLI commands: init (scaffold project), serve (start MCP server), sync (manual trigger), rebuild (destructive rebuild), and status (health check). All configuration reads from `.spec-db/config.yaml`.
**FRs covered:** FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR33, FR34, FR35, FR36, FR37, FR38

### Epic 7: Data Integrity & Observability
Cross-store consistency is verified on startup and after every sync — comparing git SHA and doc counts across Tantivy and Fjall. Drift triggers warnings and auto-rebuild. OpenTelemetry traces and metrics are emitted for all key operations (search, traversal, sync, MCP calls, consistency checks).
**FRs covered:** FR39, FR40, FR41, FR42, FR43, FR44, FR45, FR46

## Epic 1: Foundation & Causal Knowledge Graph

The spec-db workspace is scaffolded with core domain types, trait interfaces, and error hierarchy. The riskiest integration — DeepCausality + Fjall — is proven. Specs can be stored as causal graph nodes with `depends_on` edges, and agents can traverse impact chains and dependency paths.

### Story 1.1: Scaffold Workspace & Core Domain Types

As a developer,
I want a properly scaffolded Cargo workspace with core domain types, trait interfaces, and error hierarchy,
So that all subsequent development has a consistent, version-locked foundation.

**Acceptance Criteria:**

**Given** a clean checkout of the repository
**When** I run `cargo build --workspace`
**Then** the workspace compiles with `spec-db` (binary), `spec-db-core` (lib), and `spec-db-causal` (lib) crates
**And** `spec-db-core` exports `SpecId`, `SpecDoc`, `SpecNode`, `CausalEdge`, and `TrustLevel` types
**And** `SpecId` validates the `spec::{segment}::{segment}` pattern and rejects invalid formats
**And** `spec-db-core` exports `SearchEngine`, `CausalGraph`, and `SpecStore` traits
**And** `spec-db-core` exports `SpecDbError` with variants: `SearchError`, `GraphError`, `SyncError`, `IngestError`, `ConsistencyError`, `ConfigError`
**And** `workspace.dependencies` in root `Cargo.toml` locks all dependency versions per architecture spec
**And** `rustfmt.toml` and `clippy.toml` are configured per architecture patterns
**And** `cargo clippy --workspace -- -D warnings` passes with zero warnings
**And** `cargo fmt --all -- --check` passes

### Story 1.2: Fjall Persistent Storage for Causal Graph

As a developer,
I want spec nodes and causal edges persisted in Fjall keyspaces with bincode serialization,
So that the causal graph survives process restarts and can be loaded into memory on startup.

**Acceptance Criteria:**

**Given** the `spec-db-causal` crate with Fjall 3.0.x and bincode dependencies
**When** I store a `SpecNode` with a valid `SpecId`
**Then** the node is persisted in the `nodes` keyspace with key=SpecId string and value=bincode-serialized SpecNode
**And** I can retrieve the identical SpecNode by key

**Given** a `CausalEdge` representing A depends_on B
**When** I store the edge
**Then** it is persisted in the `edges` keyspace with key=`{from_id}\x00{to_id}` and value=bincode-serialized CausalEdge
**And** I can retrieve the edge by its composite key

**Given** system metadata (last_sync_sha, doc_count)
**When** I store metadata values
**Then** they are persisted in the `meta` keyspace with string keys
**And** I can retrieve them accurately after reopening the store

**Given** a node with associated edges being added
**When** the operation executes
**Then** node and edge writes are atomic via Fjall cross-keyspace batch
**And** partial writes never occur on failure

### Story 1.3: DeepCausality In-Memory Graph Engine

As a developer,
I want specs loaded into a DeepCausality in-memory graph with edges automatically created from `depends_on` fields,
So that causal relationships are traversable in memory with sub-50ms performance.

**Acceptance Criteria:**

**Given** a Fjall store containing persisted spec nodes and causal edges
**When** the graph engine initializes
**Then** all nodes and edges are loaded into the DeepCausality in-memory graph
**And** startup completes in < 1 second for 100+ specs (NFR4)

**Given** a spec with `depends_on: ["spec::auth::token-issuance"]` in its frontmatter
**When** the spec is added to the graph
**Then** a causal edge is automatically created from the spec to `spec::auth::token-issuance` (FR9)
**And** the edge has type `depends_on` and trust level 1.0 (human-curated)

**Given** a spec node in the graph
**When** I request the node view
**Then** I receive the node with all inbound edges (specs that depend on it) and all outbound edges (specs it depends on) (FR10)

### Story 1.4: Causal Graph Traversal (trace_impact & find_dependencies)

As an AI agent,
I want to trace the downstream impact of a spec and discover its upstream dependencies with configurable depth,
So that I understand the blast radius before proposing changes and know what a spec relies on.

**Acceptance Criteria:**

**Given** spec A depends_on spec B, and spec C depends_on spec A
**When** I call `trace_impact(B)`
**Then** I receive A and C as downstream impacts (everything that transitively depends on B) (FR6)

**Given** spec A depends_on spec B, and spec B depends_on spec D
**When** I call `find_dependencies(A)`
**Then** I receive B and D as upstream dependencies (everything A transitively depends on) (FR7)

**Given** a deep causal chain (A→B→C→D→E)
**When** I call `trace_impact(E, depth=2)`
**Then** I receive only nodes within 2 hops (D, C) — not the full chain (FR8)
**And** when I call `trace_impact(E)` without depth limit, I receive the complete chain

**Given** a graph with 100+ specs
**When** I call `trace_impact` or `find_dependencies`
**Then** the traversal completes in < 50ms (NFR2)

**Given** a spec ID that does not exist in the graph
**When** I call `trace_impact` or `find_dependencies`
**Then** I receive a clear `GraphError` indicating the spec was not found

## Epic 2: Spec Discovery & Search

Full-text BM25 search over spec documents is operational. Agents get relevance-ranked results with title boosting over body, tag-based filtering, and results that include spec ID, title, tags, and score.

### Story 2.1: Tantivy Schema Definition & Spec Indexing

As a developer,
I want a Tantivy search index with the defined schema and operations to add, remove, and commit spec documents,
So that specs are indexed and ready for full-text search queries.

**Acceptance Criteria:**

**Given** the `spec-db-search` crate with Tantivy 0.25.0
**When** I create a new search index
**Then** the schema contains fields: `id` (STRING|STORED), `title` (TEXT|STORED), `body` (TEXT), `tags` (STRING|STORED), `meta` (JSON|STORED)

**Given** a `SpecDoc` with id, title, body, tags, and metadata
**When** I call `add_doc` on the `SearchEngine` implementation
**Then** the document is added to the Tantivy index
**And** the document is retrievable after `commit`

**Given** a spec ID of an indexed document
**When** I call `remove_doc` with that ID
**Then** the document is removed from the index after `commit`
**And** subsequent searches do not return the removed document

**Given** an index with uncommitted changes
**When** I call `commit`
**Then** all pending additions and removals are persisted atomically

### Story 2.2: BM25 Search with Title Boosting, Tag Filtering & Spec Retrieval

As an AI agent,
I want to search specs by keyword with relevance ranking, filter by tags, and retrieve full spec content by ID,
So that I can quickly discover the most relevant specs for my task.

**Acceptance Criteria:**

**Given** an index containing specs with titles and body text
**When** I call `search_specs("rate limiting")`
**Then** I receive results ranked by BM25 relevance score (FR1)
**And** each result includes spec ID, title, tags, and relevance score (FR5)

**Given** a spec with "rate limiting" in the title and another with "rate limiting" only in the body
**When** I search for "rate limiting"
**Then** the title-match spec ranks higher than the body-match spec (FR4)

**Given** specs tagged with "auth" and "api"
**When** I search with tag filter `tags: "auth"`
**Then** only specs with the "auth" tag are returned (FR2)
**And** tag filtering uses exact string matching, not full-text

**Given** a spec with known ID `spec::auth::jwt-validation`
**When** I call `get_spec("spec::auth::jwt-validation")`
**Then** I receive the full spec content including all stored fields (FR3)

**Given** a search query on an index with 100+ specs
**When** the search executes
**Then** results are returned in < 10ms (NFR1)

**Given** a search query matching no specs
**When** the search executes
**Then** I receive an empty result set (not an error)

## Epic 3: Spec Authoring & Ingestion

Spec authors write markdown documents with YAML frontmatter in the defined format. The system parses frontmatter, validates spec IDs for uniqueness, indexes content for full-text search, and creates causal graph nodes and edges — all in a single ingestion pipeline.

### Story 3.1: Spec Format Definition & Markdown/YAML Parsing

As a spec author,
I want to write specs in markdown with YAML frontmatter following a defined format,
So that the system can parse and understand my specifications.

**Acceptance Criteria:**

**Given** a markdown file with valid YAML frontmatter containing `id`, `title`, `version`, `tags`, `depends_on`, `owner`, and `created`
**When** the parser processes the file
**Then** all frontmatter fields are correctly extracted into a `SpecDoc` struct (FR15)
**And** the markdown body is extracted separately from the frontmatter

**Given** a spec with the format:
```yaml
---
id: "spec::auth::jwt-validation"
title: "JWT Token Validation"
version: 1
tags: ["auth", "security"]
depends_on: ["spec::auth::token-issuance"]
owner: "backend-team"
created: 2026-03-15
---
# JWT Token Validation
...markdown body...
```
**When** the parser processes it
**Then** each field maps to the corresponding `SpecDoc` field (FR19)

**Given** a spec with a `SpecId` that does not match the `spec::{segment}::{segment}` pattern
**When** the parser validates it
**Then** an `IngestError` is returned with a clear message about the invalid ID format

**Given** a spec missing required frontmatter fields (e.g., no `id` or `title`)
**When** the parser processes it
**Then** an `IngestError` is returned identifying the missing fields

**Given** a markdown file with no YAML frontmatter
**When** the parser processes it
**Then** an `IngestError` is returned indicating missing frontmatter

### Story 3.2: Unified Spec Ingestion Pipeline

As a developer,
I want a single ingestion pipeline that parses a spec, validates uniqueness, indexes it for search, and creates graph nodes and edges,
So that a spec flows from raw markdown into both stores atomically.

**Acceptance Criteria:**

**Given** a valid spec markdown string
**When** I call `add_spec(markdown)` on the ingestion pipeline
**Then** the spec is parsed, validated, and ingested into both the search index and causal graph (FR14)

**Given** a spec being ingested with `depends_on` references
**When** the ingestion completes
**Then** the spec content is indexed in Tantivy for full-text search (FR16)
**And** a graph node is created in DeepCausality/Fjall (FR17)
**And** causal edges are created for each `depends_on` entry (FR17)

**Given** a spec with an ID that already exists in the stores
**When** I attempt to ingest it
**Then** an `IngestError` is returned indicating duplicate spec ID (FR18)
**And** neither store is modified (no partial writes)

**Given** a valid spec
**When** ingestion executes
**Then** the entire operation completes in < 100ms (NFR7)

**Given** a spec with `depends_on` referencing a spec ID not yet in the graph
**When** the spec is ingested
**Then** the edge is created with the target ID (forward reference)
**And** the edge resolves when the target spec is later ingested

## Epic 4: Git-Centric Sync

Git is the single source of truth. The system performs full rebuilds from git tree walks and incremental sync via `git diff` with rename detection. Deleted specs are removed from indexes and graph. Rebuilds are idempotent. Both stores track the last-synced commit SHA.

### Story 4.1: Full Rebuild from Git Tree Walk

As a spec author,
I want to rebuild the entire search index and causal graph from the git repository,
So that I can recover from any data corruption and guarantee my indexes match the source of truth.

**Acceptance Criteria:**

**Given** a git repository containing spec files in the configured directory
**When** I trigger a full rebuild
**Then** the system walks the git tree, discovers all spec files, and ingests each through the pipeline (FR20)
**And** the rebuild produces identical indexes regardless of when it is run (idempotent) (FR24)

**Given** a full rebuild in progress
**When** the new indexes are ready
**Then** they are built in temporary directories and atomically swapped into place (NFR15)
**And** the old indexes are not modified until the swap succeeds

**Given** a repository with 100+ specs
**When** a full rebuild executes
**Then** it completes in < 5 seconds (NFR5)

**Given** a completed full rebuild
**When** the operation finishes
**Then** both Tantivy and Fjall stores record the current git commit SHA (FR25)
**And** both stores record the correct document count

**Given** a previous index exists with stale data
**When** I run a full rebuild
**Then** the old data is completely replaced — no remnants of stale specs remain

### Story 4.2: Incremental Sync via Git Diff

As a spec author,
I want changed specs automatically detected and re-indexed via git diff,
So that my updates are reflected in search and graph within seconds without a full rebuild.

**Acceptance Criteria:**

**Given** specs that have been modified since the last sync
**When** incremental sync runs
**Then** only the changed files are processed via `git diff` against the last-synced SHA (FR21)
**And** modified specs are re-parsed and re-indexed in both stores

**Given** a spec file that was renamed (path changed, content same or different)
**When** incremental sync runs with git rename detection (`-M` flag)
**Then** the renamed file is correctly identified and re-indexed without duplication (FR22)

**Given** a spec file that was deleted from the repository
**When** incremental sync runs
**Then** the spec is removed from both the search index and causal graph (FR23)
**And** edges referencing the deleted spec are cleaned up

**Given** a repository with a few changed files among 100+ specs
**When** incremental sync executes
**Then** it completes in < 2 seconds (NFR6)

**Given** incremental sync completes
**When** the operation finishes
**Then** the last-synced SHA is updated in both stores (FR25)
**And** document counts are compared across stores as a sanity check
**And** if counts diverge, the system auto-escalates to a full rebuild

## Epic 5: Intelligent Query Routing

Agents submit natural-language queries that are automatically classified and routed to search, causal reasoning, or both. When search returns zero results, the router still provides causal context. Composed results combine search hits with causal relationships.

### Story 5.1: Query Intent Classification

As an AI agent,
I want my natural-language queries automatically classified by intent,
So that they are routed to the correct engine without me needing to choose the right tool.

**Acceptance Criteria:**

**Given** a query containing causal signal words ("impact", "depends", "breaks", "affects", "upstream", "downstream")
**When** the classifier processes it
**Then** it is classified as a causal query and routed to the causal graph engine (FR11)

**Given** a query without causal signal words (e.g., "rate limiting API")
**When** the classifier processes it
**Then** it is classified as a search query and routed to the Tantivy search engine (FR11)

**Given** a query containing both causal signals and search terms (e.g., "what depends on rate limiting")
**When** the classifier processes it
**Then** it is classified as a hybrid query and routed to both engines (FR11)

**Given** any query
**When** classification executes
**Then** the overhead is < 5ms (NFR3)

### Story 5.2: Hybrid Query Execution & Result Composition

As an AI agent,
I want composed results that combine search hits with causal context, and causal fallback when search returns nothing,
So that I always get the richest possible answer to my question.

**Acceptance Criteria:**

**Given** a hybrid query routed to both engines
**When** both return results
**Then** I receive a composed response combining search results with causal context for each hit (FR13)
**And** search results include their causal edges where applicable

**Given** a search query that returns zero results
**When** the router processes the empty result
**Then** it falls back to the causal graph, traversing from related nodes to provide context (FR12)
**And** the response includes the causal context with an indication that no direct search matches were found

**Given** a causal query (e.g., "what breaks if I change spec::auth::jwt-validation")
**When** the router processes it
**Then** the causal engine result is returned directly without unnecessary search

**Given** a query where both engines return empty results
**When** the router processes it
**Then** I receive an empty result with a clear message — no fabricated results

## Epic 6: MCP Server & CLI

AI agents discover and call spec-db tools/resources via MCP over stdio (default) and optional streamable-http transport. Users manage spec-db via CLI commands: init (scaffold project), serve (start MCP server), sync (manual trigger), rebuild (destructive rebuild), and status (health check). All configuration reads from `.spec-db/config.yaml`.

### Story 6.1: Project Initialization & Configuration

As a spec author,
I want to initialize a new spec-db project with scaffolded structure and sensible defaults,
So that I can start writing specs immediately without manual setup.

**Acceptance Criteria:**

**Given** an empty directory
**When** I run `spec-db init`
**Then** a `specs/` directory is created with example spec files demonstrating frontmatter format, `depends_on` relationships, and tag conventions (FR33)
**And** a `.spec-db/config.yaml` is created with documented defaults (spec directory, data directory, transport settings)
**And** next-steps instructions are printed to stdout

**Given** a `.spec-db/config.yaml` file
**When** the system starts
**Then** all configuration is read from this file (FR38)
**And** missing optional fields use sensible defaults
**And** missing required fields produce a clear `ConfigError`

**Given** `spec-db init` is run in a directory that already has `.spec-db/config.yaml`
**When** the command executes
**Then** it warns the user and does not overwrite existing configuration

### Story 6.2: MCP Server with Tools over Stdio

As an AI agent,
I want to discover and call spec-db tools via MCP over stdio transport,
So that I can search, reason, and manage specs as a native capability.

**Acceptance Criteria:**

**Given** the MCP server started via `spec-db serve`
**When** an agent connects over stdio
**Then** all spec-db tools are discoverable via MCP protocol (FR28)
**And** the transport uses stdio with zero network configuration (FR26)

**Given** an agent calling `search_specs(query, filters?)`
**When** the tool executes
**Then** it delegates to the search engine via `spawn_blocking` and returns JSON results: `[{id, title, score, snippet}]`

**Given** an agent calling `get_spec(id)`
**When** the tool executes
**Then** it returns the full spec content as JSON

**Given** an agent calling `trace_impact(id, depth?)` or `find_dependencies(id)`
**When** the tool executes
**Then** it delegates to the causal engine and returns JSON: `{node, edges: [{from, to, type}]}`

**Given** an agent calling `query(natural_language)`
**When** the tool executes
**Then** it delegates to the query router and returns composed JSON results

**Given** an agent calling `add_spec(markdown)` or `sync(mode?)`
**When** the tool executes
**Then** it delegates to the ingestion/sync pipeline and returns JSON: `{status, message, details}`

**Given** any MCP tool call
**When** it executes
**Then** the response completes in < 100ms end-to-end (NFR8)
**And** errors return consistent JSON: `{error_type, message, context}` per architecture pattern F2

### Story 6.3: MCP Resources & Streamable-HTTP Transport

As an AI agent,
I want to read spec content and graph summaries via MCP resources, and optionally connect over HTTP,
So that I can access spec intelligence through multiple access patterns and transports.

**Acceptance Criteria:**

**Given** an agent requesting `spec://{id}`
**When** the resource is read
**Then** the full spec content is returned (FR29)

**Given** an agent requesting `graph://overview`
**When** the resource is read
**Then** it returns causal graph summary statistics: total specs, total edges, and a list of disconnected clusters (specs with no causal edges) (FR30, FR32)

**Given** an agent requesting `graph://node/{id}`
**When** the resource is read
**Then** it returns the spec node with all inbound and outbound edges (FR31)

**Given** streamable-http transport enabled in `.spec-db/config.yaml` with `http.auth_token` set
**When** an agent connects over HTTP
**Then** requests without a valid bearer token are rejected with 401 (FR27, NFR24)
**And** requests with a valid token are processed identically to stdio

**Given** streamable-http transport is not configured
**When** the server starts
**Then** only stdio transport is available — no network surface (NFR23)

### Story 6.4: CLI Administration Commands

As a spec author,
I want CLI commands to manage the MCP server, trigger syncs, rebuild indexes, and check system health,
So that I can operate spec-db without needing an AI agent.

**Acceptance Criteria:**

**Given** a configured spec-db project
**When** I run `spec-db serve`
**Then** the MCP server starts from `.spec-db/config.yaml` (FR34)
**And** an initial sync runs automatically if no index exists
**And** a cross-store consistency check runs before serving

**Given** a running spec-db project
**When** I run `spec-db sync`
**Then** an incremental sync is triggered (FR35)
**And** when I run `spec-db sync --full`, a full rebuild is triggered instead (FR35)

**Given** a spec-db project with existing indexes
**When** I run `spec-db rebuild`
**Then** a destructive full index rebuild executes (FR36)
**And** both stores are rebuilt from git (idempotent)

**Given** a spec-db project
**When** I run `spec-db status`
**Then** I see document count, last sync commit SHA, and consistency check result (FR37)
**And** the output clearly indicates whether stores are consistent or drifted

**Given** any CLI command
**When** it encounters an error
**Then** a human-readable error message is printed via `anyhow` — no stack traces unless `RUST_BACKTRACE=1`

## Epic 7: Data Integrity & Observability

Cross-store consistency is verified on startup and after every sync — comparing git SHA and doc counts across Tantivy and Fjall. Drift triggers warnings and auto-rebuild. OpenTelemetry traces and metrics are emitted for all key operations (search, traversal, sync, MCP calls, consistency checks).

### Story 7.1: Cross-Store Consistency Checks

As an operator,
I want the system to verify that Tantivy and Fjall are in sync on startup and after every sync operation,
So that agents never get stale or inconsistent results.

**Acceptance Criteria:**

**Given** the system starting up
**When** both stores are loaded
**Then** the system compares the last-synced git SHA and document count across Tantivy and Fjall (FR39, FR41)
**And** if both match, the system proceeds normally

**Given** a sync operation (full or incremental) completing
**When** post-sync verification runs
**Then** the system compares SHA and doc count across both stores (FR40, FR41)
**And** if both match, the sync is marked successful

**Given** startup or post-sync verification detects SHA or doc count mismatch
**When** drift is detected
**Then** the system emits a warning to stderr with details of the mismatch (FR42)
**And** offers to auto-rebuild from git to restore consistency (FR42)

**Given** incremental sync completes but document counts diverge between stores
**When** the sanity check runs
**Then** the system automatically escalates to a full rebuild (FR43)
**And** logs the escalation reason

**Given** a full rebuild triggered by auto-escalation
**When** the rebuild completes
**Then** consistency is re-verified
**And** if still inconsistent after rebuild, a clear error is raised (no infinite retry loop)

### Story 7.2: OpenTelemetry Traces & Metrics

As an operator,
I want OpenTelemetry traces and metrics emitted for all key operations,
So that I can monitor performance, diagnose issues, and track system health.

**Acceptance Criteria:**

**Given** OpenTelemetry export configured in `.spec-db/config.yaml`
**When** a search query executes
**Then** a trace span `spec_db.search.query` is emitted with query text and result count (FR44)
**And** a metric `spec_db.search.latency_ms` is recorded (FR45)

**Given** a graph traversal (trace_impact or find_dependencies)
**When** it executes
**Then** a trace span `spec_db.graph.traverse` is emitted with spec ID, direction, and depth (FR44)

**Given** a sync operation (full or incremental)
**When** it executes
**Then** a trace span `spec_db.sync.{mode}` is emitted with duration and spec count (FR44)
**And** a metric `spec_db.sync.duration_ms` is recorded (FR45)

**Given** any MCP tool call
**When** it executes
**Then** a trace span `spec_db.mcp.tool_call` is emitted with tool name and duration (FR44)

**Given** a consistency check detecting drift
**When** drift is found
**Then** a metric `spec_db.consistency.drift_detected` is incremented (FR46)
**And** a metric `spec_db.consistency.check_result` records pass/fail (FR45)

**Given** OpenTelemetry export is NOT configured
**When** the system runs
**Then** no telemetry is exported — zero external network calls (NFR25)
**And** local console logging via `tracing-subscriber` still works (human-readable format)

**Given** OpenTelemetry export is configured
**When** the system emits traces and metrics
**Then** they use standard OTLP protocol compatible with Jaeger, Grafana, and Datadog (NFR22)
