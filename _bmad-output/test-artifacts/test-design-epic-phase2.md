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

**Scope:** Epic-level test design for Phase 2 (Epics 8-12): AI-inferred causal links, human review, MCP prompts, causal graph web UI, on-canvas editing with git write-back.

**Risk Summary:**

- Total risks identified: 14
- High-priority risks (>=6): 5
- Critical categories: TECH (4), SEC (2), DATA (3), PERF (2), BUS (2), OPS (1)

**Coverage Summary:**

- P0 scenarios: 24 (~15-25 hours)
- P1 scenarios: 22 (~12-20 hours)
- P2/P3 scenarios: 26 (~8-16 hours)
- **Total effort**: ~35-60 hours (~1-1.5 weeks)

---

## Not in Scope

| Item | Reasoning | Mitigation |
|------|-----------|------------|
| **Browser E2E tests (Playwright)** | Svelte UI compiled to static assets; backend REST API is testing boundary | REST API integration tests cover all data flows; manual visual QA |
| **Multi-user concurrent sessions** | Single-user desktop tool (NFR40) | Write-back serialized via Mutex; no concurrent user scenario |
| **Cross-browser compatibility (NFR41)** | Manual QA scope — Chrome/Firefox/Safari/Edge | Document as manual checklist |
| **WCAG accessibility (NFR39)** | Requires browser tooling (axe/lighthouse) | Deferred to separate accessibility audit |
| **Load/stress testing** | Single-user scale, 100-500 specs | Performance assertions in integration tests |
| **Phase 1 regression** | Covered by test-design-epic-all.md (90 scenarios, 263 passing tests) | Phase 1 tests run on every PR |

---

## Risk Assessment

### High-Priority Risks (Score >=6)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner | Timeline |
|---------|----------|-------------|------|--------|-------|------------|-------|----------|
| R2-001 | TECH | Write-back `set_field` corrupts YAML frontmatter — line-based manipulation breaks multiline values, special chars, or unterminated delimiters | 2 | 3 | 6 | Roundtrip tests (parse->modify->parse); edge-case fixtures (unicode, multiline, empty, comments) | Dev | Epic 12 |
| R2-002 | DATA | Git revert (undo) fails or leaves dirty state — `repo.revert()` stages but doesn't commit; `cleanup_state()` must be called | 2 | 3 | 6 | Test undo on linear history, empty undo state, expired 5s window; verify cleanup always called | Dev | Epic 12 |
| R2-003 | SEC | Unauthenticated write-back when `web.host: 0.0.0.0` — bearer token middleware bypass exposes git commit API | 2 | 3 | 6 | Auth rejection tests; verify default localhost; test 401 on missing/invalid token | Dev | Epic 11 |
| R2-004 | DATA | CSM/Fjall state divergence after AI edge insertion — in-memory graph diverges from persisted store | 2 | 3 | 6 | Verify both stores consistent post-insert; test cycle detection on 3-5 node cycles; test disconnected subgraph connection | Dev | Epic 8 |
| R2-005 | TECH | edges.yaml atomic write race — temp+rename fails under rapid sequential `add_causal_link` calls | 2 | 3 | 6 | Test atomic write under rapid calls; verify partial-write recovery; test empty/missing file | Dev | Epic 9 |

### Medium-Priority Risks (Score 3-5)

| Risk ID | Category | Description | Prob | Impact | Score | Mitigation | Owner |
|---------|----------|-------------|------|--------|-------|------------|-------|
| R2-006 | TECH | rust-embed debug/release divergence in content/MIME types | 2 | 2 | 4 | Test asset serving in both modes | Dev |
| R2-007 | PERF | CSM validation >100ms on large graphs (NFR32); API >500ms for 200+ nodes | 2 | 2 | 4 | Performance benchmarks at scale | Dev |
| R2-008 | BUS | MCP prompt resolves stale data after sync | 2 | 2 | 4 | Test prompt resolution after sync | Dev |
| R2-009 | TECH | Bincode roundtrip breaks with new enum variants on Phase 1 data | 2 | 2 | 4 | Migration test with `#[serde(default)]` | Dev |
| R2-010 | BUS | promote/reject edge returns incorrect error type on edge cases | 1 | 3 | 3 | Error contract tests for all tools | Dev |
| R2-011 | DATA | Write-back doesn't re-sync — graph shows stale data until manual sync | 2 | 2 | 4 | Verify auto re-sync after write-back | Dev |

### Low-Priority Risks (Score 1-2)

| Risk ID | Category | Description | Prob | Impact | Score | Action |
|---------|----------|-------------|------|--------|-------|--------|
| R2-012 | OPS | Web UI port conflict on default 3000 | 1 | 1 | 1 | Monitor |
| R2-013 | PERF | Write-back round-trip >2s on large repos (NFR35) | 1 | 2 | 2 | Monitor |
| R2-014 | BUS | Toast auto-dismiss races with Undo click within 5s window | 1 | 1 | 1 | Monitor |

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
- [x] All Phase 2 stories implemented and committed
- [x] Architecture document with Phase 2 extension available
- [x] PRD with FR47-FR75 and NFR32-NFR41 defined
- [x] 58 Phase 2-specific tests already passing
- [ ] Tempdir-based git repository fixtures for write-back tests

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (>=95%)
- [ ] No open high-priority risks (>=6) without documented mitigation
- [ ] Performance NFRs validated: CSM <100ms, write-back <2s, undo <2s
- [ ] Security: bearer token auth enforced when web.host=0.0.0.0
- [ ] Write-back pipeline: frontmatter roundtrip integrity verified

---

## Test Coverage Plan

> **Note:** P0/P1/P2/P3 = risk-based priority classification, NOT execution timing. All tests run on PR unless explicitly deferred.

### P0 (Critical)

**Criteria:** Blocks core functionality + High risk (>=6) + No workaround

#### Epic 8: AI-Inferred Causal Links & Trust Scoring

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.1-UNIT-001 | EdgeType enum: DependsOn, Constrains, Implements variants | Unit | R2-009 | Serde roundtrip for all variants |
| 8.1-UNIT-002 | EdgeOrigin enum: Human, Ai with default Human | Unit | R2-009 | `#[serde(default)]` behavior |
| 8.1-INT-001 | CausalEdge with new fields roundtrips through Fjall (bincode) | Integration | R2-009 | edge_type, trust, origin survive put/get |
| 8.1-INT-002 | trace_impact response includes edge_type, trust, origin per edge | Integration | - | JSON output shape |
| 8.1-INT-003 | Ingestion creates edges with DependsOn, trust=1.0, origin=Human | Integration | - | From depends_on frontmatter |
| 8.2-INT-001 | add_causal_link creates edge with trust=0.5, origin=Ai | Integration | R2-004 | Default AI trust |
| 8.2-INT-002 | add_causal_link: nonexistent source returns not_found | Integration | - | Error contract |
| 8.2-INT-003 | add_causal_link: self-reference returns validation_error | Integration | - | Error contract |
| 8.2-INT-004 | add_causal_link: duplicate edge returns conflict | Integration | - | Error contract |
| 8.3-INT-001 | CSM rejects edge creating cycle (A->B->C->A) | Integration | R2-004 | csm_validation_failed with cycle path |
| 8.3-INT-002 | CSM accepts edge between disconnected subgraphs | Integration | R2-004 | Valid: connecting components |
| 8.3-INT-003 | Valid edge persists to both Fjall and in-memory graph | Integration | R2-004 | Cross-store consistency |

#### Epic 9: Human Review & Edge Curation

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 9.1-INT-001 | AI edges exported to edges.yaml with all fields | Integration | R2-005 | source, target, edge_type, trust, origin, created_at |
| 9.1-INT-002 | Human-curated edges excluded from edges.yaml | Integration | - | Only AiInferred |
| 9.1-INT-003 | Atomic write: edges.yaml not corrupted on sequential writes | Integration | R2-005 | Temp+rename pattern |
| 9.2-INT-001 | promote_edge: origin->Human, trust->1.0, removed from yaml | Integration | - | Full lifecycle |
| 9.2-INT-002 | reject_edge: removed from graph, Fjall, and yaml | Integration | - | Full lifecycle |
| 9.2-INT-003 | promote_edge on nonexistent returns not_found | Integration | R2-010 | Error contract |
| 9.2-INT-004 | promote_edge on human edge returns validation_error | Integration | R2-010 | Error contract |

#### Epic 12: On-Canvas Editing & Git Write-Back

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 12.1-UNIT-001 | set_field modifies inline YAML field correctly | Unit | R2-001 | title, owner |
| 12.1-UNIT-002 | set_field modifies block-style depends_on array | Unit | R2-001 | Array fields |
| 12.1-UNIT-003 | set_field preserves other fields and body content | Unit | R2-001 | No collateral damage |
| 12.1-INT-001 | Write-back: modify frontmatter -> write file -> git commit | Integration | R2-001 | Full pipeline in tempdir |
| 12.1-INT-002 | Git revert (undo) restores file and creates revert commit | Integration | R2-002 | Verify file content after undo |

**Total P0**: 24 tests, ~15-25 hours

### P1 (High)

**Criteria:** Important features + Medium risk (3-5) + Common workflows

#### Epic 8

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.2-INT-005 | Configurable ai.default_trust overrides 0.5 | Integration | - | Config-driven |
| 8.3-UNIT-001 | CSM validation <100ms per edge (NFR32) | Unit | R2-007 | Performance assertion |
| 8.1-INT-004 | find_dependencies includes trust/origin for edges | Integration | - | Symmetric with trace_impact |

#### Epic 10: MCP Prompts

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 10.1-INT-001 | prompts/list includes impact_analysis with args | Integration | - | MCP compliance |
| 10.1-INT-002 | impact_analysis resolves with spec+deps+impact chain | Integration | R2-008 | Message sequence structure |
| 10.1-INT-003 | impact_analysis: nonexistent spec returns not_found | Integration | - | Error contract |
| 10.1-INT-004 | impact_analysis: isolated node returns empty deps with note | Integration | - | Edge case |
| 10.2-INT-001 | prompts/list includes spec_review with args | Integration | - | MCP compliance |
| 10.2-INT-002 | spec_review resolves with content+graph+checklist | Integration | R2-008 | Checklist structure |
| 10.2-INT-003 | spec_review: nonexistent spec returns not_found | Integration | - | Error contract |
| 10.2-INT-004 | spec_review flags broken dependency references | Integration | - | Checklist validation |

#### Epic 11: Web UI REST API

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 11.1-INT-001 | GET /api/graph returns {nodes, edges} shape | Integration | - | API contract |
| 11.1-INT-002 | GET /api/status returns spec_count, sha, consistency, drift | Integration | - | API contract |
| 11.1-INT-003 | GET /api/spec/{id} returns full spec with edges | Integration | - | API contract |
| 11.1-INT-004 | POST /api/sync?mode=full triggers rebuild | Integration | - | API contract |
| 11.1-INT-005 | POST /api/sync?mode=incremental triggers sync | Integration | - | API contract |
| 11.5-INT-001 | API errors use {error_type, message, context} shape | Integration | - | Format consistency |

#### Epic 12

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 12.1-INT-003 | POST /api/writeback add_edge creates commit | Integration | R2-001 | API + pipeline |
| 12.1-INT-004 | POST /api/writeback remove_edge removes depends_on | Integration | R2-001 | API + pipeline |
| 12.1-INT-005 | POST /api/writeback/undo reverts last commit | Integration | R2-002 | API + undo |
| 12.3-INT-001 | Field edit (title change) creates correct commit | Integration | R2-001 | Field editing |
| 12.3-INT-002 | Tags array edit creates correct YAML | Integration | R2-001 | Array handling |

**Total P1**: 22 tests, ~12-20 hours

### P2 (Medium)

**Criteria:** Secondary features + Low risk + Edge cases

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.1-UNIT-003 | EdgeType Display produces lowercase (depends_on, constrains, implements) | Unit | - | Serialization format |
| 8.2-INT-006 | add_causal_link: invalid edge_type returns validation_error | Integration | - | Input validation |
| 9.1-INT-004 | Empty AI edge set: edges: [] or no file | Integration | - | Boundary condition |
| 9.1-INT-005 | edges.yaml includes ISO 8601 created_at | Integration | - | Format compliance |
| 9.2-INT-005 | reject_edge: nonexistent returns not_found | Integration | - | Error contract |
| 10.1-INT-005 | impact_analysis includes trust scores in edge metadata | Integration | - | Phase 2 data in prompts |
| 10.2-INT-005 | spec_review checklist covers all 4 dimensions | Integration | - | Completeness |
| 11.1-INT-006 | Static assets: correct MIME types (html, js, css, svg) | Integration | R2-006 | rust-embed |
| 11.1-INT-007 | SPA fallback: non-API routes return index.html | Integration | R2-006 | Catch-all |
| 11.1-INT-008 | Bearer token rejects when host=0.0.0.0, no token | Integration | R2-003 | Security |
| 11.1-INT-009 | No auth required when host=127.0.0.1 | Integration | R2-003 | Default permissive |
| 12.1-UNIT-004 | set_field: frontmatter with no trailing newline | Unit | R2-001 | Edge case |
| 12.1-UNIT-005 | set_field: frontmatter with comments | Unit | R2-001 | Edge case |
| 12.1-INT-006 | Concurrent write-back serialized via Mutex | Integration | - | Concurrency |
| 12.1-INT-007 | Undo after >5s returns error (window expired) | Integration | R2-002 | Time window |
| 12.3-INT-003 | Write-back with depends_on updates file and graph | Integration | R2-011 | Auto re-sync |
| 11.1-UNIT-001 | WebConfig defaults: host=127.0.0.1, port=3000, enabled=true | Unit | - | Config |
| 11.1-UNIT-002 | WebConfig from YAML: custom host/port parsed | Unit | - | Config |

**Total P2**: 18 tests, ~6-12 hours

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + Benchmarks

| Test ID | Requirement | Test Level | Risk Link | Notes |
|---------|-------------|------------|-----------|-------|
| 8.3-PERF-001 | CSM validation <100ms for 500-node graph (NFR32) | Integration | R2-007 | Scale benchmark |
| 11.1-PERF-001 | /api/graph response <500ms for 200+ nodes | Integration | R2-007 | API perf |
| 12.1-PERF-001 | Write-back round-trip <2s (NFR35) | Integration | R2-013 | Timing |
| 12.3-PERF-001 | Undo round-trip <2s (NFR36) | Integration | R2-013 | Timing |
| 12.1-UNIT-006 | set_field: Unicode in field values | Unit | - | i18n |
| 12.1-UNIT-007 | set_field: multiline YAML string values | Unit | R2-001 | Complex YAML |
| 9.1-INT-006 | edges.yaml survives 100 rapid sequential inserts | Integration | R2-005 | Stress |
| 8.2-INT-007 | add_causal_link with all 3 edge types | Integration | - | Completeness |

**Total P3**: 8 tests, ~2-4 hours

---

## Execution Strategy

**Philosophy:** Run everything on PR via `cargo test --workspace`. Target <2 minutes total.

| Trigger | What Runs | Expected Duration |
|---------|-----------|-------------------|
| **Every PR** | `cargo test --workspace` — all P0-P2 functional tests | ~60-90 seconds |
| **Nightly** | `cargo test --workspace -- --ignored` — P3 performance benchmarks | ~5-10 minutes |

Performance benchmarks use `#[ignore]` attribute and run via nightly CI job.

---

## Resource Estimates

### Test Development Effort

| Priority | Count | Hours/Test | Total Hours | Notes |
|----------|-------|------------|-------------|-------|
| P0 | 24 | ~0.75-1.0 | ~15-25 | Tempdir git repos, multi-store verification |
| P1 | 22 | ~0.5-1.0 | ~12-20 | REST API contracts, MCP prompt validation |
| P2 | 18 | ~0.3-0.5 | ~6-12 | Edge cases, config, auth |
| P3 | 8 | ~0.3-0.5 | ~2-4 | Benchmarks, stress |
| **Total** | **72** | **-** | **~35-60** | **~1-1.5 weeks** |

**Note:** 58 existing Phase 2 tests provide partial coverage. Net new tests: ~14-20.

### Prerequisites

**Test Data:**

- Tempdir git repository factory (init, add specs, commit)
- Spec markdown fixtures with valid/invalid/edge-case frontmatter
- Multi-spec graph fixture (5-10 nodes, mixed edge types/trust scores)

**Tooling:**

- `tempfile` crate for isolated test directories
- `git2` for programmatic test repo creation
- Existing FjallStore + SearchIndex test helpers from Phase 1

**Environment:**

- Rust 1.85+ with workspace test runner
- Git available in CI (for write-back integration tests)

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate**: 100% (no exceptions)
- **P1 pass rate**: >=95% (waivers required)
- **P2/P3 pass rate**: >=90% (informational)
- **High-risk mitigations**: 100% complete or approved waivers

### Coverage Targets

- **Critical paths** (write-back, CSM, edge lifecycle): >=80%
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

### R2-001: Write-Back Frontmatter Corruption (Score: 6)

**Mitigation Strategy:**
1. Roundtrip tests: parse spec -> set_field -> parse again -> verify all fields intact
2. Edge-case YAML fixtures: multiline values, special chars, empty fields, comments
3. Test set_field for all supported field types (string, array, date)
**Owner:** Dev
**Timeline:** Epic 12 implementation
**Status:** In Progress (11 unit tests in writeback.rs)
**Verification:** P0 tests 12.1-UNIT-001 through 12.1-UNIT-003

### R2-002: Git Revert Failure (Score: 6)

**Mitigation Strategy:**
1. Test undo on clean linear history (happy path)
2. Test undo when no prior commit exists (empty undo state)
3. Verify cleanup_state() called after revert
4. Test 5-second window expiry
**Owner:** Dev
**Timeline:** Epic 12 implementation
**Status:** In Progress
**Verification:** P0 test 12.1-INT-002, P2 test 12.1-INT-007

### R2-003: Unauthenticated Write-Back (Score: 6)

**Mitigation Strategy:**
1. Test web.host=0.0.0.0 + auth_token enforces bearer token on all API routes
2. Test web.host=127.0.0.1 does not require auth
3. Test missing/invalid token returns 401
**Owner:** Dev
**Timeline:** Epic 11 implementation
**Status:** In Progress (2 auth tests exist)
**Verification:** P2 tests 11.1-INT-008, 11.1-INT-009

### R2-004: CSM/Fjall State Divergence (Score: 6)

**Mitigation Strategy:**
1. After every add_causal_link, verify edge in both in-memory graph and Fjall
2. Test cycle detection on 3, 4, 5-node cycles
3. Test edge insertion between disconnected subgraphs
4. Verify edge count matches across stores
**Owner:** Dev
**Timeline:** Epic 8 implementation
**Status:** In Progress (cycle detection + add_causal_link tests exist)
**Verification:** P0 tests 8.3-INT-001 through 8.3-INT-003

### R2-005: edges.yaml Atomic Write Race (Score: 6)

**Mitigation Strategy:**
1. Test atomic write pattern (temp file + rename)
2. Test rapid sequential AI edge insertions
3. Test empty edge set and missing directory
**Owner:** Dev
**Timeline:** Epic 9 implementation
**Status:** In Progress (5 export tests exist)
**Verification:** P0 tests 9.1-INT-001 through 9.1-INT-003

---

## Assumptions and Dependencies

### Assumptions

1. Phase 1 tests continue to pass without modification (backward compatibility)
2. git2 crate supports all revert operations needed for undo
3. Frontend Svelte UI is not tested in Rust test suite — REST API is the testing surface
4. DeepCausality CausaloidGraph cycle detection is reliable (has_path_from_to)
5. rust-embed correctly serves assets in both debug and release modes

### Dependencies

1. `tempfile` crate — isolated git repo test fixtures
2. `git2` crate — programmatic test repo creation (already in workspace)
3. CI environment must have `git` available

### Risks to Plan

- **Risk**: Phase 2 schema changes break Phase 1 bincode data
  - **Impact**: Existing Fjall data unreadable after upgrade
  - **Contingency**: `#[serde(default)]` on new fields; verified by R2-009 test

---

## Interworking & Regression

| Service/Component | Impact | Regression Scope |
|-------------------|--------|------------------|
| **spec-db-core** (types.rs) | New EdgeType/EdgeOrigin enums | Phase 1 causal engine tests pass with new variants |
| **spec-db-causal** (engine.rs) | add_causal_link, CSM, edge lifecycle | Phase 1 traversal tests include trust/origin |
| **spec-db-mcp** (prompts.rs) | New MCP prompts | Phase 1 MCP tool tests unaffected |
| **spec-db-ingest** (pipeline) | Sets edge_type/trust/origin defaults | Phase 1 ingestion tests verify new defaults |
| **spec-db-web** (NEW crate) | REST API, write-back, assets | No Phase 1 regression — new crate |

---

## Existing Coverage Baseline

58 Phase 2-specific tests already passing:

| Area | Count | What's Covered |
|------|-------|----------------|
| Core types (EdgeType, TrustLevel, serde) | 7 | Enum variants, serde roundtrip, defaults |
| Causal engine (CSM, cycles, trust) | 3 | Cycle detection, auto-trust |
| Edge export (edges.yaml) | 5 | YAML structure, atomic write, exclusion |
| MCP add_causal_link tool | 8 | Create, errors, edge types, custom trust |
| MCP promote/reject tools | 8 | Lifecycle, errors, yaml cleanup |
| MCP prompts | 9 | Both prompts: structure, resolution, errors |
| Web config/API | 5 | Defaults, API shape, auth |
| Write-back (set_field) | 11 | Split, reassemble, field ops |
| Config (AI trust) | 2 | YAML parse, validation |

**Gaps to fill:** REST API endpoint tests (sync, writeback, undo), write-back integration (git roundtrip), performance benchmarks.

---

## Appendix

### Knowledge Base References

- `risk-governance.md` — Risk classification (P x I scoring, gate decisions)
- `probability-impact.md` — Risk scoring (1-3 scales, DOCUMENT/MONITOR/MITIGATE/BLOCK)
- `test-levels-framework.md` — Test level selection (Unit/Integration/E2E)
- `test-priorities-matrix.md` — P0-P3 prioritization criteria

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md` (FR47-FR75, NFR32-NFR41)
- Epics: `_bmad-output/planning-artifacts/epics-phase2.md` (Epics 8-12)
- Architecture: `_bmad-output/planning-artifacts/architecture.md`
- Phase 1 Test Design: `_bmad-output/test-artifacts/test-design-epic-all.md`

### FR-to-Test Traceability

| FR | Test IDs |
|----|----------|
| FR47 (add_causal_link) | 8.2-INT-001 through 8.2-INT-007 |
| FR48 (trust=0.5) | 8.2-INT-001, 8.2-INT-005 |
| FR49 (CSM validation) | 8.3-INT-001 through 8.3-INT-003, 8.3-PERF-001 |
| FR50 (CSM rejection) | 8.3-INT-001 |
| FR51 (trust in traversal) | 8.1-INT-002, 8.1-INT-004, 10.1-INT-005 |
| FR55 (edge types) | 8.1-UNIT-001, 8.2-INT-007 |
| FR52 (edges.yaml export) | 9.1-INT-001 through 9.1-INT-006 |
| FR53 (promote edge) | 9.2-INT-001, 9.2-INT-003, 9.2-INT-004 |
| FR54 (reject edge) | 9.2-INT-002, 9.2-INT-005 |
| FR56 (impact_analysis) | 10.1-INT-001 through 10.1-INT-005 |
| FR57 (spec_review) | 10.2-INT-001 through 10.2-INT-005 |
| FR58 (web UI HTTP) | 11.1-INT-001 through 11.1-INT-009 |
| FR59 (rust-embed) | 11.1-INT-006, 11.1-INT-007 |
| FR66 (sync status) | 11.1-INT-002 |
| FR70 (write-back) | 12.1-INT-001 through 12.1-INT-007, 12.1-PERF-001 |
| FR71 (undo) | 12.1-INT-002, 12.1-INT-005, 12.1-INT-007, 12.3-PERF-001 |
| FR72 (confirmation toast) | Frontend-only — not in Rust test scope |
| FR73 (rebuild trigger) | 11.1-INT-004 |
| FR75 (concurrent serve) | 11.1-INT-001 |
| NFR32 (CSM <100ms) | 8.3-UNIT-001, 8.3-PERF-001 |
| NFR35 (write-back <2s) | 12.1-PERF-001 |
| NFR36 (undo <2s) | 12.3-PERF-001 |
| NFR37 (localhost default) | 11.1-INT-009, 11.1-UNIT-001 |
| NFR38 (bearer token) | 11.1-INT-008 |

---

## Follow-on Workflows (Manual)

- Run `*atdd` to generate failing P0 tests (separate workflow; not auto-run).
- Run `*automate` for broader coverage once gap tests are written.

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `_bmad/tea/workflows/testarch/test-design`
**Version**: 4.0 (BMad v6)
