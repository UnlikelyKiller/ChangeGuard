use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvResult {
    pub results: Vec<OsvPackageResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvPackageResult {
    pub package: OsvPackage,
    pub vulnerabilities: Option<Vec<OsvVulnerability>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvPackage {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvVulnerability {
    pub id: String,
    pub summary: Option<String>,
    pub details: Option<String>,
    pub modified: String,
    pub published: Option<String>,
    pub database_specific: Option<serde_json::Value>,
}

pub struct OsvImporter;

impl OsvImporter {
    pub fn import_from_json(path: &Path) -> Result<OsvResult> {
        let content = std::fs::read_to_string(path).into_diagnostic()?;
        let result: OsvResult = serde_json::from_str(&content).into_diagnostic()?;
        Ok(result)
    }
}
