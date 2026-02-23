# Story 8.3: DeepCausality CSM Validation for AI-Proposed Edges

Status: ready-for-dev

## Story

As a system operator,
I want all AI-proposed edges validated against DeepCausality's Causal State Machine before acceptance,
so that only structurally valid causal relationships enter the graph.

## Acceptance Criteria (BDD)

**Given** an AI agent calls `add_causal_link` with a valid source, target, and edge_type
**When** the system processes the proposal
**Then** it runs CSM validation on the proposed edge before inserting it into the graph

**Given** a proposed edge that would create a cycle in the causal graph (A→B→C→A)
**When** CSM validation detects the cycle
**Then** the edge is rejected with `{ error_type: "csm_validation_failed", message: "Proposed edge creates a causal cycle", context: { cycle: ["A", "B", "C", "A"] } }`

**Given** a proposed edge that passes CSM validation
**When** the edge is accepted
**Then** it is inserted into the `CausaloidGraph` and persisted to Fjall KV with `origin: AiInferred` and the configured trust score

**Given** CSM validation processing
**When** the validation runs on a single proposed edge
**Then** it completes in under 100ms (NFR32)

**Given** a proposed edge between specs in disconnected subgraphs
**When** CSM validation runs
**Then** the edge passes validation (connecting disconnected components is valid)

**Covers:** FR49, FR50, NFR32

## Tasks / Subtasks

- [ ] Implement CSM validation module in `crates/causal/` (AC: 1, 2, 5)
  - [ ] Create validation function: `pub fn validate_proposed_edge(graph: &CausaloidGraph, source: NodeIndex, target: NodeIndex) -> Result<(), CsmValidationError>`
  - [ ] Implement cycle detection: check if adding edge source→target would create a cycle (DFS/BFS from target looking for source)
  - [ ] Return detailed cycle path in error context when cycle detected
  - [ ] Allow edges between disconnected subgraphs (no error for disconnected components)
- [ ] Define `CsmValidationError` type (AC: 2)
  - [ ] Create error variant with cycle path: `CycleDetected { cycle: Vec<String> }`
  - [ ] Map to MCP error shape: `{ error_type: "csm_validation_failed", message: "...", context: { cycle: [...] } }`
- [ ] Integrate CSM validation into `add_causal_link` tool (AC: 1, 3)
  - [ ] Call `validate_proposed_edge` BEFORE inserting edge into graph
  - [ ] On validation pass: proceed with edge insertion (existing Story 8.2 logic)
  - [ ] On validation fail: return CSM error response, do NOT insert edge
- [ ] Performance validation (AC: 4)
  - [ ] Add benchmark or timed test: validate single edge in graph with 100+ nodes completes in < 100ms
  - [ ] Use `std::time::Instant` in test, assert duration < 100ms
- [ ] Add tests (AC: 1-5)
  - [ ] Unit test: valid edge between connected nodes passes validation
  - [ ] Unit test: edge creating direct cycle (A→B, propose B→A) is rejected with cycle path
  - [ ] Unit test: edge creating indirect cycle (A→B→C, propose C→A) is rejected with full cycle path
  - [ ] Unit test: edge between disconnected subgraphs passes validation
  - [ ] Unit test: self-loop detection (handled by Story 8.2 but verify CSM also catches it)
  - [ ] Integration test: `add_causal_link` MCP call with cycle-causing edge returns correct error
  - [ ] Performance test: validation on 100+ node graph < 100ms

## Dev Notes

- This story modifies the `add_causal_link` flow from Story 8.2 by inserting a validation step before edge persistence.
- The `CausaloidGraph` from DeepCausality uses `ultragraph` internally. Cycle detection can use the graph's adjacency structure directly.
- CSM validation in this context is primarily structural (cycle detection). The DeepCausality `Causaloid::verify_single_cause` API may or may not be suitable — research during implementation. If not applicable, implement cycle detection directly on the graph topology.
- Performance: cycle detection is O(V+E) via DFS — should easily meet 100ms for graphs with hundreds of nodes.
- Architecture pattern P1: fail-fast on validation failure, return typed error.

### Project Structure Notes

- Primary files: `crates/causal/src/engine.rs` (or new `validation.rs` module), `crates/mcp/src/tools.rs` (integration point)
- May add `crates/causal/src/validation.rs` for CSM validation logic if engine.rs is already large.

### References

- [Source: _bmad-output/planning-artifacts/epics-phase2.md#Story 8.3]
- [Source: _bmad-output/planning-artifacts/architecture.md#DeepCausality Integration]
- [Source: crates/causal/src/engine.rs]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
