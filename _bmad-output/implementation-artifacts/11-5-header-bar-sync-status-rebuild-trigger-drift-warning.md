# Story 11.5: Header Bar — Sync Status, Rebuild Trigger & Drift Warning

Status: ready-for-dev

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

- [ ] Create `HeaderBar` component (AC: 1)
  - [ ] Create `web-ui/src/lib/components/HeaderBar.svelte`
  - [ ] Fixed height 40px, full width, positioned at top
  - [ ] Dark theme styling consistent with canvas
  - [ ] Layout: logo/title left, status center, actions right
- [ ] Implement sync status display (AC: 1)
  - [ ] Fetch status from `GET /api/status` on page load and after sync operations
  - [ ] Display: spec count (e.g., "42 specs"), abbreviated SHA (e.g., "a1b2c3d"), relative timestamp (e.g., "synced 2 min ago")
  - [ ] Auto-refresh relative timestamp every 30 seconds
- [ ] Implement `POST /api/sync` endpoint (AC: 2, 3)
  - [ ] Add endpoint in `crates/web/src/api.rs`
  - [ ] Query parameter: `mode=full` (full rebuild) or `mode=incremental` (incremental sync)
  - [ ] Call existing sync logic from `crates/ingest/`
  - [ ] Return updated status on success
  - [ ] Return error in standard shape on failure
- [ ] Implement rebuild/sync buttons (AC: 2, 3)
  - [ ] "Sync" button: calls `POST /api/sync?mode=incremental`
  - [ ] "Rebuild" button: calls `POST /api/sync?mode=full`
  - [ ] Show loading spinner during operation (disable buttons)
  - [ ] On completion: refresh graph data and status display
  - [ ] On error: show error toast
- [ ] Implement drift warning indicator (AC: 4)
  - [ ] Add drift detection to `GET /api/status` response (boolean `drift_detected` field)
  - [ ] When `drift_detected: true`, show yellow warning icon in header
  - [ ] Tooltip on hover: "Index drift detected — consider a full rebuild"
  - [ ] Warning disappears after successful full rebuild
- [ ] Add tests
  - [ ] Component test: header renders with spec count, SHA, timestamp
  - [ ] Component test: sync button triggers API call
  - [ ] Component test: rebuild button triggers API call with mode=full
  - [ ] Component test: drift warning icon shown when drift detected
  - [ ] Unit test: `POST /api/sync` endpoint triggers sync and returns status
  - [ ] Unit test: drift detection in status response

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

### Debug Log References

### Completion Notes List

### Change Log

### File List
