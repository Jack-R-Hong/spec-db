---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
status: complete
inputDocuments:
  - prd.md
  - architecture.md
  - ux-design-specification.md
---

# lattice - Phase 2 Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for lattice Phase 2, decomposing the Phase 2 requirements from the PRD ("Self-Growing Intelligence"), the Architecture Web UI Extension, and the UX Design Specification into implementable stories.

## Requirements Inventory

### Functional Requirements

**Self-Growing Intelligence (5):**

FR47: Agents can propose new causal edges between existing specs via an `add_causal_link` MCP tool, specifying source spec, target spec, and edge type
FR48: AI-proposed causal edges are assigned a configurable trust score (default: 0.5) lower than human-curated edges (1.0)
FR49: The system validates AI-proposed edges using DeepCausality's Causal State Machine (CSM) before accepting them into the graph
FR50: AI-proposed edges that fail CSM validation are rejected with a clear explanation of why the causal relationship is invalid
FR51: Graph traversal results distinguish between human-curated and AI-inferred edges, showing trust scores in edge metadata

**Edge Lifecycle & Human Review (4):**

FR52: The system exports all AI-inferred edges to `.lattice/edges.yaml` for human review, including source, target, trust score, origin, and creation timestamp
FR53: Human reviewers can promote AI-inferred edges to human-curated status (trust=1.0) via CLI command or MCP tool
FR54: Human reviewers can reject AI-inferred edges, removing them from the graph and the export file
FR55: The system supports additional edge types beyond `depends_on`: `constrains` and `implements`

**MCP Prompts (2):**

FR56: The system provides an `impact_analysis` MCP Prompt that guides agents through structured impact assessment before proposing spec changes
FR57: The system provides a `spec_review` MCP Prompt that guides agents through structured spec review with checklist

**Web UI - Graph Visualization (6):**

FR58: The lattice binary serves a web-based causal graph visualization via HTTP on a configurable port (default: 3000)
FR59: The web UI frontend is compiled as static assets and embedded in the Rust binary via `rust-embed` (zero separate deployment)
FR60: The web UI renders the full causal graph using Svelte Flow with force-directed layout (dagre/elkjs)
FR61: Users can click a spec node to highlight its downstream impact chain in red and upstream dependencies in blue
FR62: Users can search specs by name, ID, or tag — non-matching nodes dim and the graph centers on matches
FR63: Disconnected specs (no causal edges) are rendered at reduced opacity and visually separated from connected clusters

**Web UI - Inspection (3):**

FR64: Users can view spec details (frontmatter fields, markdown body preview, inbound/outbound edges) in a 360px slide-out detail panel
FR65: The detail panel shows downstream impact as a text list alongside the visual graph highlights
FR66: The web UI header displays real-time sync status: spec count, last sync git SHA, and relative timestamp

**Web UI - Editing & Write-Back (6):**

FR67: Users can add `depends_on` edges by dragging from a source node's connection handle to a target node
FR68: Users can remove `depends_on` edges by selecting an edge and pressing Delete
FR69: Users can edit spec frontmatter fields (title, tags, owner, depends_on) in the detail panel's edit mode
FR70: All UI edits modify the spec markdown file's YAML frontmatter and create a git commit via the write-back pipeline
FR71: Users can undo the last git write-back within a 5-second window
FR72: The web UI displays a confirmation toast before every git write-back action

**Web UI - Administration (3):**

FR73: Users can trigger a full rebuild or incremental sync from the web UI header
FR74: The web UI displays a warning indicator when cross-store drift is detected
FR75: The `lattice serve` command starts both MCP (stdio) and web UI (HTTP) concurrently from a single process

### NonFunctional Requirements

**Performance (5):**

NFR32: CSM edge validation completes in < 100ms per proposed edge
NFR33: Web UI graph renders in < 1 second for 100+ spec nodes with edges
NFR34: Web UI interactions (pan, zoom, select, search) respond in < 100ms
NFR35: Git write-back round-trip (UI edit → file modify → git commit → re-sync → graph refresh) completes in < 2 seconds
NFR36: Undo operation (git revert → re-sync → graph refresh) completes in < 2 seconds

**Security (2):**

NFR37: Web UI binds to localhost (127.0.0.1) by default — no remote network access unless explicitly configured
NFR38: When `web.host` is set to `0.0.0.0`, bearer token authentication from `http.auth_token` config is enforced via middleware

**Accessibility & Compatibility (3):**

NFR39: Web UI meets WCAG 2.1 Level AA accessibility standards (4.5:1 contrast, keyboard navigation, ARIA labels, screen reader support)
NFR40: Web UI is desktop-only — minimum viewport 1024px × 768px
NFR41: Web UI works in Chrome, Firefox, Safari, Edge (latest versions)

### Additional Requirements

- **New crate `spec-db-web`:** Source files: `lib.rs`, `api.rs`, `assets.rs`, `state.rs`, `writeback.rs`. Follows all existing architecture patterns (S1-S5, N1-N6, F1-F4, P1-P3).
- **Frontend stack:** Svelte 5 + @xyflow/svelte (Svelte Flow) + SvelteKit + Vite 6.x. Source in `web-ui/` directory at project root (not inside `crates/`).
- **Static asset embedding:** `rust-embed` 8.x compiles Vite build output into binary. Debug builds read from filesystem for hot reload.
- **REST API format:** Same error shape as MCP tools: `{error_type, message, context}`. Success: `{data: {...}}`.
- **Write-back pipeline lives in `crates/web/src/writeback.rs`** — not in `ingest` crate. Write-back is web-UI-only; `ingest` handles git→index (read), write-back handles index→git (write).
- **Shared AppState:** `Arc<AppState>` with `Mutex<Option<UndoState>>` for serialized write operations. Read operations are concurrent.
- **Configuration extension:** New `web` section in `.lattice/config.yaml`: `web.enabled` (default: true), `web.host` (default: 127.0.0.1), `web.port` (default: 3000).
- **Tracing spans:** `spec_db.web.api.{endpoint}`, `spec_db.web.writeback.apply`.
- **New workspace dependencies:** `axum 0.8` (already in workspace), `rust-embed 8.x`, `tower-http 0.6`.
- **Svelte Flow components:** Custom SpecNode (title + ID + tags + version + handles), Custom CausalEdge (bezier + arrow + label), DetailPanel (view/edit modes), HeaderBar (40px, search, status, rebuild), ToastNotification (bottom-center, auto-dismiss), SearchFilter (Ctrl+K).
- **Dark theme:** Navy base `#1a1a2e`, impact red `#e94560`, upstream blue `#5c7cfa`, CSS custom properties.
- **Interaction model:** Single click=select, Double-click=edit, Hover=show affordances, Drag handle=connect, Delete key=remove edge.
- **Keyboard shortcuts:** Ctrl+K (search), Escape (dismiss/cancel), Delete (remove edge), Ctrl+Z (undo), Ctrl+0 (fit graph).
- **DeepCausality CSM integration:** `CausaloidGraph` already in engine.rs; CSM validation adds `Causaloid::verify_single_cause` or equivalent to validate proposed edges against graph structure.
- **Edge type expansion:** `CausalEdge` in `spec-db-core/types.rs` needs an `edge_type` field (enum: `DependsOn`, `Constrains`, `Implements`).
- **Trust score visibility:** All MCP tool responses and REST API responses that include edges must show `trust` and `origin` fields.

### FR Coverage Map

FR47: Epic 8 - `add_causal_link` MCP tool for AI edge proposals
FR48: Epic 8 - AI edge trust scoring (default 0.5)
FR49: Epic 8 - CSM validation for AI-proposed edges
FR50: Epic 8 - Rejection with explanation on CSM failure
FR51: Epic 8 - Trust scores visible in traversal results
FR55: Epic 8 - New edge types: `constrains`, `implements`
FR52: Epic 9 - Edge export to `.lattice/edges.yaml`
FR53: Epic 9 - Promote AI edges to human-curated
FR54: Epic 9 - Reject AI edges
FR56: Epic 10 - `impact_analysis` MCP Prompt
FR57: Epic 10 - `spec_review` MCP Prompt
FR58: Epic 11 - Web UI served via HTTP
FR59: Epic 11 - Static assets embedded in binary
FR60: Epic 11 - Svelte Flow graph rendering with force-directed layout
FR61: Epic 11 - Impact chain highlighting (red downstream, blue upstream)
FR62: Epic 11 - Search-to-focus with node dimming
FR63: Epic 11 - Disconnected cluster visualization
FR64: Epic 11 - Slide-out detail panel (360px)
FR65: Epic 11 - Impact text list in detail panel
FR66: Epic 11 - Sync status display in header
FR73: Epic 11 - Rebuild trigger from web UI
FR74: Epic 11 - Drift warning indicator
FR75: Epic 11 - Concurrent MCP + HTTP serve
FR67: Epic 12 - Drag-to-connect edge creation
FR68: Epic 12 - Delete key edge removal
FR69: Epic 12 - Frontmatter field editing in detail panel
FR70: Epic 12 - Git write-back pipeline
FR71: Epic 12 - 5-second undo window
FR72: Epic 12 - Confirmation toast before git write-back

## Epic List

### Epic 8: AI-Inferred Causal Links & Trust Scoring
Agents can autonomously propose new causal edges between specs. The system validates proposals using DeepCausality's Causal State Machine, assigns trust scores to differentiate human and AI knowledge, and supports richer edge types (`constrains`, `implements`) beyond `depends_on`.
**FRs covered:** FR47, FR48, FR49, FR50, FR51, FR55

### Epic 9: Human Review & Edge Curation
AI-inferred edges are exported to `.lattice/edges.yaml` for human review. Spec authors and architects can promote good AI edges to human-curated status (trust=1.0) or reject bad ones. Full audit trail of all AI contributions.
**FRs covered:** FR52, FR53, FR54

### Epic 10: MCP Prompts for Structured Agent Workflows
The system provides MCP Prompts (`impact_analysis`, `spec_review`) that guide agents through structured analysis workflows, producing consistent, auditable outputs that spec authors can trust.
**FRs covered:** FR56, FR57

### Epic 11: Causal Graph Web UI — Visualization & Exploration
The lattice binary serves a web-based causal graph visualization. Spec authors and architects can see the full architecture at a glance, click nodes to trace impact visually, search by name/tag, inspect spec details in a slide-out panel, trigger rebuilds, and monitor sync status — all from a browser.
**FRs covered:** FR58, FR59, FR60, FR61, FR62, FR63, FR64, FR65, FR66, FR73, FR74, FR75

### Epic 12: On-Canvas Graph Editing & Git Write-Back
Users can add/remove `depends_on` edges by dragging on the graph canvas, edit spec frontmatter fields in the detail panel, and have all changes automatically written back to markdown files via git commit. A 5-second undo window provides safety for accidental changes.
**FRs covered:** FR67, FR68, FR69, FR70, FR71, FR72

## Epic 8: AI-Inferred Causal Links & Trust Scoring

Agents can autonomously propose new causal edges between specs. The system validates proposals using DeepCausality's Causal State Machine, assigns trust scores to differentiate human and AI knowledge, and supports richer edge types (`constrains`, `implements`) beyond `depends_on`.

### Story 8.1: Edge Type Expansion & Trust Score Visibility

As a **spec author**,
I want the causal graph to support `constrains` and `implements` edge types alongside `depends_on`, with every edge displaying its trust score and origin,
So that I can model richer architectural relationships and distinguish human-curated from AI-inferred knowledge.

**Acceptance Criteria:**

**Given** the `CausalEdge` type in `spec-db-core/types.rs`
**When** I add an `edge_type` field with variants `DependsOn`, `Constrains`, `Implements`
**Then** all existing edges default to `DependsOn` and the system compiles without breaking existing tests

**Given** the `CausalEdge` struct now has `edge_type`, `trust` (f64), and `origin` (enum: `Human`, `AiInferred`) fields
**When** I call `trace_impact` or `find_dependencies` via MCP tools
**Then** the response JSON includes `edge_type`, `trust`, and `origin` for every edge in the result

**Given** a spec with `depends_on: ["spec::auth::token-format"]` in its frontmatter
**When** the ingestion pipeline parses the spec
**Then** the resulting `CausalEdge` has `edge_type: DependsOn`, `trust: 1.0`, and `origin: Human`

**Given** the updated `CausalEdge` model
**When** I serialize/deserialize edges to/from Fjall KV store
**Then** the new fields round-trip correctly and existing stored edges migrate gracefully

**Covers:** FR55, FR51

### Story 8.2: AI Causal Link Proposal via MCP Tool

As an **AI agent**,
I want to propose new causal edges between existing specs via an `add_causal_link` MCP tool,
So that I can grow the causal knowledge graph as I discover relationships during analysis.

**Acceptance Criteria:**

**Given** two existing specs `spec::auth::jwt-validation` and `spec::auth::token-format`
**When** I call `add_causal_link(source: "spec::auth::jwt-validation", target: "spec::auth::token-format", edge_type: "depends_on")`
**Then** a new `CausalEdge` is created with `trust: 0.5` (configurable default), `origin: AiInferred`, and the specified `edge_type`

**Given** a call to `add_causal_link` with a `source` ID that does not exist in the graph
**When** the tool processes the request
**Then** it returns an error: `{ error_type: "not_found", message: "Source spec not found", context: { id: "..." } }`

**Given** a call to `add_causal_link` with `source` equal to `target` (self-referencing)
**When** the tool processes the request
**Then** it returns an error: `{ error_type: "validation_error", message: "Self-referencing edges are not allowed" }`

**Given** a duplicate edge proposal (same source, target, and edge_type already exists)
**When** the tool processes the request
**Then** it returns an error: `{ error_type: "conflict", message: "Edge already exists" }`

**Given** the default trust score is `0.5`
**When** `.lattice/config.yaml` sets `ai.default_trust: 0.7`
**Then** new AI-proposed edges use `0.7` as their trust score

**Covers:** FR47, FR48

### Story 8.3: DeepCausality CSM Validation for AI-Proposed Edges

As a **system operator**,
I want all AI-proposed edges validated against DeepCausality's Causal State Machine before acceptance,
So that only structurally valid causal relationships enter the graph.

**Acceptance Criteria:**

**Given** an AI agent calls `add_causal_link` with a valid source, target, and edge_type
**When** the system processes the proposal
**Then** it runs CSM validation on the proposed edge before inserting it into the graph

**Given** a proposed edge that would create a cycle in the causal graph (A→B→C→A)
**When** CSM validation detects the cycle
**Then** the edge is rejected with `{ error_type: "csm_validation_failed", message: "Proposed edge creates a causal cycle", context: { cycle: ["A", "B", "C", "A"] } }`

**Given** a proposed edge that passes CSM validation
**When** the edge is accepted
**Then** it is inserted into the `CausaloidGraph` and persisted to Fjall KV with `origin: AiInferred` and the configured trust score

**Given** CSM validation processing
**When** the validation runs on a single proposed edge
**Then** it completes in under 100ms (NFR32)

**Given** a proposed edge between specs in disconnected subgraphs
**When** CSM validation runs
**Then** the edge passes validation (connecting disconnected components is valid)

**Covers:** FR49, FR50, NFR32

## Epic 9: Human Review & Edge Curation

AI-inferred edges are exported to `.lattice/edges.yaml` for human review. Spec authors and architects can promote good AI edges to human-curated status (trust=1.0) or reject bad ones. Full audit trail of all AI contributions.

### Story 9.1: AI-Inferred Edge Export to YAML

As a **spec author**,
I want all AI-inferred edges automatically exported to `.lattice/edges.yaml`,
So that I can review AI contributions outside the running system using my preferred text editor or CI pipeline.

**Acceptance Criteria:**

**Given** one or more AI-inferred edges exist in the causal graph (origin: `AiInferred`)
**When** the system writes to `.lattice/edges.yaml`
**Then** the file contains an entry for each AI-inferred edge with fields: `source`, `target`, `edge_type`, `trust`, `origin`, and `created_at` (ISO 8601 timestamp)

**Given** a new AI edge is accepted via `add_causal_link`
**When** the edge passes CSM validation and is persisted
**Then** `.lattice/edges.yaml` is updated to include the new edge within the same operation

**Given** `.lattice/edges.yaml` already contains edges
**When** a new AI edge is added
**Then** the file is rewritten atomically (write-to-temp + rename) to avoid partial writes

**Given** no AI-inferred edges exist
**When** the system checks `.lattice/edges.yaml`
**Then** the file either does not exist or contains an empty list `edges: []`

**Given** human-curated edges (origin: `Human`, trust: 1.0) in the graph
**When** the system exports to `.lattice/edges.yaml`
**Then** human-curated edges are **not** included — only `AiInferred` edges appear

**Covers:** FR52

### Story 9.2: Promote & Reject AI-Inferred Edges

As a **spec author**,
I want to promote AI-inferred edges to human-curated status or reject them entirely, via CLI command or MCP tool,
So that I maintain authority over which causal relationships are trusted in the graph.

**Acceptance Criteria:**

**Given** an AI-inferred edge from `spec::auth::jwt` to `spec::auth::tokens` exists in the graph
**When** I call `promote_edge(source: "spec::auth::jwt", target: "spec::auth::tokens", edge_type: "depends_on")` via MCP tool
**Then** the edge's `origin` changes to `Human`, `trust` becomes `1.0`, and it is removed from `.lattice/edges.yaml`

**Given** an AI-inferred edge exists in the graph
**When** I call `reject_edge(source: "spec::auth::jwt", target: "spec::auth::tokens", edge_type: "depends_on")` via MCP tool
**Then** the edge is removed from the `CausaloidGraph`, deleted from Fjall KV, and removed from `.lattice/edges.yaml`

**Given** I call `promote_edge` or `reject_edge` with an edge that does not exist
**When** the tool processes the request
**Then** it returns `{ error_type: "not_found", message: "Edge not found", context: { source: "...", target: "...", edge_type: "..." } }`

**Given** I call `promote_edge` on an edge that is already human-curated (origin: `Human`)
**When** the tool processes the request
**Then** it returns `{ error_type: "validation_error", message: "Edge is already human-curated" }`

**Given** the CLI exposes `lattice edge promote <source> <target> [--type depends_on]` and `lattice edge reject <source> <target> [--type depends_on]`
**When** I run either command
**Then** it performs the same operation as the corresponding MCP tool and prints a confirmation message

**Covers:** FR53, FR54

## Epic 10: MCP Prompts for Structured Agent Workflows

The system provides MCP Prompts (`impact_analysis`, `spec_review`) that guide agents through structured analysis workflows, producing consistent, auditable outputs that spec authors can trust.

### Story 10.1: Impact Analysis MCP Prompt

As an **AI agent**,
I want an `impact_analysis` MCP Prompt that guides me through structured impact assessment before proposing spec changes,
So that my analysis is systematic, auditable, and consistent regardless of which agent performs it.

**Acceptance Criteria:**

**Given** an MCP client lists available prompts
**When** it calls `prompts/list`
**Then** the response includes `impact_analysis` with a description and its required arguments (e.g., `spec_id: string`)

**Given** an agent calls `prompts/get` for `impact_analysis` with `spec_id: "spec::auth::jwt-validation"`
**When** the prompt is resolved
**Then** it returns a structured message sequence that includes:
1. The spec's current content and metadata
2. Its direct downstream dependents (from `trace_impact`)
3. Its direct upstream dependencies (from `find_dependencies`)
4. A structured template asking the agent to assess: scope of change, affected specs, risk level, and recommended actions

**Given** the target `spec_id` does not exist
**When** the prompt is resolved
**Then** it returns an error: `{ error_type: "not_found", message: "Spec not found", context: { id: "..." } }`

**Given** the spec has no causal edges (isolated node)
**When** the prompt is resolved
**Then** it returns the spec content with empty dependency/impact lists and notes "No causal relationships found — impact is isolated"

**Covers:** FR56

### Story 10.2: Spec Review MCP Prompt

As an **AI agent**,
I want a `spec_review` MCP Prompt that guides me through a structured spec review with a checklist,
So that my reviews are thorough, consistent, and cover all quality dimensions.

**Acceptance Criteria:**

**Given** an MCP client lists available prompts
**When** it calls `prompts/list`
**Then** the response includes `spec_review` with a description and its required arguments (e.g., `spec_id: string`)

**Given** an agent calls `prompts/get` for `spec_review` with `spec_id: "spec::auth::jwt-validation"`
**When** the prompt is resolved
**Then** it returns a structured message sequence that includes:
1. The spec's full content (frontmatter + body)
2. Its causal graph context (edges in/out, trust scores, edge types)
3. A review checklist covering: completeness (required frontmatter fields present), clarity (title, body coherence), dependency accuracy (do declared `depends_on` specs exist?), and consistency (tags, version, dates)

**Given** the target `spec_id` does not exist
**When** the prompt is resolved
**Then** it returns an error: `{ error_type: "not_found", message: "Spec not found", context: { id: "..." } }`

**Given** a spec has `depends_on` references to specs that don't exist in the graph
**When** the review prompt is resolved
**Then** the checklist flags these as "broken dependency references" with the missing spec IDs listed

**Covers:** FR57

## Epic 11: Causal Graph Web UI — Visualization & Exploration

The lattice binary serves a web-based causal graph visualization. Spec authors and architects can see the full architecture at a glance, click nodes to trace impact visually, search by name/tag, inspect spec details in a slide-out panel, trigger rebuilds, and monitor sync status — all from a browser.

### Story 11.1: Web Server Scaffold & Static Asset Embedding

As a **spec author**,
I want `lattice serve` to start both the MCP server (stdio) and an HTTP web UI server concurrently from a single process,
So that I can access the graph visualization in my browser without running a separate service.

**Acceptance Criteria:**

**Given** I run `lattice serve`
**When** the process starts
**Then** it binds an HTTP server on `127.0.0.1:3000` (default) concurrently with the existing MCP stdio server

**Given** `.lattice/config.yaml` contains `web.port: 4000` and `web.host: "127.0.0.1"`
**When** I run `lattice serve`
**Then** the HTTP server binds on `127.0.0.1:4000`

**Given** the web UI frontend has been compiled to static assets by Vite
**When** the Rust binary is built in release mode
**Then** `rust-embed` embeds the static assets into the binary — no external files needed at runtime

**Given** the Rust binary is built in debug mode
**When** the HTTP server serves assets
**Then** it reads from the filesystem (`web-ui/dist/`) for hot-reload during development

**Given** `web.host` is set to `0.0.0.0` and `http.auth_token` is configured
**When** a request arrives without a valid `Authorization: Bearer <token>` header
**Then** the server returns `401 Unauthorized` (NFR38)

**Given** `web.host` is `127.0.0.1` (default)
**When** a request arrives
**Then** no bearer token authentication is required (NFR37)

**Given** the new `spec-db-web` crate
**When** I inspect its structure
**Then** it contains `lib.rs`, `api.rs`, `assets.rs`, `state.rs` and follows existing architecture patterns (S1-S5, N1-N6, F1-F4, P1-P3)

**Covers:** FR58, FR59, FR75, NFR37, NFR38

### Story 11.2: Svelte Flow Graph Rendering with Force-Directed Layout

As a **spec author**,
I want the web UI to render the full causal graph using Svelte Flow with a force-directed layout,
So that I can see the entire architecture's dependency structure at a glance.

**Acceptance Criteria:**

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

### Story 11.3: Impact Chain Highlighting & Search-to-Focus

As a **spec author**,
I want to click a node to see its impact chain highlighted in color, and search specs to focus the view,
So that I can quickly trace dependencies and find specific specs in a large graph.

**Acceptance Criteria:**

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

### Story 11.4: Detail Panel & Spec Inspection

As a **spec author**,
I want to view a spec's full details in a slide-out panel when I select a node,
So that I can inspect frontmatter, content, and edges without leaving the graph view.

**Acceptance Criteria:**

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

### Story 11.5: Header Bar — Sync Status, Rebuild Trigger & Drift Warning

As a **spec author**,
I want the web UI header to show real-time sync status and let me trigger rebuilds,
So that I always know whether I'm looking at current data and can refresh on demand.

**Acceptance Criteria:**

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

## Epic 12: On-Canvas Graph Editing & Git Write-Back

Users can add/remove `depends_on` edges by dragging on the graph canvas, edit spec frontmatter fields in the detail panel, and have all changes automatically written back to markdown files via git commit. A 5-second undo window provides safety for accidental changes.

### Story 12.1: Git Write-Back Pipeline & Confirmation Toast

As a **spec author**,
I want all UI edits to automatically write back to spec markdown files via git commit, with a confirmation toast before each action,
So that I can trust that my visual edits are persisted to the source of truth without manual file editing.

**Acceptance Criteria:**

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

### Story 12.2: Drag-to-Connect Edge Creation & Delete Key Removal

As a **spec author**,
I want to add edges by dragging from one node's connection handle to another, and remove edges by selecting and pressing Delete,
So that I can visually edit the dependency graph without touching markdown files.

**Acceptance Criteria:**

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

### Story 12.3: Frontmatter Field Editing & Undo

As a **spec author**,
I want to edit spec frontmatter fields (title, tags, owner, depends_on) in the detail panel's edit mode, with a 5-second undo window after each commit,
So that I can make quick metadata corrections visually and recover from mistakes.

**Acceptance Criteria:**

**Given** the detail panel is open for a spec
**When** I double-click the panel (or click an "Edit" button)
**Then** the panel switches to edit mode, showing editable fields for: title, tags, owner, and depends_on (FR69)

**Given** I modify a field in edit mode and click "Save"
**When** the save is triggered
**Then** a confirmation toast appears; on confirm, the write-back pipeline updates the spec file's YAML frontmatter and creates a git commit (FR70)

**Given** a git write-back just completed
**When** I look at the bottom of the screen within 5 seconds
**Then** an "Undo" button is visible; clicking it reverts the git commit (git revert), re-syncs, and refreshes the graph (FR71)

**Given** the undo operation is triggered
**When** the revert runs
**Then** it completes the full round-trip (git revert → re-sync → graph refresh) in under 2 seconds (NFR36)

**Given** more than 5 seconds have passed since the last write-back
**When** I look at the bottom of the screen
**Then** the "Undo" button is no longer visible

**Given** I press Ctrl+Z within the 5-second undo window
**When** the shortcut fires
**Then** it triggers the same undo operation as clicking the "Undo" button

**Given** I am in edit mode
**When** I press Escape
**Then** the panel discards unsaved changes and returns to view mode

**Covers:** FR69, FR71, NFR36
