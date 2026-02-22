---
stepsCompleted:
  - step-01-init
  - step-02-discovery
  - step-03-success
  - step-04-journeys
  - step-05-domain
  - step-06-innovation
  - step-07-project-type
  - step-08-scoping
  - step-09-functional
  - step-10-nonfunctional
  - step-11-polish
  - step-12-complete
completedAt: '2026-02-23T00:00:00Z'
inputDocuments:
  - product-brief-spec-db-2026-02-17.md
  - research-technical-spec-db.md
  - brainstorming-spec-db.md
  - docs/project-context.md
documentCounts:
  briefs: 1
  research: 1
  brainstorming: 1
  projectDocs: 1
classification:
  projectType: developer_tool
  domain: scientific_computational
  complexity: medium
  projectContext: brownfield
workflowType: 'prd'
---

# Product Requirements Document - spec-db

**Author:** Jack
**Date:** 2026-02-22

## Executive Summary

spec-db is a causal specification database for AI agent teams. It combines full-text search (Tantivy) with causal knowledge graph reasoning (DeepCausality + Fjall) to give agents a shared, structured specification layer — enabling them to discover specs, trace architectural impact, propose changes, and maintain a complete evolution record.

**Core Differentiator:** No existing tool combines full-text spec discovery with causal graph reasoning as a native AI agent skill. Search engines return documents; spec-db returns documents *plus their causal web*. The `trace_impact` capability ("if I change this spec, what breaks?") is a fundamentally new primitive for AI-assisted development.

**Two Modes of Intelligence:**
- **Discovery Mode** ("What exists?") — Tantivy-powered full-text search with BM25 scoring, sub-10ms response
- **Reasoning Mode** ("What connects?") — DeepCausality-powered causal chain traversal, impact analysis, dependency graphs

**Target Users:**
- **AI Agents** (primary consumer) — high-frequency MCP tool callers that search, reason, and propose
- **Spec Authors** (primary human) — developers who write specs in markdown + YAML frontmatter and review agent proposals
- **Architects** (secondary) — team leads who monitor spec relationships and maintain architectural coherence

**Technology:** 100% Rust, zero external dependencies, single binary via `cargo install`. Git is the single source of truth; all indexes are derived and fully rebuildable. Exposed via MCP (Model Context Protocol) over stdio and streamable-http transport.

## Success Criteria

### User Success

**AI Agents (Primary Consumer)**
- First `search_specs` call returns the exact relevant spec in < 10ms — this is the adoption hook
- `trace_impact` becomes habitual: agents check blast radius before proposing any spec modification
- Token consumption for spec-related context drops measurably vs. file-reading/grep approaches
- Agents prefer spec-db MCP tools over reading raw files within the first session

**Spec Author (Wei)**
- Write-to-queryable latency < 30 seconds (git commit + `sync`)
- Agent-proposed spec changes arrive with full `trace_impact` output — review confidence increases
- Complete spec evolution history reconstructable from git at any point

**Architect (Mei)**
- `graph://overview` provides accurate, up-to-date view of spec relationships
- Breaking changes caught by `trace_impact` before reaching production
- `spec-db rebuild` from git produces identical results — zero data lock-in anxiety

### Business Success

| Milestone | Timeframe | Indicator |
|-----------|-----------|-----------|
| Prove the concept | 3 months | Working MVP: search + causal reasoning + MCP + git sync with real specs |
| Prove agent adoption | 6 months | Agent teams actively using spec-db in real workflows; ~500 GitHub stars |
| Prove community interest | 12 months | External contributors, sustained crate download growth |
| Prove the standard | 18+ months | Other projects adopt spec-driven development patterns; 6,000 stars (stretch) |

### Technical Success

- Search latency: < 10ms for full-text queries across 100+ specs
- Startup time: < 1 second to load full causal graph from Fjall into memory
- Rebuild reliability: `git clone` + `spec-db rebuild` = 100% idempotent
- Incremental sync: Changed specs re-indexed via `git diff` in < 5 seconds
- Causal traversal: `trace_impact` correctly traverses human-curated `depends_on` chains
- Query router: Correctly classifies intent and routes to Tantivy, DeepCausality, or both
- MCP integration: AI agents discover and call spec-db tools without friction
- Observability: OpenTelemetry traces and metrics emitted for all key operations

### Measurable Outcomes

- **Aha! moment:** Agent's first `search_specs` returns the right spec instantly instead of grepping — the hook that drives repeat usage
- **Validation moment:** First `trace_impact` reveals a hidden dependency the agent didn't know about — proves causal reasoning value
- **Dogfooding:** spec-db manages its own specifications during development

## Product Scope & Phased Development

### MVP Strategy

**Approach:** Problem-solving MVP — prove the full thesis (search + causal reasoning) ships together or not at all.

**Hard Gate:** Causal reasoning ships with search or nothing ships. There is no "search-only" intermediate release. spec-db without causal reasoning is just another search tool.

**Resource Reality:** Solo side project at 10-15 hours/week. 6-month target timeline. Quality bar stays high — Rust helps here.

### MVP Components (Phase 1)

| Component | What Ships |
|-----------|-----------|
| Fjall Key-Value Store | Persistent storage for causal graph data |
| Tantivy Search Index | Full-text BM25 search over spec documents |
| DeepCausality Causal Graph | In-memory graph with human-curated edges |
| MCP Server (rmcp) | AI agent API — tools, resources over stdio + streamable-http |
| Git Sync Engine | Full rebuild + incremental sync via `git diff` |
| Query Router | Intent classification → route to Tantivy, DeepCausality, or both |
| CLI | init, serve, sync, rebuild, status |
| OpenTelemetry | Traces and metrics for all operations |
| Cross-store consistency | SHA tracking, doc count validation, startup checks |

**Build Order (Risk-First):**

| Order | Subsystem | Rationale | Est. Effort |
|-------|-----------|-----------|-------------|
| 1 | **spec-db-core** | Shared types (SpecId, SpecDoc, CausalEdge) — foundation | Small |
| 2 | **DeepCausality + Fjall** | Riskiest integration — prove graph loads, persists, traverses | Large |
| 3 | **Tantivy Search** | Well-documented, low risk — BM25 indexing and query | Medium |
| 4 | **Spec Ingestion** | Markdown + YAML parsing, frontmatter → graph nodes + search docs | Medium |
| 5 | **Git Sync** | Full rebuild + incremental sync via git2 | Medium |
| 6 | **Query Router** | Intent classification → route to appropriate engine | Medium |
| 7 | **MCP Server (rmcp)** | Wire up tools/resources over stdio | Medium |
| 8 | **CLI** | init, serve, sync, rebuild, status | Small |
| 9 | **OpenTelemetry** | Instrument all key operations | Small |
| 10 | **Cross-store consistency** | SHA tracking, doc count validation, startup checks | Small |

### Phase 2 — Self-Growing Intelligence (Post-MVP)

- AI-inferred causal edges via `add_causal_link` MCP tool
- DeepCausality CSM validation for AI-proposed edges
- Trust scoring system (human=1.0, AI=0.x)
- Edge export to `.spec-db/edges.yaml` for human review
- MCP Prompts (`impact_analysis`, `spec_review`)

### Phase 3 — Ecosystem Expansion

- Embeddable Rust library API (`spec-db-lib`)
- Multi-repo federation — causal graphs spanning organizational boundaries
- CI/CD integration — spec impact checks before merge
- IDE plugins — causal context surfaced in editor
- Docs site (mdbook)

## User Journeys

### Journey 1: The Agent That Stops Guessing — AI Agent Success Path

An AI agent is tasked with implementing a new rate-limiting feature for an API. In the old world, it `grep`s through 47 files, reads 12 that look vaguely relevant, burns 30,000 tokens building context, and still misses that rate-limiting has a dependency on the authentication token-issuance spec.

Today, the agent calls `search_specs("rate limiting API")`. In 6ms, it gets back the exact spec with BM25 relevance scoring. It reads the spec via `get_spec("spec::api::rate-limiting")`. Before proposing any changes, it calls `trace_impact("spec::api::rate-limiting")` — and discovers that rate-limiting constrains `spec::api::gateway-routing` and `spec::billing::usage-metering`. Two dependencies it would have missed entirely.

The agent drafts its implementation respecting all three specs. When it proposes a spec modification to add burst-rate support, the proposal includes the full impact chain. Wei reviews the PR and approves in minutes instead of hours — because the blast radius is already mapped.

**Capabilities revealed:** `search_specs`, `get_spec`, `trace_impact`, MCP tool discovery, relevance-ranked results, causal chain traversal.

### Journey 2: The Agent That Hits a Wall — AI Agent Edge Case

An agent calls `search_specs("websocket authentication")` and gets zero results. The spec doesn't exist yet. The agent calls `query("how should websocket connections be authenticated?")` — the query router classifies this as a reasoning query and traverses the causal graph from `spec::auth::jwt-validation`. It returns: "No direct spec exists, but JWT validation is the upstream dependency for all authenticated connections. Related specs: `spec::auth::token-issuance`, `spec::api::gateway-routing`."

The agent now has enough context to draft a new websocket auth spec with proper `depends_on` references. It calls `add_spec(markdown)` to ingest the draft. On the next `sync`, the causal graph automatically reflects the new node and its edges.

**Capabilities revealed:** Zero-result handling, query router hybrid mode, `add_spec`, graceful degradation, causal graph auto-update on ingestion.

### Journey 3: Wei Authors His First Spec

Wei has been writing specs as scattered markdown files with no structure. His team lead Mei has just deployed spec-db. Wei creates his first spec:

```yaml
---
id: "spec::auth::jwt-validation"
title: "JWT Token Validation Specification"
version: 1
tags: ["auth", "security", "api"]
depends_on: ["spec::auth::token-issuance"]
owner: "backend-team"
created: 2026-03-15
---
```

He writes the markdown body, commits, and runs `sync`. In under 30 seconds, his spec is indexed and queryable. He asks the team's AI agent: "What specs relate to authentication?" — and his new spec appears instantly in the results.

A week later, an agent working on a payment feature proposes modifying the JWT validation spec to add scope-based claims. The proposal arrives in a PR with `trace_impact` output showing exactly what's affected: token-issuance, gateway-routing, and three downstream API specs. Wei reviews the change with full context — no guesswork about what might break. He approves with confidence.

**Capabilities revealed:** Spec authoring format, git-centric workflow, `sync`, frontmatter-to-graph auto-creation, agent proposals with impact analysis, PR review workflow.

### Journey 4: Wei Debugs a Broken Sync

Wei runs `sync` after a large refactor and notices that two specs show stale data in search results. He checks the OpenTelemetry traces and sees that incremental sync skipped those files because the `git diff` output didn't include them (they were renamed, not modified).

He runs `sync(mode: "full")` — a complete rebuild from the git tree. In under 5 seconds, the index is reconstructed. Search results now reflect the renamed specs correctly. The causal graph edges are intact because spec IDs didn't change, only file paths.

**Capabilities revealed:** `sync` full vs. incremental modes, OpenTelemetry observability, rebuild reliability, edge case in `git diff` handling, recovery path.

### Journey 5: Mei Evaluates and Deploys spec-db

Mei manages a 6-person dev team with integrated AI agents. Specs are scattered across 4 directories with no relationships. Agents read entire repos to find context — slow, expensive, and they still miss dependencies.

Mei evaluates spec-db. She adds it to the team's MCP config, runs `spec-db rebuild` on the existing specs directory. In seconds, the full index and causal graph are built. She calls `graph://overview` — a summary shows 47 specs, 23 causal edges (from `depends_on` frontmatter), and 3 disconnected clusters (specs with no relationships).

Those 3 disconnected clusters are a red flag. Mei reviews them — they're legacy specs that never declared dependencies. She adds `depends_on` to their frontmatter, commits, syncs. The graph now shows a connected architecture.

From this point, every agent on the team has spec intelligence. The first time an agent catches a breaking change via `trace_impact` before it ships — Mei knows the investment paid off.

**Capabilities revealed:** Team-wide deployment, `rebuild`, `graph://overview`, disconnected cluster detection, architectural visibility, zero-config agent adoption via MCP.

### Journey Requirements Summary

| Journey | Key Capabilities Revealed |
|---------|--------------------------|
| Agent Success Path | `search_specs`, `get_spec`, `trace_impact`, MCP discovery, BM25 relevance |
| Agent Edge Case | Query router hybrid mode, zero-result handling, `add_spec`, graceful degradation |
| Wei — First Spec | Spec format, git workflow, `sync`, frontmatter→graph, agent proposals with impact |
| Wei — Broken Sync | `sync` modes, OpenTelemetry traces, full rebuild recovery, rename edge case |
| Mei — Deployment | `rebuild`, `graph://overview`, disconnected cluster detection, team-wide MCP adoption |

## Domain-Specific Requirements

### Causal Graph Correctness

Validation strategy: Human review + AI analysis.
- Human-curated edges (`depends_on` in frontmatter) are trust=1.0 by definition — correctness depends on the author
- `graph://overview` exposes disconnected clusters (specs with no edges) as a signal for missing relationships
- Post-MVP (P2): AI-inferred edges validated by DeepCausality's Causal State Machine before acceptance
- Completeness is a human responsibility; spec-db surfaces gaps but doesn't guarantee completeness

### Search Relevance Tuning

Vanilla Tantivy with field boosting is sufficient for MVP:
- Title field boosted over body field (title match = higher relevance)
- Tags field as exact-match filter (STRING, not TEXT)
- BM25 scoring handles relevance ranking out of the box
- No custom tokenization or domain-specific analyzers needed at this scale
- Post-MVP: Evaluate if query patterns reveal need for custom boosting or synonyms

### Cross-Store Data Integrity

Two derived stores (Tantivy search index + Fjall/DeepCausality causal graph) must stay consistent with git as source of truth.

**Detection strategy (MVP):**
- Both stores record the git commit SHA they were last built from
- Both stores track their document count
- On startup: compare SHA + doc count across stores → fail-fast if drift detected
- After every `sync`: verify SHA + doc count match → warn if drift detected
- OpenTelemetry metrics: `spec_db.consistency.check_result`, `spec_db.consistency.drift_detected`

**Recovery strategy:**
- `sync(mode: "full")` rebuilds both stores atomically from git tree walk
- At hundreds of specs, full rebuild < 5 seconds — cheap enough to be the default recovery path
- Atomic rebuild: build in temp directories, verify consistency, then swap — prevents partial rebuilds

**Incremental sync robustness:**
- Supplement `git diff` with git's rename detection (`-M` flag) to catch file renames
- After incremental sync, compare doc counts as a sanity check
- If counts diverge, auto-escalate to full rebuild

### Domain Risk Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Silent store drift | Agents get stale/inconsistent results | Startup + post-sync consistency checks with fail-fast |
| Incremental sync misses renames | Stale specs in search index | Git rename detection + doc count sanity check |
| Fjall corruption | Causal graph lost | Full rebuild from git (< 5 seconds); Fjall has LSM-tree crash recovery |
| Tantivy index corruption | Search broken | Full rebuild from git; index is fully derived |
| Spec ID collision | Graph edges point to wrong node | Enforce unique ID validation during ingestion |

## Innovation & Novel Patterns

### Detected Innovation Areas

**1. Causal Knowledge Graph for Software Specifications (Core — No Prior Art)**
No existing tool combines full-text spec discovery with causal graph reasoning for AI agents. Current developer tools offer either search (Sourcegraph, grep) or graph databases (Neo4j, Dgraph) — but none merges both into a single spec-native intelligence layer exposed as an agent skill.

**2. Agent-Native Spec Intelligence (Interaction Model)**
spec-db is designed from the ground up as an MCP server. Agents don't query spec-db like a database; they think with it. The search → reason → propose → trace workflow is a new interaction paradigm where specifications become a first-class agent capability.

**3. Spec-Driven Development as a Discipline (Paradigm)**
spec-db proposes that specifications should be structured, versioned, causally linked, and machine-queryable. This is an emerging engineering discipline with no reference implementation. spec-db aims to be that reference implementation.

**4. Self-Growing Intelligence (Long-term, P2)**
Hybrid human + AI causal edge creation means the knowledge graph compounds in value over time. Human-curated edges (trust=1.0) anchor truth; AI-inferred edges (CSM-validated, trust=0.x) expand coverage.

### Competitive Landscape

| Category | Closest Tools | How spec-db Differs |
|----------|--------------|---------------------|
| Code search | Sourcegraph, ripgrep | Searches *specifications*, not code — adds causal reasoning |
| Knowledge graphs | Neo4j, Dgraph | Spec-native, git-centric, MCP-exposed — not general-purpose |
| Documentation | Notion, Confluence | Treats specs as structured, machine-parseable data |
| AI agent tools | LangChain, CrewAI | Provides *domain intelligence* — not agent orchestration |

**No direct competitor exists.** Genuinely green-field category.

### Innovation Validation & Risk

**Validation approach:**
- MVP ships search + causal reasoning together — validate full thesis from day one
- Dogfooding: spec-db manages its own specs during development
- Agent adoption signal: Do agents call `trace_impact` before modifications?
- Community signal: GitHub traction indicates whether the concept resonates

| Innovation Risk | Fallback | Why It's Still Viable |
|----------------|----------|----------------------|
| Agents don't use `trace_impact` enough | Search-only spec-db | Git-centric spec search with version management and pruning is valuable standalone |
| Causal graph too complex for adoption | Ship without causal layer | Tantivy search + MCP + git sync is useful on its own |
| "Spec-driven development" doesn't resonate | Position as "spec search for agents" | Narrower framing, same technology |
| DeepCausality integration proves difficult | Use petgraph as fallback | Simpler graph primitives; CSM validation deferred |

## Developer Tool Specific Requirements

### Installation & Setup

**Installation:**
```bash
cargo install spec-db
```

**Initialization:**
```bash
spec-db init
```
- Scaffolds `specs/` directory with example spec files
- Creates `.spec-db/config.yaml` with sensible defaults
- Generates starter specs demonstrating frontmatter format, `depends_on` relationships, and tag conventions
- Outputs next-steps instructions (add to MCP config, write first spec, run sync)

**Running:**
```bash
spec-db serve
```
- Reads `.spec-db/config.yaml` for configuration
- Starts MCP server (stdio by default, streamable-http optional)
- Runs initial `sync` on first startup
- Performs cross-store consistency check before serving

**MCP Client Configuration:**
```json
{
  "mcpServers": {
    "spec-db": {
      "command": "spec-db",
      "args": ["serve"]
    }
  }
}
```

### API Surface (MVP)

**Primary API: MCP Protocol**

| Tool | Type | Description |
|------|------|-------------|
| `search_specs(query, filters?)` | Discovery | Full-text BM25 search with optional tag/field filters |
| `get_spec(id)` | Discovery | Retrieve full spec content by ID |
| `trace_impact(id, depth?)` | Reasoning | Causal impact chain traversal downstream |
| `find_dependencies(id)` | Reasoning | Upstream dependency traversal |
| `add_spec(markdown)` | Mutation | Ingest new spec document |
| `query(natural_language)` | Hybrid | Smart-routed query (discovery + reasoning) |
| `sync(mode?)` | Admin | Sync index with git (full or incremental) |

| Resource | Description |
|----------|-------------|
| `spec://{id}` | Individual spec content |
| `graph://overview` | Causal graph summary statistics + disconnected clusters |
| `graph://node/{id}` | Node with all inbound/outbound edges |

**Secondary API: Rust Library — Deferred to Phase 3**

### CLI Commands (MVP)

| Command | Description |
|---------|-------------|
| `spec-db init` | Scaffold project structure with example specs and config |
| `spec-db serve` | Start MCP server from config |
| `spec-db sync [--full]` | Manual sync trigger (full rebuild or incremental) |
| `spec-db rebuild` | Full index rebuild from git (destructive, idempotent) |
| `spec-db status` | Show index health: doc count, last sync commit, consistency check |

### Documentation Strategy (MVP)

- **README.md** — Installation, quick start, MCP config, spec format reference, CLI commands
- **Example specs** — Shipped via `spec-db init`, demonstrating frontmatter conventions
- **Inline `--help`** — Each CLI command has comprehensive help text
- **Post-MVP:** rustdoc for library API, mdbook for guides

### Implementation Considerations

- **Single binary** — `cargo install` produces one binary with all dependencies statically linked
- **Zero external services** — No databases, no Docker, no cloud accounts
- **Config-driven** — `.spec-db/config.yaml` controls all behavior
- **Graceful first-run** — First `serve` triggers automatic full sync if no index exists
- **Cross-platform** — Linux, macOS, Windows without platform-specific code

## Project Risk Mitigation

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| DeepCausality integration harder than expected | Medium-High | Blocks entire project | Tackle first (build order #2). If stuck after 3 weeks, evaluate petgraph |
| DeepCausality API unstable or poorly documented | Medium | Slows development | Study source code early; engage maintainers if needed |
| Fjall ↔ DeepCausality serialization issues | Medium | Delays graph persistence | Prototype serialization roundtrip before full integration |
| rmcp breaking changes | Low | Delays MCP layer | Pin version; MCP layer is last in build order |

### Market Risks

| Risk | Mitigation |
|------|------------|
| "Spec-driven development" doesn't resonate | Dogfood immediately — if it's useful for you, it's useful for others |
| No community adoption | Focus on README quality, `spec-db init` experience, one compelling demo |
| Agents don't adopt spec-db tools | Test with Claude Code / OpenCode during development continuously |

### Resource Risks

| Risk | Mitigation |
|------|------------|
| Side project momentum stalls | Risk-first build order means highest-value learning happens first |
| 6-month timeline slips | MVP scope is fixed. Timeline slips before scope cuts — don't cut causal reasoning |
| Burnout on solo project | Ship small wins: each subsystem is independently testable and demoable |

## Functional Requirements

### Spec Discovery

- **FR1:** Agents can search specs by keyword with BM25 relevance-ranked results
- **FR2:** Agents can filter search results by tag
- **FR3:** Agents can retrieve the full content of a spec by its ID
- **FR4:** Agents can receive title-boosted search results (title matches rank higher than body matches)
- **FR5:** Agents can receive search results that include spec ID, title, tags, and relevance score

### Causal Reasoning

- **FR6:** Agents can trace the downstream causal impact of a spec ("if I change this, what breaks?")
- **FR7:** Agents can discover the upstream dependencies of a spec ("what does this depend on?")
- **FR8:** Agents can specify traversal depth when tracing impact or dependencies
- **FR9:** The system automatically creates causal edges from `depends_on` fields in spec frontmatter
- **FR10:** Agents can view a spec's node with all its inbound and outbound causal edges

### Hybrid Intelligence

- **FR11:** Agents can submit natural-language queries that are automatically routed to search, causal reasoning, or both
- **FR12:** The query router provides causal context when search returns zero results
- **FR13:** Agents can receive composed results that combine search results with causal context

### Spec Lifecycle

- **FR14:** Agents can ingest a new spec document (markdown + YAML frontmatter)
- **FR15:** The system parses spec frontmatter to extract ID, title, version, tags, depends_on, owner, and created date
- **FR16:** The system indexes spec content for full-text search on ingestion
- **FR17:** The system creates causal graph nodes and edges on ingestion
- **FR18:** The system validates spec IDs for uniqueness during ingestion
- **FR19:** Spec authors can write specs in markdown with YAML frontmatter following a defined format

### Git Integration

- **FR20:** The system can perform a full rebuild of all indexes from a git repository tree walk
- **FR21:** The system can perform incremental sync by processing only changed files via `git diff`
- **FR22:** The system detects renamed files during incremental sync using git rename detection
- **FR23:** The system removes specs from indexes and graph when deleted from git
- **FR24:** Full rebuild produces identical indexes regardless of when it is run (idempotent)
- **FR25:** The system tracks the last-synced git commit SHA for both stores

### Agent Integration (MCP)

- **FR26:** The system exposes all capabilities as MCP tools over stdio transport
- **FR27:** The system optionally exposes MCP tools over streamable-http transport
- **FR28:** Agents can discover available spec-db tools through MCP protocol
- **FR29:** Agents can read individual spec content via `spec://{id}` resource
- **FR30:** Agents can read causal graph summary statistics via `graph://overview` resource
- **FR31:** Agents can read a specific node with all edges via `graph://node/{id}` resource
- **FR32:** The `graph://overview` resource exposes disconnected clusters (specs with no causal edges)

### System Administration (CLI)

- **FR33:** Users can initialize a new spec-db project with scaffolded directory structure and example specs
- **FR34:** Users can start the MCP server from a configuration file
- **FR35:** Users can manually trigger sync (full or incremental) via CLI
- **FR36:** Users can perform a destructive full index rebuild via CLI
- **FR37:** Users can view index health status (document count, last sync commit, consistency check result)
- **FR38:** The system reads all configuration from `.spec-db/config.yaml`

### Data Integrity

- **FR39:** The system verifies cross-store consistency (Tantivy vs. Fjall) on startup
- **FR40:** The system verifies cross-store consistency after every sync operation
- **FR41:** The system compares git commit SHA and document count across both stores to detect drift
- **FR42:** The system warns and offers auto-rebuild when drift is detected
- **FR43:** The system auto-escalates to full rebuild when incremental sync doc counts diverge

### Observability

- **FR44:** The system emits OpenTelemetry traces for search queries, graph traversals, sync operations, and MCP tool calls
- **FR45:** The system emits OpenTelemetry metrics for search latency, sync duration, consistency check results, and document counts
- **FR46:** The system emits drift detection metrics when cross-store inconsistency is found

## Non-Functional Requirements

### Performance

| Metric | Target | Context |
|--------|--------|---------|
| Full-text search latency | < 10ms | Queries across 100+ specs |
| Causal graph traversal | < 50ms | `trace_impact` and `find_dependencies` at depth ≤ 5 |
| Query router classification | < 5ms | Intent classification overhead |
| Startup time (graph load) | < 1 second | Full causal graph from Fjall into memory |
| Full rebuild | < 5 seconds | Complete index + graph from git for 100+ specs |
| Incremental sync | < 2 seconds | Changed files only via `git diff` |
| Spec ingestion | < 100ms per spec | Parse, index, and graph a single spec |
| MCP tool response | < 100ms end-to-end | Any single tool call |
| Memory footprint | < 50MB | 100+ specs with full causal graph |
| Binary size | < 30MB | Single statically-linked binary |

### Reliability

- **Rebuild idempotency:** `git clone` + `spec-db rebuild` produces bit-identical indexes every time
- **Crash recovery:** Fjall's LSM-tree provides durability — no data loss on unexpected shutdown
- **Zero data lock-in:** All state derived from git. Deleting `data/` and rebuilding loses nothing
- **Graceful degradation:** If causal graph fails to load, search-only mode with clear warning
- **Atomic rebuilds:** Temp directories + swap prevents serving partially-built indexes
- **Error propagation:** All errors surface clearly — no silent failures

### Integration

- **MCP protocol compliance:** Full compatibility with MCP spec (version 2025-11-25)
- **Git compatibility:** Any standard git repository via libgit2 / git2 crate
- **Cross-platform:** Linux, macOS, Windows without platform-specific code
- **Stdio transport:** Default for local MCP — zero network configuration
- **Streamable-http transport:** Optional for remote access, configurable
- **OpenTelemetry export:** Standard OTLP protocol — compatible with Jaeger, Grafana, Datadog

### Security

- **Local-first by default:** stdio has no network surface — specs never leave the machine unless configured
- **Streamable-http auth:** When http enabled, token-based authentication required
- **No telemetry home:** No data sent to external services — OpenTelemetry export is opt-in
- **File system scoping:** Reads only configured spec directories, writes only configured data directories
- **No code execution:** Parses markdown and YAML only — never evaluates spec content

### Scalability (Design Limits)

- **Target scale:** Hundreds of specs (100-500) per team
- **Upper bound:** Thousands of specs without architectural changes
- **Not designed for:** Millions of documents, multi-tenant SaaS, cross-org federation
- **Concurrent access:** Single MCP server process; concurrent agent access deferred to post-MVP
