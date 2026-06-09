use changeguard::commands::ask::execute_ask;
use changeguard::gemini::modes::GeminiMode;
use changeguard::state::layout::Layout;
use changeguard::state::storage::StorageManager;
use std::fs;
use tempfile::tempdir;

use crate::common::{DirGuard, cwd_lock};

#[test]
fn test_ask_kg_fallback_logic() {
    let _lock = cwd_lock().lock().unwrap();

    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    // Initialize a minimal git repo
    std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();

    // Write a dummy config to bypass early config errors
    fs::write(
        layout.config_file(),
        "[gemini]\nfast_model = \"dummy\"\n[local_model]\nbase_url = \"http://localhost:11434\"\n",
    )
    .unwrap();

    // Initialize storage and inject a node into KG
    {
        let storage_path = layout.state_subdir().join("ledger.db");
        let storage = StorageManager::init(storage_path.as_std_path()).unwrap();
        let cozo = storage
            .cozo
            .as_ref()
            .expect("Cozo storage should be available");

        // Create the node table and insert a dummy node
        // In a real scenario, this is done by 'index'
        cozo.run_script("
            ?[id, label, category, risk_score, metadata] <- [['test_id', 'SpecialNodeLabel', 'TEST_CATEGORY', 0.0, {}]]
            :insert node {id, label, category, risk_score, metadata}
        ").unwrap();
    }

    // Now run execute_ask. Since semantic index is empty, it should try KG fallback.
    // We expect it to get past the "No codebase context" error because KG fallback provides context.
    // It might fail later when trying to call the LLM, which is fine for this test.

    let result = execute_ask(
        Some("SpecialNodeLabel".into()),
        false, // semantic
        10,    // limit
        GeminiMode::Analyze,
        false, // narrative
        None,  // backend
        false, // auto_index
        1,     // timeout_secs (short)
        false, // no_kg_fallback
    );

    if let Err(e) = result {
        let err_str = e.to_string();
        // It should NOT contain the error about missing codebase context
        assert!(
            !err_str.contains("Global Ask requires codebase context"),
            "Should have used KG context instead of failing: {}",
            err_str
        );
    }
}

#[test]
fn test_ask_no_kg_fallback_suppression() {
    let _lock = cwd_lock().lock().unwrap();

    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    std::process::Command::new("git")
        .arg("init")
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();
    fs::write(
        layout.config_file(),
        "[gemini]\nfast_model = \"dummy\"\n[local_model]\nbase_url = \"http://localhost:11434\"\n",
    )
    .unwrap();

    {
        let storage_path = layout.state_subdir().join("ledger.db");
        let storage = StorageManager::init(storage_path.as_std_path()).unwrap();
        let cozo = storage
            .cozo
            .as_ref()
            .expect("Cozo storage should be available");
        cozo.run_script("
            ?[id, label, category, risk_score, metadata] <- [['test_id', 'SpecialNodeLabel', 'TEST_CATEGORY', 0.0, {}]]
            :insert node {id, label, category, risk_score, metadata}
        ").unwrap();
    }

    // Run with no_kg_fallback = true
    let result = execute_ask(
        Some("SpecialNodeLabel".into()),
        false, // semantic
        10,    // limit
        GeminiMode::Analyze,
        false, // narrative
        None,  // backend
        false, // auto_index
        1,     // timeout_secs
        true,  // no_kg_fallback = TRUE
    );

    // It SHOULD proceed to the LLM call with no context, which fails with dummy model
    let err = result.unwrap_err();
    assert!(
        !err.to_string()
            .contains("Global Ask requires codebase context"),
        "Error should have been removed: {}",
        err
    );
}
