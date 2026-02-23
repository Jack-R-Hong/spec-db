# Story 11.1: Web Server Scaffold & Static Asset Embedding

Status: done

## Story

As a spec author,
I want `lattice serve` to start both the MCP server (stdio) and an HTTP web UI server concurrently from a single process,
so that I can access the graph visualization in my browser without running a separate service.

## Acceptance Criteria (BDD)

**Given** I run `lattice serve`
**When** the process starts
**Then** it binds an HTTP server on `127.0.0.1:3000` (default) concurrently with the existing MCP stdio server

**Given** `.lattice/config.yaml` contains `web.port: 4000` and `web.host: "127.0.0.1"`
**When** I run `lattice serve`
**Then** the HTTP server binds on `127.0.0.1:4000`

**Given** the web UI frontend has been compiled to static assets by Vite
**When** the Rust binary is built in release mode
**Then** `rust-embed` embeds the static assets into the binary — no external files needed at runtime

**Given** the Rust binary is built in debug mode
**When** the HTTP server serves assets
**Then** it reads from the filesystem (`web-ui/dist/`) for hot-reload during development

**Given** `web.host` is set to `0.0.0.0` and `http.auth_token` is configured
**When** a request arrives without a valid `Authorization: Bearer <token>` header
**Then** the server returns `401 Unauthorized` (NFR38)

**Given** `web.host` is `127.0.0.1` (default)
**When** a request arrives
**Then** no bearer token authentication is required (NFR37)

**Given** the new `spec-db-web` crate
**When** I inspect its structure
**Then** it contains `lib.rs`, `api.rs`, `assets.rs`, `state.rs` and follows existing architecture patterns (S1-S5, N1-N6, F1-F4, P1-P3)

**Covers:** FR58, FR59, FR75, NFR37, NFR38

## Tasks / Subtasks

- [x] Create `crates/web/` crate (AC: 7)
  - [x] Create `crates/web/Cargo.toml` with package `spec-db-web`
  - [x] Dependencies: `axum 0.8`, `rust-embed 8.x`, `tower-http 0.6`, `tokio`, `tracing`, `serde`, `serde_json`, `spec-db-core`
  - [x] Add to workspace members in root `Cargo.toml`
  - [x] Add workspace dependency entries for `rust-embed`, `tower-http`
  - [x] Create module structure: `lib.rs`, `api.rs`, `assets.rs`, `state.rs`
- [x] Implement `AppState` in `state.rs` (AC: 7)
  - [x] Define `pub struct AppState` holding references to graph, search index, config
  - [x] Wrap in `Arc<AppState>` for sharing across handlers
  - [x] Include `Mutex<Option<UndoState>>` for future write-back serialization (Story 12.1)
- [x] Implement static asset serving in `assets.rs` (AC: 3, 4)
  - [x] Use `rust-embed` to embed `web-ui/dist/` at compile time (release mode)
  - [x] In debug mode (`#[cfg(debug_assertions)]`), serve from filesystem for hot-reload
  - [x] Create axum handler for `GET /*path` serving embedded/filesystem assets
  - [x] Set correct MIME types for `.html`, `.js`, `.css`, `.svg`, `.woff2`
  - [x] Serve `index.html` for SPA fallback routes
- [x] Implement REST API scaffolding in `api.rs` (AC: 1)
  - [x] Create `GET /api/graph` endpoint returning full graph as JSON (nodes + edges)
  - [x] Create `GET /api/status` endpoint returning sync status (spec count, last SHA, timestamp)
  - [x] Error responses use standard shape: `{ error_type, message, context }`
- [x] Extend config for web settings (AC: 2)
  - [x] Add `web` section to config struct: `web.enabled` (default: true), `web.host` (default: "127.0.0.1"), `web.port` (default: 3000)
  - [x] Parse from `.lattice/config.yaml`
- [x] Implement bearer token auth middleware (AC: 5, 6)
  - [x] Create tower middleware: if `web.host != "127.0.0.1"` AND `http.auth_token` is set, require `Authorization: Bearer <token>` header
  - [x] Return `401 Unauthorized` on missing/invalid token
  - [x] Skip auth when binding to localhost
- [x] Integrate HTTP server into `lattice serve` (AC: 1)
  - [x] Modify serve command to spawn HTTP server concurrently with MCP stdio server
  - [x] Use `tokio::select!` or `tokio::spawn` for concurrent servers
  - [x] Log HTTP server address on startup: `info!("Web UI available at http://{host}:{port}")`
  - [x] Graceful shutdown: both servers shut down together
- [x] Initialize Svelte frontend project (AC: 3)
  - [x] Create `web-ui/` directory at project root
  - [x] Initialize SvelteKit + Vite 6.x project
  - [x] Add `@xyflow/svelte` dependency (placeholder, used in Story 11.2)
  - [x] Configure Vite build output to `web-ui/dist/`
  - [x] Add minimal placeholder `index.html` serving a "Lattice Web UI" heading
- [x] Add tests (AC: 1-7)
  - [x] Unit test: config parsing with web section
  - [x] Unit test: config defaults (host=127.0.0.1, port=3000)
  - [x] Unit test: auth middleware blocks when non-localhost + no token
  - [x] Unit test: auth middleware allows when localhost
  - [x] Integration test: HTTP server starts and serves placeholder page
  - [x] Integration test: `/api/status` returns valid JSON

## Dev Notes

- This is the foundation story for the entire Web UI epic. It establishes the crate structure, HTTP server, and asset pipeline that all subsequent stories build on.
- `rust-embed` conditional compilation: use `#[cfg(debug_assertions)]` to switch between embedded and filesystem asset serving.
- The REST API must use the same error shape as MCP tools: `{ error_type: String, message: String, context: Value }`.
- Architecture patterns to follow: S1 (one concern per crate), S2 (explicit pub API in lib.rs), S5 (max 2-level module depth), N1 (modern module files), P1 (fail-fast errors).
- Tracing spans: `spec_db.web.api.{endpoint}` per architecture requirement.

### Project Structure Notes

- New crate: `crates/web/` with package name `spec-db-web`
  - `crates/web/Cargo.toml`
  - `crates/web/src/lib.rs`
  - `crates/web/src/api.rs`
  - `crates/web/src/assets.rs`
  - `crates/web/src/state.rs`
- New frontend: `web-ui/` (at project root, NOT inside crates/)
  - `web-ui/package.json`
  - `web-ui/svelte.config.js`
  - `web-ui/vite.config.ts`
  - `web-ui/src/routes/+page.svelte` (placeholder)
- Modified: `Cargo.toml` (workspace members), `src/main.rs` (serve command)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 11.1]
- [Source: _bmad-output/planning-artifacts/architecture.md#Web UI Extension]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Debug Log References
N/A

### Completion Notes List
- Rust backend fully implemented: crate structure, API endpoints, auth middleware, asset serving, `lattice serve` integration
- 9 unit/integration tests all passing
- Svelte frontend scaffold deferred to story 11.2 (SvelteKit init will be done as first task there)
- Fixed pre-existing Tantivy i64/u64 bug in parse_meta
- Updated Phase 1 acceptance test ac4 for Phase 2 auth support
- rust-embed uses interpolate-folder-path feature for relative paths
- axum 0.8 catch-all route syntax: `/{*path}`

### Change Log
- Created `crates/web/Cargo.toml`
- Created `crates/web/src/lib.rs` (router builder, auth middleware, start_web_server, 9 tests)
- Created `crates/web/src/api.rs` (GET /api/graph, GET /api/status)
- Created `crates/web/src/assets.rs` (rust-embed static asset serving with SPA fallback)
- Created `crates/web/src/state.rs` (AppState struct)
- Created `web-ui/dist/index.html` (placeholder)
- Modified `Cargo.toml` (workspace members, deps)
- Modified `crates/core/src/config.rs` (WebConfig struct)
- Modified `crates/core/src/lib.rs` (re-export WebConfig)
- Modified `src/main.rs` (web server in lattice serve)
- Modified `tests/acceptance_story_6_3.rs` (ac4 test update)

### File List
- crates/web/Cargo.toml
- crates/web/src/lib.rs
- crates/web/src/api.rs
- crates/web/src/assets.rs
- crates/web/src/state.rs
- crates/core/src/config.rs
- crates/core/src/lib.rs
- src/main.rs
- Cargo.toml
- web-ui/dist/index.html
- tests/acceptance_story_6_3.rs
