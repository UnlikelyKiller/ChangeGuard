use std::fs;
use std::process::Command;
use tempfile::tempdir;

use crate::common::{DirGuard, cwd_lock, setup_git_repo};

#[test]
fn test_search_fuzzy_fallback_and_hint() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    setup_git_repo(root);
    let _guard = DirGuard::new(root);

    use crate::common::git_add_and_commit;
    fs::write(root.join("test_file.rs"), "pub fn execute_scan_impact() {}").unwrap();
    git_add_and_commit(root, "test_file.rs");

    let changeguard_bin = env!("CARGO_BIN_EXE_changeguard");

    // 1. Fuzzy match success
    let output = Command::new(changeguard_bin)
        .args(["search", "excute", "--index"])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Falling back to fuzzy search"),
        "Expected fallback to fuzzy search: {}",
        stdout
    );
    assert!(stdout.contains("Fuzzy Search Results:"));
    assert!(stdout.contains("test_file.rs"));

    // 1.5 JSON Output test
    let output_json = Command::new(changeguard_bin)
        .args(["search", "excute", "--index", "--json"])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout_json = String::from_utf8_lossy(&output_json.stdout);
    assert!(
        stdout_json.contains(r#"record_kind":"fuzzy_match"#),
        "Expected JSON fallback record: {}",
        stdout_json
    );

    // 2. Semantic Handoff Hint
    let output2 = Command::new(changeguard_bin)
        .args(["search", "nonexistent_symbol_12345"])
        .current_dir(root)
        .output()
        .unwrap();

    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(stdout2.contains("Falling back to fuzzy search"));
    assert!(stdout2.contains("No exact symbols found."));
    assert!(stdout2.contains("Alternatively, try semantic search instead:"));
}
