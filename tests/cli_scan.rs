use changeguard::commands::scan::execute_scan;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

struct DirGuard(PathBuf);

impl DirGuard {
    fn new<P: AsRef<Path>>(new_dir: P) -> Self {
        let old_dir = env::current_dir().unwrap();
        env::set_current_dir(new_dir.as_ref()).expect("Failed to set current dir");
        Self(old_dir)
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.0);
    }
}

fn git_cmd(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to execute git command");
    if !output.status.success() {
        panic!(
            "git command failed: {:?}\nstderr: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_scan_integration_clean() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    git_cmd(root, &["init"]);
    git_cmd(root, &["config", "user.email", "test@example.com"]);
    git_cmd(root, &["config", "user.name", "Test User"]);

    fs::write(root.join("initial.txt"), "hello").unwrap();
    git_cmd(root, &["add", "initial.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    let _guard = DirGuard::new(root);

    let result = execute_scan();
    assert!(result.is_ok());
}

#[test]
fn test_scan_integration_dirty() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    git_cmd(root, &["init"]);
    git_cmd(root, &["config", "user.email", "test@example.com"]);
    git_cmd(root, &["config", "user.name", "Test User"]);

    fs::write(root.join("initial.txt"), "hello").unwrap();
    git_cmd(root, &["add", "initial.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    // Add untracked file
    fs::write(root.join("untracked.txt"), "new").unwrap();

    // Modify existing file
    fs::write(root.join("initial.txt"), "modified").unwrap();

    // Stage a change
    fs::write(root.join("staged.txt"), "staged").unwrap();
    git_cmd(root, &["add", "staged.txt"]);

    let _guard = DirGuard::new(root);

    let result = execute_scan();
    assert!(result.is_ok());
}

#[test]
fn test_scan_integration_detached() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    git_cmd(root, &["init"]);
    git_cmd(root, &["config", "user.email", "test@example.com"]);
    git_cmd(root, &["config", "user.name", "Test User"]);

    fs::write(root.join("initial.txt"), "hello").unwrap();
    git_cmd(root, &["add", "initial.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .unwrap();
    let head_sha = String::from_utf8(output.stdout).unwrap().trim().to_string();

    git_cmd(root, &["checkout", &head_sha]);

    let _guard = DirGuard::new(root);

    let result = execute_scan();
    assert!(result.is_ok());
}
