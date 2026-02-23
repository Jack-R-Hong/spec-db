# Story 1.1: Scaffold Workspace & Core Domain Types

Status: review

## Story

As a developer,
I want a properly scaffolded Cargo workspace with core domain types, trait interfaces, and error hierarchy,
so that all subsequent development has a consistent, version-locked foundation.

## Acceptance Criteria (BDD)

**Given** a clean checkout of the repository  
**When** I run `cargo build --workspace`  
**Then** the workspace compiles with `lattice` (binary), `spec-db-core` (lib), and `spec-db-causal` (lib) crates  
**And** `spec-db-core` exports `SpecId`, `SpecDoc`, `SpecNode`, `CausalEdge`, and `TrustLevel` types  
**And** `SpecId` validates the `spec::{segment}::{segment}` pattern and rejects invalid formats  
**And** `spec-db-core` exports `SearchEngine`, `CausalGraph`, and `SpecStore` traits  
**And** `spec-db-core` exports `SpecDbError` with variants: `SearchError`, `GraphError`, `SyncError`, `IngestError`, `ConsistencyError`, `ConfigError`  
**And** `workspace.dependencies` in root `Cargo.toml` locks all dependency versions per architecture spec  
**And** `rustfmt.toml` and `clippy.toml` are configured per architecture patterns  
**And** `cargo clippy --workspace -- -D warnings` passes with zero warnings  
**And** `cargo fmt --all -- --check` passes

## Tasks / Subtasks

- [x] Initialize the lean workspace skeleton and crate manifests (AC: 1)
- [x] Create root `Cargo.toml` as both workspace root and binary package:
  - [x] Define `[package] name = "lattice"`, edition `2024`, rust-version `1.85` (or higher enforced by CI), and root `src/main.rs`
  - [x] Define `[workspace] members = ["crates/spec-db-core", "crates/spec-db-causal"]`
  - [x] Define `[workspace.dependencies]` with explicit pins: `deep_causality = "=0.13.4"`, `fjall = "3.0"`, `thiserror = "2"`, `anyhow = "1"`, `serde = { version = "1", features = ["derive"] }`, `serde_yml = "0.0.12"`, `tracing = "0.1"`, `tokio = { version = "1.49", features = ["rt-multi-thread", "macros"] }`, `clap = { version = "4.5", features = ["derive"] }`, `git2 = "0.20.4"`, `pulldown-cmark = "0.13"`, `tantivy = "0.25.0"`, `rmcp = "=0.16.0"`, `bincode = { version = "=2.0.1", features = ["serde"] }`
  - [x] Add `[profile.release] lto = true`, `codegen-units = 1`, `strip = true`
- [x] Create crate manifests and wire dependencies to workspace pins (AC: 1, 8)
  - [x] `crates/spec-db-core/Cargo.toml`: package `spec-db-core`, deps on `serde`, `thiserror`
  - [x] `crates/spec-db-causal/Cargo.toml`: package `spec-db-causal`, deps on `spec-db-core`, `deep_causality`, `fjall`, `bincode`, `tracing`
  - [x] Root crate deps: `anyhow`, `clap`, `tokio`, `tracing` (minimum wiring for compile)
- [x] Implement core domain types in `spec-db-core` (AC: 2, 3)
  - [x] Create `crates/spec-db-core/src/types.rs` defining `SpecId`, `SpecDoc`, `SpecNode`, `CausalEdge`, `TrustLevel`
  - [x] Implement `impl SpecId { pub fn try_new(raw: impl Into<String>) -> Result<Self, SpecDbError> }`
  - [x] Add `impl AsRef<str> for SpecId`, `impl core::fmt::Display for SpecId`, `impl core::str::FromStr for SpecId`
  - [x] Validate with a compiled regex or equivalent parser enforcing `spec::{segment}::{segment}` where segment charset is `[a-z0-9-]+`
- [x] Implement trait interfaces in `spec-db-core` (AC: 4)
  - [x] Create `crates/spec-db-core/src/traits.rs` with public traits: `SearchEngine`, `CausalGraph`, `SpecStore`
  - [x] Keep trait signatures implementation-agnostic and return `Result<_, SpecDbError>`
  - [x] Include methods required by downstream stories: `trace_impact`, `find_dependencies`, node/edge persistence, metadata get/set
- [x] Implement error hierarchy in `spec-db-core` (AC: 5)
  - [x] Create `crates/spec-db-core/src/error.rs` with `#[derive(thiserror::Error, Debug)] pub enum SpecDbError`
  - [x] Add exactly these variants: `SearchError`, `GraphError`, `SyncError`, `IngestError`, `ConsistencyError`, `ConfigError`
  - [x] Ensure error strings are human-readable and include contextual payload (`String` or source error wrappers)
- [x] Expose explicit public API in crate root (AC: 2, 4, 5)
  - [x] Create `crates/spec-db-core/src/lib.rs` with `pub mod types; pub mod traits; pub mod error;`
  - [x] Add explicit re-exports only: `pub use types::{...};`, `pub use traits::{...};`, `pub use error::SpecDbError;`
  - [x] Do not use wildcard re-exports
- [x] Configure formatting and lint guardrails (AC: 9)
  - [x] Add `rustfmt.toml` with `edition = "2024"`, `max_width = 100`, `use_small_heuristics = "Max"`
  - [x] Add `clippy.toml` with project lint constraints (at minimum set `allow-unwrap-in-tests = true`, keep lib code unwrap-free)
  - [x] Ensure CI-equivalent local commands are documented in `README` or `CONTRIBUTING`
- [x] Add baseline tests for core invariants (AC: 3, 10, 11)
  - [x] Unit tests in `crates/spec-db-core/src/types.rs` for valid/invalid `SpecId`
  - [x] Unit tests for error construction and trait object usability
  - [x] Confirm `cargo build --workspace`, `cargo fmt --all -- --check`, and `cargo clippy --workspace -- -D warnings` pass
- [x] Capture handoff constraints for downstream stories (cross-story dependency)
  - [x] Add short note in `crates/spec-db-core/src/lib.rs` docs: Story 1.2 consumes `SpecNode`/`CausalEdge`; Story 1.3/1.4 consume `CausalGraph` trait contracts

## Dev Notes

- Story 1.1 is the contract layer for the rest of Epic 1; Stories 1.2, 1.3, and 1.4 depend on type names, trait signatures, and error variants defined here.
- Apply architecture consistency patterns: `N1` (modern module files, no `mod.rs`), `N2` (hyphen crate names / underscore imports), `S2` (explicit public API in `lib.rs`), `S3` (trait-based boundaries), `S4` (all domain types in `spec-db-core`), `S5` (module depth max 2), `P1` (fail-fast errors, no silent failures).
- Error handling rule is mandatory: `thiserror` in library crates and no `unwrap()`/`expect()` in non-test code; use `?` with typed `SpecDbError` propagation.
- `SpecId` is a universal key across Tantivy, Fjall, and graph engine; this story must enforce validation once at construction to prevent downstream key drift.
- Version lock guidance: architecture locks DeepCausality `0.13.4` and Fjall `3.0.x`; web research adds a bincode gotcha: avoid `bincode 3.0.0` (marked unmaintained + non-functional release), use `2.0.1` with `serde` feature for stable encode/decode API.
- Add minimal placeholder `src/main.rs` for compile-only root binary (`fn main() -> anyhow::Result<()> { Ok(()) }`) so workspace commands pass while later stories fill CLI behavior.

### Project Structure Notes

- Use these concrete paths in this story:
- `Cargo.toml`, `rustfmt.toml`, `clippy.toml`, `src/main.rs`
- `crates/spec-db-core/Cargo.toml`, `crates/spec-db-core/src/lib.rs`, `crates/spec-db-core/src/types.rs`, `crates/spec-db-core/src/traits.rs`, `crates/spec-db-core/src/error.rs`
- `crates/spec-db-causal/Cargo.toml`, `crates/spec-db-causal/src/lib.rs` (placeholder exports for Story 1.2)
- Naming convention: Cargo package names use hyphens (`spec-db-core`), Rust paths use underscores (`spec_db_core`).
- Architectural variance resolution: the architecture tree also references `crates/core` and `crates/causal`; for this Epic use `crates/spec-db-core` and `crates/spec-db-causal` to match acceptance criteria and crate names while preserving dependency boundaries.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic 1]
- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.1]
- [Source: _bmad-output/planning-artifacts/architecture.md#Starter Template Evaluation]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Error Handling]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/project-context.md#Key Patterns for AI Agents]
- [Source: https://docs.rs/fjall/latest/fjall/]
- [Source: https://docs.rs/deep_causality/latest/deep_causality/]
- [Source: https://docs.rs/bincode/2.0.1/bincode/serde/index.html]
- [Source: https://docs.rs/bincode/latest/bincode/serde/index.html]

## Dev Agent Record

### Agent Model Used

anthropic/claude-opus-4-6

### Completion Notes List

- Story file created with full AC coverage and implementation-ready guardrails.
- Cross-story dependencies called out for Stories 1.2-1.4.
- Version lock section includes current library gotchas from web research.
- Workspace scaffolded with 3 crates (lattice binary, spec-db-core lib, spec-db-causal lib).
- All 5 domain types implemented: SpecId (with validation), SpecDoc, SpecNode, CausalEdge, TrustLevel. Added EdgeOrigin enum for edge provenance.
- All 3 trait interfaces implemented: SearchEngine, CausalGraph, SpecStore with typed Result returns.
- SpecDbError enum with 6 variants using thiserror derive.
- SpecId validation uses manual parser (no regex dep) enforcing spec::{segment}::{segment} format.
- 15 unit tests covering valid/invalid SpecId, TrustLevel clamping, error display.
- cargo build, cargo test, cargo clippy -D warnings, cargo fmt --check all pass clean.

### Change Log

- Initial draft.
- 2026-02-23: Full implementation of Story 1.1 — workspace scaffold, domain types, traits, errors, tests.

### File List

- `Cargo.toml`
- `src/main.rs`
- `rustfmt.toml`
- `clippy.toml`
- `crates/spec-db-core/Cargo.toml`
- `crates/spec-db-core/src/lib.rs`
- `crates/spec-db-core/src/types.rs`
- `crates/spec-db-core/src/traits.rs`
- `crates/spec-db-core/src/error.rs`
- `crates/spec-db-causal/Cargo.toml`
- `crates/spec-db-causal/src/lib.rs`
- `_bmad-output/implementation-artifacts/1-1-scaffold-workspace-core-domain-types.md`
