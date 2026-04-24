use camino::Utf8Path;
use changeguard::commands::federate::{
    execute_federate_export, execute_federate_scan, execute_federate_status,
};
use changeguard::commands::init::execute_init;
use std::fs;
use tempfile::tempdir;

mod common;
use common::{DirGuard, cwd_lock, setup_git_repo};

#[test]
fn test_federate_export_from_subdirectory() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap().to_path_buf();
    setup_git_repo(tmp.path());

    let _guard = DirGuard::from_utf8(&root);
    execute_init(false).unwrap();

    let subdir = root.join("src").join("inner");
    fs::create_dir_all(&subdir).unwrap();

    // Switch to subdirectory
    let _subguard = DirGuard::from_utf8(&subdir);

    // This should find the repo root and work correctly
    execute_federate_export().unwrap();

    assert!(
        root.join(".changeguard")
            .join("state")
            .join("schema.json")
            .exists()
    );
}

#[test]
fn test_federate_status_from_subdirectory() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = Utf8Path::from_path(tmp.path()).unwrap().to_path_buf();
    setup_git_repo(tmp.path());

    let _guard = DirGuard::from_utf8(&root);
    execute_init(false).unwrap();

    let subdir = root.join("some").join("nested").join("dir");
    fs::create_dir_all(&subdir).unwrap();

    // Switch to subdirectory
    let _subguard = DirGuard::from_utf8(&subdir);

    // This should find the repo root and work correctly (even if no links yet)
    execute_federate_status().unwrap();
}

#[test]
fn test_federate_scan_from_subdirectory() {
    let _lock = cwd_lock().lock().unwrap();

    // Setup sibling repo structure
    let workspace = tempdir().unwrap();
    let workspace_path = Utf8Path::from_path(workspace.path()).unwrap();

    let repo1 = workspace_path.join("repo1");
    let repo2 = workspace_path.join("repo2");

    fs::create_dir_all(&repo1).unwrap();
    fs::create_dir_all(&repo2).unwrap();

    setup_git_repo(repo1.as_std_path());
    setup_git_repo(repo2.as_std_path());

    // Init and export repo2
    {
        let _guard = DirGuard::from_utf8(&repo2);
        execute_init(false).unwrap();
        execute_federate_export().unwrap();
    }

    // Init and scan from repo1 subdirectory
    {
        let _guard = DirGuard::from_utf8(&repo1);
        execute_init(false).unwrap();

        // Mock a scan packet so scan doesn't fail early
        let db_path = repo1.join(".changeguard").join("state").join("ledger.db");
        let storage =
            changeguard::state::storage::StorageManager::init(db_path.as_std_path()).unwrap();
        let packet = changeguard::impact::packet::ImpactPacket::default();
        storage.save_packet(&packet).unwrap();

        let subdir = repo1.join("src");
        fs::create_dir_all(&subdir).unwrap();
        let _subguard = DirGuard::from_utf8(&subdir);

        // This should find repo2 as a sibling
        execute_federate_scan().unwrap();

        // Verify link was created in repo1's ledger
        let links =
            changeguard::federated::storage::get_federated_links(storage.get_connection()).unwrap();
        assert!(links.iter().any(|(name, _, _)| name == "repo2"));
    }
}
