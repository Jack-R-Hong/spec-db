# Story 3.3: Backend Resolution and Dispatch

Status: ready-for-dev

## Story

As an AI agent,
I want my search queries automatically routed to my assigned backend,
so that I only see the knowledge store I'm authorized to access.

## Acceptance Criteria

1. `BackendRegistry::resolve(agent_context: Option<&str>) -> Result<&dyn VectorSearchBackend, SpecDbError>`
2. Agent matching routing rule → corresponding backend returned
3. Rules evaluated in order; first match wins
4. No match + agent provided → default backend
5. No agent context → default backend
6. Explicit `backend` param overrides routing (FR17)
7. Resolved name not in registry → `BackendNotFound`
8. Integration test: match → correct backend, no match → default, override works

## Tasks / Subtasks

- [ ] Task 1: Add `resolve()` to `BackendRegistry` (AC: #1, #2, #3, #4, #5)
  - [ ] Accept `agent_context: Option<&str>`
  - [ ] Iterate routing rules in order, use glob match from Story 3.1
  - [ ] Return first matching backend, or default
- [ ] Task 2: Add `resolve_explicit()` for override (AC: #6, #7)
  - [ ] Accept `backend_name: Option<&str>, agent_context: Option<&str>`
  - [ ] If `backend_name` provided, use it directly (override routing)
  - [ ] Otherwise fall through to `resolve(agent_context)`
- [ ] Task 3: Wire into router (AC: #1)
  - [ ] `search_with_mode()` calls `resolve_explicit(backend, agent_context)` to pick backend
- [ ] Task 4: Store routing rules in registry (AC: #2, #3)
  - [ ] Add `routing_rules: Vec<RoutingRule>` to `BackendRegistry`
  - [ ] Populate from config during startup
- [ ] Task 5: Integration test (AC: #8)
  - [ ] Setup 2 backends + routing rules
  - [ ] Test agent match, no match, explicit override

## Dev Notes

### Resolution Logic

```rust
impl BackendRegistry {
    pub fn resolve_explicit(
        &self,
        backend_name: Option<&str>,
        agent_context: Option<&str>,
    ) -> Result<&dyn VectorSearchBackend, SpecDbError> {
        if let Some(name) = backend_name {
            return self.get(name);
        }
        self.resolve(agent_context)
    }

    pub fn resolve(&self, agent_context: Option<&str>) -> Result<&dyn VectorSearchBackend, SpecDbError> {
        if let Some(agent) = agent_context {
            for rule in &self.routing_rules {
                if matches_agent(&rule.agent_pattern, agent) {
                    return self.get(&rule.backend);
                }
            }
        }
        self.get(&self.default_backend)
    }
}
```

### Architecture Compliance

- [Source: architecture.md#Decision 2] — BackendRegistry.resolve()
- [Source: architecture.md#Boundary Rules] — Router resolves via registry
- [Source: prd.md#Agent Routing] — FR9, FR10, FR17

### References

- [Source: crates/search-vector/src/registry.rs] — BackendRegistry (Story 1.4)
- [Source: architecture.md#Decision 2] — Routing architecture

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
