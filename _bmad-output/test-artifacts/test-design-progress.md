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

# Test Design Progress — Phase 2

## Step 1: Mode Detection & Prerequisites

- **Mode**: Epic-Level (Phase 4)
- **Detection method**: File-based — `sprint-status.yaml` exists in `_bmad-output/implementation-artifacts/`
- **User intent**: "phase 2" — Epics 8-12
- **Scope**: 5 epics, 15 stories, 29 FRs (FR47-FR75), 7 NFRs (NFR32-NFR41)
- **Prerequisites check**:
  - Epic/story requirements with acceptance criteria: `epics-phase2.md` — PRESENT
  - Architecture context: `architecture.md` — PRESENT
  - PRD (supplementary): `prd.md` — PRESENT
- **All prerequisites satisfied. Proceeding.**

## Step 2: Context Loaded

### Configuration
- `tea_use_playwright_utils`: true (not applicable — Rust backend)
- `tea_browser_automation`: auto (skipped — CLI/MCP server, not web app)
- `test_artifacts`: `_bmad-output/test-artifacts`

### Project Artifacts Loaded
- PRD: FR47-FR75 (29 FRs), NFR32-NFR41 (7 NFRs)
- Architecture: 7 crates + new `spec-db-web` crate
- Epics: 5 epics (8-12), 15 stories, all `done` status
- Tech additions: axum 0.8, rust-embed 8.x, tower-http 0.6, Svelte 5 + @xyflow/svelte

### Existing Test Coverage (263 total, ~58 Phase 2-specific)
| Area | Count |
|------|-------|
| Core types (EdgeType, TrustLevel, serde) | 7 |
| Causal engine (CSM, cycles, trust) | 3 |
| Edge export (edges.yaml) | 5 |
| MCP add_causal_link tool | 8 |
| MCP promote/reject tools | 8 |
| MCP prompts (impact_analysis, spec_review) | 9 |
| Web config/API | 5 |
| Write-back pipeline (set_field) | 11 |
| Config (AI trust) | 2 |

### Coverage Gaps
1. No write-back integration (file → git commit → re-sync roundtrip)
2. No git revert (undo) integration
3. No REST API POST /api/writeback endpoint test
4. No REST API POST /api/writeback/undo endpoint test
5. No REST API POST /api/sync endpoint test
6. No REST API GET /api/spec/{id} endpoint test
7. No performance benchmarks (CSM scale, write-back timing)
8. No static asset MIME type / SPA fallback tests
9. No concurrent write-back serialization test

### Knowledge Fragments Loaded
- risk-governance.md (scoring matrix, gate decisions)
- probability-impact.md (1-3 scales, thresholds)
- test-levels-framework.md (unit/integration/E2E selection)
- test-priorities-matrix.md (P0-P3 criteria)

## Step 3: Risk Assessment

- 14 risks identified across TECH/SEC/PERF/DATA/BUS/OPS categories
- 5 high-priority risks (score >= 6): R2-001 through R2-005
  - R2-001 (TECH/6): Write-back frontmatter corruption via line-based YAML manipulation
  - R2-002 (DATA/6): Git revert failure / dirty state on undo
  - R2-003 (SEC/6): Unauthenticated write-back when web.host: 0.0.0.0
  - R2-004 (DATA/6): CSM/Fjall state divergence after AI edge insertion
  - R2-005 (TECH/6): edges.yaml atomic write race under rapid calls
- 6 medium risks (score 3-5): R2-006 through R2-011
- 3 low risks (score 1-2): R2-012 through R2-014
- No score-9 blockers — no BLOCK gate condition
- Mitigation plans defined for all high-priority risks

## Step 4: Coverage Plan

- 72 total test scenarios across P0-P3
- P0: 24 tests (~15-25 hours) — core path + high risk
- P1: 22 tests (~12-20 hours) — important features + medium risk
- P2: 18 tests (~6-12 hours) — secondary + edge cases
- P3: 8 tests (~2-4 hours) — benchmarks + stress tests
- Execution: PR = `cargo test --workspace` (all P0-P2, <2 min); Nightly = `--ignored` (P3 benchmarks)
- Quality gates: P0 = 100%, P1 >= 95%, security (R2-003) = 100%
- Net new tests needed: ~14-20 (58 existing cover many scenarios partially)
- No duplicate coverage across test levels

## Step 5: Output Generated

- **Mode**: Epic-Level (Epics 8-12)
- **Output file**: `_bmad-output/test-artifacts/test-design-epic-phase2.md`
- **Template used**: `test-design-template.md`
- **Validated against**: `checklist.md` — all epic-level criteria satisfied
- **Key risks**: R2-001 (frontmatter corruption), R2-002 (undo failure), R2-003 (auth bypass), R2-004 (store divergence), R2-005 (atomic write race)
- **Gate thresholds**: P0=100%, P1>=95%, security=100%
- **Open assumptions**: Phase 1 backward compat, no browser E2E, REST API is testing boundary
