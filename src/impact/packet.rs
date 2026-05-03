use crate::contracts::AffectedContract;
use crate::index::env_schema::EnvVarDep;
use crate::index::references::ImportExport;
use crate::index::runtime_usage::RuntimeUsage;
use crate::index::symbols::Symbol;
use crate::observability::signal::ObservabilitySignal;
use crate::util::clock::Clock;
use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize};
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_path: Option<PathBuf>,
    pub is_staged: bool,
    pub symbols: Option<Vec<Symbol>>,
    pub imports: Option<ImportExport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelevantDecision {
    pub file_path: PathBuf,
    pub heading: Option<String>,
    pub excerpt: String,
    pub similarity: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness_days: Option<u32>,
}

impl PartialEq for RelevantDecision {
    fn eq(&self, other: &Self) -> bool {
        self.similarity == other.similarity && self.file_path == other.file_path
    }
}

impl Eq for RelevantDecision {}

impl PartialOrd for RelevantDecision {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RelevantDecision {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .similarity
            .partial_cmp(&self.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.file_path.cmp(&other.file_path))
    }
}

fn deserialize_score<'de, D: Deserializer<'de>>(d: D) -> Result<f32, D::Error> {
    Ok(Option::<f32>::deserialize(d)?.unwrap_or(0.0))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hotspot {
    pub path: PathBuf,
    #[serde(deserialize_with = "deserialize_score")]
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
pub struct RuntimeUsageDelta {
    pub file_path: String,
    pub env_vars_previous_count: usize,
    pub env_vars_current_count: usize,
    pub config_keys_previous_count: usize,
    pub config_keys_current_count: usize,
}

impl Eq for RuntimeUsageDelta {}

impl PartialOrd for RuntimeUsageDelta {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuntimeUsageDelta {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file_path.cmp(&other.file_path)
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct CallChainNode {
    pub symbol: String,
    pub file_path: PathBuf,
    pub is_data_model: bool,
    pub is_external: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct CallChain {
    pub nodes: Vec<CallChainNode>,
    pub has_cycle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TraceConfigType {
    OpenTelemetryCollector,
    JaegerAgent,
    DataDogAgent,
    GrafanaAgent,
    GrafanaTempo,
    Unknown,
}

impl TraceConfigType {
    pub fn from_path(path: &std::path::Path) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("otel") {
            Self::OpenTelemetryCollector
        } else if path_str.contains("jaeger") {
            Self::JaegerAgent
        } else if path_str.contains("datadog") {
            Self::DataDogAgent
        } else if path_str.contains("grafana-agent") {
            Self::GrafanaAgent
        } else if path_str.contains("tempo") {
            Self::GrafanaTempo
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct TraceConfigChange {
    pub file: PathBuf,
    pub config_type: TraceConfigType,
    pub risk_weight: u8,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct TraceEnvVarChange {
    pub var_name: String,
    pub pattern: String,
    pub risk_weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct SdkDependencyDelta {
    pub added: Vec<SdkDependency>,
    pub removed: Vec<SdkDependency>,
    pub modified: Vec<SdkDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct SdkDependency {
    pub sdk_name: String,
    pub file_path: PathBuf,
    pub import_statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ManifestType {
    Dockerfile,
    DockerCompose,
    Kubernetes,
    Terraform,
    Helm,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct DeployManifestChange {
    pub file: PathBuf,
    pub manifest_type: ManifestType,
    pub risk_weight: u8,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataFlowMatch {
    pub chain_label: String,
    pub changed_nodes: Vec<String>,
    pub total_nodes: usize,
    pub change_pct: f64,
    pub risk: RiskLevel,
}

impl Eq for DataFlowMatch {}

impl PartialOrd for DataFlowMatch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DataFlowMatch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .change_pct
            .partial_cmp(&self.change_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.chain_label.cmp(&other.chain_label))
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
    pub env_var_deps: Vec<EnvVarDep>,
    #[serde(default)]
    pub test_coverage: Vec<TestCoverage>,
    #[serde(default)]
    pub runtime_usage_delta: Vec<RuntimeUsageDelta>,
    pub hotspots: Vec<Hotspot>,
    pub verification_results: Vec<VerificationResult>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relevant_decisions: Vec<RelevantDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observability: Vec<ObservabilitySignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_contracts: Vec<AffectedContract>,
    #[serde(default)]
    pub data_flow_matches: Vec<DataFlowMatch>,
    #[serde(default)]
    pub service_map_delta: Option<ServiceMapDelta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trace_config_drift: Vec<TraceConfigChange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trace_env_vars: Vec<TraceEnvVarChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdk_dependencies_delta: Option<SdkDependencyDelta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deploy_manifest_changes: Vec<DeployManifestChange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub analysis_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ServiceMapDelta {
    pub services: Vec<Service>,
    pub affected_services: Vec<String>,
    pub cross_service_edges: Vec<(String, String, usize)>, // (caller_service, callee_service, count)
    pub total_services: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Service {
    pub name: String,
    pub directory: PathBuf,
    pub routes: Vec<String>,      // paths
    pub data_models: Vec<String>, // names
}

impl Default for ImpactPacket {
    fn default() -> Self {
        Self {
            schema_version: "v1".to_string(),
            timestamp_utc: Utc::now().to_rfc3339(),
            head_hash: None,
            branch_name: None,
            risk_level: RiskLevel::Medium,
            risk_reasons: Vec::new(),
            changes: Vec::new(),
            temporal_couplings: Vec::new(),
            structural_couplings: Vec::new(),
            centrality_risks: Vec::new(),
            logging_coverage_delta: Vec::new(),
            error_handling_delta: Vec::new(),
            telemetry_coverage_delta: Vec::new(),
            infrastructure_dirs: Vec::new(),
            env_var_deps: Vec::new(),
            test_coverage: Vec::new(),
            runtime_usage_delta: Vec::new(),
            hotspots: Vec::new(),
            verification_results: Vec::new(),
            relevant_decisions: Vec::new(),
            observability: Vec::new(),
            affected_contracts: Vec::new(),
            service_map_delta: None,
            data_flow_matches: Vec::new(),
            trace_config_drift: Vec::new(),
            trace_env_vars: Vec::new(),
            sdk_dependencies_delta: None,
            deploy_manifest_changes: Vec::new(),
            analysis_warnings: Vec::new(),
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
        self.env_var_deps.sort_unstable();
        self.env_var_deps.dedup();
        self.test_coverage.sort_unstable();
        self.runtime_usage_delta.sort_unstable();
        self.hotspots.sort_unstable_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.path.cmp(&b.path))
        });
        self.verification_results.sort_unstable();
        self.relevant_decisions.sort_unstable_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.file_path.cmp(&b.file_path))
        });
        // Sort observability by severity descending
        self.observability.sort_unstable();
        // Sort affected_contracts by similarity descending, path ascending for ties
        self.affected_contracts.sort_unstable();
        self.data_flow_matches.sort_unstable();
        self.trace_config_drift.sort_unstable();
        self.trace_env_vars.sort_unstable();
        if let Some(ref mut sdk) = self.sdk_dependencies_delta {
            sdk.added.sort_unstable();
            sdk.removed.sort_unstable();
            sdk.modified.sort_unstable();
        }
        self.deploy_manifest_changes.sort_unstable();
    }

    /// Escalate risk_level by one tier for observability/contract signals.
    /// High → Low→Medium or Medium→High; Elevated → Low→Medium only.
    pub fn escalate_risk(&mut self, elevation: crate::observability::signal::RiskElevation) {
        use crate::observability::signal::RiskElevation;
        match elevation {
            RiskElevation::High => {
                self.risk_level = match self.risk_level {
                    RiskLevel::Low => RiskLevel::Medium,
                    _ => RiskLevel::High,
                };
            }
            RiskElevation::Elevated => {
                if self.risk_level == RiskLevel::Low {
                    self.risk_level = RiskLevel::Medium;
                }
            }
            RiskElevation::None => {}
        }
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
        self.env_var_deps.clear();
        self.test_coverage.clear();
        self.runtime_usage_delta.clear();
        self.relevant_decisions.clear();
        // CRITICAL: Clear observability signals which can contain unbounded log excerpts
        self.observability.clear();
        self.affected_contracts.clear();
        self.data_flow_matches.clear();
        self.trace_config_drift.clear();
        self.trace_env_vars.clear();
        self.sdk_dependencies_delta = None;
        self.deploy_manifest_changes.clear();
        self.service_map_delta = None;

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
            old_path: None,
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
            old_path: None,
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
            old_path: None,
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

    #[test]
    fn test_relevant_decision_serialization_roundtrip() {
        let decisions = vec![
            RelevantDecision {
                file_path: PathBuf::from("docs/guide.md"),
                heading: Some("Introduction".to_string()),
                excerpt: "This guide explains...".to_string(),
                similarity: 0.85,
                rerank_score: Some(0.92),
                staleness_days: None,
            },
            RelevantDecision {
                file_path: PathBuf::from("docs/api.md"),
                heading: None,
                excerpt: "API reference section".to_string(),
                similarity: 0.6,
                rerank_score: None,
                staleness_days: None,
            },
        ];

        let packet = ImpactPacket {
            relevant_decisions: decisions,
            ..ImpactPacket::default()
        };

        let json = serde_json::to_string_pretty(&packet).unwrap();
        assert!(json.contains("relevantDecisions"));
        assert!(json.contains("docs/guide.md"));
        assert!(json.contains("rerankScore"));

        // Round-trip
        let parsed: ImpactPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.relevant_decisions.len(), 2);
        assert_eq!(
            parsed.relevant_decisions[0].file_path,
            PathBuf::from("docs/guide.md")
        );
    }

    #[test]
    fn test_relevant_decision_empty_absent_from_json() {
        let packet = ImpactPacket::default();
        let json = serde_json::to_string_pretty(&packet).unwrap();
        assert!(!json.contains("relevantDecisions"));
    }

    #[test]
    fn test_finalize_sorts_relevant_decisions_descending() {
        let mut packet = ImpactPacket {
            relevant_decisions: vec![
                RelevantDecision {
                    file_path: PathBuf::from("docs/c.md"),
                    heading: None,
                    excerpt: "C".to_string(),
                    similarity: 0.5,
                    rerank_score: None,
                    staleness_days: None,
                },
                RelevantDecision {
                    file_path: PathBuf::from("docs/a.md"),
                    heading: None,
                    excerpt: "A".to_string(),
                    similarity: 0.9,
                    rerank_score: None,
                    staleness_days: None,
                },
                RelevantDecision {
                    file_path: PathBuf::from("docs/b.md"),
                    heading: None,
                    excerpt: "B".to_string(),
                    similarity: 0.5,
                    rerank_score: None,
                    staleness_days: None,
                },
            ],
            ..ImpactPacket::default()
        };

        packet.finalize();

        // Sorted descending by similarity, then by file_path for ties
        assert_eq!(packet.relevant_decisions[0].similarity, 0.9);
        assert_eq!(
            packet.relevant_decisions[0].file_path,
            PathBuf::from("docs/a.md")
        );
        // Tie at 0.5: b.md < c.md alphabetically
        assert_eq!(packet.relevant_decisions[1].similarity, 0.5);
        assert_eq!(
            packet.relevant_decisions[1].file_path,
            PathBuf::from("docs/b.md")
        );
        assert_eq!(packet.relevant_decisions[2].similarity, 0.5);
        assert_eq!(
            packet.relevant_decisions[2].file_path,
            PathBuf::from("docs/c.md")
        );
    }

    #[test]
    fn test_truncate_for_context_clears_relevant_decisions() {
        let mut packet = ImpactPacket {
            changes: vec![ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            }],
            relevant_decisions: vec![RelevantDecision {
                file_path: PathBuf::from("docs/a.md"),
                heading: Some("Intro".to_string()),
                excerpt: "Content".to_string(),
                similarity: 0.9,
                rerank_score: None,
                staleness_days: None,
            }],
            ..ImpactPacket::default()
        };

        // Truncate with a very small target to force Phase 3 clearing
        let truncated = packet.truncate_for_context(100);
        assert!(truncated);
        assert!(packet.relevant_decisions.is_empty());
    }

    #[test]
    fn test_observability_sorted_by_severity_in_finalize() {
        use crate::observability::signal::{ObservabilitySignal, SignalSeverity};

        let mut packet = ImpactPacket {
            observability: vec![
                ObservabilitySignal::new(
                    "metric",
                    "label-a",
                    1.0,
                    SignalSeverity::Normal,
                    "normal",
                    "source",
                ),
                ObservabilitySignal::new(
                    "metric",
                    "label-b",
                    1.0,
                    SignalSeverity::Critical,
                    "critical",
                    "source",
                ),
                ObservabilitySignal::new(
                    "metric",
                    "label-c",
                    1.0,
                    SignalSeverity::Warning,
                    "warning",
                    "source",
                ),
            ],
            ..ImpactPacket::default()
        };

        packet.finalize();

        assert_eq!(packet.observability[0].severity, SignalSeverity::Critical);
        assert_eq!(packet.observability[1].severity, SignalSeverity::Warning);
        assert_eq!(packet.observability[2].severity, SignalSeverity::Normal);
    }

    #[test]
    fn test_observability_cleared_in_truncate_phase_3() {
        use crate::observability::signal::{ObservabilitySignal, SignalSeverity};

        let mut packet = ImpactPacket {
            changes: vec![ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            }],
            observability: vec![ObservabilitySignal::new(
                "error_rate",
                "svc",
                0.15,
                SignalSeverity::Critical,
                "Error rate high",
                "prometheus",
            )],
            temporal_couplings: vec![TemporalCoupling {
                file_a: PathBuf::from("src/a.rs"),
                file_b: PathBuf::from("src/b.rs"),
                score: 0.9,
            }],
            ..ImpactPacket::default()
        };

        // Truncate with very small target to push through to Phase 3
        let truncated = packet.truncate_for_context(100);
        assert!(truncated);
        assert!(packet.observability.is_empty());
    }

    #[test]
    fn test_observability_serialization_roundtrip() {
        use crate::observability::signal::{ObservabilitySignal, SignalSeverity};

        let packet = ImpactPacket {
            observability: vec![ObservabilitySignal::new(
                "error_rate",
                "GET /api",
                0.15,
                SignalSeverity::Critical,
                "Error rate 15%",
                "prometheus",
            )],
            changes: vec![ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            }],
            ..ImpactPacket::default()
        };

        let json = serde_json::to_string_pretty(&packet).unwrap();
        assert!(json.contains("observability"));
        assert!(json.contains("Error rate 15%"));

        let parsed: ImpactPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.observability.len(), 1);
        assert_eq!(parsed.observability[0].signal_type, "error_rate");
        assert_eq!(parsed.observability[0].severity, SignalSeverity::Critical);
    }

    #[test]
    fn test_observability_empty_absent_from_json() {
        let packet = ImpactPacket::default();
        let json = serde_json::to_string_pretty(&packet).unwrap();
        assert!(!json.contains("observability"));
    }

    #[test]
    fn test_affected_contracts_serialization_roundtrip() {
        let packet = ImpactPacket {
            affected_contracts: vec![AffectedContract {
                endpoint_id: "api/openapi.json::GET::/pets".to_string(),
                path: "/pets".to_string(),
                method: "GET".to_string(),
                summary: "List all pets".to_string(),
                similarity: 0.85,
                spec_file: "api/openapi.json".to_string(),
            }],
            ..ImpactPacket::default()
        };

        let json = serde_json::to_string_pretty(&packet).unwrap();
        assert!(json.contains("affectedContracts"));
        assert!(json.contains("/pets"));
        assert!(json.contains("GET"));

        let parsed: ImpactPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.affected_contracts.len(), 1);
        assert_eq!(parsed.affected_contracts[0].path, "/pets");
        assert_eq!(parsed.affected_contracts[0].method, "GET");
        assert!((parsed.affected_contracts[0].similarity - 0.85).abs() < 1e-6);
    }

    #[test]
    fn test_affected_contracts_empty_absent_from_json() {
        let packet = ImpactPacket::default();
        let json = serde_json::to_string_pretty(&packet).unwrap();
        assert!(!json.contains("affectedContracts"));
    }

    #[test]
    fn test_finalize_sorts_affected_contracts() {
        let mut packet = ImpactPacket {
            affected_contracts: vec![
                AffectedContract {
                    endpoint_id: "c".to_string(),
                    path: "/pets".to_string(),
                    method: "GET".to_string(),
                    summary: "".to_string(),
                    similarity: 0.5,
                    spec_file: "api.yaml".to_string(),
                },
                AffectedContract {
                    endpoint_id: "a".to_string(),
                    path: "/users".to_string(),
                    method: "POST".to_string(),
                    summary: "".to_string(),
                    similarity: 0.9,
                    spec_file: "api.yaml".to_string(),
                },
                AffectedContract {
                    endpoint_id: "b".to_string(),
                    path: "/items".to_string(),
                    method: "GET".to_string(),
                    summary: "".to_string(),
                    similarity: 0.5,
                    spec_file: "api.yaml".to_string(),
                },
            ],
            ..ImpactPacket::default()
        };

        packet.finalize();

        assert_eq!(packet.affected_contracts[0].similarity, 0.9);
        assert_eq!(packet.affected_contracts[1].similarity, 0.5);
        assert_eq!(packet.affected_contracts[2].similarity, 0.5);
        // Ties sorted by path ascending
        assert_eq!(packet.affected_contracts[1].path, "/items");
        assert_eq!(packet.affected_contracts[2].path, "/pets");
    }

    #[test]
    fn test_truncate_clears_affected_contracts() {
        let mut packet = ImpactPacket {
            changes: vec![ChangedFile {
                path: PathBuf::from("src/main.rs"),
                status: "Modified".to_string(),
                old_path: None,
                is_staged: true,
                symbols: None,
                imports: None,
                runtime_usage: None,
                analysis_status: FileAnalysisStatus::default(),
                analysis_warnings: Vec::new(),
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            }],
            affected_contracts: vec![AffectedContract {
                endpoint_id: "a".to_string(),
                path: "/pets".to_string(),
                method: "GET".to_string(),
                summary: "List pets".to_string(),
                similarity: 0.9,
                spec_file: "api.yaml".to_string(),
            }],
            temporal_couplings: vec![TemporalCoupling {
                file_a: PathBuf::from("src/a.rs"),
                file_b: PathBuf::from("src/b.rs"),
                score: 0.9,
            }],
            ..ImpactPacket::default()
        };

        let truncated = packet.truncate_for_context(100);
        assert!(truncated);
        assert!(packet.affected_contracts.is_empty());
    }
}
