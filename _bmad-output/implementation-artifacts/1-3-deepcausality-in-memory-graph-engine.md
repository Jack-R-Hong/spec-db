# Story 1.3: DeepCausality In-Memory Graph Engine

Status: ready-for-dev

## Story

As a developer,
I want specs loaded into a DeepCausality in-memory graph with edges automatically created from `depends_on` fields,
so that causal relationships are traversable in memory with sub-50ms performance.

## Acceptance Criteria (BDD)

**Given** a Fjall store containing persisted spec nodes and causal edges  
**When** the graph engine initializes  
**Then** all nodes and edges are loaded into the DeepCausality in-memory graph  
**And** startup completes in < 1 second for 100+ specs (NFR4)

**Given** a spec with `depends_on: ["spec::auth::token-issuance"]` in its frontmatter  
**When** the spec is added to the graph  
**Then** a causal edge is automatically created from the spec to `spec::auth::token-issuance` (FR9)  
**And** the edge has type `depends_on` and trust level 1.0 (human-curated)

**Given** a spec node in the graph  
**When** I request the node view  
**Then** I receive the node with all inbound edges (specs that depend on it) and all outbound edges (specs it depends on) (FR10)

## Tasks / Subtasks

- [ ] Confirm Story 1.1 and 1.2 contracts before engine work (AC: all)
  - [ ] Verify `SpecNode`, `CausalEdge`, `TrustLevel`, and graph/store traits are stable
  - [ ] Verify store APIs for bulk node/edge load and write-through operations exist
- [ ] Create the in-memory graph engine module (AC: 1, 3)
  - [ ] Add `crates/spec-db-causal/src/engine.rs` with `pub struct CausalEngine`
  - [ ] Include fields for graph structure plus lookup indices by `SpecId` for O(1) node lookup
  - [ ] Add constructor: `pub fn from_store(store: Arc<FjallStore>) -> Result<Self, SpecDbError>`
  - [ ] Keep implementation synchronous; no async in this crate
- [ ] Implement startup loading from persistence (AC: 1)
  - [ ] Add `pub fn load_from_store(&mut self) -> Result<(), SpecDbError>`
  - [ ] Load all nodes first, then all edges to guarantee node existence during edge attach
  - [ ] Build inbound/outbound adjacency indexes while loading
  - [ ] Add startup timing instrumentation with `tracing` span (`spec_db.graph.load`)
  - [ ] Add performance gate test for 100+ synthetic specs `< 1s` using `std::time::Instant`
- [ ] Implement add-node flow with automatic depends_on edge creation (AC: 2)
  - [ ] Add API `pub fn add_spec_node(&mut self, node: SpecNode) -> Result<(), SpecDbError>`
  - [ ] For each `depends_on` entry in node metadata/frontmatter, construct `CausalEdge { from, to, edge_type: "depends_on", trust: 1.0 }`
  - [ ] Validate target `SpecId` format using Story 1.1 constructor; return `GraphError` on invalid IDs
  - [ ] Persist node + generated edges atomically via Story 1.2 `put_node_with_edges`
  - [ ] Insert into in-memory indexes in same operation after persistence success
- [ ] Implement node view API with inbound and outbound edges (AC: 3)
  - [ ] Define a view struct (in `spec-db-core` if needed by trait) containing `node`, `inbound_edges`, `outbound_edges`
  - [ ] Add `pub fn node_view(&self, id: &SpecId) -> Result<NodeView, SpecDbError>`
  - [ ] Resolve inbound set from reverse adjacency and outbound set from forward adjacency
  - [ ] Return `SpecDbError::GraphError` if node does not exist
- [ ] Integrate with trait boundary (AC: all)
  - [ ] Implement `CausalGraph` trait for `CausalEngine` in `engine.rs` or `lib.rs`
  - [ ] Ensure methods used by Story 1.4 traversal are exposed (`trace_impact`, `find_dependencies`, depth parameters)
- [ ] Add deterministic tests and benchmark-style checks (AC: 1, 2, 3)
  - [ ] Unit tests for auto-created depends_on edges with trust `1.0`
  - [ ] Integration tests for restart reload correctness: persisted store -> in-memory graph parity
  - [ ] Node view tests assert complete inbound/outbound edge sets

## Dev Notes

- This is the highest-risk integration in Epic 1 and architecture explicitly flags DeepCausality mapping risk; keep trait boundaries strict so fallback is possible without API churn.
- Required architecture patterns to enforce: `S3` (trait-based boundary), `S4` (domain types in core only), `P1` (clear failure vs graceful degradation), `P3` (sync graph ops below MCP async boundary), plus anti-patterns (`unwrap` ban, no wildcard re-exports).
- Edge direction is canonical: `A -> B` means `A depends_on B`; this direction directly controls traversal semantics in Story 1.4.
- DeepCausality 0.13.4 API surface is broad and not purpose-built for simple spec DAGs; contain it behind a thin adapter layer in `engine.rs` so traversal logic can switch to petgraph if integration friction or performance regressions appear.
- **Riskiest integration + fallback strategy (mandatory):**
- Implement a local `GraphBackend` enum in `engine.rs`: `DeepCausality` primary, `PetgraphFallback` optional feature-gated (`petgraph = "0.6"`)
- Keep public engine contract independent of backend internals (all callers go through `CausalGraph` trait)
- On DeepCausality model-build failure during startup, emit warning span and either: (a) fallback to petgraph backend if feature enabled, or (b) bubble typed `GraphError` to caller for search-only degradation at startup (per `P1`)
- Add parity tests (same nodes/edges -> same traversal results) for DeepCausality and petgraph backends where fallback is enabled
- Performance target coupling: startup `<1s` for 100+ specs and traversal `<50ms` in Story 1.4; preserve adjacency caches to avoid repeated expensive graph scans.

### Project Structure Notes

- Files to create/modify:
- `crates/spec-db-causal/src/engine.rs`
- `crates/spec-db-causal/src/lib.rs` (explicit exports only)
- `crates/spec-db-causal/src/store.rs` (read APIs consumed by engine)
- `crates/spec-db-causal/tests/integration.rs`
- Optional fallback module:
- `crates/spec-db-causal/src/engine_petgraph.rs` (feature-gated adapter implementing same internal backend trait)
- Do not place graph domain structs in causal crate if they are shared contracts; move them to `spec-db-core` first.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3]
- [Source: _bmad-output/planning-artifacts/architecture.md#Technical Constraints & Dependencies]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Error Handling]
- [Source: _bmad-output/planning-artifacts/architecture.md#Decision Impact Analysis]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/architecture.md#Architecture Validation Results]
- [Source: docs/project-context.md#Critical Architectural Decisions]
- [Source: https://docs.rs/deep_causality/latest/deep_causality/]
- [Source: https://crates.io/crates/deep_causality/0.13.0]
- [Source: https://deepcausality.com/getting-started/]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story includes explicit risk-containment strategy for DeepCausality integration.
- Petgraph fallback path defined behind stable trait boundary as required by architecture.
- Story 1.2 persistence dependency and Story 1.4 traversal dependency are explicit.

### Change Log

- Initial draft.

### File List

- `_bmad-output/implementation-artifacts/1-3-deepcausality-in-memory-graph-engine.md`
