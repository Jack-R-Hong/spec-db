# Story 12.2: Drag-to-Connect Edge Creation & Delete Key Removal

Status: ready-for-dev

## Story

As a spec author,
I want to add edges by dragging from one node's connection handle to another, and remove edges by selecting and pressing Delete,
so that I can visually edit the dependency graph without touching markdown files.

## Acceptance Criteria (BDD)

**Given** I drag from a source node's connection handle to a target node
**When** I release on the target node
**Then** a confirmation toast appears; on confirm, a `depends_on` edge is created between the two specs via the write-back pipeline (FR67)

**Given** I drag from a node's connection handle but release on empty canvas (no target)
**When** I release
**Then** no edge is created and no toast appears

**Given** I drag to create an edge that already exists (same source, target, edge_type)
**When** the system validates
**Then** it shows an error toast "Edge already exists" and does not trigger write-back

**Given** I select an existing edge on the canvas
**When** I press the Delete key
**Then** a confirmation toast appears; on confirm, the edge is removed from the spec's `depends_on` frontmatter via write-back pipeline (FR68)

**Given** no edge is selected
**When** I press the Delete key
**Then** nothing happens

**Given** the interaction model
**When** I hover over a node's connection handle
**Then** it shows a visual affordance (highlight/grow) indicating it's draggable

**Covers:** FR67, FR68

## Tasks / Subtasks

- [ ] Implement drag-to-connect edge creation (AC: 1, 2, 3)
  - [ ] Enable Svelte Flow's `onConnect` event handler
  - [ ] On connection attempt: validate source != target
  - [ ] Check for duplicate edge (same source, target, edge_type=depends_on)
  - [ ] If duplicate: show error toast "Edge already exists"
  - [ ] If valid: show confirmation toast via ToastNotification component
  - [ ] On confirm: call `POST /api/writeback` with edge add operation
  - [ ] On cancel: remove the optimistic edge from the graph
  - [ ] On drop to empty canvas: Svelte Flow handles this natively (no connection event fires)
- [ ] Implement connection handle affordances (AC: 6)
  - [ ] Style connection handles with hover effect: grow + highlight on mouseover
  - [ ] Use CSS transitions for smooth feedback
  - [ ] Handles visible on hover over node, hidden otherwise (reduce visual noise)
- [ ] Implement edge selection and Delete key removal (AC: 4, 5)
  - [ ] Enable edge selection in Svelte Flow (edges selectable)
  - [ ] Track `selectedEdgeId` in graph store
  - [ ] On Delete keypress: check if an edge is selected
  - [ ] If edge selected: show confirmation toast
  - [ ] On confirm: call `POST /api/writeback` with edge remove operation
  - [ ] On cancel: deselect edge, dismiss toast
  - [ ] If no edge selected: do nothing
  - [ ] Prevent Delete key from triggering when input fields are focused
- [ ] Optimistic UI updates (AC: 1, 4)
  - [ ] On confirm, immediately show the new edge / remove the edge visually
  - [ ] If write-back fails: revert visual change, show error toast
  - [ ] If write-back succeeds: graph data refreshes from API to confirm server state
- [ ] Add tests
  - [ ] Component test: dragging handle to target node shows confirmation toast
  - [ ] Component test: dropping on canvas shows nothing
  - [ ] Component test: duplicate edge shows error toast
  - [ ] Component test: selecting edge + Delete shows confirmation toast
  - [ ] Component test: no selection + Delete does nothing
  - [ ] Component test: handle shows hover affordance
  - [ ] Integration test: edge creation via drag → write-back → graph refresh

## Dev Notes

- This story depends on Story 12.1 (write-back pipeline and toast component must exist).
- Svelte Flow provides `onConnect` callback for new connections and edge selection state. Use these native APIs.
- All new edges created via drag are `depends_on` type. Other edge types (`constrains`, `implements`) are only created via the `add_causal_link` MCP tool.
- Optimistic UI pattern: update the visual graph immediately, then confirm with the server. Revert on failure.
- Edge selection: Svelte Flow supports `edgesSelectable` prop and `onEdgeClick` events.

### Project Structure Notes

- Modified: `web-ui/src/routes/+page.svelte` (add onConnect handler, Delete key listener)
- Modified: `web-ui/src/lib/stores/graph.ts` (add selectedEdgeId, optimistic update helpers)
- Modified: `web-ui/src/lib/components/SpecNode.svelte` (handle hover styling)
- Reuses: `web-ui/src/lib/components/ToastNotification.svelte` from Story 12.1

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 12.2]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Interaction Model]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
