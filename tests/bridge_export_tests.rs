use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_bridge_export_subcommand_exists() {
    let binary = option_env!("CARGO_BIN_EXE_changeguard").unwrap_or("target/debug/changeguard");
    let output = Command::new(binary)
        .args(["bridge", "export", "--help"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export"));
}

#[test]
fn test_bridge_export_file_creation() {
    let dir = tempdir().unwrap();
    let out_path = dir.path().join("export.ndjson");

    let binary = option_env!("CARGO_BIN_EXE_changeguard").unwrap_or("target/debug/changeguard");
    let output = Command::new(binary)
        .args(["bridge", "export", "--out", out_path.to_str().unwrap()])
        .output()
        .expect("failed to execute process");

    // This should fail initially because the command doesn't exist
    assert!(output.status.success());
    assert!(out_path.exists());

    let content = fs::read_to_string(out_path).unwrap();
    // Should be valid NDJSON if records were exported
    if !content.is_empty() {
        assert!(content.contains(r#""type":"#));
    }
}
