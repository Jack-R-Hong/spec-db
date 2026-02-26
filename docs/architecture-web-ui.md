# Web UI Architecture (Svelte)

## Overview

The Lattice web UI is a SvelteKit application that provides a visual interface for exploring the specification graph. It is built as static files and embedded into the Rust binary at compile time.

## Architecture Pattern

**Single-Page Application (SPA)** with:
- SvelteKit for routing and SSG
- Static adapter for embedded deployment
- REST API communication with backend
- Interactive graph visualization

## Technology Stack

| Layer | Technology | Version |
|-------|------------|---------|
| Framework | SvelteKit | 2.x |
| UI Library | Svelte | 5.x |
| Build Tool | Vite | 6.x |
| Graph Rendering | @xyflow/svelte | 1.x |
| Graph Layout | @dagrejs/dagre | 1.1 |
| Markdown | snarkdown | 2.x |
| Type System | TypeScript | 5.x |
| Static Adapter | @sveltejs/adapter-static | 3.x |

## Project Structure

```
web-ui/
├── src/
│   ├── app.html           # HTML template
│   ├── app.css            # Global styles
│   ├── routes/            # SvelteKit routes
│   │   ├── +page.svelte   # Main app page
│   │   └── +layout.svelte # Root layout
│   └── lib/               # Shared code
│       ├── components/    # Reusable UI components
│       ├── stores/        # Svelte stores (state)
│       └── api/           # Backend API client
├── static/                # Static assets
├── dist/                  # Build output (embedded into binary)
├── package.json           # Dependencies
├── svelte.config.js       # SvelteKit configuration
├── vite.config.ts         # Vite build configuration
└── tsconfig.json          # TypeScript configuration
```

## Key Features

### Graph Visualization
- Interactive node-edge graph using @xyflow/svelte
- Automatic layout via dagre algorithm
- Node selection and detail panel
- Edge type visualization (DependsOn, Constrains, Implements)

### Spec Viewer
- Markdown rendering with snarkdown
- Metadata display (tags, version, owner)
- Dependency links
- Impact tracing trigger

### Search Interface
- Full-text search input
- Tag filtering
- Result highlighting

## API Integration

The web UI communicates with the backend via REST API:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/search` | GET | Full-text search |
| `/api/spec/:id` | GET | Get spec by ID |
| `/api/graph/overview` | GET | Graph statistics |
| `/api/graph/node/:id` | GET | Node with edges |
| `/api/impact/:id` | GET | Trace impact |
| `/api/dependencies/:id` | GET | Find dependencies |

## Build & Deployment

### Development
```bash
cd web-ui
npm install
npm run dev      # Vite dev server with HMR
```

### Production Build
```bash
npm run build    # Output to dist/
```

### Embedding into Binary
The `dist/` folder is embedded into the Rust binary using `rust-embed`:

```rust
#[derive(RustEmbed)]
#[folder = "web-ui/dist"]
struct WebAssets;
```

Axum serves these static files at runtime.

## State Management

Uses Svelte's built-in stores:
- `selectedNode` - Currently selected graph node
- `searchResults` - Search query results
- `graphData` - Cached graph structure

## Styling

- Global CSS in `app.css`
- Scoped component styles using `<style>` blocks
- No external CSS framework (lightweight)

## Configuration

### svelte.config.js
- Static adapter for SPA output
- Prerender all routes
- Fallback to index.html

### vite.config.ts
- Svelte plugin configuration
- Build output to `dist/`

---

*Generated: 2026-02-27 | Scan Level: Quick*
