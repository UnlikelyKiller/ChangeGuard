use std::process::Command;

#[test]
fn test_bridge_query_subcommand_exists() {
    let output = Command::new("cargo")
        .args(["run", "--", "bridge", "query", "--help"])
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
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "bridge",
            "query",
            "unlikely-to-find-anything-12345",
        ])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
}
