use changeguard::commands::impact::execute_impact;
use changeguard::state::layout::Layout;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct DirGuard {
    original: PathBuf,
}

impl DirGuard {
    fn new(dir: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir).unwrap();
        Self { original }
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

fn setup_git_repo(dir: &std::path::Path) {
    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output();

    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output();

    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir)
        .output();
}

fn git_add_and_commit(dir: &std::path::Path, msg: &str) {
    let _ = std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(dir)
        .output();

    let _ = std::process::Command::new("git")
        .args(["commit", "-m", msg])
        .current_dir(dir)
        .output();
}

#[test]
fn test_impact_warns_on_rules_failure() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    setup_git_repo(dir);

    // Create a file and commit
    fs::create_dir_all(dir.join("src")).unwrap_or(());
    fs::write(dir.join("src").join("main.rs"), "fn main() {}").unwrap();
    git_add_and_commit(dir, "initial");

    // Modify the file
    fs::write(
        dir.join("src").join("main.rs"),
        "fn main() { /* modified */ }",
    )
    .unwrap();

    // Init changeguard
    let _guard = DirGuard::new(dir);
    let layout = Layout::new(dir.to_string_lossy().as_ref());
    layout.ensure_state_dir().unwrap();

    // Write invalid rules.toml
    let rules_path = layout.rules_file();
    let rules_std = rules_path.as_std_path();
    fs::write(rules_std, "this is not valid toml [[[[").unwrap();

    // Impact should still succeed but warn about rules
    let result = execute_impact();
    // The impact command should succeed even with bad rules
    // (it warns but doesn't fail)
    assert!(
        result.is_ok(),
        "Impact should succeed even with invalid rules"
    );
}

#[test]
fn test_impact_succeeds_without_rules_file() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    setup_git_repo(dir);

    fs::create_dir_all(dir.join("src")).unwrap_or(());
    fs::write(dir.join("src").join("main.rs"), "fn main() {}").unwrap();
    git_add_and_commit(dir, "initial");

    fs::write(
        dir.join("src").join("main.rs"),
        "fn main() { /* modified */ }",
    )
    .unwrap();

    let _guard = DirGuard::new(dir);
    let layout = Layout::new(dir.to_string_lossy().as_ref());
    layout.ensure_state_dir().unwrap();

    // No rules file at all — should use defaults
    let result = execute_impact();
    assert!(result.is_ok(), "Impact should succeed with no rules file");
}

#[test]
fn test_impact_creates_report_file() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    setup_git_repo(dir);

    fs::create_dir_all(dir.join("src")).unwrap_or(());
    fs::write(dir.join("src").join("main.rs"), "fn main() {}").unwrap();
    git_add_and_commit(dir, "initial");

    fs::write(
        dir.join("src").join("main.rs"),
        "fn main() { /* modified */ }",
    )
    .unwrap();

    let _guard = DirGuard::new(dir);
    let layout = Layout::new(dir.to_string_lossy().as_ref());
    layout.ensure_state_dir().unwrap();

    let result = execute_impact();
    assert!(result.is_ok());

    let report_path = layout.reports_dir().join("latest-impact.json");
    assert!(report_path.exists(), "Impact report should be written");
}
