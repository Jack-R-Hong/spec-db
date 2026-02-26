# Web UI Development Guide

## Prerequisites

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Node.js | 20+ | JavaScript runtime |
| npm | 10+ | Package manager |

## Getting Started

### Install Dependencies

```bash
cd web-ui
npm install
```

### Development Server

```bash
npm run dev
```

Opens at `http://localhost:5173` with hot module replacement (HMR).

**Note**: For full functionality, the backend must be running:
```bash
# In another terminal
cargo run -- serve
```

### Type Checking

```bash
npm run check
```

### Production Build

```bash
npm run build
```

Output is placed in `dist/` which gets embedded into the Rust binary.

### Preview Production Build

```bash
npm run preview
```

## Project Structure

```
web-ui/
├── src/
│   ├── app.html           # HTML shell
│   ├── app.css            # Global styles
│   ├── routes/
│   │   ├── +page.svelte   # Main page component
│   │   └── +layout.svelte # Root layout (shared UI)
│   └── lib/
│       ├── components/    # Reusable components
│       ├── stores/        # Svelte stores
│       └── api/           # API client functions
├── static/                # Static assets (favicon, etc.)
├── package.json           # Dependencies and scripts
├── svelte.config.js       # SvelteKit config
├── vite.config.ts         # Vite config
└── tsconfig.json          # TypeScript config
```

## Development Patterns

### Component Structure

```svelte
<!-- MyComponent.svelte -->
<script lang="ts">
  // Props
  export let title: string;
  
  // Local state
  let count = 0;
  
  // Reactive
  $: doubled = count * 2;
</script>

<div class="container">
  <h1>{title}</h1>
  <p>Count: {count}, Doubled: {doubled}</p>
  <button on:click={() => count++}>Increment</button>
</div>

<style>
  .container {
    padding: 1rem;
  }
</style>
```

### Store Usage

```typescript
// lib/stores/graph.ts
import { writable } from 'svelte/store';

export const selectedNode = writable<string | null>(null);
export const graphData = writable<GraphData | null>(null);
```

```svelte
<!-- Using stores -->
<script lang="ts">
  import { selectedNode } from '$lib/stores/graph';
  
  function selectNode(id: string) {
    $selectedNode = id;
  }
</script>
```

### API Client

```typescript
// lib/api/client.ts
const BASE_URL = '/api';

export async function searchSpecs(query: string): Promise<SearchResult[]> {
  const res = await fetch(`${BASE_URL}/search?q=${encodeURIComponent(query)}`);
  if (!res.ok) throw new Error('Search failed');
  return res.json();
}

export async function getSpec(id: string): Promise<SpecDoc> {
  const res = await fetch(`${BASE_URL}/spec/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error('Spec not found');
  return res.json();
}
```

### Graph Visualization

```svelte
<script lang="ts">
  import { SvelteFlow, Background, Controls } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  
  let nodes = [];
  let edges = [];
</script>

<div style="height: 100vh;">
  <SvelteFlow {nodes} {edges}>
    <Background />
    <Controls />
  </SvelteFlow>
</div>
```

## Common Tasks

| Task | Command |
|------|---------|
| Add dependency | `npm install <package>` |
| Add dev dependency | `npm install -D <package>` |
| Update dependencies | `npm update` |
| Check for outdated | `npm outdated` |
| Clean install | `rm -rf node_modules && npm install` |

## Styling Guidelines

- Use scoped `<style>` blocks in components
- Global styles in `app.css` only for reset/typography
- No external CSS frameworks (keep bundle small)
- Use CSS custom properties for theming

## TypeScript Guidelines

- Strict mode enabled
- Define types for API responses
- Use `unknown` over `any`
- Export types from `lib/types.ts`

## Build Integration

The production build is embedded into the Rust binary:

1. `npm run build` → outputs to `dist/`
2. Rust build uses `rust-embed` to include `dist/`
3. Axum serves embedded files at runtime

To test the full integration:
```bash
# Build web UI
cd web-ui && npm run build && cd ..

# Build and run Rust binary
cargo build --release
./target/release/lattice serve
```

Then visit `http://127.0.0.1:3000` (or configured port).

---

*Generated: 2026-02-27 | Scan Level: Quick*
