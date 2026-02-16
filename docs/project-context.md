# Project Context: spec-db

**Last Updated:** 2026-02-17

---

## What Is spec-db

A causal specification database for AI agents. Combines full-text search (Tantivy) with causal reasoning (DeepCausality + Fjall) to let AI agents discover specs and trace architectural impact, exposed via MCP (Model Context Protocol).

---

## Critical Architectural Decisions

### 1. Git Is Source of Truth
- All spec content lives as markdown files in a git repository
- spec-db indexes are **derived** — fully rebuildable from `git clone` + `spec-db rebuild`
- Runtime data (`data/tantivy/`, `data/fjall/`) is NOT in git
- AI-inferred causal edges exported to `.spec-db/edges.yaml` in git

### 2. Two Subsystems, Not Three
- **Tantivy** = independent search index (owns its own storage)
- **DeepCausality + Fjall** = causal reasoning (Fjall is DeepCausality's persistence backend)
- These are NOT three independent layers — Fjall serves DeepCausality specifically

### 3. Full In-Memory Graph
- Entire causal graph loaded from Fjall into memory at startup
- Target scale: hundreds of specs (single-digit MB in memory)
- All graph operations happen in-memory for maximum speed
- Fjall handles durability and crash recovery

### 4. Hybrid Causal Edge Creation
- Human edges: `depends_on` in YAML frontmatter → trust=1.0
- AI edges: via `add_causal_link` MCP tool → trust=0.x, CSM-validated
- DeepCausality's Causal State Machine validates AI-proposed edges

### 5. MCP as Primary API
- AI agents communicate via MCP protocol (rmcp crate)
- Transport: stdio (local) + streamable-http (remote)
- Rust library API available for direct embedding

---

## Technology Stack (Locked)

| Component | Crate | Version |
|-----------|-------|---------|
| Search | `tantivy` | 0.22+ |
| KV Storage | `fjall` | 2.x |
| Causal Reasoning | `deep_causality` | 0.13+ |
| MCP Server | `rmcp` | 0.8+ |
| Git Access | `git2` | latest |
| Markdown | `pulldown-cmark` | 0.12+ |
| YAML | `serde_yaml` | 0.9+ |
| Async Runtime | `tokio` | 1.x |

**Language:** Rust (100%)
**External dependencies:** None — fully embeddable, no servers needed

---

## Spec Document Format

Markdown with YAML frontmatter:

```markdown
---
id: "spec::domain::name"
title: "Human Readable Title"
version: 1
tags: ["tag1", "tag2"]
depends_on: ["spec::other::spec"]
owner: "team-name"
created: 2026-02-17
---

# Spec Title

Body content in markdown...
```

### Frontmatter Fields
- `id` (required) — unique spec identifier, used as key across all subsystems
- `title` (required) — human-readable title
- `version` (required) — spec version number
- `tags` (optional) — categorization tags
- `depends_on` (optional) — array of spec IDs this spec depends on → auto-creates causal edges
- `owner` (optional) — team or person responsible
- `created` (required) — creation date

---

## Repository Layout

```
specs/                    # git-tracked spec markdown files
.spec-db/                 # git-tracked spec-db metadata
  edges.yaml              # AI-inferred edges (exported for review)
  config.yaml             # spec-db configuration
data/                     # NOT in git — runtime data
  tantivy/                # search index (rebuildable)
  fjall/                  # causal graph persistence
```

---

## Key Patterns for AI Agents

1. **Shared ID Scheme** — `SpecId` is the canonical identifier used by Tantivy (stored field), Fjall (key), and DeepCausality (node ID)
2. **Incremental Sync** — use `git diff` between last-synced commit and HEAD to process only changed specs
3. **Cross-keyspace atomic writes** — when adding a node, atomically write node + edges via Fjall batch
4. **Query Router** — classify query intent before routing to Tantivy, DeepCausality, or both in sequence

---

## Phased Delivery

- **P0:** Tantivy search + Fjall store + MCP server + git sync → agents can search specs
- **P1:** DeepCausality graph + human edges + query router → agents can trace causal chains
- **P2:** AI-inferred edges + CSM validation + trust scoring → self-growing knowledge base
