# Story 10.1: Impact Analysis MCP Prompt

Status: ready-for-dev

## Story

As an AI agent,
I want an `impact_analysis` MCP Prompt that guides me through structured impact assessment before proposing spec changes,
so that my analysis is systematic, auditable, and consistent regardless of which agent performs it.

## Acceptance Criteria (BDD)

**Given** an MCP client lists available prompts
**When** it calls `prompts/list`
**Then** the response includes `impact_analysis` with a description and its required arguments (e.g., `spec_id: string`)

**Given** an agent calls `prompts/get` for `impact_analysis` with `spec_id: "spec::auth::jwt-validation"`
**When** the prompt is resolved
**Then** it returns a structured message sequence that includes:
1. The spec's current content and metadata
2. Its direct downstream dependents (from `trace_impact`)
3. Its direct upstream dependencies (from `find_dependencies`)
4. A structured template asking the agent to assess: scope of change, affected specs, risk level, and recommended actions

**Given** the target `spec_id` does not exist
**When** the prompt is resolved
**Then** it returns an error: `{ error_type: "not_found", message: "Spec not found", context: { id: "..." } }`

**Given** the spec has no causal edges (isolated node)
**When** the prompt is resolved
**Then** it returns the spec content with empty dependency/impact lists and notes "No causal relationships found — impact is isolated"

**Covers:** FR56

## Tasks / Subtasks

- [ ] Implement MCP Prompt registration for `impact_analysis` (AC: 1)
  - [ ] Register `impact_analysis` in `prompts/list` response with description: "Guides structured impact assessment for a spec before proposing changes"
  - [ ] Define required argument: `spec_id: string` with description
  - [ ] Follow rmcp prompt registration pattern
- [ ] Implement `impact_analysis` prompt resolver (AC: 2, 3, 4)
  - [ ] Parse and validate `spec_id` argument
  - [ ] Look up spec by ID → return `not_found` error if missing
  - [ ] Fetch spec content and metadata (frontmatter + body)
  - [ ] Call `trace_impact` to get downstream dependents
  - [ ] Call `find_dependencies` to get upstream dependencies
  - [ ] Build structured message sequence with 4 sections:
    1. Spec content and metadata
    2. Downstream impact list (with edge types, trust scores)
    3. Upstream dependency list (with edge types, trust scores)
    4. Assessment template (scope, affected specs, risk level, recommendations)
  - [ ] Handle isolated node case: empty lists with explanatory note
- [ ] Add tests (AC: 1-4)
  - [ ] Unit test: prompt appears in `prompts/list`
  - [ ] Unit test: prompt resolution returns correct structure for spec with edges
  - [ ] Unit test: prompt resolution for isolated node includes "impact is isolated" note
  - [ ] Unit test: non-existent spec_id returns not_found error
  - [ ] Integration test: full MCP `prompts/get` round-trip

## Dev Notes

- MCP Prompts are a distinct capability from Tools — they return message sequences (system/user messages) that guide agent behavior. See rmcp docs for `prompts/list` and `prompts/get` protocol.
- This is the first MCP Prompt in the lattice system. Establish the registration pattern here for Story 10.2 to follow.
- The assessment template in section 4 should be markdown-formatted text that the agent fills in, not a structured schema.
- Reuse existing `trace_impact` and `find_dependencies` logic from `crates/causal/` — call the internal functions directly, not via MCP tool dispatch.

### Project Structure Notes

- Primary files: `crates/mcp/src/prompts.rs` (new module for prompt handlers)
- Modify: `crates/mcp/src/lib.rs` to register prompts module
- Reuses: `crates/causal/src/traversal.rs` (trace_impact, find_dependencies)

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 10.1]
- [Source: _bmad-output/planning-artifacts/prd.md#MCP Prompts]
- [Source: https://spec.modelcontextprotocol.io/specification/2025-03-26/server/prompts/]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
