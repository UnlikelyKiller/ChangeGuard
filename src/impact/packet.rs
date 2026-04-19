use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::Utc;
use crate::index::symbols::Symbol;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: PathBuf,
    pub status: String, // e.g., "Added", "Modified", "Deleted", "Renamed"
    pub is_staged: bool,
    pub symbols: Option<Vec<Symbol>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactPacket {
    pub schema_version: String,
    pub timestamp_utc: String, // ISO 8601 string
    pub head_hash: Option<String>,
    pub branch_name: Option<String>,
    pub risk_level: RiskLevel,
    pub risk_reasons: Vec<String>,
    pub changes: Vec<ChangedFile>,
}

impl Default for ImpactPacket {
    fn default() -> Self {
        Self {
            schema_version: "v1".to_string(),
            timestamp_utc: Utc::now().to_rfc3339(),
            head_hash: None,
            branch_name: None,
            risk_level: RiskLevel::Medium,
            risk_reasons: vec!["Provisional baseline risk".to_string()],
            changes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let mut packet = ImpactPacket::default();
        packet.timestamp_utc = "2023-10-27T10:00:00Z".to_string();
        packet.head_hash = Some("abcdef123456".to_string());
        packet.branch_name = Some("main".to_string());
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
        });

        let json = serde_json::to_string_pretty(&packet).unwrap();
        
        // Assert schema version and camelCase
        assert!(json.contains(r#""schemaVersion": "v1""#));
        assert!(json.contains(r#""timestampUtc": "2023-10-27T10:00:00Z""#));
        assert!(json.contains(r#""headHash": "abcdef123456""#));
        assert!(json.contains(r#""isStaged": true"#));
    }
}
