use crate::common::{cwd_lock, setup_git_repo};
use camino::Utf8Path;
use changeguard::state::storage::StorageManager;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_data_models_impact_binary_output() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let root_utf8 = Utf8Path::from_path(root).unwrap();

    setup_git_repo(root);
    fs::write(root.join("models.rs"), "struct User;").unwrap();

    // Use the binary to capture output
    let changeguard_bin = env!("CARGO_BIN_EXE_changeguard");

    Command::new(changeguard_bin)
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();

    Command::new(changeguard_bin)
        .arg("index")
        .current_dir(root)
        .output()
        .unwrap();

    // We need to commit so it's not "changed" (clean tree)
    Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(root)
        .output()
        .unwrap();

    // Manually insert model because detector is picky in this env
    let storage = StorageManager::open_read_only(root_utf8).unwrap();
    let conn = storage.get_connection();
    let file_id: i64 = conn
        .query_row(
            "SELECT id FROM project_files WHERE file_path = 'models.rs'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(1);
    conn.execute(
        "INSERT INTO data_models (model_name, model_file_id, language, model_kind, confidence, evidence, last_indexed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params!["User", file_id, "Rust", "STRUCT", 1.0_f64, "manual", "2026-05-01T00:00:00Z"],
    ).unwrap();

    let output = Command::new(changeguard_bin)
        .args(["data-models", "impact", "--changed"])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // EXPECTED BEHAVIOR: contains "No changed data models found." AND NO empty table/header
    assert!(
        stdout.contains("No changed data models found."),
        "Output was: {}",
        stdout
    );
    assert!(
        !stdout.contains("No data models indexed."),
        "Should not contain misleading help message"
    );
    assert!(
        !stdout.contains("Name | File"),
        "Should not contain table header"
    );
    assert!(output.status.success());
}

#[test]
fn test_data_models_impact_binary_output_no_models_at_all() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();

    let changeguard_bin = env!("CARGO_BIN_EXE_changeguard");

    Command::new(changeguard_bin)
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();

    let output = Command::new(changeguard_bin)
        .args(["data-models", "impact", "--changed"])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No data models indexed."),
        "Output was: {}",
        stdout
    );
    assert!(
        !stdout.contains("No changed data models found."),
        "Output was: {}",
        stdout
    );
    assert!(
        !stdout.contains("Name | File"),
        "Should not contain table header"
    );
    assert!(output.status.success());
}
