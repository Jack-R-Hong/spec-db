# Story 1.4: Causal Graph Traversal (trace_impact & find_dependencies)

Status: ready-for-dev

## Story

As an AI agent,
I want to trace the downstream impact of a spec and discover its upstream dependencies with configurable depth,
so that I understand the blast radius before proposing changes and know what a spec relies on.

## Acceptance Criteria (BDD)

**Given** spec A depends_on spec B, and spec C depends_on spec A  
**When** I call `trace_impact(B)`  
**Then** I receive A and C as downstream impacts (everything that transitively depends on B) (FR6)

**Given** spec A depends_on spec B, and spec B depends_on spec D  
**When** I call `find_dependencies(A)`  
**Then** I receive B and D as upstream dependencies (everything A transitively depends on) (FR7)

**Given** a deep causal chain (A→B→C→D→E)  
**When** I call `trace_impact(E, depth=2)`  
**Then** I receive only nodes within 2 hops (D, C) — not the full chain (FR8)  
**And** when I call `trace_impact(E)` without depth limit, I receive the complete chain

**Given** a graph with 100+ specs  
**When** I call `trace_impact` or `find_dependencies`  
**Then** the traversal completes in < 50ms (NFR2)

**Given** a spec ID that does not exist in the graph  
**When** I call `trace_impact` or `find_dependencies`  
**Then** I receive a clear `GraphError` indicating the spec was not found

## Tasks / Subtasks

- [ ] Lock traversal contracts against Story 1.1 and Story 1.3 APIs (AC: all)
  - [ ] Confirm `CausalGraph` trait signatures include depth-optional traversal methods
  - [ ] Confirm engine stores both outbound and inbound adjacency indexes
- [ ] Implement traversal module and method entry points (AC: 1, 2, 3, 5)
  - [ ] Add `crates/spec-db-causal/src/traversal.rs`
  - [ ] Implement `pub fn trace_impact(&self, start: &SpecId, depth: Option<usize>) -> Result<Vec<SpecId>, SpecDbError>`
  - [ ] Implement `pub fn find_dependencies(&self, start: &SpecId, depth: Option<usize>) -> Result<Vec<SpecId>, SpecDbError>`
  - [ ] Wire methods into `CausalEngine` and `CausalGraph` trait impl
- [ ] Implement graph-walk algorithm with explicit directionality (AC: 1, 2)
  - [ ] Use BFS as default traversal algorithm to support predictable hop-based depth limits
  - [ ] For `trace_impact`, traverse inbound adjacency (nodes depending on current node)
  - [ ] For `find_dependencies`, traverse outbound adjacency (nodes current node depends on)
  - [ ] Maintain `visited: HashSet<SpecId>` to prevent cycles and duplicates
- [ ] Implement depth-limiting semantics exactly (AC: 3)
  - [ ] Track level per queue entry: `(SpecId, depth_from_start)`
  - [ ] If `depth` is `Some(limit)`, enqueue neighbors only when `current_depth < limit`
  - [ ] Return nodes discovered within hop limit, excluding start node unless explicitly required by API contract
  - [ ] Add deterministic ordering strategy (stable insertion or sorted output) and document it in trait docs
- [ ] Implement not-found and error propagation behavior (AC: 5)
  - [ ] Validate start node exists before traversal begins
  - [ ] Return `SpecDbError::GraphError(format!("Spec not found: {id}"))` on missing node
  - [ ] Do not return empty set for missing node; missing-node is a typed error
- [ ] Optimize for NFR2 (<50ms for 100+ specs) (AC: 4)
  - [ ] Avoid full graph scans; read neighbors from prebuilt adjacency maps
  - [ ] Pre-size visited and queue structures where possible
  - [ ] Avoid allocations in tight loop (reuse buffers when feasible)
  - [ ] Add micro-benchmark-style integration test in `tests/integration.rs` with generated 100-500 node graph
- [ ] Add comprehensive traversal tests (AC: 1, 2, 3, 4, 5)
  - [ ] Scenario test for downstream chain (B -> A -> C for `trace_impact(B)`)
  - [ ] Scenario test for upstream chain (`find_dependencies(A)` returns B then D)
  - [ ] Depth=2 chain test with exact expected set
  - [ ] No-depth full transitive closure test
  - [ ] Missing-node error test asserts `GraphError` message and variant

## Dev Notes

- Story dependency chain is strict: Story 1.4 depends on Story 1.3 in-memory graph + adjacency indexes, which depends on Story 1.2 persistence and Story 1.1 contracts.
- Directionality rule from architecture is non-negotiable: `A -> B` means `A depends_on B`; therefore `trace_impact` walks reverse/inbound edges and `find_dependencies` walks forward/outbound edges.
- Algorithm choice for this story: BFS (not DFS) because hop-based depth limiting (`depth=2`) is naturally level-aware and easier to keep deterministic.
- Performance requirement (`<50ms`) is feasible only with in-memory adjacency maps; do not query Fjall during traversal paths.
- Pattern compliance: `N5` tracing spans (`spec_db.graph.traverse`), `S3` trait boundary, `P1` explicit typed failures, and anti-pattern bans (`unwrap` in libs, wildcard re-exports, deep module nesting).
- Add instrumentation spans and fields for operation timing (`operation=trace_impact|find_dependencies`, `start_id`, `depth_limit`, `result_count`) to support later observability story without API churn.

### Project Structure Notes

- Files to create/modify for this story:
- `crates/spec-db-causal/src/traversal.rs`
- `crates/spec-db-causal/src/engine.rs` (delegate traversal calls or host shared adjacency accessors)
- `crates/spec-db-causal/src/lib.rs` (public exports and trait impl wiring)
- `crates/spec-db-causal/tests/integration.rs`
- Keep traversal logic isolated from storage logic; traversal consumes in-memory structures only.
- If fallback backend (petgraph) exists from Story 1.3, enforce traversal behavior parity via shared test cases.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.4]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/project-context.md#Critical Architectural Decisions]
- [Source: docs/project-context.md#Key Patterns for AI Agents]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story defines explicit BFS traversal strategy and depth-limit semantics.
- Not-found behavior and performance constraints are specified as implementation guardrails.
- Cross-story dependencies to 1.1/1.2/1.3 are explicitly captured.

### Change Log

- Initial draft.

### File List

- `_bmad-output/implementation-artifacts/1-4-causal-graph-traversal.md`
