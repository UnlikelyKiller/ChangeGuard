use crate::index::references::ImportExport;
use crate::index::runtime_usage::RuntimeUsage;
use crate::index::symbols::Symbol;
use crate::util::clock::Clock;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataModel {
    pub model_name: String,
    pub model_kind: String,
    pub confidence: f64,
    pub evidence: Option<String>,
}

impl Eq for DataModel {}

impl PartialOrd for DataModel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataModel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.model_name
            .cmp(&other.model_name)
            .then_with(|| self.model_kind.cmp(&other.model_kind))
            .then_with(|| {
                self.confidence
                    .partial_cmp(&other.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiRoute {
    pub method: String,
    pub path_pattern: String,
    pub handler_symbol_name: Option<String>,
    pub framework: String,
    pub route_source: String,
    pub mount_prefix: Option<String>,
    pub is_dynamic: bool,
    pub route_confidence: f64,
    pub evidence: Option<String>,
}

impl Eq for ApiRoute {}

impl PartialOrd for ApiRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ApiRoute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.method
            .cmp(&other.method)
            .then_with(|| self.path_pattern.cmp(&other.path_pattern))
            .then_with(|| self.framework.cmp(&other.framework))
            .then_with(|| {
                self.route_confidence
                    .partial_cmp(&other.route_confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CIGate {
    pub platform: String,
    pub job_name: String,
    pub trigger: Option<String>,
}

impl Eq for CIGate {}

impl PartialOrd for CIGate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CIGate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.platform
            .cmp(&other.platform)
            .then_with(|| self.job_name.cmp(&other.job_name))
            .then_with(|| self.trigger.cmp(&other.trigger))
    }
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
    #[serde(default)]
    pub api_routes: Vec<ApiRoute>,
    #[serde(default)]
    pub data_models: Vec<DataModel>,
    #[serde(default)]
    pub ci_gates: Vec<CIGate>,
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

impl Eq for TemporalCoupling {}

impl PartialOrd for TemporalCoupling {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TemporalCoupling {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file_a
            .cmp(&other.file_a)
            .then_with(|| self.file_b.cmp(&other.file_b))
            .then_with(|| {
                self.score
                    .partial_cmp(&other.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StructuralCoupling {
    pub caller_symbol_name: String,
    pub callee_symbol_name: String,
    pub caller_file_path: PathBuf,
}

impl Eq for StructuralCoupling {}

impl PartialOrd for StructuralCoupling {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StructuralCoupling {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.caller_symbol_name
            .cmp(&other.caller_symbol_name)
            .then_with(|| self.callee_symbol_name.cmp(&other.callee_symbol_name))
            .then_with(|| self.caller_file_path.cmp(&other.caller_file_path))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CentralityRisk {
    pub symbol_name: String,
    pub entrypoints_reachable: usize,
}

impl Eq for CentralityRisk {}

impl PartialOrd for CentralityRisk {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CentralityRisk {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.symbol_name
            .cmp(&other.symbol_name)
            .then_with(|| self.entrypoints_reachable.cmp(&other.entrypoints_reachable))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hotspot {
    pub path: PathBuf,
    pub score: f32,
    pub complexity: i32,
    pub frequency: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub centrality: Option<usize>,
}

impl Eq for Hotspot {}

impl PartialOrd for Hotspot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hotspot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path).then_with(|| {
            self.score
                .partial_cmp(&other.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoverageDelta {
    pub file_path: String,
    pub pattern_kind: String,
    pub previous_count: usize,
    pub current_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CoveringTest {
    pub test_file: String,
    pub test_symbol: String,
    pub confidence: f64,
    pub mapping_kind: String,
}

impl Eq for CoveringTest {}

impl PartialOrd for CoveringTest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CoveringTest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.test_file
            .cmp(&other.test_file)
            .then_with(|| self.test_symbol.cmp(&other.test_symbol))
            .then_with(|| {
                self.confidence
                    .partial_cmp(&other.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| self.mapping_kind.cmp(&other.mapping_kind))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestCoverage {
    pub changed_symbol: String,
    pub changed_file: String,
    pub covering_tests: Vec<CoveringTest>,
}

impl Eq for TestCoverage {}

impl PartialOrd for TestCoverage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TestCoverage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.changed_symbol
            .cmp(&other.changed_symbol)
            .then_with(|| self.changed_file.cmp(&other.changed_file))
    }
}

impl Eq for CoverageDelta {}

impl PartialOrd for CoverageDelta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CoverageDelta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file_path
            .cmp(&other.file_path)
            .then_with(|| self.pattern_kind.cmp(&other.pattern_kind))
    }
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
    pub structural_couplings: Vec<StructuralCoupling>,
    pub centrality_risks: Vec<CentralityRisk>,
    #[serde(default)]
    pub logging_coverage_delta: Vec<CoverageDelta>,
    #[serde(default)]
    pub error_handling_delta: Vec<CoverageDelta>,
    #[serde(default)]
    pub telemetry_coverage_delta: Vec<CoverageDelta>,
    #[serde(default)]
    pub infrastructure_dirs: Vec<String>,
    #[serde(default)]
    pub test_coverage: Vec<TestCoverage>,
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
            structural_couplings: Vec::new(),
            centrality_risks: Vec::new(),
            logging_coverage_delta: Vec::new(),
            error_handling_delta: Vec::new(),
            telemetry_coverage_delta: Vec::new(),
            infrastructure_dirs: Vec::new(),
            test_coverage: Vec::new(),
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
        self.structural_couplings.sort_unstable();
        self.centrality_risks.sort_unstable();
        self.logging_coverage_delta.sort_unstable();
        self.error_handling_delta.sort_unstable();
        self.telemetry_coverage_delta.sort_unstable();
        self.infrastructure_dirs.sort_unstable();
        self.test_coverage.sort_unstable();
        self.hotspots.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
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

        // Phase 3: Strip temporal and structural couplings
        self.temporal_couplings.clear();
        self.structural_couplings.clear();
        self.centrality_risks.clear();
        self.logging_coverage_delta.clear();
        self.error_handling_delta.clear();
        self.telemetry_coverage_delta.clear();
        self.infrastructure_dirs.clear();
        self.test_coverage.clear();

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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
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
                    line_start: None,
                    line_end: None,
                    qualified_name: None,
                    byte_start: None,
                    byte_end: None,
                    entrypoint_kind: None,
                },
                Symbol {
                    name: "bar".into(),
                    kind: crate::index::symbols::SymbolKind::Function,
                    is_public: true,
                    cognitive_complexity: None,
                    cyclomatic_complexity: None,
                    line_start: None,
                    line_end: None,
                    qualified_name: None,
                    byte_start: None,
                    byte_end: None,
                    entrypoint_kind: None,
                },
            ]),
            imports: None,
            runtime_usage: None,
            analysis_status: FileAnalysisStatus::default(),
            analysis_warnings: Vec::new(),
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
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
            api_routes: Vec::new(),
            data_models: Vec::new(),
            ci_gates: Vec::new(),
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
