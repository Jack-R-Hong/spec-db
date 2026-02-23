//! Acceptance tests for Story 6.1: Project Initialization & Configuration

use std::process::Command;

use spec_db_core::{SpecDbError, load_config};

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

/// AC1: `lattice init` scaffolds specs/config/default data dirs and prints next steps.
#[test]
fn ac1_init_creates_project_structure_and_next_steps() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(_binary_path()).arg("init").current_dir(dir.path()).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Initialized lattice project:"));
    assert!(stdout.contains("Next steps:"));
    assert!(stdout.contains("lattice sync"));
    assert!(stdout.contains("lattice serve"));
    assert!(stdout.contains("lattice status"));

    assert!(dir.path().join("specs/example/hello-world.md").exists());
    assert!(dir.path().join("specs/example/getting-started.md").exists());
    assert!(dir.path().join(".lattice/config.yaml").exists());
    assert!(dir.path().join("data/tantivy").exists());
    assert!(dir.path().join("data/fjall").exists());
}

/// AC2: Config loading uses defaults for missing optional fields and errors on missing required fields.
#[test]
fn ac2_config_loader_defaults_and_required_field_validation() {
    let dir = tempfile::tempdir().unwrap();

    let partial_config = dir.path().join("partial.yaml");
    std::fs::write(&partial_config, "transport:\n  stdio: false\n").unwrap();
    let loaded = load_config(&partial_config).unwrap();
    assert_eq!(loaded.specs_dir, "specs");
    assert_eq!(loaded.data_dir, "data");
    assert!(!loaded.transport.stdio);
    assert!(loaded.transport.http.is_none());
    assert!(!loaded.telemetry.enabled);
    assert_eq!(loaded.telemetry.protocol, "grpc");

    let invalid_required = dir.path().join("invalid-required.yaml");
    std::fs::write(&invalid_required, "transport:\n  http:\n    host: 127.0.0.1\n").unwrap();
    let err = load_config(&invalid_required).unwrap_err();
    match err {
        SpecDbError::ConfigError(message) => {
            assert!(message.contains("invalid config YAML"));
            assert!(message.contains("port"));
        }
        other => panic!("expected ConfigError, got {other:?}"),
    }
}

/// AC3: Re-running `lattice init` warns and preserves existing config file content.
#[test]
fn ac3_init_when_config_exists_warns_and_does_not_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let first = Command::new(_binary_path()).arg("init").current_dir(dir.path()).output().unwrap();
    assert!(first.status.success());

    let config_path = dir.path().join(".lattice/config.yaml");
    let before = std::fs::read_to_string(&config_path).unwrap();

    let second = Command::new(_binary_path()).arg("init").current_dir(dir.path()).output().unwrap();
    assert!(second.status.success());

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("Warning: .lattice/config.yaml already exists."));

    let after = std::fs::read_to_string(&config_path).unwrap();
    assert_eq!(before, after);
}
