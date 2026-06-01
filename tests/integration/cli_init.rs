use camino::Utf8Path;
use changeguard::commands::init::execute_init;
use std::fs;
use tempfile::tempdir;

use crate::common::{DirGuard, cwd_lock, setup_git_repo};

struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(value) = &self.original {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}

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

    let pre_commit = fs::read_to_string(root.join(".git").join("hooks").join("pre-commit"))
        .expect("pre-commit hook should be installed");
    assert!(pre_commit.contains("# changeguard-ledger-gate"));
    assert!(pre_commit.contains("changeguard ledger status --compact --exit-code"));
    assert!(pre_commit.contains("git commit --no-verify"));

    let pre_push = fs::read_to_string(root.join(".git").join("hooks").join("pre-push"))
        .expect("pre-push hook should be installed");
    assert!(pre_push.contains("# changeguard-ledger-gate"));
    assert!(pre_push.contains("changeguard ledger status --compact --exit-code"));
    assert!(pre_push.contains("git push --no-verify"));
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
    assert!(root.join(".git").join("hooks").join("pre-commit").exists());
    assert!(root.join(".git").join("hooks").join("pre-push").exists());
}

#[test]
fn test_init_uses_default_config_template_env() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();
    let template = root.join("default-config.toml");

    fs::write(
        &template,
        "[core]\nstrict = true\nauto_fix = true\n\n[hotspots]\nlimit = 3\n",
    )
    .unwrap();

    setup_git_repo(tmp.path());

    let _guard = DirGuard::from_utf8(root);
    let _env = EnvGuard::set("CHANGEGUARD_DEFAULT_CONFIG", template.as_std_path());

    let result = execute_init(false);
    assert!(result.is_ok());

    let config = fs::read_to_string(root.join(".changeguard").join("config.toml")).unwrap();
    assert!(config.contains("strict = true"));
    assert!(config.contains("limit = 3"));
}

#[test]
fn test_init_git_hooks_are_idempotent() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    setup_git_repo(tmp.path());

    let _guard = DirGuard::from_utf8(root);

    execute_init(false).unwrap();
    execute_init(false).unwrap();

    let pre_commit = fs::read_to_string(root.join(".git").join("hooks").join("pre-commit"))
        .expect("pre-commit hook should be installed");
    let pre_push = fs::read_to_string(root.join(".git").join("hooks").join("pre-push"))
        .expect("pre-push hook should be installed");

    assert_eq!(pre_commit.matches("# changeguard-ledger-gate").count(), 1);
    assert_eq!(pre_push.matches("# changeguard-ledger-gate").count(), 1);
}

#[test]
fn test_init_appends_git_hooks_without_replacing_existing_content() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    setup_git_repo(tmp.path());

    let hooks_dir = root.join(".git").join("hooks");
    fs::write(
        hooks_dir.join("pre-commit"),
        "#!/usr/bin/env bash\necho existing pre-commit\n",
    )
    .unwrap();
    fs::write(
        hooks_dir.join("pre-push"),
        "#!/usr/bin/env bash\necho existing pre-push\n",
    )
    .unwrap();

    let _guard = DirGuard::from_utf8(root);

    execute_init(false).unwrap();

    let pre_commit = fs::read_to_string(root.join(".git").join("hooks").join("pre-commit"))
        .expect("pre-commit hook should be installed");
    let pre_push = fs::read_to_string(root.join(".git").join("hooks").join("pre-push"))
        .expect("pre-push hook should be installed");

    assert!(pre_commit.contains("echo existing pre-commit"));
    assert!(pre_commit.contains("# changeguard-ledger-gate"));
    assert!(pre_push.contains("echo existing pre-push"));
    assert!(pre_push.contains("# changeguard-ledger-gate"));
}
