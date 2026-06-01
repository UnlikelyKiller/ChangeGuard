use std::process::Command;

#[test]
fn test_cli_binary_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_changeguard"))
        .arg("--help")
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ChangeGuard"));
    assert!(stdout.contains("reset"));
}
