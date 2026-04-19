use camino::Utf8Path;
use changeguard::commands::watch::execute_watch;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock, setup_git_repo};

#[test]
fn test_watch_invalid_config_fails_visibly() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    setup_git_repo(tmp.path());
    let _guard = DirGuard::from_utf8(root);

    let state_dir = root.join(".changeguard");
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(state_dir.join("config.toml"), "[watch]\ndebounce_ms = 0\n").unwrap();

    let err = execute_watch(100).unwrap_err();
    assert!(format!("{err:?}").contains("debounce_ms"));
}
