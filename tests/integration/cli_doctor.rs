use crate::common::{DirGuard, cwd_lock};
use changeguard::commands::doctor::execute_doctor;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_doctor_command_integration() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();

    // Initialize a mock git repository
    Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .expect("Failed to run git init");

    let _guard = DirGuard::new(tmp.path());

    // execute_doctor prints to stdout, so we just check if it returns Ok
    let result = execute_doctor();

    assert!(result.is_ok());
}
