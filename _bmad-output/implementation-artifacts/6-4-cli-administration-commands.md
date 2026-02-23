# Story 6.4: CLI Administration Commands

Status: review

## Story

As a spec author,
I want CLI commands to manage the MCP server, trigger syncs, rebuild indexes, and check system health,
so that I can operate spec-db without needing an AI agent.

## Acceptance Criteria (BDD)

**Given** a configured spec-db project
**When** I run `spec-db serve`
**Then** the MCP server starts from `.spec-db/config.yaml` (FR34)
**And** an initial sync runs automatically if no index exists
**And** a cross-store consistency check runs before serving

**Given** a running spec-db project
**When** I run `spec-db sync`
**Then** an incremental sync is triggered (FR35)
**And** when I run `spec-db sync --full`, a full rebuild is triggered instead (FR35)

**Given** a spec-db project with existing indexes
**When** I run `spec-db rebuild`
**Then** a destructive full index rebuild executes (FR36)
**And** both stores are rebuilt from git (idempotent)

**Given** a spec-db project
**When** I run `spec-db status`
**Then** I see document count, last sync commit SHA, and consistency check result (FR37)
**And** the output clearly indicates whether stores are consistent or drifted

**Given** any CLI command
**When** it encounters an error
**Then** a human-readable error message is printed via `anyhow` — no stack traces unless `RUST_BACKTRACE=1`

## Tasks / Subtasks

- [x] Define clap 4.5.x command model in `src/main.rs`
  - [x] Add `Commands` enum variants: `Init`, `Serve`, `Sync { full: bool }`, `Rebuild`, `Status`
  - [x] Add help text/examples and `--full` flag on `sync`
  - [x] Parse once and dispatch through `run_command(cli.command)`
- [x] Implement startup/config wiring
  - [x] Add `fn load_project_config() -> anyhow::Result<SpecDbConfig>` from `.spec-db/config.yaml`
  - [x] Build shared app services from config (search, causal, ingest, router, mcp)
  - [x] Keep typed errors in crates; map to `anyhow::Error` at binary boundary
- [x] Implement `serve` flow
  - [x] Add `async fn run_serve(cfg: &SpecDbConfig) -> anyhow::Result<()>`
  - [x] If indexes absent, run initial sync automatically
  - [x] Run consistency check before opening MCP transport
  - [x] Start stdio MCP server always; start streamable-http only if configured
- [x] Implement `sync` flow
  - [x] `spec-db sync` invokes incremental sync path
  - [x] `spec-db sync --full` invokes full rebuild path
  - [x] Print Admin F1 result with status/message/details summary
- [x] Implement `rebuild` flow
  - [x] Confirm destructive rebuild execution path (full index reset + reindex from git)
  - [x] Ensure Tantivy + Fjall are rebuilt idempotently
  - [x] Print completion status and rebuilt doc counts
- [x] Implement `status` flow
  - [x] Read current doc count from search index metadata
  - [x] Read last sync SHA from causal/meta keyspace
  - [x] Run/report consistency state as `consistent` or `drifted`
  - [x] Format output for human operators (single command diagnostic)
- [x] Harden error handling and UX
  - [x] Wrap top-level `main` return type as `anyhow::Result<()>`
  - [x] Print concise human-readable errors; rely on `RUST_BACKTRACE` for traces
  - [x] Avoid panics/unwraps in command handlers
- [x] Add integration coverage in `tests/cli.rs`
  - [x] Command parsing tests for all subcommands and `--full`
  - [x] End-to-end temp-repo tests for `init`, `sync`, `rebuild`, `status`
  - [x] `serve` preflight tests: initial sync trigger + consistency check invocation

## Dev Notes

- This story closes Epic 6 by wiring all prior subsystems into operator-facing commands.
- `src/main.rs` is the binary orchestration boundary: clap parsing, config load, service construction, and command dispatch.
- Error pattern is strict: `thiserror` in crates, `anyhow` only at binary level.
- `serve` must exercise both administration and MCP concerns: index bootstrap, consistency validation, then transport startup.
- `sync` and `rebuild` semantics must stay distinct: incremental vs destructive full rebuild.

### Project Structure Notes

- Core CLI work is in `src/main.rs`; helper extraction to `src/commands.rs` is acceptable if command handlers grow.
- CLI directly invokes crate APIs but should depend on traits/types from `spec-db-core`, not internal concrete modules across crate boundaries.
- Maintain output consistency with MCP admin response vocabulary (`status`, `message`, `details`) where relevant.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 6.4]
- [Source: _bmad-output/planning-artifacts/architecture.md#Error Handling]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Flow]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Expanded CLI command model with `serve`, `sync`, `rebuild`, and `status` alongside `init`.
- Added binary-level config loading, sync/rebuild orchestration, status reporting, and serve preflight consistency checks.
- Added integration coverage for command parsing and expected config-loading failure behavior on empty projects.

### Change Log

- Implemented Story 6.4 and moved status to review.

### File List

- _bmad-output/implementation-artifacts/6-4-cli-administration-commands.md
- Cargo.toml
- src/main.rs
- tests/integration.rs
