//! Acceptance tests for Story 5.2: Hybrid Query Execution and Result Composition

use std::sync::Arc;

use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{
    CausalEdge, CausalGraph, EdgeOrigin, EdgeType, SearchEngine, SpecDoc, SpecId, SpecNode,
    TrustLevel,
};
use spec_db_router::{ComposedHit, QueryRouter};
use spec_db_search::SearchIndex;

fn _spec_doc(id: &str, title: &str, body: &str) -> SpecDoc {
    SpecDoc {
        id: SpecId::try_new(id).unwrap(),
        title: title.to_owned(),
        version: 1,
        tags: vec!["acceptance".to_owned()],
        depends_on: Vec::new(),
        owner: Some("qa".to_owned()),
        created: "2026-02-23".to_owned(),
        body: body.to_owned(),
    }
}

fn _spec_node(id: &str, title: &str) -> SpecNode {
    SpecNode { id: SpecId::try_new(id).unwrap(), title: title.to_owned(), version: 1 }
}

fn _human_edge(source: &str, target: &str) -> CausalEdge {
    CausalEdge {
        source: SpecId::try_new(source).unwrap(),
        target: SpecId::try_new(target).unwrap(),
        edge_type: EdgeType::DependsOn,
        trust: TrustLevel::human(),
        origin: EdgeOrigin::Human,
        created_at: None,
    }
}

fn _assert_hit_shape(hit: &ComposedHit) {
    assert!(!hit.id.is_empty());
    assert!(hit.score >= 0.0);
}

fn _hybrid_router_fixture() -> (tempfile::TempDir, QueryRouter<SearchIndex, CausalEngine>) {
    let dir = tempfile::tempdir().unwrap();
    let tantivy_path = dir.path().join("tantivy");
    let fjall_path = dir.path().join("fjall");

    let mut search = SearchIndex::open_or_create(&tantivy_path).unwrap();
    let store = Arc::new(FjallStore::open(&fjall_path).unwrap());
    let mut graph = CausalEngine::from_store(store).unwrap();

    let docs = vec![
        _spec_doc(
            "spec::api::rate-limits",
            "Rate Limiting",
            "rate limiting policy for API requests",
        ),
        _spec_doc("spec::api::gateway", "Gateway", "gateway request orchestration and retries"),
        _spec_doc("spec::auth::login", "Login", "credential validation and auth checks"),
    ];

    for doc in &docs {
        search.index_spec(doc).unwrap();
    }

    graph.upsert_node(_spec_node("spec::api::rate-limits", "Rate Limiting")).unwrap();
    graph.upsert_node(_spec_node("spec::api::gateway", "Gateway")).unwrap();
    graph.upsert_node(_spec_node("spec::auth::login", "Login")).unwrap();
    graph.add_edge(_human_edge("spec::api::gateway", "spec::api::rate-limits")).unwrap();
    graph.add_edge(_human_edge("spec::api::rate-limits", "spec::auth::login")).unwrap();

    (dir, QueryRouter::new(search, graph))
}

fn _fallback_router_fixture() -> (tempfile::TempDir, QueryRouter<SearchIndex, CausalEngine>) {
    let dir = tempfile::tempdir().unwrap();
    let tantivy_path = dir.path().join("tantivy");
    let fjall_path = dir.path().join("fjall");

    let search = SearchIndex::open_or_create(&tantivy_path).unwrap();
    let store = Arc::new(FjallStore::open(&fjall_path).unwrap());
    let mut graph = CausalEngine::from_store(store).unwrap();

    graph.upsert_node(_spec_node("spec::auth::login", "Login")).unwrap();
    graph.upsert_node(_spec_node("spec::api::gateway", "Gateway")).unwrap();
    graph.add_edge(_human_edge("spec::api::gateway", "spec::auth::login")).unwrap();

    (dir, QueryRouter::new(search, graph))
}

fn _empty_router_fixture() -> (tempfile::TempDir, QueryRouter<SearchIndex, CausalEngine>) {
    let dir = tempfile::tempdir().unwrap();
    let tantivy_path = dir.path().join("tantivy");
    let fjall_path = dir.path().join("fjall");

    let search = SearchIndex::open_or_create(&tantivy_path).unwrap();
    let store = Arc::new(FjallStore::open(&fjall_path).unwrap());
    let graph = CausalEngine::from_store(store).unwrap();

    (dir, QueryRouter::new(search, graph))
}

/// AC1: Hybrid query returns composed search results and causal context.
#[test]
fn ac1_hybrid_query_returns_composed_search_and_causal_context() {
    let (_dir, router) = _hybrid_router_fixture();

    let result = router.query("what depends on rate limiting").unwrap();

    assert_eq!(result.intent, "hybrid");
    assert!(!result.search_results.is_empty());
    assert!(!result.causal_context.is_empty());
    for hit in &result.search_results {
        _assert_hit_shape(hit);
        assert!(!hit.causal_edges.is_empty());
    }
}

/// AC2: Search queries with zero matches fall back to causal graph context.
#[test]
fn ac2_search_zero_results_falls_back_to_causal_context() {
    let (_dir, router) = _fallback_router_fixture();

    let result = router.query("lookup \"spec::auth::login\"").unwrap();

    assert_eq!(result.intent, "search");
    assert!(result.search_results.is_empty());
    assert!(!result.causal_context.is_empty());
    assert!(result.message.contains("No direct search matches"));
}

/// AC3: Causal queries return graph context without search results.
#[test]
fn ac3_causal_query_returns_graph_result_without_search_hits() {
    let (_dir, router) = _hybrid_router_fixture();

    let result = router.query("what depends on spec::auth::login").unwrap();

    assert_eq!(result.intent, "causal");
    assert!(result.search_results.is_empty());
    assert!(result.causal_context.iter().any(|id| id == "spec::api::rate-limits"));
}

/// AC4: Empty search and graph engines return an explicit empty response.
#[test]
fn ac4_empty_engines_return_clear_non_fabricated_empty_result() {
    let (_dir, router) = _empty_router_fixture();

    let result = router.query("totally-unmatched-query-token").unwrap();

    assert_eq!(result.intent, "empty");
    assert!(result.search_results.is_empty());
    assert!(result.causal_context.is_empty());
    assert!(result.message.contains("No search or causal results"));
}
