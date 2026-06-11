use crate::common::{cwd_lock, setup_git_repo};
use camino::Utf8Path;
use changeguard::state::storage::StorageManager;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cargo_lock_ingestion() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let root_utf8 = Utf8Path::from_path(root).unwrap();

    setup_git_repo(root);
    fs::write(
        root.join("Cargo.lock"),
        r#"
[[package]]
name = "serde"
version = "1.0.152"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb132488d2348f7a79a296f187a7412ee291e0a24f0a0d9223011400e955f134"
"#,
    )
    .unwrap();

    let changeguard_bin = env!("CARGO_BIN_EXE_changeguard");

    // Init and Index with --analyze-graph
    Command::new(changeguard_bin)
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();
    Command::new(changeguard_bin)
        .args(["index", "--analyze-graph"])
        .current_dir(root)
        .output()
        .unwrap();

    // Check CozoDB
    let storage = StorageManager::open_read_only(root_utf8).unwrap();
    let cozo = storage.cozo.as_ref().expect("CozoDB should be initialized");

    let res = cozo
        .run_script("?[id, label] := *node{id, label, category: 'package'}")
        .unwrap();
    assert!(!res.rows.is_empty(), "CozoDB should have package nodes");

    let found_serde = res.rows.iter().any(|row| {
        row.first()
            .map(|v| v.to_string().contains("serde"))
            .unwrap_or(false)
    });
    assert!(
        found_serde,
        "Should have found serde package with correct URN"
    );

    // Verify edges
    let _edge_res = cozo
        .run_script("?[src, tgt] := *edge{source: src, target: tgt, relation: 'depends_on'}")
        .unwrap();
    // In our mock lock, serde has no deps, but we can add another package with deps to verify edges.
}

#[test]
fn test_cargo_lock_edges() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = tmp.path();
    let root_utf8 = Utf8Path::from_path(root).unwrap();

    setup_git_repo(root);
    fs::write(
        root.join("Cargo.lock"),
        r#"
[[package]]
name = "anyhow"
version = "1.0.0"

[[package]]
name = "thiserror"
version = "1.0.0"
dependencies = [
 "anyhow",
]
"#,
    )
    .unwrap();

    let changeguard_bin = env!("CARGO_BIN_EXE_changeguard");

    Command::new(changeguard_bin)
        .arg("init")
        .current_dir(root)
        .output()
        .unwrap();
    Command::new(changeguard_bin)
        .args(["index", "--analyze-graph"])
        .current_dir(root)
        .output()
        .unwrap();

    let storage = StorageManager::open_read_only(root_utf8).unwrap();
    let cozo = storage.cozo.as_ref().expect("CozoDB should be initialized");

    let edge_res = cozo
        .run_script("?[src, tgt] := *edge{source: src, target: tgt, relation: 'depends_on'}")
        .unwrap();
    assert!(!edge_res.rows.is_empty(), "Should have depends_on edges");

    let found_edge = edge_res.rows.iter().any(|row| {
        let src = row.first().unwrap().to_string();
        let tgt = row.get(1).unwrap().to_string();
        src.contains("thiserror") && tgt.contains("anyhow")
    });
    assert!(found_edge, "Should have edge from thiserror to anyhow");
}
