//! Acceptance tests for Story 1.4: Causal Graph Traversal

use std::sync::Arc;
use std::time::{Duration, Instant};

use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{
    CausalEdge, CausalGraph, EdgeOrigin, SpecDbError, SpecId, SpecNode, TrustLevel,
};

fn spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn node(id: &str) -> SpecNode {
    SpecNode { id: spec_id(id), title: id.to_owned(), version: 1 }
}

fn human_edge(source: &SpecNode, target: &SpecNode) -> CausalEdge {
    CausalEdge {
        source: source.id.clone(),
        target: target.id.clone(),
        trust: TrustLevel::human(),
        origin: EdgeOrigin::Human,
    }
}

fn seeded_engine() -> (tempfile::TempDir, CausalEngine) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(FjallStore::open(dir.path()).unwrap());
    let mut engine = CausalEngine::from_store(store).unwrap();

    let keepalive = node("spec::keep::alive");
    engine.upsert_node(keepalive).unwrap();

    (dir, engine)
}

/// AC1: trace_impact(B) returns all transitive downstream dependents of B.
#[test]
fn ac1_trace_impact_returns_transitive_downstream_dependents() {
    let (_dir, mut engine) = seeded_engine();
    let a = node("spec::svc::a");
    let b = node("spec::svc::b");
    let c = node("spec::svc::c");

    engine.upsert_node(a.clone()).unwrap();
    engine.upsert_node(b.clone()).unwrap();
    engine.upsert_node(c.clone()).unwrap();
    engine.add_edge(human_edge(&a, &b)).unwrap();
    engine.add_edge(human_edge(&c, &a)).unwrap();

    let impacted = engine.trace_impact(&b.id, None).unwrap();
    assert_eq!(impacted, vec![a.id.clone(), c.id.clone()]);
}

/// AC2: find_dependencies(A) returns all transitive upstream dependencies of A.
#[test]
fn ac2_find_dependencies_returns_transitive_upstream_dependencies() {
    let (_dir, mut engine) = seeded_engine();
    let a = node("spec::domain::a");
    let b = node("spec::domain::b");
    let d = node("spec::domain::d");

    engine.upsert_node(a.clone()).unwrap();
    engine.upsert_node(b.clone()).unwrap();
    engine.upsert_node(d.clone()).unwrap();
    engine.add_edge(human_edge(&a, &b)).unwrap();
    engine.add_edge(human_edge(&b, &d)).unwrap();

    let dependencies = engine.find_dependencies(&a.id, None).unwrap();
    assert_eq!(dependencies, vec![b.id.clone(), d.id.clone()]);
}

/// AC3: Depth-limited traversal returns only nodes within hop limit; unlimited returns full chain.
#[test]
fn ac3_depth_limit_and_unbounded_traversal_behave_correctly() {
    let (_dir, mut engine) = seeded_engine();
    let a = node("spec::chain::a");
    let b = node("spec::chain::b");
    let c = node("spec::chain::c");
    let d = node("spec::chain::d");
    let e = node("spec::chain::e");

    for n in [&a, &b, &c, &d, &e] {
        engine.upsert_node(n.clone()).unwrap();
    }

    engine.add_edge(human_edge(&a, &b)).unwrap();
    engine.add_edge(human_edge(&b, &c)).unwrap();
    engine.add_edge(human_edge(&c, &d)).unwrap();
    engine.add_edge(human_edge(&d, &e)).unwrap();

    let depth_limited = engine.trace_impact(&e.id, Some(2)).unwrap();
    let full = engine.trace_impact(&e.id, None).unwrap();

    assert_eq!(depth_limited, vec![d.id.clone(), c.id.clone()]);
    assert_eq!(full, vec![d.id.clone(), c.id.clone(), b.id.clone(), a.id.clone()]);
}

/// AC4: Traversal on a 100+ node graph completes within 50ms.
#[test]
#[ignore = "expensive: verifies sub-50ms traversal on 100+ node graph"]
fn ac4_traversal_under_fifty_milliseconds_for_100_plus_specs() {
    let (_dir, mut engine) = seeded_engine();

    for i in 0..150 {
        engine.upsert_node(node(&format!("spec::perf::node-{i}"))).unwrap();
    }

    for i in 0..149 {
        let from = node(&format!("spec::perf::node-{i}"));
        let to = node(&format!("spec::perf::node-{}", i + 1));
        engine.add_edge(human_edge(&from, &to)).unwrap();
    }

    let root = spec_id("spec::perf::node-0");
    let leaf = spec_id("spec::perf::node-149");

    let impact_start = Instant::now();
    let impacted = engine.trace_impact(&leaf, None).unwrap();
    let impact_elapsed = impact_start.elapsed();

    let dependency_start = Instant::now();
    let dependencies = engine.find_dependencies(&root, None).unwrap();
    let dependency_elapsed = dependency_start.elapsed();

    assert_eq!(impacted.len(), 149);
    assert_eq!(dependencies.len(), 149);
    assert!(impact_elapsed < Duration::from_millis(50), "trace_impact took {impact_elapsed:?}");
    assert!(
        dependency_elapsed < Duration::from_millis(50),
        "find_dependencies took {dependency_elapsed:?}"
    );
}

/// AC5: Missing start IDs return GraphError for trace_impact and find_dependencies.
#[test]
fn ac5_missing_start_id_returns_clear_graph_error() {
    let (_dir, engine) = seeded_engine();
    let missing = spec_id("spec::missing::node");

    let trace_err = engine.trace_impact(&missing, None).unwrap_err();
    let dep_err = engine.find_dependencies(&missing, None).unwrap_err();

    match trace_err {
        SpecDbError::GraphError(message) => {
            assert_eq!(message, "Spec not found: spec::missing::node");
        }
        other => panic!("expected GraphError, got {other:?}"),
    }

    match dep_err {
        SpecDbError::GraphError(message) => {
            assert_eq!(message, "Spec not found: spec::missing::node");
        }
        other => panic!("expected GraphError, got {other:?}"),
    }
}
