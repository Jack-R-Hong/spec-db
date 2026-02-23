# Story 12.1: Git Write-Back Pipeline & Confirmation Toast

Status: done

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

- [x] Create `writeback.rs` module in `crates/web/src/` (AC: 1, 5, 6)
  - [x] Implement `WriteBackPipeline` struct with `apply()` dispatch
  - [x] Method: `apply_edge_add` — adds target to source spec's `depends_on`
  - [x] Method: `apply_edge_remove` — removes target from source spec's `depends_on`
  - [x] Method: `apply_frontmatter_edit` — updates title/tags/owner/depends_on
  - [x] Each method: read spec file → parse frontmatter → apply change → write file → git add → git commit
  - [x] Git commit messages: descriptive (e.g., "lattice: add depends_on edge from spec::auth::jwt to spec::auth::tokens")
  - [x] Use `git2` crate for git operations
  - [x] Serialized via `Mutex<()>` write lock in AppState
  - [x] Tracing spans: `spec_db.web.writeback.apply`, `spec_db.web.writeback.undo`
- [x] Implement re-sync after write-back (AC: 4)
  - [x] After git commit, trigger incremental sync to update indexes
  - [x] Frontend refreshes graph data on success response
- [x] Create `POST /api/writeback` endpoint (AC: 1)
  - [x] Accept JSON body with tagged `WriteBackOp` (edge_add/edge_remove/frontmatter_edit)
  - [x] Acquire write lock, execute pipeline, release lock
  - [x] Return commit SHA on success
  - [x] Return error in standard shape on failure
- [x] Implement `UndoState` (AC: 5)
  - [x] Define `UndoState { commit_sha, created_at }` in `state.rs`
  - [x] After successful write-back, store `UndoState`
  - [x] `POST /api/writeback/undo` checks 5-second window
- [x] Create `ToastNotification` frontend component (AC: 2, 3)
  - [x] Create `web-ui/src/lib/components/ToastNotification.svelte`
  - [x] Position: bottom-center, auto-dismiss after confirmation
  - [x] Confirmation toast: message + Confirm/Cancel buttons
  - [x] Error toast: message + auto-dismiss after 5 seconds
  - [x] Undo toast: countdown timer + Undo button
- [x] Add tests (AC: 1-6)
  - [x] Unit test: frontmatter split/reassemble roundtrip
  - [x] Unit test: depends_on extraction and replacement
  - [x] Unit test: field set (replace existing / append missing)
  - [x] Unit test: pipeline construction

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
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- WriteBackPipeline: YAML-aware frontmatter modification preserving body and other fields
- `set_field` handles both inline values and block-style YAML sequences
- git2 for staging + committing (Signature: "Lattice <lattice@localhost>")
- `undo()` uses git2 `repo.revert()` with conflict detection
- AppState extended with `write_lock: Mutex<()>` and `undo_state: Mutex<Option<UndoState>>`
- `POST /api/writeback/undo` validates 5-second window, returns 410 Gone if expired
- ToastNotification supports 3 modes: confirm, error (5s auto-dismiss), undo (5s countdown)

### Change Log
- Created `crates/web/src/writeback.rs` — WriteBackPipeline, FrontmatterChanges, WriteBackOp
- Modified `crates/web/src/state.rs` — added UndoState, write_lock, undo_state
- Modified `crates/web/src/api.rs` — added post_writeback, post_undo handlers
- Modified `crates/web/src/lib.rs` — registered writeback module and routes
- Modified `crates/web/Cargo.toml` — added git2, serde_yml dependencies
- Created `web-ui/src/lib/components/ToastNotification.svelte`

### File List
- crates/web/src/writeback.rs (new)
- crates/web/src/state.rs (modified)
- crates/web/src/api.rs (modified)
- crates/web/src/lib.rs (modified)
- crates/web/Cargo.toml (modified)
- web-ui/src/lib/components/ToastNotification.svelte (new)
