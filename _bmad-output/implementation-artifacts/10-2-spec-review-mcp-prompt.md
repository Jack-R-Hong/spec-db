# Story 10.2: Spec Review MCP Prompt

Status: ready-for-dev

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

- [ ] Register `spec_review` MCP Prompt (AC: 1)
  - [ ] Register in `prompts/list` with description: "Guides structured spec review with quality checklist"
  - [ ] Define required argument: `spec_id: string`
  - [ ] Follow pattern established in Story 10.1
- [ ] Implement `spec_review` prompt resolver (AC: 2, 3, 4)
  - [ ] Parse and validate `spec_id` argument
  - [ ] Look up spec by ID → return `not_found` error if missing
  - [ ] Fetch full spec content (frontmatter + markdown body)
  - [ ] Fetch causal graph context (inbound/outbound edges with types and trust scores)
  - [ ] Check each `depends_on` reference exists in graph → collect broken references
  - [ ] Build structured message sequence with 3 sections:
    1. Full spec content
    2. Causal graph context (edges table)
    3. Review checklist with pre-populated findings (broken deps flagged)
  - [ ] Checklist dimensions: completeness, clarity, dependency accuracy, consistency
- [ ] Add tests (AC: 1-4)
  - [ ] Unit test: prompt appears in `prompts/list`
  - [ ] Unit test: prompt resolution returns correct structure for healthy spec
  - [ ] Unit test: broken `depends_on` references flagged in checklist
  - [ ] Unit test: non-existent spec_id returns not_found error
  - [ ] Integration test: full MCP `prompts/get` round-trip

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

### Debug Log References

### Completion Notes List

### Change Log

### File List
