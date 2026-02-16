---
stepsCompleted: [1, 2, 3, 4, 5, 6]
status: complete
inputDocuments:
  - brainstorming-spec-db.md
  - research-technical-spec-db.md
  - docs/project-context.md
  - "https://github.com/fjall-rs/fjall"
  - "https://github.com/deepcausality-rs/deep_causality"
  - "https://github.com/quickwit-oss/tantivy"
date: 2026-02-17
author: Jack
---

# Product Brief: spec-db

## Executive Summary

spec-db is a causal specification database purpose-built for AI agent teams. It combines full-text search (Tantivy) with causal knowledge graph reasoning (DeepCausality + Fjall) to give agent teams a shared, structured specification layer — enabling agents to discover specs, trace architectural impact, propose changes, and maintain a complete evolution record. Exposed via MCP (Model Context Protocol) as a native agent skill, spec-db transforms software specification management from a manual, fragmented process into an engineering-grade, automated discipline. Written in 100% Rust with zero external dependencies, it treats git as the single source of truth while maintaining a derived causal index that grows smarter over time through hybrid human + AI edge creation.

---

## Core Vision

### Problem Statement

As AI-assisted development evolves from single-agent interactions to coordinated agent teams, a critical infrastructure gap has emerged: there is no structured way for multiple agents to efficiently discover, understand, and evolve project specifications. Today, agents grep through files, read entire repositories, and operate with no visibility into the causal relationships between specs, architecture decisions, and implementations. When an agent team works on a project, each agent rebuilds context from scratch — a wasteful, error-prone process that collapses as project complexity grows.

### Problem Impact

- **Agent teams lack shared memory**: Each agent re-reads the entire codebase to understand what already exists, wasting tokens and time
- **No causal visibility**: Changes to one specification can silently break downstream components with no automated way to trace impact
- **No structured spec evolution**: Agents follow specs rigidly or propose ad-hoc changes with no formal proposal, review, or versioning workflow
- **Knowledge is lost**: As specs evolve through rapid iteration, the reasoning behind decisions disappears — there is no audit trail connecting "why" to "what"

### Why Existing Solutions Fall Short

Current approaches treat specifications as static text files. Search is limited to keyword matching — asking "what specs relate to authentication?" returns only documents containing that word, not the causal web of dependencies, constraints, and downstream impacts. No existing tool combines full-text discovery with causal graph reasoning, and none is designed as a native agent skill that fits ergonomically into multi-agent workflows. Existing knowledge graph tools lack the spec-native domain model, git-centric workflow, and MCP interface that agent teams require.

### Proposed Solution

spec-db provides two distinct modes of intelligence for agent teams:

- **Discovery Mode** ("What exists?") — Powered by Tantivy, agents search specs by keyword, tag, or natural language with sub-10ms response times and BM25 relevance scoring
- **Reasoning Mode** ("What connects?") — Powered by DeepCausality + Fjall, agents trace causal chains, impact analysis, and dependency graphs across the entire specification landscape

With git as the source of truth, spec-db supports a structured spec lifecycle: agents read specs, propose modifications, and once approved, the causal graph automatically reflects the change and its ripple effects. AI-inferred causal edges are validated by DeepCausality's Causal State Machine (CSM) and exported to git for human review — creating a knowledge base that grows smarter with every interaction.

### Key Differentiators

1. **Causal Knowledge Graph, Not Just Search** — Querying a feature returns its full causal web: dependencies, downstream impacts, design reasoning chains. This is fundamentally different from text search.
2. **Agent-Native Skill Design** — Built as an MCP server from the ground up, spec-db feels like a native agent capability, not an external service bolted on.
3. **Structured Spec Evolution** — Agents propose spec changes through a formal workflow with complete version history, turning rapid iteration from chaos into engineering discipline.
4. **Self-Growing Intelligence** — Hybrid human + AI causal edge creation means the knowledge graph becomes more valuable over time. Human-curated edges (trust=1.0) anchor the graph; AI-inferred edges (CSM-validated) expand it.
5. **Zero-Dependency Rust Core** — 100% Rust, fully embeddable, no external services. Can be destroyed and fully rebuilt from `git clone` + rebuild.

---

## Target Users

### Primary Users

#### 1. AI Agents (Primary Consumer)

**Profile:** Any MCP-compatible AI agent operating within a development team — Claude Code, OpenCode, Cursor, Windsurf, or custom orchestration layers. These agents are the high-frequency consumers of spec-db, making dozens to hundreds of tool calls per session.

**Core Needs:**
- Rapidly discover relevant specs without reading entire repositories
- Understand causal relationships between specs before making changes
- Propose spec modifications through a structured workflow
- Verify that proposed changes won't break downstream components

**Current Pain:**
- Grep through files or read whole repos to find relevant specs — slow, noisy, token-wasteful
- No visibility into causal dependencies — changes made blind
- No structured way to propose spec modifications — ad-hoc edits with no traceability

**Success Looks Like:**
- One MCP call to `search_specs` returns exactly the relevant specs
- `trace_impact` reveals the full blast radius before any change is proposed
- Spec proposals flow through a reviewable workflow instead of silent file edits

---

#### 2. Spec Author (Primary Human)

**Persona: Wei** — Senior backend developer, 5 years experience. Works on a team of 4 developers with multiple AI agents assisting across the codebase. Wei writes the initial specifications in markdown with YAML frontmatter, defines `depends_on` relationships, and reviews agent-proposed spec changes during PR review.

**Core Needs:**
- Author specs in a format that's both human-readable and machine-parseable
- Review agent-proposed spec changes with full context on why the change was proposed
- Maintain confidence that the causal graph accurately reflects the real architecture
- Track spec evolution history — who changed what, when, and why

**Current Pain:**
- Specs live as scattered markdown files with no structured relationships
- When agents suggest changes, there's no way to assess downstream impact
- Spec evolution is invisible — changes happen in git commits but the "why" is lost

**Success Looks Like:**
- Writes a spec with `depends_on` and the causal graph auto-updates
- Reviews an agent's proposed spec change alongside its `trace_impact` output showing exactly what's affected
- Can reconstruct the full decision history behind any spec at any point

---

### Secondary Users

#### 3. Team Lead / Architect (Decision Maker & Adopter)

**Persona: Mei** — Technical architect and team lead managing a 6-person dev team with an integrated AI agent workflow. Mei makes tooling decisions for the team, defines architectural boundaries, and is responsible for ensuring that rapid agent-driven iteration doesn't compromise system coherence.

**Core Needs:**
- Standardize how specs are created, evolved, and governed across the team
- Maintain architectural integrity as agents rapidly iterate on features
- Visibility into the causal knowledge graph to catch hidden dependency risks
- Confidence that the spec-db toolchain is reliable, recoverable, and low-maintenance

**Current Pain:**
- No single source of truth for how specs relate to each other
- Agents make changes without understanding architectural constraints
- Manual effort to trace impact of changes across the system

**Success Looks Like:**
- Deploys spec-db once, agents across the team immediately gain spec intelligence
- Reviews `graph://overview` to see the health of the entire spec landscape
- Trusts that `spec-db rebuild` from git fully recovers everything — zero lock-in risk

---

### User Journey

| Phase | AI Agent | Spec Author (Wei) | Architect (Mei) |
|---|---|---|---|
| **Adoption** | — | Adds spec-db to MCP config | Evaluates spec-db, decides to standardize for the team |
| **Setup** | — | Authors initial specs in `specs/*.md` with frontmatter | Defines key `depends_on` relationships, sets architectural boundaries |
| **Onboarding** | Discovers spec-db tools via MCP | Runs first `spec-db sync` to build index | Reviews `graph://overview` to validate initial graph |
| **Daily Use** | `search_specs` → `get_spec` → `trace_impact` → `add_causal_link` | Reviews agent-proposed changes in PRs, authors new specs | Monitors causal graph health, reviews AI-inferred edges |
| **Aha Moment** | First `trace_impact` call reveals a hidden dependency | First agent proposal comes with full impact analysis attached | First time an agent catches a breaking change before it ships |
| **Long-term** | Knowledge graph becomes primary context source — faster, cheaper than reading files | Specs become living documents with complete evolution history | Causal graph becomes the team's architectural source of truth |

---

## Success Metrics

spec-db is an open-source experiment in standardizing specification-driven development for AI agent teams. Success is measured across three horizons: proving the technology works, proving agents adopt it, and proving the concept spreads.

### User Success Metrics

**Agent Efficiency (Primary Consumer)**
- Agents resolve spec queries via `search_specs` instead of reading entire repositories
- `trace_impact` is called before spec modifications — agents develop the habit of checking blast radius
- AI-inferred causal edges pass CSM validation at a meaningful rate — the graph grows through use
- Token consumption for spec-related context drops measurably compared to file-reading approaches

**Spec Author Success (Wei)**
- Time from "write a spec" to "spec is indexed and queryable" is under 30 seconds (git commit + sync)
- Agent-proposed spec changes arrive with full impact analysis — review confidence increases
- Complete spec evolution history is reconstructable from git at any point

**Architect Success (Mei)**
- `graph://overview` provides accurate, up-to-date view of spec relationships
- Breaking changes caught by `trace_impact` before reaching production
- `spec-db rebuild` from git produces identical results — zero data anxiety

### Business Objectives

As an open-source project, "business" objectives center on community adoption and ecosystem impact:

| Objective | Description | Timeframe |
|---|---|---|
| **Prove the concept** | Working P0 (search + store + MCP + git sync) with real specs | 3 months |
| **Prove agent adoption** | Agent teams actively using spec-db in real workflows | 6 months |
| **Prove community interest** | Meaningful GitHub traction and external contributors | 12 months |
| **Prove the standard** | Other projects adopt spec-driven development patterns inspired by spec-db | 18+ months |

### Key Performance Indicators

**Technical KPIs (P0 — Does it work?)**
- Search latency: < 10ms for full-text queries across hundreds of specs
- Startup time: < 1 second to load full causal graph from Fjall into memory
- Rebuild reliability: `git clone` + `spec-db rebuild` produces identical index — 100% idempotent
- Sync speed: Incremental sync processes only changed files via `git diff` in < 5 seconds

**Adoption KPIs (P1 — Do agents use it?)**
- Agent tool call frequency: spec-db MCP tools become preferred over file-reading for spec context
- Causal edge growth: AI-inferred edges accumulate and a meaningful percentage survive human review
- Spec evolution velocity: teams with spec-db iterate on specs faster with complete traceability

**Community KPIs (Long-term — Does it spread?)**
- GitHub stars: 6,000 stars milestone (positioning alongside established Rust ecosystem tools)
- Crate downloads: Sustained weekly download growth on crates.io
- External contributions: PRs from non-core contributors
- Ecosystem influence: Other tools or frameworks reference or integrate spec-db patterns

---

## MVP Scope

### Core Features

**Phase 0 — Specification Infrastructure**

| Feature | Description |
|---|---|
| **Fjall Key-Value Store** | Persistent storage backend for causal graph data with cross-keyspace atomic writes, compression, and crash recovery |
| **Tantivy Search Index** | Full-text search over spec documents with BM25 scoring, schema-based indexing (id, title, body, tags, meta), sub-10ms query response |
| **MCP Server (rmcp)** | AI agent API via Model Context Protocol — tools, resources, and prompts exposed over stdio and streamable-http transport |
| **Git Sync Engine** | Bidirectional sync between git repository and spec-db indexes — full rebuild from tree walk and incremental sync via `git diff` |
| **Spec Ingestion Pipeline** | Parse markdown + YAML frontmatter (`pulldown-cmark` + `serde_yaml`), extract metadata, index content, create graph nodes |
| **OpenTelemetry Instrumentation** | Traces and metrics for all spec-db operations — search latency, graph traversal time, sync duration, MCP tool call frequency. Enables teams to observe agent usage patterns and validate adoption KPIs with real data |

**Phase 1 — Causal Reasoning**

| Feature | Description |
|---|---|
| **DeepCausality Causal Graph** | In-memory causal graph loaded from Fjall at startup — nodes (specs, modules, interfaces) and edges (depends_on, constrains, implements) |
| **Human-Curated Edges** | `depends_on` in YAML frontmatter auto-creates causal edges with trust=1.0 on ingestion |
| **Impact Tracing** | `trace_impact(id, depth?)` traverses causal graph downstream — "if I change spec X, what breaks?" |
| **Dependency Discovery** | `find_dependencies(id)` traverses causal graph upstream — "what does spec X depend on?" |
| **Query Router** | Intent classification layer that routes queries to Tantivy, DeepCausality, or both — agents send natural language, router handles orchestration |

**MCP Tool Surface (MVP)**

| Tool | Type | Description |
|---|---|---|
| `search_specs(query, filters?)` | Discovery | Full-text search with optional tag/field filters |
| `get_spec(id)` | Discovery | Retrieve full spec content by ID |
| `trace_impact(id, depth?)` | Reasoning | Causal impact chain traversal |
| `find_dependencies(id)` | Reasoning | Upstream dependency traversal |
| `add_spec(markdown)` | Mutation | Ingest new spec document |
| `query(natural_language)` | Hybrid | Smart-routed query (discovery + reasoning) |
| `sync(mode?)` | Admin | Sync index with git (full or incremental) |

**MCP Resources (MVP)**

| Resource | Description |
|---|---|
| `spec://{id}` | Individual spec content |
| `graph://overview` | Causal graph summary statistics |
| `graph://node/{id}` | Node with all edges |

### Out of Scope for MVP

| Deferred Feature | Rationale | Target Phase |
|---|---|---|
| **AI-Inferred Causal Edges** | Requires CSM validation pipeline and trust scoring — adds complexity without proving core thesis | P2 |
| **Causal State Machine (CSM) Validation** | Needed for AI edge validation, not human-curated edges | P2 |
| **Trust Scoring System** | Only needed when AI edges coexist with human edges | P2 |
| **Edge Export to Git** (`.spec-db/edges.yaml`) | Only needed when AI-inferred edges exist to review | P2 |
| **`add_causal_link` MCP Tool** | Agent-initiated edge creation deferred until trust model is in place | P2 |
| **MCP Prompts** (`impact_analysis`, `spec_review`) | Guided workflows are valuable but not essential for MVP | P2 |
| **Multi-repo Support** | Single repo is sufficient for initial validation | Future |
| **Web UI / Dashboard** | CLI and MCP interface are sufficient; visual tools deferred | Future |

### MVP Success Criteria

| Criteria | Measurement | Gate |
|---|---|---|
| **Search works** | `search_specs` returns relevant results in < 10ms across 100+ specs | Must pass |
| **Causal reasoning works** | `trace_impact` correctly traverses human-curated `depends_on` chains | Must pass |
| **Query router works** | `query` correctly classifies intent and routes to appropriate engine | Must pass |
| **Git sync is reliable** | `spec-db rebuild` from clean git clone produces identical index | Must pass |
| **Incremental sync works** | Changed specs re-indexed without full rebuild via `git diff` | Must pass |
| **MCP integration works** | AI agent (Claude Code / OpenCode) successfully discovers and calls spec-db tools | Must pass |
| **Observability works** | OpenTelemetry traces and metrics emitted for all key operations | Must pass |
| **Real-world validation** | spec-db used to manage its own specifications during development (dogfooding) | Should pass |

### Future Vision

**P2 — Self-Growing Intelligence (Post-MVP)**
- AI agents propose causal edges via `add_causal_link` tool
- DeepCausality's Causal State Machine validates proposed edges before acceptance
- Trust scoring differentiates human-curated (1.0) from AI-inferred (0.x) edges
- AI-inferred edges periodically exported to `.spec-db/edges.yaml` in git for human review
- The knowledge graph becomes a self-improving system — each agent interaction makes it smarter

**Long-Term Vision**
- spec-db becomes the standard specification management layer for AI agent teams
- Ecosystem of integrations: CI/CD pipelines check spec impact before merge, IDE plugins surface causal context
- Multi-repo federation: causal graphs span across organizational boundaries
- Spec-driven development emerges as a recognized engineering discipline — spec-db is the reference implementation
