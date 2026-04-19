use camino::Utf8Path;
use changeguard::commands::init::execute_init;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock, setup_git_repo};

#[test]
fn test_init_command_integration() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    setup_git_repo(tmp.path());

    let _guard = DirGuard::from_utf8(root);

    let result = execute_init(false);
    assert!(result.is_ok());

    let cg_dir = root.join(".changeguard");
    assert!(cg_dir.exists());
    assert!(cg_dir.join("config.toml").exists());
    assert!(cg_dir.join("rules.toml").exists());
    assert!(cg_dir.join("logs").exists());

    let gitignore = root.join(".gitignore");
    assert!(gitignore.exists());
    let gitignore_content = fs::read_to_string(gitignore).unwrap();
    assert!(gitignore_content.contains(".changeguard/"));
}

#[test]
fn test_init_no_gitignore() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    setup_git_repo(tmp.path());

    let _guard = DirGuard::from_utf8(root);

    let result = execute_init(true);
    assert!(result.is_ok());

    let cg_dir = root.join(".changeguard");
    assert!(cg_dir.exists());
    assert!(!root.join(".gitignore").exists());
}
