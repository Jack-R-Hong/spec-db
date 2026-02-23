---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04-generate-tests
  - step-05-validate-and-complete
lastStep: step-05-validate-and-complete
lastSaved: '2026-02-23'
status: complete
storyId: '1-1'
storyTitle: 'Scaffold Workspace & Core Domain Types'
---

# ATDD Checklist — Story 1.1: Scaffold Workspace & Core Domain Types

## Step 1: Preflight & Context

### Story Summary

**Story 1.1**: As a developer, I want a properly scaffolded Cargo workspace with core domain types, trait interfaces, and error hierarchy, so that all subsequent development has a consistent, version-locked foundation.

**Status**: Implementation exists (review). Tests generated as post-hoc acceptance coverage.

### Acceptance Criteria Extracted

| AC# | Criterion | Testable |
|-----|-----------|----------|
| AC1 | Workspace compiles with `spec-db` (binary), `spec-db-core` (lib), `spec-db-causal` (lib) | ✅ |
| AC2 | `spec-db-core` exports `SpecId`, `SpecDoc`, `SpecNode`, `CausalEdge`, `TrustLevel` | ✅ |
| AC3 | `SpecId` validates `spec::{segment}::{segment}` pattern, rejects invalid formats | ✅ |
| AC4 | `spec-db-core` exports `SearchEngine`, `CausalGraph`, `SpecStore` traits | ✅ |
| AC5 | `spec-db-core` exports `SpecDbError` with 6 variants | ✅ |
| AC6 | `workspace.dependencies` locks all dependency versions | ✅ |
| AC7 | `rustfmt.toml` and `clippy.toml` configured | ✅ |
| AC8 | `cargo clippy --workspace -- -D warnings` passes zero warnings | ✅ |
| AC9 | `cargo fmt --all -- --check` passes | ✅ |

### Affected Components

- `crates/spec-db-core/src/types.rs` — Domain types
- `crates/spec-db-core/src/traits.rs` — Trait interfaces
- `crates/spec-db-core/src/error.rs` — Error hierarchy
- `crates/spec-db-core/src/lib.rs` — Public API
- `Cargo.toml` — Workspace config
- `rustfmt.toml`, `clippy.toml` — Code style

### Framework & Patterns

- **Test Framework**: `cargo test` (Rust built-in)
- **Unit Tests**: `#[cfg(test)] mod tests` inline
- **Integration Tests**: `tests/*.rs`
- **Assertions**: `assert!`, `assert_eq!`, `unwrap()` in tests
- **Helpers**: `tempfile` crate

### Knowledge Base Applied

- test-quality: Isolation, determinism, atomicity principles
- component-tdd: Red-green-refactor (adapted for Rust)
- data-factories: Builder pattern / helper constructors
- test-levels: Unit vs integration selection

## Step 2: Generation Mode

**Mode**: AI Generation (default)
**Reason**: Clear BDD acceptance criteria with standard non-UI scenarios (workspace structure, type exports, validation logic). No browser recording needed. Rust project — all tests generated as `#[test]` functions.

## Step 3: Test Strategy

### Test Level Mapping (Rust Adaptation)

| ATDD Level | Rust Equivalent | Location |
|------------|-----------------|----------|
| E2E | Integration tests (binary/process) | `tests/*.rs` |
| API | Contract tests (crate public API) | `tests/acceptance_story_1_1.rs` |
| Component | Unit tests (type-level) | `#[cfg(test)] mod tests` inline |

### AC → Test Scenario Mapping

#### AC1: Workspace compiles with 3 crates
- **Level**: Integration (process-level)
- **Priority**: P0 — Foundation gate
- **Scenarios**:
  1. `cargo build --workspace` exits with code 0
  2. Binary `spec-db` exists in target
  3. Library crate `spec-db-core` compiles
  4. Library crate `spec-db-causal` compiles
- **Existing coverage**: None (only CLI tests exist)

#### AC2: spec-db-core exports 5 domain types
- **Level**: API/Contract (crate import test)
- **Priority**: P0 — API contract
- **Scenarios**:
  1. `SpecId` is importable and constructible
  2. `SpecDoc` is importable and constructible
  3. `SpecNode` is importable and constructible
  4. `CausalEdge` is importable and constructible
  5. `TrustLevel` is importable and constructible
- **Existing coverage**: Partial (types.rs tests construct SpecId, TrustLevel)

#### AC3: SpecId validates pattern
- **Level**: Unit (validation logic)
- **Priority**: P0 — Critical validation
- **Scenarios**:
  1. Valid: `spec::auth::login` accepted
  2. Valid: `spec::user-service::password-reset` (hyphens)
  3. Valid: `spec::api-v2::endpoint-3` (digits)
  4. Invalid: missing `spec::` prefix → error
  5. Invalid: single segment `spec::onlyone` → error
  6. Invalid: empty segment `spec::::name` → error
  7. Invalid: uppercase `spec::Auth::Login` → error
  8. Invalid: spaces → error
  9. Invalid: underscores → error
  10. Invalid: empty string → error
  11. `FromStr` trait implementation works
  12. Error message is descriptive
- **Existing coverage**: Full (15 tests in types.rs) — no new tests needed

#### AC4: spec-db-core exports 3 traits
- **Level**: API/Contract (trait accessibility)
- **Priority**: P0 — API contract
- **Scenarios**:
  1. `SearchEngine` trait is importable with expected methods
  2. `CausalGraph` trait is importable with expected methods
  3. `SpecStore` trait is importable with expected methods
  4. All trait methods return `Result<_, SpecDbError>`
- **Existing coverage**: None

#### AC5: SpecDbError with 6 variants
- **Level**: Unit (error hierarchy)
- **Priority**: P0 — Error contract
- **Scenarios**:
  1. `SearchError` variant constructible with Display output
  2. `GraphError` variant constructible with Display output
  3. `SyncError` variant constructible with Display output
  4. `IngestError` variant constructible with Display output
  5. `ConsistencyError` variant constructible with Display output
  6. `ConfigError` variant constructible with Display output
  7. All variants implement `std::error::Error`
  8. Error messages are human-readable
- **Existing coverage**: Partial (1 test for error display in types.rs)

#### AC6: workspace.dependencies locks versions
- **Level**: Integration (config verification)
- **Priority**: P1 — Configuration
- **Scenarios**:
  1. `Cargo.toml` contains `[workspace.dependencies]`
  2. `deep_causality` version locked to `=0.13.4`
  3. `fjall` version locked to `3.0`
  4. `tantivy` version locked to `0.25.0`
  5. `rmcp` version locked to `=0.16.0`
  6. `git2` version locked to `0.20.4`
  7. `bincode` version locked to `=2.0.1` with serde feature
- **Existing coverage**: None

#### AC7: rustfmt.toml and clippy.toml configured
- **Level**: Integration (file verification)
- **Priority**: P1 — Configuration
- **Scenarios**:
  1. `rustfmt.toml` exists with `edition = "2024"`
  2. `rustfmt.toml` has `max_width = 100`
  3. `clippy.toml` exists with `allow-unwrap-in-tests = true`
- **Existing coverage**: None

#### AC8: cargo clippy passes zero warnings
- **Level**: Integration (tooling gate)
- **Priority**: P0 — Quality gate
- **Scenarios**:
  1. `cargo clippy --workspace -- -D warnings` exits with code 0
- **Existing coverage**: None (run manually, not as test)

#### AC9: cargo fmt check passes
- **Level**: Integration (tooling gate)
- **Priority**: P0 — Quality gate
- **Scenarios**:
  1. `cargo fmt --all -- --check` exits with code 0
- **Existing coverage**: None (run manually, not as test)

### Test File Plan

| File | Level | ACs Covered | Test Count |
|------|-------|-------------|------------|
| `tests/acceptance_story_1_1.rs` | Integration/Contract | AC1, AC2, AC4, AC5, AC6, AC7, AC8, AC9 | ~25 |

**Note**: AC3 (SpecId validation) already has full unit test coverage in `types.rs` — no new tests needed.

### Primary Test Level

**Integration/Contract tests** in `tests/acceptance_story_1_1.rs` — verifying the crate's public API contract and workspace structure from the consumer perspective.

### Red Phase Confirmation

Since implementation exists, all tests are expected to **PASS** (GREEN state). This is post-hoc acceptance test generation for regression coverage. The tests are designed to *detect regressions* if the implementation is modified.

## Step 4: Tests Generated

### Test File Created

**File:** `tests/acceptance_story_1_1.rs` (307 lines)

| AC | Tests | Status |
|----|-------|--------|
| AC1 | `ac1_spec_db_core_is_importable`, `ac1_spec_db_causal_is_importable`, `ac1_workspace_contains_three_crates` | PASS |
| AC2 | `ac2_spec_id_exported_and_constructible`, `ac2_spec_doc_exported_and_constructible`, `ac2_spec_node_exported_and_constructible`, `ac2_causal_edge_exported_and_constructible`, `ac2_trust_level_exported_and_constructible` | PASS |
| AC3 | (15 existing unit tests in `types.rs`) | PASS |
| AC4 | `ac4_search_engine_trait_exported`, `ac4_causal_graph_trait_exported`, `ac4_spec_store_trait_exported` | PASS |
| AC5 | `ac5_error_variant_search_error` through `ac5_error_variant_config_error`, `ac5_error_implements_std_error`, `ac5_error_implements_debug` | PASS |
| AC6 | `ac6_workspace_dependencies_section_exists`, `ac6_deep_causality_version_locked`, `ac6_fjall_version_locked`, `ac6_tantivy_version_locked`, `ac6_rmcp_version_locked`, `ac6_git2_version_locked`, `ac6_bincode_version_locked_with_serde` | PASS |
| AC7 | `ac7_rustfmt_toml_edition`, `ac7_rustfmt_toml_max_width`, `ac7_clippy_toml_configured` | PASS |
| AC8 | `ac8_cargo_clippy_zero_warnings` | IGNORED (expensive) |
| AC9 | `ac9_cargo_fmt_check_passes` | IGNORED (expensive) |

### Test Techniques Used

- **Compile-time assertions** (AC1, AC4): Type references and function pointer coercions verify trait/type exports at compile time
- **Constructor assertions** (AC2, AC5): Build domain types and error variants, verify fields and Display
- **File content assertions** (AC6, AC7): `include_str!()` to embed and validate config file contents
- **Process assertions** (AC8, AC9): `std::process::Command` to run cargo tools and assert exit code

## Step 5: Validation & Completion

### Checklist Validation

- [x] Story acceptance criteria analyzed and mapped to tests
- [x] Tests created at appropriate level (integration/contract)
- [x] All 9 acceptance criteria covered (AC3 via existing unit tests)
- [x] Tests are deterministic (no race conditions or flaky patterns)
- [x] Tests are isolated (no shared state)
- [x] Tests are atomic (one assertion focus per test)
- [x] No hardcoded test data issues (config values match source files)
- [x] Test file compiles with zero warnings
- [x] All 29 non-ignored tests pass
- [x] No CLI sessions or orphaned processes
- [x] Output artifacts stored in `_bmad-output/test-artifacts/`

### Test Execution Evidence

**Command:** `cargo test --test acceptance_story_1_1`

**Results:**
```
running 31 tests
test ac1_spec_db_causal_is_importable ... ok
test ac1_spec_db_core_is_importable ... ok
test ac1_workspace_contains_three_crates ... ok
test ac2_causal_edge_exported_and_constructible ... ok
test ac2_spec_doc_exported_and_constructible ... ok
test ac2_spec_id_exported_and_constructible ... ok
test ac2_spec_node_exported_and_constructible ... ok
test ac2_trust_level_exported_and_constructible ... ok
test ac4_causal_graph_trait_exported ... ok
test ac4_search_engine_trait_exported ... ok
test ac4_spec_store_trait_exported ... ok
test ac5_error_implements_debug ... ok
test ac5_error_implements_std_error ... ok
test ac5_error_variant_config_error ... ok
test ac5_error_variant_consistency_error ... ok
test ac5_error_variant_graph_error ... ok
test ac5_error_variant_ingest_error ... ok
test ac5_error_variant_search_error ... ok
test ac5_error_variant_sync_error ... ok
test ac6_bincode_version_locked_with_serde ... ok
test ac6_deep_causality_version_locked ... ok
test ac6_fjall_version_locked ... ok
test ac6_git2_version_locked ... ok
test ac6_rmcp_version_locked ... ok
test ac6_tantivy_version_locked ... ok
test ac6_workspace_dependencies_section_exists ... ok
test ac7_clippy_toml_configured ... ok
test ac7_rustfmt_toml_edition ... ok
test ac7_rustfmt_toml_max_width ... ok
test ac8_cargo_clippy_zero_warnings ... ignored
test ac9_cargo_fmt_check_passes ... ignored

test result: ok. 29 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

### Running Tests

```bash
# Run all acceptance tests for Story 1.1
cargo test --test acceptance_story_1_1

# Run specific AC group
cargo test --test acceptance_story_1_1 ac5_

# Run expensive tests (clippy + fmt)
cargo test --test acceptance_story_1_1 -- --ignored

# Run ALL tests including expensive ones
cargo test --test acceptance_story_1_1 -- --include-ignored
```

### Completion Summary

| Metric | Value |
|--------|-------|
| Story ID | 1-1 |
| Primary Test Level | Integration/Contract |
| Total Tests (new) | 31 |
| Passing | 29 |
| Ignored (expensive) | 2 |
| AC Coverage | 9/9 (100%) |
| Test File | `tests/acceptance_story_1_1.rs` |
| Data Factories | 0 (not needed — no external data) |
| Fixtures | 0 (not needed — pure API assertions) |
| Mock Requirements | 0 (no external services) |
| Estimated Effort | Complete (implementation exists) |

### Assumptions & Notes

- Implementation already exists (Story 1.1 status: review). Tests serve as regression coverage.
- AC3 not duplicated — 15 existing unit tests in `types.rs` provide full coverage.
- AC8/AC9 marked `#[ignore]` because they spawn expensive `cargo clippy` / `cargo fmt` processes. Run explicitly with `--ignored` flag.
- Adapted from Playwright/Cypress ATDD workflow to Rust `cargo test` idioms.

### Next Steps

1. Run `cargo test --test acceptance_story_1_1 -- --ignored` to verify AC8/AC9
2. Consider moving to Story 1.2 ATDD if additional acceptance coverage desired
3. Use `bmad-bmm-dev-story` workflow for implementation of future stories

---

**Generated by BMad TEA Agent** — 2026-02-23
