---
stepsCompleted:
  - step-01-detect-mode
  - step-02-load-context
  - step-03-risk-and-testability
  - step-04-coverage-plan
  - step-05-generate-output
lastStep: 'step-05-generate-output'
lastSaved: '2026-02-23'
---

# Test Design: All Epics - spec-db

**Date:** 2026-02-23
**Author:** Jack
**Status:** Draft

---

## Executive Summary

**Scope:** Full test design covering all 7 epics (18 stories) for spec-db MVP.

**Risk Summary:**

- Total risks identified: 16
- High-priority risks (>=6): 7
- Critical categories: TECH (4), DATA (3), SEC (1), BUS (2), PERF (2), OPS (2)

**Coverage Summary:**

- P0 scenarios: 28 (~15-25 hours)
- P1 scenarios: 35 (~15-25 hours)
- P2 scenarios: 22 (~8-15 hours)
- P3 scenarios: 10 (~3-6 hours)
- **Total effort**: ~40-70 hours (~1-2 weeks)

---

## Not in Scope

| Item | Reasoning | Mitigation |
|------|-----------|------------|
| **Multi-repo federation** | Phase 3 feature, not in MVP | Deferred to post-MVP architecture |
| **AI-inferred causal edges** | Phase 2 feature | Only human-curated edges (trust=1.0) in MVP |
| **Concurrent agent access** | Single-process design, deferred | Document as known limitation |
| **Browser/UI testing** | No UI component; CLI + MCP server | CLI integration tests cover user-facing surface |
| **Windows-specific testing** | Cross-platform is NFR but low-risk for pure Rust | CI matrix covers Linux/macOS/Windows |
| **Load/stress testing** | Single-user scale (100-500 specs), low concurrency risk | Performance assertions embedded in integration tests |

---

## Risk Assessment

### High-Priority Risks (Score >=6)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner | Timeline |
|---------|----------|-------------|------|--------|-------|------------|-------|----------|
| R-001 | TECH | DeepCausality 0.13.4 API stability and in-memory graph correctness | 2 | 3 | 6 | Comprehensive graph operation tests; petgraph fallback path exists | Dev | Epic 1 |
| R-002 | DATA | Cross-store consistency drift between Tantivy and Fjall | 2 | 3 | 6 | SHA + doc count verification on startup and after every sync | Dev | Epic 7 |
| R-003 | TECH | rmcp 0.16.0 rapid API evolution; MCP protocol compliance | 2 | 3 | 6 | Pin exact version; protocol conformance tests; trait boundaries isolate blast radius | Dev | Epic 6 |
| R-004 | SEC | Streamable-HTTP bearer token auth bypass when HTTP enabled | 2 | 3 | 6 | Auth rejection test suite; default-deny verification; no-config = no network | Dev | Epic 6 |
| R-005 | DATA | Incremental sync partial failure leaves stores inconsistent | 2 | 3 | 6 | Fault injection tests; auto-escalation to full rebuild on count divergence | Dev | Epic 4 |
| R-006 | TECH | Git rename detection (-M flag) misses complex rename scenarios | 2 | 3 | 6 | Rename-specific integration tests covering path changes with content changes | Dev | Epic 4 |
| R-007 | DATA | Spec ID uniqueness not enforced across concurrent ingestion | 2 | 3 | 6 | Duplicate rejection tests; SpecId validation unit tests; ingestion pipeline atomicity | Dev | Epic 3 |

### Medium-Priority Risks (Score 3-5)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner |
|---------|----------|-------------|------|--------|-------|------------|-------|
| R-008 | BUS | Query classification misclassifies edge cases (hybrid vs search) | 2 | 2 | 4 | Classification boundary tests with diverse query patterns | Dev |
| R-009 | TECH | Fjall v3 cross-keyspace batch API atomicity not proven | 2 | 2 | 4 | Atomic write tests, failure rollback verification | Dev |
| R-010 | PERF | Startup time exceeds 1s NFR with 100+ spec graph load | 2 | 2 | 4 | Performance benchmark at 100+ spec scale | Dev |
| R-011 | BUS | MCP tool error responses deviate from consistent JSON format (F2) | 2 | 2 | 4 | Error format contract tests across all 7 MCP tools | Dev |
| R-012 | PERF | OTel tracing overhead degrades search latency (<10ms target) | 2 | 2 | 4 | Performance tests with OTel enabled vs disabled | Dev |

### Low-Priority Risks (Score 1-3)

| Risk ID | Category | Description | Prob | Impact | Score | Action |
|---------|----------|-------------|------|--------|-------|--------|
| R-013 | TECH | serde_yml API compatibility (replacing deprecated serde_yaml) | 1 | 2 | 2 | Monitor |
| R-014 | OPS | Atomic rebuild temp-dir swap failure during full rebuild | 1 | 3 | 3 | Monitor |
| R-015 | BUS | Zero-result fallback to causal context quality | 1 | 2 | 2 | Monitor |
| R-016 | OPS | Auto-rebuild infinite loop on persistent cross-store drift | 1 | 3 | 3 | Monitor |

### Risk Category Legend

- **TECH**: Technical/Architecture (flaws, integration, scalability)
- **SEC**: Security (access controls, auth, data exposure)
- **PERF**: Performance (SLA violations, degradation, resource limits)
- **DATA**: Data Integrity (loss, corruption, inconsistency)
- **BUS**: Business Impact (UX harm, logic errors, revenue)
- **OPS**: Operations (deployment, config, monitoring)

---

## Entry Criteria

- [x] Workspace scaffolded with all 7 crates (`core`, `causal`, `search`, `ingest`, `router`, `mcp`, root binary)
- [x] All dependency versions locked in `workspace.dependencies`
- [x] `cargo build --workspace` compiles successfully
- [x] `cargo clippy --workspace -- -D warnings` passes
- [ ] Test fixtures available (valid/invalid spec markdown files)
- [ ] Tempdir-based test infrastructure for Fjall and Tantivy stores

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (>=95%)
- [ ] No open high-priority risks (>=6) without documented mitigation
- [ ] Performance NFRs validated: search <10ms, traversal <50ms, startup <1s, rebuild <5s
- [ ] Cross-store consistency verified after every sync operation
- [ ] CLI commands execute without panics on valid and invalid inputs

---

## Test Coverage Plan

> **Note:** P0/P1/P2/P3 = risk-based priority classification, NOT execution timing. All tests run on PR unless explicitly deferred.

### P0 (Critical)

**Criteria:** Blocks core functionality + High risk (>=6) + No workaround

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 1.1-UNIT-001 | SpecId validates `spec::{segment}::{segment}` pattern | Unit | R-007 | Reject invalid formats, empty segments, missing prefix |
| 1.1-UNIT-002 | SpecDbError hierarchy has all 6 variants | Unit | - | Verify error type matching |
| 1.2-INT-001 | Fjall node roundtrip (put + get) with bincode | Integration | R-009 | Existing: `node_roundtrip` |
| 1.2-INT-002 | Fjall edge roundtrip with composite key | Integration | R-009 | Existing: `edge_roundtrip` |
| 1.2-INT-003 | Fjall cross-keyspace batch atomicity (node + edges) | Integration | R-009 | Existing: `put_node_with_edges_atomic` |
| 1.2-INT-004 | Fjall data survives process restart | Integration | R-001 | Existing: `reopen_durability` |
| 1.3-INT-001 | DeepCausality graph loads all nodes/edges from Fjall on startup | Integration | R-001 | Verify in-memory state matches persisted state |
| 1.3-INT-002 | `depends_on` edges auto-created with trust=1.0 | Integration | R-001 | Verify edge type and trust level |
| 1.4-INT-001 | `trace_impact` finds transitive downstream dependents | Integration | R-001 | A->B->C: trace_impact(C) returns {B, A} |
| 1.4-INT-002 | `find_dependencies` finds transitive upstream deps | Integration | R-001 | A->B->C: find_dependencies(A) returns {B, C} |
| 1.4-INT-003 | Depth-limited traversal respects max depth | Integration | R-001 | depth=2 on 5-node chain returns only 2 hops |
| 2.1-INT-001 | Tantivy schema has correct field types (STRING/TEXT/JSON) | Integration | - | Existing: `schema_has_expected_fields` |
| 2.1-INT-002 | Add + commit + search roundtrip | Integration | - | Existing: `add_and_commit_roundtrip` |
| 2.2-INT-001 | Title-match ranks higher than body-match (BM25 boost) | Integration | - | Existing: `title_match_ranks_higher` |
| 2.2-INT-002 | Tag filter returns exact matches only | Integration | - | Existing: `tag_filter_exact_match` |
| 3.1-UNIT-001 | Frontmatter parsing extracts all 7 fields correctly | Unit | R-007 | id, title, version, tags, depends_on, owner, created |
| 3.2-INT-001 | Ingestion pipeline writes to both search + graph atomically | Integration | R-007 | Existing: `ingest_valid_spec` |
| 3.2-INT-002 | Duplicate spec ID rejected without modifying stores | Integration | R-007 | Existing: `ingest_duplicate_rejected` |
| 4.1-INT-001 | Full rebuild ingests all specs from git tree walk | Integration | R-005 | Existing: `full_rebuild_ingests_specs` |
| 4.1-INT-002 | Full rebuild is idempotent (identical results on re-run) | Integration | R-005 | Existing: `full_rebuild_idempotent` |
| 4.2-INT-001 | Incremental sync detects modified files | Integration | R-005 | Existing: `incremental_sync_modified` |
| 4.2-INT-002 | Incremental sync removes deleted specs from both stores | Integration | R-005 | Existing: `incremental_sync_deleted` |
| 4.2-INT-003 | Doc count divergence auto-escalates to full rebuild | Integration | R-005 | Existing: `incremental_divergence_escalates` |
| 5.1-INT-001 | Search-only query classified as "search" intent | Integration | R-008 | Existing: `search_only_query` |
| 5.1-INT-002 | Causal query classified as "causal" intent | Integration | R-008 | Existing: `causal_only_query` |
| 6.2-INT-001 | MCP server advertises name, version, capabilities | Integration | R-003 | Existing: `server_info_contains_name_and_capabilities` |
| 7.1-INT-001 | Cross-store consistency passes after full rebuild | Integration | R-002 | Existing: `consistency_after_rebuild` |
| 7.1-INT-002 | SHA mismatch detected as drift | Integration | R-002 | Verify ConsistencyStatus::Drifted on mismatched SHA |

**Total P0**: 28 tests, ~15-25 hours

### P1 (High)

**Criteria:** Important features + Medium risk (3-5) + Common workflows

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 1.2-INT-005 | Metadata roundtrip (last_sync_sha, doc_count) | Integration | - | Existing: `metadata_roundtrip` |
| 1.2-INT-006 | iter_nodes and iter_edges return all stored items | Integration | - | Existing: `iter_edges_returns_all`, `iter_nodes_returns_all` |
| 1.2-INT-007 | Get nonexistent node/edge returns None | Integration | - | Existing: `get_nonexistent_returns_none` |
| 1.3-INT-003 | Node view returns all inbound + outbound edges (FR10) | Integration | R-001 | Verify complete edge set for a node |
| 1.4-INT-004 | Traversal on nonexistent spec ID returns GraphError | Integration | - | Error path coverage |
| 2.1-INT-003 | Remove + commit deletes document from index | Integration | - | Existing: `remove_and_commit` |
| 2.1-INT-004 | Multiple docs added in single commit | Integration | - | Existing: `multiple_updates_single_commit` |
| 2.2-INT-003 | Search returns empty set for no match (not error) | Integration | - | Existing: `search_returns_empty_for_no_match` |
| 2.2-INT-004 | Search results include score > 0 | Integration | - | Existing: `search_results_include_score` |
| 2.2-PERF-001 | Search <10ms across 100+ specs (NFR1) | Integration | R-012 | Existing: `search_perf_100_specs` |
| 3.1-UNIT-002 | Invalid SpecId format returns IngestError | Unit | - | Missing `spec::` prefix, empty segments |
| 3.1-UNIT-003 | Missing required frontmatter fields returns IngestError | Unit | - | No id, no title |
| 3.1-UNIT-004 | Markdown file without frontmatter returns IngestError | Unit | - | Edge case |
| 3.2-INT-003 | Forward reference edges resolve when target ingested later | Integration | - | Existing: `ingest_forward_reference` |
| 3.2-PERF-001 | Single spec ingestion <100ms (NFR7) | Integration | - | Existing: `ingest_perf_single_spec` |
| 4.1-INT-003 | Full rebuild replaces stale specs (no remnants) | Integration | - | Existing: `full_rebuild_replaces_stale` |
| 4.1-INT-004 | Full rebuild records SHA and doc count in both stores | Integration | - | Existing: `full_rebuild_records_metadata` |
| 4.2-INT-004 | Incremental sync adds new specs correctly | Integration | - | Existing: `incremental_sync_added` |
| 4.2-INT-005 | Incremental sync with no changes is a no-op | Integration | - | Existing: `incremental_sync_no_changes` |
| 4.2-INT-006 | Incremental sync with no prior SHA triggers full rebuild | Integration | - | Existing: `incremental_sync_no_prior_sha_triggers_full_rebuild` |
| 5.1-INT-003 | Hybrid query classified correctly | Integration | R-008 | Existing: `hybrid_query` |
| 5.2-INT-001 | Both engines empty returns clear message | Integration | - | Existing: `both_empty_returns_clear_message` |
| 5.2-INT-002 | Hybrid result combines search hits with causal context | Integration | R-008 | Verify composed response |
| 5.2-INT-003 | Zero search results falls back to causal context (FR12) | Integration | R-015 | Verify fallback behavior |
| 6.1-CLI-001 | `spec-db init` creates expected directory structure | E2E/CLI | - | Existing: `init_creates_structure` |
| 6.1-CLI-002 | `spec-db init` is idempotent (no overwrite) | E2E/CLI | - | Existing: `init_idempotent` |
| 6.2-INT-002 | MCP tool input deserialization works | Integration | R-003 | Existing: `tool_input_deserialization_works` |
| 6.3-INT-001 | Resource URI parsing (spec://, graph://overview, graph://node/) | Integration | R-003 | Existing: `resource_uri_parsing_works` |
| 7.1-INT-003 | Doc count mismatch detected as drift | Integration | R-002 | Verify doc count comparison |
| 7.1-INT-004 | Auto-rebuild triggered on drift detection | Integration | R-002 | Verify escalation path |

**Total P1**: 30 tests, ~15-25 hours

### P2 (Medium)

**Criteria:** Secondary features + Low risk (1-2) + Edge cases

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 1.1-UNIT-003 | SpecDoc, SpecNode, CausalEdge, TrustLevel serde roundtrip | Unit | - | Bincode serialization correctness |
| 1.2-INT-008 | SpecStore trait list_ids returns all stored spec IDs | Integration | - | Existing: `spec_store_list_ids` |
| 1.3-INT-004 | Startup <1s for 100+ spec graph (NFR4) | Integration | R-010 | Performance benchmark |
| 1.4-PERF-001 | Traversal <50ms for 100+ spec graph (NFR2) | Integration | R-010 | Performance benchmark |
| 3.2-INT-004 | Ingestion pipeline remove_spec cleans both stores | Integration | - | Existing: `ingest_removes_spec` |
| 4.1-PERF-001 | Full rebuild <5s for 100+ specs (NFR5) | Integration | R-010 | Performance benchmark |
| 4.2-INT-007 | Rename detection preserves spec without duplication (FR22) | Integration | R-006 | NEW: Git rename scenario |
| 4.2-PERF-001 | Incremental sync <2s for changed files (NFR6) | Integration | - | Performance benchmark |
| 5.1-PERF-001 | Classification overhead <5ms (NFR3) | Integration | - | Existing: `classification_perf` |
| 6.1-UNIT-001 | Config parsing with valid YAML | Unit | - | Existing: config.rs tests |
| 6.1-UNIT-002 | Config with missing optional fields uses defaults | Unit | - | Sensible defaults |
| 6.1-UNIT-003 | Config with missing required fields returns ConfigError | Unit | - | Error path |
| 6.2-INT-003 | MCP search_specs tool returns JSON with id, title, score, snippet | Integration | R-011 | Verify F1 response format |
| 6.2-INT-004 | MCP trace_impact tool returns JSON with node, edges | Integration | R-011 | Verify F1 response format |
| 6.2-INT-005 | MCP tool error returns consistent JSON format (F2) | Integration | R-011 | Verify error_type, message, context |
| 6.2-PERF-001 | MCP tool response <100ms end-to-end (NFR8) | Integration | R-012 | Performance benchmark |
| 6.3-INT-002 | HTTP without valid bearer token returns 401 (NFR24) | Integration | R-004 | Auth enforcement |
| 6.3-INT-003 | HTTP disabled by default = no network surface (NFR23) | Integration | R-004 | Default-deny |
| 6.4-CLI-001 | `spec-db sync` command parsed correctly | E2E/CLI | - | Existing: `sync_command_is_parsed` |
| 6.4-CLI-002 | `spec-db rebuild` command parsed correctly | E2E/CLI | - | Existing: `rebuild_command_is_parsed` |
| 6.4-CLI-003 | `spec-db status` shows doc count, SHA, consistency | E2E/CLI | - | Existing: `status_command_is_parsed` |
| 7.1-INT-005 | Auto-rebuild terminates (no infinite loop) on persistent drift | Integration | R-016 | Verify escalation cap |

**Total P2**: 22 tests, ~8-15 hours

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + Benchmarks

| Test ID | Requirement | Test Level | Notes |
|---------|-------------|------------|-------|
| 1.1-UNIT-004 | Trait definitions compile (SearchEngine, CausalGraph, SpecStore) | Unit | Structural check |
| 2.2-PERF-002 | Search with 500+ specs performance profile | Integration | Scalability exploration |
| 3.1-UNIT-005 | Frontmatter with unknown fields preserved in meta JSON | Unit | F3 pattern |
| 4.1-PERF-002 | Full rebuild with 500+ specs performance profile | Integration | Scalability exploration |
| 6.2-INT-006 | MCP `add_spec` and `sync` tools execute correctly | Integration | Full tool coverage |
| 6.3-INT-004 | `spec://{id}` resource returns full spec content | Integration | Resource handler test |
| 6.3-INT-005 | `graph://overview` returns stats + disconnected clusters | Integration | Resource handler test |
| 6.3-INT-006 | `graph://node/{id}` returns node with edges | Integration | Resource handler test |
| 7.2-INT-001 | OTel span emitted for search query | Integration | OTel instrumentation |
| 7.2-INT-002 | No OTel export when not configured (NFR25) | Integration | Default-off verification |

**Total P3**: 10 tests, ~3-6 hours

---

## Execution Strategy

**Philosophy:** Run everything on PR unless expensive or long-running. The full Rust test suite should complete in <5 minutes with `cargo test --workspace`.

| Trigger | What Runs | Duration |
|---------|-----------|----------|
| **Every PR** | `cargo test --workspace` (all unit + integration tests) | ~2-5 min |
| **Nightly** | Performance benchmarks at 100+ / 500+ spec scale | ~5-10 min |
| **Weekly** | Full rebuild idempotency stress test, cross-platform CI matrix | ~15-30 min |

Rust's built-in test framework with `#[test]` and `cargo test` parallelization handles the full suite efficiently. No test sharding needed at this scale.

---

## Resource Estimates

### Test Development Effort

| Priority | Count | Est. Hours | Notes |
|----------|-------|------------|-------|
| P0 | 28 | ~15-25 | Many already exist; new tests focus on error paths and edge cases |
| P1 | 30 | ~15-25 | Mix of existing and new; moderate setup complexity |
| P2 | 22 | ~8-15 | Performance benchmarks, secondary paths |
| P3 | 10 | ~3-6 | Exploratory, OTel verification |
| **Total** | **90** | **~40-70** | **~1-2 weeks** |

### Prerequisites

**Test Data:**
- Spec markdown fixtures (valid, invalid ID, missing fields, multi-depends) - already in `crates/ingest/tests/fixtures/`
- Tempdir-based store setup functions (Fjall, Tantivy)
- Git repository fixtures for sync tests (using `git2::Repository::init`)

**Tooling:**
- `tempfile` crate for isolated test directories
- `git2` crate for programmatic git operations in tests
- `cargo test --workspace` as the single test runner

**Environment:**
- Local development machine (no external services)
- CI: GitHub Actions matrix (Linux, macOS, Windows) + MSRV + stable

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate**: 100% (no exceptions)
- **P1 pass rate**: >=95% (waivers required for failures)
- **P2/P3 pass rate**: >=90% (informational)
- **High-risk mitigations**: 100% complete or approved waivers

### Coverage Targets

- **Critical paths (search, graph, sync)**: >=80%
- **Security scenarios (HTTP auth)**: 100%
- **Business logic (ingestion, routing)**: >=70%
- **Edge cases (error paths, empty inputs)**: >=50%

### Non-Negotiable Requirements

- [ ] All P0 tests pass
- [ ] No high-risk (>=6) items unmitigated
- [ ] Performance NFRs validated (search <10ms, traversal <50ms, startup <1s)
- [ ] Cross-store consistency verified after every sync operation
- [ ] No `unwrap()` in library code (clippy enforced)

---

## Mitigation Plans

### R-001: DeepCausality API Stability (Score: 6)

**Mitigation Strategy:**
1. Verify DeepCausality 0.13.4 API with comprehensive graph operation tests (add node, add edge, traverse, load)
2. Maintain petgraph fallback implementation behind trait boundary (`CausalGraph` trait in core)
3. Integration tests that exercise full graph lifecycle (create -> persist -> reload -> traverse)

**Owner:** Dev
**Timeline:** Epic 1 completion
**Status:** Partially mitigated (engine.rs unit tests exist, need integration tests for full lifecycle)
**Verification:** All 1.3-* and 1.4-* tests pass; graph loads from Fjall in <1s

### R-002: Cross-Store Consistency Drift (Score: 6)

**Mitigation Strategy:**
1. SHA + doc count comparison on every startup and post-sync
2. Auto-escalation to full rebuild on count divergence
3. Termination guard on persistent drift (no infinite retry loop)

**Owner:** Dev
**Timeline:** Epic 7 completion
**Status:** Partially mitigated (consistency_after_rebuild test exists)
**Verification:** 7.1-INT-001 through 7.1-INT-005 all pass; drift scenarios tested

### R-003: rmcp 0.16.0 API Evolution (Score: 6)

**Mitigation Strategy:**
1. Pin exact rmcp version in `workspace.dependencies`
2. Trait boundary (`ServerHandler` impl) isolates rmcp API changes
3. MCP protocol conformance tests verify tool discovery, call, response format

**Owner:** Dev
**Timeline:** Epic 6 completion
**Status:** Planned
**Verification:** 6.2-* and 6.3-* tests pass; MCP tools discoverable and callable

### R-004: HTTP Transport Auth Bypass (Score: 6)

**Mitigation Strategy:**
1. Default config has no HTTP transport (stdio only = zero network surface)
2. When HTTP enabled, bearer token auth is mandatory
3. Test: requests without valid token return 401

**Owner:** Dev
**Timeline:** Epic 6 completion (Story 6.3)
**Status:** Planned
**Verification:** 6.3-INT-002 and 6.3-INT-003 pass; no unauthenticated access possible

### R-005: Incremental Sync Partial Failure (Score: 6)

**Mitigation Strategy:**
1. Doc count sanity check after every incremental sync
2. Auto-escalation to full rebuild on count divergence
3. Full rebuild uses temp-dir-then-swap for atomicity

**Owner:** Dev
**Timeline:** Epic 4 completion
**Status:** Partially mitigated (divergence_escalates test exists)
**Verification:** 4.2-INT-003 passes; stores are consistent after any sync mode

### R-006: Git Rename Detection Reliability (Score: 6)

**Mitigation Strategy:**
1. Use git diff `-M` flag for rename detection
2. Integration tests with renamed files (path change with/without content change)
3. Fall back to full rebuild if rename detection fails

**Owner:** Dev
**Timeline:** Epic 4 completion (Story 4.2)
**Status:** Planned (no rename-specific test exists yet)
**Verification:** 4.2-INT-007 passes; renamed specs re-indexed without duplication

### R-007: Spec ID Uniqueness Enforcement (Score: 6)

**Mitigation Strategy:**
1. SpecId validated at ingestion boundary (parser level)
2. Duplicate check before write to either store
3. Atomic rejection: neither store modified on duplicate

**Owner:** Dev
**Timeline:** Epic 3 completion
**Status:** Mitigated (ingest_duplicate_rejected test exists)
**Verification:** 3.2-INT-002 passes; no partial writes on duplicate

---

## Assumptions and Dependencies

### Assumptions

1. DeepCausality 0.13.4 API is stable enough for MVP use (petgraph fallback documented)
2. Fjall 3.0.x cross-keyspace batch API works as documented
3. rmcp 0.16.0 `#[tool]` macro API is stable for the duration of MVP development
4. All tests can run without network access (local-only, tempdir-based)
5. 100+ spec scale is sufficient for performance NFR validation

### Dependencies

1. All 7 crates compile and link correctly - Required before integration tests
2. Test fixture spec files available in `crates/ingest/tests/fixtures/` - Required for ingestion tests
3. `tempfile` crate for isolated test directories - Already in dev-dependencies
4. `git2` crate for programmatic git operations - Required for sync tests

### Risks to Plan

- **Risk**: DeepCausality API changes in a patch release
  - **Impact**: Graph tests break, traversal logic needs updating
  - **Contingency**: Switch to petgraph behind `CausalGraph` trait boundary

---

## Interworking & Regression

| Service/Component | Impact | Regression Scope |
|-------------------|--------|------------------|
| **spec-db-core types** | All crates depend on SpecId, SpecDoc, error types | Core type changes require full workspace test run |
| **Tantivy index** | Search crate owns; ingest/sync write to it | Any schema change requires search + ingest tests |
| **Fjall database** | Causal crate owns; ingest/sync write to it | Any keyspace change requires causal + ingest tests |
| **Git repository** | Ingest crate reads via git2 | Sync tests use fixture repos; git2 version changes need validation |
| **MCP protocol** | MCP crate exposes tools/resources | rmcp version changes need full MCP integration test run |

---

## Follow-on Workflows (Manual)

- Run `*atdd` to generate failing P0 tests (separate workflow; not auto-run).
- Run `*automate` for broader coverage once implementation exists.

---

## Appendix

### Knowledge Base References

- `risk-governance.md` - Risk classification framework (probability x impact scoring, gate decisions)
- `probability-impact.md` - Risk scoring methodology (1-3 scales, DOCUMENT/MONITOR/MITIGATE/BLOCK thresholds)
- `test-levels-framework.md` - Test level selection (unit/integration/E2E decision matrix)
- `test-priorities-matrix.md` - P0-P3 prioritization (risk score to priority mapping)

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md`
- Epics: `_bmad-output/planning-artifacts/epics.md`
- Architecture: `_bmad-output/planning-artifacts/architecture.md`
- Sprint Status: `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Existing Test Inventory (Baseline)

| Crate | File | Test Count | Coverage Focus |
|-------|------|------------|----------------|
| spec-db-core | types.rs, config.rs | ~10 | SpecId validation, config parsing |
| spec-db-causal | engine.rs, integration.rs | ~12 | Fjall roundtrip, durability, atomic writes |
| search | integration.rs | ~9 | Schema, search ops, title boost, tag filter, perf |
| ingest | consistency.rs, integration.rs | ~13 | Ingestion pipeline, sync (full + incremental), consistency |
| router | classifier.rs, lib.rs, integration.rs | ~7 | Query classification, routing, perf |
| mcp | integration.rs | ~3 | Deserialization, URI parsing, server info |
| Root binary | integration.rs | ~5 | CLI command parsing (init, sync, rebuild, status) |
| **Total** | | **~59** | |

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `_bmad/tea/testarch/test-design`
**Version**: 5.0 (Step-File Architecture)
