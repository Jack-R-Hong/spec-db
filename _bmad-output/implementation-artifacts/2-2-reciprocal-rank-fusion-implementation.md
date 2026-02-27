# Story 2.2: Reciprocal Rank Fusion Implementation

Status: ready-for-dev

## Story

As a lattice developer,
I want a Reciprocal Rank Fusion (RRF) function that merges two ranked result lists,
so that hybrid search can combine FTS and vector results into a single ranking.

## Acceptance Criteria

1. `reciprocal_rank_fusion(fts: &[ScoredHit], vector: &[ScoredHit], k: u32) -> Vec<ScoredHit>` in `crates/search-vector/src/fusion.rs`
2. Fused score: `Σ 1/(k + rank_i)` for each unique spec ID across both lists
3. Default `k` = 60
4. Results sorted by fused score descending
5. Specs in both lists get higher scores than single-list specs
6. Handles empty inputs gracefully
7. Unit tests: both lists, single-list, empty inputs

## Tasks / Subtasks

- [ ] Task 1: Implement RRF function (AC: #1, #2, #3, #4)
  - [ ] Create `crates/search-vector/src/fusion.rs`
  - [ ] Build rank maps: `HashMap<SpecId, f32>` accumulating `1/(k + rank)`
  - [ ] Merge, sort descending, return `Vec<ScoredHit>`
- [ ] Task 2: Add convenience wrapper (AC: #3)
  - [ ] `hybrid_merge(fts: &[ScoredHit], vector: &[ScoredHit]) -> Vec<ScoredHit>` with default k=60
- [ ] Task 3: Update lib.rs
  - [ ] Re-export `fusion::reciprocal_rank_fusion` and `fusion::hybrid_merge`
- [ ] Task 4: Unit tests (AC: #5, #6, #7)
  - [ ] Test: 3 FTS + 3 vector results, 1 overlap → overlap has highest score
  - [ ] Test: FTS only, empty vector → returns FTS items
  - [ ] Test: empty both → returns empty
  - [ ] Test: single item in each → merged correctly

## Dev Notes

### RRF Algorithm

```rust
use std::collections::HashMap;
use spec_db_core::{ScoredHit, SpecId};

pub fn reciprocal_rank_fusion(
    fts_results: &[ScoredHit],
    vector_results: &[ScoredHit],
    k: u32,
) -> Vec<ScoredHit> {
    let mut scores: HashMap<&SpecId, f32> = HashMap::new();

    for (rank, hit) in fts_results.iter().enumerate() {
        *scores.entry(&hit.id).or_insert(0.0) += 1.0 / (k as f32 + rank as f32 + 1.0);
    }
    for (rank, hit) in vector_results.iter().enumerate() {
        *scores.entry(&hit.id).or_insert(0.0) += 1.0 / (k as f32 + rank as f32 + 1.0);
    }

    let mut results: Vec<ScoredHit> = scores
        .into_iter()
        .map(|(id, score)| ScoredHit { id: id.clone(), score })
        .collect();
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}
```

**Note**: Rank is 1-indexed in the formula: `1/(k + rank)` where rank starts at 1. Adjust for 0-indexed iteration.

### Architecture Compliance

- [Source: architecture.md#Decision 4] — RRF with k=60, exact formula
- [Source: architecture.md#Decision 4] — Score-distribution agnostic

### CRITICAL: What NOT To Do

- Do NOT normalize individual backend scores — RRF is rank-based, not score-based
- Do NOT import or depend on Tantivy or LanceDB — this is pure algorithm
- `SpecId` needs `Clone` and `Hash` (already has both from types.rs)

### References

- [Source: architecture.md#Decision 4] — RRF design and rationale
- [Source: crates/core/src/types.rs] — ScoredHit, SpecId types

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
