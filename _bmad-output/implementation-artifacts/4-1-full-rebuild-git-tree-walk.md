# Story 4.1: Full Rebuild from Git Tree Walk

Status: review

## Story

As a spec author,
I want to rebuild the entire search index and causal graph from the git repository,
so that I can recover from any data corruption and guarantee my indexes match the source of truth.

## Acceptance Criteria (BDD)

**Given** a git repository containing spec files in the configured directory
**When** I trigger a full rebuild
**Then** the system walks the git tree, discovers all spec files, and ingests each through the pipeline (FR20)
**And** the rebuild produces identical indexes regardless of when it is run (idempotent) (FR24)

**Given** a full rebuild in progress
**When** the new indexes are ready
**Then** they are built in temporary directories and atomically swapped into place (NFR15)
**And** the old indexes are not modified until the swap succeeds

**Given** a repository with 100+ specs
**When** a full rebuild executes
**Then** it completes in < 5 seconds (NFR5)

**Given** a completed full rebuild
**When** the operation finishes
**Then** both Tantivy and Fjall stores record the current git commit SHA (FR25)
**And** both stores record the correct document count

**Given** a previous index exists with stale data
**When** I run a full rebuild
**Then** the old data is completely replaced — no remnants of stale specs remain

## Tasks / Subtasks

- [x] Implement the full rebuild entrypoint in `crates/ingest/src/sync.rs`.
  - [x] Add `pub fn full_rebuild(&mut self) -> Result<SyncReport, SpecDbError>` on `GitSync` as the single orchestration API used by CLI `rebuild` and MCP `sync(mode="full")`.
  - [x] Load repository HEAD commit (`Repository::head` + peel to commit) and capture `head_sha` for metadata writes.
  - [x] Resolve configured specs root (for example `specs/`) and ensure rebuild only processes files under that subtree.
- [x] Implement git tree-walk based discovery in `crates/ingest/src/sync.rs`.
  - [x] Add `fn discover_specs_from_head_tree(repo: &Repository, head_commit: &Commit, specs_root: &Path) -> Result<Vec<RepoSpecEntry>, SyncError>`.
  - [x] Use `Commit::tree()` and `Tree::walk(TreeWalkMode::PreOrder, ...)` to enumerate entries.
  - [x] Filter to markdown specs (`.md`) and normalize repo-relative paths with deterministic ordering (sort ascending by canonical path) to enforce idempotency.
- [x] Build isolated temp targets before swap (Process Pattern P2) in `crates/ingest/src/sync.rs`.
  - [x] Add `fn prepare_rebuild_staging_dirs(...) -> Result<RebuildStaging, SyncError>` that creates sibling temp dirs for both stores.
  - [x] Construct fresh Tantivy index and fresh Fjall store on temp paths only; never mutate live paths during staging.
  - [x] Feed each discovered file through Story 3.2 ingestion pipeline APIs (`parser` + `validate` + unified ingest) against staging handles.
- [x] Implement atomic swap semantics for stale-data-free replacement in `crates/ingest/src/sync.rs`.
  - [x] Add `fn atomic_swap_rebuild_outputs(staging: &RebuildStaging, live_paths: &StorePaths) -> Result<(), SyncError>`.
  - [x] Execute rename/swap as temp-dir-then-swap for both `data/tantivy/` and `data/fjall/` only after successful staging verification.
  - [x] Ensure cleanup policy removes old paths only after swap success; on failure, keep old live stores untouched.
- [x] Persist post-rebuild sync metadata in both stores in `crates/ingest/src/sync.rs` and `crates/ingest/src/consistency.rs`.
  - [x] Write `last_sync_sha=<head_sha>` into meta keyspace and corresponding search metadata location.
  - [x] Write `doc_count=<ingested_count>` into both stores.
  - [x] Run immediate cross-store check (`doc_count` + `last_sync_sha`) and fail loudly if mismatch is detected.
- [x] Add idempotency and performance validation tests in `crates/ingest/tests/integration.rs`.
  - [x] Add fixture repo test: run full rebuild twice at same HEAD and assert identical counts/IDs and no duplicates.
  - [x] Add stale-data replacement test: seed obsolete docs in live stores, run rebuild, verify obsolete IDs absent.
  - [x] Add perf guard test/harness for 100+ specs target (<5s) with deterministic fixture generation.

## Dev Notes

- Story scope is Build Order #5 in existing `spec-db-ingest`; implement in `crates/ingest/src/sync.rs`, not a new crate.
- Full rebuild must be deterministic and idempotent: same git tree snapshot yields byte-equivalent logical corpus in both stores.
- Use Git as source of truth exclusively: enumerate from commit tree object, not filesystem walk, to avoid untracked/dirty-file drift.
- Apply process pattern P2 strictly: stage complete rebuild in temp dirs, validate, then atomic swap; no partial visible state.
- Cross-store consistency invariant after rebuild: Tantivy and Fjall must agree on `last_sync_sha` and `doc_count`.
- Reuse Story 3.2 ingestion pipeline for parsing, SpecId validation, duplicate handling, and node/edge creation.
- Respect crate boundaries from architecture:
  - `search` owns Tantivy IO,
  - `causal` owns Fjall IO,
  - `ingest` orchestrates git2 + pipeline + consistency.
- NFR targeting:
  - NFR5 `<5s` for 100+ specs,
  - NFR15 atomic replacement semantics,
  - FR24 idempotent outputs,
  - FR25 SHA/doc_count persistence in both stores.
- git2 0.20.4 API details for implementation:
  - Tree traversal: `Tree::walk(TreeWalkMode::PreOrder, callback)` with `TreeWalkResult::{Ok, Skip, Abort}`.
  - Commit/tree access: `Commit::tree()`, `Commit::id()`.
  - If repo-relative subtree filtering is needed, use walk path prefix and path normalization.
- Instrument orchestration span names per architecture naming pattern: `spec_db.sync.full_rebuild`, `spec_db.sync.tree_walk`, `spec_db.sync.atomic_swap`.

### Project Structure Notes

- Primary implementation file: `crates/ingest/src/sync.rs`.
- Supporting consistency checks: `crates/ingest/src/consistency.rs`.
- Existing pipeline dependencies to call from sync path:
  - `crates/ingest/src/parser.rs`,
  - `crates/ingest/src/validate.rs`,
  - unified ingestion logic produced in Story 3.2.
- Expected store paths remain architecture-aligned: `data/tantivy/` and `data/fjall/`.
- Do not modify `sprint-status.yaml` in this story.

### References

- Epic 4 story definition and BDD acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Epic 4: Git-Centric Sync]
- Unified ingestion pipeline dependency (Story 3.2): [Source: _bmad-output/planning-artifacts/epics.md#Story 3.2: Unified Spec Ingestion Pipeline]
- Search subsystem responsibilities (Epic 2): [Source: _bmad-output/planning-artifacts/epics.md#Epic 2: Spec Discovery & Search]
- Causal graph responsibilities (Epic 1): [Source: _bmad-output/planning-artifacts/epics.md#Epic 1: Foundation & Causal Knowledge Graph]
- Sync crate target and file map (`sync.rs`, `consistency.rs`): [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- Process pattern P2 atomicity rules: [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- Cross-store consistency requirements (`last_sync_sha`, `doc_count`): [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns Identified]
- Git source-of-truth and incremental/full sync principle: [Source: docs/project-context.md#Git Is Source of Truth]
- git2 tree walking API: [Source: https://docs.rs/git2/0.20.4/git2/struct.Tree.html]
- git2 commit tree access API: [Source: https://docs.rs/git2/0.20.4/git2/struct.Commit.html]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Implemented `GitSync` full rebuild orchestration with HEAD SHA capture, git tree walk discovery, deterministic sort, and parse-error-tolerant ingestion.
- Added staging-dir rebuild flow with rollback-aware atomic swap semantics for Tantivy and Fjall stores.
- Persisted sync metadata (`last_sync_sha`, `doc_count`) to Fjall meta and Tantivy metadata file.
- Added integration coverage for ingest count, idempotency, stale-data replacement, and metadata persistence.
- Verified `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --all -- --check`.

### Change Log

- 2026-02-23: Initial ready-for-dev draft for Story 4.1.

### File List

- `_bmad-output/implementation-artifacts/4-1-full-rebuild-git-tree-walk.md`
- `crates/ingest/Cargo.toml`
- `crates/ingest/src/lib.rs`
- `crates/ingest/src/sync.rs`
- `crates/ingest/tests/integration.rs`
