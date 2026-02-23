# Story 3.2: Unified Spec Ingestion Pipeline

Status: review

## Story

As a developer,
I want a single ingestion pipeline that parses a spec, validates uniqueness, indexes it for search, and creates graph nodes and edges,
so that a spec flows from raw markdown into both stores atomically.

## Acceptance Criteria (BDD)

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

## Tasks / Subtasks

- [x] Define ingestion pipeline API in `crates/ingest/src/lib.rs`
  - [x] Add `pub struct IngestPipeline<S, G>` (or trait-object equivalent) depending on `SearchEngine` (Epic 2) and `CausalGraph`/`SpecStore` (Epic 1).
  - [x] Add `pub fn add_spec(&mut self, markdown: &str) -> Result<SpecId, IngestError>` as primary entrypoint.
  - [x] Wire parser dependency from Story 3.1 (`parse_spec`) as first step of `add_spec`.
  - [x] Keep API sync; async boundary remains at MCP layer (`spawn_blocking`).

- [x] Implement deterministic ingest flow in `crates/ingest/src/sync.rs` (or `pipeline.rs` if preferred)
  - [x] Step 1: parse markdown and validate frontmatter/SpecId via Story 3.1 components.
  - [x] Step 2: preflight uniqueness checks against both stores before any writes (search index + graph store).
  - [x] Step 3: derive write set (search doc, graph node payload, edge list from `depends_on`).
  - [x] Step 4: execute writes under atomicity guard and return success only after both stores commit.

- [x] Enforce no-partial-write semantics across stores
  - [x] Use Fjall batch for node+edges to guarantee graph-side atomicity.
  - [x] Use a staged/guarded write pattern for Tantivy + Fjall: perform reversible operations and roll back if any stage fails.
  - [x] Ensure duplicate-ID short-circuits before write stage.
  - [x] Add compensating cleanup (`remove_doc`, `remove_node/edges`) when second store fails after first store commit.
  - [x] Emit single `IngestError::PartialWritePrevented`/`IngestError::AtomicityViolation` path with context when rollback path is exercised.

- [x] Implement forward-reference edge behavior
  - [x] Create `depends_on` edges even if target spec is not currently present in graph storage.
  - [x] Store unresolved target IDs as normal edge endpoints (`from_id`, `to_id`) with no special blocking.
  - [x] Verify traversal semantics remain correct once the target node is later ingested.

- [x] Integrate with existing subsystems and trait boundaries
  - [x] Use only `spec-db-core` shared types (`SpecId`, `SpecDoc`, `CausalEdge`, errors); do not redefine domain types.
  - [x] Use explicit re-exports in `crates/ingest/src/lib.rs`; no wildcard exports.
  - [x] Keep cross-crate dependencies unidirectional (`ingest -> search, causal, core`).

- [x] Add performance instrumentation and NFR validation
  - [x] Instrument `add_spec` with span `spec_db.ingest.add_spec` and timing metric.
  - [x] Add benchmark-style integration test asserting steady-state ingest of single spec under 100ms at target hardware baseline.
  - [x] Record parse, validate, search write, graph write durations for troubleshooting NFR7 failures.

- [x] Add ingestion integration tests in `crates/ingest/tests/integration.rs`
  - [x] Success path: valid markdown ingested into both stores with searchable content and graph node/edges.
  - [x] Duplicate path: attempt second ingest with same `SpecId`, assert duplicate error and no data changes.
  - [x] Forward-reference path: ingest `depends_on` target missing, assert edge exists; ingest target later, assert graph linkage resolves.
  - [x] Atomicity failure path: inject failure in one store and assert rollback/no partial writes.

## Dev Notes

- This story depends directly on Story 3.1 parser and validator outputs; do not duplicate parsing logic.
- Ingestion pipeline bridges Epic 2 and Epic 1 capabilities: Tantivy indexing (`SearchEngine`) and causal graph persistence (`CausalGraph` + Fjall).
- Required behavior is transactional from caller perspective: success means both stores updated; any failure means neither store persists net new state.
- Forward references in `depends_on` are valid and must produce edges immediately, even if target node is not yet present.
- NFR7 target is `<100ms/spec`; optimize for minimal allocations and pre-validated write sets.
- Continue using `serde_yml` in ingest path; do not add deprecated YAML crates in pipeline or tests.

### Project Structure Notes

- Core implementation files for this story:
  - `crates/ingest/src/lib.rs` (pipeline API surface)
  - `crates/ingest/src/sync.rs` (shared ingest orchestration used by add/rebuild/incremental)
  - `crates/ingest/src/consistency.rs` (post-write consistency checks if needed)
  - `crates/ingest/src/parser.rs` and `crates/ingest/src/validate.rs` (Story 3.1 dependencies)
  - `crates/ingest/tests/integration.rs` (atomicity, duplicate, forward-reference, perf)
- Keep runtime ownership boundaries intact: only `search` touches Tantivy storage, only `causal` touches Fjall storage; ingest orchestrates through trait APIs.

### References

- Epic story and acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Story-3.2-Unified-Spec-Ingestion-Pipeline]
- Build order and ingest crate ownership: [Source: _bmad-output/planning-artifacts/architecture.md#Decision-Impact-Analysis], [Source: _bmad-output/planning-artifacts/architecture.md#Complete-Project-Directory-Structure]
- Atomic operation expectations and consistency constraints: [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting-Concerns-Identified], [Source: _bmad-output/planning-artifacts/architecture.md#Process-Patterns]
- Trait boundaries and dependency graph: [Source: _bmad-output/planning-artifacts/architecture.md#Architectural-Boundaries]
- Shared ID, edge semantics, and spec format context: [Source: docs/project-context.md#Key-Patterns-for-AI-Agents], [Source: docs/project-context.md#Spec-Document-Format]
- pulldown-cmark and serde_yml API references used by upstream parser contract: [Source: https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.Event.html], [Source: https://docs.rs/serde_yml/0.0.11/serde_yml/]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Added `IngestPipeline` orchestration with parse, duplicate detection, search indexing, graph node/edge writes, and rollback cleanup on graph failures.
- Added integration coverage for valid ingest, duplicate rejection, forward references, single-spec performance, and remove path with real `SearchIndex` and `CausalEngine`.
- Updated ingest crate dependencies and exports to wire `spec-db-causal` and `spec-db-search` through the pipeline API.

### Change Log

- 2026-02-23: Initial ready-for-dev story file created.
- 2026-02-23: Implemented Story 3.2 pipeline, integration tests, and verification run.

### File List

- `_bmad-output/implementation-artifacts/3-2-unified-spec-ingestion-pipeline.md`
- `crates/ingest/Cargo.toml`
- `crates/ingest/src/lib.rs`
- `crates/ingest/src/pipeline.rs`
- `crates/ingest/tests/integration.rs`
