use crate::common::{DirGuard, cwd_lock, git_add_and_commit, setup_git_repo};
use changeguard::commands::init::execute_init;
use changeguard::commands::ledger_audit::execute_ledger_audit;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_audit_basic() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    // Audit with limit 5, no entity filter, not json, no unaudited
    let result = execute_ledger_audit(None, false, 5, 0, false);
    assert!(result.is_ok());
}
