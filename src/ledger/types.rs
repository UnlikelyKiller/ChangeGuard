use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum, Default,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[value(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Category {
    Architecture,
    #[default]
    Feature,
    Bugfix,
    Refactor,
    Infra,
    Tooling,
    Docs,
    Chore,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum, Default,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[value(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeType {
    Create,
    #[default]
    Modify,
    Deprecate,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[value(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntryType {
    Implementation,
    Architecture,
    Lesson,
    Reconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Unverified,
    PartiallyVerified,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum VerificationBasis {
    Tests,
    Build,
    Lint,
    Runtime,
    ManualInspection,
    Inferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransactionRequest {
    pub category: Category,
    pub entity: String,
    pub planned_action: Option<String>,
    pub issue_ref: Option<String>,
    pub operation_id: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitRequest {
    pub change_type: ChangeType,
    pub summary: String,
    pub reason: String,
    pub is_breaking: bool,
    pub verification_status: Option<VerificationStatus>,
    pub verification_basis: Option<VerificationBasis>,
    pub outcome_notes: Option<String>,
    pub issue_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub tx_id: String,
    pub operation_id: Option<String>,
    pub status: String,
    pub category: Category,
    pub entity: String,
    pub entity_normalized: String,
    pub planned_action: Option<String>,
    pub session_id: String,
    pub source: String,
    pub started_at: String,
    pub resolved_at: Option<String>,
    pub detected_at: Option<String>,
    pub drift_count: i32,
    pub first_seen_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub issue_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub id: i64,
    pub tx_id: String,
    pub category: Category,
    pub entry_type: EntryType,
    pub entity: String,
    pub entity_normalized: String,
    pub change_type: ChangeType,
    pub summary: String,
    pub reason: String,
    pub is_breaking: bool,
    pub committed_at: String,
    pub verification_status: Option<VerificationStatus>,
    pub verification_basis: Option<VerificationBasis>,
    pub outcome_notes: Option<String>,
    pub origin: String,
    pub trace_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_category_serialization() {
        assert_eq!(
            serde_json::to_string(&Category::Architecture).unwrap(),
            "\"ARCHITECTURE\""
        );
        assert_eq!(
            serde_json::from_str::<Category>("\"FEATURE\"").unwrap(),
            Category::Feature
        );
    }

    #[test]
    fn test_change_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ChangeType::Create).unwrap(),
            "\"CREATE\""
        );
        assert_eq!(
            serde_json::from_str::<ChangeType>("\"MODIFY\"").unwrap(),
            ChangeType::Modify
        );
    }

    #[test]
    fn test_entry_type_serialization() {
        assert_eq!(
            serde_json::to_string(&EntryType::Implementation).unwrap(),
            "\"IMPLEMENTATION\""
        );
        assert_eq!(
            serde_json::from_str::<EntryType>("\"ARCHITECTURE\"").unwrap(),
            EntryType::Architecture
        );
    }

    #[test]
    fn test_verification_status_serialization() {
        assert_eq!(
            serde_json::to_string(&VerificationStatus::Verified).unwrap(),
            "\"verified\""
        );
        assert_eq!(
            serde_json::from_str::<VerificationStatus>("\"partially_verified\"").unwrap(),
            VerificationStatus::PartiallyVerified
        );
    }

    #[test]
    fn test_verification_basis_serialization() {
        assert_eq!(
            serde_json::to_string(&VerificationBasis::ManualInspection).unwrap(),
            "\"manual_inspection\""
        );
        assert_eq!(
            serde_json::from_str::<VerificationBasis>("\"tests\"").unwrap(),
            VerificationBasis::Tests
        );
    }
}
