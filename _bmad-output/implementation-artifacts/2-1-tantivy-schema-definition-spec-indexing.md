# Story 2.1: Tantivy Schema Definition & Spec Indexing

Status: ready-for-dev

## Story

As a developer,
I want a Tantivy search index with the defined schema and operations to add, remove, and commit spec documents,
So that specs are indexed and ready for full-text search queries.

## Acceptance Criteria (BDD)

**Given** the `spec-db-search` crate with Tantivy 0.25.0
**When** I create a new search index
**Then** the schema contains fields: `id` (STRING|STORED), `title` (TEXT|STORED), `body` (TEXT), `tags` (STRING|STORED), `meta` (JSON|STORED)

**Given** a `SpecDoc` with id, title, body, tags, and metadata
**When** I call `add_doc` on the `SearchEngine` implementation
**Then** the document is added to the Tantivy index
**And** the document is retrievable after `commit`

**Given** a spec ID of an indexed document
**When** I call `remove_doc` with that ID
**Then** the document is removed from the index after `commit`
**And** subsequent searches do not return the removed document

**Given** an index with uncommitted changes
**When** I call `commit`
**Then** all pending additions and removals are persisted atomically

## Tasks / Subtasks

- [ ] Create workspace crate wiring for build order #3 (AC: 1)
  - [ ] Update workspace members in `Cargo.toml` to include `crates/search` and crate package name `spec-db-search`.
  - [ ] Add/pin workspace dependency `tantivy = "0.25.0"` (plus `serde_json`, `thiserror`, `tracing` if not already in workspace dependencies) in root `Cargo.toml`.
  - [ ] Create `crates/search/Cargo.toml` with dependencies on `spec-db-core` and workspace-shared crates.

- [ ] Scaffold `spec-db-search` module layout following N1/S2/S5 (AC: 1)
  - [ ] Create `crates/search/src/lib.rs` as the only public API surface with explicit `pub use` exports (no wildcard exports).
  - [ ] Create `crates/search/src/schema.rs` for Tantivy schema and field handles.
  - [ ] Create `crates/search/src/indexer.rs` for index lifecycle + write operations.
  - [ ] Create `crates/search/src/query.rs` placeholder interface used by Story 2.2.

- [ ] Implement Tantivy schema builder exactly as defined (AC: 1)
  - [ ] In `crates/search/src/schema.rs`, implement `pub struct SearchSchemaFields` containing `Schema` and handles for `id`, `title`, `body`, `tags`, `meta`.
  - [ ] Implement `pub fn build_schema() -> SearchSchemaFields` using `Schema::builder()` and:
    - `add_text_field("id", STRING | STORED)`
    - `add_text_field("title", TEXT | STORED)`
    - `add_text_field("body", TEXT)`
    - `add_text_field("tags", STRING | STORED)`
    - `add_json_field("meta", STORED)`
  - [ ] Keep field names as constants to prevent query/index drift across stories.

- [ ] Implement index writer operations and SearchEngine trait binding (AC: 2, 3, 4)
  - [ ] In `crates/search/src/indexer.rs`, implement `pub struct SearchIndex` that owns Tantivy `Index`, `IndexReader`, and `IndexWriter` plus `SearchSchemaFields`.
  - [ ] Implement `pub fn open_or_create(index_dir: &Path) -> Result<SearchIndex, SearchError>`.
  - [ ] Implement `pub fn add_doc_internal(&mut self, doc: &SpecDoc) -> Result<(), SearchError>` mapping `SpecDoc` fields into Tantivy `doc!` values.
  - [ ] Implement `pub fn remove_doc_internal(&mut self, id: &SpecId) -> Result<(), SearchError>` using `Term::from_field_text(id_field, id.as_ref())` + delete API.
  - [ ] Implement `pub fn commit_internal(&mut self) -> Result<(), SearchError>` and refresh reader state after successful commit.
  - [ ] Implement `SearchEngine` for `SearchIndex` in `crates/search/src/lib.rs` by delegating trait methods to internal methods above (exact signatures must match `spec-db-core` trait).

- [ ] Implement typed error mapping and tracing instrumentation (AC: 2, 3, 4)
  - [ ] Create `SearchError` in this crate only for implementation-specific failures; map into core error hierarchy as required by `spec-db-core` interfaces.
  - [ ] Add tracing spans on public operations with N5 naming (`spec_db.search.add_doc`, `spec_db.search.remove_doc`, `spec_db.search.commit`).
  - [ ] Use `?` propagation and typed errors; no `unwrap()`/`expect()` in library code.

- [ ] Add integration coverage for indexing lifecycle behavior (AC: 1, 2, 3, 4)
  - [ ] Create `crates/search/tests/integration.rs` with a temporary on-disk index fixture.
  - [ ] Test schema field existence + options by reading schema metadata from `SearchSchemaFields`.
  - [ ] Test add + commit + query roundtrip for one `SpecDoc`.
  - [ ] Test remove + commit and assert document no longer appears in search results.
  - [ ] Test multiple queued updates committed in one `commit` call (atomic persistence behavior from caller perspective).

## Dev Notes

- Story dependency and boundary
  - Story 2.1 creates the index and write-path foundations that Story 2.2 reads/query-executes.
  - Import all domain types (`SpecId`, `SpecDoc`, trait interfaces, shared errors) from `spec-db-core`; do not redefine domain models.
  - This crate is the `SearchEngine` trait implementation crate for Tantivy.

- Required crate versions and ecosystem constraints
  - `tantivy = 0.25.0` (locked for Epic 2; note architecture migration from 0.22 to 0.25).
  - Keep versions aligned with workspace dependency policy in root `Cargo.toml`.
  - Tantivy 0.25 schema/query APIs confirmed from docs.rs latest pages for `SchemaBuilder`, `QueryParser`, `TermQuery`, `BooleanQuery`, `Occur`, `TopDocs`, `SnippetGenerator`.

- Tantivy 0.25 schema/indexing API patterns to use
  - Schema construction: `Schema::builder()`, `add_text_field`, `add_json_field`, then `build()`.
  - Canonical field options use bitflags (`TEXT | STORED`, `STRING | STORED`, `TEXT`).
  - Document ingestion uses `doc!` macro; write operations stay buffered until `commit`.
  - Deletion uses exact `Term` on `id` field, then `commit`.

- Pattern enforcement checklist for this story
  - N1: modern module layout (`schema.rs`, `indexer.rs`, `query.rs`), no `mod.rs`.
  - N2: crate named `spec-db-search`, imported as `spec_db_search`.
  - N3: `SpecId` textual value is the exact key stored in Tantivy `id` field.
  - N4: expose/search methods in snake_case (e.g., `add_doc`, `remove_doc`, `commit`).
  - N5: tracing span names `spec_db.search.*`.
  - N6: any config structs remain snake_case.
  - S1: unit tests inline; integration tests in `crates/search/tests/`.
  - S2: explicit `pub use` in `lib.rs`; no `pub use foo::*`.
  - S3: implement trait(s) from `spec-db-core`; no cross-crate concrete coupling.
  - S4: keep domain types in core; this crate only defines search implementation types.
  - S5: max 2-level module depth.
  - F1: search output target shape for downstream MCP remains compatible with `{id, title, score, snippet}` in Story 2.2.
  - F2: map errors into consistent typed error payloads; do not return unstructured error strings from library boundaries.
  - F3: index known fields and preserve additional frontmatter in `meta` JSON.
  - F4: use tracing for human-readable logs with optional structured export.
  - P1: fail fast on index init failures; no silent fallback in this crate.
  - P2: rely on Tantivy commit boundary; caller sees add/remove persisted only after commit.
  - P3: keep this crate sync; async wrapping belongs at MCP layer (`spawn_blocking`).

- Error handling requirements
  - Library code: typed errors + `?` propagation.
  - No panic-oriented flow (`unwrap`, `expect`) outside test code.
  - Include context (field name, spec id, operation) in error variants for diagnosis.

### Project Structure Notes

- Create/modify
  - `Cargo.toml` (workspace members/dependencies)
  - `crates/search/Cargo.toml`
  - `crates/search/src/lib.rs`
  - `crates/search/src/schema.rs`
  - `crates/search/src/indexer.rs`
  - `crates/search/src/query.rs` (stub contracts consumed by Story 2.2)
  - `crates/search/tests/integration.rs`

- Ownership and boundaries
  - `spec-db-search` is sole owner of `data/tantivy/` read/write logic.
  - Query/router/MCP crates consume this crate through `SearchEngine` trait boundary only.

### References

- Epic story and AC source: [Source: `_bmad-output/planning-artifacts/epics.md#Epic 2: Spec Discovery & Search`]
- Story 2.1 AC lines: [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.1: Tantivy Schema Definition & Spec Indexing`]
- Tantivy schema definition and field contract: [Source: `_bmad-output/planning-artifacts/architecture.md#Tantivy Schema`]
- Crate/file target structure and build order: [Source: `_bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure`]
- Trait boundary (`SearchEngine` implemented by search crate): [Source: `_bmad-output/planning-artifacts/architecture.md#Trait Boundaries (defined in core)`]
- Naming/structure/format/process patterns N1-N6, S1-S5, F1-F4, P1-P3: [Source: `_bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules`]
- Anti-patterns (`unwrap`, wildcard exports, domain type duplication): [Source: `_bmad-output/planning-artifacts/architecture.md#Anti-Patterns`]
- Git-derived runtime/index constraints: [Source: `docs/project-context.md#Critical Architectural Decisions`]
- Tantivy 0.25 API pages used for implementation details:
  - [Source: `https://docs.rs/tantivy/latest/tantivy/schema/struct.SchemaBuilder.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/query/struct.QueryParser.html`]
  - [Source: `https://docs.rs/tantivy/latest/tantivy/query/struct.TermQuery.html`]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story authored from Epic 2 + architecture constraints with Tantivy 0.25 API validation.
- AC copied verbatim from source artifact.
- Tasks map each AC to concrete files/functions and preserve trait boundary with `spec-db-core`.

### Change Log

- Created initial ready-for-dev story draft for Epic 2 Story 2.1.

### File List

- `_bmad-output/implementation-artifacts/2-1-tantivy-schema-definition-spec-indexing.md`
