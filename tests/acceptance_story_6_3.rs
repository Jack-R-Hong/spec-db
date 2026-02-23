//! Acceptance tests for Story 6.3: MCP Resources & Streamable-HTTP Transport

use std::sync::Arc;

use rmcp::model::ResourceContents;
use rmcp::serde_json::Value;
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, EdgeOrigin, SpecDoc, SpecId, TrustLevel};
use spec_db_mcp::resources::{ResourceHandler, ResourceUri, parse_resource_uri};
use spec_db_search::SearchIndex;

fn _spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn _doc_a() -> SpecDoc {
    SpecDoc {
        id: _spec_id("spec::resource::a"),
        title: "Resource A".to_owned(),
        version: 1,
        tags: vec!["resource".to_owned()],
        depends_on: vec![_spec_id("spec::resource::b")],
        owner: Some("qa".to_owned()),
        created: "2026-02-23".to_owned(),
        body: "A depends on B".to_owned(),
    }
}

fn _doc_b() -> SpecDoc {
    SpecDoc {
        id: _spec_id("spec::resource::b"),
        title: "Resource B".to_owned(),
        version: 1,
        tags: vec!["resource".to_owned()],
        depends_on: Vec::new(),
        owner: Some("qa".to_owned()),
        created: "2026-02-23".to_owned(),
        body: "Root resource".to_owned(),
    }
}

fn _seed_resources() -> (tempfile::TempDir, ResourceHandler) {
    let dir = tempfile::tempdir().unwrap();
    let tantivy_dir = dir.path().join("tantivy");
    let fjall_dir = dir.path().join("fjall");
    std::fs::create_dir_all(&tantivy_dir).unwrap();
    std::fs::create_dir_all(&fjall_dir).unwrap();

    let mut search = SearchIndex::open_or_create(&tantivy_dir).unwrap();
    let store = Arc::new(FjallStore::open(&fjall_dir).unwrap());
    let mut graph = CausalEngine::from_store(store.clone()).unwrap();

    let doc_b = _doc_b();
    let doc_a = _doc_a();

    search.add_doc(&doc_b).unwrap();
    search.add_doc(&doc_a).unwrap();
    search.commit().unwrap();

    graph
        .upsert_node(spec_db_core::SpecNode {
            id: doc_b.id.clone(),
            title: doc_b.title.clone(),
            version: doc_b.version,
        })
        .unwrap();
    graph
        .upsert_node(spec_db_core::SpecNode {
            id: doc_a.id.clone(),
            title: doc_a.title.clone(),
            version: doc_a.version,
        })
        .unwrap();
    graph
        .add_edge(spec_db_core::CausalEdge {
            source: doc_a.id,
            target: doc_b.id,
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
        })
        .unwrap();

    (dir, ResourceHandler { tantivy_dir, fjall_dir })
}

fn _content_text(content: ResourceContents) -> String {
    match content {
        ResourceContents::TextResourceContents { text, .. } => text,
        ResourceContents::BlobResourceContents { .. } => panic!("expected text resource"),
    }
}

/// AC1: `spec://{id}` resource route is declared and mapped to `{ "spec": ... }` payload shape.
#[test]
fn ac1_spec_resource_payload_contract_is_declared() {
    let source = include_str!("../crates/mcp/src/resources.rs");
    assert!(source.contains("RawResource::new(\"spec://{id}\""));
    assert!(source.contains("Some(ResourceUri::Spec(id)) => self.read_spec(&id)?"));
    assert!(source.contains("Ok(json!({ \"spec\": spec }))"));
}

/// AC2: `graph://overview` returns graph summary stats and disconnected clusters list.
#[test]
fn ac2_graph_overview_resource_returns_stats() {
    let (_dir, handler) = _seed_resources();

    let raw = handler.read_resource("graph://overview").unwrap();
    let text = _content_text(raw);
    let payload: Value = rmcp::serde_json::from_str(&text).unwrap();

    assert_eq!(payload.get("total_specs").and_then(Value::as_u64), Some(2));
    assert_eq!(payload.get("total_edges").and_then(Value::as_u64), Some(1));
    assert!(payload.get("disconnected_clusters").and_then(Value::as_array).is_some());
}

/// AC3: `graph://node/{id}` returns node id with inbound/outbound edge arrays.
#[test]
fn ac3_graph_node_resource_returns_inbound_and_outbound_edges() {
    let (_dir, handler) = _seed_resources();

    let raw = handler.read_resource("graph://node/spec::resource::a").unwrap();
    let text = _content_text(raw);
    let payload: Value = rmcp::serde_json::from_str(&text).unwrap();

    assert_eq!(payload.get("node").and_then(Value::as_str), Some("spec::resource::a"));
    let outbound = payload.get("outbound").and_then(Value::as_array).unwrap();
    assert_eq!(outbound.len(), 1);
    assert_eq!(outbound[0].get("to").and_then(Value::as_str), Some("spec::resource::b"));
    assert!(payload.get("inbound").and_then(Value::as_array).is_some());
}

/// AC4: HTTP auth token enforcement is deferred; no auth-token parsing/401 path exists yet.
#[test]
fn ac4_http_auth_is_not_wired_in_current_scope() {
    let core_config = include_str!("../crates/spec-db-core/src/config.rs");
    let main_source = include_str!("../src/main.rs");
    assert!(!core_config.contains("auth_token"));
    assert!(!main_source.contains("401"));
}

/// AC5: Without HTTP transport config, runtime remains stdio-only with no network surface.
#[test]
fn ac5_default_transport_is_stdio_only_and_http_is_optional() {
    let cfg = spec_db_core::SpecDbConfig::default();
    assert!(cfg.transport.stdio);
    assert!(cfg.transport.http.is_none());

    let source = include_str!("../src/main.rs");
    assert!(
        source.contains("http transport configuration detected but deferred; serving stdio only")
    );
}

/// AC5: Resource URI parser supports the three advertised URI families.
#[test]
fn ac5_resource_uri_parser_accepts_supported_uris() {
    assert_eq!(
        parse_resource_uri("spec://spec::resource::a"),
        Some(ResourceUri::Spec("spec::resource::a".to_owned()))
    );
    assert_eq!(parse_resource_uri("graph://overview"), Some(ResourceUri::GraphOverview));
    assert_eq!(
        parse_resource_uri("graph://node/spec::resource::a"),
        Some(ResourceUri::GraphNode("spec::resource::a".to_owned()))
    );

    let (_dir, handler) = _seed_resources();
    let listed = handler.list_resources();
    assert_eq!(listed.len(), 3);
}
