use std::path::PathBuf;

use rmcp::ServerHandler;
use serde_json::json;
use spec_db_mcp::resources::{ResourceUri, parse_resource_uri};
use spec_db_mcp::server::SpecDbMcpServer;
use spec_db_mcp::tools::{
    AddCausalLinkInput, AddSpecInput, EdgeActionInput, GetSpecInput, SearchSpecsInput, ToolHandler,
};

const SPEC_A: &str = r#"---
id: "spec::test::a"
title: "Spec A"
version: 1
tags: ["test"]
depends_on: []
created: "2026-02-23"
---
# Spec A
"#;

const SPEC_B: &str = r#"---
id: "spec::test::b"
title: "Spec B"
version: 1
tags: ["test"]
depends_on: []
created: "2026-02-23"
---
# Spec B
"#;

fn setup_handler(ai_default_trust: f64) -> (tempfile::TempDir, ToolHandler) {
    let dir = tempfile::tempdir().unwrap();
    let repo_path = dir.path().join("repo");
    let tantivy_dir = dir.path().join("data/tantivy");
    let fjall_dir = dir.path().join("data/fjall");
    std::fs::create_dir_all(&repo_path).unwrap();
    std::fs::create_dir_all(&tantivy_dir).unwrap();
    std::fs::create_dir_all(&fjall_dir).unwrap();

    let handler = ToolHandler {
        repo_path,
        specs_root: "specs".to_owned(),
        tantivy_dir,
        fjall_dir,
        ai_default_trust,
    };
    (dir, handler)
}

fn seed_specs(handler: &ToolHandler) {
    handler.add_spec(AddSpecInput { markdown: SPEC_A.to_owned() }).unwrap();
    handler.add_spec(AddSpecInput { markdown: SPEC_B.to_owned() }).unwrap();
}

fn parse_tool_error(err: spec_db_core::SpecDbError) -> serde_json::Value {
    let message = match err {
        spec_db_core::SpecDbError::IngestError(msg) => msg,
        other => panic!("expected IngestError, got {other:?}"),
    };
    let payload = message
        .strip_prefix("mcp_error::")
        .unwrap_or_else(|| panic!("missing mcp_error prefix: {message}"));
    serde_json::from_str(payload).unwrap_or_else(|e| panic!("invalid error payload: {e}"))
}

#[test]
fn tool_input_deserialization_works() {
    let value = json!({
        "query": "auth",
        "limit": 5,
        "tags": ["security", "api"]
    });
    let input: SearchSpecsInput =
        serde_json::from_value(value).unwrap_or_else(|e| panic!("deserialize failed: {e}"));
    assert_eq!(input.query, "auth");
    assert_eq!(input.limit, Some(5));
    assert_eq!(input.tags.unwrap_or_default().len(), 2);

    let spec: GetSpecInput = serde_json::from_value(json!({ "id": "spec::auth::login" }))
        .unwrap_or_else(|e| panic!("deserialize failed: {e}"));
    assert_eq!(spec.id, "spec::auth::login");
}

#[test]
fn resource_uri_parsing_works() {
    assert_eq!(
        parse_resource_uri("spec://spec::auth::login"),
        Some(ResourceUri::Spec("spec::auth::login".to_owned()))
    );
    assert_eq!(
        parse_resource_uri("graph://node/spec::auth::login"),
        Some(ResourceUri::GraphNode("spec::auth::login".to_owned()))
    );
    assert_eq!(parse_resource_uri("graph://overview"), Some(ResourceUri::GraphOverview));
    assert_eq!(parse_resource_uri("unknown://x"), None);
}

#[test]
fn server_info_contains_name_and_capabilities() {
    let server = SpecDbMcpServer::new(
        PathBuf::from("."),
        "specs".to_owned(),
        PathBuf::from("data/tantivy"),
        PathBuf::from("data/fjall"),
        0.5,
    );
    let info = server.get_info();
    assert_eq!(info.server_info.name, "lattice");
    assert_eq!(info.server_info.version, "0.1.0");
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
}

#[test]
fn add_causal_link_creates_edge_with_default_trust() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);

    let payload = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let edge = payload.get("edge").unwrap();
    assert_eq!(edge.get("from").and_then(|v| v.as_str()), Some("spec::test::a"));
    assert_eq!(edge.get("to").and_then(|v| v.as_str()), Some("spec::test::b"));
    assert_eq!(edge.get("edge_type").and_then(|v| v.as_str()), Some("depends_on"));
    assert_eq!(edge.get("origin").and_then(|v| v.as_str()), Some("ai"));
    assert_eq!(edge.get("trust").and_then(|v| v.as_f64()), Some(0.5));
}

#[test]
fn add_causal_link_uses_custom_default_trust() {
    let (_dir, handler) = setup_handler(0.7);
    seed_specs(&handler);

    let payload = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: Some("depends_on".to_owned()),
        })
        .unwrap();

    let edge = payload.get("edge").unwrap();
    assert_eq!(edge.get("trust").and_then(|v| v.as_f64()), Some(0.7));
}

#[test]
fn add_causal_link_returns_not_found_for_missing_source() {
    let (_dir, handler) = setup_handler(0.5);
    handler.add_spec(AddSpecInput { markdown: SPEC_B.to_owned() }).unwrap();

    let err = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::missing".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("not_found"));
    assert_eq!(payload.get("message").and_then(|v| v.as_str()), Some("Source spec not found"));
    assert_eq!(
        payload.get("context").and_then(|v| v.get("id")).and_then(|v| v.as_str()),
        Some("spec::test::missing")
    );
}

#[test]
fn add_causal_link_returns_not_found_for_missing_target() {
    let (_dir, handler) = setup_handler(0.5);
    handler.add_spec(AddSpecInput { markdown: SPEC_A.to_owned() }).unwrap();

    let err = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::missing".to_owned(),
            edge_type: None,
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("not_found"));
    assert_eq!(payload.get("message").and_then(|v| v.as_str()), Some("Target spec not found"));
    assert_eq!(
        payload.get("context").and_then(|v| v.get("id")).and_then(|v| v.as_str()),
        Some("spec::test::missing")
    );
}

#[test]
fn add_causal_link_rejects_self_reference() {
    let (_dir, handler) = setup_handler(0.5);
    handler.add_spec(AddSpecInput { markdown: SPEC_A.to_owned() }).unwrap();

    let err = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::a".to_owned(),
            edge_type: None,
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("validation_error"));
    assert_eq!(
        payload.get("message").and_then(|v| v.as_str()),
        Some("Self-referencing edges are not allowed")
    );
}

#[test]
fn add_causal_link_rejects_duplicates() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);
    handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: Some("depends_on".to_owned()),
        })
        .unwrap();

    let err = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: Some("depends_on".to_owned()),
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("conflict"));
    assert_eq!(payload.get("message").and_then(|v| v.as_str()), Some("Edge already exists"));
}

#[test]
fn add_causal_link_parses_all_edge_types_and_rejects_invalid() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);

    let cases = [
        ("depends_on", "spec::test::a", "spec::test::b"),
        ("constrains", "spec::test::a", "spec::test::b"),
        ("implements", "spec::test::a", "spec::test::b"),
    ];

    for (index, (edge_type, source, target)) in cases.into_iter().enumerate() {
        let source_id = format!("{source}-{index}");
        let target_id = format!("{target}-{index}");
        let source_markdown = format!(
            "---\nid: \"{source_id}\"\ntitle: \"Source {index}\"\nversion: 1\ntags: [\"test\"]\ndepends_on: []\ncreated: \"2026-02-23\"\n---\n# Source\n"
        );
        let target_markdown = format!(
            "---\nid: \"{target_id}\"\ntitle: \"Target {index}\"\nversion: 1\ntags: [\"test\"]\ndepends_on: []\ncreated: \"2026-02-23\"\n---\n# Target\n"
        );
        handler.add_spec(AddSpecInput { markdown: source_markdown }).unwrap();
        handler.add_spec(AddSpecInput { markdown: target_markdown }).unwrap();

        let payload = handler
            .add_causal_link(AddCausalLinkInput {
                source: source_id,
                target: target_id,
                edge_type: Some(edge_type.to_owned()),
            })
            .unwrap();
        assert_eq!(
            payload.get("edge").and_then(|v| v.get("edge_type")).and_then(|v| v.as_str()),
            Some(edge_type)
        );
    }

    let err = handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: Some("invalid".to_owned()),
        })
        .unwrap_err();
    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("validation_error"));
    assert_eq!(payload.get("message").and_then(|v| v.as_str()), Some("Invalid edge type"));
}

#[test]
fn add_causal_link_tool_is_registered_on_server() {
    let source = include_str!("../src/server.rs");
    assert!(source.contains("\"add_causal_link\""));
    assert!(source.contains("AddCausalLinkInput"));
}

#[test]
fn promote_edge_changes_origin_to_human() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);
    handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let payload = handler
        .promote_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
    let edge = payload.get("edge").unwrap();
    assert_eq!(edge.get("origin").and_then(|v| v.as_str()), Some("human"));
    assert_eq!(edge.get("trust").and_then(|v| v.as_f64()), Some(1.0));
}

#[test]
fn promote_already_human_edge_returns_validation_error() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);
    handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();
    handler
        .promote_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let err = handler
        .promote_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("validation_error"));
    assert_eq!(
        payload.get("message").and_then(|v| v.as_str()),
        Some("Edge is already human-curated")
    );
}

#[test]
fn promote_nonexistent_edge_returns_not_found() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);

    let err = handler
        .promote_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("not_found"));
}

#[test]
fn reject_edge_removes_from_graph() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);
    handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let payload = handler
        .reject_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    assert_eq!(payload.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(payload.get("message").and_then(|v| v.as_str()), Some("edge rejected and removed"));
}

#[test]
fn reject_nonexistent_edge_returns_not_found() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);

    let err = handler
        .reject_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap_err();

    let payload = parse_tool_error(err);
    assert_eq!(payload.get("error_type").and_then(|v| v.as_str()), Some("not_found"));
}

#[test]
fn promote_removes_from_edges_yaml() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);
    handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let edges_yaml = handler.repo_path.join(".lattice/edges.yaml");
    let before = std::fs::read_to_string(&edges_yaml).unwrap();
    assert!(before.contains("spec::test::a"));

    handler
        .promote_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let after = std::fs::read_to_string(&edges_yaml).unwrap();
    assert!(!after.contains("spec::test::a"));
    assert!(after.contains("edges: []"));
}

#[test]
fn reject_removes_from_edges_yaml() {
    let (_dir, handler) = setup_handler(0.5);
    seed_specs(&handler);
    handler
        .add_causal_link(AddCausalLinkInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let edges_yaml = handler.repo_path.join(".lattice/edges.yaml");
    let before = std::fs::read_to_string(&edges_yaml).unwrap();
    assert!(before.contains("spec::test::a"));

    handler
        .reject_edge(EdgeActionInput {
            source: "spec::test::a".to_owned(),
            target: "spec::test::b".to_owned(),
            edge_type: None,
        })
        .unwrap();

    let after = std::fs::read_to_string(&edges_yaml).unwrap();
    assert!(!after.contains("spec::test::a"));
    assert!(after.contains("edges: []"));
}

#[test]
fn promote_and_reject_tools_are_registered_on_server() {
    let source = include_str!("../src/server.rs");
    assert!(source.contains("\"promote_edge\""));
    assert!(source.contains("\"reject_edge\""));
    assert!(source.contains("EdgeActionInput"));
}
