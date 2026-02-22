# Story 1.2: Fjall Persistent Storage for Causal Graph

Status: ready-for-dev

## Story

As a developer,
I want spec nodes and causal edges persisted in Fjall keyspaces with bincode serialization,
so that the causal graph survives process restarts and can be loaded into memory on startup.

## Acceptance Criteria (BDD)

**Given** the `spec-db-causal` crate with Fjall 3.0.x and bincode dependencies  
**When** I store a `SpecNode` with a valid `SpecId`  
**Then** the node is persisted in the `nodes` keyspace with key=SpecId string and value=bincode-serialized SpecNode  
**And** I can retrieve the identical SpecNode by key

**Given** a `CausalEdge` representing A depends_on B  
**When** I store the edge  
**Then** it is persisted in the `edges` keyspace with key=`{from_id}\x00{to_id}` and value=bincode-serialized CausalEdge  
**And** I can retrieve the edge by its composite key

**Given** system metadata (last_sync_sha, doc_count)  
**When** I store metadata values  
**Then** they are persisted in the `meta` keyspace with string keys  
**And** I can retrieve them accurately after reopening the store

**Given** a node with associated edges being added  
**When** the operation executes  
**Then** node and edge writes are atomic via Fjall cross-keyspace batch  
**And** partial writes never occur on failure

## Tasks / Subtasks

- [ ] Finalize Story 1.1 dependency contracts before coding (AC: all)
  - [ ] Confirm `SpecId`, `SpecNode`, `CausalEdge`, `SpecDbError`, and `SpecStore` trait exist in `spec-db-core`
  - [ ] Confirm bincode compatibility derives on persisted types: `Serialize`, `Deserialize` (or bincode-native traits if intentionally selected)
- [ ] Create persistent store module in `spec-db-causal` (AC: 1, 2, 3, 4)
  - [ ] Add `crates/spec-db-causal/src/store.rs` with `pub struct FjallStore`
  - [ ] Define fields: `db: fjall::Database`, `nodes: fjall::Keyspace`, `edges: fjall::Keyspace`, `meta: fjall::Keyspace`
  - [ ] Add constructor signature: `pub fn open(path: &std::path::Path) -> Result<Self, SpecDbError>`
  - [ ] Open DB with `fjall::Database::builder(path).open()?`
  - [ ] Open keyspaces with explicit names: `nodes`, `edges`, `meta`
- [ ] Implement key codec helpers with exact key formats (AC: 1, 2, 3)
  - [ ] `fn node_key(id: &SpecId) -> String` -> raw spec id string
  - [ ] `fn edge_key(from: &SpecId, to: &SpecId) -> Vec<u8>` -> bytes for `{from_id}\x00{to_id}`
  - [ ] `fn edge_key_parts(key: &[u8]) -> Result<(SpecId, SpecId), SpecDbError>` -> split on single null byte
  - [ ] `fn meta_key_last_sync_sha() -> &'static str` and `fn meta_key_doc_count() -> &'static str`
- [ ] Implement bincode encode/decode helpers in store module (AC: 1, 2, 3)
  - [ ] `fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SpecDbError>` via `bincode::serde::encode_to_vec`
  - [ ] `fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SpecDbError>` via `bincode::serde::decode_from_slice`
  - [ ] Use `bincode::config::standard()` consistently for both encode and decode
  - [ ] Map codec failures to `SpecDbError::GraphError` with clear context text
- [ ] Implement node persistence API (AC: 1)
  - [ ] `pub fn put_node(&self, node: &SpecNode) -> Result<(), SpecDbError>` inserts in `nodes`
  - [ ] `pub fn get_node(&self, id: &SpecId) -> Result<Option<SpecNode>, SpecDbError>` reads from `nodes`
  - [ ] Round-trip test uses semantic equality (`assert_eq!(stored, loaded)`)
- [ ] Implement edge persistence API (AC: 2)
  - [ ] `pub fn put_edge(&self, edge: &CausalEdge) -> Result<(), SpecDbError>` writes `edges`
  - [ ] `pub fn get_edge(&self, from: &SpecId, to: &SpecId) -> Result<Option<CausalEdge>, SpecDbError>` reads by composite key
  - [ ] Add edge scan helper for Story 1.3 preload: `pub fn iter_edges(&self) -> Result<Vec<CausalEdge>, SpecDbError>`
- [ ] Implement metadata persistence API (AC: 3)
  - [ ] `pub fn set_last_sync_sha(&self, sha: &str) -> Result<(), SpecDbError>` and `pub fn last_sync_sha(&self) -> Result<Option<String>, SpecDbError>`
  - [ ] `pub fn set_doc_count(&self, count: usize) -> Result<(), SpecDbError>` and `pub fn doc_count(&self) -> Result<Option<usize>, SpecDbError>`
  - [ ] Persist metadata as UTF-8 bytes in `meta` keyspace, parse with explicit error mapping
- [ ] Implement atomic node+edge write path (AC: 4)
  - [ ] `pub fn put_node_with_edges(&self, node: &SpecNode, edges: &[CausalEdge]) -> Result<(), SpecDbError>`
  - [ ] Use `let mut batch = self.db.batch();` then `batch.insert(&self.nodes, ...)` and `batch.insert(&self.edges, ...)`
  - [ ] Commit exactly once with `batch.commit()?`; do not perform side writes outside batch
  - [ ] Add failure-path test where one edge encode fails and assert no node is committed
- [ ] Implement trait contract from Story 1.1 (AC: all)
  - [ ] Implement `SpecStore` for `FjallStore` in `store.rs` or `lib.rs` with thin wrappers over above methods
  - [ ] Keep API synchronous (architecture `P3` async boundary at MCP layer only)
- [ ] Add tests for reopen durability and atomicity (AC: 1, 2, 3, 4)
  - [ ] Integration tests in `crates/spec-db-causal/tests/integration.rs` using temp dir
  - [ ] Test sequence: open -> write nodes/edges/meta -> drop -> reopen -> verify byte-for-byte logical roundtrip
  - [ ] Test `put_node_with_edges` atomic guarantees under simulated error

## Dev Notes

- Story dependency: this story consumes Story 1.1 types and trait contracts; do not redefine any domain type in `spec-db-causal`.
- Keyspace design must be fixed and explicit for cross-story compatibility:
- `nodes` keyspace: key = `SpecId` UTF-8 string; value = bincode-serialized `SpecNode`
- `edges` keyspace: key = `{from_id}\x00{to_id}` bytes; value = bincode-serialized `CausalEdge`
- `meta` keyspace: key constants = `"last_sync_sha"`, `"doc_count"`; values as UTF-8 bytes (sha string / decimal count)
- Follow architecture patterns: `N3` key format (null-byte edge separator), `S3` trait boundaries, `S4` shared types in core only, `P2` atomicity via batch, and anti-pattern ban on `unwrap` in library code.
- Fjall 3.0.x API specifics (researched): use `Database::builder(path).open()`, `db.keyspace("name", KeyspaceCreateOptions::default)`, and cross-keyspace atomic writes through `db.batch()` + `commit()`.
- bincode specifics (researched): use serde integration API `encode_to_vec` / `decode_from_slice` from `bincode::serde`; avoid serde attributes documented as unsafe for bincode (`flatten`, `skip*`, `untagged`, `tag`) on persisted structs.
- Gotcha from latest registry: bincode `3.0.0` is an unmaintained marker release; pin `2.0.1` for working serde API until architecture is explicitly updated.
- Prepare Story 1.3 preload path now: provide efficient iterators or snapshot read methods for all nodes and edges.

### Project Structure Notes

- Primary files in scope:
- `crates/spec-db-causal/src/lib.rs`
- `crates/spec-db-causal/src/store.rs`
- `crates/spec-db-causal/tests/integration.rs`
- Required imports originate from `spec-db-core`: `use spec_db_core::{SpecId, SpecNode, CausalEdge, SpecStore, SpecDbError};`
- Keep module depth <=2 and modern module style (`store.rs`, no `store/mod.rs`).
- Storage path ownership remains inside causal crate (`data/fjall/` at runtime), matching crate data boundary rules.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.2]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure & Boundaries]
- [Source: docs/project-context.md#Critical Architectural Decisions]
- [Source: docs/project-context.md#Key Patterns for AI Agents]
- [Source: https://docs.rs/fjall/latest/fjall/]
- [Source: https://docs.rs/fjall/latest/fjall/struct.Database.html]
- [Source: https://docs.rs/fjall/latest/fjall/struct.OwnedWriteBatch.html]
- [Source: https://docs.rs/bincode/2.0.1/bincode/serde/index.html]
- [Source: https://docs.rs/bincode/latest/bincode/serde/index.html]

## Dev Agent Record

### Agent Model Used

openai/gpt-5.3-codex

### Completion Notes List

- Story file includes exact keyspace/key format contract for nodes/edges/meta.
- Atomic cross-keyspace batch behavior mapped to concrete Fjall API usage.
- Cross-story dependency with Story 1.1 and preload support for Story 1.3 included.

### Change Log

- Initial draft.

### File List

- `_bmad-output/implementation-artifacts/1-2-fjall-persistent-storage-causal-graph.md`
