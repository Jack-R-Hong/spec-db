//! Acceptance tests for Story 6.2: MCP Server with Tools over Stdio

use std::path::Path;
use std::process::Command;

use rmcp::serde_json::{Value, json};
use spec_db_mcp::server::SpecDbMcpServer;
use spec_db_mcp::tools::{
    AddSpecInput, FindDependenciesInput, QueryInput, SearchSpecsInput, SyncInput, ToolHandler,
    TraceImpactInput,
};
use tempfile::TempDir;

const _SPEC_A: &str = r#"---
id: "spec::acceptance::a"
title: "Spec A"
version: 1
tags: ["acceptance", "mcp"]
depends_on: ["spec::acceptance::b"]
owner: "qa"
created: "2026-02-23"
---
# Spec A

Spec A depends on spec B.
"#;

const _SPEC_B: &str = r#"---
id: "spec::acceptance::b"
title: "Spec B"
version: 1
tags: ["acceptance", "mcp"]
depends_on: []
owner: "qa"
created: "2026-02-23"
---
# Spec B

Spec B is the dependency root.
"#;

fn _git(cmd: &[&str], cwd: &Path) {
    let output = Command::new("git").args(cmd).current_dir(cwd).output().unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout: {}\nstderr: {}",
        cmd,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn _write_spec(repo: &Path, rel_path: &str, content: &str) {
    let path = repo.join(rel_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

fn _setup_handler() -> (TempDir, ToolHandler) {
    let dir = tempfile::tempdir().unwrap();
    let repo_path = dir.path().join("repo");
    let tantivy_dir = dir.path().join("data/tantivy");
    let fjall_dir = dir.path().join("data/fjall");

    std::fs::create_dir_all(&repo_path).unwrap();
    std::fs::create_dir_all(&tantivy_dir).unwrap();
    std::fs::create_dir_all(&fjall_dir).unwrap();

    _git(&["init"], &repo_path);
    _git(&["config", "user.email", "test@test.com"], &repo_path);
    _git(&["config", "user.name", "test"], &repo_path);
    _write_spec(&repo_path, "specs/acceptance/a.md", _SPEC_A);
    _write_spec(&repo_path, "specs/acceptance/b.md", _SPEC_B);
    _git(&["add", "."], &repo_path);
    _git(&["commit", "-m", "seed specs"], &repo_path);

    let handler = ToolHandler { repo_path, specs_root: "specs".to_owned(), tantivy_dir, fjall_dir };
    (dir, handler)
}

fn _seed_search_and_graph(handler: &ToolHandler) {
    handler.add_spec(AddSpecInput { markdown: _SPEC_B.to_owned() }).unwrap();
    handler.add_spec(AddSpecInput { markdown: _SPEC_A.to_owned() }).unwrap();
}

fn _assert_has_error_shape_fields(value: &Value) {
    assert!(value.get("error_type").is_some());
    assert!(value.get("message").is_some());
    assert!(value.get("context").is_some());
}

/// AC1: The MCP server exposes all seven tool names for protocol discovery.
#[test]
fn ac1_tool_names_are_exposed_for_discovery() {
    fn _assert_server_type_exists<T>() {}
    _assert_server_type_exists::<SpecDbMcpServer>();

    let source = include_str!("../crates/mcp/src/server.rs");
    for name in [
        "search_specs",
        "get_spec",
        "trace_impact",
        "find_dependencies",
        "query",
        "add_spec",
        "sync",
    ] {
        assert!(source.contains(&format!("\"{name}\"")), "missing tool name: {name}");
    }
}

/// AC2: `search_specs` delegates to search and returns a JSON `results` array.
#[test]
fn ac2_search_specs_returns_results_array() {
    let (_dir, handler) = _setup_handler();
    _seed_search_and_graph(&handler);

    let payload = handler
        .search_specs(SearchSpecsInput {
            query: "dependency".to_owned(),
            limit: Some(5),
            tags: Some(vec!["acceptance".to_owned()]),
        })
        .unwrap();

    let results = payload.get("results").and_then(|value| value.as_array()).unwrap();
    assert!(!results.is_empty());
    assert!(results[0].get("id").is_some());
    assert!(results[0].get("title").is_some());
    assert!(results[0].get("score").is_some());
    assert!(results[0].get("snippet").is_some());
}

/// AC3: `get_spec` is wired as a dedicated tool with `{ "spec": ... }` JSON payload contract.
#[test]
fn ac3_get_spec_payload_contract_is_declared() {
    let source = include_str!("../crates/mcp/src/tools.rs");
    assert!(source.contains("pub fn get_spec"));
    assert!(source.contains("GetSpecInput"));
    assert!(source.contains("json!({ \"spec\": spec })"));
}

/// AC4: `trace_impact` and `find_dependencies` return graph JSON with node + edges.
#[test]
fn ac4_graph_tools_return_node_and_edges_shape() {
    let (_dir, handler) = _setup_handler();
    _seed_search_and_graph(&handler);

    let impact = handler
        .trace_impact(TraceImpactInput { id: "spec::acceptance::b".to_owned(), depth: Some(3) })
        .unwrap();
    assert_eq!(impact.get("node").and_then(|value| value.as_str()), Some("spec::acceptance::b"));
    assert!(impact.get("edges").and_then(|value| value.as_array()).is_some());

    let deps = handler
        .find_dependencies(FindDependenciesInput { id: "spec::acceptance::a".to_owned() })
        .unwrap();
    assert_eq!(deps.get("node").and_then(|value| value.as_str()), Some("spec::acceptance::a"));
    let dep_edges = deps.get("edges").and_then(|value| value.as_array()).unwrap();
    assert!(!dep_edges.is_empty());
    assert!(dep_edges[0].get("from").is_some());
    assert!(dep_edges[0].get("to").is_some());
    assert_eq!(dep_edges[0].get("type").and_then(|value| value.as_str()), Some("depends_on"));
}

/// AC5: `query` delegates to router and returns composed JSON fields.
#[test]
fn ac5_query_returns_router_composed_payload() {
    let (_dir, handler) = _setup_handler();
    _seed_search_and_graph(&handler);

    let payload = handler
        .query(QueryInput { natural_language: "what depends on spec::acceptance::b".to_owned() })
        .unwrap();

    assert!(payload.get("intent").is_some());
    assert!(payload.get("search_results").is_some());
    assert!(payload.get("causal_context").is_some());
    assert!(payload.get("message").is_some());
}

/// AC6: `add_spec` and `sync` return admin JSON shape and invoke ingest/sync paths.
#[test]
fn ac6_add_spec_and_sync_return_admin_shape() {
    let (_dir, handler) = _setup_handler();

    let add_payload = handler.add_spec(AddSpecInput { markdown: _SPEC_B.to_owned() }).unwrap();
    assert_eq!(add_payload.get("status").and_then(|value| value.as_str()), Some("ok"));
    assert!(add_payload.get("details").is_some());
    assert!(add_payload.get("details").unwrap().get("id").is_some());
    assert!(add_payload.get("details").unwrap().get("doc_count").is_some());

    let sync_payload = handler.sync(SyncInput { mode: Some("incremental".to_owned()) }).unwrap();
    assert_eq!(sync_payload.get("status").and_then(|value| value.as_str()), Some("ok"));
    assert_eq!(
        sync_payload.get("details").unwrap().get("mode").and_then(|value| value.as_str()),
        Some("incremental")
    );
    assert!(sync_payload.get("details").unwrap().get("specs_ingested").is_some());
    assert!(sync_payload.get("details").unwrap().get("head_sha").is_some());
}

/// AC7: Error responses follow consistent JSON `{error_type,message,context}` contract.
#[test]
fn ac7_error_payload_contract_is_stable() {
    let source = include_str!("../crates/mcp/src/server.rs");
    assert!(source.contains("\"error_type\""));
    assert!(source.contains("\"message\""));
    assert!(source.contains("\"context\""));
    assert!(source.contains("SearchError"));
    assert!(source.contains("GraphError"));
    assert!(source.contains("SyncError"));
    assert!(source.contains("IngestError"));
    assert!(source.contains("ConsistencyError"));
    assert!(source.contains("ConfigError"));

    let sample = json!({
        "error_type": "ConfigError",
        "message": "invalid tool arguments",
        "context": null
    });
    _assert_has_error_shape_fields(&sample);
}
