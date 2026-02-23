use std::sync::Arc;
use std::time::{Duration, Instant};

use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{
    CausalEdge, CausalGraph, EdgeOrigin, SearchEngine, SpecDoc, SpecId, SpecNode, TrustLevel,
};
use spec_db_router::{QueryRouter, classify};
use spec_db_search::SearchIndex;

fn doc(id: &str, title: &str, body: &str) -> SpecDoc {
    SpecDoc {
        id: SpecId::try_new(id).unwrap(),
        title: title.to_owned(),
        version: 1,
        tags: vec!["routing".to_owned()],
        depends_on: Vec::new(),
        owner: Some("router-team".to_owned()),
        created: "2026-01-01T00:00:00Z".to_owned(),
        body: body.to_owned(),
    }
}

fn node(id: &SpecId, title: &str) -> SpecNode {
    SpecNode { id: id.clone(), title: title.to_owned(), version: 1 }
}

fn edge(source: &SpecId, target: &SpecId) -> CausalEdge {
    CausalEdge {
        source: source.clone(),
        target: target.clone(),
        trust: TrustLevel::human(),
        origin: EdgeOrigin::Human,
    }
}

fn setup_router() -> (tempfile::TempDir, QueryRouter<SearchIndex, CausalEngine>) {
    let dir = tempfile::tempdir().unwrap();
    let tantivy = dir.path().join("tantivy");
    let fjall = dir.path().join("fjall");

    let mut search = SearchIndex::open_or_create(&tantivy).unwrap();
    let store = Arc::new(FjallStore::open(&fjall).unwrap());
    let mut graph = CausalEngine::from_store(store).unwrap();

    let auth_login = SpecId::try_new("spec::auth::login").unwrap();
    let rate_limits = SpecId::try_new("spec::api::rate-limits").unwrap();
    let api_gateway = SpecId::try_new("spec::api::gateway").unwrap();
    let auth_mfa = SpecId::try_new("spec::auth::mfa").unwrap();

    let docs = [
        doc(auth_login.as_ref(), "Login Flow", "credential validation and jwt issuance"),
        doc(rate_limits.as_ref(), "Rate Limiting", "rate limiting policy for API requests"),
        doc(api_gateway.as_ref(), "Gateway Routing", "gateway request orchestration and retries"),
        doc(auth_mfa.as_ref(), "MFA Challenge", "second factor authentication challenge"),
    ];

    for spec in &docs {
        search.index_spec(spec).unwrap();
    }

    graph.upsert_node(node(&auth_login, "Login Flow")).unwrap();
    graph.upsert_node(node(&rate_limits, "Rate Limiting")).unwrap();
    graph.upsert_node(node(&api_gateway, "Gateway Routing")).unwrap();
    graph.upsert_node(node(&auth_mfa, "MFA Challenge")).unwrap();

    graph.add_edge(edge(&api_gateway, &rate_limits)).unwrap();
    graph.add_edge(edge(&rate_limits, &auth_login)).unwrap();
    graph.add_edge(edge(&auth_mfa, &auth_login)).unwrap();

    (dir, QueryRouter::new(search, graph))
}

#[test]
fn search_only_query() {
    let (_dir, router) = setup_router();

    let result = router.query("rate limiting policy").unwrap();

    assert_eq!(result.intent, "search");
    assert!(!result.search_results.is_empty());
    assert!(result.causal_context.is_empty());
}

#[test]
fn causal_only_query() {
    let (_dir, router) = setup_router();

    let result = router.query("what depends on spec::auth::login").unwrap();

    assert_eq!(result.intent, "causal");
    assert!(result.search_results.is_empty());
    assert!(result.causal_context.iter().any(|id| id == "spec::api::rate-limits"));
}

#[test]
fn hybrid_query() {
    let (_dir, router) = setup_router();

    let result = router.query("what depends on rate limiting").unwrap();

    assert_eq!(result.intent, "hybrid");
    assert!(!result.search_results.is_empty());
    assert!(!result.causal_context.is_empty());
}

#[test]
fn both_empty_returns_clear_message() {
    let (_dir, router) = setup_router();

    let result = router.query("totally-unmatched-query-token").unwrap();

    assert_eq!(result.intent, "empty");
    assert!(result.search_results.is_empty());
    assert!(result.causal_context.is_empty());
    assert!(result.message.contains("No search or causal results"));
}

#[test]
fn classification_perf() {
    let start = Instant::now();

    for _ in 0..1_000 {
        let _ = classify("what depends on rate limiting");
    }

    assert!(start.elapsed() < Duration::from_secs(5));
}
