# Story 6.1: Project Initialization & Configuration

Status: review

## Story

As a spec author,
I want to initialize a new spec-db project with scaffolded structure and sensible defaults,
so that I can start writing specs immediately without manual setup.

## Acceptance Criteria (BDD)

**Given** an empty directory
**When** I run `spec-db init`
**Then** a `specs/` directory is created with example spec files demonstrating frontmatter format, `depends_on` relationships, and tag conventions (FR33)
**And** a `.spec-db/config.yaml` is created with documented defaults (spec directory, data directory, transport settings)
**And** next-steps instructions are printed to stdout

**Given** a `.spec-db/config.yaml` file
**When** the system starts
**Then** all configuration is read from this file (FR38)
**And** missing optional fields use sensible defaults
**And** missing required fields produce a clear `ConfigError`

**Given** `spec-db init` is run in a directory that already has `.spec-db/config.yaml`
**When** the command executes
**Then** it warns the user and does not overwrite existing configuration

## Tasks / Subtasks

- [x] Define CLI shape for init command in `src/main.rs`
  - [x] Add `Commands::Init` to clap enum and wire to `run_init()`
  - [x] Keep command help text aligned with FR33 and config-first startup
- [x] Implement initialization service in `src/main.rs` (or `src/init.rs` if extracted)
  - [x] Create `fn run_init(cwd: &Path) -> anyhow::Result<()>`
  - [x] Ensure directories exist: `specs/`, `.spec-db/`, `data/tantivy/`, `data/fjall/`
  - [x] Guard existing `.spec-db/config.yaml` and return warning without overwrite
- [x] Generate scaffold spec files in `specs/example/`
  - [x] Write `hello-world.md` with required frontmatter fields (`id`, `title`, `version`, `created`)
  - [x] Write `getting-started.md` with `depends_on` and `tags` examples
  - [x] Validate scaffold IDs use `spec::{segment}::{segment}` form
- [x] Implement config schema + loader using `serde_yml`
  - [x] Add `SpecDbConfig` and nested transport structs in `crates/core/src/types.rs` or config module
  - [x] Add `impl Default for SpecDbConfig` with sensible defaults (`specs/`, `data/`, stdio enabled, HTTP optional)
  - [x] Add `fn load_config(path: &Path) -> Result<SpecDbConfig, SpecDbError>` in root startup path
  - [x] Emit `ConfigError` for missing required fields and malformed YAML
- [x] Emit clear init UX
  - [x] Print created paths and next-step commands (`spec-db sync`, `spec-db serve`, `spec-db status`)
  - [x] Print non-destructive warning when config exists
- [x] Add tests
  - [x] Unit test defaults + merge semantics for optional fields
  - [x] Integration test `spec-db init` in empty temp dir
  - [x] Integration test `spec-db init` idempotency when config already exists

## Dev Notes

- Initialization is part of system administration FR33/FR38 and must keep git as source of truth (scaffold specs in git-tracked `specs/`, runtime index state in `data/`).
- YAML parser must use `serde_yml` (not deprecated `serde_yaml`) per architecture migration note.
- Root binary should use `anyhow` for user-facing errors; libraries should emit typed `SpecDbError::ConfigError`.
- Keep config field names snake_case to align with pattern N6.
- Default transport posture is local-first: stdio available by default; HTTP disabled unless explicitly configured.
- Preserve modern module style (`foo.rs` + `foo/bar.rs`) and avoid `mod.rs`.

### Project Structure Notes

- Primary touchpoints: `src/main.rs`, optional `src/init.rs`, config type location in `crates/core/src/types.rs` (or dedicated core config module).
- Generated runtime/config tree must match architecture layout: `specs/`, `.spec-db/config.yaml`, `data/tantivy/`, `data/fjall/`.
- No changes to `sprint-status.yaml`; story output only.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic 6]
- [Source: _bmad-output/planning-artifacts/architecture.md#Configuration]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/project-context.md#Repository Layout]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Added `spec-db init` CLI scaffolding with idempotent config guard and printed next-step guidance.
- Added `SpecDbConfig` YAML schema/loader with defaults and tests for defaults, partial YAML merge, and missing file errors.
- Added root integration tests that validate project structure creation and idempotent re-run behavior.
- Verified `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` all pass.

### Change Log

- Implemented Story 6.1 in code and updated artifact status to review.

### File List

- _bmad-output/implementation-artifacts/6-1-project-initialization-configuration.md
- Cargo.toml
- crates/spec-db-core/Cargo.toml
- crates/spec-db-core/src/config.rs
- crates/spec-db-core/src/lib.rs
- src/main.rs
- tests/integration.rs
