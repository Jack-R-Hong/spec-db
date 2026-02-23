# Story 8.2: AI Causal Link Proposal via MCP Tool

Status: ready-for-dev

## Story

As an AI agent,
I want to propose new causal edges between existing specs via an `add_causal_link` MCP tool,
so that I can grow the causal knowledge graph as I discover relationships during analysis.

## Acceptance Criteria (BDD)

**Given** two existing specs `spec::auth::jwt-validation` and `spec::auth::token-format`
**When** I call `add_causal_link(source: "spec::auth::jwt-validation", target: "spec::auth::token-format", edge_type: "depends_on")`
**Then** a new `CausalEdge` is created with `trust: 0.5` (configurable default), `origin: AiInferred`, and the specified `edge_type`

**Given** a call to `add_causal_link` with a `source` ID that does not exist in the graph
**When** the tool processes the request
**Then** it returns an error: `{ error_type: "not_found", message: "Source spec not found", context: { id: "..." } }`

**Given** a call to `add_causal_link` with `source` equal to `target` (self-referencing)
**When** the tool processes the request
**Then** it returns an error: `{ error_type: "validation_error", message: "Self-referencing edges are not allowed" }`

**Given** a duplicate edge proposal (same source, target, and edge_type already exists)
**When** the tool processes the request
**Then** it returns an error: `{ error_type: "conflict", message: "Edge already exists" }`

**Given** the default trust score is `0.5`
**When** `.lattice/config.yaml` sets `ai.default_trust: 0.7`
**Then** new AI-proposed edges use `0.7` as their trust score

**Covers:** FR47, FR48

## Tasks / Subtasks

- [ ] Add `ai.default_trust` config field (AC: 5)
  - [ ] Extend config struct in `crates/core/` or config module to include `ai.default_trust: f64` with default `0.5`
  - [ ] Parse from `.lattice/config.yaml` during startup
  - [ ] Validate range: must be `0.0..=1.0`
- [ ] Implement `add_causal_link` MCP tool handler (AC: 1, 2, 3, 4)
  - [ ] Define tool input schema: `source: String`, `target: String`, `edge_type: String` (optional, default "depends_on")
  - [ ] Register tool in MCP server's tool list with description
  - [ ] Validate `source` and `target` are valid `SpecId` format
  - [ ] Validate `source != target` (reject self-referencing)
  - [ ] Parse `edge_type` string into `EdgeType` enum (reject unknown types)
  - [ ] Check both source and target specs exist in the graph
  - [ ] Check no duplicate edge exists (same source, target, edge_type)
  - [ ] Create `CausalEdge` with `trust: config.ai.default_trust`, `origin: AiInferred`, parsed `edge_type`
  - [ ] Insert edge into `CausaloidGraph` and persist to Fjall KV
  - [ ] Return success response with the created edge details
- [ ] Implement error responses matching MCP error shape (AC: 2, 3, 4)
  - [ ] `not_found`: source or target spec not in graph
  - [ ] `validation_error`: self-referencing edge
  - [ ] `conflict`: duplicate edge
  - [ ] All errors use shape: `{ error_type, message, context }`
- [ ] Add tests (AC: 1-5)
  - [ ] Unit test: successful edge creation with default trust
  - [ ] Unit test: successful edge creation with custom trust from config
  - [ ] Unit test: error on non-existent source
  - [ ] Unit test: error on non-existent target
  - [ ] Unit test: error on self-referencing edge
  - [ ] Unit test: error on duplicate edge
  - [ ] Unit test: edge_type parsing (all 3 variants + invalid)
  - [ ] Integration test: full MCP tool call round-trip

## Dev Notes

- This story depends on Story 8.1 (EdgeType, EdgeOrigin types must exist).
- The `add_causal_link` tool does NOT run CSM validation — that is Story 8.3. In this story, edges are inserted directly after basic validation.
- Follow existing MCP tool registration pattern in `crates/mcp/src/tools.rs`.
- Error shape must match existing MCP error format: `{ error_type: String, message: String, context: serde_json::Value }`.
- Config extension pattern: follow existing `.lattice/config.yaml` parsing in config module.

### Project Structure Notes

- Primary files: `crates/mcp/src/tools.rs` (new tool registration), `crates/causal/src/engine.rs` (edge insertion method)
- Config files: wherever config struct is defined (likely `crates/core/` or root crate)
- No new crates needed.

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 8.2]
- [Source: _bmad-output/planning-artifacts/architecture.md#MCP Tools]
- [Source: crates/mcp/src/tools.rs]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
