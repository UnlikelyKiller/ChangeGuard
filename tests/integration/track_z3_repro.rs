use crate::common::{cwd_lock, setup_git_repo};
use camino::Utf8Path;
use changeguard::state::storage::StorageManager;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_config_diff_identifies_references() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let root_utf8 = Utf8Path::from_path(root).unwrap();

    setup_git_repo(root);
    fs::write(root.join(".env.example"), "MY_VAR=default").unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/main.rs"),
        r#"fn main() { let _ = std::env::var("MY_VAR"); }"#,
    )
    .unwrap();

    let changeguard_bin = env!("CARGO_BIN_EXE_changeguard");

    // Init and Index
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

    // Check config diff
    let output = Command::new(changeguard_bin)
        .args(["config", "diff"])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // CURRENT BEHAVIOR: "Declared but not referenced in code"
    // EXPECTED BEHAVIOR: No such warning for MY_VAR
    assert!(!stdout.contains("- MY_VAR"), "Output was: {}", stdout);

    // Check database directly
    let storage = StorageManager::open_read_only(root_utf8).unwrap();
    let conn = storage.get_connection();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM env_references WHERE var_name = 'MY_VAR'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(count > 0, "env_references table should contain MY_VAR");
}
