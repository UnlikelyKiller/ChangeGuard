use camino::Utf8Path;
use changeguard::commands::federate::execute_federate_scan;
use changeguard::commands::init::execute_init;
use changeguard::commands::scan::execute_scan;
use changeguard::federated::schema::{FederatedLedgerEntry, FederatedSchema};
use changeguard::ledger::db::LedgerDb;
use changeguard::ledger::types::{Category, ChangeType, EntryType};
use changeguard::state::layout::Layout;
use changeguard::state::storage::StorageManager;
use chrono::Utc;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock, setup_git_repo};

#[test]
fn test_ledger_federation_flow() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap();

    // 1. Setup "local" repo
    let local_path = root.join("local_repo");
    fs::create_dir_all(&local_path).unwrap();
    setup_git_repo(local_path.as_std_path());
    fs::write(local_path.join("main.rs"), "fn main() {}").unwrap();

    {
        let _guard = DirGuard::from_utf8(&local_path);
        execute_init(false).unwrap();
        execute_scan(true).unwrap();
    }

    // 2. Setup "sibling" repo with schema
    let sibling_path = root.join("sibling_repo");
    fs::create_dir_all(&sibling_path).unwrap();
    setup_git_repo(sibling_path.as_std_path());

    let cg_sibling = sibling_path.join(".changeguard");
    fs::create_dir_all(&cg_sibling).unwrap();

    let ledger_entry = FederatedLedgerEntry {
        tx_id: "test-tx-id".to_string(),
        category: Category::Feature,
        entry_type: EntryType::Implementation,
        entity: "lib.rs".to_string(),
        change_type: ChangeType::Modify,
        summary: "Federated change summary".to_string(),
        reason: "Test reason".to_string(),
        is_breaking: true,
        committed_at: Utc::now().to_rfc3339(),
        trace_id: None,
    };

    let schema =
        FederatedSchema::new("sibling_repo".to_string(), vec![]).with_ledger(vec![ledger_entry]);

    let schema_json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write(cg_sibling.join("schema.json"), schema_json).unwrap();

    // 3. Run federate scan in local repo
    {
        let _guard = DirGuard::from_utf8(&local_path);
        execute_federate_scan().unwrap();
    }

    // 4. Verify federated entry is imported into local DB
    let layout = Layout::new(&local_path);
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path()).unwrap();

    // Use LedgerDb directly to avoid path normalization issues for non-existent files
    let db = LedgerDb::new(storage.get_connection());
    let entries = db
        .get_federated_entries_by_entity("lib.rs", "sibling_repo", 30)
        .unwrap();

    assert!(!entries.is_empty(), "Federated entries should be imported");

    let federated = entries
        .iter()
        .find(|e| e.origin == "SIBLING" && e.trace_id == Some("sibling_repo".to_string()))
        .expect("Should find entry with SIBLING origin and sibling_repo trace_id");

    assert_eq!(federated.summary, "Federated change summary");
    assert!(federated.is_breaking);
}
