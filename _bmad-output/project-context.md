---
project_name: 'lattice'
user_name: 'Jack'
date: '2026-02-27'
sections_completed:
  ['technology_stack', 'language_rules', 'structure_rules', 'testing_rules', 'quality_rules', 'workflow_rules', 'anti_patterns']
status: 'complete'
rule_count: 47
optimized_for_llm: true
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## What Is Lattice

A causal specification database for AI agents. Combines full-text search (Tantivy) with causal reasoning (DeepCausality + Fjall) to let AI agents discover specs and trace architectural impact, exposed via MCP (Model Context Protocol).

---

## Technology Stack (Locked Versions)

| Component | Crate | Version | Notes |
|-----------|-------|---------|-------|
| Language | Rust | Edition 2024, MSRV 1.85 | No FFI, no unsafe unless absolutely necessary |
| Search | tantivy | 0.25.0 | Breaking from 0.22: columnar storage redesign |
| KV Storage | fjall | 3.0.x | Breaking from 2.x: new transaction API |
| Causal Graph | deep_causality | 0.13.4 | Stable |
| MCP Server | rmcp | 0.16.0 | Pin exact version — rapid API evolution |
| Git | git2 | 0.20.4 | Security patch applied |
| Markdown | pulldown-cmark | 0.13.0 | Minor breaking: event model update |
| YAML | serde_yml | 0.0.12 | Replaces deprecated serde_yaml |
| Async Runtime | tokio | 1.49.0 | Stable 1.x series |
| CLI | clap | 4.5.x | Derive macros |
| Serialization | bincode | 2.0.1 | Fjall key-value encoding |
| HTTP (Web) | axum | 0.8 | REST API + static serving |
| Frontend | Svelte | 5.x | @xyflow/svelte for graph canvas |

**CRITICAL:** Do NOT upgrade pinned versions (rmcp, bincode) without explicit approval.

---

## Critical Implementation Rules

### Rust Language Rules

#### Error Handling (CRITICAL)
- **Library crates** (`spec-db-*`): Use `thiserror` for typed, matchable errors
- **Binary entry point** (`src/main.rs`): Use `anyhow` for ergonomic error reporting
- **NEVER** use `unwrap()` or `expect()` in library code — always propagate with `?`
- **Tests are exempt** — `clippy.toml` allows unwrap in tests

#### Error Type Hierarchy
```
SpecDbError (thiserror)
├── SearchError    // Tantivy failures
├── GraphError     // DeepCausality/Fjall failures
├── SyncError      // git2 failures
├── IngestError    // parsing/validation failures
├── ConsistencyError // cross-store drift
└── ConfigError    // config loading failures
```

#### Module Organization
- **Modern file style**: `foo.rs` + `foo/bar.rs` — NO `mod.rs` files except crate roots
- **Maximum depth**: 2 levels within a crate (`crate::module::submodule`)
- **Crate naming**: Hyphens in Cargo.toml (`spec-db-core`), underscores in imports (`use spec_db_core`)

#### Public API Surface
- Each crate's `lib.rs` defines public API via explicit `pub use` re-exports
- Internal modules are `pub(crate)`
- **NEVER** use wildcard re-exports (`pub use foo::*`)

#### Validated Newtypes
- `SpecId`: Validated at construction, pattern `spec::{segment}::{segment}`
- `TrustLevel`: Clamped to `[0.0, 1.0]`
- Import ALL domain types from `spec-db-core` — NEVER redefine

---

### Project Structure & Crate Boundaries

#### Workspace Architecture (7 Crates)
```
lattice (binary)
├── mcp/       → MCP server (rmcp, stdio/http)
│   ├── router/    → Query classification
│   │   ├── search/    → Tantivy operations
│   │   └── causal/    → DeepCausality operations
│   └── ingest/    → Sync trigger
├── web/       → REST API + embedded Svelte UI
│   ├── search/
│   ├── causal/
│   └── ingest/
└── core/      → Shared types (EVERYONE depends on this)
```

#### Dependency Rules (STRICT)
- **Unidirectional only** — no cycles allowed
- **Import domain types from `spec-db-core` ONLY** — never redefine `SpecId`, `SpecDoc`, etc.
- **Use trait interfaces** when referencing other subsystems — never depend on concrete implementations across crate boundaries
- **Traits defined in core**: `SearchEngine`, `CausalGraph`, `SpecStore`

#### Data Ownership Boundaries
| Boundary | Owner Crate | Access Pattern |
|----------|-------------|----------------|
| `data/tantivy/` | `search` | Only search reads/writes |
| `data/fjall/` | `causal` | Only causal reads/writes |
| Git repository | `ingest` | Only ingest reads via git2 |
| `.lattice/config.yaml` | Root binary | Parsed once at startup |

#### Async Boundary (CRITICAL)
| Layer | Runtime | Pattern |
|-------|---------|---------|
| `mcp` handlers | Async (Tokio) | `#[tool]` async handlers |
| `web` handlers | Async (Tokio) | axum async handlers |
| `router`, `search`, `causal`, `ingest` | **Sync** | Called via `spawn_blocking` |

**NEVER** make Tantivy/Fjall/DeepCausality calls from async context without `spawn_blocking`.

---

### Testing Rules

#### Test Organization
- **Unit tests**: Inline with `#[cfg(test)] mod tests` in each source file
- **Integration tests**: In `tests/` directory at crate root — each crate owns its tests
- **Acceptance tests**: Workspace-level `tests/` with `acceptance_story_X_Y.rs` naming
- **NO** workspace-level test aggregation — each crate is independently testable

#### Test File Naming
- Unit tests: Same file as implementation
- Integration tests: `tests/integration.rs` or `tests/{feature}.rs`
- Acceptance tests: `tests/acceptance_story_{epic}_{story}.rs`
- Test fixtures: `tests/fixtures/*.md` for spec file samples

#### Test Commands
```bash
cargo test --workspace          # All tests
cargo test --test '*'           # Integration/acceptance only
cargo test --test acceptance_story_1_1  # Single acceptance test
cargo test -p spec-db-core      # Single crate tests
```

#### Test Requirements
- **Coverage**: All public API functions must have tests
- **Fixtures**: Use `tests/fixtures/` for sample spec files
- **No mocks for Fjall/Tantivy**: Use real instances with tempdir
- **NEVER delete failing tests** to make builds pass — fix the code

---

### Code Quality & Style Rules

#### Formatting (rustfmt.toml)
```toml
edition = "2024"
max_width = 100
use_small_heuristics = "Max"
```
Run: `cargo fmt --all -- --check` (CI enforced)

#### Linting (clippy.toml)
```toml
allow-unwrap-in-tests = true
```
Run: `cargo clippy --workspace -- -D warnings` (CI enforced)

#### Clippy Rules
- **Warn on**: `all` + `pedantic`
- **Allow**: `module_name_repetitions`
- **NEVER** add `#[allow(clippy::...)]` without a comment explaining why

#### Naming Conventions
| Context | Convention | Example |
|---------|------------|---------|
| Crate names (Cargo.toml) | kebab-case | `spec-db-core` |
| Rust imports | snake_case | `use spec_db_core` |
| Fjall node keys | Raw SpecId string | `spec::auth::jwt-validation` |
| Fjall edge keys | Null-byte separator | `{from_id}\x00{to_id}` |
| Fjall meta keys | Plain strings | `"last_sync_sha"`, `"doc_count"` |
| MCP tools | snake_case | `search_specs`, `trace_impact` |
| MCP resources | Colon-separated URI | `spec://{id}`, `graph://overview` |
| Tracing spans | Dot-separated | `spec_db.search.query`, `spec_db.graph.traverse` |
| Config fields (YAML) | snake_case | `specs_dir`, `data_dir` |

---

### Development Workflow Rules

#### Git Sync Model
- **Git is source of truth** — all spec content lives as markdown in git
- **Indexes are derived** — fully rebuildable from `git clone` + `lattice rebuild`
- **Runtime data NOT in git**: `data/tantivy/`, `data/fjall/`
- **AI edges exported to git**: `.lattice/edges.yaml` for review

#### Sync Operations
| Operation | Behavior | Use Case |
|-----------|----------|----------|
| `lattice sync` | Incremental via `git diff` | Normal workflow |
| `lattice sync --full` | Full tree walk | After drift detected |
| `lattice rebuild` | Destructive full rebuild | Recovery |

#### Sync Atomicity (CRITICAL)
- **Full rebuild**: temp-dir-then-swap (build → verify → atomic rename)
- **Incremental sync**: Fjall batches with rollback on failure
- **Cross-store consistency**: SHA + doc count checked after every sync
- **Auto-escalation**: Incremental fails → auto-trigger full rebuild

#### Initialization Failure Handling
- **Fail-fast**: Config not found, data dir not writable → exit immediately
- **Graceful degradation**: Graph load fails → search-only mode with stderr warning
- **NEVER** silently swallow errors

#### MCP Tool Response Format
All tools return JSON within `CallToolResult`:
```json
// Search results
[{"id": "...", "title": "...", "score": 0.95, "snippet": "..."}]

// Graph results  
{"node": {...}, "edges": [{"from": "...", "to": "...", "type": "..."}]}

// Errors (consistent shape)
{"error_type": "SearchError|GraphError|...", "message": "...", "context": null}
```
**NEVER** return unstructured text from MCP tools.

---

### Critical Anti-Patterns (NEVER DO)

#### Code Quality Violations
- ❌ `unwrap()` or `expect()` in library crate code — use `?` with typed errors
- ❌ `pub use foo::*` wildcard re-exports — explicit re-exports only
- ❌ Define domain types outside `spec-db-core` — single source of truth
- ❌ `mod.rs` for module files — use modern file naming (`foo.rs`)
- ❌ Nest modules deeper than 2 levels — signal to split the crate
- ❌ `#[allow(clippy::...)]` without explanatory comment

#### Async Violations
- ❌ Call Tantivy/Fjall/DeepCausality from async context without `spawn_blocking`
- ❌ Make sync subsystem code async — async boundary is MCP/web handler layer ONLY

#### MCP Violations
- ❌ Return unstructured text from MCP tools — always JSON
- ❌ Inconsistent error format — use `{error_type, message, context}` shape

#### Data Integrity Violations
- ❌ Write to Tantivy from any crate except `search`
- ❌ Write to Fjall from any crate except `causal`
- ❌ Read git directly from any crate except `ingest`
- ❌ Skip consistency checks after sync operations

#### Testing Violations
- ❌ Delete failing tests to make builds pass — fix the code
- ❌ Mock Tantivy/Fjall — use real instances with tempdir
- ❌ Skip `cargo clippy -- -D warnings` before commit

#### SpecId Violations
- ❌ Construct `SpecId` without validation — always use `SpecId::try_new()`
- ❌ Accept arbitrary strings as spec IDs — validate at ingestion boundary
- ❌ Use underscores in spec segments — only lowercase alphanumeric + hyphens

#### Serialization Violations
- ❌ Change bincode serialization format without migration — breaks Fjall data
- ❌ Use serde_yaml — use serde_yml (deprecated crate replacement)

---

## Usage Guidelines

**For AI Agents:**
- Read this file before implementing any code
- Follow ALL rules exactly as documented
- When in doubt, prefer the more restrictive option
- Update this file if new patterns emerge

**For Humans:**
- Keep this file lean and focused on agent needs
- Update when technology stack changes
- Review quarterly for outdated rules
- Remove rules that become obvious over time

---

Last Updated: 2026-02-27
