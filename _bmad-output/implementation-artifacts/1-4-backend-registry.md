# Story 1.4: Backend Registry

Status: ready-for-dev

## Story

As a lattice developer,
I want a `BackendRegistry` that manages multiple named vector backends,
so that the system can hold and resolve multiple backends simultaneously.

## Acceptance Criteria

1. `BackendRegistry` in `crates/search-vector/src/registry.rs` with `HashMap<String, Box<dyn VectorSearchBackend>>`
2. `add_backend(name, backend)` registers a named backend
3. `remove_backend(name)` removes; returns `BackendNotFound` if absent
4. `get(name)` returns `&dyn VectorSearchBackend` or `BackendNotFound`
5. `list()` returns all registered backend names
6. `default_backend` field for when no name is specified
7. Unit tests verify add/remove/get/list operations

## Tasks / Subtasks

- [ ] Task 1: Implement `BackendRegistry` (AC: #1, #2, #3, #4, #5, #6)
  - [ ] Create `crates/search-vector/src/registry.rs`
  - [ ] `BackendRegistry` struct with `backends: HashMap`, `default_backend: String`
  - [ ] Implement `add_backend`, `remove_backend`, `get`, `list`, `get_default`
- [ ] Task 2: Update lib.rs (AC: #1)
  - [ ] Re-export `BackendRegistry` from `crates/search-vector/src/lib.rs`
- [ ] Task 3: Unit tests (AC: #7)
  - [ ] Create mock `VectorSearchBackend` impl for testing
  - [ ] Test add/get/remove/list/default operations
  - [ ] Test `BackendNotFound` error case

## Dev Notes

### Implementation Pattern

```rust
use std::collections::HashMap;
use spec_db_core::{VectorSearchBackend, SpecDbError};

pub struct BackendRegistry {
    backends: HashMap<String, Box<dyn VectorSearchBackend>>,
    default_backend: String,
}
```

- `get()` should return `Result<&dyn VectorSearchBackend, SpecDbError>` using `BackendNotFound`
- `get_default()` calls `get(&self.default_backend)`
- Do NOT add routing logic here (that's Story 3.3)

### Architecture Compliance

- [Source: architecture.md#Decision 2] — `BackendRegistry` with `HashMap<String, Box<dyn VectorSearchBackend>>`
- [Source: architecture.md#Decision 2] — `default_backend` field

### CRITICAL: What NOT To Do

- Do NOT add `resolve(agent_context)` method yet (that's Story 3.3)
- Do NOT add routing rules (that's Story 3.1)
- Do NOT add config deserialization (that's Story 1.5)

### References

- [Source: architecture.md#Decision 2] — Backend Registry architecture
- [Source: crates/core/src/traits.rs] — VectorSearchBackend trait (Story 1.1)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
