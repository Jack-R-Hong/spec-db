# Story 5.4: Embedding Provider Configuration

Status: ready-for-dev

## Story

As a lattice operator,
I want to configure which embedding provider each backend uses,
so that I can choose local or remote embedding per backend.

## Acceptance Criteria

1. `provider: local` + `model: all-MiniLM-L6-v2` → initializes `LocalEmbedding`
2. `provider: openai` + `model: text-embedding-3-small` → initializes `RemoteEmbedding`
3. `dimensions` validated against model's output at startup
4. Dimension mismatch with existing LanceDB index → clear error
5. OpenAI API key read from `OPENAI_API_KEY` env var
6. Missing `embedding` section for vector backend → startup error
7. Integration test for both provider types

## Tasks / Subtasks

- [ ] Task 1: Implement provider factory (AC: #1, #2)
  - [ ] Create `create_embedding_provider(config: &EmbeddingConfig) -> Result<Box<dyn EmbeddingProvider>>`
  - [ ] Match on `provider` field: "local" → LocalEmbedding, "openai" → RemoteEmbedding
- [ ] Task 2: Dimension validation (AC: #3, #4)
  - [ ] After creating provider, verify `provider.dimensions() == config.dimensions`
  - [ ] Check against existing LanceDB table schema if table exists
- [ ] Task 3: API key handling (AC: #5)
  - [ ] Read `OPENAI_API_KEY` from env for OpenAI provider
  - [ ] Error if env var missing when openai provider requested
- [ ] Task 4: Startup validation (AC: #6)
  - [ ] During backend initialization, require embedding config for vector backends
  - [ ] Clear error: "Backend 'xxx' requires embedding configuration"
- [ ] Task 5: Integration test (AC: #7)
  - [ ] Test local provider init from config
  - [ ] Test openai provider init (with mock or #[ignore])

## Dev Notes

### Provider Factory Pattern

```rust
pub fn create_embedding_provider(
    config: &EmbeddingConfig,
) -> Result<Box<dyn EmbeddingProvider>, SpecDbError> {
    match config.provider.as_str() {
        "local" => {
            let provider = LocalEmbedding::new(&config.model)?;
            if provider.dimensions() != config.dimensions {
                return Err(SpecDbError::ConfigError(format!(
                    "dimension mismatch: model {} produces {}, config specifies {}",
                    config.model, provider.dimensions(), config.dimensions
                )));
            }
            Ok(Box::new(provider))
        }
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| SpecDbError::ConfigError(
                    "OPENAI_API_KEY env var required for openai provider".into()
                ))?;
            Ok(Box::new(RemoteEmbedding::new(&api_key, &config.model, config.dimensions)?))
        }
        other => Err(SpecDbError::ConfigError(format!("unknown embedding provider: {other}"))),
    }
}
```

### Architecture Compliance

- [Source: architecture.md#Decision 3] — Provider abstraction
- [Source: architecture.md#Configuration Pattern] — Config-driven initialization
- [Source: prd.md#Configuration] — FR23, FR24

### References

- [Source: crates/embedding/src/local.rs] — LocalEmbedding (Story 1.2)
- [Source: crates/embedding/src/remote.rs] — RemoteEmbedding (Story 5.3)
- [Source: crates/core/src/config.rs] — EmbeddingConfig struct (Story 1.5)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
