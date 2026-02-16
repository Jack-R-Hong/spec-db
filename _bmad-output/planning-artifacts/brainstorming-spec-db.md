# Brainstorming Report: spec-db

**Date:** 2026-02-17
**Session:** BMAD Party Mode — Multi-Agent Collaborative Discussion
**Participants:** Winston (Architect), Mary (Analyst), Amelia (Dev), John (PM), Barry (Quick Flow), Bob (SM)

---

## Product Concept

**spec-db** is a causal specification database designed for AI agents to rapidly find specifications and understand program architecture. It combines full-text search with causal reasoning to provide two distinct modes of intelligence:

- **Discovery Mode** — "What exists?" — Find specs by keyword, tag, or natural language
- **Reasoning Mode** — "What connects?" — Trace causal relationships, dependencies, and downstream impacts

This is not just a search engine — it is a **knowledge graph with search capabilities**.

---

## Problem Statement

When AI agents need to find specifications or understand program architecture, current approaches are inadequate:

- Agents grep through files or read entire repositories — slow, noisy, and imprecise
- No structured way to understand **causal relationships** between specs, architecture decisions, and implementations
- Changes to one specification can silently break downstream components with no visibility
- AI agents lack a standardized tool interface to query project knowledge

---

## Core Value Proposition

spec-db gives AI agents a **first-class tool** (via MCP — Model Context Protocol) to:

1. **Search** specs efficiently through full-text indexing
2. **Trace** causal chains — "if I change spec A, what breaks?"
3. **Discover** dependencies between specs, architecture decisions, and code modules
4. **Grow** the knowledge base through hybrid human + AI causal edge creation

---

## Two Modes of Intelligence

| Mode | Engine | Question Type | Example |
|------|--------|--------------|---------|
| **Discovery** | Tantivy | "What exists?" | "Find specs about authentication" |
| **Reasoning** | DeepCausality + Fjall | "What connects?" | "What's causally affected by auth changes?" |

An agent can **discover** relevant specs through search, then **reason** about their causal relationships through the graph.

---

## Key Design Decisions (from Party Mode discussion)

### 1. Spec Format: Markdown + YAML Frontmatter
- Human-readable, AI-parseable
- YAML frontmatter for structured metadata (id, title, version, tags, depends_on, owner)
- Markdown body for prose content
- `depends_on` field in frontmatter = human-curated causal edges

### 2. Git as Source of Truth
- Spec markdown files live in a git repository (`specs/*.md`)
- Version history comes from git, not the database
- spec-db is a **derived index** — can be destroyed and rebuilt from `git clone` + rebuild
- Changes tracked via git commits, enabling PR-based review workflows

### 3. Hybrid Causal Edge Creation
- **Human-curated edges:** `depends_on` in YAML frontmatter (trust=1.0)
- **AI-inferred edges:** Added via `add_causal_link` MCP tool (trust=0.x, CSM-validated)
- DeepCausality's Causal State Machine (CSM) validates AI-proposed edges
- AI-inferred edges periodically exported to `.spec-db/edges.yaml` in git for review

### 4. Full In-Memory Graph
- Entire causal graph loaded from Fjall into memory at startup
- Hundreds of specs = single-digit megabytes in memory = sub-second startup
- Fjall handles persistence and crash recovery
- All graph reasoning happens in-memory for maximum speed

### 5. MCP + Rust Library API
- Primary API: MCP server (via `rmcp` crate) — AI agents connect natively
- Secondary API: Embeddable Rust library for direct integration
- MCP transport: stdio (local) + streamable-http (remote)

### 6. Query Router
- Orchestration layer between Tantivy and DeepCausality
- Classifies query intent → routes to appropriate engine → composes unified response
- The `query` MCP tool — agent sends natural language, router handles the rest

---

## MCP Tool Design

### Tools (actions AI agents can call)
- `search_specs(query, filters?)` → full-text search results
- `get_spec(id)` → full spec content
- `trace_impact(id, depth?)` → causal impact chain
- `find_dependencies(id)` → upstream specs
- `add_spec(markdown)` → ingest new spec
- `add_causal_link(from, to, type, trust?)` → add causal relationship
- `query(natural_language)` → smart routed query
- `sync(mode?)` → sync index with git (full or incremental)

### Resources (data AI agents can read)
- `spec://{id}` → individual spec
- `graph://overview` → causal graph summary
- `graph://node/{id}` → node with edges

### Prompts (guided workflows)
- `impact_analysis(spec_id)` → guided impact analysis
- `spec_review(spec_id)` → completeness check

---

## Data Recovery Model

| Data | Source of Truth | Recoverable? |
|------|----------------|-------------|
| Spec content (markdown) | **Git** | Always — `git clone` + re-index |
| Tantivy search index | Derived from Git | Fully rebuildable |
| Human causal edges (`depends_on`) | **Git** (frontmatter) | Re-parsed from specs |
| AI-inferred causal edges | **Fjall** + `.spec-db/edges.yaml` | From git export |

---

## Phased Delivery Plan (from PM John)

| Phase | What Ships | Value Delivered |
|-------|-----------|----------------|
| **P0** | Fjall store + Tantivy search + MCP server + git sync | AI agents can search specs by text |
| **P1** | DeepCausality graph + human-curated edges + query router | Agents can trace causal chains |
| **P2** | AI-inferred edges + CSM validation + trust scoring | Self-growing knowledge base |

---

## Spec Lifecycle (Git-Centric Workflow)

```
1. Author writes spec       → specs/auth-jwt.md
2. git add + commit          → version tracked
3. spec-db sync()            → indexed + graphed
4. AI agent queries          → search / trace_impact

── AI-Initiated Flow ──
5. Agent infers edge         → add_causal_link()
6. Edge stored in Fjall      → trust=0.x
7. Periodic export           → .spec-db/edges.yaml
8. Human reviews             → git commit (promoted)

── Spec Update Flow ──
9. Human edits spec          → git commit
10. git hook / sync()        → re-index changed spec
11. trace_impact(id)         → "what else changed?"
```

---

## Acceptance Criteria (from SM Bob)

- AC-1: `specs/*.md` in git repo is the single source of truth for spec content
- AC-2: `spec-db sync` rebuilds index from git — zero data loss
- AC-3: Incremental sync processes only changed files (`git diff`)
- AC-4: `depends_on` frontmatter auto-creates causal edges on ingest
- AC-5: AI-inferred edges exportable to git (`.spec-db/edges.yaml`)
- AC-6: Deleting a spec from git removes it from index + graph
- AC-7: spec-db is fully recoverable from `git clone` + `spec-db rebuild`

---

## Repository Structure

```
spec-db-repo/
├── specs/                      # git-tracked specs
│   ├── auth/
│   │   ├── jwt-validation.md
│   │   └── token-issuance.md
│   ├── api/
│   │   └── rate-limiting.md
│   └── ...
├── .spec-db/                   # git-tracked metadata
│   ├── edges.yaml              #   AI-inferred edges (exported)
│   └── config.yaml             #   spec-db configuration
├── .gitignore                  # ignore runtime data
└── data/                       # NOT in git (runtime)
    ├── tantivy/                #   search index (rebuildable)
    └── fjall/                  #   causal graph persistence
```

---

## Example Spec Format

```markdown
---
id: "spec::auth::jwt-validation"
title: "JWT Token Validation Specification"
version: 2
tags: ["auth", "security", "api"]
depends_on: ["spec::auth::token-issuance"]
owner: "backend-team"
created: 2026-02-17
---

# JWT Token Validation

## Overview
All API endpoints requiring authentication MUST validate
JWT tokens according to this specification...

## Acceptance Criteria
- AC-1: Tokens with expired `exp` claim MUST be rejected
- AC-2: Tokens with invalid signature MUST return 401
```
