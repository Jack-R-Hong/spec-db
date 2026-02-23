use std::process::Command;

fn binary_path() -> String {
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

#[test]
fn init_creates_structure() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(binary_path()).arg("init").current_dir(dir.path()).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Initialized lattice project:"));
    assert!(stdout.contains("lattice sync"));
    assert!(stdout.contains("lattice serve"));
    assert!(stdout.contains("lattice status"));

    assert!(dir.path().join(".lattice/config.yaml").exists());
    assert!(dir.path().join("specs/example/hello-world.md").exists());
    assert!(dir.path().join("specs/example/getting-started.md").exists());
    assert!(dir.path().join("data/tantivy").exists());
    assert!(dir.path().join("data/fjall").exists());
}

#[test]
fn init_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let first = Command::new(binary_path()).arg("init").current_dir(dir.path()).output().unwrap();
    assert!(first.status.success());

    let config_path = dir.path().join(".lattice/config.yaml");
    let before = std::fs::read_to_string(&config_path).unwrap();

    let second = Command::new(binary_path()).arg("init").current_dir(dir.path()).output().unwrap();
    assert!(second.status.success());

    let stdout = String::from_utf8(second.stdout).unwrap();
    assert!(stdout.contains("Warning: .lattice/config.yaml already exists."));

    let after = std::fs::read_to_string(&config_path).unwrap();
    assert_eq!(before, after);
}

#[test]
fn sync_command_is_parsed() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(binary_path()).arg("sync").current_dir(dir.path()).output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to load project config"));
}

#[test]
fn sync_full_command_is_parsed() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(binary_path())
        .arg("sync")
        .arg("--full")
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to load project config"));
}

#[test]
fn rebuild_command_is_parsed() {
    let dir = tempfile::tempdir().unwrap();
    let output =
        Command::new(binary_path()).arg("rebuild").current_dir(dir.path()).output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to load project config"));
}

#[test]
fn status_command_is_parsed() {
    let dir = tempfile::tempdir().unwrap();
    let output =
        Command::new(binary_path()).arg("status").current_dir(dir.path()).output().unwrap();
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to load project config"));
}
