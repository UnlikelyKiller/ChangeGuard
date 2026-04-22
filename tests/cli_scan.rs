use changeguard::commands::scan::execute_scan;
use changeguard::git::repo::open_repo;
use changeguard::git::status::get_repo_status;
use changeguard::git::{ChangeType, FileChange};
use changeguard::state::layout::Layout;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock, git_cmd, setup_git_repo};

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

    let result = execute_scan(false);
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

    let result = execute_scan(false);
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

    let result = execute_scan(false);
    assert!(result.is_ok());
}

fn read_status(root: &Path) -> Vec<FileChange> {
    let repo = open_repo(root).expect("repository should open");
    get_repo_status(&repo).expect("status should be readable")
}

#[test]
fn test_scan_status_classifies_staged_add_delete_rename() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    fs::write(root.join("old_name.txt"), "tracked").unwrap();
    fs::write(root.join("delete_me.txt"), "remove").unwrap();
    git_cmd(root, &["add", "."]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    git_cmd(root, &["mv", "old_name.txt", "new_name.txt"]);
    git_cmd(root, &["rm", "delete_me.txt"]);
    fs::write(root.join("added.txt"), "new").unwrap();
    git_cmd(root, &["add", "added.txt"]);

    let changes = read_status(root);

    assert!(changes.iter().any(|change| {
        change.is_staged
            && change.path == Path::new("added.txt")
            && matches!(change.change_type, ChangeType::Added)
    }));

    assert!(changes.iter().any(|change| {
        change.is_staged
            && change.path == Path::new("delete_me.txt")
            && matches!(change.change_type, ChangeType::Deleted)
    }));

    assert!(changes.iter().any(|change| {
        change.is_staged
            && change.path == Path::new("new_name.txt")
            && matches!(
                &change.change_type,
                ChangeType::Renamed { old_path } if old_path == &PathBuf::from("old_name.txt")
            )
    }));
}

#[test]
fn test_scan_status_keeps_staged_and_unstaged_entries_for_same_file() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    setup_git_repo(root);

    fs::write(root.join("dual_state.txt"), "line one\n").unwrap();
    git_cmd(root, &["add", "dual_state.txt"]);
    git_cmd(root, &["commit", "-m", "initial commit"]);

    fs::write(root.join("dual_state.txt"), "line one\nstaged\n").unwrap();
    git_cmd(root, &["add", "dual_state.txt"]);
    fs::write(root.join("dual_state.txt"), "line one\nstaged\nunstaged\n").unwrap();

    let changes = read_status(root);

    let matching: Vec<_> = changes
        .iter()
        .filter(|change| {
            change.path == Path::new("dual_state.txt")
                && matches!(change.change_type, ChangeType::Modified)
        })
        .collect();

    assert_eq!(matching.len(), 2, "expected staged and unstaged entries");
    assert!(matching.iter().any(|change| change.is_staged));
    assert!(matching.iter().any(|change| !change.is_staged));
}
