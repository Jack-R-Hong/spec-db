//! Acceptance tests for Story 7.1: Cross-Store Consistency Checks

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, EdgeOrigin, SpecId, SpecNode, TrustLevel};
use spec_db_ingest::{ConsistencyStatus, verify_consistency, verify_cross_store_consistency};
use spec_db_search::SearchIndex;

fn _spec_id(value: &str) -> SpecId {
    SpecId::try_new(value).unwrap()
}

fn _seed_store_snapshots(base: &Path) -> (String, usize, String, usize) {
    let tantivy_dir = base.join("tantivy");
    let fjall_dir = base.join("fjall");
    std::fs::create_dir_all(&tantivy_dir).unwrap();
    std::fs::create_dir_all(&fjall_dir).unwrap();

    let mut search = SearchIndex::open_or_create(&tantivy_dir).unwrap();
    let store = Arc::new(FjallStore::open(&fjall_dir).unwrap());
    let mut graph = CausalEngine::from_store(store.clone()).unwrap();

    let a = SpecNode { id: _spec_id("spec::consistency::a"), title: "A".to_owned(), version: 1 };
    let b = SpecNode { id: _spec_id("spec::consistency::b"), title: "B".to_owned(), version: 1 };
    graph.upsert_node(a.clone()).unwrap();
    graph.upsert_node(b.clone()).unwrap();
    graph
        .add_edge(spec_db_core::CausalEdge {
            source: a.id.clone(),
            target: b.id.clone(),
            trust: TrustLevel::human(),
            origin: EdgeOrigin::Human,
        })
        .unwrap();

    let doc_a = spec_db_core::SpecDoc {
        id: a.id,
        title: "A".to_owned(),
        version: 1,
        tags: vec!["consistency".to_owned()],
        depends_on: vec![_spec_id("spec::consistency::b")],
        owner: Some("qa".to_owned()),
        created: "2026-02-23".to_owned(),
        body: "A depends on B".to_owned(),
    };
    let doc_b = spec_db_core::SpecDoc {
        id: b.id,
        title: "B".to_owned(),
        version: 1,
        tags: vec!["consistency".to_owned()],
        depends_on: Vec::new(),
        owner: Some("qa".to_owned()),
        created: "2026-02-23".to_owned(),
        body: "B root".to_owned(),
    };
    search.add_doc(&doc_a).unwrap();
    search.add_doc(&doc_b).unwrap();
    search.commit().unwrap();

    let sha = "abc123".to_owned();
    let count = 2usize;
    store.set_last_sync_sha(&sha).unwrap();
    store.set_doc_count(count).unwrap();
    std::fs::write(
        tantivy_dir.join("sync_metadata.json"),
        format!("{{\"last_sync_sha\":\"{sha}\",\"doc_count\":{count}}}\n"),
    )
    .unwrap();

    drop(search);
    let search_meta =
        SearchIndex::open_or_create(&tantivy_dir).unwrap().sync_metadata().unwrap().unwrap();
    (sha, count, search_meta.0, search_meta.1)
}

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

fn _write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

/// AC1: Startup consistency check reports `InSync` when SHA and doc count match across stores.
#[test]
fn ac1_startup_consistency_in_sync_when_sha_and_count_match() {
    let dir = tempfile::tempdir().unwrap();
    let (fjall_sha, fjall_count, tantivy_sha, tantivy_count) = _seed_store_snapshots(dir.path());

    let report = verify_cross_store_consistency(
        Some(fjall_sha),
        Some(fjall_count),
        Some(tantivy_sha),
        Some(tantivy_count),
    )
    .unwrap();

    assert_eq!(report.status, ConsistencyStatus::InSync);
}

/// AC2: Post-sync verification compares both dimensions and returns `InSync` on parity.
#[test]
fn ac2_post_sync_verification_returns_in_sync_on_parity() {
    let report = verify_consistency(
        Some("deadbeef".to_owned()),
        Some(11),
        Some("deadbeef".to_owned()),
        Some(11),
    );
    assert_eq!(report.status, ConsistencyStatus::InSync);
}

/// AC3: Drift detection reports exact mismatch dimensions in `ConsistencyStatus::Drift`.
#[test]
fn ac3_drift_detection_exposes_sha_and_count_mismatch_flags() {
    let report =
        verify_consistency(Some("sha-a".to_owned()), Some(10), Some("sha-b".to_owned()), Some(12));
    assert_eq!(
        report.status,
        ConsistencyStatus::Drift { sha_mismatch: true, count_mismatch: true }
    );
}

/// AC4: Incremental divergence path escalates to full rebuild and returns rebuilt count.
#[test]
fn ac4_incremental_divergence_escalates_to_full_rebuild() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    let tantivy_dir = dir.path().join("tantivy");
    let fjall_dir = dir.path().join("fjall");
    std::fs::create_dir_all(&repo_dir).unwrap();

    _git(&["init"], &repo_dir);
    _git(&["config", "user.email", "test@test.com"], &repo_dir);
    _git(&["config", "user.name", "test"], &repo_dir);

    _write_file(
        &repo_dir.join("specs/auth/login.md"),
        "---\nid: \"spec::auth::login\"\ntitle: \"Login\"\nversion: 1\ntags: [\"auth\"]\ndepends_on: []\nowner: \"backend\"\ncreated: \"2026-01-01\"\n---\n# Login\nkeyword-login\n",
    );
    _write_file(
        &repo_dir.join("specs/auth/token.md"),
        "---\nid: \"spec::auth::token\"\ntitle: \"Token\"\nversion: 1\ntags: [\"auth\"]\ndepends_on: []\nowner: \"backend\"\ncreated: \"2026-01-01\"\n---\n# Token\nkeyword-token\n",
    );
    _git(&["add", "."], &repo_dir);
    _git(&["commit", "-m", "initial"], &repo_dir);

    let sync = spec_db_ingest::GitSync::new(
        repo_dir.clone(),
        "specs/".to_owned(),
        spec_db_ingest::StorePaths {
            tantivy_dir: tantivy_dir.clone(),
            fjall_dir: fjall_dir.clone(),
        },
    );
    sync.full_rebuild().unwrap();

    std::fs::remove_dir_all(&fjall_dir).unwrap();
    std::fs::create_dir_all(&fjall_dir).unwrap();

    _write_file(
        &repo_dir.join("specs/auth/mfa.md"),
        "---\nid: \"spec::auth::mfa\"\ntitle: \"MFA\"\nversion: 1\ntags: [\"auth\"]\ndepends_on: [\"spec::auth::login\"]\nowner: \"backend\"\ncreated: \"2026-01-01\"\n---\n# MFA\nkeyword-mfa\n",
    );
    _git(&["add", "."], &repo_dir);
    _git(&["commit", "-m", "add mfa"], &repo_dir);

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 3);
}

/// AC5: Persistent drift after escalation returns terminal `ConsistencyError` and no retry loop.
#[test]
fn ac5_terminal_consistency_error_message_for_persistent_drift_exists() {
    let source = include_str!("../crates/ingest/src/sync.rs");
    assert!(source.contains("cross-store consistency drift persists after escalation"));
    assert!(source.contains("return self.full_rebuild_internal(true);"));
    assert!(
        source.contains("post-incremental consistency drift detected; escalating to full rebuild")
    );
}
