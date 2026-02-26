# Lattice Documentation Index

> A causal specification database for AI agents

---

## Project Overview

| Attribute | Value |
|-----------|-------|
| **Type** | Monorepo with 2 parts |
| **Primary Language** | Rust (Edition 2024, MSRV 1.85) |
| **Secondary Language** | TypeScript (Svelte 5) |
| **Architecture** | Service-oriented modular monolith |

### Parts

| Part | Type | Technology | Root Path |
|------|------|------------|-----------|
| **Backend** | backend | Rust + Tantivy + Fjall + DeepCausality | `/` |
| **Web UI** | web | SvelteKit 2 + @xyflow/svelte | `/web-ui` |

---

## Quick Reference

### CLI Commands
```bash
lattice init      # Scaffold project structure
lattice serve     # Start MCP server + web UI
lattice sync      # Incremental sync from git
lattice rebuild   # Full index rebuild
lattice status    # Show index status
```

### MCP Tools
| Tool | Purpose |
|------|---------|
| `search_specs` | Full-text search |
| `get_spec` | Get spec by ID |
| `trace_impact` | Trace downstream impact |
| `find_dependencies` | Find upstream dependencies |
| `query` | Natural language query |

### Key Ports
| Service | Default |
|---------|---------|
| Web UI | `http://127.0.0.1:3000` |
| MCP stdio | Always-on |

---

## Generated Documentation

### Project-Wide
- [Project Overview](./project-overview.md) - Executive summary and capabilities
- [Source Tree Analysis](./source-tree-analysis.md) - Directory structure and organization
- [Integration Architecture](./integration-architecture.md) - How parts communicate
- [Project Context (AI Agents)](./project-context.md) - Critical rules for AI implementation

### Backend (Rust)
- [Backend Architecture](./architecture-backend.md) - Crate structure and patterns
- [Backend Development Guide](./development-guide-backend.md) - Setup and workflows

### Web UI (Svelte)
- [Web UI Architecture](./architecture-web-ui.md) - Component structure and tech stack
- [Web UI Development Guide](./development-guide-web-ui.md) - Setup and workflows

### Metadata
- [Project Parts](./project-parts.json) - Machine-readable project structure

---

## Existing Planning Artifacts

| Document | Description |
|----------|-------------|
| [Architecture Decision Document](../_bmad-output/planning-artifacts/architecture.md) | Comprehensive architecture decisions |
| [Product Requirements Document](../_bmad-output/planning-artifacts/prd.md) | Product requirements |
| [Epics & Stories](../_bmad-output/planning-artifacts/epics.md) | Implementation roadmap |
| [Phase 2 Epics](../_bmad-output/planning-artifacts/epics-phase2.md) | Future work |
| [Product Brief](../_bmad-output/planning-artifacts/product-brief-lattice-2026-02-17.md) | Original product vision |
| [UX Design Specification](../_bmad-output/planning-artifacts/ux-design-specification.md) | UI/UX patterns |
| [Technical Research](../_bmad-output/planning-artifacts/research-technical-lattice.md) | Technology evaluation |

---

## Getting Started

### For Development

```bash
# Clone and build
git clone <repo-url> && cd spec-db
cargo build

# Create test project
mkdir /tmp/my-specs && cd /tmp/my-specs && git init
lattice init
git add -A && git commit -m "init"
lattice sync
lattice serve
```

### For AI Agents

1. Read [Project Context](./project-context.md) before implementing
2. Reference [Backend Architecture](./architecture-backend.md) for Rust work
3. Reference [Web UI Architecture](./architecture-web-ui.md) for frontend work
4. Check [Integration Architecture](./integration-architecture.md) for cross-part changes

### For Brownfield PRD

When planning new features, provide this index as context input to the PRD workflow.

---

## AI-Assisted Development Guidance

### Before Implementing

1. **Read project-context.md** — Critical rules AI agents must follow
2. **Check architecture docs** — Understand crate boundaries and patterns
3. **Review existing code patterns** — Match style and conventions

### Key Constraints

- **Error handling**: thiserror in libs, anyhow at binary level
- **Async boundary**: MCP/web handlers async, subsystems sync with `spawn_blocking`
- **SpecId validation**: Always use `SpecId::try_new()`, never construct directly
- **No type suppression**: Never use `as any`, `@ts-ignore`, `@ts-expect-error`

### Testing Requirements

```bash
# Before committing
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

*Documentation generated: 2026-02-27 | Scan level: Quick | Workflow: document-project v1.2.0*
