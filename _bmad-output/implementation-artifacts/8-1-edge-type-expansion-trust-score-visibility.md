# Story 8.1: Edge Type Expansion & Trust Score Visibility

Status: review

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

- [x] Add `EdgeType` enum to `crates/core/src/types.rs` (AC: 1)
  - [x] Define `#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub enum EdgeType { DependsOn, Constrains, Implements }`
  - [x] Implement `Default for EdgeType` returning `DependsOn`
  - [x] Implement `Display` for EdgeType (lowercase snake_case: `depends_on`, `constrains`, `implements`)
  - [x] Implement `FromStr` for EdgeType with error on unknown variants
- [x] Add `EdgeOrigin` enum to `crates/core/src/types.rs` (AC: 2)
  - [x] Define `#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub enum EdgeOrigin { Human, AiInferred }`
  - [x] Implement `Display` for EdgeOrigin
- [x] Extend `CausalEdge` struct with new fields (AC: 1, 2)
  - [x] Add `edge_type: EdgeType` field with `#[serde(default)]`
  - [x] Add `trust: f64` field (existing or new — verify current struct shape)
  - [x] Add `origin: EdgeOrigin` field with `#[serde(default = "EdgeOrigin::human")]`
  - [x] Ensure `Default` impl sets `edge_type: DependsOn`, `trust: 1.0`, `origin: Human`
- [x] Update ingestion pipeline to set new fields on human-curated edges (AC: 3)
  - [x] In `crates/ingest/src/` where `CausalEdge` is constructed from frontmatter `depends_on`, set `edge_type: DependsOn`, `trust: 1.0`, `origin: Human`
- [x] Update MCP tool responses to include new fields (AC: 2)
  - [x] In `trace_impact` response serialization, include `edge_type`, `trust`, `origin`
  - [x] In `find_dependencies` response serialization, include `edge_type`, `trust`, `origin`
- [x] Update Fjall KV serialization (AC: 4)
  - [x] Verify bincode serialization of updated `CausalEdge` round-trips correctly
  - [x] Add migration path: edges deserialized without new fields get defaults (`DependsOn`, `1.0`, `Human`)
- [x] Export new types from `spec-db-core/src/lib.rs` (AC: 1)
  - [x] Add `EdgeType` and `EdgeOrigin` to public re-exports
- [x] Update and add tests (AC: 1, 2, 3, 4)
  - [x] Unit tests for `EdgeType` Display/FromStr round-trip
  - [x] Unit tests for `CausalEdge` serde with new fields
  - [x] Unit tests for backward-compatible deserialization (missing fields → defaults)
  - [x] Integration test: ingest a spec → verify edge has correct `edge_type`, `trust`, `origin`
  - [x] Verify all 175 existing tests still pass

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

openai/gpt-5.3-codex

### Debug Log References

- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`

### Completion Notes List

- Added `EdgeType` to core domain types with `Display`, `FromStr`, and default `DependsOn` behavior.
- Added `edge_type` to `CausalEdge` with `#[serde(default)]` and propagated it through causal engine metadata/storage/rebuild paths.
- Updated frontmatter ingestion (`depends_on`) to produce `EdgeType::DependsOn` human edges.
- Updated MCP `trace_impact` and `find_dependencies` edge payloads to include `edge_type`, `trust`, and `origin`.
- Updated all touched test edge helpers and inline fixtures to set `edge_type` explicitly.
- Added and extended tests for `EdgeType` round-trip/default, CausalEdge serde/backward compatibility, and ingestion edge metadata assertions.
- Verified `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` are green.

### Change Log

- Implemented Story 8.1 edge-type expansion and trust/origin visibility across core, causal, ingest, and MCP layers.
- Added regression coverage for edge metadata propagation and backward-compatible edge deserialization.

### File List

- `crates/core/src/types.rs`
- `crates/core/src/lib.rs`
- `crates/causal/src/engine.rs`
- `crates/causal/src/store.rs`
- `crates/ingest/src/pipeline.rs`
- `crates/mcp/src/tools.rs`
- `crates/causal/tests/integration.rs`
- `crates/ingest/tests/integration.rs`
- `crates/router/tests/integration.rs`
- `tests/acceptance_story_1_1.rs`
- `tests/acceptance_story_1_2.rs`
- `tests/acceptance_story_1_4.rs`
- `tests/acceptance_story_5_2.rs`
- `tests/acceptance_story_6_2.rs`
- `tests/acceptance_story_6_3.rs`
- `tests/acceptance_story_7_1.rs`
