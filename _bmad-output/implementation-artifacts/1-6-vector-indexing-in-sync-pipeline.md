# Story 1.6: Vector Indexing in Sync Pipeline

Status: ready-for-dev

## Story

As a lattice operator,
I want `lattice sync` to automatically generate embeddings and index specs into configured vector backends,
so that vector search is populated without manual steps.

## Acceptance Criteria

1. During `lattice sync`, specs are: parsed → embedded → indexed to FTS AND vector backend
2. `lattice sync --full` rebuilds vector index from scratch
3. `lattice rebuild` also rebuilds vector backends
4. Embedding generation runs in `spawn_blocking`
5. If no vector backend configured, sync proceeds as before (FTS only)
6. Errors from embedding/vector indexing are logged but do not block FTS sync
7. Integration test verifies specs appear in both Tantivy and LanceDB after sync

## Tasks / Subtasks

- [ ] Task 1: Extend `IngestPipeline` to accept optional vector components (AC: #1, #5)
  - [ ] Add `embedding_provider: Option<Box<dyn EmbeddingProvider>>` parameter
  - [ ] Add `vector_backends: Option<&mut BackendRegistry>` parameter
  - [ ] Update `crates/ingest/Cargo.toml` with deps on `spec-db-embedding`, `spec-db-search-vector`
- [ ] Task 2: Extend sync pipeline (AC: #1, #4, #6)
  - [ ] After parsing each spec, if embedding_provider exists: generate embedding
  - [ ] If vector_backends exists: call `index_spec(doc, embedding)` on each backend
  - [ ] Wrap embedding generation in `spawn_blocking` at async boundary
  - [ ] Log + continue on embedding/vector errors (non-fatal)
- [ ] Task 3: Full rebuild support (AC: #2, #3)
  - [ ] On `--full` or `rebuild`: clear vector backends before re-indexing
  - [ ] Call `remove_spec` for all existing then re-index, OR drop/recreate table
- [ ] Task 4: Integration test (AC: #7)
  - [ ] Set up Tantivy + LanceDB + LocalEmbedding in test
  - [ ] Sync sample specs
  - [ ] Verify searchable in both FTS and vector

## Dev Notes

### Existing Sync Pipeline

From `crates/ingest/src/pipeline.rs` and `crates/ingest/src/sync.rs`:
- `IngestPipeline` coordinates: parse markdown → index to Tantivy → upsert graph nodes
- `GitSync` handles git diff detection for incremental sync
- Pipeline is called from CLI commands in `src/main.rs`

Key pattern: pipeline methods are sync, called within `spawn_blocking` from CLI async handlers.

### Architecture Compliance

- [Source: architecture.md#Async Boundary Pattern] — `spawn_blocking` for sync ops
- [Source: architecture.md#Boundary Rules] — "Ingest orchestrates: parse → embed → index to FTS + vector"
- [Source: architecture.md#Boundary Rules] — "Backends do NOT call embedding; ingest layer pre-computes"

### CRITICAL: What NOT To Do

- Do NOT create new CLI commands (that's Epic 4)
- Do NOT modify existing SearchEngine/Tantivy code
- Do NOT make embedding failures fatal — log and continue
- Do NOT add routing logic to sync — index to ALL configured backends

### References

- [Source: crates/ingest/src/lib.rs] — IngestPipeline, GitSync exports
- [Source: crates/ingest/src/pipeline.rs] — Existing pipeline flow
- [Source: architecture.md#Boundary Rules] — Ingest orchestration pattern

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
