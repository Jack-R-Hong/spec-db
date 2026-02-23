# Story 5.2: Hybrid Query Execution & Result Composition

Status: review

## Story

As an AI agent,
I want composed results that combine search hits with causal context, and causal fallback when search returns nothing,
so that I always get the richest possible answer to my question.

## Acceptance Criteria (BDD)

**Given** a hybrid query routed to both engines
**When** both return results
**Then** I receive a composed response combining search results with causal context for each hit (FR13)
**And** search results include their causal edges where applicable

**Given** a search query that returns zero results
**When** the router processes the empty result
**Then** it falls back to the causal graph, traversing from related nodes to provide context (FR12)
**And** the response includes the causal context with an indication that no direct search matches were found

**Given** a causal query (e.g., "what breaks if I change spec::auth::jwt-validation")
**When** the router processes it
**Then** the causal engine result is returned directly without unnecessary search

**Given** a query where both engines return empty results
**When** the router processes it
**Then** I receive an empty result with a clear message — no fabricated results

## Tasks / Subtasks

- [x] Implement fan-out execution orchestration in `crates/router/src/lib.rs`.
  - [x] Add `fn query(&self, natural_language: &str) -> Result<ComposedQueryResult, SpecDbError>` on `QueryRouter<S: SearchEngine, C: CausalGraph>`.
  - [x] Route using `IntentClassifier::classify()` from Story 5.1 and dispatch execution paths: search-only, causal-only, hybrid.
  - [x] Ensure causal-only path bypasses Tantivy calls entirely.
- [x] Implement response composer in `crates/router/src/composer.rs`.
  - [x] Define `ComposedQueryResult` JSON-shape structs with explicit sections: `intent`, `search_results`, `causal_context`, `message`.
  - [x] Define per-hit enrichment structure (e.g., `ComposedHit { id, title, score, snippet, causal_edges }`) so search hits can embed causal edges.
  - [x] Implement `compose_hybrid(search_hits, causal_data)` to merge search results with causal context for each relevant hit.
- [x] Implement zero-result fallback behavior in `crates/router/src/composer.rs` and `crates/router/src/lib.rs`.
  - [x] When search returns empty for search/hybrid flows, derive related node candidates and invoke causal traversal.
  - [x] Return explicit message indicating no direct search matches while including causal context payload.
  - [x] If both search and causal are empty, return deterministic empty response with clear no-results message and no fabricated entries.
- [x] Align router output with MCP response contract expectations.
  - [x] Ensure composed response is JSON-serializable and consistent with architecture format patterns in preparation for `mcp::tools::query()` wiring.
  - [x] Preserve stable field names for downstream MCP clients and tests.
- [x] Add integration and edge-case tests in `crates/router/tests/integration.rs`.
  - [x] Hybrid path test: both engines non-empty -> composed results include search hits + causal edges.
  - [x] Search-zero fallback test: empty search -> causal fallback context included with explanatory message.
  - [x] Causal-only test: causal query executes graph path without search invocation.
  - [x] Both-empty test: clear empty response and no hallucinated data.
  - [x] Contract test: composed payload shape remains stable (`intent`, `search_results`, `causal_context`, `message`).

## Dev Notes

- This story completes FR12 and FR13 behavior in the router crate (`spec-db-router`) within Build Order #6.
- Router remains a convenience API for unified `query()`; explicit `search_specs()` and `trace_impact()` calls are still the primary agent workflow.
- Use `SearchEngine` (Epic 2) and `CausalGraph` (Epic 1) trait abstractions from `spec-db-core`; avoid concrete implementation coupling.
- Keep routing/composition logic synchronous and deterministic; async orchestration stays in MCP layer via `spawn_blocking`.
- Fallback logic must prioritize truthful output: no fabricated matches, explicit no-results messaging.
- Use typed `SpecDbError` propagation and include contextual error mapping inputs for MCP F2 error-shape conversion.
- Instrument query execution and composition with tracing spans (for example: `spec_db.router.query`, `spec_db.router.compose`).

### Project Structure Notes

- Primary files for this story: `crates/router/src/lib.rs`, `crates/router/src/composer.rs`, `crates/router/tests/integration.rs`.
- `crates/router/src/classifier.rs` is reused from Story 5.1; do not duplicate classification logic in composer.
- Keep public surface in `crates/router/src/lib.rs`; composer internals should remain minimal and explicit.
- Ensure crate boundaries remain unidirectional: router depends on `search`/`causal` through `spec-db-core` traits only.

### References

- Epic 5 Story 5.2 acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Epic 5: Intelligent Query Routing]
- Query router behavior and convenience-role guidance: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- Hybrid flow definition in architecture data flow: [Source: _bmad-output/planning-artifacts/architecture.md#Data Flow]
- Router crate and file layout: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- Trait interfaces for engine abstraction: [Source: _bmad-output/planning-artifacts/architecture.md#Trait Boundaries (defined in core)]
- Agent-side pattern for query router use: [Source: docs/project-context.md#Key Patterns for AI Agents]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Story file authored with explicit fan-out, composition, fallback, and response contract tasks.
- Acceptance criteria copied verbatim from Epic 5 source section.
- Cross-references added for SearchEngine/CausalGraph trait dependency and router convenience role.

### Change Log

- 2026-02-23: Initial draft created with `ready-for-dev` status.

### File List

- `_bmad-output/implementation-artifacts/5-2-hybrid-query-execution-result-composition.md`
