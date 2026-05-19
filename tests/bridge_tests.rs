use changeguard::bridge::model::{BridgeRecord, deserialize_record, serialize_record};

#[test]
fn test_bridge_record_serialization() {
    let record = BridgeRecord::Hotspot {
        path: "src/lib.rs".to_string(),
        score: 0.9,
        reason: "high temporal coupling".to_string(),
    };

    let serialized = serialize_record(&record).unwrap();
    assert!(serialized.contains(r#""type":"Hotspot""#));
    assert!(serialized.contains(r#""version":"0.2""#));
}

#[test]
fn test_bridge_record_deserialization() {
    let json = r#"{"type":"Insight","version":"0.2","memory_id":"abc","relevance":0.8,"content":"refactor suggested"}"#;
    let record: BridgeRecord = deserialize_record(json).unwrap();

    if let BridgeRecord::Insight { memory_id, .. } = record {
        assert_eq!(memory_id, "abc");
    } else {
        panic!("Expected Insight variant");
    }
}
