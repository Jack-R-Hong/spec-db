//! Acceptance tests for Story 4.2: Incremental Sync via Git Diff

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use spec_db_causal::FjallStore;
use spec_db_core::{SearchEngine, SpecId};
use spec_db_ingest::{GitSync, StorePaths};
use spec_db_search::SearchIndex;

fn _assert_success(status: std::process::ExitStatus, action: &str) {
    assert!(status.success(), "{action} failed with status {status}");
}

fn _spec_markdown(id: &str, title: &str, body: &str, depends_on: &[&str]) -> String {
    let deps = if depends_on.is_empty() {
        "[]".to_owned()
    } else {
        let joined =
            depends_on.iter().map(|dep| format!("\"{dep}\"")).collect::<Vec<_>>().join(", ");
        format!("[{joined}]")
    };

    format!(
        "---\nid: \"{id}\"\ntitle: \"{title}\"\nversion: 1\ntags: [\"acceptance\"]\ndepends_on: {deps}\nowner: \"qa\"\ncreated: \"2026-02-23\"\n---\n# {title}\n{body}\n"
    )
}

fn _run_git(repo_path: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .env("GIT_AUTHOR_NAME", "lattice-tests")
        .env("GIT_AUTHOR_EMAIL", "tests@lattice.local")
        .env("GIT_COMMITTER_NAME", "lattice-tests")
        .env("GIT_COMMITTER_EMAIL", "tests@lattice.local")
        .status()
        .unwrap();
    _assert_success(status, &format!("git {}", args.join(" ")));
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

fn _head_sha(repo_path: &Path) -> String {
    let output =
        Command::new("git").args(["rev-parse", "HEAD"]).current_dir(repo_path).output().unwrap();
    _assert_success(output.status, "git rev-parse HEAD");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn _store_paths(base: &Path) -> StorePaths {
    StorePaths { tantivy_dir: base.join("tantivy"), fjall_dir: base.join("fjall") }
}

fn _setup_sync() -> (tempfile::TempDir, PathBuf, StorePaths, GitSync) {
    let dir = tempfile::tempdir().unwrap();
    let repo_path = dir.path().join("repo");
    fs::create_dir_all(&repo_path).unwrap();
    _run_git(&repo_path, &["init"]);

    let initial_specs = vec![
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
    _commit_specs(&repo_path, &initial_specs, "initial specs");

    let paths = _store_paths(dir.path());
    let sync = GitSync::new(repo_path.clone(), "specs/".to_owned(), paths.clone());
    (dir, repo_path, paths, sync)
}

/// AC1: Incremental sync processes only changed files from git diff.
#[test]
fn ac1_incremental_sync_only_processes_changed_files() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let first = sync.full_rebuild().unwrap();
    assert_eq!(first.specs_ingested, 3);

    let changed_specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown(
                "spec::auth::login",
                "Login",
                "credential-check keyword-login-updated",
                &[],
            ),
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
    _commit_specs(&repo_path, &changed_specs, "modify only login");

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 1);
    assert_eq!(report.head_sha, _head_sha(&repo_path));

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert_eq!(search.search("keyword-login-updated", 10).unwrap().len(), 1);
}

/// AC2: Renamed spec files are re-indexed without duplicate spec IDs.
#[test]
fn ac2_incremental_sync_handles_renames_without_duplication() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let _ = sync.full_rebuild().unwrap();

    fs::rename(repo_path.join("specs/auth/login.md"), repo_path.join("specs/auth/login-v2.md"))
        .unwrap();
    _run_git(&repo_path, &["add", "-A"]);
    _run_git(&repo_path, &["commit", "-m", "rename login file"]);

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 1);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    let login_id = SpecId::try_new("spec::auth::login").unwrap();
    let login_count =
        fjall.iter_nodes().unwrap().into_iter().filter(|node| node.id == login_id).count();
    assert_eq!(login_count, 1);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert_eq!(search.search("keyword-login", 10).unwrap().len(), 1);
}

/// AC3: Deleted specs are removed from both stores during incremental sync.
#[test]
fn ac3_incremental_sync_removes_deleted_specs_from_both_stores() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let _ = sync.full_rebuild().unwrap();

    let retained_specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown("spec::auth::login", "Login", "credential-check keyword-login", &[]),
        ),
        (
            "specs/auth/token.md",
            _spec_markdown("spec::auth::token", "Token", "issue-token keyword-token", &[]),
        ),
    ];
    _commit_specs(&repo_path, &retained_specs, "delete session spec");

    let report = sync.incremental_sync().unwrap();
    assert_eq!(report.specs_ingested, 0);

    let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
    assert!(search.search("keyword-session", 10).unwrap().is_empty());

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    let session_id = SpecId::try_new("spec::auth::session").unwrap();
    assert!(fjall.get_node(&session_id).unwrap().is_none());
}

/// AC4: Incremental sync on few changes among 100+ specs finishes under two seconds.
#[test]
#[ignore = "expensive: performance timing for incremental sync with 100+ corpus"]
fn ac4_incremental_sync_100_plus_specs_under_two_seconds_for_small_delta() {
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
                &format!("perf payload keyword-{i}"),
                &[],
            ),
        ));
    }
    let staged: Vec<(&str, String)> =
        specs.iter().map(|(path, content)| (path.as_str(), content.clone())).collect();
    _commit_specs(&repo_path, &staged, "seed perf corpus");

    let paths = _store_paths(dir.path());
    let sync = GitSync::new(repo_path.clone(), "specs/".to_owned(), paths.clone());
    let _ = sync.full_rebuild().unwrap();

    specs[42].1 =
        _spec_markdown("spec::perf::spec-42", "Spec 42", "perf payload keyword-42-updated", &[]);
    let restaged: Vec<(&str, String)> =
        specs.iter().map(|(path, content)| (path.as_str(), content.clone())).collect();
    _commit_specs(&repo_path, &restaged, "modify single perf spec");

    let start = Instant::now();
    let report = sync.incremental_sync().unwrap();
    let elapsed = start.elapsed();

    assert_eq!(report.specs_ingested, 1);
    assert!(elapsed < Duration::from_secs(2), "incremental sync took {elapsed:?}");
}

/// AC5: Incremental sync updates metadata and escalates to full rebuild on divergence.
#[test]
fn ac5_incremental_sync_updates_sha_and_escalates_on_divergence() {
    let (_dir, repo_path, paths, sync) = _setup_sync();

    let baseline = sync.full_rebuild().unwrap();

    let changed_specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown("spec::auth::login", "Login", "credential-check keyword-login-v2", &[]),
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
    _commit_specs(&repo_path, &changed_specs, "update login for metadata check");

    let incremental = sync.incremental_sync().unwrap();
    assert_ne!(incremental.head_sha, baseline.head_sha);

    let fjall = FjallStore::open(&paths.fjall_dir).unwrap();
    let node_count = fjall.iter_nodes().unwrap().len();
    assert_eq!(fjall.last_sync_sha().unwrap().as_deref(), Some(incremental.head_sha.as_str()));
    assert_eq!(fjall.doc_count().unwrap(), Some(node_count));

    {
        let search = SearchIndex::open_or_create(&paths.tantivy_dir).unwrap();
        let metadata = search.sync_metadata().unwrap().unwrap();
        assert_eq!(metadata.0, incremental.head_sha);
        assert_eq!(metadata.1, node_count);
    }

    fs::remove_dir_all(&paths.fjall_dir).unwrap();
    fs::create_dir_all(&paths.fjall_dir).unwrap();

    let diverged_specs = vec![
        (
            "specs/auth/login.md",
            _spec_markdown("spec::auth::login", "Login", "credential-check keyword-login-v2", &[]),
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
    _commit_specs(&repo_path, &diverged_specs, "introduce divergence then recover");

    let escalated = sync.incremental_sync().unwrap();
    assert_eq!(escalated.specs_ingested, 4);
    assert_eq!(escalated.head_sha, _head_sha(&repo_path));

    let repaired = FjallStore::open(&paths.fjall_dir).unwrap();
    assert_eq!(repaired.doc_count().unwrap(), Some(4));
}
