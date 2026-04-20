use camino::Utf8PathBuf;
use changeguard::federated::scanner::FederatedScanner;
use changeguard::federated::schema::{FederatedSchema, PublicInterface};
use changeguard::federated::impact::check_cross_repo_impact;
use changeguard::federated::storage::update_federated_link;
use changeguard::index::symbols::SymbolKind;
use changeguard::impact::packet::ImpactPacket;
use changeguard::state::storage::StorageManager;
use std::fs;
use tempfile::tempdir;

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
        }],
    );
    fs::write(
        repo_a_cg.join("schema.json"),
        serde_json::to_string(&schema_a).unwrap(),
    )
    .unwrap();

    // Create sibling repo B (no schema)
    let repo_b = root_path.join("repo-b");
    fs::create_dir_all(repo_b).unwrap();

    // Current repo
    let current_repo = root_path.join("current");
    fs::create_dir_all(&current_repo).unwrap();

    let scanner = FederatedScanner::new(current_repo);
    let (siblings, warnings) = scanner.scan_siblings().unwrap();

    assert_eq!(siblings.len(), 1);
    assert_eq!(siblings[0].1.repo_name, "repo-a");
    assert!(warnings.is_empty());
    }

#[test]
fn test_federated_path_confinement_security() {
    let tmp = tempdir().unwrap();
    let root_path = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
    
    // Discovery root
    let discovery_root = root_path.join("discovery");
    fs::create_dir_all(&discovery_root).unwrap();

    // Current repo deep inside
    let current_repo = discovery_root.join("nested/repo");
    fs::create_dir_all(&current_repo).unwrap();

    // Sibling outside hierarchy
    let evil_sibling = root_path.join("evil");
    fs::create_dir_all(&evil_sibling).unwrap();

    let scanner = FederatedScanner::new(current_repo);
    let (siblings, _warnings) = scanner.scan_siblings().unwrap();

    // Should only see siblings at nested/ LEVEL, not outside discovery_root
    assert!(siblings.is_empty());
    // Warnings might contain canonicalization failures or "escapes discovery root" if it tried
}

#[test]
fn test_federated_invalid_schema_recovery() {
    let tmp = tempdir().unwrap();
    let root_path = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();

    let repo_a = root_path.join("repo-a");
    let repo_a_cg = repo_a.join(".changeguard");
    fs::create_dir_all(&repo_a_cg).unwrap();

    // Write malformed JSON
    fs::write(repo_a_cg.join("schema.json"), "{ invalid json ]").unwrap();

    let current_repo = root_path.join("current");
    fs::create_dir_all(&current_repo).unwrap();

    let scanner = FederatedScanner::new(current_repo);
    let (siblings, warnings) = scanner.scan_siblings().unwrap();

    assert!(siblings.is_empty());
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("Failed to load schema"));
}

#[test]
fn test_federated_cross_repo_impact_resolution() {
    let tmp = tempdir().unwrap();
    let root_path = Utf8PathBuf::from_path_buf(tmp.path().to_path_buf()).unwrap();
    
    let db_path = tmp.path().join("test.db");
    let storage = StorageManager::init(&db_path).unwrap();

    // 1. Link sibling repo-a
    let repo_a = root_path.join("repo-a");
    let repo_a_cg = repo_a.join(".changeguard");
    fs::create_dir_all(&repo_a_cg).unwrap();
    
    // Initial schema has 'old_symbol'
    let schema_v1 = FederatedSchema::new(
        "repo-a".to_string(),
        vec![PublicInterface {
            symbol: "old_symbol".to_string(),
            file: "src/lib.rs".to_string(),
            kind: SymbolKind::Function,
        }]
    );
    fs::write(repo_a_cg.join("schema.json"), serde_json::to_string(&schema_v1).unwrap()).unwrap();

    update_federated_link(storage.get_connection(), "repo-a", repo_a.as_str(), "2026-01-01").unwrap();

    // 2. Record local dependency on 'old_symbol'
    storage.get_connection().execute(
        "INSERT INTO federated_dependencies (local_symbol, sibling_name, sibling_symbol) VALUES (?1, ?2, ?3)",
        ("my_app", "repo-a", "old_symbol")
    ).unwrap();

    // 3. Sibling REMOVES 'old_symbol'
    let schema_v2 = FederatedSchema::new("repo-a".to_string(), vec![]);
    fs::write(repo_a_cg.join("schema.json"), serde_json::to_string(&schema_v2).unwrap()).unwrap();

    // 4. Run impact analysis
    let mut packet = ImpactPacket::default();
    check_cross_repo_impact(&mut packet, &storage).unwrap();

    println!("ACTUAL REASONS: {:?}", packet.risk_reasons);
    assert!(!packet.risk_reasons.is_empty());
    assert!(packet.risk_reasons.iter().any(|r| r.contains("interface 'old_symbol' which was removed")));
}
