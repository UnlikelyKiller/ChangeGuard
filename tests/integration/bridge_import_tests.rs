use crate::common::{DirGuard, cwd_lock};
use changeguard::bridge::import::execute_import;
use changeguard::bridge::model::{BridgeRecord, calculate_hash};
use changeguard::impact::packet::ImpactPacket;
use changeguard::state::layout::Layout;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_bridge_import_enrichment() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();

    // Create a dummy latest-impact.json
    let dummy_packet = ImpactPacket::default();
    let dummy_json = serde_json::to_string_pretty(&dummy_packet).unwrap();
    let latest_impact_path = layout.reports_dir().join("latest-impact.json");
    fs::write(&latest_impact_path, dummy_json).unwrap();

    let in_path = root.join("import.ndjson");
    let insight = r#"{"bridge_version":"0.3","direction":"inbound","timestamp":"2026-05-19T12:00:00Z","project_id":"test-project","record_kind":"insight","payload":{"type":"Insight","memory_id":"mem-123","relevance":0.95,"content":"Architecture note: Use trait-based dispatch for bridge providers."},"privacy":"Public"}"#;
    fs::write(&in_path, insight).unwrap();

    // Call execute_import directly
    let res = execute_import(in_path.to_string());
    assert!(res.is_ok(), "execute_import failed: {:?}", res);

    // Read the updated latest-impact.json
    let updated_content = fs::read_to_string(&latest_impact_path).unwrap();
    assert!(
        updated_content.contains("mem-123"),
        "mem-123 not found in updated content: {}",
        updated_content
    );
    assert!(
        updated_content.contains("trait-based dispatch"),
        "content not found in updated content: {}",
        updated_content
    );
}

#[test]
fn test_bridge_import_lineage_bootstrap() {
    let _lock = cwd_lock().lock().unwrap();
    let tmp = tempdir().unwrap();
    let root = camino::Utf8Path::from_path(tmp.path()).unwrap();
    let _guard = DirGuard::from_utf8(root);

    let layout = Layout::new(root);
    layout.ensure_state_dir().unwrap();

    // Create a dummy latest-impact.json
    let dummy_packet = ImpactPacket::default();
    let dummy_json = serde_json::to_string_pretty(&dummy_packet).unwrap();
    let latest_impact_path = layout.reports_dir().join("latest-impact.json");
    fs::write(&latest_impact_path, dummy_json).unwrap();

    // Construct record 1 with parent_hash: null (None)
    let in_path = root.join("import.ndjson");
    let record1 = r#"{"bridge_version":"0.3","direction":"inbound","timestamp":"2026-05-19T12:00:00Z","project_id":"test-project","record_kind":"insight","payload":{"type":"Insight","memory_id":"mem-1","relevance":0.95,"content":"Initial bootstrapped record"},"privacy":"Public"}"#;

    // De-serialize to compute hash for record 2
    let parsed1: BridgeRecord = serde_json::from_str(record1).unwrap();
    let h1 = calculate_hash(&parsed1);

    // Construct record 2 matching record 1's hash
    let record2 = format!(
        r#"{{"bridge_version":"0.3","direction":"inbound","timestamp":"2026-05-19T12:01:00Z","parent_hash":"{}","project_id":"test-project","record_kind":"insight","payload":{{"type":"Insight","memory_id":"mem-2","relevance":0.95,"content":"Valid second record"}},"privacy":"Public"}}"#,
        h1
    );

    // Construct record 3 with invalid parent hash
    let record3 = r#"{"bridge_version":"0.3","direction":"inbound","timestamp":"2026-05-19T12:02:00Z","parent_hash":"invalid_hash","project_id":"test-project","record_kind":"insight","payload":{"type":"Insight","memory_id":"mem-3","relevance":0.95,"content":"Invalid third record"},"privacy":"Public"}"#;

    let ndjson = format!("{}\n{}\n{}\n", record1, record2, record3);
    fs::write(&in_path, ndjson).unwrap();

    // Call execute_import directly
    let res = execute_import(in_path.to_string());
    assert!(res.is_ok(), "execute_import failed: {:?}", res);

    // Read the updated latest-impact.json to verify which records were imported
    let updated_content = fs::read_to_string(&latest_impact_path).unwrap();

    // mem-1 and mem-2 should be present because parent_hash: null starts the chain and record 2 matches
    assert!(updated_content.contains("mem-1"), "mem-1 not found");
    assert!(updated_content.contains("mem-2"), "mem-2 not found");

    // mem-3 should NOT be present because parent_hash was invalid and rejected
    assert!(!updated_content.contains("mem-3"), "mem-3 was not rejected");
}
