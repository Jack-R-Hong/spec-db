# Story 8.1: Edge Type Expansion & Trust Score Visibility

Status: ready-for-dev

## Story

As a spec author,
I want the causal graph to support `constrains` and `implements` edge types alongside `depends_on`, with every edge displaying its trust score and origin,
so that I can model richer architectural relationships and distinguish human-curated from AI-inferred knowledge.

## Acceptance Criteria (BDD)

**Given** the `CausalEdge` type in `spec-db-core/types.rs`
**When** I add an `edge_type` field with variants `DependsOn`, `Constrains`, `Implements`
**Then** all existing edges default to `DependsOn` and the system compiles without breaking existing tests

**Given** the `CausalEdge` struct now has `edge_type`, `trust` (f64), and `origin` (enum: `Human`, `AiInferred`) fields
**When** I call `trace_impact` or `find_dependencies` via MCP tools
**Then** the response JSON includes `edge_type`, `trust`, and `origin` for every edge in the result

**Given** a spec with `depends_on: ["spec::auth::token-format"]` in its frontmatter
**When** the ingestion pipeline parses the spec
**Then** the resulting `CausalEdge` has `edge_type: DependsOn`, `trust: 1.0`, and `origin: Human`

**Given** the updated `CausalEdge` model
**When** I serialize/deserialize edges to/from Fjall KV store
**Then** the new fields round-trip correctly and existing stored edges migrate gracefully

**Covers:** FR55, FR51

## Tasks / Subtasks

- [ ] Add `EdgeType` enum to `crates/core/src/types.rs` (AC: 1)
  - [ ] Define `#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub enum EdgeType { DependsOn, Constrains, Implements }`
  - [ ] Implement `Default for EdgeType` returning `DependsOn`
  - [ ] Implement `Display` for EdgeType (lowercase snake_case: `depends_on`, `constrains`, `implements`)
  - [ ] Implement `FromStr` for EdgeType with error on unknown variants
- [ ] Add `EdgeOrigin` enum to `crates/core/src/types.rs` (AC: 2)
  - [ ] Define `#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub enum EdgeOrigin { Human, AiInferred }`
  - [ ] Implement `Display` for EdgeOrigin
- [ ] Extend `CausalEdge` struct with new fields (AC: 1, 2)
  - [ ] Add `edge_type: EdgeType` field with `#[serde(default)]`
  - [ ] Add `trust: f64` field (existing or new — verify current struct shape)
  - [ ] Add `origin: EdgeOrigin` field with `#[serde(default = "EdgeOrigin::human")]`
  - [ ] Ensure `Default` impl sets `edge_type: DependsOn`, `trust: 1.0`, `origin: Human`
- [ ] Update ingestion pipeline to set new fields on human-curated edges (AC: 3)
  - [ ] In `crates/ingest/src/` where `CausalEdge` is constructed from frontmatter `depends_on`, set `edge_type: DependsOn`, `trust: 1.0`, `origin: Human`
- [ ] Update MCP tool responses to include new fields (AC: 2)
  - [ ] In `trace_impact` response serialization, include `edge_type`, `trust`, `origin`
  - [ ] In `find_dependencies` response serialization, include `edge_type`, `trust`, `origin`
- [ ] Update Fjall KV serialization (AC: 4)
  - [ ] Verify bincode serialization of updated `CausalEdge` round-trips correctly
  - [ ] Add migration path: edges deserialized without new fields get defaults (`DependsOn`, `1.0`, `Human`)
- [ ] Export new types from `spec-db-core/src/lib.rs` (AC: 1)
  - [ ] Add `EdgeType` and `EdgeOrigin` to public re-exports
- [ ] Update and add tests (AC: 1, 2, 3, 4)
  - [ ] Unit tests for `EdgeType` Display/FromStr round-trip
  - [ ] Unit tests for `CausalEdge` serde with new fields
  - [ ] Unit tests for backward-compatible deserialization (missing fields → defaults)
  - [ ] Integration test: ingest a spec → verify edge has correct `edge_type`, `trust`, `origin`
  - [ ] Verify all 175 existing tests still pass

## Dev Notes

- `CausalEdge` already has `trust: f64` and `origin: EdgeOrigin` fields from Phase 1 (verify in `crates/core/src/types.rs`). The `EdgeOrigin` enum already exists. This story primarily adds `EdgeType` and ensures all API responses expose these fields.
- The `edge_type` field must use `#[serde(default)]` so that existing serialized edges in Fjall deserialize without breaking.
- Architecture patterns: S4 (all domain types in spec-db-core), N1 (modern module files), P1 (fail-fast errors).
- Bincode 2.0.1 with serde feature is used for Fjall serialization — test backward compat carefully.

### Project Structure Notes

- Primary files to modify: `crates/core/src/types.rs`, `crates/core/src/lib.rs`
- Secondary files: `crates/ingest/src/pipeline.rs` (or equivalent ingestion module), `crates/mcp/src/tools.rs` (response serialization)
- No new files or crates needed.

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 8.1]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: crates/core/src/types.rs]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
