# Backend Development Guide

## Prerequisites

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Rust | 1.85+ | Language runtime (Edition 2024) |
| Git | Any | Source control, sync functionality |
| Buck2 | Optional | Alternative build system |

## Getting Started

### Clone and Build

```bash
# Clone repository
git clone <repo-url> && cd spec-db

# Build (debug)
cargo build

# Build (release with LTO)
cargo build --release

# Install locally
cargo install --path .
```

### Initialize a Test Project

```bash
# Create a test project
mkdir /tmp/my-specs && cd /tmp/my-specs && git init

# Initialize lattice project structure
lattice init

# Commit scaffolded specs (required for sync)
git add -A && git commit -m "init"

# Build search index and causal graph
lattice sync

# Start MCP server
lattice serve

# Check status
lattice status
```

## Development Workflow

### Running Tests

```bash
# All tests (unit + integration)
cargo test --workspace

# Integration/acceptance tests only
cargo test --test '*'

# Single acceptance test
cargo test --test acceptance_story_1_1

# Single crate tests
cargo test -p spec-db-core
```

### Code Quality

```bash
# Format check
cargo fmt --all -- --check

# Lint check (warnings as errors)
cargo clippy --workspace -- -D warnings

# Format and lint (fix)
cargo fmt --all
cargo clippy --workspace --fix --allow-dirty
```

### Running the Server

```bash
# Development (debug build)
cargo run -- serve

# With specific config directory
cargo run -- --config /path/to/.lattice/config.yaml serve

# Release mode
cargo run --release -- serve
```

## Crate Development

### Adding a New Crate

1. Create crate directory:
```bash
mkdir -p crates/new-crate/src
```

2. Add `Cargo.toml`:
```toml
[package]
name = "spec-db-new-crate"
version = "0.1.0"
edition = "2024"

[dependencies]
spec-db-core = { workspace = true }
```

3. Register in workspace `Cargo.toml`:
```toml
[workspace]
members = [
    # ...
    "crates/new-crate",
]

[workspace.dependencies]
spec-db-new-crate = { path = "crates/new-crate" }
```

### Crate Guidelines

- **Import domain types from `spec-db-core`** — never redefine `SpecId`, `SpecDoc`, etc.
- **Use trait interfaces** for cross-crate dependencies
- **Follow modern module style**: `foo.rs` + `foo/bar.rs` (no `mod.rs`)
- **Maximum module depth**: 2 levels (`crate::module::submodule`)

## Error Handling Patterns

### Library Crates (thiserror)

```rust
use crate::error::SpecDbError;

pub fn do_something() -> Result<(), SpecDbError> {
    something_fallible()
        .map_err(|e| SpecDbError::SearchError(e.to_string()))?;
    Ok(())
}
```

### Binary Entry Point (anyhow)

```rust
use anyhow::Result;

fn main() -> Result<()> {
    let config = load_config()?;
    run_server(config)?;
    Ok(())
}
```

### Tests (unwrap allowed)

```rust
#[test]
fn test_something() {
    let result = do_something().unwrap();  // OK in tests
    assert_eq!(result, expected);
}
```

## Async Patterns

### Handler Layer (Async)

```rust
#[tool]
async fn search_specs(query: String) -> Result<Vec<SearchResult>> {
    let results = tokio::task::spawn_blocking(move || {
        search_engine.search(&query)  // Sync code
    }).await??;
    Ok(results)
}
```

### Subsystem Layer (Sync)

```rust
impl SearchEngine for TantivySearch {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Direct Tantivy calls - no async
        let searcher = self.index.reader()?.searcher();
        // ...
    }
}
```

## Testing Patterns

### Unit Tests (Inline)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_spec_id() {
        let id = SpecId::try_new("spec::auth::login").unwrap();
        assert_eq!(id.as_ref(), "spec::auth::login");
    }
}
```

### Integration Tests

```rust
// tests/integration.rs
use spec_db_core::*;
use spec_db_search::*;

#[test]
fn search_and_causal_integration() {
    let dir = tempfile::tempdir().unwrap();
    // Test cross-crate functionality
}
```

### Acceptance Tests

```rust
// tests/acceptance_story_1_1.rs
// Story-based tests following epic/story structure
```

## Build with Buck2 (Alternative)

```bash
# Build
buck2 build //:lattice

# Run
buck2 run //:lattice -- serve

# After Cargo.toml changes
reindeer buckify
```

## Debugging

### Tracing

Enable debug logging:
```bash
RUST_LOG=debug cargo run -- serve
RUST_LOG=spec_db_search=trace cargo run -- serve
```

### Index Inspection

```bash
# Check current status
lattice status

# Force full rebuild
lattice rebuild
```

## Common Tasks

| Task | Command |
|------|---------|
| Add dependency | Edit `Cargo.toml`, run `cargo build` |
| Update lockfile | `cargo update` |
| Check for outdated | `cargo outdated` |
| Security audit | `cargo audit` |
| Generate docs | `cargo doc --workspace --open` |

---

*Generated: 2026-02-27 | Scan Level: Quick*
