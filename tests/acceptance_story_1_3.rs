//! Acceptance tests for Story 1.3: DeepCausality In-Memory Graph Engine

use std::sync::Arc;

use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, EdgeOrigin, SpecDoc, SpecId, SpecStore};

fn spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn spec_doc(id: &str, title: &str, depends_on: Vec<SpecId>) -> SpecDoc {
    SpecDoc {
        id: spec_id(id),
        title: title.to_owned(),
        version: 1,
        tags: vec!["acceptance".to_owned()],
        depends_on,
        owner: Some("qa".to_owned()),
        created: "2026-02-23".to_owned(),
        body: "body".to_owned(),
    }
}

/// AC1: Engine initialization loads all persisted nodes and edges into memory.
#[test]
fn ac1_engine_loads_nodes_and_edges_from_store() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FjallStore::open(dir.path()).unwrap();
    let target = spec_doc("spec::auth::token-issuance", "Token Issuance", vec![]);
    let source =
        spec_doc("spec::auth::session-management", "Session Management", vec![target.id.clone()]);

    store.put(target.clone()).unwrap();
    store.put(source.clone()).unwrap();

    let engine = CausalEngine::from_store(Arc::new(store)).unwrap();
    let source_node = engine.get_node(&source.id).unwrap();
    let target_node = engine.get_node(&target.id).unwrap();
    let outbound = engine.edges_from(&source.id).unwrap();

    assert!(source_node.is_some());
    assert!(target_node.is_some());
    assert_eq!(outbound.len(), 1);
    assert_eq!(outbound[0].source.as_ref(), source.id.as_ref());
    assert_eq!(outbound[0].target.as_ref(), target.id.as_ref());
}

/// AC1: Startup remains under one second for 100+ persisted specs.
#[test]
#[ignore = "expensive: verifies startup timing for 100+ specs"]
fn ac1_startup_under_one_second_for_100_plus_specs() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FjallStore::open(dir.path()).unwrap();

    for i in 0..120 {
        let id = format!("spec::perf::node-{i}");
        let deps =
            if i == 0 { Vec::new() } else { vec![spec_id(&format!("spec::perf::node-{}", i - 1))] };
        store.put(spec_doc(&id, &format!("Node {i}"), deps)).unwrap();
    }

    let start = std::time::Instant::now();
    let _engine = CausalEngine::from_store(Arc::new(store)).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs_f64() < 1.0, "startup took {elapsed:?}");
}

/// AC2: A depends_on relationship is represented as an edge with human trust.
#[test]
fn ac2_depends_on_edge_has_human_trust() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FjallStore::open(dir.path()).unwrap();
    let dependency = spec_doc("spec::auth::token-issuance", "Token Issuance", vec![]);
    let dependent = spec_doc("spec::auth::api-gateway", "API Gateway", vec![dependency.id.clone()]);

    store.put(dependency.clone()).unwrap();
    store.put(dependent.clone()).unwrap();

    let engine = CausalEngine::from_store(Arc::new(store)).unwrap();
    let outbound = engine.edges_from(&dependent.id).unwrap();

    assert_eq!(outbound.len(), 1);
    assert_eq!(outbound[0].target.as_ref(), dependency.id.as_ref());
    assert_eq!(outbound[0].origin, EdgeOrigin::Human);
    assert!((outbound[0].trust.value() - 1.0).abs() < f64::EPSILON);
}

/// AC3: Node view returns the node plus complete inbound and outbound edge sets.
#[test]
fn ac3_node_view_includes_inbound_and_outbound_edges() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FjallStore::open(dir.path()).unwrap();
    let dependency = spec_doc("spec::svc::database", "Database", vec![]);
    let center = spec_doc("spec::svc::backend", "Backend", vec![dependency.id.clone()]);
    let dependent = spec_doc("spec::svc::frontend", "Frontend", vec![center.id.clone()]);

    store.put(dependency.clone()).unwrap();
    store.put(center.clone()).unwrap();
    store.put(dependent.clone()).unwrap();

    let engine = CausalEngine::from_store(Arc::new(store)).unwrap();
    let view = engine.node_view(&center.id).unwrap();

    assert_eq!(view.node.id.as_ref(), center.id.as_ref());
    assert_eq!(view.inbound_edges.len(), 1);
    assert_eq!(view.outbound_edges.len(), 1);
    assert_eq!(view.inbound_edges[0].source.as_ref(), dependent.id.as_ref());
    assert_eq!(view.outbound_edges[0].target.as_ref(), dependency.id.as_ref());
}
