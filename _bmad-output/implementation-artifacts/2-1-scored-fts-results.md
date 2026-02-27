# Story 2.1: Scored FTS Results

Status: ready-for-dev

## Story

As a lattice developer,
I want the existing Tantivy `SearchEngine` to return scored results,
so that FTS scores can participate in hybrid search fusion.

## Acceptance Criteria

1. `search_scored(&self, query: &str, limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>` added as default method on `SearchEngine`
2. Default implementation delegates to `search()`, returns `ScoredHit { id, score: 0.0 }`
3. Tantivy `SearchIndex` overrides `search_scored()` to return actual BM25 scores
4. Existing `search()` and `search_with_tags()` unchanged
5. All existing tests pass
6. Unit test verifies `search_scored()` returns non-zero scores

## Tasks / Subtasks

- [ ] Task 1: Add `search_scored()` default to `SearchEngine` trait (AC: #1, #2)
  - [ ] Import `ScoredHit` in `crates/core/src/traits.rs`
  - [ ] Add default method that wraps `self.search()` with score 0.0
- [ ] Task 2: Implement scored search in Tantivy `SearchIndex` (AC: #3)
  - [ ] Modify `crates/search/src/query.rs` — `execute_search` already returns `SearchHit` with score
  - [ ] Override `search_scored()` in `impl SearchEngine for SearchIndex` in `crates/search/src/lib.rs`
  - [ ] Convert `SearchHit { id, score }` → `ScoredHit { id, score }`
- [ ] Task 3: Verify backward compatibility (AC: #4, #5)
  - [ ] Run `cargo test -p spec-db-search` — all existing tests must pass
- [ ] Task 4: New test (AC: #6)
  - [ ] Test that `search_scored()` returns scores > 0.0 for matching docs

## Dev Notes

### Existing Search Implementation

From `crates/search/src/lib.rs`:
- `SearchIndex` implements `SearchEngine` with `search()` returning `Vec<SpecId>`
- `query::execute_search()` already returns `Vec<SearchHit>` where `SearchHit` has `id` and `score` fields
- Current `search()` discards scores: `hits.into_iter().map(|hit| hit.id).collect()`

**Key insight**: Scores already exist internally. `search_scored()` just needs to preserve them.

From `crates/search/src/query.rs`:
```rust
pub struct SearchHit {
    pub id: SpecId,
    pub score: f32,  // BM25 score from Tantivy
}
```

### Architecture Compliance

- [Source: architecture.md#Decision 1] — `search_scored()` as default method, backward compatible
- [Source: architecture.md#Decision 4] — Scores needed for RRF fusion
- [Source: architecture.md#Decision 1] — Default returns score 0.0 for non-overriding impls

### CRITICAL: What NOT To Do

- Do NOT modify existing `search()` or `search_with_tags()` return types
- Do NOT change existing test assertions
- Do NOT add `search_scored_with_tags()` — not needed for MVP
- `SearchIndex` already has `SearchHit` with score internally — reuse it

### References

- [Source: crates/search/src/lib.rs] — SearchEngine impl, SearchIndex
- [Source: crates/search/src/query.rs] — SearchHit struct, execute_search function
- [Source: crates/core/src/traits.rs] — SearchEngine trait
- [Source: architecture.md#Decision 1] — search_scored() design

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
