use crate::index::symbols::SymbolKind;
use crate::ledger::types::{Category, ChangeType, EntryType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicInterface {
    pub symbol: String,
    pub file: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FederatedLedgerEntry {
    pub tx_id: String,
    pub category: Category,
    pub entry_type: EntryType,
    pub entity: String,
    pub change_type: ChangeType,
    pub summary: String,
    pub reason: String,
    pub is_breaking: bool,
    pub committed_at: String,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedSchema {
    pub schema_version: String,
    pub repo_name: String,
    pub public_interfaces: Vec<PublicInterface>,
    pub ledger: Option<Vec<FederatedLedgerEntry>>,
}

impl FederatedSchema {
    pub const VERSION: &'static str = "1.0";

    pub fn validate(&self) -> miette::Result<()> {
        if self.schema_version != Self::VERSION {
            return Err(miette::miette!(
                "Unsupported schema version: {}. Expected: {}",
                self.schema_version,
                Self::VERSION
            ));
        }
        if self.repo_name.trim().is_empty() {
            return Err(miette::miette!(
                "Invalid schema: repo_name must not be empty"
            ));
        }
        for interface in &self.public_interfaces {
            if interface.symbol.trim().is_empty() {
                return Err(miette::miette!(
                    "Invalid schema: public interface symbol must not be empty"
                ));
            }
            if interface.file.trim().is_empty() {
                return Err(miette::miette!(
                    "Invalid schema: public interface file must not be empty"
                ));
            }
        }
        if let Some(entries) = &self.ledger {
            for entry in entries {
                if entry.tx_id.trim().is_empty() {
                    return Err(miette::miette!(
                        "Invalid schema: ledger tx_id must not be empty"
                    ));
                }
                if entry.entity.trim().is_empty() {
                    return Err(miette::miette!(
                        "Invalid schema: ledger entity must not be empty"
                    ));
                }
                if entry.entity.contains("..") {
                    return Err(miette::miette!(
                        "Security violation: ledger entity '{}' contains path traversal",
                        entry.entity
                    ));
                }
                if entry.entity.starts_with('/') || entry.entity.contains(':') {
                    // Basic check for absolute paths (Unix and Windows-ish)
                    return Err(miette::miette!(
                        "Security violation: ledger entity '{}' is an absolute path",
                        entry.entity
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn new(repo_name: String, mut public_interfaces: Vec<PublicInterface>) -> Self {
        // Engineering standard: deterministic sorting
        public_interfaces.sort();
        Self {
            schema_version: Self::VERSION.to_string(),
            repo_name,
            public_interfaces,
            ledger: None,
        }
    }

    pub fn with_ledger(mut self, mut ledger: Vec<FederatedLedgerEntry>) -> Self {
        ledger.sort();
        self.ledger = Some(ledger);
        self
    }
}
