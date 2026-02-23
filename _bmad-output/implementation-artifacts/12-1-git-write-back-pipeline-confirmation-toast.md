# Story 12.1: Git Write-Back Pipeline & Confirmation Toast

Status: ready-for-dev

## Story

As a spec author,
I want all UI edits to automatically write back to spec markdown files via git commit, with a confirmation toast before each action,
so that I can trust that my visual edits are persisted to the source of truth without manual file editing.

## Acceptance Criteria (BDD)

**Given** the `spec-db-web` crate contains `writeback.rs`
**When** a write-back operation is triggered
**Then** the pipeline: modifies the target spec's YAML frontmatter → writes the file → creates a git commit with a descriptive message (e.g., "lattice: add depends_on edge from spec::A to spec::B")

**Given** any UI action that would trigger a git write-back (edge add, edge remove, frontmatter edit)
**When** the user initiates the action
**Then** a confirmation toast appears at bottom-center: "Write to [spec-id]? This will create a git commit." with Confirm/Cancel buttons (FR72)

**Given** the user clicks "Cancel" on the confirmation toast
**When** the toast dismisses
**Then** no file modification or git commit occurs and the graph reverts the visual change

**Given** the user clicks "Confirm" on the confirmation toast
**When** the write-back pipeline runs
**Then** it completes the full round-trip (file modify → git commit → re-sync → graph refresh) in under 2 seconds (NFR35)

**Given** `AppState` uses `Arc<AppState>` with `Mutex<Option<UndoState>>`
**When** multiple write-back operations are requested concurrently
**Then** they are serialized — only one write-back executes at a time; reads remain concurrent

**Given** tracing is enabled
**When** a write-back operation runs
**Then** it emits spans under `spec_db.web.writeback.apply`

**Covers:** FR70, FR72, NFR35

## Tasks / Subtasks

- [ ] Create `writeback.rs` module in `crates/web/src/` (AC: 1, 5, 6)
  - [ ] Implement `WriteBackPipeline` struct
  - [ ] Method: `apply_edge_add(source: &SpecId, target: &SpecId, edge_type: &EdgeType) -> Result<()>`
  - [ ] Method: `apply_edge_remove(source: &SpecId, target: &SpecId, edge_type: &EdgeType) -> Result<()>`
  - [ ] Method: `apply_frontmatter_edit(spec_id: &SpecId, changes: FrontmatterChanges) -> Result<()>`
  - [ ] Each method: read spec file → parse frontmatter → apply change → write file → git add → git commit
  - [ ] Git commit messages: descriptive (e.g., "lattice: add depends_on edge from spec::auth::jwt to spec::auth::tokens")
  - [ ] Use `git2` crate for git operations (already in workspace)
  - [ ] Wrap in `Mutex` for serialized write access
  - [ ] Add tracing span: `spec_db.web.writeback.apply`
- [ ] Implement re-sync after write-back (AC: 4)
  - [ ] After git commit, trigger incremental sync to update indexes
  - [ ] After sync, notify frontend to refresh graph data
  - [ ] Full round-trip target: < 2 seconds
- [ ] Create `POST /api/writeback` endpoint (AC: 1)
  - [ ] Accept JSON body describing the write-back operation type and parameters
  - [ ] Acquire write lock, execute pipeline, release lock
  - [ ] Return updated graph state on success
  - [ ] Return error in standard shape on failure
- [ ] Implement `UndoState` (AC: 5)
  - [ ] Define `UndoState { commit_sha: String, created_at: Instant }` in `state.rs`
  - [ ] After successful write-back, store `UndoState` with the new commit SHA
  - [ ] Clear after 5 seconds (used by Story 12.3)
- [ ] Create `ToastNotification` frontend component (AC: 2, 3)
  - [ ] Create `web-ui/src/lib/components/ToastNotification.svelte`
  - [ ] Position: bottom-center, auto-dismiss after confirmation
  - [ ] Confirmation toast: message + Confirm/Cancel buttons
  - [ ] Error toast: message + auto-dismiss after 5 seconds
  - [ ] On Cancel: revert optimistic UI change, dismiss toast
  - [ ] On Confirm: call `POST /api/writeback`, show loading state
- [ ] Add tests (AC: 1-6)
  - [ ] Unit test: write-back adds `depends_on` to frontmatter correctly
  - [ ] Unit test: write-back removes `depends_on` from frontmatter correctly
  - [ ] Unit test: git commit created with descriptive message
  - [ ] Unit test: concurrent write-backs are serialized (second waits for first)
  - [ ] Unit test: UndoState is set after write-back
  - [ ] Component test: toast shows Confirm/Cancel buttons
  - [ ] Component test: Cancel reverts change
  - [ ] Integration test: full write-back round-trip < 2 seconds

## Dev Notes

- This is the foundation story for all editing operations in Epic 12. Stories 12.2 and 12.3 depend on this pipeline.
- Write-back modifies YAML frontmatter only — never the markdown body. Use a YAML-aware parser that preserves formatting.
- The `Mutex<Option<UndoState>>` pattern ensures only one write at a time while reads remain concurrent.
- `git2` crate is already used by `crates/ingest/` for sync operations. Follow the same git operation patterns.
- Important: the write-back pipeline lives in `crates/web/src/writeback.rs`, NOT in `crates/ingest/`. Write-back is web-UI-only.

### Project Structure Notes

- New file: `crates/web/src/writeback.rs`
- Modified: `crates/web/src/state.rs` (add UndoState)
- Modified: `crates/web/src/api.rs` (add POST /api/writeback endpoint)
- New component: `web-ui/src/lib/components/ToastNotification.svelte`

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 12.1]
- [Source: _bmad-output/planning-artifacts/architecture.md#Write-Back Pipeline]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Toast Notification]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
