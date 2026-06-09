use crate::common::{DirGuard, cwd_lock, git_add_and_commit, setup_git_repo};
use changeguard::commands::federate::execute_federate_scan;
use changeguard::commands::init::execute_init;
use changeguard::commands::scan::execute_scan;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_federate_scan_no_remotes() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    // Stage and commit the .gitignore that init created, so scan has clean state
    git_add_and_commit(root, "after init");

    // Need a scan with impact first to produce the packet that federated_scan reads
    execute_scan(true, false, false, None).unwrap();

    let result = execute_federate_scan();
    // In a temp repo with no remotes, it should still succeed (no remotes to scan)
    assert!(result.is_ok());
}
