# Story 1.5: Backend Configuration Schema

Status: ready-for-dev

## Story

As a lattice operator,
I want to configure vector backends via `.lattice/config.yaml`,
so that backends are initialized automatically on startup.

## Acceptance Criteria

1. `search_backends` section accepted in `.lattice/config.yaml` with `default`, `backends` list
2. Each backend entry: `name`, `type`, `path` (for lancedb), optional `embedding` subsection
3. `embedding` subsection: `provider` (local/openai), `model`, `dimensions`
4. Config validated at startup: unknown types rejected, missing fields produce clear error
5. Missing `search_backends` section is valid (backward compatible, no vector backends)
6. Config deserialization uses `serde` with typed structs
7. Integration test verifies config parsing with valid and invalid YAML

## Tasks / Subtasks

- [ ] Task 1: Define config types in core (AC: #1, #2, #3, #6)
  - [ ] Add `SearchBackendsConfig` struct to `crates/core/src/config.rs`
  - [ ] Add `BackendEntry`, `EmbeddingConfig` structs
  - [ ] Add `search_backends: Option<SearchBackendsConfig>` to `SpecDbConfig`
- [ ] Task 2: Implement validation (AC: #4, #5)
  - [ ] Validate backend type is `fts` or `lancedb`
  - [ ] Validate required fields per type (`path` for lancedb)
  - [ ] Validate `search_backends: None` → backward compatible
- [ ] Task 3: Tests (AC: #7)
  - [ ] Test valid config with search_backends section
  - [ ] Test valid config without search_backends (backward compat)
  - [ ] Test invalid: unknown backend type
  - [ ] Test invalid: missing required `path` for lancedb

## Dev Notes

### Config Schema Design

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBackendsConfig {
    #[serde(default = "default_tantivy")]
    pub default: String,
    #[serde(default)]
    pub routing: Vec<RoutingRuleConfig>,  // Placeholder, used in Story 3.1
    #[serde(default)]
    pub backends: Vec<BackendEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub backend_type: String,  // "fts" | "lancedb"
    pub path: Option<String>,
    pub embedding: Option<EmbeddingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: String,       // "local" | "openai"
    pub model: String,
    pub dimensions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRuleConfig {
    pub agent: String,
    pub backend: String,
}
```

### Existing Config Pattern — MUST FOLLOW

From `crates/core/src/config.rs`:
- All config structs use `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Use `#[serde(default)]` for optional sections
- Use default functions for default values
- `load_config()` validates after deserialization

Add to existing `SpecDbConfig`:
```rust
#[serde(default)]
pub search_backends: Option<SearchBackendsConfig>,
```

### Architecture Compliance

- [Source: architecture.md#Configuration Pattern] — YAML schema structure
- [Source: architecture.md#Decision 5] — Config types in core crate
- [Source: prd.md#Configuration Schema] — Expected YAML structure

### CRITICAL: What NOT To Do

- Do NOT implement backend initialization from config (that will be in the startup/serve path)
- Do NOT add routing rule validation against backend names yet (Story 3.1)
- Do NOT modify any code outside `crates/core/`

### References

- [Source: crates/core/src/config.rs] — Existing SpecDbConfig, load_config()
- [Source: architecture.md#Configuration Pattern] — Config schema
- [Source: prd.md#Configuration Schema] — YAML example

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
