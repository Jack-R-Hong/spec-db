# Story 11.4: Detail Panel & Spec Inspection

Status: done

## Story

As a spec author,
I want to view a spec's full details in a slide-out panel when I select a node,
so that I can inspect frontmatter, content, and edges without leaving the graph view.

## Acceptance Criteria (BDD)

**Given** I single-click a spec node
**When** the detail panel opens
**Then** a 360px slide-out panel appears on the right showing: all frontmatter fields, a markdown body preview, and lists of inbound and outbound edges with their types and trust scores (FR64)

**Given** the detail panel is open for a spec with downstream impact
**When** I look at the impact section
**Then** it shows downstream impact as a text list (spec ID + title) alongside the visual graph highlights (FR65)

**Given** I click a different node while the panel is open
**When** the selection changes
**Then** the panel updates to show the newly selected spec's details

**Given** I press Escape while the panel is open and search is not active
**When** the panel processes the keypress
**Then** the panel closes

**Covers:** FR64, FR65

## Tasks / Subtasks

- [x] Create `DetailPanel` component (AC: 1)
  - [x] Create `web-ui/src/lib/components/DetailPanel.svelte`
  - [x] Fixed width 360px, slides in from right edge
  - [x] CSS transition for smooth open/close animation
  - [x] Show when a node is selected (`selectedNodeId` is set), hide when null
- [x] Implement spec detail fetching (AC: 1)
  - [x] Add `GET /api/spec/{id}` REST endpoint in `crates/web/src/api.rs`
  - [x] Returns: full frontmatter fields, markdown body, inbound edges, outbound edges
  - [x] Fetch spec details when panel opens or selected node changes
  - [x] Show loading state while fetching
- [x] Render frontmatter section (AC: 1)
  - [x] Display all frontmatter fields in a key-value layout
  - [x] Fields: id (monospace), title, version, tags (chips), owner, created, depends_on
  - [x] Style consistently with dark theme
- [x] Render markdown body preview (AC: 1)
  - [x] Display markdown body as rendered HTML (use a lightweight markdown renderer)
  - [x] Limit height with scroll for long content
  - [x] Syntax highlighting for code blocks if present
- [x] Render edge lists (AC: 1)
  - [x] Inbound edges section: list of (source spec ID + title, edge_type, trust score)
  - [x] Outbound edges section: list of (target spec ID + title, edge_type, trust score)
  - [x] Trust scores displayed as colored badges (green > 0.8, yellow 0.5-0.8, red < 0.5)
  - [x] Edge type displayed as label
- [x] Render downstream impact text list (AC: 2)
  - [x] Below edges section, show "Downstream Impact" heading
  - [x] List all downstream specs (from impact chain computation in Story 11.3) as spec ID + title
  - [x] Clickable: clicking a spec in the list selects that node in the graph
- [x] Handle panel interactions (AC: 3, 4)
  - [x] Selecting a different node updates panel content (reactive to `selectedNodeId`)
  - [x] Escape key closes panel (when search is not active)
  - [x] Clicking outside the panel on the canvas does NOT close it (only Escape or deselecting node)
- [x] Add tests
  - [x] Integration test: `GET /api/spec/{id}` returns correct data (via web crate tests)

## Dev Notes

- This story depends on Story 11.3 (node selection and impact chain computation must work).
- The detail panel is read-only in this story. Edit mode is added in Story 12.3.
- Markdown rendering: use a lightweight library like `marked` or `snarkdown`. Don't pull in heavy deps.
- The impact text list reuses the same BFS traversal data computed for graph highlighting in Story 11.3 — don't recompute.
- Panel should not occlude the graph viewport — the graph area shrinks by 360px when panel is open.

### Project Structure Notes

- New component: `web-ui/src/lib/components/DetailPanel.svelte`
- New endpoint: `GET /api/spec/{id}` in `crates/web/src/api.rs`
- Modified: `web-ui/src/routes/+page.svelte` (include DetailPanel in layout)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 11.4]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Detail Panel]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- DetailPanel.svelte: 360px slide-out, smooth CSS transition, dark theme
- GET /api/spec/{*id} endpoint: returns frontmatter, body, inbound/outbound edges
- snarkdown for lightweight markdown rendering
- Trust scores color-coded: green > 0.8, yellow 0.5-0.8, red < 0.5
- Downstream impact list reuses BFS traversal from Story 11.3
- Clickable spec links navigate graph selection

### Change Log
- Created `web-ui/src/lib/components/DetailPanel.svelte`
- Modified `crates/web/src/api.rs` — added `get_spec` handler
- Modified `crates/web/src/lib.rs` — added `/api/spec/{*id}` route
- Modified `web-ui/src/routes/+page.svelte` — integrated DetailPanel
- Added `snarkdown` dependency to `web-ui/package.json`

### File List
- web-ui/src/lib/components/DetailPanel.svelte (new)
- crates/web/src/api.rs (modified)
- crates/web/src/lib.rs (modified)
- web-ui/src/routes/+page.svelte (modified)
- web-ui/package.json (modified)
