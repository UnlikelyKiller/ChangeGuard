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

    // Should fail because no .changeguard/state/ledger.db exists
    let result = execute_ask(Some("What's up?".into()), GeminiMode::Analyze, false, None);
    assert!(result.is_err());
}

#[test]
fn test_ask_invalid_config_fails_before_query_execution() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();
    fs::write(layout.config_file(), "[watch]\ndebounce_ms = 0\n").unwrap();

    let storage =
        StorageManager::init(layout.state_subdir().join("ledger.db").as_std_path()).unwrap();
    storage.save_packet(&ImpactPacket::default()).unwrap();

    let err = execute_ask(Some("What's up?".into()), GeminiMode::Analyze, false, None).unwrap_err();
    assert!(format!("{err:?}").contains("debounce_ms"));
}
