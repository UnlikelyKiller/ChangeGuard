use changeguard::commands::scan::execute_scan;
use changeguard::state::layout::Layout;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

use crate::common::{DirGuard, cwd_lock, setup_git_repo};

fn git_cmd(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_scan_integration_clean() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    fs::write(root.join("initial.txt"), "hello").unwrap();
    git_cmd(root, &["add", "initial.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    let _guard = DirGuard::new(root);

    let result = execute_scan(false, false, false, None);
    assert!(result.is_ok());

    let layout = Layout::new(root.to_string_lossy().as_ref());
    let report = fs::read_to_string(layout.reports_dir().join("latest-scan.json")).unwrap();
    assert!(report.contains("\"isClean\": true"));
}

#[test]
fn test_scan_integration_dirty() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

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

    let result = execute_scan(false, false, false, None);
    assert!(result.is_ok());

    let layout = Layout::new(root.to_string_lossy().as_ref());
    let report = fs::read_to_string(layout.reports_dir().join("latest-scan.json")).unwrap();
    assert!(report.contains("initial.txt"));
    assert!(report.contains("untracked.txt"));
}

#[test]
fn test_scan_integration_detached() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    fs::write(root.join("initial.txt"), "hello").unwrap();
    git_cmd(root, &["add", "initial.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .unwrap();
    let head_sha = String::from_utf8(output.stdout).unwrap().trim().to_string();

    git_cmd(root, &["checkout", &head_sha]);

    let _guard = DirGuard::new(root);

    let result = execute_scan(false, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_scan_impact_out_writes_json_without_json_flag() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    fs::write(root.join("initial.txt"), "hello").unwrap();
    git_cmd(root, &["add", "initial.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    fs::write(root.join("initial.txt"), "modified").unwrap();

    let out_path = root.join("impact.json");
    let _guard = DirGuard::new(root);

    execute_scan(true, false, false, Some(out_path.clone())).unwrap();

    let content = fs::read_to_string(out_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed["schemaVersion"], "v1");
    assert!(parsed["changes"].is_array());
}

#[test]
fn test_scan_out_requires_impact() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    let _guard = DirGuard::new(root);
    let error = execute_scan(false, false, false, Some("out.json".into())).unwrap_err();
    assert!(
        error.to_string().contains("--impact"),
        "expected impact requirement error, got {error:?}"
    );
}

#[test]
fn test_scan_json_requires_impact() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    let _guard = DirGuard::new(root);
    let error = execute_scan(false, false, true, None).unwrap_err();
    assert!(
        error.to_string().contains("--impact"),
        "expected impact requirement error, got {error:?}"
    );
}

#[test]
fn test_scan_summary_requires_impact() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    let _guard = DirGuard::new(root);
    let error = execute_scan(false, true, false, None).unwrap_err();
    assert!(
        error.to_string().contains("--impact"),
        "expected impact requirement error, got {error:?}"
    );
}

#[test]
fn test_scan_impact_excludes_tracked_ignored() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    fs::create_dir_all(root.join(".changeguard")).unwrap();
    fs::write(
        root.join(".changeguard/config.toml"),
        "[watch]\nignore_patterns = [\"ignored.rs\"]\n",
    )
    .unwrap();

    fs::write(root.join("ignored.rs"), "// ignored content").unwrap();
    git_cmd(root, &["add", "ignored.rs"]);
    git_cmd(root, &["commit", "-m", "add ignored"]);
    fs::write(root.join("ignored.rs"), "// modified ignored content").unwrap();

    fs::write(root.join("normal.rs"), "// normal content").unwrap();
    git_cmd(root, &["add", "normal.rs"]);
    git_cmd(root, &["commit", "-m", "add normal"]);
    fs::write(root.join("normal.rs"), "// modified normal content").unwrap();

    let _guard = DirGuard::new(root);

    let result = execute_scan(true, false, false, None);
    assert!(result.is_ok());

    let layout = Layout::new(root.to_string_lossy().as_ref());
    let report = fs::read_to_string(layout.reports_dir().join("latest-scan.json")).unwrap();
    assert!(
        !report.contains("ignored.rs"),
        "Report should not contain ignored.rs under impact"
    );
    assert!(
        report.contains("normal.rs"),
        "Report should contain normal.rs"
    );
}
