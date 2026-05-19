mod common;
use changeguard::bridge::import::execute_import;
use changeguard::impact::packet::ImpactPacket;
use changeguard::state::layout::Layout;
use common::{DirGuard, cwd_lock};
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
    let insight = r#"{"bridge_version":"0.2","direction":"inbound","timestamp":"2026-05-19T12:00:00Z","project_id":"test-project","record_kind":"insight","payload":{"type":"Insight","memory_id":"mem-123","relevance":0.95,"content":"Architecture note: Use trait-based dispatch for bridge providers."},"privacy":"Public"}"#;
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
