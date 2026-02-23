//! Acceptance tests for Story 1.2: Fjall Persistent Storage for Causal Graph

use spec_db_causal::FjallStore;
use spec_db_core::{CausalEdge, EdgeOrigin, SpecId, SpecNode, TrustLevel};

fn spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn node(id: &str, title: &str, version: u32) -> SpecNode {
    SpecNode { id: spec_id(id), title: title.to_owned(), version }
}

fn edge(source: &SpecNode, target: &SpecNode) -> CausalEdge {
    CausalEdge {
        source: source.id.clone(),
        target: target.id.clone(),
        trust: TrustLevel::human(),
        origin: EdgeOrigin::Human,
    }
}

/// AC1: Storing a SpecNode persists it and retrieval returns an identical node.
#[test]
fn ac1_node_roundtrip_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();
    let original = node("spec::auth::token", "Token Issuance", 7);

    store.put_node(&original).unwrap();

    let loaded = store.get_node(&original.id).unwrap().unwrap();
    assert_eq!(loaded.id.as_ref(), original.id.as_ref());
    assert_eq!(loaded.title, original.title);
    assert_eq!(loaded.version, original.version);
}

/// AC2: Edges are retrievable via their composite source-target key.
#[test]
fn ac2_edge_roundtrip_by_composite_key() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();
    let source = node("spec::svc::checkout", "Checkout", 1);
    let target = node("spec::svc::inventory", "Inventory", 3);
    let persisted = edge(&source, &target);

    store.put_node(&source).unwrap();
    store.put_node(&target).unwrap();
    store.put_edge(&persisted).unwrap();

    let loaded = store.get_edge(&source.id, &target.id).unwrap().unwrap();
    assert_eq!(loaded.source.as_ref(), source.id.as_ref());
    assert_eq!(loaded.target.as_ref(), target.id.as_ref());
    assert_eq!(loaded.origin, EdgeOrigin::Human);
    assert!((loaded.trust.value() - 1.0).abs() < f64::EPSILON);
}

/// AC3: Metadata values persist across store reopen.
#[test]
fn ac3_metadata_persists_after_reopen() {
    let dir = tempfile::tempdir().unwrap();

    {
        let store = FjallStore::open(dir.path()).unwrap();
        store.set_last_sync_sha("deadbeef").unwrap();
        store.set_doc_count(11).unwrap();
    }

    let reopened = FjallStore::open(dir.path()).unwrap();
    assert_eq!(reopened.last_sync_sha().unwrap().as_deref(), Some("deadbeef"));
    assert_eq!(reopened.doc_count().unwrap(), Some(11));
}

/// AC4: Atomic node+edge writes commit complete graph changes without partial results.
#[test]
fn ac4_atomic_node_with_edges_write() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();
    let root = node("spec::graph::root", "Root", 1);
    let dep_a = node("spec::graph::dep-a", "Dependency A", 1);
    let dep_b = node("spec::graph::dep-b", "Dependency B", 1);
    let edge_a = edge(&root, &dep_a);
    let edge_b = edge(&root, &dep_b);

    store.put_node(&dep_a).unwrap();
    store.put_node(&dep_b).unwrap();
    store.put_node_with_edges(&root, &[edge_a, edge_b]).unwrap();

    assert!(store.get_node(&root.id).unwrap().is_some());
    assert!(store.get_edge(&root.id, &dep_a.id).unwrap().is_some());
    assert!(store.get_edge(&root.id, &dep_b.id).unwrap().is_some());
}
