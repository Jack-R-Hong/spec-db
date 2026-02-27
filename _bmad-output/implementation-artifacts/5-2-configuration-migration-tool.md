# Story 5.2: Configuration Migration Tool

Status: ready-for-dev

## Story

As an existing lattice user,
I want a `lattice migrate` command to upgrade my config schema,
so that I can adopt the new features incrementally.

## Acceptance Criteria

1. `lattice migrate` detects current config schema version
2. Already up-to-date → prints "Config is already current", exits
3. Creates backup: `config.yaml.bak`
4. Adds minimal `search_backends` with `default: tantivy` and FTS backend entry
5. Preserves all existing values (specs_dir, data_dir, transport, telemetry)
6. Prints summary of changes
7. Migrated config passes validation
8. Integration test: legacy → new preserves all fields

## Tasks / Subtasks

- [ ] Task 1: Add `Migrate` subcommand to CLI (AC: #1)
  - [ ] Add `Migrate` variant to `Commands` enum
- [ ] Task 2: Implement migration detection (AC: #1, #2)
  - [ ] Load config, check if `search_backends` exists
  - [ ] If exists → already current
- [ ] Task 3: Implement migration (AC: #3, #4, #5, #6)
  - [ ] Create backup file
  - [ ] Add `search_backends: { default: tantivy, backends: [{ name: tantivy, type: fts }] }`
  - [ ] Write updated YAML preserving existing fields
  - [ ] Print summary
- [ ] Task 4: Validation (AC: #7)
  - [ ] Load migrated config through `load_config()` to verify
- [ ] Task 5: Integration test (AC: #8)
  - [ ] Start with legacy config, run migrate, verify output

## Dev Notes

### YAML Manipulation

Use `serde_yml` to load → modify → serialize. This preserves structure but may reorder keys. Alternative: string manipulation to append section. Recommend: load as typed, add defaults, serialize back.

### Architecture Compliance

- [Source: prd.md#Migration Guide] — Expected migration steps
- [Source: prd.md#Installation & Setup] — `lattice migrate` command

### References

- [Source: src/main.rs] — CLI commands
- [Source: crates/core/src/config.rs] — Config types
- [Source: prd.md#Migration Guide] — Migration requirements

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
