//! Acceptance tests for Story 7.2: OpenTelemetry Traces & Metrics

use spec_db_core::load_config;

/// AC1: Search path includes `spec_db.search.query` span naming contract.
#[test]
fn ac1_search_query_span_name_contract() {
    assert_eq!(spec_db_core::telemetry::SPAN_SEARCH_QUERY, "spec_db.search.query");
    let source = include_str!("../crates/search/src/lib.rs");
    assert!(source.contains("spec_db.search.query"));
}

/// AC2: Graph traversal path includes `spec_db.graph.traverse` span naming contract.
#[test]
fn ac2_graph_traverse_span_name_contract() {
    assert_eq!(spec_db_core::telemetry::SPAN_GRAPH_TRAVERSE, "spec_db.graph.traverse");
    let source = include_str!("../crates/spec-db-causal/src/engine.rs");
    assert!(source.contains("name = \"spec_db.graph.traverse\""));
}

/// AC3: Sync paths use `spec_db.sync.{mode}` family via incremental/full constants.
#[test]
fn ac3_sync_span_mode_family_contract() {
    assert!(spec_db_core::telemetry::SPAN_SYNC_INCREMENTAL.starts_with("spec_db.sync."));
    assert!(spec_db_core::telemetry::SPAN_SYNC_FULL_REBUILD.starts_with("spec_db.sync."));
    assert_eq!(spec_db_core::telemetry::SPAN_SYNC_INCREMENTAL, "spec_db.sync.incremental");
    assert_eq!(spec_db_core::telemetry::SPAN_SYNC_FULL_REBUILD, "spec_db.sync.full_rebuild");
}

/// AC4: MCP tool invocation span uses `spec_db.mcp.tool_call` constant.
#[test]
fn ac4_mcp_tool_call_span_name_contract() {
    assert_eq!(spec_db_core::telemetry::SPAN_MCP_TOOL_CALL, "spec_db.mcp.tool_call");
}

/// AC5: Drift metric wiring is deferred; consistency checks currently expose spans/status only.
#[test]
fn ac5_drift_metric_is_deferred_in_current_scope() {
    let source = include_str!("../crates/spec-db-core/src/telemetry.rs");
    assert!(!source.contains("spec_db.consistency.drift_detected"));
    assert!(source.contains("spec_db.consistency.check"));
}

/// AC6: Without telemetry config, defaults disable export and keep local logging mode.
#[test]
fn ac6_telemetry_defaults_disable_export() {
    let cfg = spec_db_core::TelemetryConfig::default();
    assert!(!cfg.enabled);
    assert!(cfg.endpoint.is_none());
    assert_eq!(cfg.protocol, "grpc");
}

/// AC7: When telemetry is configured, endpoint/protocol parse as OTLP-compatible config fields.
#[test]
fn ac7_telemetry_config_parses_endpoint_and_protocol() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.yaml");
    std::fs::write(
        &config_path,
        "telemetry:\n  enabled: true\n  endpoint: http://localhost:4318\n  protocol: http\n",
    )
    .unwrap();

    let cfg = load_config(&config_path).unwrap();
    assert!(cfg.telemetry.enabled);
    assert_eq!(cfg.telemetry.endpoint.as_deref(), Some("http://localhost:4318"));
    assert_eq!(cfg.telemetry.protocol, "http");

    let permissive = dir.path().join("permissive.yaml");
    std::fs::write(&permissive, "telemetry:\n  enabled: true\n  endpoint: 42\n").unwrap();
    let permissive_cfg = load_config(&permissive).unwrap();
    assert_eq!(permissive_cfg.telemetry.endpoint.as_deref(), Some("42"));
    assert_eq!(permissive_cfg.telemetry.protocol, "grpc");
}
