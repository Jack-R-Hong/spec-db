# Story 10.2: Spec Review MCP Prompt

Status: review

## Story

As an AI agent,
I want a `spec_review` MCP Prompt that guides me through a structured spec review with a checklist,
so that my reviews are thorough, consistent, and cover all quality dimensions.

## Acceptance Criteria (BDD)

**Given** an MCP client lists available prompts
**When** it calls `prompts/list`
**Then** the response includes `spec_review` with a description and its required arguments (e.g., `spec_id: string`)

**Given** an agent calls `prompts/get` for `spec_review` with `spec_id: "spec::auth::jwt-validation"`
**When** the prompt is resolved
**Then** it returns a structured message sequence that includes:
1. The spec's full content (frontmatter + body)
2. Its causal graph context (edges in/out, trust scores, edge types)
3. A review checklist covering: completeness (required frontmatter fields present), clarity (title, body coherence), dependency accuracy (do declared `depends_on` specs exist?), and consistency (tags, version, dates)

**Given** the target `spec_id` does not exist
**When** the prompt is resolved
**Then** it returns an error: `{ error_type: "not_found", message: "Spec not found", context: { id: "..." } }`

**Given** a spec has `depends_on` references to specs that don't exist in the graph
**When** the review prompt is resolved
**Then** the checklist flags these as "broken dependency references" with the missing spec IDs listed

**Covers:** FR57

## Tasks / Subtasks

- [x] Register `spec_review` MCP Prompt (AC: 1)
  - [x] Register in `prompts/list` with description: "Guides structured spec review with quality checklist"
  - [x] Define required argument: `spec_id: string`
  - [x] Follow pattern established in Story 10.1
- [x] Implement `spec_review` prompt resolver (AC: 2, 3, 4)
  - [x] Parse and validate `spec_id` argument
  - [x] Look up spec by ID → return `not_found` error if missing
  - [x] Fetch full spec content (frontmatter + markdown body)
  - [x] Fetch causal graph context (inbound/outbound edges with types and trust scores)
  - [x] Check each `depends_on` reference exists in graph → collect broken references
  - [x] Build structured message sequence with 3 sections:
    1. Full spec content
    2. Causal graph context (edges table)
    3. Review checklist with pre-populated findings (broken deps flagged)
  - [x] Checklist dimensions: completeness, clarity, dependency accuracy, consistency
- [x] Add tests (AC: 1-4)
  - [x] Unit test: prompt appears in `prompts/list`
  - [x] Unit test: prompt resolution returns correct structure for healthy spec
  - [x] Unit test: broken `depends_on` references flagged in checklist
  - [x] Unit test: non-existent spec_id returns not_found error

## Dev Notes

- Follows the prompt registration pattern established in Story 10.1.
- The review checklist should be a markdown-formatted checklist in the message content. Pre-populate known findings (broken deps) while leaving other dimensions for the agent to assess.
- Broken dependency detection: iterate `depends_on` from frontmatter, check each ID exists in the graph. Missing ones are flagged.
- Required frontmatter fields for completeness check: `id`, `title`, `version`, `tags`, `created`. Optional but recommended: `owner`, `depends_on`.

### Project Structure Notes

- Primary files: `crates/mcp/src/prompts.rs` (add second prompt handler alongside impact_analysis)
- Reuses existing spec retrieval and graph query logic

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 10.2]
- [Source: _bmad-output/planning-artifacts/prd.md#MCP Prompts]

## Dev Agent Record

### Agent Model Used
claude-opus-4-6

### Completion Notes List
- Added `spec_review` prompt to `prompt_definitions()` alongside `impact_analysis`
- Refactored common `extract_spec_id()` and `lookup_spec()` helpers from shared code
- Implemented `resolve_spec_review()` with 3-section message sequence: full spec content, causal graph context (markdown tables), and review checklist
- Checklist pre-populates: completeness checks (required fields), broken dependency detection, empty tags/owner warnings
- Broken deps detected by checking each `depends_on` ID exists in the graph; flagged as "BROKEN DEPENDENCY REFERENCES"
- 4 new tests: prompt listing, healthy spec review, broken deps flagging, not_found error

### Change Log
- `crates/mcp/src/prompts.rs` — Added spec_review prompt, extract_spec_id/lookup_spec helpers, resolve_spec_review fn, 4 new tests

### File List
- crates/mcp/src/prompts.rs (modified)
