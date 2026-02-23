//! Acceptance tests for Story 4.1: Full Rebuild from Git Tree Walk

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use spec_db_causal::FjallStore;
use spec_db_core::SearchEngine;
use spec_db_ingest::{GitSync, StorePaths};
use spec_db_search::SearchIndex;

fn _assert_success(status: std::process::ExitStatus, action: &str) {
    assert!(status.success(), "{action} failed with status {status}");
}

fn _assert_no_swap_artifacts(base: &Path) {
    assert!(!base.join("tantivy_rebuild_tmp").exists());
    assert!(!base.join("fjall_rebuild_tmp").exists());
    assert!(!base.join("tantivy_old").exists());
    assert!(!base.join("fjall_old").exists());
}

fn _spec_markdown(id: &str, title: &str, body: &str, depends_on: &[&str]) -> String {
    let deps = if depends_on.is_empty() {
        "[]".to_owned()
    } else {
        let parts =
            depends_on.iter().map(|dep| format!("\"{dep}\"")).collect::<Vec<_>>().join(", ");
        format!("[{parts}]")
    };

    format!(
        "---\nid: \"{id}\"\ntitle: \"{title}\"\nversion: 1\ntags: [\"acceptance\"]\ndepends_on: {deps}\nowner: \"qa\"\ncreated: \"2026-02-23\"\n---\n# {title}\n{body}\n"
    )
}

fn _run_git(repo_path: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .env("GIT_AUTHOR_NAME", "spec-db-tests")
        .env("GIT_AUTHOR_EMAIL", "tests@spec-db.local")
        .env("GIT_COMMITTER_NAME", "spec-db-tests")
        .env("GIT_COMMITTER_EMAIL", "tests@spec-db.local")
        .status()
        .unwrap();
    _assert_success(status, &format!("git {}", args.join(" ")));
}

fn _head_sha(repo_path: &Path) -> String {
    let output =
        Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repo_path).output().unwrap();
    _assert_success(output.status, "git rev-parse HEAD");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn _commit_specs(repo_path: &Path, specs: &[(&str, String)], message: &str) {
    let specs_dir = repo_path.join("specs");
    if specs_dir.exists() {
        fs::remove_dir_all(&specs_dir).unwrap();
    }
    fs::create_dir_all(&specs_dir).unwrap();

    for (relative_path, content) in specs {
        let file_path = repo_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    _run_git(repo_path, &["add", "-A"]);
    _run_git(repo_path, &["commit", "-m", message]);
}

fn _seed_repo(repo_path: &Path) {
    _run_git(repo_path, &["init"]);

    let specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown("spec::auth::login", "Login", "credential-check keyword-login", &[]),
        ),
        (
            "specs/auth/token.md",
            _spec_markdown("spec::auth::token", "Token", "issue-token keyword-token", &[]),
        ),
        (
            "specs/auth/session.md",
            _spec_markdown(
                "spec::auth::session",
                "Session",
                "session-tracking keyword-session",
                &["spec::auth::token"],
            ),
        ),
    ];
    _commit_specs(repo_path, &specs, "initial specs");
}

fn _store_paths(base: &Path) -> StorePaths {
    StorePaths { tantivy_dir: base.join("tantivy"), fjall_dir: base.join("fjall") }
}

fn _setup_sync() -> (tempfile::TempDir, PathBuf, StorePaths, GitSync) {
    let dir = tempfile::tempdir().unwrap();
    let repo_path = dir.path().join("repo");
    fs::create_dir_all(&repo_path).unwrap();
    _seed_repo(&repo_path);
    let paths = _store_paths(dir.path());
    let sync = GitSync::new(repo_path.clone(), "specs/".to_owned(), paths.clone());
    (dir, repo_path, paths, sync)
}

/// AC1: Full rebuild walks the git tree and is idempotent for unchanged HEAD.
#[test]
fn ac1_full_rebuild_walks_git_tree_and_is_idempotent() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let first = sync.full_rebuild().unwrap();
    let second = sync.full_rebuild().unwrap();

    assert_eq!(first.specs_ingested, 3);
    assert_eq!(second.specs_ingested, 3);
    assert_eq!(first.head_sha, _head_sha(&repo_path));
    assert_eq!(first.head_sha, second.head_sha);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert_eq!(search.doc_count().unwrap(), 3);
}

/// AC2: Rebuild outputs are staged and swapped atomically with no temp/backup residue.
#[test]
fn ac2_rebuild_uses_atomic_swap_without_leftover_artifacts() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let _first = sync.full_rebuild().unwrap();
    _assert_no_swap_artifacts(paths.tantivy_dir.parent().unwrap());

    let updated_specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown("spec::auth::login", "Login", "credential-check keyword-login-v2", &[]),
        ),
        (
            "specs/auth/token.md",
            _spec_markdown("spec::auth::token", "Token", "issue-token keyword-token", &[]),
        ),
        (
            "specs/auth/mfa.md",
            _spec_markdown(
                "spec::auth::mfa",
                "MFA",
                "one-time-code keyword-mfa",
                &["spec::auth::login"],
            ),
        ),
    ];
    _commit_specs(&repo_path, &updated_specs, "replace session with mfa");

    let _second = sync.full_rebuild().unwrap();
    _assert_no_swap_artifacts(paths.tantivy_dir.parent().unwrap());
    assert!(paths.tantivy_dir.exists());
    assert!(paths.fjall_dir.exists());
}

/// AC3: Full rebuild on 100+ specs finishes under five seconds.
#[test]
#[ignore = "expensive: performance timing for 100+ full rebuild"]
fn ac3_full_rebuild_100_plus_specs_under_five_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let repo_path = dir.path().join("repo");
    fs::create_dir_all(&repo_path).unwrap();
    _run_git(&repo_path, &["init"]);

    let mut specs = Vec::new();
    for i in 0..130 {
        specs.push((
            format!("specs/perf/spec-{i}.md"),
            _spec_markdown(
                &format!("spec::perf::spec-{i}"),
                &format!("Spec {i}"),
                &format!("perf keyword-{i}"),
                &[],
            ),
        ));
    }
    let staged: Vec<(&str, String)> =
        specs.iter().map(|(path, content)| (path.as_str(), content.clone())).collect();
    _commit_specs(&repo_path, &staged, "seed perf corpus");

    let paths = _store_paths(dir.path());
    let sync = GitSync::new(repo_path, "specs/".to_owned(), paths);

    let start = Instant::now();
    let report = sync.full_rebuild().unwrap();
    let elapsed = start.elapsed();

    assert_eq!(report.specs_ingested, 130);
    assert!(elapsed < Duration::from_secs(5), "full rebuild took {elapsed:?}");
}

/// AC4: Both stores persist current git SHA and document counts after rebuild.
#[test]
fn ac4_full_rebuild_persists_sha_and_doc_count_in_both_stores() {
    let (_dir, _repo_path, paths, sync) = _setup_sync();

    let report = sync.full_rebuild().unwrap();

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(fjall.last_sync_sha().unwrap().as_deref(), Some(report.head_sha.as_str()));
    assert_eq!(fjall.doc_count().unwrap(), Some(report.specs_ingested));

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    let metadata = search.sync_metadata().unwrap().unwrap();
    assert_eq!(metadata.0, report.head_sha);
    assert_eq!(metadata.1, report.specs_ingested);
}

/// AC5: Full rebuild fully replaces stale data with no stale spec remnants.
#[test]
fn ac5_full_rebuild_replaces_stale_data_completely() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let first = sync.full_rebuild().unwrap();
    assert_eq!(first.specs_ingested, 3);

    let replacement_specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown("spec::auth::login", "Login", "credential-check keyword-login", &[]),
        ),
        (
            "specs/auth/token.md",
            _spec_markdown("spec::auth::token", "Token", "issue-token keyword-token", &[]),
        ),
        (
            "specs/auth/mfa.md",
            _spec_markdown(
                "spec::auth::mfa",
                "MFA",
                "one-time-code keyword-mfa",
                &["spec::auth::login"],
            ),
        ),
    ];
    _commit_specs(&repo_path, &replacement_specs, "replace stale corpus");

    let second = sync.full_rebuild().unwrap();
    assert_eq!(second.specs_ingested, 3);
    assert_ne!(first.head_sha, second.head_sha);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert!(search.search("keyword-session", 10).unwrap().is_empty());
    assert_eq!(search.search("keyword-mfa", 10).unwrap().len(), 1);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    let node_ids =
        fjall.iter_nodes().unwrap().into_iter().map(|node| node.id.to_string()).collect::<Vec<_>>();
    assert!(node_ids.iter().all(|id| id != "spec::auth::session"));
    assert!(node_ids.iter().any(|id| id == "spec::auth::mfa"));
}
