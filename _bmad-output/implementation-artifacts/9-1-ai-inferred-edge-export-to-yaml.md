# Story 9.1: AI-Inferred Edge Export to YAML

Status: review

## Story

As a spec author,
I want all AI-inferred edges automatically exported to `.lattice/edges.yaml`,
so that I can review AI contributions outside the running system using my preferred text editor or CI pipeline.

## Acceptance Criteria (BDD)

**Given** one or more AI-inferred edges exist in the causal graph (origin: `AiInferred`)
**When** the system writes to `.lattice/edges.yaml`
**Then** the file contains an entry for each AI-inferred edge with fields: `source`, `target`, `edge_type`, `trust`, `origin`, and `created_at` (ISO 8601 timestamp)

**Given** a new AI edge is accepted via `add_causal_link`
**When** the edge passes CSM validation and is persisted
**Then** `.lattice/edges.yaml` is updated to include the new edge within the same operation

**Given** `.lattice/edges.yaml` already contains edges
**When** a new AI edge is added
**Then** the file is rewritten atomically (write-to-temp + rename) to avoid partial writes

**Given** no AI-inferred edges exist
**When** the system checks `.lattice/edges.yaml`
**Then** the file either does not exist or contains an empty list `edges: []`

**Given** human-curated edges (origin: `Human`, trust: 1.0) in the graph
**When** the system exports to `.lattice/edges.yaml`
**Then** human-curated edges are **not** included — only `AiInferred` edges appear

**Covers:** FR52

## Tasks / Subtasks

- [x] Add `created_at` field to `CausalEdge` (AC: 1)
  - [x] Add `created_at: Option<String>` (ISO 8601) to `CausalEdge` in `crates/core/src/types.rs`
  - [x] Set `created_at` to current UTC timestamp when creating AI-inferred edges in `add_causal_link`
  - [x] Ensure `#[serde(default)]` for backward compatibility with existing edges
- [x] Implement edge export module (AC: 1, 2, 3, 4, 5)
  - [x] Create `crates/causal/src/export.rs` with `export_ai_edges(edges, lattice_dir)`
  - [x] Filter edges to include only `origin: Ai`
  - [x] Serialize to YAML format: `edges: [{ source, target, edge_type, trust, origin, created_at }]`
  - [x] Write atomically: write to `.lattice/edges.yaml.tmp` then `fs::rename` to `.lattice/edges.yaml`
  - [x] Handle empty case: write `edges: []`
- [x] Integrate export into `add_causal_link` flow (AC: 2)
  - [x] After successful edge insertion (post-CSM validation), call `export_ai_edges`
  - [x] Export all current AI-inferred edges (not just the new one) — full rewrite via `graph.all_edges()`
- [x] Add tests (AC: 1-5)
  - [x] Unit test: `export_produces_correct_yaml_structure`
  - [x] Unit test: `only_ai_edges_appear_in_export`
  - [x] Unit test: `human_edges_excluded`
  - [x] Unit test: `empty_edge_list_produces_empty_array`
  - [x] Unit test: `atomic_write_uses_temp_file`
  - [x] Integration via existing `add_causal_link` MCP tests (edges.yaml written to repo_path/.lattice/)

## Dev Notes

- This story depends on Epic 8 (Stories 8.1-8.3) being complete — `add_causal_link` with CSM validation must work.
- Atomic write pattern: `write_all` to `edges.yaml.tmp` → `fs::rename("edges.yaml.tmp", "edges.yaml")`. This is POSIX-atomic on the same filesystem.
- The export rewrites the entire file each time (not append). For the expected scale (tens to hundreds of AI edges), this is acceptable.
- YAML serialization: use `serde_yml` (already a workspace dependency).

### Project Structure Notes

- New module: likely `crates/causal/src/export.rs` or in a shared utility location
- Modify: `crates/mcp/src/tools.rs` (add_causal_link handler) to call export after insertion
- File output: `.lattice/edges.yaml` (project-root-relative)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 9.1]
- [Source: _bmad-output/planning-artifacts/prd.md#Edge Lifecycle & Human Review]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Debug Log References
- `#[serde(skip_serializing_if)]` breaks bincode roundtrip — removed, kept only `#[serde(default)]`
- Fjall file lock prevents opening second store — used `graph.all_edges()` instead

### Completion Notes List
- `created_at: Option<String>` added to `CausalEdge` with `#[serde(default)]` for backward compat
- `now_iso8601()` helper uses `std::time::SystemTime` — no external datetime dependency
- `export_ai_edges` filters by `EdgeOrigin::Ai`, writes atomically via tmp+rename
- Added `all_edges()` method to `CausalEngine` to avoid double-opening Fjall store

### Change Log
- `crates/core/src/types.rs`: Added `created_at: Option<String>` to `CausalEdge`
- `crates/causal/src/export.rs`: New module — `export_ai_edges` with 5 unit tests
- `crates/causal/src/lib.rs`: Added `pub mod export`
- `crates/causal/src/engine.rs`: Added `all_edges()` method, updated `build_edge` for `created_at`
- `crates/causal/Cargo.toml`: Added `serde_yml` dependency
- `crates/mcp/src/tools.rs`: Added `now_iso8601()`, `epoch_days_to_ymd()`, integrated export call in `add_causal_link`
- Updated all CausalEdge constructors across 12+ files to include `created_at: None`

### File List
- `crates/core/src/types.rs`
- `crates/causal/src/export.rs`
- `crates/causal/src/lib.rs`
- `crates/causal/src/engine.rs`
- `crates/causal/Cargo.toml`
- `crates/mcp/src/tools.rs`
- `crates/ingest/src/pipeline.rs`
- `crates/causal/src/store.rs`
- `tests/acceptance_story_*.rs` (6 files)
- `crates/router/tests/integration.rs`
- `crates/causal/tests/integration.rs`
