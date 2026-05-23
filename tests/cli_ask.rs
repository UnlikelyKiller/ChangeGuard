use changeguard::commands::ask::execute_ask;
use changeguard::gemini::modes::GeminiMode;
use changeguard::impact::packet::ImpactPacket;
use changeguard::state::layout::Layout;
use changeguard::state::storage::StorageManager;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock};

#[test]
fn test_ask_command_no_packet() {
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
    
    // We must init the storage so it can be queried, even if there's no packet.
    // If it's totally missing, execute_ask will try to create it.
    // We expect it to succeed now (fallback to global mode) instead of erroring out.
    // However, it will fail when trying to connect to Gemini/Local if no config is set.
    // We'll write a dummy config to trigger a specific error later in the chain, 
    // proving it got past the "No impact report" check.

    fs::write(layout.config_file(), "[gemini]\nfast_model = \"dummy\"\n").unwrap();

    let result = execute_ask(
        Some("What's up?".into()),
        false, // semantic
        10,    // limit
        GeminiMode::Analyze,
        false, // narrative
        None,  // backend
        false, // auto_index
    );
    
    // It should NOT fail with "No impact report found" anymore.
    // Depending on the test environment, it might fail to reach Gemini or the local model.
    if let Err(e) = result {
        let err_str = e.to_string();
        assert!(!err_str.contains("No impact report found"), "Should fallback to global mode");
    }
}

#[test]
fn test_ask_invalid_config_fails_before_query_execution() {
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
    fs::write(layout.config_file(), "[watch]\ndebounce_ms = 0\n").unwrap();

    {
        let storage =
            StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path()).unwrap();
        storage.save_packet(&ImpactPacket::default()).unwrap();
        // storage is dropped here, releasing the CozoDB lock
    }

    let err = execute_ask(
        Some("What's up?".into()),
        false, // semantic
        10,    // limit
        GeminiMode::Analyze,
        false, // narrative
        None,  // backend
        false, // auto_index
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("debounce_ms"));
}
