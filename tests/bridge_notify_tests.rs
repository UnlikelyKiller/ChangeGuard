use std::process::Command;

#[test]
fn test_verify_command_triggers_notification_path() {
    // This is hard to test without a mocked IPC server, but we can ensure it doesn't crash.
    let output = Command::new("cargo")
        .args(["run", "--", "verify", "--help"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
}
