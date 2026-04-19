use changeguard::commands::doctor::execute_doctor;
use std::env;
use tempfile::tempdir;
use std::process::Command;

#[test]
fn test_doctor_command_integration() {
    let tmp = tempdir().unwrap();
    let old_dir = env::current_dir().unwrap();
    
    // Initialize a mock git repository
    Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .expect("Failed to run git init");
    
    env::set_current_dir(tmp.path()).expect("Failed to set current dir");
    
    // execute_doctor prints to stdout, so we just check if it returns Ok
    let result = execute_doctor();
    
    // Restore directory before assertion to avoid being stuck in temp dir on failure
    env::set_current_dir(old_dir).expect("Failed to restore current dir");
    
    assert!(result.is_ok());
}
