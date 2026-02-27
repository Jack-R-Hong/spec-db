# Story 5.3: Remote Embedding Provider (OpenAI)

Status: ready-for-dev

## Story

As a lattice operator,
I want to use OpenAI's embedding API for higher quality vectors,
so that semantic search returns more relevant results for domain-specific content.

## Acceptance Criteria

1. `RemoteEmbedding` struct in `crates/embedding/src/remote.rs` implements `EmbeddingProvider`
2. `RemoteEmbedding::new(api_key, model, dimensions)` configures OpenAI client
3. `embed(text)` calls OpenAI embedding API, returns vector
4. `embed_batch(texts)` sends batch requests
5. Errors converted to `SpecDbError::EmbeddingError` with descriptive messages
6. API key never logged or in error messages (NFR7)
7. 3 retries with exponential backoff on network errors
8. Works with `text-embedding-3-small` and `text-embedding-ada-002`
9. Integration test with mock HTTP server

## Tasks / Subtasks

- [ ] Task 1: Add async-openai dependency (AC: #1)
  - [ ] Add to `crates/embedding/Cargo.toml`: `async-openai = "0.27"`, `tokio`
- [ ] Task 2: Implement `RemoteEmbedding` (AC: #2, #3, #4, #8)
  - [ ] Create `crates/embedding/src/remote.rs`
  - [ ] Wrap `async_openai::Client` with `CreateEmbeddingRequestArgs`
  - [ ] `embed()` — single text request using `block_on`
  - [ ] `embed_batch()` — batch request
- [ ] Task 3: Error handling and security (AC: #5, #6, #7)
  - [ ] Strip API key from error messages
  - [ ] Implement retry with backoff (3 attempts)
  - [ ] Convert errors to `SpecDbError::EmbeddingError`
- [ ] Task 4: Update lib.rs re-exports
  - [ ] `pub use remote::RemoteEmbedding;`
- [ ] Task 5: Integration test (AC: #9)
  - [ ] Use mock server or `#[ignore]` for real API test
  - [ ] Test error handling, retry behavior

## Dev Notes

### Library: async-openai

```rust
use async_openai::{Client, types::CreateEmbeddingRequestArgs};

let client = Client::new(); // Uses OPENAI_API_KEY env var
let request = CreateEmbeddingRequestArgs::default()
    .model("text-embedding-3-small")
    .input(["text to embed"])
    .build()?;
let response = client.embeddings().create(request).await?;
let embedding: Vec<f32> = response.data[0].embedding.clone();
```

**CRITICAL**: `async-openai` is async. `EmbeddingProvider` trait is sync. Use `tokio::runtime::Handle::current().block_on()` inside sync method (backend is called within `spawn_blocking`).

### Dimensions

- `text-embedding-3-small`: 1536 dimensions
- `text-embedding-ada-002`: 1536 dimensions

### Architecture Compliance

- [Source: architecture.md#Decision 3] — RemoteEmbedding wraps async-openai
- [Source: architecture.md#Async Boundary Pattern] — sync trait, async internally

### CRITICAL: What NOT To Do

- Do NOT store API key in config — read from `OPENAI_API_KEY` env var
- Do NOT log the API key in any error path
- Do NOT make the trait async — keep sync, use `block_on` internally

### References

- [Source: architecture.md#Decision 3] — EmbeddingProvider implementations
- [Source: prd.md#Embedding Considerations] — Local vs remote tradeoffs
- [Source: GitHub 64bit/async-openai] — Rust SDK examples

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
