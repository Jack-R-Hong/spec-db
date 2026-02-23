# Story 8.2: AI Causal Link Proposal via MCP Tool

Status: review

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

- [x] Add `ai.default_trust` config field (AC: 5)
  - [x] Extend config struct in `crates/core/` or config module to include `ai.default_trust: f64` with default `0.5`
  - [x] Parse from `.lattice/config.yaml` during startup
  - [x] Validate range: must be `0.0..=1.0`
- [x] Implement `add_causal_link` MCP tool handler (AC: 1, 2, 3, 4)
  - [x] Define tool input schema: `source: String`, `target: String`, `edge_type: String` (optional, default "depends_on")
  - [x] Register tool in MCP server's tool list with description
  - [x] Validate `source` and `target` are valid `SpecId` format
  - [x] Validate `source != target` (reject self-referencing)
  - [x] Parse `edge_type` string into `EdgeType` enum (reject unknown types)
  - [x] Check both source and target specs exist in the graph
  - [x] Check no duplicate edge exists (same source, target, edge_type)
  - [x] Create `CausalEdge` with `trust: config.ai.default_trust`, `origin: AiInferred`, parsed `edge_type`
  - [x] Insert edge into `CausaloidGraph` and persist to Fjall KV
  - [x] Return success response with the created edge details
- [x] Implement error responses matching MCP error shape (AC: 2, 3, 4)
  - [x] `not_found`: source or target spec not in graph
  - [x] `validation_error`: self-referencing edge
  - [x] `conflict`: duplicate edge
  - [x] All errors use shape: `{ error_type, message, context }`
- [x] Add tests (AC: 1-5)
  - [x] Unit test: successful edge creation with default trust
  - [x] Unit test: successful edge creation with custom trust from config
  - [x] Unit test: error on non-existent source
  - [x] Unit test: error on non-existent target
  - [x] Unit test: error on self-referencing edge
  - [x] Unit test: error on duplicate edge
  - [x] Unit test: edge_type parsing (all 3 variants + invalid)
  - [x] Integration test: full MCP tool call round-trip

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

openai/gpt-5.3-codex

### Debug Log References

### Completion Notes List

- Added `AiConfig` with `ai.default_trust` defaulting to `0.5` and added load-time validation for `0.0..=1.0`.
- Registered new MCP tool `add_causal_link` with schema (`source`, `target`, optional `edge_type`) and server dispatch wiring.
- Implemented causal-link creation flow with SpecId validation, self-edge rejection, edge type parsing, source/target existence checks, duplicate checks, and insertion as `origin: Ai` using configured trust.
- Added structured MCP error payload support (`{ error_type, message, context }`) for tool-level `not_found`, `validation_error`, and `conflict` responses.
- Added/updated tests for config parsing/validation and MCP causal-link behavior; `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` pass.

### Change Log

- 2026-02-23: Implemented Story 8.2 with config trust extension, MCP `add_causal_link` tool, error payload handling, and test coverage.

### File List

- `crates/core/src/config.rs`
- `crates/core/src/lib.rs`
- `crates/mcp/src/tools.rs`
- `crates/mcp/src/server.rs`
- `crates/mcp/tests/integration.rs`
- `src/main.rs`
- `tests/acceptance_story_6_2.rs`
