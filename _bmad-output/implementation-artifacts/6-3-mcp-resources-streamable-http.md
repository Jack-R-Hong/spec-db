# Story 6.3: MCP Resources & Streamable-HTTP Transport

Status: ready-for-dev

## Story

As an AI agent,
I want to read spec content and graph summaries via MCP resources, and optionally connect over HTTP,
so that I can access spec intelligence through multiple access patterns and transports.

## Acceptance Criteria (BDD)

**Given** an agent requesting `spec://{id}`
**When** the resource is read
**Then** the full spec content is returned (FR29)

**Given** an agent requesting `graph://overview`
**When** the resource is read
**Then** it returns causal graph summary statistics: total specs, total edges, and a list of disconnected clusters (specs with no causal edges) (FR30, FR32)

**Given** an agent requesting `graph://node/{id}`
**When** the resource is read
**Then** it returns the spec node with all inbound and outbound edges (FR31)

**Given** streamable-http transport enabled in `.spec-db/config.yaml` with `http.auth_token` set
**When** an agent connects over HTTP
**Then** requests without a valid bearer token are rejected with 401 (FR27, NFR24)
**And** requests with a valid token are processed identically to stdio

**Given** streamable-http transport is not configured
**When** the server starts
**Then** only stdio transport is available — no network surface (NFR23)

## Tasks / Subtasks

- [ ] Implement MCP resource advertisement in `crates/mcp/src/resources.rs`
  - [ ] Add `list_resources()` override in `impl ServerHandler for SpecDbMcpServer`
  - [ ] Publish resources: `spec://{id}`, `graph://overview`, `graph://node/{id}`
  - [ ] Include stable names/descriptions so clients can discover semantics
- [ ] Implement resource resolution in `crates/mcp/src/resources.rs`
  - [ ] Add `read_resource(ReadResourceRequestParams { uri, .. }, ...)`
  - [ ] Route `spec://{id}` to spec store lookup and return full content JSON text
  - [ ] Route `graph://overview` to causal engine summary (`total_specs`, `total_edges`, `disconnected_clusters`)
  - [ ] Route `graph://node/{id}` to node detail with inbound/outbound edges
  - [ ] Return `resource_not_found` McpError for unknown URIs
- [ ] Define response payload shapes for resources
  - [ ] `spec://{id}` body: `{ "spec": { id, title, version, tags, depends_on, owner, created, body } }`
  - [ ] `graph://overview` body: `{ "total_specs": number, "total_edges": number, "disconnected_clusters": [string] }`
  - [ ] `graph://node/{id}` body: `{ "node": string, "inbound": [{from,to,type}], "outbound": [{from,to,type}] }`
- [ ] Add optional streamable-http server wiring in `src/main.rs` and `crates/mcp/src/server.rs`
  - [ ] Extend config model with `transport.http.enabled`, `transport.http.bind`, `transport.http.auth_token`
  - [ ] If HTTP disabled/unset, serve stdio only
  - [ ] If enabled, create `StreamableHttpService<SpecDbMcpServer, LocalSessionManager>` with `StreamableHttpServerConfig`
  - [ ] Mount service at `/mcp` using `axum` router and run alongside stdio (Tokio task)
- [ ] Add bearer token authentication middleware for HTTP mode
  - [ ] Parse `Authorization: Bearer <token>` header
  - [ ] Compare against configured `http.auth_token`
  - [ ] Return HTTP 401 for missing/invalid token before MCP handler execution
  - [ ] Ensure valid-token calls invoke the exact same server handler as stdio path
- [ ] Add transport/resource integration tests in `crates/mcp/tests/integration.rs`
  - [ ] Resource discovery + read tests for all three URI families
  - [ ] HTTP auth tests: 401 invalid token, success valid token
  - [ ] Parity test confirming stdio and HTTP responses are schema-equivalent

## Dev Notes

- rmcp 0.16.0 resources are exposed through `ServerHandler` methods (`list_resources`, `read_resource`), not a separate resource macro layer.
- Streamable HTTP uses `StreamableHttpService` + `LocalSessionManager` + `StreamableHttpServerConfig`; auth is typically enforced in outer HTTP middleware.
- Security posture must remain local-first: no HTTP listener unless config explicitly enables it and provides token.
- Resource URI naming must follow architecture N4 exactly (scheme + path forms).
- Graph summaries rely on causal store/engine from Epic 1 and should avoid expensive recomputation in handler path.

### Project Structure Notes

- Resource logic belongs in `crates/mcp/src/resources.rs`; transport wiring in `crates/mcp/src/server.rs` and root `src/main.rs`.
- Keep HTTP optional and isolated behind config feature flags/branches so default install has no network surface.
- Preserve shared response/error shape conventions from F1/F2 for consistent client behavior.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 6.3]
- [Source: _bmad-output/planning-artifacts/architecture.md#Authentication & Security]
- [Source: _bmad-output/planning-artifacts/architecture.md#MCP Transport]
- [Source: https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/simple_auth_streamhttp.rs]
- [Source: https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/common/counter.rs]
- [Source: https://github.com/modelcontextprotocol/rust-sdk/blob/main/crates/rmcp/src/transport/streamable_http_server/tower.rs]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story captures resource URI handling, streamable-http transport gating, and bearer token requirements with concrete implementation tasks.

### Change Log

- Created initial ready-for-dev story document.

### File List

- _bmad-output/implementation-artifacts/6-3-mcp-resources-streamable-http.md
