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

# Test Design Progress

## Step 1: Mode Detection & Prerequisites

- **Mode**: Epic-Level (Phase 4)
- **Detection basis**: `sprint-status.yaml` exists in implementation-artifacts
- **Scope**: All 7 epics (user requested full coverage)
- **Prerequisites**: All satisfied
  - Epics/stories with acceptance criteria: ✅ (7 epics, 18 stories)
  - Architecture context: ✅ (architecture.md available)
  - PRD: ✅ (prd.md available)

## Step 2: Context Loaded

### Configuration
- `tea_use_playwright_utils`: true
- `tea_browser_automation`: auto
- `test_artifacts`: `_bmad-output/test-artifacts`

### Project Artifacts Loaded
- PRD: 46 FRs, 31 NFRs, Rust CLI + MCP server
- Architecture: 7 crates, trait-based boundaries, async at MCP handler level only
- Epics: 7 epics, 18 stories, all in `review` status
- Tech stack: Tantivy 0.25, Fjall 3.0, DeepCausality 0.13.4, rmcp 0.16, git2 0.20.4

### Existing Test Coverage
| Crate | Unit Tests | Integration Tests | Test Count |
|-------|-----------|-------------------|------------|
| `spec-db-core` | types.rs, config.rs | None | ~10 |
| `spec-db-causal` | engine.rs | integration.rs | ~12 |
| `search` | None | integration.rs | ~9 |
| `ingest` | consistency.rs | integration.rs | ~13 |
| `router` | lib.rs, classifier.rs | integration.rs | ~5 |
| `mcp` | None | integration.rs | ~3 |
| Root binary | None | integration.rs | ~5 |

### Coverage Gaps Identified
1. No E2E MCP server workflow tests (connect → call tool → get result)
2. No streamable-HTTP transport tests (FR27)
3. No OTel instrumentation verification (FR44-46)
4. MCP integration tests only cover deserialization/parsing, not actual tool execution
5. No graceful degradation tests (search-only mode when graph fails)
6. No cross-store consistency startup flow tests
7. No rename detection tests for incremental sync (FR22)
8. Limited negative/error path coverage in several crates

### Knowledge Fragments Loaded
- risk-governance.md (scoring matrix, gate decisions)
- probability-impact.md (1-3 scales, thresholds)
- test-levels-framework.md (unit/integration/E2E selection)
- test-priorities-matrix.md (P0-P3 criteria)

## Step 3: Risk Assessment

- 16 risks identified across TECH/DATA/SEC/BUS/PERF/OPS categories
- 7 high-priority risks (score >= 6): R-001 through R-007
- 5 medium risks (score 3-5): R-008 through R-012
- 4 low risks (score 1-3): R-013 through R-016
- Mitigation plans defined for all high-priority risks

## Step 4: Coverage Plan

- 90 total test scenarios across P0-P3
- P0: 28 tests (~15-25 hours) - critical path + high risk
- P1: 30 tests (~15-25 hours) - important features + medium risk
- P2: 22 tests (~8-15 hours) - secondary + edge cases
- P3: 10 tests (~3-6 hours) - exploratory + benchmarks
- Execution: All tests run on PR via `cargo test --workspace`; perf benchmarks nightly

## Step 5: Output Generated

- Output file: `_bmad-output/test-artifacts/test-design-epic-all.md`
- Mode: Epic-Level (all 7 epics)
- Validated against checklist.md
