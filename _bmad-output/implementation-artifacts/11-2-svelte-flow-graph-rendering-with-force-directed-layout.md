# Story 11.2: Svelte Flow Graph Rendering with Force-Directed Layout

Status: ready-for-dev

## Story

As a spec author,
I want the web UI to render the full causal graph using Svelte Flow with a force-directed layout,
so that I can see the entire architecture's dependency structure at a glance.

## Acceptance Criteria (BDD)

**Given** the web UI frontend project in `web-ui/`
**When** I inspect its stack
**Then** it uses Svelte 5, @xyflow/svelte (Svelte Flow), SvelteKit, and Vite 6.x

**Given** the REST API serves graph data at `GET /api/graph`
**When** the web UI loads
**Then** it fetches the full graph and renders it using Svelte Flow with dagre/elkjs force-directed layout

**Given** a graph with 100+ spec nodes and edges
**When** the web UI renders the graph
**Then** it completes initial render in under 1 second (NFR33)

**Given** a custom `SpecNode` component
**When** a node renders
**Then** it displays: spec title, ID, tags (as chips), version, and connection handles (source/target)

**Given** a custom `CausalEdge` component
**When** an edge renders
**Then** it displays: bezier curve with directional arrow and edge type label

**Given** the dark theme configuration
**When** the UI renders
**Then** it uses navy base `#1a1a2e` with CSS custom properties for all theme colors

**Given** specs with no causal edges (disconnected nodes)
**When** the graph renders
**Then** they appear at reduced opacity and are visually separated from connected clusters (FR63)

**Covers:** FR60, FR63, NFR33

## Tasks / Subtasks

- [ ] Set up Svelte Flow with layout engine (AC: 1, 2)
  - [ ] Install `@xyflow/svelte`, `dagre` (or `elkjs`) for layout
  - [ ] Create graph store that fetches from `GET /api/graph` on mount
  - [ ] Transform API response (nodes/edges) into Svelte Flow node/edge format
  - [ ] Apply dagre/elkjs layout algorithm to compute node positions
- [ ] Implement custom `SpecNode` component (AC: 4)
  - [ ] Create `web-ui/src/lib/components/SpecNode.svelte`
  - [ ] Display: spec title (bold), ID (monospace, smaller), tags (colored chips), version badge
  - [ ] Include source and target connection handles (Svelte Flow Handle components)
  - [ ] Style with dark theme: node background slightly lighter than canvas
- [ ] Implement custom `CausalEdge` component (AC: 5)
  - [ ] Create `web-ui/src/lib/components/CausalEdge.svelte`
  - [ ] Render bezier curve with directional arrow marker
  - [ ] Display edge type label on the edge path
  - [ ] Color-code by edge type: `depends_on` (default), `constrains` (yellow), `implements` (green)
- [ ] Implement dark theme with CSS custom properties (AC: 6)
  - [ ] Define CSS custom properties in `:root`: `--color-base: #1a1a2e`, `--color-impact: #e94560`, `--color-upstream: #5c7cfa`, etc.
  - [ ] Apply to all components: canvas background, node cards, edge lines, text colors
  - [ ] Ensure 4.5:1 contrast ratio for text (WCAG 2.1 Level AA)
- [ ] Handle disconnected nodes (AC: 7)
  - [ ] Detect nodes with no edges (degree 0)
  - [ ] Render at reduced opacity (e.g., 0.4)
  - [ ] Position in a separate cluster area (bottom/right) via layout configuration
- [ ] Implement basic graph interactions (AC: 2)
  - [ ] Pan: click-drag on canvas
  - [ ] Zoom: mouse wheel / pinch
  - [ ] Minimap component for orientation in large graphs
- [ ] Performance optimization (AC: 3)
  - [ ] Verify render time < 1s for 100+ nodes (browser dev tools performance tab)
  - [ ] Use Svelte Flow's built-in virtualization for off-screen nodes
- [ ] Add tests
  - [ ] Component test: SpecNode renders title, ID, tags, version
  - [ ] Component test: CausalEdge renders with label
  - [ ] Component test: disconnected nodes have reduced opacity class
  - [ ] E2E test: graph loads from API and renders nodes

## Dev Notes

- This story depends on Story 11.1 (web server scaffold and `GET /api/graph` endpoint must exist).
- Svelte Flow (@xyflow/svelte) is the Svelte port of React Flow. Use `SvelteFlow`, `MiniMap`, `Controls`, `Background` components.
- Layout: dagre is simpler and faster; elkjs is more configurable. Start with dagre. Switch to elkjs only if layout quality is poor.
- Dark theme colors from UX spec: navy base `#1a1a2e`, impact red `#e94560`, upstream blue `#5c7cfa`.
- Desktop-only: minimum viewport 1024x768 (NFR40). No responsive/mobile design needed.

### Project Structure Notes

- All frontend files in `web-ui/src/`:
  - `web-ui/src/lib/components/SpecNode.svelte`
  - `web-ui/src/lib/components/CausalEdge.svelte`
  - `web-ui/src/lib/stores/graph.ts` (graph data store)
  - `web-ui/src/lib/layout/dagre.ts` (layout algorithm wrapper)
  - `web-ui/src/routes/+page.svelte` (main graph page)
  - `web-ui/src/app.css` (global theme variables)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 11.2]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Graph Visualization]
- [Source: https://svelteflow.dev/]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
