# Story 3.2: Agent Context Propagation

Status: ready-for-dev

## Story

As a lattice developer,
I want agent identity propagated from MCP/REST interfaces to the router,
so that the router can resolve the correct backend for each agent.

## Acceptance Criteria

1. `agent_context: Option<String>` parameter added to router search methods
2. MCP tools accept optional `agent_id` parameter in `search_specs` and `query`
3. REST API accepts `X-Agent-Id` header or `agent_id` query parameter
4. Agent context passed through: interface → router → backend resolution
5. If `agent_id` not provided, `agent_context` is `None`
6. Existing tool calls without `agent_id` work unchanged
7. Integration test: agent_id flows from MCP tool call to router

## Tasks / Subtasks

- [ ] Task 1: Add `agent_context` to router methods (AC: #1, #4)
  - [ ] Extend `search_with_mode()` signature in `QueryRouter`
  - [ ] Thread through to backend resolution calls
- [ ] Task 2: Extend MCP tools (AC: #2, #5, #6)
  - [ ] Add `agent_id: Option<String>` param to `search_specs` tool
  - [ ] Add `agent_id: Option<String>` param to `query` tool
  - [ ] Pass to router; `None` if omitted (backward compatible)
- [ ] Task 3: Extend REST API (AC: #3, #5)
  - [ ] Read `X-Agent-Id` header from request
  - [ ] Fallback: `agent_id` query parameter
  - [ ] Pass to router; `None` if absent
- [ ] Task 4: Integration test (AC: #7)
  - [ ] Verify agent_id propagates through the full chain

## Dev Notes

### MCP Tool Extension Pattern

From `crates/mcp/` — existing tools use rmcp attributes:
```rust
#[tool(name = "search_specs")]
async fn search_specs(query: String, limit: Option<usize>, tags: Option<Vec<String>>) -> ...
```

Add optional parameter — MCP spec allows adding optional params without breaking clients.

### REST Pattern

From `crates/web/` — Axum handlers use extractors:
```rust
// Extract from header
fn extract_agent_id(headers: &HeaderMap) -> Option<String> {
    headers.get("X-Agent-Id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
}
```

### CRITICAL: What NOT To Do

- Do NOT implement the actual routing resolution logic (that's Story 3.3)
- Do NOT break existing MCP/REST interfaces — all new params are optional
- Do NOT add agent_id to CLI commands (CLI doesn't have agent context)

### References

- [Source: crates/mcp/] — MCP tool definitions
- [Source: crates/web/] — REST endpoint handlers
- [Source: architecture.md#Boundary Rules] — "Interface → Router: pass agent_context"

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
