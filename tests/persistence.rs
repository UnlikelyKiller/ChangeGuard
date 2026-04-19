use changeguard::impact::packet::{ChangedFile, ImpactPacket};
use changeguard::state::storage::StorageManager;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_persistence_integration() {
    let tmp = tempdir().unwrap();
    let db_path = tmp.path().join("test_ledger.db");

    // 1. Initialize and save
    {
        let storage = StorageManager::init(&db_path).unwrap();
        let mut packet = ImpactPacket {
            head_hash: Some("commit_1".to_string()),
            ..ImpactPacket::default()
        };
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
        });

        storage.save_packet(&packet).unwrap();
    }

    // 2. Re-open and verify
    {
        let storage = StorageManager::init(&db_path).unwrap();
        let latest = storage.get_latest_packet().unwrap().unwrap();
        assert_eq!(latest.head_hash, Some("commit_1".to_string()));
        assert_eq!(latest.changes.len(), 1);
        assert_eq!(latest.changes[0].path, PathBuf::from("src/main.rs"));
    }
}
