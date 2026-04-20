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
