# Story 1.2: Local Embedding Provider

Status: ready-for-dev

## Story

As a lattice operator,
I want a local embedding provider using fastembed-rs,
so that I can generate embeddings without external API dependencies.

## Acceptance Criteria

1. New crate `crates/embedding/` created with `Cargo.toml` (package name `spec-db-embedding`)
2. `LocalEmbedding` struct implements `EmbeddingProvider` trait from core
3. `LocalEmbedding::new(model_name: &str)` loads the fastembed model (default: `all-MiniLM-L6-v2`)
4. `embed(text)` returns `Vec<f32>` of correct dimensions (384 for MiniLM)
5. `embed_batch(texts)` returns embeddings for multiple texts
6. `dimensions()` returns model dimension, `model_name()` returns configured model name
7. All operations are sync (wrapped in `spawn_blocking` at async boundary by caller)
8. Errors from fastembed converted to `SpecDbError::EmbeddingError`
9. Integration tests pass using temp directory for model cache

## Tasks / Subtasks

- [ ] Task 1: Create crate structure (AC: #1)
  - [ ] Create `crates/embedding/Cargo.toml` with deps: `spec-db-core`, `fastembed = "5"`
  - [ ] Add `"crates/embedding"` to workspace members in root `Cargo.toml`
  - [ ] Create `crates/embedding/src/lib.rs` with re-exports
- [ ] Task 2: Implement `LocalEmbedding` (AC: #2, #3, #4, #5, #6)
  - [ ] Create `crates/embedding/src/local.rs`
  - [ ] Implement `EmbeddingProvider` for `LocalEmbedding`
  - [ ] Store `fastembed::TextEmbedding` model + metadata
- [ ] Task 3: Error conversion (AC: #8)
  - [ ] Map fastembed errors to `SpecDbError::EmbeddingError(msg)`
- [ ] Task 4: Integration tests (AC: #9)
  - [ ] Create `crates/embedding/tests/local_test.rs`
  - [ ] Test embed single text, embed batch, dimensions, model_name
  - [ ] Use `#[ignore]` attribute (requires model download)

## Dev Notes

### Library: fastembed-rs v5

```toml
[dependencies]
fastembed = "5"
```

```rust
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

// Initialize
let model = TextEmbedding::try_new(
    InitOptions::new(EmbeddingModel::AllMiniLML6V2)
        .with_show_download_progress(false)
)?;

// Embed
let embeddings = model.embed(vec!["text here"], None)?;
// embeddings[0] is Vec<f32>, len = 384
```

- `TextEmbedding::try_new()` downloads model on first run (cached after)
- `model.embed(texts, batch_size)` — `None` for default batch size (256)
- All operations are sync — no async needed

### Architecture Compliance

- [Source: architecture.md#Decision 3] — Independent `EmbeddingProvider` trait, `LocalEmbedding` wraps fastembed-rs
- [Source: architecture.md#Decision 5] — New crate `crates/embedding/`
- [Source: architecture.md#Async Boundary Pattern] — All ops sync, `spawn_blocking` at caller
- [Source: architecture.md#Naming Patterns] — Crate name: `spec-db-embedding`

### Crate Dependencies

```toml
[package]
name = "spec-db-embedding"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
spec-db-core = { path = "../core" }
fastembed = "5"

[dev-dependencies]
tempfile = "3"
```

### CRITICAL: What NOT To Do

- Do NOT add OpenAI/remote embedding yet (that's Story 5.3)
- Do NOT add config deserialization (that's Story 1.5)
- Do NOT make any async functions — trait is sync per architecture decision
- Do NOT modify the core crate

### Project Structure Notes

```
crates/embedding/
  Cargo.toml
  src/
    lib.rs          # Re-exports: pub use local::LocalEmbedding;
    local.rs        # LocalEmbedding implementation
  tests/
    local_test.rs   # Integration tests
```

### References

- [Source: architecture.md#Decision 3] — EmbeddingProvider trait design
- [Source: architecture.md#Decision 5] — Crate organization
- [Source: architecture.md#Structure Patterns] — File layout convention
- [Source: crates/core/src/traits.rs] — EmbeddingProvider trait (added in Story 1.1)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
