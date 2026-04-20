use camino::Utf8PathBuf;
use changeguard::federated::scanner::FederatedScanner;
use changeguard::federated::schema::{FederatedSchema, PublicInterface};
use changeguard::index::symbols::SymbolKind;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_federated_sibling_discovery() {
    let tmp = tempdir().unwrap();
    let root_path = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
    
    // Create sibling repo A
    let repo_a = root_path.join("repo-a");
    let repo_a_cg = repo_a.join(".changeguard");
    fs::create_dir_all(&repo_a_cg).unwrap();
    
    let schema_a = FederatedSchema::new(
        "repo-a".to_string(),
        vec![PublicInterface {
            symbol: "api_v1".to_string(),
            file: "src/lib.rs".to_string(),
            kind: SymbolKind::Function,
        }]
    );
    fs::write(repo_a_cg.join("schema.json"), serde_json::to_string(&schema_a).unwrap()).unwrap();

    // Create sibling repo B (no schema)
    let repo_b = root_path.join("repo-b");
    fs::create_dir_all(repo_b).unwrap();

    // Current repo
    let current_repo = root_path.join("current");
    fs::create_dir_all(&current_repo).unwrap();

    let scanner = FederatedScanner::new(current_repo);
    let siblings = scanner.scan_siblings().unwrap();

    assert_eq!(siblings.len(), 1);
    assert_eq!(siblings[0].1.repo_name, "repo-a");
    assert_eq!(siblings[0].1.public_interfaces.len(), 1);
}

#[test]
fn test_federated_security_symlink_skip() {
    // Only run on non-windows or if we have symlink privileges
    if cfg!(windows) { return; }

    let tmp = tempdir().unwrap();
    let root_path = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
    
    let current_repo = root_path.join("current");
    fs::create_dir_all(&current_repo).unwrap();

    // Create a target outside the hierarchy
    let outside = tempdir().unwrap();
    let outside_path = outside.path();
    
    // Symlink it into siblings
    let link_path = root_path.join("evil-link");
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside_path, &link_path).unwrap();

    let scanner = FederatedScanner::new(current_repo);
    let siblings = scanner.scan_siblings().unwrap();

    // Should skip the symlink
    assert!(siblings.is_empty());
}
