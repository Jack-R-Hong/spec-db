# Story 9.2: Promote & Reject AI-Inferred Edges

Status: ready-for-dev

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

- [ ] Implement `promote_edge` MCP tool (AC: 1, 3, 4)
  - [ ] Define tool input schema: `source: String`, `target: String`, `edge_type: String` (optional, default "depends_on")
  - [ ] Look up edge in graph by (source, target, edge_type)
  - [ ] Validate edge exists → `not_found` error if missing
  - [ ] Validate edge is `AiInferred` → `validation_error` if already `Human`
  - [ ] Update edge: set `origin: Human`, `trust: 1.0`
  - [ ] Persist updated edge to Fjall KV
  - [ ] Re-export AI edges to `.lattice/edges.yaml` (promoted edge now excluded)
  - [ ] Return success response with updated edge details
- [ ] Implement `reject_edge` MCP tool (AC: 2, 3)
  - [ ] Define tool input schema: same as promote_edge
  - [ ] Look up edge in graph by (source, target, edge_type)
  - [ ] Validate edge exists → `not_found` error if missing
  - [ ] Remove edge from `CausaloidGraph`
  - [ ] Delete edge from Fjall KV
  - [ ] Re-export AI edges to `.lattice/edges.yaml`
  - [ ] Return success response confirming deletion
- [ ] Implement CLI commands (AC: 5)
  - [ ] Add `lattice edge promote <source> <target> [--type <edge_type>]` subcommand
  - [ ] Add `lattice edge reject <source> <target> [--type <edge_type>]` subcommand
  - [ ] Both commands call the same core logic as MCP tools
  - [ ] Print human-readable confirmation message on success
  - [ ] Print human-readable error message on failure
- [ ] Add tests (AC: 1-5)
  - [ ] Unit test: promote AI edge → origin becomes Human, trust becomes 1.0
  - [ ] Unit test: promote already-human edge → validation_error
  - [ ] Unit test: promote non-existent edge → not_found
  - [ ] Unit test: reject AI edge → edge removed from graph and KV
  - [ ] Unit test: reject non-existent edge → not_found
  - [ ] Unit test: after promote, edge no longer in `.lattice/edges.yaml`
  - [ ] Unit test: after reject, edge no longer in `.lattice/edges.yaml`
  - [ ] Integration test: CLI `lattice edge promote` end-to-end
  - [ ] Integration test: CLI `lattice edge reject` end-to-end

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

### Debug Log References

### Completion Notes List

### Change Log

### File List
