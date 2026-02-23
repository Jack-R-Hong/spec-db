# Story 11.4: Detail Panel & Spec Inspection

Status: ready-for-dev

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

- [ ] Create `DetailPanel` component (AC: 1)
  - [ ] Create `web-ui/src/lib/components/DetailPanel.svelte`
  - [ ] Fixed width 360px, slides in from right edge
  - [ ] CSS transition for smooth open/close animation
  - [ ] Show when a node is selected (`selectedNodeId` is set), hide when null
- [ ] Implement spec detail fetching (AC: 1)
  - [ ] Add `GET /api/spec/{id}` REST endpoint in `crates/web/src/api.rs`
  - [ ] Returns: full frontmatter fields, markdown body, inbound edges, outbound edges
  - [ ] Fetch spec details when panel opens or selected node changes
  - [ ] Show loading state while fetching
- [ ] Render frontmatter section (AC: 1)
  - [ ] Display all frontmatter fields in a key-value layout
  - [ ] Fields: id (monospace), title, version, tags (chips), owner, created, depends_on
  - [ ] Style consistently with dark theme
- [ ] Render markdown body preview (AC: 1)
  - [ ] Display markdown body as rendered HTML (use a lightweight markdown renderer)
  - [ ] Limit height with scroll for long content
  - [ ] Syntax highlighting for code blocks if present
- [ ] Render edge lists (AC: 1)
  - [ ] Inbound edges section: list of (source spec ID + title, edge_type, trust score)
  - [ ] Outbound edges section: list of (target spec ID + title, edge_type, trust score)
  - [ ] Trust scores displayed as colored badges (green > 0.8, yellow 0.5-0.8, red < 0.5)
  - [ ] Edge type displayed as label
- [ ] Render downstream impact text list (AC: 2)
  - [ ] Below edges section, show "Downstream Impact" heading
  - [ ] List all downstream specs (from impact chain computation in Story 11.3) as spec ID + title
  - [ ] Clickable: clicking a spec in the list selects that node in the graph
- [ ] Handle panel interactions (AC: 3, 4)
  - [ ] Selecting a different node updates panel content (reactive to `selectedNodeId`)
  - [ ] Escape key closes panel (when search is not active)
  - [ ] Clicking outside the panel on the canvas does NOT close it (only Escape or deselecting node)
- [ ] Add tests
  - [ ] Component test: panel renders with correct width (360px)
  - [ ] Component test: frontmatter fields displayed correctly
  - [ ] Component test: edge lists show type and trust score
  - [ ] Component test: downstream impact text list rendered
  - [ ] Component test: Escape closes panel
  - [ ] Component test: selecting different node updates content
  - [ ] Integration test: `GET /api/spec/{id}` returns correct data

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

### Debug Log References

### Completion Notes List

### Change Log

### File List
