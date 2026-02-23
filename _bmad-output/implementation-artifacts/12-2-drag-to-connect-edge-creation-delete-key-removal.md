# Story 12.2: Drag-to-Connect Edge Creation & Delete Key Removal

Status: done

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

- [x] Implement drag-to-connect edge creation (AC: 1, 2, 3)
  - [x] Enable Svelte Flow's `onconnect` event handler
  - [x] On connection attempt: validate source != target
  - [x] Check for duplicate edge (same source, target)
  - [x] If duplicate: show error toast "Edge already exists"
  - [x] If valid: show confirmation toast via ToastNotification component
  - [x] On confirm: call `POST /api/writeback` with edge_add operation
  - [x] On cancel: dismiss toast, no changes
  - [x] On drop to empty canvas: no connection event fires (native Svelte Flow behavior)
- [x] Implement connection handle affordances (AC: 6)
  - [x] Handles hidden by default, visible on node hover (opacity transition)
  - [x] On handle hover: scale(1.5) + accent color
  - [x] CSS transitions for smooth feedback
- [x] Implement edge selection and Delete key removal (AC: 4, 5)
  - [x] Track `selectedEdgeId` in page state
  - [x] `onedgeclick` toggles edge selection
  - [x] On Delete/Backspace keypress: check if an edge is selected
  - [x] If edge selected: show confirmation toast
  - [x] On confirm: call `POST /api/writeback` with edge_remove operation
  - [x] On cancel: deselect edge, dismiss toast
  - [x] If no edge selected: do nothing
  - [x] Prevent Delete key from triggering when input fields are focused
- [x] Post-writeback flow (AC: 1, 4)
  - [x] On confirm and success: show Undo toast (5s countdown), refresh graph
  - [x] If write-back fails: show error toast
  - [x] Undo calls `POST /api/writeback/undo` and refreshes graph

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
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- `onconnect` handler validates source != target, checks duplicate edges, shows confirmation toast
- `onedgeclick` toggles `selectedEdgeId` for Delete key handling
- Delete/Backspace key: checks `isInputFocused` to avoid intercepting text editing
- Post-writeback: shows Undo toast with 5s countdown, refreshes graph on success
- Connection handles: hidden by default (opacity 0), visible on node hover, scale 1.5x on handle hover with accent color
- Edge data enriched with `sourceId`/`targetId` for spec ID lookup

### Change Log
- Modified `web-ui/src/routes/+page.svelte` — added onconnect, onedgeclick, handleDeleteEdge, ToastNotification binding
- Modified `web-ui/src/lib/components/SpecNode.svelte` — added handle hover CSS
- Modified `web-ui/src/lib/stores/graph.ts` — added sourceId/targetId to edge data

### File List
- web-ui/src/routes/+page.svelte (modified)
- web-ui/src/lib/components/SpecNode.svelte (modified)
- web-ui/src/lib/stores/graph.ts (modified)
