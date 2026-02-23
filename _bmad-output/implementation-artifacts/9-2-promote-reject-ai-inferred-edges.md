# Story 9.2: Promote & Reject AI-Inferred Edges

Status: review

## Story

As a spec author,
I want to promote AI-inferred edges to human-curated status or reject them entirely, via CLI command or MCP tool,
so that I maintain authority over which causal relationships are trusted in the graph.

## Acceptance Criteria (BDD)

**Given** an AI-inferred edge from `spec::auth::jwt` to `spec::auth::tokens` exists in the graph
**When** I call `promote_edge(source: "spec::auth::jwt", target: "spec::auth::tokens", edge_type: "depends_on")` via MCP tool
**Then** the edge's `origin` changes to `Human`, `trust` becomes `1.0`, and it is removed from `.lattice/edges.yaml`

**Given** an AI-inferred edge exists in the graph
**When** I call `reject_edge(source: "spec::auth::jwt", target: "spec::auth::tokens", edge_type: "depends_on")` via MCP tool
**Then** the edge is removed from the `CausaloidGraph`, deleted from Fjall KV, and removed from `.lattice/edges.yaml`

**Given** I call `promote_edge` or `reject_edge` with an edge that does not exist
**When** the tool processes the request
**Then** it returns `{ error_type: "not_found", message: "Edge not found", context: { source: "...", target: "...", edge_type: "..." } }`

**Given** I call `promote_edge` on an edge that is already human-curated (origin: `Human`)
**When** the tool processes the request
**Then** it returns `{ error_type: "validation_error", message: "Edge is already human-curated" }`

**Given** the CLI exposes `lattice edge promote <source> <target> [--type depends_on]` and `lattice edge reject <source> <target> [--type depends_on]`
**When** I run either command
**Then** it performs the same operation as the corresponding MCP tool and prints a confirmation message

**Covers:** FR53, FR54

## Tasks / Subtasks

- [x] Implement `promote_edge` MCP tool (AC: 1, 3, 4)
  - [x] `EdgeActionInput` with source, target, edge_type (optional, default "depends_on")
  - [x] Look up edge via `graph.edges_from(&source)` + find by target
  - [x] Validate edge exists → `not_found` error if missing
  - [x] Validate edge is `Ai` → `validation_error` if already `Human`
  - [x] Update via `graph.update_edge_origin()` to `Human` / trust 1.0
  - [x] Persist updated edge to Fjall KV
  - [x] Re-export AI edges to `.lattice/edges.yaml`
  - [x] Return success response with updated edge details
- [x] Implement `reject_edge` MCP tool (AC: 2, 3)
  - [x] Same input schema as promote_edge
  - [x] Look up edge → `not_found` if missing
  - [x] Remove edge via `graph.remove_edge()`
  - [x] Re-export AI edges to `.lattice/edges.yaml`
  - [x] Return success response
- [x] Implement CLI commands (AC: 5)
  - [x] `lattice edge promote <source> <target> [--type <edge_type>]`
  - [x] `lattice edge reject <source> <target> [--type <edge_type>]`
  - [x] Both reuse `ToolHandler.promote_edge()` / `reject_edge()`
  - [x] Print confirmation message on success, error on failure
- [x] Add tests (AC: 1-5)
  - [x] `promote_edge_changes_origin_to_human`
  - [x] `promote_already_human_edge_returns_validation_error`
  - [x] `promote_nonexistent_edge_returns_not_found`
  - [x] `reject_edge_removes_from_graph`
  - [x] `reject_nonexistent_edge_returns_not_found`
  - [x] `promote_removes_from_edges_yaml`
  - [x] `reject_removes_from_edges_yaml`
  - [x] `promote_and_reject_tools_are_registered_on_server`

## Dev Notes

- This story depends on Story 9.1 (edges.yaml export must work for re-export after promote/reject).
- Promote modifies in-place; reject deletes entirely. Both trigger a full re-export of AI edges.
- CLI subcommand pattern: follow existing `lattice` CLI structure using clap derive. Add `edge` subcommand group with `promote` and `reject` sub-subcommands.
- The `--type` flag defaults to `depends_on` since that's the most common edge type.

### Project Structure Notes

- Primary files: `crates/mcp/src/tools.rs` (new tools), `crates/causal/src/engine.rs` (edge mutation methods)
- CLI files: `src/main.rs` or CLI module (add `edge promote`/`edge reject` subcommands)
- Reuses export logic from Story 9.1

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 9.2]
- [Source: _bmad-output/planning-artifacts/prd.md#Edge Lifecycle & Human Review]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- `update_edge_origin()` added to CausalEngine for in-place edge modification
- Both tools re-export edges.yaml after mutation for consistency
- CLI uses `Edge { Promote, Reject }` subcommand pattern with clap derive

### Change Log
- `crates/causal/src/engine.rs`: Added `update_edge_origin()` method
- `crates/mcp/src/tools.rs`: Added `EdgeActionInput`, `promote_edge()`, `reject_edge()` handlers
- `crates/mcp/src/server.rs`: Registered promote_edge/reject_edge tool definitions and call routing
- `crates/mcp/src/lib.rs`: Re-exported `EdgeActionInput` and `ToolHandler`
- `src/main.rs`: Added `Edge { Promote, Reject }` CLI subcommands with `run_edge_action()`
- `crates/mcp/tests/integration.rs`: Added 8 new tests

### File List
- `crates/causal/src/engine.rs`
- `crates/mcp/src/tools.rs`
- `crates/mcp/src/server.rs`
- `crates/mcp/src/lib.rs`
- `crates/mcp/tests/integration.rs`
- `src/main.rs`
