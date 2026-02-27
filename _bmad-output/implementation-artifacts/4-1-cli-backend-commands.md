# Story 4.1: CLI Backend Commands

Status: ready-for-dev

## Story

As a lattice operator,
I want CLI commands to manage search backends,
so that I can list, add, remove, check status, and rebuild backends from the terminal.

## Acceptance Criteria

1. `lattice backend list` — outputs backends with type and status
2. `lattice backend add <name> <type>` — adds backend, writes config, initializes
3. `lattice backend remove <name>` — removes from config and registry
4. `lattice backend status [name]` — health info: type, doc count, index size
5. `lattice rebuild --backend=<name>` — rebuild specific backend
6. Clear error messages for invalid inputs
7. Integration test for list/add/remove/status

## Tasks / Subtasks

- [ ] Task 1: Add `Backend` subcommand to CLI (AC: #1, #2, #3, #4)
  - [ ] Add `BackendCmd` enum to `src/main.rs` using clap derive
  - [ ] Subcommands: `List`, `Add { name, backend_type }`, `Remove { name }`, `Status { name: Option }`
- [ ] Task 2: Implement list (AC: #1)
  - [ ] Read config, display backends table
- [ ] Task 3: Implement add (AC: #2)
  - [ ] Write new entry to config file
  - [ ] Initialize backend in registry
- [ ] Task 4: Implement remove (AC: #3)
  - [ ] Remove from config file
  - [ ] Remove from registry
- [ ] Task 5: Implement status (AC: #4)
  - [ ] Show type, document count, storage path
- [ ] Task 6: Extend rebuild (AC: #5)
  - [ ] Add `--backend` flag to existing `rebuild` command
- [ ] Task 7: Integration test (AC: #6, #7)

## Dev Notes

### Existing CLI Pattern

From `src/main.rs` — uses clap derive:
```rust
#[derive(Subcommand)]
enum Commands {
    Init, Serve, Sync { ... }, Rebuild, Status,
}
```

Add `Backend(BackendCmd)` variant. Follow existing pattern.

### Architecture Compliance

- [Source: architecture.md#API Patterns] — CLI command pattern with clap derive
- [Source: prd.md#CLI Commands] — Expected command structure

### References

- [Source: src/main.rs] — CLI entry point
- [Source: architecture.md#API Patterns] — BackendCmd enum design

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
