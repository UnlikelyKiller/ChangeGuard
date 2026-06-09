use crate::common::{DirGuard, cwd_lock, git_add_and_commit, setup_git_repo};
use changeguard::commands::dead_code::execute_dead_code;
use changeguard::commands::init::execute_init;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_dead_code_basic() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    // threshold 0.9, limit 50, auto_index false
    let result = execute_dead_code(0.9, 50, false);
    assert!(result.is_ok());
}
