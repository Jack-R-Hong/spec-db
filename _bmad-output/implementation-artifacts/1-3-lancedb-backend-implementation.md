# Story 1.3: LanceDB Backend Implementation

Status: ready-for-dev

## Story

As a lattice operator,
I want a LanceDB vector search backend,
so that I can perform vector similarity search on indexed specs.

## Acceptance Criteria

1. New crate `crates/search-vector/` created (package name `spec-db-search-vector`)
2. `LanceDbBackend` struct implements `VectorSearchBackend` trait
3. `LanceDbBackend::new(path, dimensions)` opens/creates LanceDB database
4. `index_spec(doc, embedding)` stores spec ID, embedding, title, tags in LanceDB table
5. `remove_spec(id)` deletes record by spec ID
6. `search(embedding, limit)` returns top-N as `Vec<ScoredHit>` with cosine similarity
7. `search_with_tags(embedding, tags, limit)` filters by tags
8. All operations sync; errors converted to `SpecDbError::VectorError`
9. Integration tests pass using `tempfile::TempDir`

## Tasks / Subtasks

- [ ] Task 1: Create crate structure (AC: #1)
  - [ ] Create `crates/search-vector/Cargo.toml`
  - [ ] Add to workspace members in root `Cargo.toml`
  - [ ] Create `src/lib.rs` with re-exports
- [ ] Task 2: Define LanceDB table schema (AC: #4)
  - [ ] Arrow schema: `id` (Utf8), `title` (Utf8), `tags` (Utf8/JSON), `vector` (FixedSizeList<Float32>)
  - [ ] Create table if not exists, open if exists
- [ ] Task 3: Implement `LanceDbBackend` (AC: #2, #3, #4, #5, #6, #7)
  - [ ] Create `crates/search-vector/src/lancedb.rs`
  - [ ] `new()` — connect + open_or_create table
  - [ ] `index_spec()` — build RecordBatch, add to table
  - [ ] `remove_spec()` — delete where id = ?
  - [ ] `search()` — vector search with cosine, return ScoredHit
  - [ ] `search_with_tags()` — add SQL WHERE filter for tags
- [ ] Task 4: Error handling (AC: #8)
  - [ ] Map lancedb errors to `SpecDbError::VectorError`
- [ ] Task 5: Integration tests (AC: #9)
  - [ ] Create `crates/search-vector/tests/lancedb_test.rs`
  - [ ] Test index, search, remove, search_with_tags

## Dev Notes

### Library: LanceDB Rust SDK

**CRITICAL**: LanceDB Rust API is async. But our trait is sync per architecture. Solution: use `tokio::runtime::Handle::current().block_on()` inside sync methods, OR create the backend with a runtime handle and use `block_on`. The backend will be called inside `spawn_blocking` from the async boundary, so blocking is safe.

```toml
[dependencies]
spec-db-core = { path = "../core" }
lancedb = "0.16"          # Latest stable
arrow-array = "54"
arrow-schema = "54"
serde_json = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tempfile = "3"
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

**LanceDB Rust API patterns:**
```rust
// Connect (async)
let db = lancedb::connect(path).execute().await?;

// Create table from Arrow RecordBatch
let schema = Arc::new(Schema::new(vec![
    Field::new("id", DataType::Utf8, false),
    Field::new("title", DataType::Utf8, false),
    Field::new("tags", DataType::Utf8, true),  // JSON-encoded Vec<String>
    Field::new("vector", DataType::FixedSizeList(
        Arc::new(Field::new("item", DataType::Float32, true)), dimensions as i32
    ), false),
]));
let batches = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());
let table = db.create_table("specs", batches).execute().await?;

// Search (async)
use lancedb::query::{ExecutableQuery, QueryBase};
let results = table.vector_search(query_vec)?
    .distance_type(DistanceType::Cosine)
    .limit(limit)
    .execute().await?
    .try_collect::<Vec<_>>().await?;

// Filter: WHERE clause string
let results = table.vector_search(query_vec)?
    .distance_type(DistanceType::Cosine)
    .only_if("tags LIKE '%\"security\"%'")
    .limit(limit)
    .execute().await?;
```

### Architecture Compliance

- [Source: architecture.md#Decision 1] — VectorSearchBackend trait signatures
- [Source: architecture.md#Decision 5] — New crate `crates/search-vector/`
- [Source: architecture.md#Decision 6] — MVP tag filtering only (SQL WHERE on tags column)
- [Source: architecture.md#Async Boundary Pattern] — sync trait, `spawn_blocking` at boundary

### Tag Storage Strategy

Store tags as JSON string: `serde_json::to_string(&tags)`. Filter with SQL LIKE: `tags LIKE '%"tagname"%'`. This avoids complex Arrow list types for MVP.

### CRITICAL: What NOT To Do

- Do NOT create BackendRegistry yet (that's Story 1.4)
- Do NOT add config types (that's Story 1.5)
- Do NOT add fusion/RRF (that's Story 2.2)
- Do NOT use external index creation — LanceDB auto-indexes for small datasets

### Project Structure Notes

```
crates/search-vector/
  Cargo.toml
  src/
    lib.rs          # Re-exports
    lancedb.rs      # LanceDbBackend implementation
  tests/
    lancedb_test.rs
```

### References

- [Source: architecture.md#Decision 1] — VectorSearchBackend trait
- [Source: architecture.md#Decision 5] — Crate structure
- [Source: architecture.md#Decision 6] — Tag filter strategy
- [Source: crates/core/src/types.rs] — SpecDoc, SpecId types
- [Source: GitHub lancedb/lancedb/rust/] — Rust SDK examples

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
