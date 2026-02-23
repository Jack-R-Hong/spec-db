# Sprint Change Proposal

**Project:** lattice
**Date:** 2026-02-23
**Triggered by:** Story 1-3 (DeepCausality In-Memory Graph Engine)
**Author:** Jack (via correct-course workflow)

---

## 1. Issue Summary

**Problem Statement:** Story 1-3 was implemented using HashMap adjacency lists + custom BFS traversal instead of the DeepCausality library as specified in the architecture. The `deep_causality = "=0.13.4"` dependency is declared in `Cargo.toml` but zero Rust code imports or uses it. The dev agent noted: *"Used HashMap-based adjacency list instead of DeepCausality internals (too complex for simple spec DAGs)."*

**Discovery Context:** During pre-implementation readiness check for DeepCausality integration, code audit revealed the library is entirely unused despite being a locked dependency.

**Evidence:**
- `grep` for `deep_causality` across all `.rs` files returns zero imports — only a version-pinning acceptance test
- `CausalEngine` in `crates/causal/src/engine.rs` uses `HashMap<String, SpecNode>` and `HashMap<String, Vec<CausalEdge>>` for all graph operations
- Architecture document explicitly states: *"DeepCausality + Fjall = causal reasoning (Fjall is DeepCausality's persistence backend)"*
- Project context document lists DeepCausality as locked technology for causal reasoning

---

## 2. Impact Analysis

### Epic Impact

| Epic | Impact | Details |
|------|--------|---------|
| Epic 1: Foundation & Causal Knowledge Graph | **Direct** | Story 1-3 is the core integration point. Story 1-4 (traversal) depends on engine internals. |
| Epic 2–7 | **None** | All other epics interact via `CausalGraph` trait — API-stable replacement. |

### Story Impact

| Story | Status | Impact |
|-------|--------|--------|
| 1-3 (DeepCausality In-Memory Graph Engine) | review | **Re-implement** — replace HashMap with `CausaloidGraph` |
| 1-4 (Causal Graph Traversal) | review | **Verify** — traversal logic may need adapter changes |
| All other stories | review | **No change** — they use `CausalGraph` trait, not `CausalEngine` directly |

### Artifact Conflicts

| Artifact | Conflict | Action Needed |
|----------|----------|---------------|
| PRD | None | DeepCausality is already a requirement |
| Architecture | None | Architecture already specifies DeepCausality; current code deviates from it |
| Story 1-3 file | **Yes** | Tasks/subtasks need new implementation plan |
| sprint-status.yaml | **Yes** | Story 1-3 status: `review` → `in-progress` |

### Technical Impact

**API Contract (PRESERVED):** The `CausalGraph` trait in `crates/core/src/traits.rs` defines the stable API. All 11 calling locations use this trait. The replacement is internal to `crates/causal/src/engine.rs`.

**Key Challenge — DeepCausality API Mismatch:**

| Aspect | Lattice Design | DeepCausality API |
|--------|---------------|-------------------|
| Node ID | `SpecId` (string) | `usize` (index) |
| Node Type | `SpecNode { id, title, version }` | `Causaloid<I,O,STATE,CTX>` (4 generics) |
| Edge Metadata | `CausalEdge { source, target, trust, origin }` | `(usize, usize, u64)` weight only |
| Traversal | `trace_impact`, `find_dependencies` | Not provided — only `shortest_path` |

**Required Adapter Work:**
1. Bidirectional `SpecId ↔ usize` index mapping
2. Lightweight `Causaloid` wrapper around `SpecNode` (or use `CausaloidGraph<SpecNode>` directly if node type only needs `Clone`)
3. Edge metadata stored in parallel `HashMap` (trust, origin) since DeepCausality edges only carry `u64` weight
4. BFS traversal re-implemented on top of `CausaloidGraph` adjacency queries
5. Freeze/unfreeze lifecycle management

---

## 3. Recommended Approach

**Selected: Direct Adjustment — Modify Story 1-3 within existing plan**

- **Effort:** Medium-High — engine.rs rewrite, adapter layer, index mapping
- **Risk:** Medium — `CausalGraph` trait contract preserved; all callers unchanged
- **Timeline Impact:** Minimal — contained within single crate (`crates/causal`)

**Rationale:**
- Architecture explicitly requires DeepCausality integration
- Trait-based boundary isolates blast radius to `engine.rs` internals
- All 11 test files exercise the `CausalGraph` trait, providing regression safety
- No epic restructuring needed; no PRD changes needed

**Trade-offs Considered:**
- CSM adapter layer (add DeepCausality features on top of HashMap) — rejected by user in favor of full replacement
- Petgraph fallback — architecture-defined fallback, not applicable here since user explicitly wants DeepCausality

---

## 4. Detailed Change Proposals

### Story 1-3: Update Implementation Tasks

```
Story: 1-3-deepcausality-in-memory-graph-engine
Section: Tasks / Subtasks

OLD:
- [x] Create the in-memory graph engine module (AC: 1, 3)
  - [x] Add `crates/spec-db-causal/src/engine.rs` with `pub struct CausalEngine`
  - [x] Include fields for graph structure plus lookup indices by `SpecId` for O(1) node lookup

NEW:
- [ ] Replace HashMap engine with DeepCausality CausaloidGraph backend
  - [ ] Define `SpecCausaloid` wrapper type implementing required DeepCausality traits
  - [ ] Replace `HashMap<String, SpecNode>` with `CausaloidGraph<SpecCausaloid>`
  - [ ] Implement bidirectional `SpecId ↔ usize` index mapping
  - [ ] Store edge metadata (TrustLevel, EdgeOrigin) in parallel HashMap since DC edges are u64-only
  - [ ] Implement freeze/unfreeze lifecycle for DeepCausality graph
  - [ ] Re-implement BFS traversal (trace_impact, find_dependencies) on CausaloidGraph
  - [ ] Preserve all CausalGraph trait method signatures exactly
  - [ ] Preserve FjallStore integration (load_from_store, write-through)
- [ ] Update unit tests to validate DeepCausality backend
  - [ ] All 8 existing engine.rs tests must pass without signature changes
  - [ ] Add test: CausaloidGraph freeze/unfreeze lifecycle
  - [ ] Add test: SpecId ↔ usize mapping consistency
- [ ] Run full test suite (32+ tests) and verify performance gates
  - [ ] Startup < 1s for 100+ specs
  - [ ] Traversal < 50ms

Rationale: Architecture requires DeepCausality as causal reasoning engine.
HashMap was an interim implementation. Full replacement with proper adapter layer.
```

### Story 1-3: Update Dev Notes

```
Story: 1-3-deepcausality-in-memory-graph-engine
Section: Dev Notes (append)

NEW (append):
- **DeepCausality 0.13.4 API mapping:**
  - `CausaloidGraph<T>` wraps `UltraGraphWeighted<T, u64>` (ultragraph crate)
  - Nodes identified by `usize` index, not string keys — requires SpecId↔usize mapping
  - Edges carry `u64` weight only — TrustLevel/EdgeOrigin stored in parallel HashMap
  - `CausableGraph<T>` trait: `add_causaloid`, `get_causaloid`, `remove_causaloid`, `add_edge`, `remove_edge`
  - Graph has freeze/unfreeze lifecycle for performance optimization
  - No built-in trace_impact/find_dependencies — implement via adjacency iteration
- **Adapter strategy:** Thin wrapper in engine.rs that maps Lattice's CausalGraph trait to DeepCausality's CausableGraph trait
```

### sprint-status.yaml: Update Story Status

```
File: _bmad-output/implementation-artifacts/sprint-status.yaml
Section: development_status

OLD:
  1-3-deepcausality-in-memory-graph-engine: review

NEW:
  1-3-deepcausality-in-memory-graph-engine: in-progress
```

---

## 5. Implementation Handoff

**Change Scope: Minor** — Direct implementation by development team. Contained within `crates/causal/src/engine.rs` with stable trait boundary.

**Handoff: Dev Agent**
- Re-implement `CausalEngine` internals using `CausaloidGraph`
- Preserve `CausalGraph` trait contract exactly
- Run full test suite for regression validation

**Success Criteria:**
1. `cargo test` — all existing tests pass (0 test changes)
2. `cargo clippy -D warnings` — clean
3. `deep_causality` crate is actually imported and used in engine.rs
4. Performance gates: startup < 1s, traversal < 50ms for 100+ specs
5. Story 1-3 status updated to `done`

**Files to Modify:**
- `crates/causal/src/engine.rs` — primary rewrite
- `crates/causal/src/traversal.rs` — adapt to CausaloidGraph adjacency
- `_bmad-output/implementation-artifacts/1-3-deepcausality-in-memory-graph-engine.md` — updated tasks
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status update
