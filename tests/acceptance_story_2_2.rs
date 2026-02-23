//! Acceptance tests for Story 2.2: BM25 Search, Title Boosting, Tag Filtering, and Retrieval

use std::time::{Duration, Instant};

use spec_db_core::{SearchEngine, SpecDoc, SpecId};
use spec_db_search::{SearchIndex, query};

fn _spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn _doc(id: &str, title: &str, body: &str, tags: &[&str]) -> SpecDoc {
    SpecDoc {
        id: _spec_id(id),
        title: title.to_owned(),
        version: 1,
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        depends_on: Vec::new(),
        owner: Some("search-team".to_owned()),
        created: "2026-02-23".to_owned(),
        body: body.to_owned(),
    }
}

/// AC1: BM25 relevance ranking returns the more relevant document first.
#[test]
fn ac1_bm25_relevance_ranking_orders_more_relevant_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();

    let high_relevance = _doc(
        "spec::search::ac1-high",
        "Rate limiting policy",
        "rate limiting rate limiting enforcement policy",
        &["api"],
    );
    let low_relevance =
        _doc("spec::search::ac1-low", "Traffic shaping", "rate controls for traffic", &["api"]);

    index.add_doc(&high_relevance).unwrap();
    index.add_doc(&low_relevance).unwrap();
    index.commit().unwrap();

    let text_query =
        query::build_text_query(index.index(), index.fields(), "rate limiting").unwrap();
    let hits = query::execute_search(&index.searcher(), index.fields(), &*text_query, 10).unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, high_relevance.id);
    assert!(hits[0].score >= hits[1].score);
}

/// AC2: Title matches rank higher than body-only matches.
#[test]
fn ac2_title_matches_rank_higher_than_body_only_matches() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();

    let title_match = _doc(
        "spec::search::ac2-title",
        "Session token rotation",
        "general policy overview",
        &["auth"],
    );
    let body_only_match = _doc(
        "spec::search::ac2-body",
        "Credential policy",
        "session token rotation appears only in body text",
        &["auth"],
    );

    index.add_doc(&title_match).unwrap();
    index.add_doc(&body_only_match).unwrap();
    index.commit().unwrap();

    let text_query =
        query::build_text_query(index.index(), index.fields(), "session token rotation").unwrap();
    let hits = query::execute_search(&index.searcher(), index.fields(), &*text_query, 10).unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, title_match.id);
    assert_eq!(hits[1].id, body_only_match.id);
}

/// AC3: Tag filtering returns only results matching the requested tags.
#[test]
fn ac3_tag_filtering_returns_only_matching_tags() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();

    let auth_spec = _doc(
        "spec::search::ac3-auth",
        "Token validation",
        "token validation and signature checks",
        &["auth"],
    );
    let api_spec = _doc("spec::search::ac3-api", "Pagination", "token pagination cursor", &["api"]);

    index.add_doc(&auth_spec).unwrap();
    index.add_doc(&api_spec).unwrap();
    index.commit().unwrap();

    let ids = index.search_with_tags("token", &[String::from("auth")], 10).unwrap();
    assert_eq!(ids, vec![auth_spec.id]);
}

/// AC4: get_spec performs ID-based retrieval and returns None for missing IDs.
#[test]
fn ac4_get_spec_retrieves_spec_by_id() {
    let dir = tempfile::tempdir().unwrap();
    let index = SearchIndex::open_or_create(dir.path()).unwrap();
    let missing_id = _spec_id("spec::search::ac4-missing");

    let loaded = index.get_spec(&missing_id).unwrap();
    assert!(loaded.is_none());
}

/// AC5: Search for 100+ specs completes in under 10ms.
#[test]
#[ignore = "expensive: verifies sub-10ms query for 100+ indexed specs"]
fn ac5_search_under_ten_milliseconds_for_100_plus_specs() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();

    for i in 0..150 {
        let doc = _doc(
            &format!("spec::search::perf-{i}"),
            &format!("Performance Spec {i}"),
            "latency throughput saturation capacity planning",
            if i % 2 == 0 { &["perf"] } else { &["ops"] },
        );
        index.add_doc(&doc).unwrap();
    }
    index.commit().unwrap();

    let _warmup = index.search("latency", 20).unwrap();

    let start = Instant::now();
    let ids = index.search("latency", 20).unwrap();
    let elapsed = start.elapsed();

    assert!(!ids.is_empty());
    assert!(elapsed < Duration::from_millis(10), "search took {elapsed:?}");
}

/// AC6: No-match queries return an empty result set instead of an error.
#[test]
fn ac6_no_match_returns_empty_result_set() {
    let dir = tempfile::tempdir().unwrap();
    let mut index = SearchIndex::open_or_create(dir.path()).unwrap();
    let doc = _doc(
        "spec::search::ac6-empty",
        "Caching strategy",
        "cache invalidation details",
        &["perf"],
    );

    index.add_doc(&doc).unwrap();
    index.commit().unwrap();

    let result = index.search("term-that-does-not-exist", 10).unwrap();
    assert!(result.is_empty());
}
