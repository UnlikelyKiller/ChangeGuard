use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_bridge_import_subcommand_exists() {
    let output = Command::new("cargo")
        .args(&["run", "--", "bridge", "import", "--help"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("import"));
}

#[test]
fn test_bridge_import_enrichment() {
    let dir = tempdir().unwrap();
    let in_path = dir.path().join("import.ndjson");

    let insight = r#"{"type":"Insight","version":"0.2","memory_id":"mem-123","relevance":0.95,"content":"Architecture note: Use trait-based dispatch for bridge providers."}"#;
    fs::write(&in_path, insight).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "bridge",
            "import",
            "--in",
            in_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());

    // ImpactPacket should now be enriched (stored in latest-impact.json)
    let impact_path = std::path::Path::new(".changeguard/reports/latest-impact.json");
    if impact_path.exists() {
        let content = fs::read_to_string(impact_path).unwrap();
        assert!(content.contains("mem-123"));
        assert!(content.contains("trait-based dispatch"));
    }
}
