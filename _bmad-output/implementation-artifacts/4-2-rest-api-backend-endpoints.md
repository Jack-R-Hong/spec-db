# Story 4.2: REST API Backend Endpoints

Status: ready-for-dev

## Story

As a platform engineer,
I want REST API endpoints to manage backends at runtime,
so that automation scripts can provision backends without CLI access.

## Acceptance Criteria

1. `GET /api/backends` — JSON array of backends with name, type, status
2. `POST /api/backends` — create backend from `{ name, type, config }`, return 201
3. `DELETE /api/backends/:name` — remove backend, return 204
4. `GET /api/backends/:name/status` — health info as JSON
5. Invalid requests return 400/404/409
6. Behind existing Axum web server (`web.enabled: true`)
7. Integration test for CRUD via HTTP

## Tasks / Subtasks

- [ ] Task 1: Add routes to `crates/web/` (AC: #1, #2, #3, #4, #6)
  - [ ] Add `/api/backends` routes to Axum router
  - [ ] `list_backends` handler
  - [ ] `create_backend` handler
  - [ ] `delete_backend` handler
  - [ ] `backend_status` handler
- [ ] Task 2: Request/response types (AC: #1, #2, #5)
  - [ ] Define JSON request/response structs with serde
  - [ ] Error responses with proper status codes
- [ ] Task 3: Wire to BackendRegistry (AC: #1, #2, #3, #4)
  - [ ] Pass `Arc<RwLock<BackendRegistry>>` to handlers via Axum state
- [ ] Task 4: Integration test (AC: #7)
  - [ ] Test full CRUD lifecycle via reqwest or axum test client

## Dev Notes

### Existing Web Pattern

From `crates/web/` — Axum router with state:
```rust
Router::new()
    .route("/api/search", get(search_handler))
    // ... existing routes
```

Follow same pattern. Use `axum::extract::State` for shared BackendRegistry.

### Architecture Compliance

- [Source: architecture.md#API Patterns] — REST endpoint naming
- [Source: prd.md#REST API Endpoints] — Expected endpoints

### References

- [Source: crates/web/] — Existing Axum web server
- [Source: architecture.md#API Patterns] — RESTful patterns

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
