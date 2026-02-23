# Story 2.2: BM25 Search with Title Boosting, Tag Filtering & Spec Retrieval

Status: review

## Story

As an AI agent,
I want to search specs by keyword with relevance ranking, filter by tags, and retrieve full spec content by ID,
So that I can quickly discover the most relevant specs for my task.

## Acceptance Criteria (BDD)

**Given** an index containing specs with titles and body text
**When** I call `search_specs("rate limiting")`
**Then** I receive results ranked by BM25 relevance score (FR1)
**And** each result includes spec ID, title, tags, and relevance score (FR5)

**Given** a spec with "rate limiting" in the title and another with "rate limiting" only in the body
**When** I search for "rate limiting"
**Then** the title-match spec ranks higher than the body-match spec (FR4)

**Given** specs tagged with "auth" and "api"
**When** I search with tag filter `tags: "auth"`
**Then** only specs with the "auth" tag are returned (FR2)
**And** tag filtering uses exact string matching, not full-text

**Given** a spec with known ID `spec::auth::jwt-validation`
**When** I call `get_spec("spec::auth::jwt-validation")`
**Then** I receive the full spec content including all stored fields (FR3)

**Given** a search query on an index with 100+ specs
**When** the search executes
**Then** results are returned in < 10ms (NFR1)

**Given** a search query matching no specs
**When** the search executes
**Then** I receive an empty result set (not an error)

## Tasks / Subtasks

- [x] Implement query construction with BM25 defaults and title boosting (AC: 1, 2)
  - [x] In `crates/search/src/query.rs`, implement `pub fn build_text_query(index: &Index, fields: &SearchSchemaFields, q: &str) -> Result<Box<dyn Query>, SearchError>`.
  - [x] Use `QueryParser::for_index(index, vec![fields.title, fields.body])` and set title boost via `query_parser.set_field_boost(fields.title, TITLE_BOOST)`.
  - [x] Add `const TITLE_BOOST: f32 = 2.0` (tunable constant) and document rationale in code-level doc comment.
  - [x] Parse user text using `parse_query` and map parser failures to typed `SearchError`.

- [x] Implement exact tag filtering as term-level constraint (AC: 3)
  - [x] Add `pub fn build_tag_filter_query(fields: &SearchSchemaFields, tag: &str) -> Box<dyn Query>` using `TermQuery::new(Term::from_field_text(fields.tags, tag), IndexRecordOption::Basic)`.
  - [x] Compose query + tag filter with `BooleanQuery::new(vec![(Occur::Must, text_query), (Occur::Must, tag_query)])`.
  - [x] Ensure filter behavior is exact string matching on STRING field (`tags`), not tokenized full-text behavior.

- [x] Execute ranked retrieval and map to result DTO shape (AC: 1, 2, 6)
  - [x] In `crates/search/src/query.rs`, implement `pub fn search_specs_internal(&self, query: &str, tag: Option<&str>, limit: usize) -> Result<Vec<SearchHit>, SearchError>` on `SearchIndex`.
  - [x] Execute search with `searcher.search(&query, &TopDocs::with_limit(limit))` and preserve BM25 scores from collector tuples.
  - [x] Add search hit struct for downstream contract: `SearchHit { id, title, score, snippet }`.
  - [x] Build snippets via `SnippetGenerator::create(&searcher, &*query, fields.body)` and cap snippet length via `set_max_num_chars`.
  - [x] Return `Ok(vec![])` for no-match scenarios (no error).

- [x] Implement spec retrieval by exact ID lookup (AC: 4)
  - [x] In `crates/search/src/query.rs` or `indexer.rs`, implement `pub fn get_spec_internal(&self, id: &SpecId) -> Result<Option<SpecDoc>, SearchError>`.
  - [x] Use exact ID `TermQuery` over `id` field, limit to 1, and decode stored fields (`id`, `title`, `tags`, `meta`; body retrieval behavior must match core trait contract).
  - [x] Ensure `None` is returned for missing IDs and clear typed errors for decode/index failures.

- [x] Implement `SearchEngine` trait methods for search/retrieve (AC: 1, 3, 4, 6)
  - [x] In `crates/search/src/lib.rs`, wire trait method implementations to query-layer internals.
  - [x] Keep signatures and return types exactly aligned with `spec-db-core` trait definitions from Epic 1.
  - [x] Preserve cross-story dependency: this story reads the index created/committed by Story 2.1 implementation.

- [x] Add performance and relevance integration tests (AC: 1, 2, 3, 4, 5, 6)
  - [x] Extend `crates/search/tests/integration.rs` with fixtures containing title-only and body-only keyword matches.
  - [x] Assert title match ranks above body-only match for same keyword.
  - [x] Assert tag filter returns only exact tag matches.
  - [x] Assert known ID lookup returns full stored document content.
  - [x] Add NFR test/benchmark case with >=100 synthetic specs and warm reader; assert query latency `< 10ms` for representative terms (record median/p95).
  - [x] Assert no-match query returns empty vector.

- [x] Add observability + error propagation for production use (AC: 1, 5)
  - [x] Instrument spans `spec_db.search.query` and `spec_db.search.get_spec` with query length, limit, tag presence, and hit count.
  - [x] Map Tantivy/query parser/snippet errors to typed errors without panics.
  - [x] Ensure this crate remains synchronous; async wrapping stays at MCP handler (`spawn_blocking`) boundary.

## Dev Notes

- Story dependency and trait boundary
  - Story 2.2 depends on Story 2.1 index schema and committed index operations; do not duplicate schema logic.
  - `spec-db-search` implements `SearchEngine` from `spec-db-core`; import all shared types from core.

- Tantivy 0.25.0 API patterns relevant to this story
  - BM25 ranked retrieval comes from Tantivy query execution + `TopDocs` score ordering.
  - Field weighting via `QueryParser::set_field_boost(field, boost)` multiplies parser-level field relevance.
  - Tag filter should use `TermQuery` on `tags` STRING field and combine with user query via `BooleanQuery` + `Occur::Must`.
  - Result snippets supported by `SnippetGenerator::create`, `set_max_num_chars`, `snippet_from_doc`/`snippet`.

- Result contract and FR/F-pattern alignment
  - For downstream MCP formatting (F1), maintain normalized result payload shape `{id, title, score, snippet}`.
  - AC text includes tags in results; preserve tag storage/indexing and honor core trait result type. If core trait includes tags, keep tags in internal/domain result and map at transport boundary.

- Performance/NFR1 implementation guidance (<10ms for 100+ specs)
  - Reuse opened `IndexReader`; avoid per-query reopen.
  - Keep query path allocation-light (pre-resolve fields, avoid schema lookups per hit where possible).
  - Warm searcher in benchmarks before latency assertion.
  - Use integration benchmark assertions with stable fixture size and deterministic query terms.

- Pattern enforcement checklist for this story
  - N1/N2/N4/N5/N6: same naming/module/span/config rules as Story 2.1.
  - S1: integration tests in `crates/search/tests/integration.rs`.
  - S2: explicit public API from `lib.rs` only.
  - S3: trait-based boundary via `SearchEngine`.
  - S4: no domain types outside `spec-db-core`.
  - S5: keep module depth <=2.
  - F1: output shape compatibility for MCP layer.
  - F2: consistent typed error mapping.
  - F3: query/index behavior for known frontmatter fields, unknown fields remain in `meta` JSON.
  - F4: tracing format consistency.
  - P1/P2/P3: fail-fast errors, commit-bound persistence model from Story 2.1, sync crate API called via async boundary wrapper elsewhere.

- Error handling
  - No `unwrap()`/`expect()` in library code.
  - Parser errors, schema mismatch, and retrieval decode errors must be represented as typed search errors with context.

### Project Structure Notes

- Create/modify
  - `crates/search/src/lib.rs`
  - `crates/search/src/query.rs`
  - `crates/search/src/indexer.rs` (ID lookup helper and reader refresh integration if needed)
  - `crates/search/tests/integration.rs`

- Cross-crate usage
  - Use core trait/types from `crates/core/src/traits.rs` and `crates/core/src/types.rs`.
  - Maintain no dependency cycles; search crate remains an implementation leaf over core.

### References

- Epic story and AC source: [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.2: BM25 Search with Title Boosting, Tag Filtering & Spec Retrieval`]
- Schema and search crate architecture mapping: [Source: `_bmad-output/planning-artifacts/architecture.md#Tantivy Schema`]
- Search crate target files (`query.rs`, `schema.rs`, `indexer.rs`): [Source: `_bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure`]
- SearchEngine trait ownership: [Source: `_bmad-output/planning-artifacts/architecture.md#Trait Boundaries (defined in core)`]
- Pattern constraints N1-N6, S1-S5, F1-F4, P1-P3 and anti-patterns: [Source: `_bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`]
- NFR context for search latency: [Source: `_bmad-output/planning-artifacts/architecture.md#Requirements Coverage Validation`]
- Shared ID and runtime data constraints: [Source: `docs/project-context.md#Key Patterns for AI Agents`]
- Tantivy 0.25 API pages used for this story:
  - [Source: `https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/query/struct.BooleanQuery.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/query/enum.Occur.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/query/struct.TermQuery.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/collector/struct.TopDocs.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/snippet/struct.SnippetGenerator.html`]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Implemented `query.rs` BM25 text query builder with title field boosting, exact tag filters, Boolean query composition, ranked execution, and score-bearing `SearchHit` mapping.
- Wired `SearchEngine` trait methods in `lib.rs` to real Tantivy query execution and added tracing spans `spec_db.search.query` plus `spec_db.search.get_spec` helper on `SearchIndex`.
- Implemented exact ID retrieval with `get_spec_by_id` from stored fields (`id`, `title`, `tags`, `meta`) and explicit `body: ""` behavior because `body` is indexed TEXT but not STORED in schema.
- Added integration coverage for title ranking precedence, exact tag filtering, empty result behavior, score presence, and 100+ doc latency guard.
- Verification completed: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --all -- --check` all pass.

### Change Log

- Implemented Story 2.2 query layer and SearchEngine wiring in search crate.
- Added Story 2.2 integration tests for ranking, filtering, score, no-match, and perf guardrail.
- Updated story status/tasks and sprint status for review handoff.

### File List

- `crates/search/src/query.rs`
- `crates/search/src/lib.rs`
- `crates/search/src/indexer.rs`
- `crates/search/tests/integration.rs`
- `_bmad-output/implementation-artifacts/2-2-bm25-search-title-boosting-tag-filtering.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
