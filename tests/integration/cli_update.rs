use crate::common::{DirGuard, cwd_lock, git_add_and_commit, setup_git_repo};
use changeguard::commands::init::execute_init;
use changeguard::commands::update::execute_update;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_update_dry_run() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    // dry_run=true with --migrate should only print what would be done
    let result = execute_update(true, false, false, false, false, true);
    assert!(result.is_ok());
}
