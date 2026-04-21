use camino::Utf8PathBuf;
use changeguard::git::GitError;
use changeguard::impact::hotspots::calculate_hotspots;
use changeguard::impact::temporal::{CommitFileSet, HistoryProvider};
use changeguard::state::storage::StorageManager;
use std::collections::HashSet;
use tempfile::tempdir;

struct MockHistoryProvider {
    history: Vec<CommitFileSet>,
}

impl HistoryProvider for MockHistoryProvider {
    fn get_history(
        &self,
        _max_commits: usize,
        _all_parents: bool,
    ) -> Result<Vec<CommitFileSet>, GitError> {
        Ok(self.history.clone())
    }
}

#[test]
fn test_hotspots_use_normalized_multiplication_and_path_tie_breaking() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::init(&tmp.path().join("ledger.db")).unwrap();
    insert_snapshot(&storage);
    insert_complexity(&storage, "src/a.rs", 10);
    insert_complexity(&storage, "src/b.rs", 20);
    insert_complexity(&storage, "src/c.rs", 20);

    let history = MockHistoryProvider {
        history: vec![
            commit(&["src/a.rs", "src/b.rs", "src/c.rs"]),
            commit(&["src/a.rs", "src/b.rs", "src/c.rs"]),
            commit(&["src/a.rs"]),
        ],
    };

    let hotspots = calculate_hotspots(&storage, &history, 10, 10, false, None, None).unwrap();

    assert_eq!(hotspots[0].path.to_string_lossy(), "src/b.rs");
    assert_eq!(hotspots[1].path.to_string_lossy(), "src/c.rs");
    assert_eq!(hotspots[0].score, hotspots[1].score);
    assert!(hotspots[0].score > hotspots[2].score);
}

#[test]
fn test_hotspots_apply_directory_and_language_filters() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::init(&tmp.path().join("ledger.db")).unwrap();
    insert_snapshot(&storage);
    insert_complexity(&storage, "src/a.rs", 10);
    insert_complexity(&storage, "tests/a.rs", 10);
    insert_complexity(&storage, "src/a.py", 10);

    let history = MockHistoryProvider {
        history: vec![commit(&["src/a.rs", "tests/a.rs", "src/a.py"])],
    };

    let hotspots =
        calculate_hotspots(&storage, &history, 10, 10, false, Some("src/"), Some("rs")).unwrap();

    assert_eq!(hotspots.len(), 1);
    assert_eq!(hotspots[0].path.to_string_lossy(), "src/a.rs");
}

#[test]
fn test_hotspots_are_json_serializable() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::init(&tmp.path().join("ledger.db")).unwrap();
    insert_snapshot(&storage);
    insert_complexity(&storage, "src/a.rs", 10);

    let history = MockHistoryProvider {
        history: vec![commit(&["src/a.rs"])],
    };

    let hotspots = calculate_hotspots(&storage, &history, 10, 10, false, None, None).unwrap();
    let json = serde_json::to_string(&hotspots).unwrap();

    assert!(json.contains("src/a.rs"));
    assert!(json.contains("score"));
}

#[test]
fn test_hotspots_propagate_malformed_sqlite_rows() {
    let tmp = tempdir().unwrap();
    let storage = StorageManager::init(&tmp.path().join("ledger.db")).unwrap();
    insert_snapshot(&storage);
    let conn = storage.get_connection();
    conn.execute(
        "INSERT INTO symbols (snapshot_id, file_path, symbol_name, symbol_kind, is_public, cognitive_complexity, cyclomatic_complexity)
         VALUES (1, 'src/a.rs', 'a', 'Function', 1, 'bad', 0)",
        [],
    )
    .unwrap();

    let history = MockHistoryProvider {
        history: vec![commit(&["src/a.rs"])],
    };

    let error = calculate_hotspots(&storage, &history, 10, 10, false, None, None).unwrap_err();
    assert!(format!("{error:?}").contains("Invalid column type"));
}

fn insert_complexity(storage: &StorageManager, file_path: &str, complexity: i32) {
    storage
        .get_connection()
        .execute(
            "INSERT INTO symbols (snapshot_id, file_path, symbol_name, symbol_kind, is_public, cognitive_complexity, cyclomatic_complexity)
             VALUES (1, ?1, 'symbol', 'Function', 1, ?2, ?2)",
            (file_path, complexity),
        )
        .unwrap();
}

fn insert_snapshot(storage: &StorageManager) {
    storage
        .get_connection()
        .execute(
            "INSERT INTO snapshots (id, timestamp, is_clean, packet_json) VALUES (1, '2026-01-01T00:00:00Z', 0, '{}')",
            [],
        )
        .unwrap();
}

fn commit(paths: &[&str]) -> CommitFileSet {
    CommitFileSet {
        files: paths.iter().map(Utf8PathBuf::from).collect::<HashSet<_>>(),
        is_merge: false,
    }
}
