# Lattice - Project Overview

## Executive Summary

**Lattice** is a causal specification database designed for AI agents. It combines full-text search capabilities (Tantivy) with causal knowledge graph reasoning (DeepCausality + Fjall), exposed as MCP (Model Context Protocol) tools over stdio and HTTP.

The system enables AI agents to:
- **Search** specification documents using full-text search
- **Trace** causal relationships between specs (dependencies, constraints, implementations)
- **Query** using natural language with automatic routing to appropriate subsystems
- **Ingest** new specifications from git-tracked markdown files

## Project Classification

| Attribute | Value |
|-----------|-------|
| **Repository Type** | Monorepo |
| **Parts** | 2 (Backend + Web UI) |
| **Primary Language** | Rust (Edition 2024) |
| **Secondary Language** | TypeScript (Svelte 5) |
| **Architecture Pattern** | Service-oriented with embedded UI |

## Technology Stack Summary

### Backend (Rust)
- **Runtime**: Tokio 1.49 (async multi-threaded)
- **Search**: Tantivy 0.25.0 (full-text search engine)
- **Storage**: Fjall 3.0 (LSM-tree key-value store)
- **Causal Reasoning**: DeepCausality 0.13.4
- **Protocol**: rmcp 0.16.0 (MCP server implementation)
- **HTTP**: Axum 0.8 (web framework)
- **CLI**: Clap 4.5 (command-line parser)

### Web UI (Svelte)
- **Framework**: SvelteKit 2.x
- **UI Library**: Svelte 5.x
- **Graph Visualization**: @xyflow/svelte 1.x + dagre 1.1
- **Build**: Vite 6.x

## Core Capabilities

### MCP Tools (AI Agent Interface)
| Tool | Purpose |
|------|---------|
| `search_specs` | Full-text search across indexed specs |
| `get_spec` | Retrieve spec by ID |
| `trace_impact` | Trace downstream causal impact |
| `find_dependencies` | Find upstream dependencies |
| `query` | Natural language query with auto-routing |
| `add_spec` | Ingest new spec document |
| `sync` | Trigger incremental or full sync |

### MCP Resources
| URI Pattern | Purpose |
|-------------|---------|
| `spec://{id}` | Individual spec content |
| `graph://overview` | Graph statistics |
| `graph://node/{id}` | Node with edges |

### CLI Commands
| Command | Purpose |
|---------|---------|
| `lattice init` | Scaffold project structure |
| `lattice serve` | Start MCP server |
| `lattice sync` | Incremental sync from git |
| `lattice rebuild` | Full index rebuild |
| `lattice status` | Show index status |

## Key Design Decisions

1. **Git as Source of Truth**: All spec content lives as markdown in git; indexes are derived and rebuildable
2. **Validated Newtypes**: `SpecId` pattern enforced at boundaries (`spec::{segment}::{segment}`)
3. **Async Boundary**: MCP/web handlers are async; subsystems (search, causal, ingest) are sync
4. **Embedded UI**: SvelteKit app built and embedded into binary at compile time
5. **Cross-Store Consistency**: SHA + doc count verification after every sync

## Links to Detailed Documentation

- [Source Tree Analysis](./source-tree-analysis.md)
- [Backend Architecture](./architecture-backend.md)
- [Web UI Architecture](./architecture-web-ui.md)
- [Integration Architecture](./integration-architecture.md)
- [Backend Development Guide](./development-guide-backend.md)
- [Web UI Development Guide](./development-guide-web-ui.md)
- [Project Context for AI Agents](./project-context.md)

## Existing Planning Artifacts

- [Architecture Decision Document](../_bmad-output/planning-artifacts/architecture.md)
- [Product Requirements Document](../_bmad-output/planning-artifacts/prd.md)
- [Epics and Stories](../_bmad-output/planning-artifacts/epics.md)
- [UX Design Specification](../_bmad-output/planning-artifacts/ux-design-specification.md)

---

*Generated: 2026-02-27 | Scan Level: Quick*
