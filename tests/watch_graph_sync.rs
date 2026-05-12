use camino::Utf8PathBuf;
use changeguard::index::incremental::IncrementalSyncEngine;
use changeguard::index::orchestrator::ProjectIndexer;
use changeguard::state::storage::StorageManager;
use changeguard::watch::batch::WatchBatch;
use changeguard::watch::debounce::Watcher;
use std::fs;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

mod common;
use common::setup_git_repo;

#[test]
fn test_watch_graph_sync() {
    let tmp = tempfile::tempdir().unwrap();
    let root = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
    setup_git_repo(root.as_std_path());
    let state_dir = root.join(".changeguard").join("state");
    fs::create_dir_all(&state_dir).unwrap();
    let db_path = state_dir.join("ledger.db");

    // Pre-initialize storage schema
    let _ = StorageManager::init(db_path.as_std_path()).unwrap();

    let (tx, rx) = channel();
    let repo_root = root.clone();
    let cb_db_path = db_path.clone();
    let callback = Box::new(move |batch: WatchBatch| {
        if let Ok(storage) = StorageManager::init(cb_db_path.as_std_path()) {
            let indexer = ProjectIndexer::new(storage, repo_root.clone());
            let mut engine = IncrementalSyncEngine::new(indexer, repo_root.clone());
            if let Ok(delta) = engine.process_batch(&batch) {
                let _ = tx.send(delta);
            }
        }
    });

    let _watcher = Watcher::new(
        vec![root.clone()],
        Duration::from_millis(100),
        Vec::new(),
        callback,
    )
    .unwrap();

    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let file_path = src_dir.join("lib.rs");
    fs::write(&file_path, "pub fn hello() {}\n").unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut delta = None;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match rx.recv_timeout(remaining) {
            Ok(candidate) if candidate.files_processed > 0 => {
                delta = Some(candidate);
                break;
            }
            Ok(_) => continue,
            Err(err) => panic!("should receive delta within 5 seconds: {err}"),
        }
    }

    let delta = delta.expect("should receive a non-empty delta within 5 seconds");
    assert_eq!(delta.files_processed, 1);
    assert!(delta.nodes_added >= 1);
}
