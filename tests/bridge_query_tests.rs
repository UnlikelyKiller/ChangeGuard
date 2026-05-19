use std::process::Command;

#[test]
fn test_bridge_query_subcommand_exists() {
    let binary = option_env!("CARGO_BIN_EXE_changeguard").unwrap_or("target/debug/changeguard");
    let output = Command::new(binary)
        .args(["bridge", "query", "--help"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("query"));
}

#[test]
fn test_bridge_query_fail_open_on_missing_binary() {
    // We expect the command to succeed even if ai-brains is missing (fail-open)
    // but emit a warning.
    let binary = option_env!("CARGO_BIN_EXE_changeguard").unwrap_or("target/debug/changeguard");
    let output = Command::new(binary)
        .args(["bridge", "query", "unlikely-to-find-anything-12345"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
}
