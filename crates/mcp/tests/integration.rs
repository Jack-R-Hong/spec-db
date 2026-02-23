use std::path::PathBuf;

use rmcp::ServerHandler;
use serde_json::json;
use spec_db_mcp::resources::{ResourceUri, parse_resource_uri};
use spec_db_mcp::server::SpecDbMcpServer;
use spec_db_mcp::tools::{GetSpecInput, SearchSpecsInput};

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
    );
    let info = server.get_info();
    assert_eq!(info.server_info.name, "spec-db");
    assert_eq!(info.server_info.version, "0.1.0");
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
}
