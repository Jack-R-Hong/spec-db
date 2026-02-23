# Story 4.2: Incremental Sync via Git Diff

Status: review

## Story

As a spec author,
I want changed specs automatically detected and re-indexed via git diff,
so that my updates are reflected in search and graph within seconds without a full rebuild.

## Acceptance Criteria (BDD)

**Given** specs that have been modified since the last sync
**When** incremental sync runs
**Then** only the changed files are processed via `git diff` against the last-synced SHA (FR21)
**And** modified specs are re-parsed and re-indexed in both stores

**Given** a spec file that was renamed (path changed, content same or different)
**When** incremental sync runs with git rename detection (`-M` flag)
**Then** the renamed file is correctly identified and re-indexed without duplication (FR22)

**Given** a spec file that was deleted from the repository
**When** incremental sync runs
**Then** the spec is removed from both the search index and causal graph (FR23)
**And** edges referencing the deleted spec are cleaned up

**Given** a repository with a few changed files among 100+ specs
**When** incremental sync executes
**Then** it completes in < 2 seconds (NFR6)

**Given** incremental sync completes
**When** the operation finishes
**Then** the last-synced SHA is updated in both stores (FR25)
**And** document counts are compared across stores as a sanity check
**And** if counts diverge, the system auto-escalates to a full rebuild

## Tasks / Subtasks

- [x] Implement incremental sync orchestrator in `crates/ingest/src/sync.rs`.
  - [x] Add `pub fn incremental_sync(&self) -> Result<SyncReport, SpecDbError>` to `GitSync` with HEAD/metadata orchestration.
  - [x] Read `last_sync_sha` from metadata and resolve target `HEAD` SHA for comparison window.
  - [x] Fast-path return when `last_sync_sha == head_sha` with no store mutations.
- [x] Build git diff change-set from commit trees in `crates/ingest/src/sync.rs`.
  - [x] Resolve `old_commit` from stored SHA and `new_commit` from `HEAD`, then load trees using `Commit::tree()`.
  - [x] Generate diff via `Repository::diff_tree_to_tree(Some(old_tree), Some(new_tree), None)`.
  - [x] Apply rename detection (`-M` equivalent) with `Diff::find_similar(Some(&mut find_opts))` and `DiffFindOptions::renames(true)`.
  - [x] Restrict processing to configured spec directory and `.md` files using `DiffFile::path()` normalization.
- [x] Process deltas with status-aware handlers in `crates/ingest/src/sync.rs`.
  - [x] Handle `Delta::Modified` by removing old ID derived from old blob content, then ingesting the new blob.
  - [x] Handle `Delta::Renamed` by removing old ID and ingesting the renamed path once.
  - [x] Handle `Delta::Deleted` by removing spec from search + causal graph via pipeline.
  - [x] Handle `Delta::Added` by ingesting the new blob from HEAD tree.
- [x] Persist sync metadata and run divergence safety check in `crates/ingest/src/sync.rs`.
  - [x] On success, update `last_sync_sha` in Fjall metadata and search metadata file to `head_sha`.
  - [x] Recompute/verify `doc_count` using live node count after delta application.
  - [x] If counts diverge, auto-trigger `full_rebuild()` immediately.
- [x] Add integration tests in `crates/ingest/tests/integration.rs` for incremental flows.
  - [x] Modified-file test: one changed spec reindexed with updated searchable content.
  - [x] Delete test: removed spec absent from search and graph.
  - [x] Added-file test: new spec indexed and counted.
  - [x] No-change test: incremental sync returns zero ingested.
  - [x] Missing-prior-SHA test: incremental path escalates to full rebuild.

## Dev Notes

- Story scope is Build Order #5 sync work in existing `spec-db-ingest`; implementation belongs in `crates/ingest/src/sync.rs`.
- Incremental sync is commit-to-commit, not worktree-to-index: compare persisted `last_sync_sha` tree to current `HEAD` tree for deterministic behavior.
- Git rename detection (`git diff -M` equivalent) in git2 0.20.4 is two-phase:
  - build `Diff` (`Repository::diff_tree_to_tree`),
  - run similarity transform (`Diff::find_similar` + `DiffFindOptions::renames(true)`).
- Delta handling rules:
  - `Delta::Modified` and `Delta::Added` -> parse/validate/ingest through Story 3.2 pipeline,
  - `Delta::Renamed` -> remove old identity artifacts then ingest new path once,
  - `Delta::Deleted` -> remove doc/node and cleanup inbound/outbound edges.
- Cleanup must include causal edge pruning, not only node deletion, to satisfy FR23.
- Process pattern P2 applies here as batch atomicity: use Fjall batches for cross-keyspace graph consistency during incremental updates.
- Consistency guardrail after every incremental run:
  - compare doc counts across stores,
  - if divergent, auto-escalate to full rebuild instead of leaving inconsistent partial state.
- NFR targeting:
  - NFR6 `<2s` with small change set on 100+ corpus,
  - FR25 metadata update in both stores,
  - FR21/22/23 correctness for modified/renamed/deleted flows.
- Maintain tracing spans for observability: `spec_db.sync.incremental`, `spec_db.sync.diff`, `spec_db.sync.rename_detect`, `spec_db.sync.consistency_check`.

### Project Structure Notes

- Primary implementation target: `crates/ingest/src/sync.rs`.
- Consistency hooks and divergence logic: `crates/ingest/src/consistency.rs`.
- Dependency flows to respect:
  - Story 3.2 ingestion pipeline for parse/validate/upsert behavior,
  - Epic 2 search APIs for add/remove/commit,
  - Epic 1 causal APIs for node/edge mutation and traversal-safe cleanup.
- Keep module depth and public API patterns aligned with architecture (`lib.rs` explicit exports, no wildcard re-exports).
- Do not modify `sprint-status.yaml` as part of this story.

### References

- Epic 4 story definition and BDD acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Epic 4: Git-Centric Sync]
- Unified ingestion pipeline dependency (Story 3.2): [Source: _bmad-output/planning-artifacts/epics.md#Story 3.2: Unified Spec Ingestion Pipeline]
- Search index operations baseline (Epic 2): [Source: _bmad-output/planning-artifacts/epics.md#Epic 2: Spec Discovery & Search]
- Causal graph/edge semantics baseline (Epic 1): [Source: _bmad-output/planning-artifacts/epics.md#Epic 1: Foundation & Causal Knowledge Graph]
- Sync crate target and build-order placement: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- Process pattern P2 incremental atomicity (Fjall batches): [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- Cross-store consistency and auto-escalation requirement context: [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns Identified]
- Incremental sync pattern and shared ID scheme context: [Source: docs/project-context.md#Key Patterns for AI Agents]
- git2 diff API (`Diff`, `DiffDelta`, `Delta`, `DiffFile`): [Source: https://docs.rs/git2/0.20.4/git2/struct.Diff.html]
- git2 rename detection options (`DiffFindOptions`): [Source: https://docs.rs/git2/0.20.4/git2/struct.DiffFindOptions.html]
- git2 tree-to-tree diff signature: [Source: https://docs.rs/git2/latest/src/git2/repo.rs]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Added `GitSync::incremental_sync()` using git tree-to-tree diff from stored `last_sync_sha` to `HEAD` with rename detection enabled.
- Implemented delta handling for `Added`, `Modified`, `Renamed`, and `Deleted` by reading blobs from git trees and applying live-store pipeline updates.
- Added helpers for blob reads and `SpecId` extraction from markdown to support remove-then-add behavior for modified/renamed files.
- Added divergence guardrails and metadata persistence updates (`last_sync_sha`, `doc_count`) with auto-escalation to `full_rebuild()` on mismatch.
- Added integration tests for modified, deleted, added, no-change, and no-prior-sha incremental-sync scenarios.

### Change Log

- 2026-02-23: Initial ready-for-dev draft for Story 4.2.

### File List

- `_bmad-output/implementation-artifacts/4-2-incremental-sync-git-diff.md`
- `crates/ingest/src/sync.rs`
- `crates/ingest/tests/integration.rs`
