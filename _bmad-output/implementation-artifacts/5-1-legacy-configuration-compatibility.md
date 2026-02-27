# Story 5.1: Legacy Configuration Compatibility

Status: ready-for-dev

## Story

As an existing lattice user,
I want my current config (no search_backends section) to work without changes,
so that upgrading lattice doesn't break my existing setup.

## Acceptance Criteria

1. Config without `search_backends` → FTS-only mode
2. No vector backends initialized
3. All existing MCP tools work identically
4. `search_specs` without `mode`/`backend` params works as before
5. `lattice sync` indexes only to Tantivy
6. `lattice status` shows existing info without backend fields
7. No errors, warnings, or deprecation notices
8. Acceptance test verifies full existing workflow

## Tasks / Subtasks

- [ ] Task 1: Verify config backward compat (AC: #1, #7)
  - [ ] `search_backends: None` → no vector initialization
  - [ ] No startup errors or warnings
- [ ] Task 2: Verify sync compat (AC: #5)
  - [ ] Sync pipeline skips embedding when no vector backends
- [ ] Task 3: Verify MCP compat (AC: #3, #4)
  - [ ] Existing tool calls work unchanged
  - [ ] Missing new params default to backward-compatible values
- [ ] Task 4: Verify status output (AC: #6)
  - [ ] `lattice status` omits backend info when none configured
- [ ] Task 5: Acceptance test (AC: #8)
  - [ ] Full workflow: init → sync → search → status with legacy config
  - [ ] Compare output to pre-upgrade behavior

## Dev Notes

### This is primarily a verification story

All code enabling backward compatibility was built into Stories 1.5 (config), 1.6 (sync), and 4.3 (MCP). This story ensures the integration works end-to-end.

Key code paths to verify:
- `SpecDbConfig.search_backends` is `Option<SearchBackendsConfig>` — `None` by default
- `IngestPipeline` skips vector indexing when `embedding_provider` is `None`
- `QueryRouter.vector` is `Option<BackendRegistry>` — `None` when no backends

### CRITICAL: What NOT To Do

- Do NOT add deprecation warnings for legacy config
- Do NOT change default behavior — FTS must remain the default

### References

- [Source: crates/core/src/config.rs] — SpecDbConfig with Option<SearchBackendsConfig>
- [Source: prd.md#Migration Guide] — Backward compatibility requirements

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
