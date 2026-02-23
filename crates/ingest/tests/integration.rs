use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use git2::{IndexAddOption, Repository, Signature};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::{CausalGraph, EdgeOrigin, EdgeType, SearchEngine, SpecDbError, SpecId};
use spec_db_ingest::{
    ConsistencyStatus, GitSync, IngestPipeline, StorePaths, verify_cross_store_consistency,
};
use spec_db_search::SearchIndex;

const VALID_SPEC: &str = r#"---
id: "spec::auth::login"
title: "Login Flow"
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-01-01"
---
# Login Flow
User authentication via credentials.
"#;

const FORWARD_REF_SPEC: &str = r#"---
id: "spec::auth::session"
title: "Session Flow"
version: 1
tags: ["auth"]
depends_on: ["spec::auth::token"]
owner: "backend"
created: "2026-01-01"
---
# Session Flow
Session management depends on token issuance.
"#;

const FORWARD_TARGET_SPEC: &str = r#"---
id: "spec::auth::token"
title: "Token Issuance"
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-01-01"
---
# Token Issuance
Issue tokens for authenticated sessions.
"#;

const SPEC_LOGIN: &str = r#"---
id: "spec::auth::login"
title: "Login"
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-01-01"
---
# Login
credential-check keyword-login
"#;

const SPEC_TOKEN: &str = r#"---
id: "spec::auth::token"
title: "Token"
version: 1
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-01-01"
---
# Token
issue-token keyword-token
"#;

const SPEC_SESSION: &str = r#"---
id: "spec::auth::session"
title: "Session"
version: 1
tags: ["auth"]
depends_on: ["spec::auth::token"]
owner: "backend"
created: "2026-01-01"
---
# Session
session-tracking keyword-session
"#;

const SPEC_MFA: &str = r#"---
id: "spec::auth::mfa"
title: "MFA"
version: 1
tags: ["auth"]
depends_on: ["spec::auth::login"]
owner: "backend"
created: "2026-01-01"
---
# MFA
one-time-code keyword-mfa
"#;

const SPEC_LOGIN_UPDATED: &str = r#"---
id: "spec::auth::login"
title: "Login"
version: 2
tags: ["auth"]
depends_on: []
owner: "backend"
created: "2026-01-01"
---
# Login
credential-check keywordloginupdated
"#;

const SEARCH_METADATA_FILE: &str = "sync_metadata.json";

fn setup() -> (tempfile::TempDir, IngestPipeline<SearchIndex, CausalEngine>) {
    let dir = tempfile::tempdir().unwrap();
    let fjall_path = dir.path().join("fjall");
    let tantivy_path = dir.path().join("tantivy");
    let store = Arc::new(FjallStore::open(&fjall_path).unwrap());
    let engine = CausalEngine::from_store(store).unwrap();
    let search = SearchIndex::open_or_create(&tantivy_path).unwrap();
    let pipeline = IngestPipeline::new(search, engine);
    (dir, pipeline)
}

fn create_test_repo(dir: &Path) -> Repository {
    let repo = Repository::init(dir).unwrap();
    commit_specs(
        &repo,
        &[
            ("specs/auth/login.md", SPEC_LOGIN),
            ("specs/auth/token.md", SPEC_TOKEN),
            ("specs/auth/session.md", SPEC_SESSION),
        ],
        "initial",
    );
    repo
}

fn commit_specs(repo: &Repository, specs: &[(&str, &str)], message: &str) {
    let workdir = repo.workdir().unwrap();
    let specs_dir = workdir.join("specs");
    if specs_dir.exists() {
        fs::remove_dir_all(&specs_dir).unwrap();
    }
    fs::create_dir_all(&specs_dir).unwrap();

    for (path, content) in specs {
        let file_path = workdir.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    let mut index = repo.index().unwrap();
    index.clear().unwrap();
    index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = Signature::now("test", "test@test.com").unwrap();

    let parent =
        repo.head().ok().and_then(|head| head.target()).and_then(|oid| repo.find_commit(oid).ok());
    let parents: Vec<&git2::Commit<'_>> = parent.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents).unwrap();
}

fn sync_paths(base: &Path) -> StorePaths {
    StorePaths { tantivy_dir: base.join("tantivy"), fjall_dir: base.join("fjall") }
}

fn read_search_metadata(tantivy_dir: &Path) -> String {
    fs::read_to_string(tantivy_dir.join(SEARCH_METADATA_FILE)).unwrap()
}

#[test]
fn ingest_valid_spec() {
    let (_dir, mut pipeline) = setup();

    let id = pipeline.add_spec(VALID_SPEC).unwrap();

    let node = pipeline.graph().get_node(&id).unwrap();
    assert!(node.is_some());

    let hits = pipeline.search().search("credentials", 10).unwrap();
    assert_eq!(hits, vec![id]);
}

#[test]
fn ingest_duplicate_rejected() {
    let (_dir, mut pipeline) = setup();

    pipeline.add_spec(VALID_SPEC).unwrap();
    let err = pipeline.add_spec(VALID_SPEC).unwrap_err();

    match err {
        SpecDbError::IngestError(message) => assert!(message.contains("duplicate")),
        other => panic!("expected IngestError, got {other:?}"),
    }
}

#[test]
fn ingest_forward_reference() {
    let (_dir, mut pipeline) = setup();

    let source_id = pipeline.add_spec(FORWARD_REF_SPEC).unwrap();
    let target_id = SpecId::try_new("spec::auth::token").unwrap();

    let edges = pipeline.graph().edges_from(&source_id).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].target, target_id);
    assert_eq!(edges[0].edge_type, EdgeType::DependsOn);
    assert_eq!(edges[0].origin, EdgeOrigin::Human);
    assert!((edges[0].trust.value() - 1.0).abs() < f64::EPSILON);

    let target_added = pipeline.add_spec(FORWARD_TARGET_SPEC).unwrap();
    assert_eq!(target_added.as_ref(), "spec::auth::token");

    assert!(pipeline.graph().get_node(&source_id).unwrap().is_some());
    assert!(pipeline.graph().get_node(&target_added).unwrap().is_some());

    let edges_after = pipeline.graph().edges_from(&source_id).unwrap();
    assert_eq!(edges_after.len(), 1);
    assert_eq!(edges_after[0].target, target_added);
}

#[test]
fn ingest_perf_single_spec() {
    let (_dir, mut pipeline) = setup();

    let start = Instant::now();
    let _ = pipeline.add_spec(VALID_SPEC).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(100), "ingest took {elapsed:?}");
}

#[test]
fn ingest_removes_spec() {
    let (_dir, mut pipeline) = setup();

    let id = pipeline.add_spec(VALID_SPEC).unwrap();
    pipeline.remove_spec(&id).unwrap();

    assert!(pipeline.graph().get_node(&id).unwrap().is_none());
    assert!(pipeline.search().search("credentials", 10).unwrap().is_empty());
}

#[test]
fn full_rebuild_ingests_specs() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let _repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    let report = sync.full_rebuild().unwrap();
    assert_eq!(report.specs_ingested, 3);
    assert!(!report.head_sha.is_empty());

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.last_sync_sha().unwrap(), Some(report.head_sha.clone()));
    assert_eq!(fjall.doc_count().unwrap(), Some(3));

    let tantivy_meta = read_search_metadata(&paths.tantivy_dir);
    assert!(tantivy_meta.contains(&report.head_sha));
    assert!(tantivy_meta.contains("\"doc_count\":3"));
}

#[test]
fn full_rebuild_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let _repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    let first = sync.full_rebuild().unwrap();
    let second = sync.full_rebuild().unwrap();

    assert_eq!(first.specs_ingested, 3);
    assert_eq!(second.specs_ingested, 3);
    assert_eq!(first.head_sha, second.head_sha);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(3));
    assert_eq!(fjall.iter_nodes().unwrap().len(), 3);
}

#[test]
fn full_rebuild_replaces_stale() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    let first = sync.full_rebuild().unwrap();
    assert_eq!(first.specs_ingested, 3);

    commit_specs(
        &repo,
        &[
            ("specs/auth/login.md", SPEC_LOGIN),
            ("specs/auth/token.md", SPEC_TOKEN),
            ("specs/auth/mfa.md", SPEC_MFA),
        ],
        "replace stale",
    );

    let second = sync.full_rebuild().unwrap();
    assert_eq!(second.specs_ingested, 3);
    assert_ne!(first.head_sha, second.head_sha);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    let stale_hits = search.search("keyword-session", 10).unwrap();
    let fresh_hits = search.search("keyword-mfa", 10).unwrap();
    assert!(stale_hits.is_empty());
    assert_eq!(fresh_hits.len(), 1);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(3));
    let node_ids: Vec<String> =
        fjall.iter_nodes().unwrap().into_iter().map(|node| node.id.to_string()).collect();
    assert!(node_ids.iter().all(|id| id != "spec::auth::session"));
    assert!(node_ids.iter().any(|id| id == "spec::auth::mfa"));
}

#[test]
fn full_rebuild_records_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let _repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());
    let report = sync.full_rebuild().unwrap();

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.last_sync_sha().unwrap(), Some(report.head_sha.clone()));
    assert_eq!(fjall.doc_count().unwrap(), Some(report.specs_ingested));

    let metadata = read_search_metadata(&paths.tantivy_dir);
    assert!(metadata.contains(&format!("\"last_sync_sha\":\"{}\"", report.head_sha)));
    assert!(metadata.contains(&format!("\"doc_count\":{}", report.specs_ingested)));
}

#[test]
fn consistency_after_rebuild() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let _repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());
    sync.full_rebuild().unwrap();

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    let tantivy_meta = search.sync_metadata().unwrap().unwrap();

    let report = verify_cross_store_consistency(
        fjall.last_sync_sha().unwrap(),
        fjall.doc_count().unwrap(),
        Some(tantivy_meta.0),
        Some(tantivy_meta.1),
    )
    .unwrap();

    assert_eq!(report.status, ConsistencyStatus::InSync);
}

#[test]
fn incremental_sync_modified() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    let first = sync.full_rebuild().unwrap();
    commit_specs(
        &repo,
        &[
            ("specs/auth/login.md", SPEC_LOGIN_UPDATED),
            ("specs/auth/token.md", SPEC_TOKEN),
            ("specs/auth/session.md", SPEC_SESSION),
        ],
        "modify login",
    );

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 1);
    assert_ne!(report.head_sha, first.head_sha);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert_eq!(search.search("keywordloginupdated", 10).unwrap().len(), 1);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(3));
    assert_eq!(fjall.last_sync_sha().unwrap(), Some(report.head_sha));
}

#[test]
fn incremental_sync_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    sync.full_rebuild().unwrap();
    commit_specs(
        &repo,
        &[("specs/auth/login.md", SPEC_LOGIN), ("specs/auth/token.md", SPEC_TOKEN)],
        "delete session",
    );

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 0);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert!(search.search("keyword-session", 10).unwrap().is_empty());

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(2));
    let session_id = SpecId::try_new("spec::auth::session").unwrap();
    assert!(fjall.get_node(&session_id).unwrap().is_none());
}

#[test]
fn incremental_sync_added() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let repo = Repository::init(&repo_dir).unwrap();
    commit_specs(&repo, &[("specs/auth/login.md", SPEC_LOGIN)], "initial");

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    sync.full_rebuild().unwrap();
    commit_specs(
        &repo,
        &[("specs/auth/login.md", SPEC_LOGIN), ("specs/auth/token.md", SPEC_TOKEN)],
        "add token",
    );

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 1);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert_eq!(search.search("keyword-login", 10).unwrap().len(), 1);
    assert_eq!(search.search("keyword-token", 10).unwrap().len(), 1);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(2));
}

#[test]
fn incremental_sync_no_changes() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let _repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    let first = sync.full_rebuild().unwrap();
    let report = sync.incremental_sync().unwrap();

    assert_eq!(report.specs_ingested, 0);
    assert_eq!(report.head_sha, first.head_sha);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(3));
    assert_eq!(fjall.last_sync_sha().unwrap(), Some(first.head_sha));
}

#[test]
fn incremental_sync_no_prior_sha_triggers_full_rebuild() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let _repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());

    fs::create_dir_all(&paths.tantivy_dir).unwrap();
    fs::create_dir_all(&paths.fjall_dir).unwrap();

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 3);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.last_sync_sha().unwrap(), Some(report.head_sha.clone()));
    assert_eq!(fjall.doc_count().unwrap(), Some(3));
}

#[test]
fn incremental_divergence_escalates() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    fs::create_dir_all(&repo_dir).unwrap();
    let repo = create_test_repo(&repo_dir);

    let paths = sync_paths(dir.path());
    let sync = GitSync::new(repo_dir, "specs/".to_string(), paths.clone());
    sync.full_rebuild().unwrap();

    fs::remove_dir_all(&paths.fjall_dir).unwrap();
    fs::create_dir_all(&paths.fjall_dir).unwrap();

    commit_specs(
        &repo,
        &[
            ("specs/auth/login.md", SPEC_LOGIN),
            ("specs/auth/token.md", SPEC_TOKEN),
            ("specs/auth/session.md", SPEC_SESSION),
            ("specs/auth/mfa.md", SPEC_MFA),
        ],
        "divergence recovery",
    );

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 4);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.doc_count().unwrap(), Some(4));
}
