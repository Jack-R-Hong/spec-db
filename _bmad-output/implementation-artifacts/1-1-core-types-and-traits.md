# Story 1.1: Core Types and Traits

Status: ready-for-dev

## Story

As a lattice developer,
I want core types (`ScoredHit`, `VectorSearchBackend`, `EmbeddingProvider`) and error variants defined in the `core` crate,
so that all downstream crates have a stable foundation to implement against.

## Acceptance Criteria

1. `ScoredHit` struct with `id: SpecId` and `score: f32` added to `crates/core/src/types.rs`
2. `VectorSearchBackend` trait added to `crates/core/src/traits.rs` with methods:
   - `index_spec(&mut self, doc: &SpecDoc, embedding: &[f32]) -> Result<(), SpecDbError>`
   - `remove_spec(&mut self, id: &SpecId) -> Result<(), SpecDbError>`
   - `search(&self, embedding: &[f32], limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>`
   - `search_with_tags(&self, embedding: &[f32], tags: &[String], limit: usize) -> Result<Vec<ScoredHit>, SpecDbError>`
3. `EmbeddingProvider` trait added to `crates/core/src/traits.rs` with methods:
   - `embed(&self, text: &str) -> Result<Vec<f32>, SpecDbError>`
   - `embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SpecDbError>`
   - `dimensions(&self) -> usize`
   - `model_name(&self) -> &str`
4. `SpecDbError` extended with: `VectorError(String)`, `EmbeddingError(String)`, `BackendNotFound(String)`, `RoutingError(String)`
5. All traits are `Send + Sync`
6. `cargo build` succeeds with no warnings on core crate

## Tasks / Subtasks

- [ ] Task 1: Add `ScoredHit` struct to `types.rs` (AC: #1)
  - [ ] Add `#[derive(Debug, Clone, Serialize, Deserialize)]` on `ScoredHit`
  - [ ] Add `PartialEq` for test assertions
- [ ] Task 2: Add error variants to `error.rs` (AC: #4)
  - [ ] Add `VectorError(String)` variant
  - [ ] Add `EmbeddingError(String)` variant
  - [ ] Add `BackendNotFound(String)` variant
  - [ ] Add `RoutingError(String)` variant
- [ ] Task 3: Add `VectorSearchBackend` trait to `traits.rs` (AC: #2, #5)
  - [ ] Import `ScoredHit` in traits.rs
  - [ ] Define trait with all 4 methods
  - [ ] Ensure `: Send + Sync` bound
- [ ] Task 4: Add `EmbeddingProvider` trait to `traits.rs` (AC: #3, #5)
  - [ ] Define trait with all 4 methods
  - [ ] Ensure `: Send + Sync` bound
- [ ] Task 5: Update `crates/core/src/lib.rs` re-exports (AC: #6)
  - [ ] Re-export new types and traits
- [ ] Task 6: Verify build (AC: #6)
  - [ ] `cargo build -p spec-db-core` passes with zero warnings

## Dev Notes

### Existing Patterns — MUST FOLLOW

**Error pattern** (from `crates/core/src/error.rs`):
```rust
#[derive(thiserror::Error, Debug)]
pub enum SpecDbError {
    #[error("search error: {0}")]
    SearchError(String),
    // ... existing variants
}
```
New variants MUST use same `#[error("lowercase label: {0}")] VariantName(String)` pattern.

**Trait pattern** (from `crates/core/src/traits.rs`):
- Existing traits (`SearchEngine`, `CausalGraph`, `SpecStore`) do NOT have `Send + Sync` bounds
- Architecture decision D1 requires new traits to be `Send + Sync` for `Box<dyn VectorSearchBackend>`
- This is a NEW pattern for new traits only — do NOT modify existing traits

**Type pattern** (from `crates/core/src/types.rs`):
- `#[derive(Debug, Clone, Serialize, Deserialize)]` on all types
- `SpecId` is used as the primary ID across all subsystems

### Architecture Compliance

- [Source: architecture.md#Decision 1] — `ScoredHit { id: SpecId, score: f32 }`
- [Source: architecture.md#Decision 1] — `VectorSearchBackend` trait exact signatures
- [Source: architecture.md#Decision 3] — `EmbeddingProvider` trait exact signatures
- [Source: architecture.md#Error Handling Pattern] — Extend `SpecDbError` enum, do NOT create new error enums

### CRITICAL: What NOT To Do

- Do NOT modify `SearchEngine`, `CausalGraph`, or `SpecStore` traits
- Do NOT add `search_scored()` to `SearchEngine` yet (that's Story 2.1)
- Do NOT add config types yet (that's Story 1.5)
- Do NOT add any dependencies to core's Cargo.toml — `serde`, `thiserror` already present

### Project Structure Notes

- Edit only: `crates/core/src/types.rs`, `crates/core/src/traits.rs`, `crates/core/src/error.rs`, `crates/core/src/lib.rs`
- Crate name: `spec-db-core` (see `crates/core/Cargo.toml`)
- Workspace edition: 2024, rust-version: 1.85

### References

- [Source: crates/core/src/traits.rs] — Existing SearchEngine, CausalGraph, SpecStore traits
- [Source: crates/core/src/types.rs] — SpecId, SpecDoc, SpecNode, CausalEdge types
- [Source: crates/core/src/error.rs] — SpecDbError enum with 6 existing variants
- [Source: architecture.md#Decision 1] — VectorSearchBackend trait design
- [Source: architecture.md#Decision 3] — EmbeddingProvider trait design
- [Source: architecture.md#Error Handling Pattern] — Error variant naming convention

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
