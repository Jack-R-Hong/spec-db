//! Acceptance tests for Story 3.2: Unified Spec Ingestion Pipeline

use std::sync::Arc;
use std::time::{Duration, Instant};

use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, SearchEngine, SpecDbError, SpecId};
use spec_db_ingest::IngestPipeline;
use spec_db_search::SearchIndex;

fn _spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn _setup_pipeline() -> (tempfile::TempDir, IngestPipeline<SearchIndex, CausalEngine>) {
    let dir = tempfile::tempdir().unwrap();
    let search_path = dir.path().join("tantivy");
    let graph_path = dir.path().join("fjall");
    let search = SearchIndex::open_or_create(&search_path).unwrap();
    let graph_store = Arc::new(FjallStore::open(&graph_path).unwrap());
    let graph = CausalEngine::from_store(graph_store).unwrap();
    (dir, IngestPipeline::new(search, graph))
}

const SPEC_LOGIN: &str = r#"---
id: "spec::auth::login"
title: "Login Flow"
version: 1
tags: ["auth"]
depends_on: ["spec::auth::token"]
owner: "backend"
created: "2026-02-23"
---
# Login Flow
credential-check keyword-login
"#;

const SPEC_TOKEN: &str = r#"---
id: "spec::auth::token"
title: "Token Issuance"
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-02-23"
---
# Token Issuance
issue-token keyword-token
"#;

const SPEC_FORWARD_REF: &str = r#"---
id: "spec::auth::session"
title: "Session Flow"
version: 1
tags: ["auth"]
depends_on: ["spec::auth::future-target"]
owner: "backend"
created: "2026-02-23"
---
# Session Flow
session keyword-session
"#;

/// AC1: add_spec(markdown) ingests into both search index and causal graph.
#[test]
fn ac1_add_spec_ingests_into_search_and_graph() {
    let (_dir, mut pipeline) = _setup_pipeline();

    let id = pipeline.add_spec(SPEC_LOGIN).unwrap();

    let search_hits = pipeline.search().search("keyword-login", 10).unwrap();
    let graph_node = pipeline.graph().get_node(&id).unwrap();

    assert_eq!(search_hits, vec![id.clone()]);
    assert!(graph_node.is_some());
}

/// AC2: Spec ingestion creates the indexed document, graph node, and causal edges.
#[test]
fn ac2_ingest_creates_index_entry_node_and_edges() {
    let (_dir, mut pipeline) = _setup_pipeline();
    let token_id = pipeline.add_spec(SPEC_TOKEN).unwrap();
    let login_id = pipeline.add_spec(SPEC_LOGIN).unwrap();

    let indexed = pipeline.search().search("keyword-login", 10).unwrap();
    let login_node = pipeline.graph().get_node(&login_id).unwrap();
    let outbound_edges = pipeline.graph().edges_from(&login_id).unwrap();

    assert_eq!(indexed, vec![login_id.clone()]);
    assert!(login_node.is_some());
    assert_eq!(outbound_edges.len(), 1);
    assert_eq!(outbound_edges[0].target, token_id);
}

/// AC3: Duplicate spec IDs return IngestError and leave both stores unchanged.
#[test]
fn ac3_duplicate_spec_id_returns_error_and_no_store_mutation() {
    let (_dir, mut pipeline) = _setup_pipeline();
    let original_id = pipeline.add_spec(SPEC_TOKEN).unwrap();

    let before_search = pipeline.search().search("keyword-token", 10).unwrap();
    let before_nodes = pipeline.graph().get_node(&original_id).unwrap();
    let before_edges = pipeline.graph().edges_from(&original_id).unwrap();

    let err = pipeline.add_spec(SPEC_TOKEN).unwrap_err();
    match err {
        SpecDbError::IngestError(message) => assert!(message.contains("duplicate spec ID")),
        other => panic!("expected IngestError, got {other:?}"),
    }

    let after_search = pipeline.search().search("keyword-token", 10).unwrap();
    let after_nodes = pipeline.graph().get_node(&original_id).unwrap();
    let after_edges = pipeline.graph().edges_from(&original_id).unwrap();

    assert_eq!(before_search, after_search);
    assert_eq!(before_nodes.is_some(), after_nodes.is_some());
    assert_eq!(before_edges.len(), after_edges.len());
}

/// AC4: Single-spec ingestion completes in under 100ms.
#[test]
#[ignore = "expensive: verifies sub-100ms ingest timing"]
fn ac4_ingest_completes_under_one_hundred_milliseconds() {
    let (_dir, mut pipeline) = _setup_pipeline();

    let start = Instant::now();
    let _id = pipeline.add_spec(SPEC_TOKEN).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(100), "ingest took {elapsed:?}");
}

/// AC5: Forward-reference causal edges are created even when target specs are absent.
#[test]
fn ac5_forward_reference_edges_created_without_target_present() {
    let (_dir, mut pipeline) = _setup_pipeline();
    let source_id = pipeline.add_spec(SPEC_FORWARD_REF).unwrap();
    let missing_target = _spec_id("spec::auth::future-target");

    let edges = pipeline.graph().edges_from(&source_id).unwrap();
    let target_node = pipeline.graph().get_node(&missing_target).unwrap();

    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].target, missing_target);
    assert!(target_node.is_none());
}
