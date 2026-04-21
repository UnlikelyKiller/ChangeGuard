use crate::index::symbols::SymbolKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicInterface {
    pub symbol: String,
    pub file: String,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedSchema {
    pub schema_version: String,
    pub repo_name: String,
    pub public_interfaces: Vec<PublicInterface>,
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
        Ok(())
    }

    pub fn new(repo_name: String, mut public_interfaces: Vec<PublicInterface>) -> Self {
        // Engineering standard: deterministic sorting
        public_interfaces.sort_by(|a, b| {
            a.symbol
                .cmp(&b.symbol)
                .then_with(|| a.file.cmp(&b.file))
                .then_with(|| a.kind.cmp(&b.kind))
        });
        Self {
            schema_version: Self::VERSION.to_string(),
            repo_name,
            public_interfaces,
        }
    }
}
