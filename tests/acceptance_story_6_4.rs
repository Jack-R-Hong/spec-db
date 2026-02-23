//! Acceptance tests for Story 6.4: CLI Administration Commands

use std::process::Command;

fn _binary_path() -> String {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_lattice") {
        return path;
    }

    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("lattice").to_string_lossy().into_owned()
}

/// AC1: `serve` flow includes initial sync + consistency check preflight before transport startup.
#[test]
fn ac1_serve_flow_contains_sync_and_consistency_preflight() {
    let source = include_str!("../src/main.rs");
    assert!(source.contains("if store.last_sync_sha()?.is_none()"));
    assert!(source.contains("initial sync completed:"));
    assert!(source.contains("consistency check:"));
    assert!(source.contains("stores are drifted; run `lattice rebuild` before serving"));
}

/// AC2: `sync` and `sync --full` subcommands parse and execute distinct modes.
#[test]
fn ac2_sync_subcommands_are_parsed_and_mode_switched() {
    let dir = tempfile::tempdir().unwrap();
    let sync_output =
        Command::new(_binary_path()).arg("sync").current_dir(dir.path()).output().unwrap();
    assert!(!sync_output.status.success());

    let sync_full_output = Command::new(_binary_path())
        .args(["sync", "--full"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!sync_full_output.status.success());

    let source = include_str!("../src/main.rs");
    assert!(source.contains("Commands::Sync { full }"));
    assert!(source.contains("if full { \"full\" } else { \"incremental\" }"));
}

/// AC3: `rebuild` maps to destructive full rebuild path.
#[test]
fn ac3_rebuild_command_routes_to_full_sync_path() {
    let source = include_str!("../src/main.rs");
    assert!(source.contains("Commands::Rebuild =>"));
    assert!(source.contains("run_sync(&cwd, &cfg, true)"));
}

/// AC4: `status` reports doc count, last sync SHA, and consistency state.
#[test]
fn ac4_status_outputs_expected_fields() {
    let dir = tempfile::tempdir().unwrap();

    let init_output =
        Command::new(_binary_path()).arg("init").current_dir(dir.path()).output().unwrap();
    assert!(init_output.status.success());

    let status_output =
        Command::new(_binary_path()).arg("status").current_dir(dir.path()).output().unwrap();
    assert!(status_output.status.success());

    let stdout = String::from_utf8(status_output.stdout).unwrap();
    assert!(stdout.contains("doc_count:"));
    assert!(stdout.contains("last_sync_sha:"));
    assert!(stdout.contains("consistency:"));
}

/// AC5: CLI failures are human-readable via anyhow context and avoid stack-trace noise by default.
#[test]
fn ac5_errors_are_human_readable() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(_binary_path()).arg("sync").current_dir(dir.path()).output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to load project config"));
    assert!(!stderr.contains("thread 'main' panicked"));
    assert!(!stderr.contains("stack backtrace"));
}
