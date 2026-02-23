use spec_db_causal::FjallStore;
use spec_db_core::{CausalEdge, EdgeOrigin, EdgeType, SpecId, SpecNode, SpecStore, TrustLevel};

fn make_node(domain: &str, name: &str) -> SpecNode {
    let id_str = format!("spec::{domain}::{name}");
    SpecNode { id: SpecId::try_new(id_str).unwrap(), title: format!("{name} title"), version: 1 }
}

fn make_edge(from: &SpecNode, to: &SpecNode) -> CausalEdge {
    CausalEdge {
        source: from.id.clone(),
        target: to.id.clone(),
        edge_type: EdgeType::DependsOn,
        trust: TrustLevel::human(),
        origin: EdgeOrigin::Human,
        created_at: None,
    }
}

#[test]
fn node_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    let node = make_node("auth", "login");
    store.put_node(&node).unwrap();

    let loaded = store.get_node(&node.id).unwrap().unwrap();
    assert_eq!(loaded.id.as_ref(), node.id.as_ref());
    assert_eq!(loaded.title, node.title);
    assert_eq!(loaded.version, node.version);
}

#[test]
fn edge_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    let a = make_node("auth", "login");
    let b = make_node("auth", "session");
    let edge = make_edge(&a, &b);

    store.put_node(&a).unwrap();
    store.put_node(&b).unwrap();
    store.put_edge(&edge).unwrap();

    let loaded = store.get_edge(&a.id, &b.id).unwrap().unwrap();
    assert_eq!(loaded.source.as_ref(), a.id.as_ref());
    assert_eq!(loaded.target.as_ref(), b.id.as_ref());
    assert_eq!(loaded.edge_type, EdgeType::DependsOn);
    assert_eq!(loaded.trust.value(), 1.0);
}

#[test]
fn metadata_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    assert!(store.last_sync_sha().unwrap().is_none());
    assert!(store.doc_count().unwrap().is_none());

    store.set_last_sync_sha("abc123def").unwrap();
    store.set_doc_count(42).unwrap();

    assert_eq!(store.last_sync_sha().unwrap().unwrap(), "abc123def");
    assert_eq!(store.doc_count().unwrap().unwrap(), 42);
}

#[test]
fn reopen_durability() {
    let dir = tempfile::tempdir().unwrap();

    let node = make_node("api", "endpoint");
    let edge_target = make_node("api", "middleware");

    {
        let store = FjallStore::open(dir.path()).unwrap();
        store.put_node(&node).unwrap();
        store.put_node(&edge_target).unwrap();
        store.put_edge(&make_edge(&node, &edge_target)).unwrap();
        store.set_last_sync_sha("sha256abc").unwrap();
        store.set_doc_count(2).unwrap();
    }

    {
        let store = FjallStore::open(dir.path()).unwrap();
        let loaded_node = store.get_node(&node.id).unwrap().unwrap();
        assert_eq!(loaded_node.title, node.title);

        let loaded_edge = store.get_edge(&node.id, &edge_target.id).unwrap().unwrap();
        assert_eq!(loaded_edge.source.as_ref(), node.id.as_ref());

        assert_eq!(store.last_sync_sha().unwrap().unwrap(), "sha256abc");
        assert_eq!(store.doc_count().unwrap().unwrap(), 2);
    }
}

#[test]
fn put_node_with_edges_atomic() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    let node = make_node("core", "types");
    let dep1 = make_node("core", "errors");
    let dep2 = make_node("core", "traits");
    store.put_node(&dep1).unwrap();
    store.put_node(&dep2).unwrap();

    let edges = vec![make_edge(&node, &dep1), make_edge(&node, &dep2)];

    store.put_node_with_edges(&node, &edges).unwrap();

    let loaded = store.get_node(&node.id).unwrap().unwrap();
    assert_eq!(loaded.title, node.title);

    let e1 = store.get_edge(&node.id, &dep1.id).unwrap().unwrap();
    assert_eq!(e1.target.as_ref(), dep1.id.as_ref());

    let e2 = store.get_edge(&node.id, &dep2.id).unwrap().unwrap();
    assert_eq!(e2.target.as_ref(), dep2.id.as_ref());
}

#[test]
fn iter_edges_returns_all() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    let a = make_node("svc", "a");
    let b = make_node("svc", "b");
    let c = make_node("svc", "c");
    store.put_node(&a).unwrap();
    store.put_node(&b).unwrap();
    store.put_node(&c).unwrap();

    store.put_edge(&make_edge(&a, &b)).unwrap();
    store.put_edge(&make_edge(&b, &c)).unwrap();

    let all_edges = store.iter_edges().unwrap();
    assert_eq!(all_edges.len(), 2);
}

#[test]
fn iter_nodes_returns_all() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    store.put_node(&make_node("x", "one")).unwrap();
    store.put_node(&make_node("x", "two")).unwrap();
    store.put_node(&make_node("x", "three")).unwrap();

    let all_nodes = store.iter_nodes().unwrap();
    assert_eq!(all_nodes.len(), 3);
}

#[test]
fn get_nonexistent_returns_none() {
    let dir = tempfile::tempdir().unwrap();
    let store = FjallStore::open(dir.path()).unwrap();

    let id = SpecId::try_new("spec::no::exist").unwrap();
    assert!(store.get_node(&id).unwrap().is_none());

    let id2 = SpecId::try_new("spec::no::exist2").unwrap();
    assert!(store.get_edge(&id, &id2).unwrap().is_none());
}

#[test]
fn spec_store_list_ids() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FjallStore::open(dir.path()).unwrap();

    let doc = spec_db_core::SpecDoc {
        id: SpecId::try_new("spec::test::doc").unwrap(),
        title: "Test Doc".into(),
        version: 1,
        tags: vec![],
        depends_on: vec![],
        owner: None,
        created: "2026-02-23".into(),
        body: String::new(),
    };
    store.put(doc).unwrap();

    let ids = store.list_ids().unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0].as_ref(), "spec::test::doc");
}
