# Story 9.1: AI-Inferred Edge Export to YAML

Status: ready-for-dev

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

- [ ] Add `created_at` field to `CausalEdge` (AC: 1)
  - [ ] Add `created_at: Option<String>` (ISO 8601) to `CausalEdge` in `crates/core/src/types.rs`
  - [ ] Set `created_at` to current UTC timestamp when creating AI-inferred edges in `add_causal_link`
  - [ ] Ensure `#[serde(default)]` for backward compatibility with existing edges
- [ ] Implement edge export module (AC: 1, 2, 3, 4, 5)
  - [ ] Create export function: `pub fn export_ai_edges(edges: &[CausalEdge], lattice_dir: &Path) -> Result<()>`
  - [ ] Filter edges to include only `origin: AiInferred`
  - [ ] Serialize to YAML format: `edges: [{ source, target, edge_type, trust, origin, created_at }]`
  - [ ] Write atomically: write to `.lattice/edges.yaml.tmp` then `fs::rename` to `.lattice/edges.yaml`
  - [ ] Handle empty case: write `edges: []` or skip file creation
- [ ] Integrate export into `add_causal_link` flow (AC: 2)
  - [ ] After successful edge insertion (post-CSM validation), call export function
  - [ ] Export all current AI-inferred edges (not just the new one) — full rewrite
- [ ] Add tests (AC: 1-5)
  - [ ] Unit test: export produces correct YAML structure with all fields
  - [ ] Unit test: only AiInferred edges appear in export
  - [ ] Unit test: human-curated edges are excluded
  - [ ] Unit test: empty edge list produces `edges: []`
  - [ ] Unit test: atomic write (verify temp file is used)
  - [ ] Integration test: `add_causal_link` → verify `.lattice/edges.yaml` updated

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

### Debug Log References

### Completion Notes List

### Change Log

### File List
