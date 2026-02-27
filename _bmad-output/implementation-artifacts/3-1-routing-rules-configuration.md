# Story 3.1: Routing Rules Configuration

Status: ready-for-dev

## Story

As a lattice operator,
I want to define agent-to-backend routing rules in configuration,
so that different agents are automatically routed to different backends.

## Acceptance Criteria

1. `routing` list in `search_backends` config with `agent` (glob) and `backend` (name) fields
2. Parsed into `Vec<RoutingRule>` with `agent_pattern: String` and `backend: String`
3. `RoutingRule` defined in `crates/core/`
4. Wildcard patterns supported (`doc-*` matches `doc-agent`)
5. `*` matches any agent (catch-all)
6. Config validation rejects rules referencing non-existent backends
7. No routing rules → all agents use `default` backend
8. Unit tests for glob matching and config validation

## Tasks / Subtasks

- [ ] Task 1: Define `RoutingRule` type in core (AC: #2, #3)
  - [ ] Add to `crates/core/src/types.rs` or `config.rs`
- [ ] Task 2: Implement glob matching (AC: #4, #5)
  - [ ] Simple glob: `*` matches everything, `prefix-*` matches prefix
  - [ ] Use `glob-match` crate or manual implementation (simple patterns only)
- [ ] Task 3: Config validation (AC: #6, #7)
  - [ ] During config loading, verify each routing rule's `backend` exists in backends list
  - [ ] Empty routing list is valid
- [ ] Task 4: Unit tests (AC: #8)
  - [ ] Test `doc-*` matches `doc-agent`, `doc-writer`
  - [ ] Test `*` matches anything
  - [ ] Test exact match: `review-agent` matches `review-agent`
  - [ ] Test no match returns None

## Dev Notes

### Config Already Defined

`RoutingRuleConfig` was already defined as a placeholder in Story 1.5. This story implements the runtime `RoutingRule` type and glob matching logic.

### Glob Matching — Keep Simple

For MVP, support only: exact match and trailing `*` wildcard.
```rust
pub fn matches_agent(pattern: &str, agent: &str) -> bool {
    if pattern == "*" { return true; }
    if pattern.ends_with('*') {
        agent.starts_with(&pattern[..pattern.len()-1])
    } else {
        pattern == agent
    }
}
```

No need for full glob crate — these patterns are simple.

### Architecture Compliance

- [Source: architecture.md#Decision 2] — RoutingRule struct
- [Source: architecture.md#Configuration Pattern] — routing section in config
- [Source: prd.md#Agent Routing] — FR8, FR11

### References

- [Source: crates/core/src/config.rs] — RoutingRuleConfig (Story 1.5)
- [Source: architecture.md#Decision 2] — RoutingRule design
- [Source: prd.md#Configuration Schema] — routing rules YAML

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
