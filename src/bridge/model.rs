use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeRecord {
    Hotspot {
        path: String,
        score: f64,
        reason: String,
    },
    LedgerDelta {
        tx_id: String,
        intent: String,
        files_changed: usize,
    },
    Insight {
        memory_id: String,
        relevance: f64,
        content: String,
    },
    VerifyOutcome(BridgeVerifyOutcome),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeVerifyOutcome {
    pub success: bool,
    pub command: String,
    pub error_snippet: Option<String>,
}

impl BridgeRecord {
    pub const VERSION: &'static str = "0.2";
}

// Custom serialization to inject version
#[derive(Serialize)]
struct VersionedRecord<'a> {
    version: &'static str,
    #[serde(flatten)]
    record: &'a BridgeRecord,
}

pub fn serialize_record(record: &BridgeRecord) -> Result<String, serde_json::Error> {
    let versioned = VersionedRecord {
        version: BridgeRecord::VERSION,
        record,
    };
    serde_json::to_string(&versioned)
}

pub fn deserialize_record(json: &str) -> Result<BridgeRecord, serde_json::Error> {
    let raw: serde_json::Value = serde_json::from_str(json)?;
    if let Some(version) = raw.get("version").and_then(|v| v.as_str())
        && version != BridgeRecord::VERSION
    {
        tracing::warn!(
            "Bridge record version mismatch: expected {}, found {}. Attempting best-effort parsing.",
            BridgeRecord::VERSION,
            version
        );
    }
    // We can deserialize directly into BridgeRecord because it ignores extra fields like "version"
    serde_json::from_value(raw)
}
