# Story 4.3: MCP Backend Tools

Status: ready-for-dev

## Story

As an AI agent developer,
I want MCP tools to manage and query backends,
so that AI agents can discover and interact with backends through the MCP protocol.

## Acceptance Criteria

1. `configure_backend(name, type, config)` — create/update backend, return confirmation
2. `list_backends()` — return backends with name, type, health status
3. `search_specs` extended with: `backend` (name), `mode` (fts/vector/hybrid), `agent_id`
4. `backend` param overrides routing rules
5. `mode` omitted → defaults to `fts` (backward compatible)
6. `agent_id` omitted → default backend (backward compatible)
7. All tools conform to MCP spec (JSON schema)
8. Integration test via MCP tool call simulation

## Tasks / Subtasks

- [ ] Task 1: Add `configure_backend` tool (AC: #1, #7)
  - [ ] New tool in `crates/mcp/`
  - [ ] Accept name, backend_type, config JSON
  - [ ] Delegate to BackendRegistry
- [ ] Task 2: Add `list_backends` tool (AC: #2, #7)
  - [ ] Return JSON array of backend info
- [ ] Task 3: Extend `search_specs` tool (AC: #3, #4, #5, #6)
  - [ ] Add optional params: `backend`, `mode`, `agent_id`
  - [ ] Wire to `search_with_mode()` via router
  - [ ] Default mode = "fts", agent_id = None, backend = None
- [ ] Task 4: Integration test (AC: #8)
  - [ ] Simulate MCP tool calls
  - [ ] Verify backward compatibility with existing calls

## Dev Notes

### Existing MCP Pattern

From `crates/mcp/` — rmcp tool macros:
```rust
#[tool(name = "search_specs")]
async fn search_specs(
    &self,
    query: String,
    #[tool(param)] limit: Option<usize>,
    #[tool(param)] tags: Option<Vec<String>>,
) -> Result<CallToolResult, McpError> { ... }
```

Add new optional params without breaking existing clients (MCP allows optional additions).

### Architecture Compliance

- [Source: architecture.md#API Patterns] — MCP tool naming (snake_case)
- [Source: architecture.md#Boundary Rules] — Interface → Router with agent_context
- [Source: prd.md#MCP Tools] — Expected tool signatures

### CRITICAL: What NOT To Do

- Do NOT break existing `search_specs` calls — all new params optional
- Do NOT change return types of existing tools

### References

- [Source: crates/mcp/] — Existing MCP tool definitions
- [Source: architecture.md#API Patterns] — Tool naming convention
- [Source: prd.md#MCP Tools] — Expected MCP tools

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
