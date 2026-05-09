use std::path::Path;
use std::fs;
use miette::{IntoDiagnostic, Result};
use scip::types::Index;
use protobuf::Message;

pub struct ScipIndex {
    pub index: Index,
    pub file_hash: String,
}

impl ScipIndex {
    /// Loads a SCIP index from a file and calculates its BLAKE3 hash.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let bytes = fs::read(path).into_diagnostic()?;
        
        // Calculate BLAKE3 hash
        let hash = blake3::hash(&bytes).to_hex().to_string();
        
        // Decode Protobuf using protobuf crate (scip uses protobuf, not prost)
        let index = Index::parse_from_bytes(&bytes).into_diagnostic()?;
        
        Ok(Self {
            index,
            file_hash: hash,
        })
    }
}
