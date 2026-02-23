# Story 11.3: Impact Chain Highlighting & Search-to-Focus

Status: done

## Story

As a spec author,
I want to click a node to see its impact chain highlighted in color, and search specs to focus the view,
so that I can quickly trace dependencies and find specific specs in a large graph.

## Acceptance Criteria (BDD)

**Given** I single-click a spec node in the graph
**When** the selection is processed
**Then** the node's downstream impact chain is highlighted in red (`#e94560`) and upstream dependencies in blue (`#5c7cfa`); all other nodes and edges dim

**Given** I type in the search bar (Ctrl+K to focus)
**When** I enter a query matching spec name, ID, or tag
**Then** non-matching nodes dim, matching nodes remain full opacity, and the graph viewport pans/zooms to center on matches (FR62)

**Given** I press Escape while search is active
**When** the search dismisses
**Then** all nodes return to normal opacity and the viewport stays at current position

**Given** graph interactions: pan, zoom, select, search
**When** any interaction occurs
**Then** the UI responds in under 100ms (NFR34)

**Given** keyboard shortcuts
**When** I press Ctrl+0
**Then** the graph fits all nodes in the viewport

**Covers:** FR61, FR62, NFR34

## Tasks / Subtasks

- [ ] Implement node selection with impact highlighting (AC: 1)
  - [ ] Add `selectedNodeId` to graph store
  - [ ] On single-click: set selected node, compute downstream (BFS forward) and upstream (BFS backward) chains
  - [ ] Apply CSS classes: `impact-downstream` (red `#e94560`), `impact-upstream` (blue `#5c7cfa`), `dimmed` (opacity 0.2) to non-chain nodes/edges
  - [ ] Highlight edges in chain with matching colors (downstream edges red, upstream edges blue)
  - [ ] Click on canvas background to deselect and restore normal state
- [ ] Implement search functionality (AC: 2, 3)
  - [ ] Create `SearchFilter` component with text input
  - [ ] Bind Ctrl+K to focus the search input
  - [ ] Filter logic: match against node title, ID, and tags (case-insensitive substring match)
  - [ ] Non-matching nodes receive `dimmed` class
  - [ ] Matching nodes remain full opacity
  - [ ] Viewport pans/zooms to fit all matching nodes using Svelte Flow's `fitView` with node IDs filter
  - [ ] Escape key dismisses search: clear filter, restore all nodes to normal opacity, keep viewport position
- [ ] Implement keyboard shortcuts (AC: 5)
  - [ ] Ctrl+0: call `fitView()` to fit all nodes in viewport
  - [ ] Escape: dismiss search OR close detail panel (Story 11.4) — priority: search first
  - [ ] Register keyboard listeners with proper event delegation (avoid conflicts with input fields)
- [ ] Performance optimization (AC: 4)
  - [ ] BFS traversal for impact chains should be computed client-side from the graph data (already loaded)
  - [ ] Use Svelte reactivity: derive highlighted state from `selectedNodeId` reactively
  - [ ] Verify all interactions respond in < 100ms (no re-fetches, purely client-side state changes)
- [ ] Add tests
  - [ ] Component test: clicking node highlights downstream in red, upstream in blue
  - [ ] Component test: non-chain nodes are dimmed
  - [ ] Component test: search filters nodes by title/ID/tag
  - [ ] Component test: Escape dismisses search
  - [ ] Component test: Ctrl+0 fits view

## Dev Notes

- This story depends on Story 11.2 (graph rendering must work with SpecNode/CausalEdge components).
- Impact chain computation is client-side BFS on the graph data already in the store. No API calls needed for highlighting.
- The `dimmed` class should use CSS transitions for smooth visual feedback.
- Interaction model from UX spec: single-click = select, double-click = edit (Story 12.3).

### Project Structure Notes

- New components: `web-ui/src/lib/components/SearchFilter.svelte`
- Modified: `web-ui/src/lib/stores/graph.ts` (add selection/search state)
- Modified: `web-ui/src/lib/components/SpecNode.svelte` (conditional styling for highlight/dim)
- Modified: `web-ui/src/lib/components/CausalEdge.svelte` (conditional styling for highlight/dim)
- New utility: `web-ui/src/lib/utils/traversal.ts` (client-side BFS for impact chains)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 11.3]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Interaction Model]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
