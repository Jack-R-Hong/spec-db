# Story 6.2: MCP Server with Tools over Stdio

Status: ready-for-dev

## Story

As an AI agent,
I want to discover and call spec-db tools via MCP over stdio transport,
so that I can search, reason, and manage specs as a native capability.

## Acceptance Criteria (BDD)

**Given** the MCP server started via `spec-db serve`
**When** an agent connects over stdio
**Then** all spec-db tools are discoverable via MCP protocol (FR28)
**And** the transport uses stdio with zero network configuration (FR26)

**Given** an agent calling `search_specs(query, filters?)`
**When** the tool executes
**Then** it delegates to the search engine via `spawn_blocking` and returns JSON results: `[{id, title, score, snippet}]`

**Given** an agent calling `get_spec(id)`
**When** the tool executes
**Then** it returns the full spec content as JSON

**Given** an agent calling `trace_impact(id, depth?)` or `find_dependencies(id)`
**When** the tool executes
**Then** it delegates to the causal engine and returns JSON: `{node, edges: [{from, to, type}]}`

**Given** an agent calling `query(natural_language)`
**When** the tool executes
**Then** it delegates to the query router and returns composed JSON results

**Given** an agent calling `add_spec(markdown)` or `sync(mode?)`
**When** the tool executes
**Then** it delegates to the ingestion/sync pipeline and returns JSON: `{status, message, details}`

**Given** any MCP tool call
**When** it executes
**Then** the response completes in < 100ms end-to-end (NFR8)
**And** errors return consistent JSON: `{error_type, message, context}` per architecture pattern F2

## Tasks / Subtasks

- [ ] Scaffold MCP crate (Build Order #7)
  - [ ] Create `crates/mcp/Cargo.toml` with package name `spec-db-mcp`
  - [ ] Pin `rmcp = "=0.16.0"` with server/macros + required transport features
  - [ ] Add modules in `crates/mcp/src/{lib.rs,server.rs,tools.rs,resources.rs}`
- [ ] Implement server state and handler wiring in `crates/mcp/src/server.rs`
  - [ ] Define `SpecDbMcpServer` holding trait-object dependencies (`SearchEngine`, `CausalGraph`, `SpecStore`, router/sync services)
  - [ ] Add `ToolRouter<SpecDbMcpServer>` field; initialize via `Self::tool_router()`
  - [ ] Implement `ServerHandler::get_info()` with `.enable_tools()` capability
  - [ ] Use `#[tool_handler] impl ServerHandler for SpecDbMcpServer` for discoverability
- [ ] Implement tool definitions in `crates/mcp/src/tools.rs` using rmcp macros
  - [ ] Add `#[tool_router] impl SpecDbMcpServer` block
  - [ ] Add `#[tool] async fn search_specs(...) -> Result<CallToolResult, McpError>`
  - [ ] Add `#[tool] async fn get_spec(...)`, `trace_impact(...)`, `find_dependencies(...)`, `query(...)`, `add_spec(...)`, `sync(...)`
  - [ ] Wrap each sync subsystem call with `tokio::task::spawn_blocking` (pattern P3)
- [ ] Define request/response schemas (serde + schemars)
  - [ ] `search_specs` input: `{ query: string, filters?: { tags?: string[], owner?: string, limit?: number } }`
  - [ ] `search_specs` output F1: `{ "results": [{"id": string, "title": string, "score": number, "snippet": string}] }`
  - [ ] `get_spec` input: `{ id: string }`; output: `{ "spec": { id, title, version, tags, depends_on, owner, created, body } }`
  - [ ] `trace_impact` input: `{ id: string, depth?: number }`; output F1 Graph: `{ "node": string, "edges": [{"from": string, "to": string, "type": string}] }`
  - [ ] `find_dependencies` input: `{ id: string }`; output F1 Graph shape
  - [ ] `query` input: `{ natural_language: string }`; output: `{ "intent": string, "search"?: [...], "graph"?: {...} }`
  - [ ] `add_spec` input: `{ markdown: string }`; output F1 Admin: `{ "status": string, "message": string, "details": object }`
  - [ ] `sync` input: `{ mode?: "incremental" | "full" }`; output F1 Admin shape
- [ ] Standardize MCP error mapping (F2)
  - [ ] Add helper `fn mcp_error(error_type: &str, message: impl Into<String>, context: Option<Value>) -> McpError`
  - [ ] Map `SearchError`, `GraphError`, `SyncError`, `IngestError`, `ConfigError` consistently
  - [ ] Ensure all `Err` payloads serialize to `{error_type, message, context}`
- [ ] Implement stdio serve path in root CLI
  - [ ] In `src/main.rs`, `serve` command builds server and calls `.serve(rmcp::transport::stdio())`
  - [ ] Await `service.waiting().await` and handle cancellation signals cleanly
- [ ] Add MCP integration tests in `crates/mcp/tests/integration.rs`
  - [ ] Tool discovery test verifies all 7 tool names are listed
  - [ ] JSON shape tests for each tool response and error payloads
  - [ ] Latency smoke test (<100ms) under warm local fixture set

## Dev Notes

- rmcp 0.16.0 server pattern: `#[tool_router]` on impl block + `#[tool]` methods + `#[tool_handler] impl ServerHandler`.
- Keep async boundary in handlers only; downstream crates remain synchronous and invoked via `spawn_blocking`.
- Tool names must remain snake_case (`search_specs`, `trace_impact`, etc.) per pattern N4.
- Response contracts must follow F1 categories (Search, Graph, Admin) and F2 errors.
- MCP crate is integration boundary over prior epics: search (Epic 2), causal (Epic 1), ingest/sync (Epic 4), router (Epic 5).
- Use `tracing` spans for public tool handlers (`spec_db.mcp.tool_call`).

### Project Structure Notes

- New crate path: `crates/mcp/` with canonical files `lib.rs`, `server.rs`, `tools.rs`, `resources.rs`.
- Root binary (`src/main.rs`) wires dependencies and starts stdio server; transport logic should not leak into lower crates.
- Keep module depth <=2 and avoid cross-crate concrete coupling; consume trait interfaces from `spec-db-core`.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 6.2]
- [Source: _bmad-output/planning-artifacts/architecture.md#Async Boundary]
- [Source: _bmad-output/planning-artifacts/architecture.md#Format Patterns]
- [Source: https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v0.16.0]
- [Source: https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/calculator_stdio.rs]
- [Source: https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/common/counter.rs]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story defines all required MCP tools, JSON contracts, rmcp 0.16.0 macro usage, and async/sync execution boundaries.

### Change Log

- Created initial ready-for-dev story document.

### File List

- _bmad-output/implementation-artifacts/6-2-mcp-server-tools-stdio.md
