use crate::common::{DirGuard, cwd_lock, git_add_and_commit, setup_git_repo};
use changeguard::commands::config::{
    execute_config_schema, execute_config_verify, execute_config_view,
};
use changeguard::commands::init::execute_init;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_verify_default() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    // execute_config_verify uses Layout::new(cwd), so cwd must be the repo root
    let result = execute_config_verify(false, None, false);
    assert!(result.is_ok());
}

#[test]
fn test_config_view_shows_values() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    let result = execute_config_view(false, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_config_schema_output() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);
    fs::write(root.join("dummy.txt"), "content").unwrap();
    git_add_and_commit(root, "initial");

    let _guard = DirGuard::new(root);
    execute_init(false).unwrap();

    let result = execute_config_schema(false);
    assert!(result.is_ok());
}
