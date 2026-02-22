# Story 7.1: Cross-Store Consistency Checks

Status: ready-for-dev

## Story

As an operator,
I want the system to verify that Tantivy and Fjall are in sync on startup and after every sync operation,
so that agents never get stale or inconsistent results.

## Acceptance Criteria (BDD)

**Given** the system starting up
**When** both stores are loaded
**Then** the system compares the last-synced git SHA and document count across Tantivy and Fjall (FR39, FR41)
**And** if both match, the system proceeds normally

**Given** a sync operation (full or incremental) completing
**When** post-sync verification runs
**Then** the system compares SHA and doc count across both stores (FR40, FR41)
**And** if both match, the sync is marked successful

**Given** startup or post-sync verification detects SHA or doc count mismatch
**When** drift is detected
**Then** the system emits a warning to stderr with details of the mismatch (FR42)
**And** offers to auto-rebuild from git to restore consistency (FR42)

**Given** incremental sync completes but document counts diverge between stores
**When** the sanity check runs
**Then** the system automatically escalates to a full rebuild (FR43)
**And** logs the escalation reason

**Given** a full rebuild triggered by auto-escalation
**When** the rebuild completes
**Then** consistency is re-verified
**And** if still inconsistent after rebuild, a clear error is raised (no infinite retry loop)

## Tasks / Subtasks

- [ ] Implement consistency domain model and verifier in `crates/ingest/src/consistency.rs`.
  - [ ] Add `pub struct ConsistencySnapshot { source: ConsistencySource, git_sha: String, doc_count: u64 }` and `pub enum ConsistencySource { Tantivy, FjallMeta }`.
  - [ ] Add `pub enum ConsistencyStatus { InSync, Drift { sha_mismatch: bool, count_mismatch: bool } }` and `pub struct ConsistencyReport { status: ConsistencyStatus, tantivy: ConsistencySnapshot, fjall: ConsistencySnapshot }`.
  - [ ] Add `pub fn verify_cross_store_consistency(...) -> Result<ConsistencyReport, SpecDbError>` that compares `last_sync_sha` and `doc_count` from both stores.
- [ ] Wire metadata reads for Fjall meta keyspace values in `crates/causal/src/store.rs` and exports in `crates/causal/src/lib.rs`.
  - [ ] Implement `pub fn get_meta_last_sync_sha(&self) -> Result<Option<String>, SpecDbError>` reading key `last_sync_sha`.
  - [ ] Implement `pub fn get_meta_doc_count(&self) -> Result<Option<u64>, SpecDbError>` reading key `doc_count`.
  - [ ] Ensure `meta` keyspace keys stay UTF-8 constants (`"last_sync_sha"`, `"doc_count"`) and are reused by ingest consistency logic.
- [ ] Add Tantivy-side snapshot helpers in `crates/search/src/indexer.rs` (or crate-equivalent public API file).
  - [ ] Implement `pub fn current_index_doc_count(&self) -> Result<u64, SpecDbError>` using the active reader/searcher.
  - [ ] Implement `pub fn current_index_sync_sha(&self) -> Result<Option<String>, SpecDbError>` from the committed metadata field used by sync.
  - [ ] Re-export snapshot APIs in `crates/search/src/lib.rs` for ingestion-layer access via trait boundary.
- [ ] Integrate startup check in CLI bootstrap path (`src/main.rs`) after store initialization and before serve loop.
  - [ ] Call `verify_cross_store_consistency` once both Tantivy and Fjall are loaded.
  - [ ] On `InSync`, continue startup.
  - [ ] On drift, emit warning to stderr with SHA/count mismatch details and execute configured auto-rebuild flow.
- [ ] Integrate post-sync verification into sync pipeline in `crates/ingest/src/sync.rs` for both full and incremental paths.
  - [ ] Invoke consistency verification at end of `full_rebuild` and `incremental_sync` success path before marking sync successful.
  - [ ] If mismatch after incremental path, log escalation reason and call `full_rebuild` once.
  - [ ] If mismatch persists after escalation rebuild, return `SpecDbError::ConsistencyError` with clear terminal message (no loop).
- [ ] Enforce bounded remediation behavior to prevent infinite retries in `crates/ingest/src/sync.rs`.
  - [ ] Add a remediation guard token such as `ConsistencyRemediationAttempt::{None, EscalatedRebuild}` per sync request.
  - [ ] Reject additional escalation attempts when already in `EscalatedRebuild` state.
  - [ ] Ensure failure exits with non-zero CLI status for critical drift that cannot be healed.
- [ ] Add tracing and operator-visible diagnostics for drift in `crates/ingest/src/consistency.rs`.
  - [ ] Emit span `spec_db.consistency.check` with fields: `trigger=startup|post_sync`, `sha_match`, `doc_count_match`.
  - [ ] Emit warning event with mismatched values (`tantivy.sha`, `fjall.sha`, `tantivy.doc_count`, `fjall.doc_count`).
  - [ ] Emit info event for auto-escalation decisions and final failure boundary.
- [ ] Add integration tests in `crates/ingest/tests/integration.rs` and targeted unit tests in `crates/ingest/src/consistency.rs`.
  - [ ] Startup success test: matching SHA/doc count proceeds with no rebuild.
  - [ ] Startup drift test: mismatch triggers warning and rebuild offer path.
  - [ ] Incremental divergence test: post-sync count mismatch triggers one full rebuild escalation.
  - [ ] Escalation-failure test: rebuild still inconsistent returns terminal `ConsistencyError` and does not retry repeatedly.
  - [ ] Regression test: compare known Tantivy/Fjall snapshots from Epic 2 + Epic 1 fixtures to verify shared `SpecId` corpus counts.

## Dev Notes

- This story implements FR39-FR43 and sits in Build Order step 11 (after both stores exist), centered in `crates/ingest/src/consistency.rs`.
- Cross-store verification must compare both dimensions: `last_sync_sha` and `doc_count`; either mismatch is drift.
- Fjall is source for meta keyspace values (`last_sync_sha`, `doc_count`), while Tantivy must expose equivalent persisted sync metadata and live doc count.
- Startup behavior follows process pattern P1: fail-fast on unrecoverable consistency faults, but attempt bounded auto-remediation first.
- Post-incremental behavior follows architecture cross-cutting rule: document-count divergence auto-escalates to exactly one full rebuild.
- Rebuild remediation must be bounded to avoid infinite loops; a second failed consistency check is terminal.
- Keep sync, consistency, and store operations synchronous inside crates; async boundary remains MCP layer only.
- Instrumentation naming should follow N5 convention; consistency spans should use dot-separated names.

### Project Structure Notes

- Primary implementation module: `crates/ingest/src/consistency.rs`.
- Orchestration touchpoints: `crates/ingest/src/sync.rs` and `src/main.rs` startup flow.
- Store integration points:
  - `crates/causal/src/store.rs` (`meta` keyspace accessors for SHA/count),
  - `crates/search/src/indexer.rs` (Tantivy count + sync SHA accessor APIs).
- Maintain crate boundaries from architecture: ingest coordinates through public APIs/traits, not private internals of search/causal crates.
- Do not modify planning state files (including `_bmad-output/implementation-artifacts/sprint-status.yaml`) in this story.

### References

- Epic 7 Story 7.1 acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Story 7.1: Cross-Store Consistency Checks]
- Data integrity mapping to ingest consistency module: [Source: _bmad-output/planning-artifacts/architecture.md#Requirements to Structure Mapping]
- Cross-store consistency concern and escalation expectation: [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns Identified]
- Process pattern P1 (fail-fast + graceful handling boundaries): [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- Fjall meta keyspace naming (`last_sync_sha`, `doc_count`): [Source: _bmad-output/planning-artifacts/architecture.md#Fjall Keyspace Design]
- Startup and sync flow insertion points: [Source: _bmad-output/planning-artifacts/architecture.md#Data Flow]
- Shared ID/cross-store model context: [Source: docs/project-context.md#Key Patterns for AI Agents]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story file authored with explicit startup and post-sync consistency verification flow.
- Includes bounded auto-escalation design (incremental -> one full rebuild) and explicit no-infinite-loop guard.
- Tasks are grounded to concrete files and function signatures across ingest, search, causal, and CLI entrypoint.

### Change Log

- 2026-02-23: Initial ready-for-dev draft created for Story 7.1.

### File List

- `_bmad-output/implementation-artifacts/7-1-cross-store-consistency-checks.md`
