# Story 3.1: Spec Format Definition & Markdown/YAML Parsing

Status: review

## Story

As a spec author,
I want to write specs in markdown with YAML frontmatter following a defined format,
so that the system can parse and understand my specifications.

## Acceptance Criteria (BDD)

**Given** a markdown file with valid YAML frontmatter containing `id`, `title`, `version`, `tags`, `depends_on`, `owner`, and `created`
**When** the parser processes the file
**Then** all frontmatter fields are correctly extracted into a `SpecDoc` struct (FR15)
**And** the markdown body is extracted separately from the frontmatter

**Given** a spec with the format:
```yaml
---
id: "spec::auth::jwt-validation"
title: "JWT Token Validation"
version: 1
tags: ["auth", "security"]
depends_on: ["spec::auth::token-issuance"]
owner: "backend-team"
created: 2026-03-15
---
# JWT Token Validation
...markdown body...
```
**When** the parser processes it
**Then** each field maps to the corresponding `SpecDoc` field (FR19)

**Given** a spec with a `SpecId` that does not match the `spec::{segment}::{segment}` pattern
**When** the parser validates it
**Then** an `IngestError` is returned with a clear message about the invalid ID format

**Given** a spec missing required frontmatter fields (e.g., no `id` or `title`)
**When** the parser processes it
**Then** an `IngestError` is returned identifying the missing fields

**Given** a markdown file with no YAML frontmatter
**When** the parser processes it
**Then** an `IngestError` is returned indicating missing frontmatter

## Tasks / Subtasks

- [x] Create ingest workspace crate (Build Order #4)
  - [x] Add `crates/ingest/Cargo.toml` with package name `spec-db-ingest`.
  - [x] Add `crates/ingest` to `[workspace.members]` in root `Cargo.toml`.
  - [x] Add workspace deps: `pulldown-cmark = "0.13.0"`, `serde`, `serde_yml`, `thiserror`, `tracing`.
  - [x] Add local deps on `spec-db-core` and traits needed for Story 3.2 integration.

- [x] Implement parser API in `crates/ingest/src/parser.rs`
  - [x] Add `pub fn parse_spec(markdown: &str) -> Result<SpecDoc, SpecDbError>`.
  - [x] Return parsed `SpecDoc` with frontmatter fields mapped and body markdown extracted.
  - [x] Parse markdown using `pulldown_cmark::Parser::new_ext` with `Options::ENABLE_YAML_STYLE_METADATA_BLOCKS`.
  - [x] Handle pulldown-cmark 0.13 event model (`Event::Start(Tag::MetadataBlock(_))` + `Event::End(TagEnd)`), not old `Event::End(Tag)` handling.
  - [x] Extract metadata block text only from frontmatter region and preserve remaining markdown body verbatim.

- [x] Implement YAML decoding in `crates/ingest/src/parser.rs`
  - [x] Define frontmatter DTO (`RawFrontmatter`) with serde derives for fields: `id`, `title`, `version`, `tags`, `depends_on`, `owner`, `created`.
  - [x] Deserialize with `serde_yml::from_str::<RawFrontmatter>(&frontmatter)`.
  - [x] Convert DTO into `spec_db_core::SpecDoc` and normalize optionals (`tags`, `depends_on`, `owner`).
  - [x] Keep field mapping aligned to current `SpecDoc` schema (no `meta` field exists in core type).

- [x] Implement validation in `crates/ingest/src/validate.rs`
  - [x] Add `pub fn validate_frontmatter(raw: &RawFrontmatter) -> Result<(), IngestError>` for required field checks.
  - [x] Add `pub fn validate_spec_id(id: &str) -> Result<SpecId, IngestError>` that enforces `spec::{segment}::{segment}`.
  - [x] Enforce lowercase alphanumeric+hyphen segments and reject empty segments.
  - [x] Return explicit missing-field errors (`MissingField("id")`, `MissingField("title")`) and invalid-id errors with actionable text.

- [x] Wire crate exports and error flow in `crates/ingest/src/lib.rs`
  - [x] Re-export parser entry points needed by Story 3.2 pipeline.
  - [x] Ensure all library errors are typed (`thiserror`) and no `unwrap`/`expect` in non-test code.
  - [x] Add tracing spans for parser and validator public APIs (`spec_db.ingest.parse`, `spec_db.ingest.validate`).

- [x] Add parser test coverage in `crates/ingest/tests/`
  - [x] Add fixtures: `fixtures/valid_spec.md`, `fixtures/invalid_id.md`, `fixtures/missing_fields.md`, `fixtures/multi_depends.md`.
  - [x] Add integration tests for success mapping to `SpecDoc` and clean body extraction.
  - [x] Add failure tests for missing frontmatter, invalid id format, and missing required fields.
  - [x] Add test asserting multi-value `depends_on` maps exactly to `Vec<SpecId-like strings>` in parsed document.

## Dev Notes

- CRITICAL dependency rule: use `serde_yml` for YAML parsing; the older `serde_yaml` crate is deprecated and must not be introduced in new ingest code.
- pulldown-cmark version lock for this story is `0.13.0`; parser code must account for the event model where closing tags are `Event::End(TagEnd)`.
- Metadata extraction should use parser options that emit metadata blocks (`Options::ENABLE_YAML_STYLE_METADATA_BLOCKS`) and handle `Tag::MetadataBlock` events.
- Spec ID is a universal key across Tantivy, Fjall, and graph node IDs; validation at ingestion boundary is mandatory.
- This story delivers the parser that Story 3.2 depends on (`parse_spec` output feeds pipeline ingest).
- Keep module style modern (`parser.rs`, `validate.rs`; no `mod.rs`) and keep public API through `lib.rs` re-exports.

### Project Structure Notes

- New crate path: `crates/ingest/` (`spec-db-ingest`) in workspace expansion phase for Build Order #4-5.
- Primary files for this story:
  - `crates/ingest/src/lib.rs`
  - `crates/ingest/src/parser.rs`
  - `crates/ingest/src/validate.rs`
  - `crates/ingest/tests/integration.rs`
  - `crates/ingest/tests/fixtures/{valid_spec.md,invalid_id.md,missing_fields.md,multi_depends.md}`
- No changes to sync orchestration yet beyond parser/validator interfaces needed by Story 3.2.

### References

- Epic story and acceptance criteria: [Source: _bmad-output/planning-artifacts/epics.md#Story-3.1-Spec-Format-Definition--MarkdownYAML-Parsing]
- Ingest crate location and fixture set: [Source: _bmad-output/planning-artifacts/architecture.md#Complete-Project-Directory-Structure]
- Serialization + SpecId constraints: [Source: _bmad-output/planning-artifacts/architecture.md#Data-Architecture]
- Project-wide format contract for frontmatter fields: [Source: docs/project-context.md#Spec-Document-Format]
- pulldown-cmark 0.13 API details (Event/Tag/Options): [Source: https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.Event.html], [Source: https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/enum.Tag.html], [Source: https://docs.rs/pulldown-cmark/0.13.0/pulldown_cmark/struct.Options.html]
- serde_yml API and replacement guidance: [Source: https://docs.rs/serde_yml/0.0.11/serde_yml/]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Created new `spec-db-ingest` crate and wired workspace membership/dependency path updates.
- Implemented markdown+YAML frontmatter parsing with pulldown-cmark 0.13 metadata events and `serde_yml` decoding.
- Added frontmatter/spec-id validation and conversion into `SpecDoc` with optional field normalization.
- Added fixture-backed integration tests for success/failure paths and body extraction behavior.
- Verified with `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --all -- --check`.

### Change Log

- 2026-02-23: Initial ready-for-dev story file created.
- 2026-02-23: Implemented Story 3.1 ingest crate, parser/validator, fixtures, tests, and workspace wiring.

### File List

- `Cargo.toml`
- `crates/ingest/Cargo.toml`
- `crates/ingest/src/lib.rs`
- `crates/ingest/src/parser.rs`
- `crates/ingest/src/validate.rs`
- `crates/ingest/tests/integration.rs`
- `crates/ingest/tests/fixtures/valid_spec.md`
- `crates/ingest/tests/fixtures/invalid_id.md`
- `crates/ingest/tests/fixtures/missing_fields.md`
- `crates/ingest/tests/fixtures/multi_depends.md`
- `_bmad-output/implementation-artifacts/3-1-spec-format-definition-markdown-yaml-parsing.md`
