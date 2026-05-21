use std::fs;
use std::process::Command;

mod common;
use common::setup_git_repo;

#[test]
fn test_cozodb_hard_migration_integrity() {
    let binary_path = env!("CARGO_BIN_EXE_changeguard");
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    setup_git_repo(root);

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.rs"),
        "fn main() { println!(\"hello\"); }\n",
    )
    .unwrap();

    let init_output = Command::new(binary_path)
        .args(["init"])
        .current_dir(root)
        .output()
        .expect("Failed to execute changeguard init");

    if !init_output.status.success() {
        panic!(
            "init failed: {}",
            String::from_utf8_lossy(&init_output.stderr)
        );
    }

    // Loop 10 times: index -> hard migrate -> semantic index
    // This exercises the `robust_remove_dir` and checks for "Invalid neighbor degree" errors.
    for i in 0..10 {
        println!("Migration Loop Iteration: {}", i + 1);

        // 1. Semantic index (populates Cozo HNSW)
        let output = Command::new(binary_path)
            .args(["index", "--semantic"])
            .current_dir(root)
            .output()
            .expect("Failed to execute changeguard index --semantic");

        if !output.status.success() {
            panic!(
                "index --semantic failed on iteration {}: {}",
                i + 1,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // 2. Search to ensure HNSW is readable
        let output = Command::new(binary_path)
            .args(["search", "--semantic", "main"])
            .current_dir(root)
            .output()
            .expect("Failed to execute changeguard search --semantic");

        if !output.status.success() {
            panic!(
                "search --semantic failed on iteration {}: {}",
                i + 1,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // 3. Hard migrate (should safely shutdown and robustly remove directories)
        let output = Command::new(binary_path)
            .args(["update", "--migrate", "--force"])
            .current_dir(root)
            .output()
            .expect("Failed to execute changeguard update --migrate --force");

        if !output.status.success() {
            panic!(
                "update --migrate --force failed on iteration {}: {}",
                i + 1,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
