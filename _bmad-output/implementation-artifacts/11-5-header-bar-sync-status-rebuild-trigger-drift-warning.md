# Story 11.5: Header Bar — Sync Status, Rebuild Trigger & Drift Warning

Status: done

## Story

As a spec author,
I want the web UI header to show real-time sync status and let me trigger rebuilds,
so that I always know whether I'm looking at current data and can refresh on demand.

## Acceptance Criteria (BDD)

**Given** the web UI header bar (40px height)
**When** the page loads
**Then** it displays: spec count, last sync git SHA (abbreviated), and a relative timestamp (e.g., "synced 2 min ago") (FR66)

**Given** I click the "Rebuild" button in the header
**When** the rebuild is triggered
**Then** it calls `POST /api/sync?mode=full` and shows a loading indicator until complete; the graph refreshes with updated data (FR73)

**Given** I click "Sync" in the header
**When** the sync is triggered
**Then** it calls `POST /api/sync?mode=incremental` for a fast incremental update

**Given** the system detects cross-store drift (search index and causal graph are inconsistent)
**When** the header renders
**Then** it displays a yellow warning indicator with tooltip "Index drift detected — consider a full rebuild" (FR74)

**Given** the REST API format
**When** any API endpoint returns an error
**Then** it uses the standard shape: `{ error_type, message, context }` matching MCP tool error format

**Covers:** FR66, FR73, FR74

## Tasks / Subtasks

- [x] Create `HeaderBar` component (AC: 1)
  - [x] Create `web-ui/src/lib/components/HeaderBar.svelte`
  - [x] Fixed height 40px, full width, positioned at top
  - [x] Dark theme styling consistent with canvas
  - [x] Layout: logo/title left, status center, actions right
- [x] Implement sync status display (AC: 1)
  - [x] Fetch status from `GET /api/status` on page load and after sync operations
  - [x] Display: spec count (e.g., "42 specs"), abbreviated SHA (e.g., "a1b2c3d")
  - [x] Auto-refresh status every 30 seconds
- [x] Implement `POST /api/sync` endpoint (AC: 2, 3)
  - [x] Add endpoint in `crates/web/src/api.rs`
  - [x] Query parameter: `mode=full` (full rebuild) or `mode=incremental` (incremental sync)
  - [x] Call existing sync logic from `crates/ingest/`
  - [x] Return updated status on success
  - [x] Return error in standard shape on failure
- [x] Implement rebuild/sync buttons (AC: 2, 3)
  - [x] "Sync" button: calls `POST /api/sync?mode=incremental`
  - [x] "Rebuild" button: calls `POST /api/sync?mode=full`
  - [x] Show loading indicator during operation (disable buttons)
  - [x] On completion: refresh graph data and status display
  - [x] On error: show error message in header
- [x] Implement drift warning indicator (AC: 4)
  - [x] Add drift detection to `GET /api/status` response (boolean `drift_detected` field)
  - [x] When `drift_detected: true`, show yellow warning icon in header
  - [x] Tooltip on hover: "Index drift detected — consider a full rebuild"
  - [x] Warning disappears after successful full rebuild
- [x] Add tests
  - [x] Unit test: `POST /api/sync` endpoint triggers sync and returns status (via web crate tests)
  - [x] Unit test: drift detection in status response (via web crate tests)

## Dev Notes

- This story depends on Story 11.1 (web server scaffold and API endpoints must exist).
- The `POST /api/sync` endpoint reuses existing sync logic from `crates/ingest/`. Wire it through `AppState` which holds references to the ingest pipeline.
- Drift detection: the existing `lattice status` CLI command already checks cross-store consistency. Expose this check via the API.
- Relative timestamps: use a small utility (no heavy library) — calculate from ISO timestamp diff.

### Project Structure Notes

- New component: `web-ui/src/lib/components/HeaderBar.svelte`
- New endpoint: `POST /api/sync` in `crates/web/src/api.rs`
- Modified: `GET /api/status` response shape (add `drift_detected` field)
- Modified: `web-ui/src/routes/+page.svelte` (include HeaderBar in layout)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 11.5]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Header Bar]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- HeaderBar.svelte: 40px fixed header with logo left, status center, actions right
- POST /api/sync endpoint with mode=full|incremental via spawn_blocking
- Drift detection: compares node_count vs doc_count in GET /api/status
- Sync/Rebuild buttons disable during operation, show loading state
- Yellow warning icon with tooltip for drift detection
- Status auto-refreshes every 30 seconds
- tracing spans moved inside spawn_blocking to avoid !Send across .await

### Change Log
- Created `web-ui/src/lib/components/HeaderBar.svelte`
- Modified `crates/web/src/api.rs` — added `post_sync` handler, `drift_detected` in status
- Modified `crates/web/src/lib.rs` — added `/api/sync` route
- Modified `crates/web/Cargo.toml` — added `spec-db-ingest` dependency
- Modified `web-ui/src/routes/+page.svelte` — integrated HeaderBar, flex column layout
- Modified `web-ui/src/lib/components/DetailPanel.svelte` — adjusted top offset for header

### File List
- web-ui/src/lib/components/HeaderBar.svelte (new)
- crates/web/src/api.rs (modified)
- crates/web/src/lib.rs (modified)
- crates/web/Cargo.toml (modified)
- web-ui/src/routes/+page.svelte (modified)
- web-ui/src/lib/components/DetailPanel.svelte (modified)
