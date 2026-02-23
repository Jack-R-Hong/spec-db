# Story 5.1: Query Intent Classification

Status: review

## Story

As an AI agent,
I want my natural-language queries automatically classified by intent,
so that they are routed to the correct engine without me needing to choose the right tool.

## Acceptance Criteria (BDD)

**Given** a query containing causal signal words ("impact", "depends", "breaks", "affects", "upstream", "downstream")
**When** the classifier processes it
**Then** it is classified as a causal query and routed to the causal graph engine (FR11)

**Given** a query without causal signal words (e.g., "rate limiting API")
**When** the classifier processes it
**Then** it is classified as a search query and routed to the Tantivy search engine (FR11)

**Given** a query containing both causal signals and search terms (e.g., "what depends on rate limiting")
**When** the classifier processes it
**Then** it is classified as a hybrid query and routed to both engines (FR11)

**Given** any query
**When** classification executes
**Then** the overhead is < 5ms (NFR3)

## Tasks / Subtasks

- [x] Scaffold router crate and wire module surface for Build Order #6 in `crates/router/Cargo.toml` and `crates/router/src/lib.rs`.
  - [x] Set package name to `spec-db-router` and depend on `spec-db-core` traits (`SearchEngine`, `CausalGraph`) for cross-crate boundaries.
  - [x] Export classifier API from `crates/router/src/lib.rs` via explicit re-exports (no wildcard exports).
- [x] Implement keyword heuristic classifier in `crates/router/src/classifier.rs`.
  - [x] Add `enum QueryIntent { Search, Causal, Hybrid }` and `struct IntentClassifier`.
  - [x] Implement `fn classify(query: &str) -> QueryIntent` with case-insensitive token matching for causal signals: `impact`, `depends`, `breaks`, `affects`, `upstream`, `downstream`.
  - [x] Implement hybrid detection when both causal signals and non-empty search terms are present.
  - [x] Set default path to `QueryIntent::Search` when no causal signal words are detected.
- [x] Add low-overhead normalization utilities in `crates/router/src/classifier.rs`.
  - [x] Normalize once per query (lowercase + lightweight token scan) without allocation-heavy transforms.
  - [x] Keep classifier path pure sync and side-effect free to support `< 5ms` overhead target (NFR3).
- [x] Add router integration contract in `crates/router/src/lib.rs`.
  - [x] Add `QueryRouter<S: SearchEngine, C: CausalGraph>` field ownership and constructor wiring.
  - [x] Route by intent only (classification phase), with execution/composition delegated to Story 5.2 implementation.
  - [x] Document convenience-only role for unified `query()` API; direct `search_specs()` and `trace_impact()` remain primary paths.
- [x] Implement tests in `crates/router/tests/integration.rs` and unit tests in `crates/router/src/classifier.rs`.
  - [x] Verify causal classification for each required signal word.
  - [x] Verify search-default behavior for queries without causal words.
  - [x] Verify hybrid classification for mixed queries (example: "what depends on rate limiting").
  - [x] Add micro-benchmark-style timing assertion harness (non-flaky threshold guard) to validate `< 5ms` classification overhead on representative query corpus.

## Dev Notes

- Implement this story as Build Order #6 (`router`) after `core`, `causal`, `search`, and `ingest` are available.
- Classification is strictly keyword heuristics; do not introduce ML, embeddings, or probabilistic intent models.
- Causal signals are fixed for MVP: `impact`, `depends`, `breaks`, `affects`, `upstream`, `downstream`.
- Router is a convenience abstraction for MCP `query()`; agent-first flow is still explicit tool calls (`search_specs`, `trace_impact`).
- Respect trait boundaries from `spec-db-core`: router consumes `SearchEngine` and `CausalGraph` interfaces, never concrete crate internals.
- Keep router sync. Async boundary remains in MCP handlers via `spawn_blocking`.
- Add `tracing` spans for public router/classifier APIs using dot-style naming (for example `spec_db.router.classify`).
- Use typed errors from `spec-db-core` (`thiserror` hierarchy) and propagate with context; no `unwrap()`/`expect()` in library code.

### Project Structure Notes

- New crate path: `crates/router/` with `src/lib.rs`, `src/classifier.rs`, `src/composer.rs`, and `tests/integration.rs`.
- Use modern module layout (`foo.rs` + `foo/bar.rs` where needed), no `mod.rs` pattern.
- Public API only through explicit exports in `crates/router/src/lib.rs`; internal helpers stay `pub(crate)`.
- This story owns intent classification and intent-to-route mapping only; response composition behavior is completed in Story 5.2.

### References

- Epic 5 story and acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Epic 5: Intelligent Query Routing]
- Query router heuristic and causal signal definitions: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- Build sequence (router as step #7 / Build Order #6): [Source: _bmad-output/planning-artifacts/architecture.md#Decision Impact Analysis]
- Router crate/file targets: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- Trait boundaries (`SearchEngine`, `CausalGraph` in core): [Source: _bmad-output/planning-artifacts/architecture.md#Trait Boundaries (defined in core)]
- Agent usage pattern and query router role: [Source: docs/project-context.md#Key Patterns for AI Agents]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Story file authored with comprehensive implementation guidance for heuristic classification.
- Acceptance criteria copied verbatim from Epic 5 source section.
- Tasks aligned to router crate structure and trait-boundary constraints.

### Change Log

- 2026-02-23: Initial draft created with `ready-for-dev` status.

### File List

- `_bmad-output/implementation-artifacts/5-1-query-intent-classification.md`
