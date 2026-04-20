use crate::index::references::ImportExport;
use crate::index::runtime_usage::RuntimeUsage;
use crate::index::symbols::Symbol;
use crate::util::clock::Clock;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisStatus {
    #[default]
    NotRun,
    Ok,
    Unsupported,
    ReadFailed,
    ExtractionFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileAnalysisStatus {
    pub symbols: AnalysisStatus,
    pub imports: AnalysisStatus,
    pub runtime_usage: AnalysisStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: PathBuf,
    pub status: String, // e.g., "Added", "Modified", "Deleted", "Renamed"
    pub is_staged: bool,
    pub symbols: Option<Vec<Symbol>>,
    pub imports: Option<ImportExport>,
    pub runtime_usage: Option<RuntimeUsage>,
    #[serde(default)]
    pub analysis_status: FileAnalysisStatus,
    #[serde(default)]
    pub analysis_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub name: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalCoupling {
    pub file_a: PathBuf,
    pub file_b: PathBuf,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hotspot {
    pub path: PathBuf,
    pub score: f32,
    pub complexity: i32,
    pub frequency: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImpactPacket {
    pub schema_version: String,
    pub timestamp_utc: String, // ISO 8601 string
    pub head_hash: Option<String>,
    pub branch_name: Option<String>,
    pub risk_level: RiskLevel,
    pub risk_reasons: Vec<String>,
    pub changes: Vec<ChangedFile>,
    pub temporal_couplings: Vec<TemporalCoupling>,
    pub hotspots: Vec<Hotspot>,
    pub verification_results: Vec<VerificationResult>,
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
            temporal_couplings: Vec::new(),
            hotspots: Vec::new(),
            verification_results: Vec::new(),
        }
    }
}

impl ImpactPacket {
    pub fn with_clock(clock: &dyn Clock) -> Self {
        Self {
            timestamp_utc: clock.now().to_rfc3339(),
            ..Self::default()
        }
    }

    /// Finalizes the packet by sorting all internal collections deterministically.
    pub fn finalize(&mut self) {
        self.risk_reasons.sort_unstable();

        for file in &mut self.changes {
            if let Some(ref mut symbols) = file.symbols {
                symbols.sort_unstable();
            }
            if let Some(ref mut imports) = file.imports {
                imports.imported_from.sort_unstable();
                imports.exported_symbols.sort_unstable();
            }
            if let Some(ref mut runtime_usage) = file.runtime_usage {
                runtime_usage.env_vars.sort_unstable();
                runtime_usage.config_keys.sort_unstable();
            }
            file.analysis_warnings.sort_unstable();
            file.analysis_warnings.dedup();
        }
        self.changes.sort_unstable();
        self.temporal_couplings.sort_unstable();
        self.hotspots.sort_unstable_by(|a, b| {
            b.score.partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
        });
        self.verification_results.sort_unstable();
    }

    /// Truncates the packet to fit within a target character limit.
    /// Priority: 
    /// 1. Strip verification stdout/stderr
    /// 2. Strip symbol/import/runtime data for unchanged files (if any were included)
    /// 3. Strip temporal couplings
    /// 4. Strip hotspots
    pub fn truncate_for_context(&mut self, target_chars: usize) -> bool {
        let current_json = serde_json::to_string(self).unwrap_or_default();
        if current_json.len() <= target_chars {
            return false;
        }

        // Phase 1: Clear verification output
        for res in &mut self.verification_results {
            if !res.stdout.is_empty() || !res.stderr.is_empty() {
                res.stdout = "[TRUNCATED]".to_string();
                res.stderr = "[TRUNCATED]".to_string();
                res.truncated = true;
            }
        }

        let current_json = serde_json::to_string(self).unwrap_or_default();
        if current_json.len() <= target_chars {
            return true;
        }

        // Phase 2: Strip detailed analysis for non-staged files
        for change in &mut self.changes {
            if !change.is_staged {
                change.symbols = None;
                change.imports = None;
                change.runtime_usage = None;
            }
        }

        let current_json = serde_json::to_string(self).unwrap_or_default();
        if current_json.len() <= target_chars {
            return true;
        }

        // Phase 3: Strip temporal couplings
        self.temporal_couplings.clear();

        let current_json = serde_json::to_string(self).unwrap_or_default();
        if current_json.len() <= target_chars {
            return true;
        }

        // Phase 4: Strip hotspots
        self.hotspots.clear();

        let current_json = serde_json::to_string(self).unwrap_or_default();
        if current_json.len() <= target_chars {
            return true;
        }

        // Phase 5: Last resort - keep only file paths in changes
        for change in &mut self.changes {
            change.symbols = None;
            change.imports = None;
            change.runtime_usage = None;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let mut packet = ImpactPacket {
            timestamp_utc: "2023-10-27T10:00:00Z".to_string(),
            head_hash: Some("abcdef123456".to_string()),
            branch_name: Some("main".to_string()),
            ..ImpactPacket::default()
        };
        packet.changes.push(ChangedFile {
            path: PathBuf::from("src/main.rs"),
            status: "Modified".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        let json = serde_json::to_string_pretty(&packet).unwrap();

        // Assert schema version and camelCase
        assert!(json.contains(r#""schemaVersion": "v1""#));
        assert!(json.contains(r#""timestampUtc": "2023-10-27T10:00:00Z""#));
        assert!(json.contains(r#""headHash": "abcdef123456""#));
        assert!(json.contains(r#""isStaged": true"#));
    }

    #[test]
    fn test_deterministic_sorting() {
        let mut packet = ImpactPacket {
            risk_reasons: vec!["C".to_string(), "A".to_string(), "B".to_string()],
            ..ImpactPacket::default()
        };

        packet.changes.push(ChangedFile {
            path: PathBuf::from("z.rs"),
            status: "Added".to_string(),
            is_staged: true,
            symbols: Some(vec![
                Symbol {
                    name: "foo".into(),
                    kind: crate::index::symbols::SymbolKind::Function,
                    is_public: true,
                    cognitive_complexity: None,
                    cyclomatic_complexity: None,
                },
                Symbol {
                    name: "bar".into(),
                    kind: crate::index::symbols::SymbolKind::Function,
                    is_public: true,
                    cognitive_complexity: None,
                    cyclomatic_complexity: None,
                },
            ]),
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });
        packet.changes.push(ChangedFile {
            path: PathBuf::from("a.rs"),
            status: "Added".to_string(),
            is_staged: true,
            symbols: None,
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
        });

        packet.finalize();

        assert_eq!(packet.risk_reasons, vec!["A", "B", "C"]);
        assert_eq!(packet.changes[0].path, PathBuf::from("a.rs"));
        assert_eq!(packet.changes[1].path, PathBuf::from("z.rs"));

        let z_symbols = packet.changes[1].symbols.as_ref().unwrap();
        assert_eq!(z_symbols[0].name, "bar");
        assert_eq!(z_symbols[1].name, "foo");
    }
}
