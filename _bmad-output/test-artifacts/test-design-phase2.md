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

# Test Design: Phase 2 — Self-Growing Intelligence & Web UI

**Date:** 2026-02-23
**Author:** Jack
**Status:** Draft

---

## Executive Summary

**Scope:** Epic-level test design covering Phase 2 Epics 8-12 (15 stories, 29 FRs, 7 NFRs).

**Risk Summary:**

- Total risks identified: 14
- High-priority risks (>=6): 5
- Critical categories: TECH (4), SEC (2), DATA (3), PERF (2), BUS (2), OPS (1)

**Coverage Summary:**

- P0 scenarios: 32 (~20-35 hours)
- P1 scenarios: 28 (~15-25 hours)
- P2 scenarios: 18 (~6-12 hours)
- P3 scenarios: 8 (~2-4 hours)
- **Total effort**: ~45-75 hours (~1-2 weeks)

---

## Not in Scope

| Item | Reasoning | Mitigation |
|------|-----------|------------|
| **Browser E2E tests (Playwright)** | Svelte UI is compiled to static assets; backend is the testing boundary. Frontend is SPA with no SSR. | REST API integration tests cover all data flows. Manual verification for visual/interaction. |
| **Multi-user concurrent web sessions** | Single-user desktop tool (NFR40: desktop-only, 1024x768 min) | Write-back pipeline serializes via Mutex; no concurrent user scenario. |
| **Cross-browser compatibility** | NFR41 lists Chrome/Firefox/Safari/Edge but this is manual QA scope | Document as manual checklist. |
| **WCAG accessibility (NFR39)** | Requires browser-based automated tooling (axe/lighthouse) | Deferred to separate accessibility audit. |
| **Load/stress testing** | Single-user tool with 100-500 spec scale | Performance assertions embedded in integration tests. |
| **Phase 1 regression** | Already covered by existing test-design-epic-all.md (90 scenarios) | Phase 1 tests run on every PR via `cargo test --workspace`. |

---

## Risk Assessment

### High-Priority Risks (Score >=6)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner | Timeline |
|---------|----------|-------------|------|--------|-------|------------|-------|----------|
| R2-001 | TECH | Git write-back pipeline corrupts spec YAML frontmatter (line-based manipulation vs structured parse) | 2 | 3 | 6 | Roundtrip tests: parse→modify→parse; property-based fuzzing on frontmatter edge cases (multiline values, special chars, empty fields) | Dev | Epic 12 |
| R2-002 | DATA | Git revert (undo) fails on merge commits or amended history, leaving inconsistent state | 2 | 3 | 6 | Test undo against linear history, amended commits, and empty repos; verify `cleanup_state()` called after revert | Dev | Epic 12 |
| R2-003 | SEC | Web UI on `0.0.0.0` without bearer token exposes write-back pipeline to unauthenticated requests | 2 | 3 | 6 | Auth middleware rejection tests; verify default `127.0.0.1` binding; verify 401 on missing/invalid token when exposed | Dev | Epic 11 |
| R2-004 | DATA | CSM cycle detection false negatives — DeepCausality graph state diverges from Fjall persisted state after AI edge insertion | 2 | 3 | 6 | Verify in-memory graph matches Fjall after every `add_causal_link`; test cycle detection on 3-5 node cycles; test disconnected subgraph edge insertion | Dev | Epic 8 |
| R2-005 | TECH | AI-inferred edge export to `.lattice/edges.yaml` races with `add_causal_link` — atomic write (temp+rename) fails on Windows or concurrent calls | 2 | 3 | 6 | Test atomic write under sequential rapid calls; verify partial-write recovery; test empty-file and missing-file scenarios | Dev | Epic 9 |

### Medium-Priority Risks (Score 3-5)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner |
|---------|----------|-------------|------|--------|-------|------------|-------|
| R2-006 | TECH | `rust-embed` debug vs release behavior divergence — filesystem reads in debug, embedded in release | 2 | 2 | 4 | Test asset serving in both modes; verify fallback behavior | Dev |
| R2-007 | PERF | Graph render >1s for 100+ nodes (NFR33) due to dagre layout computation | 2 | 2 | 4 | Performance benchmark at 100/200/500 node scale | Dev |
| R2-008 | BUS | MCP Prompt resolution returns stale data after sync — prompt reads from engine cache, not fresh index | 2 | 2 | 4 | Test prompt resolution after sync; verify data freshness | Dev |
| R2-009 | TECH | `serde` bincode roundtrip breaks with new `EdgeType`/`EdgeOrigin` enum variants if existing data lacks them | 2 | 2 | 4 | Migration test: deserialize Phase 1 data with Phase 2 schema; verify `#[serde(default)]` handles missing fields | Dev |
| R2-010 | BUS | `promote_edge` / `reject_edge` on non-existent or already-promoted edge returns wrong error type | 1 | 3 | 3 | Error contract tests for all edge lifecycle MCP tools | Dev |
| R2-011 | DATA | Write-back pipeline doesn't re-sync after commit — graph shows stale data until manual sync | 2 | 2 | 4 | Verify automatic re-sync after write-back; test graph state reflects committed changes | Dev |

### Low-Priority Risks (Score 1-2)

| Risk ID | Category | Description | Prob | Impact | Score | Action |
|---------|----------|-------------|------|--------|-------|--------|
| R2-012 | OPS | Web UI port conflict when another process uses default 3000 | 1 | 1 | 1 | Monitor |
| R2-013 | PERF | Write-back round-trip >2s (NFR35) on large repos with many specs | 1 | 2 | 2 | Monitor |
| R2-014 | BUS | Confirmation toast auto-dismiss races with user clicking Undo within 5s window | 1 | 1 | 1 | Monitor |

### Risk Category Legend

- **TECH**: Technical/Architecture (flaws, integration, scalability)
- **SEC**: Security (access controls, auth, data exposure)
- **PERF**: Performance (SLA violations, degradation, resource limits)
- **DATA**: Data Integrity (loss, corruption, inconsistency)
- **BUS**: Business Impact (UX harm, logic errors, revenue)
- **OPS**: Operations (deployment, config, monitoring)

---

## Entry Criteria

- [x] Phase 1 complete with 263 passing tests and clean clippy
- [x] All Phase 2 stories implemented and committed (62e571df)
- [x] Architecture document available with Phase 2 extension
- [x] PRD with FR47-FR75 and NFR32-NFR41 defined
- [ ] Test fixtures available (valid/invalid spec markdown files with AI edges)
- [ ] Tempdir-based test infrastructure for git write-back scenarios

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (>=95%)
- [ ] No open high-priority risks (>=6) without documented mitigation
- [ ] Performance NFRs validated: CSM <100ms, graph render <1s, write-back round-trip <2s, undo <2s
- [ ] Security: bearer token auth enforced when `web.host: 0.0.0.0`
- [ ] Write-back pipeline: frontmatter roundtrip integrity verified

---

## Test Coverage Plan

> **Note:** P0/P1/P2/P3 = risk-based priority classification, NOT execution timing. All tests run on PR unless explicitly deferred.

### P0 (Critical)

**Criteria:** Blocks core functionality + High risk (>=6) + No workaround

#### Epic 8: AI-Inferred Causal Links & Trust Scoring

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.1-UNIT-001 | `EdgeType` enum has `DependsOn`, `Constrains`, `Implements` variants | Unit | R2-009 | Verify serde roundtrip for all variants |
| 8.1-UNIT-002 | `EdgeOrigin` enum has `Human`, `Ai` variants with default `Human` | Unit | R2-009 | Verify `#[serde(default)]` behavior |
| 8.1-INT-001 | `CausalEdge` with new fields roundtrips through Fjall (bincode) | Integration | R2-009 | edge_type, trust, origin fields survive put/get |
| 8.1-INT-002 | `trace_impact` response includes `edge_type`, `trust`, `origin` for every edge | Integration | - | Verify JSON output shape |
| 8.1-INT-003 | Ingestion pipeline creates edges with `edge_type: DependsOn`, `trust: 1.0`, `origin: Human` | Integration | - | From `depends_on` frontmatter |
| 8.2-INT-001 | `add_causal_link` creates edge with `trust: 0.5`, `origin: Ai` | Integration | R2-004 | Default AI trust score |
| 8.2-INT-002 | `add_causal_link` with nonexistent source returns `not_found` error | Integration | - | Error contract |
| 8.2-INT-003 | `add_causal_link` with self-referencing source==target returns `validation_error` | Integration | - | Error contract |
| 8.2-INT-004 | `add_causal_link` with duplicate edge returns `conflict` error | Integration | - | Error contract |
| 8.3-INT-001 | CSM rejects edge that creates cycle (A→B→C→A) | Integration | R2-004 | Verify `csm_validation_failed` error with cycle path |
| 8.3-INT-002 | CSM accepts edge between disconnected subgraphs | Integration | R2-004 | Valid: connecting components |
| 8.3-INT-003 | CSM accepts valid non-cyclic edge and persists to Fjall + in-memory graph | Integration | R2-004 | Verify both stores consistent after insert |

#### Epic 9: Human Review & Edge Curation

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 9.1-INT-001 | AI-inferred edges exported to `.lattice/edges.yaml` with correct fields | Integration | R2-005 | source, target, edge_type, trust, origin, created_at |
| 9.1-INT-002 | Human-curated edges NOT included in edges.yaml export | Integration | - | Only `AiInferred` edges |
| 9.1-INT-003 | Atomic write: edges.yaml not corrupted on rapid sequential writes | Integration | R2-005 | Write-to-temp + rename pattern |
| 9.2-INT-001 | `promote_edge` changes origin to Human, trust to 1.0, removes from edges.yaml | Integration | - | Full lifecycle test |
| 9.2-INT-002 | `reject_edge` removes edge from graph, Fjall, and edges.yaml | Integration | - | Full lifecycle test |
| 9.2-INT-003 | `promote_edge` on nonexistent edge returns `not_found` | Integration | R2-010 | Error contract |
| 9.2-INT-004 | `promote_edge` on already-human edge returns `validation_error` | Integration | R2-010 | Error contract |

#### Epic 12: On-Canvas Editing & Git Write-Back

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 12.1-UNIT-001 | `set_field` correctly modifies inline YAML frontmatter field | Unit | R2-001 | title, owner single-value fields |
| 12.1-UNIT-002 | `set_field` correctly modifies block-style `depends_on` YAML sequence | Unit | R2-001 | Array fields |
| 12.1-UNIT-003 | `set_field` preserves other frontmatter fields and body content | Unit | R2-001 | No collateral damage |
| 12.1-INT-001 | Write-back pipeline: modify frontmatter → write file → git commit succeeds | Integration | R2-001 | Full pipeline test with tempdir repo |
| 12.1-INT-002 | Git revert (undo) restores previous file content and creates revert commit | Integration | R2-002 | Verify file content after undo |

**Total P0**: 32 tests, ~20-35 hours

### P1 (High)

**Criteria:** Important features + Medium risk (3-5) + Common workflows

#### Epic 8: AI-Inferred Causal Links & Trust Scoring

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.2-INT-005 | Configurable `ai.default_trust` in config overrides default 0.5 | Integration | - | Config-driven trust score |
| 8.3-UNIT-001 | CSM validation completes in <100ms per edge (NFR32) | Unit | R2-007 | Performance assertion |
| 8.1-INT-004 | `find_dependencies` response includes trust/origin for edges | Integration | - | Symmetric with trace_impact |

#### Epic 10: MCP Prompts

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 10.1-INT-001 | `prompts/list` includes `impact_analysis` with description and args | Integration | - | MCP protocol compliance |
| 10.1-INT-002 | `impact_analysis` prompt resolves with spec content + deps + impact chain | Integration | R2-008 | Verify message sequence structure |
| 10.1-INT-003 | `impact_analysis` with nonexistent spec_id returns `not_found` error | Integration | - | Error contract |
| 10.1-INT-004 | `impact_analysis` on isolated node returns empty deps/impact with note | Integration | - | Edge case |
| 10.2-INT-001 | `prompts/list` includes `spec_review` with description and args | Integration | - | MCP protocol compliance |
| 10.2-INT-002 | `spec_review` prompt resolves with spec content + graph context + checklist | Integration | R2-008 | Verify checklist structure |
| 10.2-INT-003 | `spec_review` with nonexistent spec_id returns `not_found` error | Integration | - | Error contract |
| 10.2-INT-004 | `spec_review` flags broken dependency references (missing `depends_on` targets) | Integration | - | Checklist validation |

#### Epic 11: Causal Graph Web UI

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 11.1-INT-001 | `GET /api/graph` returns `{nodes, edges}` with correct shape | Integration | - | REST API contract |
| 11.1-INT-002 | `GET /api/status` returns `{spec_count, last_sync_sha, consistency, drift_detected}` | Integration | - | REST API contract |
| 11.1-INT-003 | `GET /api/spec/{id}` returns full spec with edges and trust info | Integration | - | REST API contract |
| 11.1-INT-004 | `POST /api/sync?mode=full` triggers full rebuild and returns status | Integration | - | REST API contract |
| 11.1-INT-005 | `POST /api/sync?mode=incremental` triggers incremental sync | Integration | - | REST API contract |
| 11.5-INT-001 | API error responses use `{error_type, message, context}` shape | Integration | - | Error format consistency |

#### Epic 12: On-Canvas Editing & Git Write-Back

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 12.1-INT-003 | `POST /api/writeback` with add_edge operation creates commit | Integration | R2-001 | REST API + pipeline integration |
| 12.1-INT-004 | `POST /api/writeback` with remove_edge operation removes depends_on and commits | Integration | R2-001 | REST API + pipeline integration |
| 12.1-INT-005 | `POST /api/writeback/undo` reverts last commit within window | Integration | R2-002 | REST API + undo pipeline |
| 12.3-INT-001 | Write-back with frontmatter field edit (title change) creates correct commit | Integration | R2-001 | Field editing flow |
| 12.3-INT-002 | Write-back with tags array edit creates correct YAML output | Integration | R2-001 | Array field handling |

**Total P1**: 28 tests, ~15-25 hours

### P2 (Medium)

**Criteria:** Secondary features + Low risk (1-2) + Edge cases

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.1-UNIT-003 | `EdgeType` Display impl produces lowercase string (depends_on, constrains, implements) | Unit | - | Serialization format |
| 8.2-INT-006 | `add_causal_link` with invalid edge_type string returns `validation_error` | Integration | - | Input validation |
| 9.1-INT-004 | Empty AI edge set produces `edges: []` or no file | Integration | - | Boundary condition |
| 9.1-INT-005 | edges.yaml includes ISO 8601 `created_at` timestamp | Integration | - | Format compliance |
| 9.2-INT-005 | `reject_edge` on nonexistent edge returns `not_found` | Integration | - | Error contract |
| 10.1-INT-005 | `impact_analysis` prompt includes trust scores in edge metadata | Integration | - | Phase 2 data in prompts |
| 10.2-INT-005 | `spec_review` checklist covers completeness, clarity, dependency accuracy, consistency | Integration | - | Checklist completeness |
| 11.1-INT-006 | Static asset serving returns correct MIME types (html, js, css, svg) | Integration | R2-006 | rust-embed content types |
| 11.1-INT-007 | SPA fallback: non-API routes return index.html | Integration | R2-006 | Catch-all route |
| 11.1-INT-008 | Bearer token auth rejects requests when `web.host: 0.0.0.0` and no token | Integration | R2-003 | Security enforcement |
| 11.1-INT-009 | No auth required when `web.host: 127.0.0.1` (default) | Integration | R2-003 | Default permissive |
| 12.1-UNIT-004 | `set_field` handles frontmatter with no trailing newline | Unit | R2-001 | Edge case |
| 12.1-UNIT-005 | `set_field` handles frontmatter with comments | Unit | R2-001 | Edge case |
| 12.1-INT-006 | Concurrent write-back requests are serialized (Mutex) | Integration | - | Concurrency safety |
| 12.1-INT-007 | Undo after >5 seconds returns error (window expired) | Integration | R2-002 | Time window enforcement |
| 12.3-INT-003 | Write-back with `depends_on` change updates both file and graph | Integration | R2-011 | Auto re-sync verification |
| 11.1-UNIT-001 | WebConfig defaults: host=127.0.0.1, port=3000, enabled=true | Unit | - | Config defaults |
| 11.1-UNIT-002 | WebConfig from YAML: custom host/port parsed correctly | Unit | - | Config parsing |

**Total P2**: 18 tests, ~6-12 hours

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + Benchmarks

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.3-PERF-001 | CSM validation <100ms for 500-node graph (NFR32) | Integration | R2-007 | Scale benchmark |
| 11.1-PERF-001 | `/api/graph` response <500ms for 200+ node graph | Integration | R2-007 | API performance |
| 12.1-PERF-001 | Write-back round-trip <2s (NFR35) | Integration | R2-013 | End-to-end timing |
| 12.3-PERF-001 | Undo round-trip <2s (NFR36) | Integration | R2-013 | Revert timing |
| 12.1-UNIT-006 | `set_field` handles Unicode in field values | Unit | - | Internationalization |
| 12.1-UNIT-007 | `set_field` handles multiline YAML string values | Unit | R2-001 | Complex YAML |
| 9.1-INT-006 | edges.yaml survives 100 rapid sequential AI edge insertions | Integration | R2-005 | Stress test |
| 8.2-INT-007 | `add_causal_link` with all three edge types (DependsOn, Constrains, Implements) | Integration | - | Completeness |

**Total P3**: 8 tests, ~2-4 hours

---

## Execution Strategy

**Philosophy:** Run everything on PR via `cargo test --workspace`. Total test suite (Phase 1 + Phase 2) targets <2 minutes.

| Trigger | What Runs | Expected Duration |
|---------|-----------|-------------------|
| **Every PR** | `cargo test --workspace` (all P0-P2 + Phase 1 regression) | ~60-90 seconds |
| **Nightly** | P3 performance benchmarks (CSM scale, API latency, write-back timing) | ~5-10 minutes |

Performance benchmarks use `#[ignore]` attribute and run via `cargo test --workspace -- --ignored` in nightly CI.

---

## Resource Estimates

### Test Development Effort

| Priority | Count | Hours/Test | Total Hours | Notes |
|----------|-------|------------|-------------|-------|
| P0 | 32 | ~0.75-1.0 | ~20-35 | Complex setup (tempdir git repos, multi-store verification) |
| P1 | 28 | ~0.5-1.0 | ~15-25 | REST API contract tests, MCP prompt validation |
| P2 | 18 | ~0.3-0.5 | ~6-12 | Edge cases, config parsing, auth middleware |
| P3 | 8 | ~0.3-0.5 | ~2-4 | Benchmarks, stress tests |
| **Total** | **86** | **-** | **~45-75** | **~1-2 weeks** |

### Prerequisites

**Test Data:**

- Tempdir-based git repository factory (init repo, add spec files, commit)
- Spec markdown file fixtures with valid/invalid/edge-case frontmatter
- Multi-spec graph fixture (5-10 nodes with mixed edge types and trust scores)

**Tooling:**

- `tempfile` crate for isolated test directories
- `git2` for programmatic test repo creation
- Existing `FjallStore` + `SearchIndex` test helpers from Phase 1

**Environment:**

- Rust 1.85+ with workspace test runner
- Git available in CI environment (for write-back tests)

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate**: 100% (no exceptions)
- **P1 pass rate**: >=95% (waivers required for failures)
- **P2/P3 pass rate**: >=90% (informational)
- **High-risk mitigations**: 100% complete or approved waivers

### Coverage Targets

- **Critical paths** (write-back, CSM validation, edge lifecycle): >=80%
- **Security scenarios** (auth middleware): 100%
- **Error contracts** (MCP tool error responses): 100%
- **Edge cases** (YAML manipulation, empty states): >=50%

### Non-Negotiable Requirements

- [ ] All P0 tests pass
- [ ] No high-risk (>=6) items unmitigated
- [ ] Security tests (R2-003) pass 100%
- [ ] Performance targets met (R2-007, R2-013)

---

## Mitigation Plans

### R2-001: Git Write-Back Frontmatter Corruption (Score: 6)

**Mitigation Strategy:**
1. Roundtrip tests: parse spec → modify field → parse again → verify all fields intact
2. Test with edge-case YAML: multiline values, special characters, empty fields, comments
3. Test `set_field` for all supported field types (string, array, date)
**Owner:** Dev
**Timeline:** Epic 12 implementation
**Status:** In Progress (11 unit tests in writeback.rs cover basic cases)
**Verification:** P0 tests 12.1-UNIT-001 through 12.1-UNIT-003

### R2-002: Git Revert Failure on Complex History (Score: 6)

**Mitigation Strategy:**
1. Test undo on clean linear history (happy path)
2. Test undo when no prior commit exists (empty undo state)
3. Verify `cleanup_state()` is called after revert to clear git state
4. Test 5-second window expiry
**Owner:** Dev
**Timeline:** Epic 12 implementation
**Status:** In Progress (basic undo test exists)
**Verification:** P0 test 12.1-INT-002, P2 test 12.1-INT-007

### R2-003: Unauthenticated Write-Back on Exposed Host (Score: 6)

**Mitigation Strategy:**
1. Test that `web.host: 0.0.0.0` + `http.auth_token` config enforces bearer token on all API routes
2. Test that `web.host: 127.0.0.1` does not require auth (default)
3. Test missing/invalid/expired token returns 401
**Owner:** Dev
**Timeline:** Epic 11 implementation
**Status:** Planned
**Verification:** P2 tests 11.1-INT-008, 11.1-INT-009

### R2-004: CSM/Fjall State Divergence After AI Edge Insertion (Score: 6)

**Mitigation Strategy:**
1. After every `add_causal_link`, verify edge exists in both in-memory graph and Fjall
2. Test cycle detection on 3, 4, 5-node cycles
3. Test edge insertion between disconnected subgraphs
4. Verify edge count in both stores matches after operations
**Owner:** Dev
**Timeline:** Epic 8 implementation
**Status:** In Progress (existing causal engine tests cover basic operations)
**Verification:** P0 tests 8.3-INT-001 through 8.3-INT-003

### R2-005: edges.yaml Atomic Write Race (Score: 6)

**Mitigation Strategy:**
1. Test atomic write pattern (temp file + rename)
2. Test rapid sequential AI edge insertions (10+ in sequence)
3. Test empty edge set → edges.yaml should be empty list or absent
4. Test missing directory (`.lattice/` not created yet)
**Owner:** Dev
**Timeline:** Epic 9 implementation
**Status:** In Progress (export.rs has 5 unit tests)
**Verification:** P0 tests 9.1-INT-001 through 9.1-INT-003

---

## Assumptions and Dependencies

### Assumptions

1. Phase 1 tests continue to pass without modification (backward compatibility)
2. `git2` crate supports all revert operations needed for undo (confirmed: `repo.revert()` + manual commit)
3. Frontend Svelte UI is not tested in Rust test suite — REST API boundary is the testing surface
4. DeepCausality `CausaloidGraph` cycle detection is reliable for DAG validation (confirmed: `has_path_from_to` used for cycle check)
5. `rust-embed` correctly serves static assets in both debug (filesystem) and release (embedded) modes

### Dependencies

1. `tempfile` crate — Required for isolated git repo test fixtures
2. `git2` crate — Already in workspace; needed for write-back test setup
3. CI environment must have `git` available (for write-back integration tests)

### Risks to Plan

- **Risk**: Phase 2 schema changes (EdgeType, EdgeOrigin) break Phase 1 bincode-serialized data
  - **Impact**: All existing Fjall data unreadable after upgrade
  - **Contingency**: `#[serde(default)]` on new fields ensures graceful migration; verified by R2-009 test

---

## Interworking & Regression

| Service/Component | Impact | Regression Scope |
|-------------------|--------|------------------|
| **spec-db-core** (types.rs) | New EdgeType/EdgeOrigin enums added | Phase 1 causal engine tests must still pass with new enum variants |
| **spec-db-causal** (engine.rs) | `add_causal_link`, CSM validation, edge lifecycle | Phase 1 traversal tests (trace_impact, find_dependencies) must include trust/origin |
| **spec-db-mcp** (prompts.rs) | New MCP prompts (impact_analysis, spec_review) | Phase 1 MCP tool tests unaffected (tools unchanged) |
| **spec-db-ingest** (pipeline) | Ingestion now sets edge_type/trust/origin defaults | Phase 1 ingestion tests must verify new default field values |
| **spec-db-web** (NEW crate) | REST API, write-back, static assets | No Phase 1 regression — new crate |

---

## Existing Phase 2 Test Coverage (Baseline)

Current tests already implemented (263 total across workspace):

| Crate | Phase 2 Tests | Count | What's Covered |
|-------|--------------|-------|----------------|
| `spec-db-causal` engine.rs | EdgeType/EdgeOrigin unit tests, CSM validation, add_causal_link, cycle detection | ~19 | Core AI edge operations |
| `spec-db-causal` export.rs | edges.yaml export, atomic write, human-edge exclusion | ~5 | Edge export lifecycle |
| `spec-db-causal` integration.rs | Edge lifecycle integration, promote/reject, trust scoring | ~9 | Full causal integration |
| `spec-db-mcp` prompts.rs | impact_analysis, spec_review prompt structure, resolution, errors | ~9 | MCP prompt contracts |
| `spec-db-mcp` integration.rs | add_causal_link tool, promote/reject tools, prompt listing | ~19 | MCP tool + prompt integration |
| `spec-db-web` writeback.rs | set_field unit tests (11 variants: title, tags, depends_on, owner, etc.) | ~11 | Frontmatter manipulation |
| `spec-db-web` lib.rs | WebConfig defaults, config parsing | ~3 | Web configuration |

**Coverage gaps to fill (from this test design):**

1. REST API endpoint contract tests (GET /api/graph, /api/status, /api/spec, POST /api/sync, /api/writeback, /api/writeback/undo)
2. Write-back pipeline integration (file→git commit→re-sync roundtrip)
3. Git revert (undo) integration with cleanup_state
4. Bearer token auth middleware tests
5. Performance benchmarks (CSM scale, API latency, write-back timing)
6. Cross-store consistency after AI edge insertion (Fjall + in-memory)

---

## Appendix

### Knowledge Base References

- `risk-governance.md` - Risk classification framework (P×I scoring, gate decisions)
- `probability-impact.md` - Risk scoring methodology (1-3 scales, DOCUMENT/MONITOR/MITIGATE/BLOCK)
- `test-levels-framework.md` - Test level selection (Unit/Integration/E2E decision matrix)
- `test-priorities-matrix.md` - P0-P3 prioritization criteria

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md` (FR47-FR75, NFR32-NFR41)
- Epics: `_bmad-output/planning-artifacts/epics-phase2.md` (Epics 8-12, 15 stories)
- Architecture: `_bmad-output/planning-artifacts/architecture.md`
- Phase 1 Test Design: `_bmad-output/test-artifacts/test-design-epic-all.md` (90 scenarios)

### FR-to-Test Traceability

| FR | Test IDs |
|----|----------|
| FR47 (add_causal_link) | 8.2-INT-001 through 8.2-INT-007 |
| FR48 (trust score 0.5) | 8.2-INT-001, 8.2-INT-005 |
| FR49 (CSM validation) | 8.3-INT-001 through 8.3-INT-003, 8.3-PERF-001 |
| FR50 (CSM rejection) | 8.3-INT-001 |
| FR51 (trust in traversal) | 8.1-INT-002, 8.1-INT-004, 10.1-INT-005 |
| FR55 (edge types) | 8.1-UNIT-001, 8.2-INT-007 |
| FR52 (edges.yaml export) | 9.1-INT-001 through 9.1-INT-006 |
| FR53 (promote edge) | 9.2-INT-001, 9.2-INT-003, 9.2-INT-004 |
| FR54 (reject edge) | 9.2-INT-002, 9.2-INT-005 |
| FR56 (impact_analysis prompt) | 10.1-INT-001 through 10.1-INT-005 |
| FR57 (spec_review prompt) | 10.2-INT-001 through 10.2-INT-005 |
| FR58 (web UI HTTP) | 11.1-INT-001 through 11.1-INT-009 |
| FR59 (rust-embed) | 11.1-INT-006, 11.1-INT-007 |
| FR66 (sync status) | 11.1-INT-002 |
| FR70 (write-back) | 12.1-INT-001 through 12.1-INT-007, 12.1-PERF-001 |
| FR71 (undo) | 12.1-INT-002, 12.1-INT-005, 12.1-INT-007, 12.3-PERF-001 |
| FR72 (confirmation toast) | Frontend-only, not in Rust test scope |
| FR73 (rebuild trigger) | 11.1-INT-004 |
| FR75 (concurrent MCP+HTTP) | 11.1-INT-001 (server responds to HTTP) |
| NFR32 (CSM <100ms) | 8.3-UNIT-001, 8.3-PERF-001 |
| NFR35 (write-back <2s) | 12.1-PERF-001 |
| NFR36 (undo <2s) | 12.3-PERF-001 |
| NFR37 (localhost default) | 11.1-INT-009, 11.1-UNIT-001 |
| NFR38 (bearer token auth) | 11.1-INT-008 |

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `_bmad/tea/workflows/testarch/test-design`
**Version**: 4.0 (BMad v6)
