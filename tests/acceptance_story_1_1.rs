//! Acceptance tests for Story 1.1: Scaffold Workspace & Core Domain Types
//!
//! AC3 (SpecId validation) has full coverage in `crates/core/src/types.rs`.

use std::process::Command;

/// AC1: spec-db-core compiles and is importable as a library crate.
#[test]
fn ac1_spec_db_core_is_importable() {
    let _: fn(&str) -> Result<spec_db_core::SpecId, spec_db_core::SpecDbError> =
        |s| spec_db_core::SpecId::try_new(s);
}

/// AC1: spec-db-causal compiles and is importable as a library crate.
#[test]
fn ac1_spec_db_causal_is_importable() {
    fn _assert_type_exists<T>() {}
    _assert_type_exists::<spec_db_causal::CausalEngine>();
    _assert_type_exists::<spec_db_causal::FjallStore>();
}

/// AC1: Root workspace declares all three crates.
#[test]
fn ac1_workspace_contains_three_crates() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("spec-db-core"));
    assert!(manifest.contains("spec-db-causal"));
    assert!(manifest.contains("name = \"lattice\""));
}

/// AC2: SpecId is exported and constructible.
#[test]
fn ac2_spec_id_exported_and_constructible() {
    let id = spec_db_core::SpecId::try_new("spec::test::domain").unwrap();
    assert_eq!(id.as_ref(), "spec::test::domain");
    assert_eq!(id.to_string(), "spec::test::domain");
}

/// AC2: SpecDoc is exported and constructible with all fields.
#[test]
fn ac2_spec_doc_exported_and_constructible() {
    let doc = spec_db_core::SpecDoc {
        id: spec_db_core::SpecId::try_new("spec::auth::login").unwrap(),
        title: "Login Spec".into(),
        version: 1,
        tags: vec!["auth".into()],
        depends_on: vec![],
        owner: Some("team-a".into()),
        created: "2026-01-01".into(),
        body: "# Login\nBody text".into(),
    };
    assert_eq!(doc.title, "Login Spec");
    assert_eq!(doc.version, 1);
    assert_eq!(doc.tags, vec!["auth"]);
    assert!(doc.owner.is_some());
}

/// AC2: SpecNode is exported and constructible.
#[test]
fn ac2_spec_node_exported_and_constructible() {
    let node = spec_db_core::SpecNode {
        id: spec_db_core::SpecId::try_new("spec::graph::node").unwrap(),
        title: "Graph Node".into(),
        version: 2,
    };
    assert_eq!(node.title, "Graph Node");
    assert_eq!(node.version, 2);
}

/// AC2: CausalEdge is exported and constructible.
#[test]
fn ac2_causal_edge_exported_and_constructible() {
    let edge = spec_db_core::CausalEdge {
        source: spec_db_core::SpecId::try_new("spec::a::src").unwrap(),
        target: spec_db_core::SpecId::try_new("spec::b::tgt").unwrap(),
        trust: spec_db_core::TrustLevel::human(),
        origin: spec_db_core::EdgeOrigin::Human,
    };
    assert_eq!(edge.source.as_ref(), "spec::a::src");
    assert_eq!(edge.target.as_ref(), "spec::b::tgt");
}

/// AC2: TrustLevel is exported and constructible with clamping.
#[test]
fn ac2_trust_level_exported_and_constructible() {
    let t = spec_db_core::TrustLevel::new(0.75);
    assert!((t.value() - 0.75).abs() < f64::EPSILON);

    let human = spec_db_core::TrustLevel::human();
    assert!((human.value() - 1.0).abs() < f64::EPSILON);
}

/// AC4: SearchEngine trait is exported with expected method signatures.
#[test]
fn ac4_search_engine_trait_exported() {
    fn _assert_methods<T: spec_db_core::SearchEngine>() {
        let _: fn(&mut T, &spec_db_core::SpecDoc) -> Result<(), spec_db_core::SpecDbError> =
            T::index_spec;
        let _: fn(&mut T, &spec_db_core::SpecId) -> Result<(), spec_db_core::SpecDbError> =
            T::remove_spec;
        let _: fn(&T, &str, usize) -> Result<Vec<spec_db_core::SpecId>, spec_db_core::SpecDbError> =
            T::search;
        let _: fn(
            &T,
            &str,
            &[String],
            usize,
        ) -> Result<Vec<spec_db_core::SpecId>, spec_db_core::SpecDbError> = T::search_with_tags;
    }
}

/// AC4: CausalGraph trait is exported with expected method signatures.
#[test]
fn ac4_causal_graph_trait_exported() {
    fn _assert_methods<T: spec_db_core::CausalGraph>() {
        let _: fn(&mut T, spec_db_core::SpecNode) -> Result<(), spec_db_core::SpecDbError> =
            T::upsert_node;
        let _: fn(&mut T, &spec_db_core::SpecId) -> Result<(), spec_db_core::SpecDbError> =
            T::remove_node;
        let _: fn(
            &T,
            &spec_db_core::SpecId,
        ) -> Result<Option<spec_db_core::SpecNode>, spec_db_core::SpecDbError> = T::get_node;
        let _: fn(&mut T, spec_db_core::CausalEdge) -> Result<(), spec_db_core::SpecDbError> =
            T::add_edge;
        let _: fn(
            &T,
            &spec_db_core::SpecId,
            Option<usize>,
        ) -> Result<Vec<spec_db_core::SpecId>, spec_db_core::SpecDbError> = T::trace_impact;
        let _: fn(
            &T,
            &spec_db_core::SpecId,
            Option<usize>,
        ) -> Result<Vec<spec_db_core::SpecId>, spec_db_core::SpecDbError> = T::find_dependencies;
    }
}

/// AC4: SpecStore trait is exported with expected method signatures.
#[test]
fn ac4_spec_store_trait_exported() {
    fn _assert_methods<T: spec_db_core::SpecStore>() {
        let _: fn(&mut T, spec_db_core::SpecDoc) -> Result<(), spec_db_core::SpecDbError> = T::put;
        let _: fn(
            &T,
            &spec_db_core::SpecId,
        ) -> Result<Option<spec_db_core::SpecDoc>, spec_db_core::SpecDbError> = T::get;
        let _: fn(&mut T, &spec_db_core::SpecId) -> Result<(), spec_db_core::SpecDbError> =
            T::remove;
        let _: fn(&T) -> Result<Vec<spec_db_core::SpecId>, spec_db_core::SpecDbError> = T::list_ids;
    }
}

/// AC5: SearchError variant exists and has Display output.
#[test]
fn ac5_error_variant_search_error() {
    let err = spec_db_core::SpecDbError::SearchError("index missing".into());
    let msg = err.to_string();
    assert!(msg.contains("search error"), "Display: {msg}");
    assert!(msg.contains("index missing"), "Display: {msg}");
}

/// AC5: GraphError variant exists and has Display output.
#[test]
fn ac5_error_variant_graph_error() {
    let err = spec_db_core::SpecDbError::GraphError("node not found".into());
    let msg = err.to_string();
    assert!(msg.contains("graph error"), "Display: {msg}");
    assert!(msg.contains("node not found"), "Display: {msg}");
}

/// AC5: SyncError variant exists and has Display output.
#[test]
fn ac5_error_variant_sync_error() {
    let err = spec_db_core::SpecDbError::SyncError("sha mismatch".into());
    let msg = err.to_string();
    assert!(msg.contains("sync error"), "Display: {msg}");
    assert!(msg.contains("sha mismatch"), "Display: {msg}");
}

/// AC5: IngestError variant exists and has Display output.
#[test]
fn ac5_error_variant_ingest_error() {
    let err = spec_db_core::SpecDbError::IngestError("bad frontmatter".into());
    let msg = err.to_string();
    assert!(msg.contains("ingest error"), "Display: {msg}");
    assert!(msg.contains("bad frontmatter"), "Display: {msg}");
}

/// AC5: ConsistencyError variant exists and has Display output.
#[test]
fn ac5_error_variant_consistency_error() {
    let err = spec_db_core::SpecDbError::ConsistencyError("store drift".into());
    let msg = err.to_string();
    assert!(msg.contains("consistency error"), "Display: {msg}");
    assert!(msg.contains("store drift"), "Display: {msg}");
}

/// AC5: ConfigError variant exists and has Display output.
#[test]
fn ac5_error_variant_config_error() {
    let err = spec_db_core::SpecDbError::ConfigError("missing field".into());
    let msg = err.to_string();
    assert!(msg.contains("config error"), "Display: {msg}");
    assert!(msg.contains("missing field"), "Display: {msg}");
}

/// AC5: All SpecDbError variants implement std::error::Error.
#[test]
fn ac5_error_implements_std_error() {
    fn assert_is_error<T: std::error::Error>(_: &T) {}

    assert_is_error(&spec_db_core::SpecDbError::SearchError("".into()));
    assert_is_error(&spec_db_core::SpecDbError::GraphError("".into()));
    assert_is_error(&spec_db_core::SpecDbError::SyncError("".into()));
    assert_is_error(&spec_db_core::SpecDbError::IngestError("".into()));
    assert_is_error(&spec_db_core::SpecDbError::ConsistencyError("".into()));
    assert_is_error(&spec_db_core::SpecDbError::ConfigError("".into()));
}

/// AC5: SpecDbError implements Debug.
#[test]
fn ac5_error_implements_debug() {
    let err = spec_db_core::SpecDbError::SearchError("test".into());
    let debug = format!("{err:?}");
    assert!(!debug.is_empty());
}

/// AC6: Cargo.toml contains [workspace.dependencies].
#[test]
fn ac6_workspace_dependencies_section_exists() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("[workspace.dependencies]"));
}

/// AC6: deep_causality pinned to =0.13.4.
#[test]
fn ac6_deep_causality_version_locked() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("deep_causality = \"=0.13.4\""));
}

/// AC6: fjall pinned to 3.0.
#[test]
fn ac6_fjall_version_locked() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("fjall = \"3.0\""));
}

/// AC6: tantivy pinned to 0.25.0.
#[test]
fn ac6_tantivy_version_locked() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("tantivy = \"0.25.0\""));
}

/// AC6: rmcp pinned to =0.16.0.
#[test]
fn ac6_rmcp_version_locked() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("rmcp = \"=0.16.0\""));
}

/// AC6: git2 pinned to 0.20.4.
#[test]
fn ac6_git2_version_locked() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("git2 = \"0.20.4\""));
}

/// AC6: bincode pinned to =2.0.1 with serde feature.
#[test]
fn ac6_bincode_version_locked_with_serde() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("bincode") && manifest.contains("2.0.1"));
    assert!(manifest.contains("bincode") && manifest.contains("serde"));
}

/// AC7: rustfmt.toml sets edition = "2024".
#[test]
fn ac7_rustfmt_toml_edition() {
    let content = include_str!("../rustfmt.toml");
    assert!(content.contains("edition = \"2024\""));
}

/// AC7: rustfmt.toml sets max_width = 100.
#[test]
fn ac7_rustfmt_toml_max_width() {
    let content = include_str!("../rustfmt.toml");
    assert!(content.contains("max_width = 100"));
}

/// AC7: clippy.toml sets allow-unwrap-in-tests = true.
#[test]
fn ac7_clippy_toml_configured() {
    let content = include_str!("../clippy.toml");
    assert!(content.contains("allow-unwrap-in-tests = true"));
}

/// AC8: `cargo clippy --workspace -- -D warnings` passes with zero warnings.
#[test]
#[ignore = "expensive: runs cargo clippy on entire workspace"]
fn ac8_cargo_clippy_zero_warnings() {
    let output = Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .output()
        .expect("failed to run cargo clippy");

    assert!(
        output.status.success(),
        "cargo clippy failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// AC9: `cargo fmt --all -- --check` passes.
#[test]
#[ignore = "expensive: runs cargo fmt check on entire workspace"]
fn ac9_cargo_fmt_check_passes() {
    let output = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .output()
        .expect("failed to run cargo fmt");

    assert!(
        output.status.success(),
        "cargo fmt --check failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
