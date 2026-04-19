use camino::{Utf8Path, Utf8PathBuf};
use changeguard::commands::init::execute_init;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

struct DirGuard(Utf8PathBuf);

impl DirGuard {
    fn new<P: AsRef<Utf8Path>>(new_dir: P) -> Self {
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(new_dir.as_ref()).expect("Failed to set current dir");
        Self(Utf8PathBuf::from_path_buf(old_dir).unwrap())
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

#[test]
fn test_init_command_integration() {
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    // Initialize a mock git repository to sandbox gix::discover
    Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .expect("Failed to run git init");

    // Use the guard to ensure we return to the original directory
    let _guard = DirGuard::new(root);

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
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .expect("Failed to run git init");

    let _guard = DirGuard::new(root);

    let result = execute_init(true);
    assert!(result.is_ok());

    let cg_dir = root.join(".changeguard");
    assert!(cg_dir.exists());
    assert!(!root.join(".gitignore").exists());
}
