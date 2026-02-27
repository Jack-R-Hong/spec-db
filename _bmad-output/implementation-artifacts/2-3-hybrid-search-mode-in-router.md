# Story 2.3: Hybrid Search Mode in Router

Status: ready-for-dev

## Story

As an AI agent,
I want to search specs using hybrid mode that combines FTS and vector results,
so that I find both keyword-matched and semantically related specs.

## Acceptance Criteria

1. Hybrid search: embed query → `search_scored()` on Tantivy → `search()` on vector → merge via RRF
2. Router accepts `mode`: `fts` (default), `vector`, `hybrid`
3. `mode=vector` queries only vector backend
4. `mode=hybrid` merges FTS + vector via RRF
5. `mode=fts` behaves exactly as before
6. Tag filtering works in all modes
7. No vector backend + vector/hybrid mode → `BackendNotFound` error
8. Integration test: hybrid returns specs from both sources

## Tasks / Subtasks

- [ ] Task 1: Add `SearchMode` enum (AC: #2)
  - [ ] Create enum in `crates/router/src/classifier.rs`: `Fts`, `Vector`, `Hybrid`
- [ ] Task 2: Extend `QueryRouter` (AC: #1, #3, #4, #5, #7)
  - [ ] Add `vector: Option<BackendRegistry>` and `embedding: Option<Box<dyn EmbeddingProvider>>` fields
  - [ ] Add `search_with_mode(query, mode, tags, limit, agent_context)` method
  - [ ] Implement `execute_vector_search` — embed query, call vector backend
  - [ ] Implement `execute_hybrid_search` — FTS scored + vector, merge with RRF
  - [ ] Keep existing `query()` method unchanged (backward compatible)
- [ ] Task 3: Tag filtering in all modes (AC: #6)
  - [ ] FTS: `search_with_tags` (existing)
  - [ ] Vector: `search_with_tags` on vector backend
  - [ ] Hybrid: both filtered, then RRF merge
- [ ] Task 4: Update Cargo.toml (AC: #1)
  - [ ] Add deps: `spec-db-search-vector`, `spec-db-embedding` to router
- [ ] Task 5: Integration test (AC: #8)
  - [ ] Set up Tantivy + LanceDB + embeddings
  - [ ] Index sample specs
  - [ ] Verify hybrid returns results from both sources

## Dev Notes

### Existing Router Pattern

From `crates/router/src/lib.rs`:
```rust
pub struct QueryRouter<S: SearchEngine, C: CausalGraph> {
    search: S,
    graph: C,
}
```

**Extend** (do not replace):
```rust
pub struct QueryRouter<S: SearchEngine, C: CausalGraph> {
    search: S,
    graph: C,
    vector: Option<BackendRegistry>,
    embedding: Option<Box<dyn EmbeddingProvider>>,
}
```

The new `search_with_mode()` is a separate method from existing `query()`. Existing `query()` remains unchanged.

### Architecture Compliance

- [Source: architecture.md#Decision 2] — `QueryRouter` gets `vector: Option<BackendRegistry>`
- [Source: architecture.md#Decision 4] — RRF for hybrid merge
- [Source: architecture.md#Async Boundary Pattern] — sync operations, `spawn_blocking` at boundary

### CRITICAL: What NOT To Do

- Do NOT modify existing `query()` method or `QueryIntent` enum
- Do NOT add agent routing logic (that's Story 3.3)
- Do NOT modify `ComposedQueryResult` — new method returns `Vec<ScoredHit>` directly

### References

- [Source: crates/router/src/lib.rs] — QueryRouter, existing execute_search/hybrid
- [Source: crates/router/src/classifier.rs] — QueryIntent enum
- [Source: architecture.md#Decision 2] — Router extension
- [Source: architecture.md#Decision 4] — RRF merge

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
