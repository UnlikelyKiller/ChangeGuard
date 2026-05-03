use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type TomlError = toml::de::Error;
pub const DEFAULT_GEMINI_FAST_MODEL: &str = "gemini-3-flash-preview";
pub const DEFAULT_GEMINI_DEEP_MODEL: &str = "gemini-3.1-pro-preview";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VerifyStep {
    /// Human-readable description of what this step verifies
    pub description: String,
    /// The shell command to execute
    pub command: String,
    /// Per-step timeout in seconds. None means use verify.default_timeout_secs.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifyConfig {
    /// Ordered list of verification steps to run when no `-c` flag is provided
    #[serde(default)]
    pub steps: Vec<VerifyStep>,
    /// Default timeout for steps that don't specify one
    #[serde(default = "default_verify_timeout")]
    pub default_timeout_secs: u64,
    /// Weight of semantic prediction in score blending [0.0, 1.0]. 0.0 disables.
    #[serde(default = "default_semantic_weight")]
    pub semantic_weight: f64,
}

fn default_semantic_weight() -> f64 {
    0.3
}

fn default_verify_timeout() -> u64 {
    300
}

impl Default for VerifyConfig {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            default_timeout_secs: default_verify_timeout(),
            semantic_weight: default_semantic_weight(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LedgerConfig {
    /// Enable tech stack enforcement at transaction start
    #[serde(default)]
    pub enforcement_enabled: bool,

    /// Require verification pass before commit for high-risk categories
    #[serde(default)]
    pub verify_to_commit: bool,

    /// Auto-reconcile watcher drift for the same entity at commit time
    #[serde(default = "default_auto_reconcile")]
    pub auto_reconcile: bool,

    /// Roll back PENDING transactions older than this many hours
    #[serde(default = "default_stale_threshold_hours")]
    pub stale_threshold_hours: u64,

    /// Category-to-stack mappings (defined in config, not just DB)
    #[serde(default)]
    pub category_mappings: Vec<CategoryMapping>,

    /// Watcher patterns for drift detection (supplements hardcoded list)
    #[serde(default)]
    pub watcher_patterns: Vec<WatcherPattern>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryMapping {
    pub ledger_category: String,
    pub stack_category: String,
    pub glob: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatcherPattern {
    pub glob: String,
    pub category: String,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            enforcement_enabled: false,
            verify_to_commit: false,
            auto_reconcile: default_auto_reconcile(),
            stale_threshold_hours: default_stale_threshold_hours(),
            category_mappings: Vec::new(),
            watcher_patterns: Vec::new(),
        }
    }
}

fn default_auto_reconcile() -> bool {
    true
}

fn default_stale_threshold_hours() -> u64 {
    24
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub watch: WatchConfig,
    #[serde(default)]
    pub gemini: GeminiConfig,
    #[serde(default)]
    pub temporal: TemporalConfig,
    #[serde(default)]
    pub hotspots: HotspotsConfig,
    #[serde(default)]
    pub verify: VerifyConfig,
    #[serde(default)]
    pub ledger: LedgerConfig,
    #[serde(default)]
    pub local_model: LocalModelConfig,
    #[serde(default)]
    pub docs: DocsConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub contracts: ContractsConfig,
    #[serde(default)]
    pub coverage: CoverageConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotspotsConfig {
    #[serde(default = "default_hotspots_max_commits")]
    pub max_commits: usize,
    #[serde(default = "default_hotspots_limit")]
    pub limit: usize,
}

impl Default for HotspotsConfig {
    fn default() -> Self {
        Self {
            max_commits: default_hotspots_max_commits(),
            limit: default_hotspots_limit(),
        }
    }
}

fn default_hotspots_max_commits() -> usize {
    500
}
fn default_hotspots_limit() -> usize {
    10
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemporalConfig {
    #[serde(default = "default_max_commits")]
    pub max_commits: usize,
    #[serde(default = "default_max_files_per_commit")]
    pub max_files_per_commit: usize,
    #[serde(default = "default_coupling_threshold")]
    pub coupling_threshold: f32,
    #[serde(default = "default_all_parents")]
    pub all_parents: bool,
    /// Minimum number of commits two files must share to be considered coupled
    #[serde(default = "default_min_shared_commits")]
    pub min_shared_commits: usize,
    /// Minimum number of commits a file must appear in to be eligible for coupling
    #[serde(default = "default_min_revisions")]
    pub min_revisions: usize,
    /// Half-life for exponential decay (in commits). Recent commits weighted higher.
    #[serde(default = "default_decay_half_life")]
    pub decay_half_life: usize,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            max_commits: default_max_commits(),
            max_files_per_commit: default_max_files_per_commit(),
            coupling_threshold: default_coupling_threshold(),
            all_parents: default_all_parents(),
            min_shared_commits: default_min_shared_commits(),
            min_revisions: default_min_revisions(),
            decay_half_life: default_decay_half_life(),
        }
    }
}

fn default_max_commits() -> usize {
    1000
}
fn default_max_files_per_commit() -> usize {
    50
}
fn default_coupling_threshold() -> f32 {
    0.75
}
fn default_all_parents() -> bool {
    false
}

fn default_min_shared_commits() -> usize {
    3
}
fn default_min_revisions() -> usize {
    5
}
fn default_decay_half_life() -> usize {
    100
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    #[serde(default = "default_strict")]
    pub strict: bool,
    #[serde(default = "default_auto_fix")]
    pub auto_fix: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            strict: default_strict(),
            auto_fix: default_auto_fix(),
        }
    }
}

fn default_strict() -> bool {
    false
}
fn default_auto_fix() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WatchConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_ignore_patterns")]
    pub ignore_patterns: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: default_debounce_ms(),
            ignore_patterns: default_ignore_patterns(),
        }
    }
}

fn default_debounce_ms() -> u64 {
    1000
}
fn default_ignore_patterns() -> Vec<String> {
    vec![
        "target/**".to_string(),
        ".git/**".to_string(),
        "node_modules/**".to_string(),
    ]
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GeminiConfig {
    pub api_key: Option<String>,
    /// Optional override used for every Gemini request.
    pub model: Option<String>,
    /// Default for routine, low-latency ChangeGuard ask modes.
    pub fast_model: Option<String>,
    /// Default for high-risk or review-heavy ChangeGuard ask modes.
    pub deep_model: Option<String>,
    pub timeout_secs: Option<u64>,
    #[serde(default = "default_context_window")]
    pub context_window: usize,
}

fn default_context_window() -> usize {
    128_000
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalModelConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub embedding_model: String,
    #[serde(default)]
    pub generation_model: String,
    #[serde(default)]
    pub rerank_model: String,
    #[serde(default)]
    pub dimensions: usize,
    #[serde(default = "default_context_window_local")]
    pub context_window: usize,
    #[serde(default = "default_local_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub prefer_local: bool,
}

fn default_context_window_local() -> usize {
    38000
}
fn default_local_timeout() -> u64 {
    60
}

impl Default for LocalModelConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            embedding_model: String::new(),
            generation_model: String::new(),
            rerank_model: String::new(),
            dimensions: 0,
            context_window: default_context_window_local(),
            timeout_secs: default_local_timeout(),
            prefer_local: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocsConfig {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default = "default_chunk_tokens")]
    pub chunk_tokens: usize,
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: usize,
    #[serde(default = "default_retrieval_top_k")]
    pub retrieval_top_k: usize,
}

fn default_chunk_tokens() -> usize {
    512
}
fn default_chunk_overlap() -> usize {
    64
}
fn default_retrieval_top_k() -> usize {
    5
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            chunk_tokens: default_chunk_tokens(),
            chunk_overlap: default_chunk_overlap(),
            retrieval_top_k: default_retrieval_top_k(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub prometheus_url: String,
    #[serde(default)]
    pub service_map: HashMap<String, String>,
    #[serde(default)]
    pub log_paths: Vec<String>,
    #[serde(default = "default_error_rate_threshold")]
    pub error_rate_threshold: f32,
    #[serde(default = "default_log_lookback_secs")]
    pub log_lookback_secs: u64,
}

fn default_error_rate_threshold() -> f32 {
    0.05
}
fn default_log_lookback_secs() -> u64 {
    3600
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            prometheus_url: String::new(),
            service_map: HashMap::new(),
            log_paths: Vec::new(),
            error_rate_threshold: default_error_rate_threshold(),
            log_lookback_secs: default_log_lookback_secs(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractsConfig {
    #[serde(default)]
    pub spec_paths: Vec<String>,
    #[serde(default = "default_match_threshold")]
    pub match_threshold: f32,
}

fn default_match_threshold() -> f32 {
    0.5
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CoverageConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub traces: TracesConfig,
    #[serde(default)]
    pub sdk: SdkConfig,
    #[serde(default)]
    pub services: ServicesConfig,
    #[serde(default)]
    pub data_flow: DataFlowConfig,
    #[serde(default)]
    pub deploy: DeployConfig,
    #[serde(default)]
    pub ci_self_awareness: CiSelfAwarenessConfig,
    #[serde(default)]
    pub adr_staleness: AdrStalenessConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TracesConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_trace_config_patterns")]
    pub config_patterns: Vec<String>,
    #[serde(default = "default_trace_env_var_patterns")]
    pub env_var_patterns: Vec<String>,
    #[serde(default = "default_exclude_env_patterns")]
    pub exclude_env_patterns: Vec<String>,
    #[serde(default = "default_trace_risk_weight_per_config")]
    pub risk_weight_per_config_file: u32,
    #[serde(default = "default_trace_risk_weight_per_env")]
    pub risk_weight_per_env_var: u32,
    #[serde(default = "default_trace_risk_cap")]
    pub risk_cap: u32,
}

fn default_trace_risk_weight_per_config() -> u32 {
    3
}
fn default_trace_risk_weight_per_env() -> u32 {
    2
}
fn default_trace_risk_cap() -> u32 {
    10
}

fn default_trace_config_patterns() -> Vec<String> {
    vec![
        "**/otel*.yaml".to_string(),
        "**/jaeger*.yaml".to_string(),
        "**/datadog*.yaml".to_string(),
    ]
}

fn default_trace_env_var_patterns() -> Vec<String> {
    vec![
        "OTEL_*".to_string(),
        "JAEGER_*".to_string(),
        "DD_*".to_string(),
        "OTLP_*".to_string(),
    ]
}

fn default_exclude_env_patterns() -> Vec<String> {
    vec!["OTEL_SDK_DISABLED".to_string()]
}

impl Default for TracesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            config_patterns: default_trace_config_patterns(),
            env_var_patterns: default_trace_env_var_patterns(),
            exclude_env_patterns: default_exclude_env_patterns(),
            risk_weight_per_config_file: default_trace_risk_weight_per_config(),
            risk_weight_per_env_var: default_trace_risk_weight_per_env(),
            risk_cap: default_trace_risk_cap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SdkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_sdk_patterns")]
    pub patterns: Vec<String>,
    #[serde(default = "default_sdk_risk_weight_new")]
    pub risk_weight_new: u32,
    #[serde(default = "default_sdk_risk_weight_modified")]
    pub risk_weight_modified: u32,
    #[serde(default = "default_sdk_risk_cap")]
    pub risk_cap: u32,
}

fn default_sdk_risk_cap() -> u32 {
    10
}

fn default_sdk_patterns() -> Vec<String> {
    vec![
        "stripe".to_string(),
        "auth0".to_string(),
        "twilio".to_string(),
        "sendgrid".to_string(),
        "openai".to_string(),
        "anthropic".to_string(),
    ]
}

fn default_sdk_risk_weight_new() -> u32 {
    5
}

fn default_sdk_risk_weight_modified() -> u32 {
    2
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            patterns: default_sdk_patterns(),
            risk_weight_new: default_sdk_risk_weight_new(),
            risk_weight_modified: default_sdk_risk_weight_modified(),
            risk_cap: default_sdk_risk_cap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServicesConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_cross_service_elevation_threshold")]
    pub cross_service_elevation_threshold: u32,
    #[serde(default = "default_svc_risk_5plus")]
    pub risk_weight_5plus: u32,
    #[serde(default = "default_svc_risk_3to4")]
    pub risk_weight_3to4: u32,
    #[serde(default = "default_svc_risk_2svcs")]
    pub risk_weight_2svcs: u32,
}

fn default_svc_risk_5plus() -> u32 {
    15
}
fn default_svc_risk_3to4() -> u32 {
    8
}
fn default_svc_risk_2svcs() -> u32 {
    3
}

fn default_cross_service_elevation_threshold() -> u32 {
    2
}

impl Default for ServicesConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cross_service_elevation_threshold: default_cross_service_elevation_threshold(),
            risk_weight_5plus: default_svc_risk_5plus(),
            risk_weight_3to4: default_svc_risk_3to4(),
            risk_weight_2svcs: default_svc_risk_2svcs(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataFlowConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_chain_depth_max")]
    pub chain_depth_max: u32,
    #[serde(default = "default_dataflow_risk_per_match")]
    pub risk_weight_per_match: u32,
    #[serde(default = "default_dataflow_risk_cap")]
    pub risk_cap: u32,
}

fn default_dataflow_risk_per_match() -> u32 {
    4
}
fn default_dataflow_risk_cap() -> u32 {
    12
}

fn default_chain_depth_max() -> u32 {
    5
}

impl Default for DataFlowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            chain_depth_max: default_chain_depth_max(),
            risk_weight_per_match: default_dataflow_risk_per_match(),
            risk_cap: default_dataflow_risk_cap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_deploy_patterns")]
    pub patterns: Vec<String>,
    #[serde(default = "default_deploy_risk_weight_per_manifest")]
    pub risk_weight_per_manifest: u32,
    #[serde(default = "default_deploy_risk_cap")]
    pub risk_cap: u32,
}

fn default_deploy_patterns() -> Vec<String> {
    vec![
        "**/Dockerfile*".to_string(),
        "**/docker-compose*.yml".to_string(),
        "**/*.tf".to_string(),
        "**/k8s/**/*.yaml".to_string(),
    ]
}

fn default_deploy_risk_weight_per_manifest() -> u32 {
    3
}

fn default_deploy_risk_cap() -> u32 {
    15
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            patterns: default_deploy_patterns(),
            risk_weight_per_manifest: default_deploy_risk_weight_per_manifest(),
            risk_cap: default_deploy_risk_cap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CiSelfAwarenessConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ci_changed_weight")]
    pub ci_changed_weight: u32,
    #[serde(default = "default_ci_plus_source_weight")]
    pub ci_plus_source_weight: u32,
}

fn default_ci_changed_weight() -> u32 {
    3
}

fn default_ci_plus_source_weight() -> u32 {
    5
}

impl Default for CiSelfAwarenessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ci_changed_weight: default_ci_changed_weight(),
            ci_plus_source_weight: default_ci_plus_source_weight(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdrStalenessConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_staleness_threshold_days")]
    pub threshold_days: u32,
}

fn default_staleness_threshold_days() -> u32 {
    365
}

impl Default for AdrStalenessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_days: default_staleness_threshold_days(),
        }
    }
}

impl Default for ContractsConfig {
    fn default() -> Self {
        Self {
            spec_paths: Vec::new(),
            match_threshold: default_match_threshold(),
        }
    }
}

pub fn resolve_local_model_config(config: &LocalModelConfig) -> LocalModelConfig {
    resolve_local_model_config_with(config, &|name| std::env::var(name).ok(), &|name| {
        read_env_key(name)
    })
}

fn resolve_local_model_config_with(
    config: &LocalModelConfig,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> LocalModelConfig {
    let mut resolved = config.clone();

    let resolve_string = |configured: &str, env_var: &str| -> String {
        if !configured.is_empty() {
            return configured.to_string();
        }
        if let Some(val) = env_reader(env_var)
            && !val.trim().is_empty()
        {
            return val.trim().to_string();
        }
        if let Some(val) = dotenv_reader(env_var) {
            return val;
        }
        String::new()
    };

    let resolve_usize = |configured: usize, env_var: &str| -> usize {
        if configured != 0 {
            return configured;
        }
        if let Some(val) = env_reader(env_var)
            && let Ok(parsed) = val.trim().parse::<usize>()
        {
            return parsed;
        }
        if let Some(val) = dotenv_reader(env_var)
            && let Ok(parsed) = val.parse::<usize>()
        {
            return parsed;
        }
        0
    };

    resolved.base_url = resolve_string(&config.base_url, "CHANGEGUARD_LOCAL_MODEL_URL");
    resolved.embedding_model =
        resolve_string(&config.embedding_model, "CHANGEGUARD_EMBEDDING_MODEL");
    resolved.generation_model =
        resolve_string(&config.generation_model, "CHANGEGUARD_GENERATION_MODEL");
    resolved.rerank_model = resolve_string(&config.rerank_model, "CHANGEGUARD_RERANK_MODEL");
    resolved.dimensions = resolve_usize(config.dimensions, "CHANGEGUARD_EMBEDDING_DIMENSIONS");

    resolved
}

pub(crate) fn read_env_key(target_key: &str) -> Option<String> {
    use std::path::Path;
    let path = Path::new(".env");
    let contents = std::fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim().strip_prefix("export ").unwrap_or(key.trim());
        if key != target_key {
            continue;
        }
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert!(!config.core.strict);
        assert_eq!(config.watch.debounce_ms, 1000);
        assert!(
            config
                .watch
                .ignore_patterns
                .contains(&"target/**".to_string())
        );
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [core]
            strict = true
            [watch]
            debounce_ms = 500
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.core.strict);
        assert_eq!(config.watch.debounce_ms, 500);
    }

    #[test]
    fn test_temporal_config_deserialization() {
        let toml_str = r#"
            [temporal]
            max_commits = 500
            max_files_per_commit = 30
            coupling_threshold = 0.5
            min_shared_commits = 4
            min_revisions = 8
            decay_half_life = 50
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.temporal.max_commits, 500);
        assert_eq!(config.temporal.max_files_per_commit, 30);
        assert!((config.temporal.coupling_threshold - 0.5).abs() < f32::EPSILON);
        assert_eq!(config.temporal.min_shared_commits, 4);
        assert_eq!(config.temporal.min_revisions, 8);
        assert_eq!(config.temporal.decay_half_life, 50);
    }

    #[test]
    fn test_temporal_config_defaults() {
        let config = TemporalConfig::default();
        assert_eq!(config.min_shared_commits, 3);
        assert_eq!(config.min_revisions, 5);
        assert_eq!(config.decay_half_life, 100);
    }

    #[test]
    fn test_verify_config_deserialization() {
        let toml_str = r#"
            [verify]
            default_timeout_secs = 120

            [[verify.steps]]
            description = "Run unit tests"
            command = "cargo test"
            timeout_secs = 60

            [[verify.steps]]
            description = "Check formatting"
            command = "cargo fmt --check"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.verify.default_timeout_secs, 120);
        assert_eq!(config.verify.steps.len(), 2);
        assert_eq!(config.verify.steps[0].description, "Run unit tests");
        assert_eq!(config.verify.steps[0].command, "cargo test");
        assert_eq!(config.verify.steps[0].timeout_secs, Some(60));
        assert_eq!(config.verify.steps[1].description, "Check formatting");
        assert_eq!(config.verify.steps[1].command, "cargo fmt --check");
        // Omitted timeout_secs should deserialize as None (uses default_timeout_secs)
        assert_eq!(config.verify.steps[1].timeout_secs, None);
        // semantic_weight not specified → uses default 0.3
        assert!((config.verify.semantic_weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_verify_config_defaults() {
        let config = Config::default();
        assert!(config.verify.steps.is_empty());
        assert_eq!(config.verify.default_timeout_secs, 300);
        assert!((config.verify.semantic_weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ledger_config_defaults() {
        let config = LedgerConfig::default();
        assert!(!config.enforcement_enabled);
        assert!(!config.verify_to_commit);
        assert!(config.auto_reconcile);
        assert_eq!(config.stale_threshold_hours, 24);
    }

    #[test]
    fn test_ledger_config_deserialization() {
        let toml_str = r#"
            [ledger]
            enforcement_enabled = true
            verify_to_commit = true
            auto_reconcile = false
            stale_threshold_hours = 48

            [[ledger.watcher_patterns]]
            glob = "**/Cargo.toml"
            category = "INFRA"

            [[ledger.category_mappings]]
            ledger_category = "ARCHITECTURE"
            stack_category = "BACKEND_LANG"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.ledger.enforcement_enabled);
        assert!(config.ledger.verify_to_commit);
        assert!(!config.ledger.auto_reconcile);
        assert_eq!(config.ledger.stale_threshold_hours, 48);
        assert_eq!(config.ledger.watcher_patterns.len(), 1);
        assert_eq!(config.ledger.watcher_patterns[0].glob, "**/Cargo.toml");
        assert_eq!(config.ledger.watcher_patterns[0].category, "INFRA");
        assert_eq!(config.ledger.category_mappings.len(), 1);
        assert_eq!(
            config.ledger.category_mappings[0].ledger_category,
            "ARCHITECTURE"
        );
        assert_eq!(
            config.ledger.category_mappings[0].stack_category,
            "BACKEND_LANG"
        );
    }

    #[test]
    fn test_local_model_config_defaults() {
        let config = LocalModelConfig::default();
        assert_eq!(config.base_url, "");
        assert_eq!(config.embedding_model, "");
        assert_eq!(config.generation_model, "");
        assert_eq!(config.rerank_model, "");
        assert_eq!(config.dimensions, 0);
        assert_eq!(config.context_window, 38000);
        assert_eq!(config.timeout_secs, 60);
        assert!(!config.prefer_local);
    }

    #[test]
    fn test_docs_config_defaults() {
        let config = DocsConfig::default();
        assert!(config.include.is_empty());
        assert_eq!(config.chunk_tokens, 512);
        assert_eq!(config.chunk_overlap, 64);
        assert_eq!(config.retrieval_top_k, 5);
    }

    #[test]
    fn test_observability_config_defaults() {
        let config = ObservabilityConfig::default();
        assert_eq!(config.prometheus_url, "");
        assert!(config.service_map.is_empty());
        assert!(config.log_paths.is_empty());
        assert!((config.error_rate_threshold - 0.05).abs() < f32::EPSILON);
        assert_eq!(config.log_lookback_secs, 3600);
    }

    #[test]
    fn test_contracts_config_defaults() {
        let config = ContractsConfig::default();
        assert!(config.spec_paths.is_empty());
        assert!((config.match_threshold - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_includes_new_sections() {
        let config = Config::default();
        assert_eq!(config.local_model.base_url, "");
        assert_eq!(config.docs.chunk_tokens, 512);
        assert_eq!(config.observability.error_rate_threshold, 0.05);
        assert!(config.contracts.spec_paths.is_empty());
    }

    #[test]
    fn test_local_model_config_deserialization() {
        let toml_str = r#"
            [local_model]
            base_url = "http://localhost:11434"
            embedding_model = "nomic-embed-text"
            dimensions = 768
            timeout_secs = 120
            prefer_local = true
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.local_model.base_url, "http://localhost:11434");
        assert_eq!(config.local_model.embedding_model, "nomic-embed-text");
        assert_eq!(config.local_model.dimensions, 768);
        assert_eq!(config.local_model.timeout_secs, 120);
        assert!(config.local_model.prefer_local);
        // Fields not specified should have defaults
        assert_eq!(config.local_model.context_window, 38000);
        assert_eq!(config.local_model.generation_model, "");
    }

    #[test]
    fn test_docs_config_deserialization() {
        let toml_str = r#"
            [docs]
            include = ["README.md", "docs/"]
            chunk_tokens = 1024
            chunk_overlap = 128
            retrieval_top_k = 10
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.docs.include, vec!["README.md", "docs/"]);
        assert_eq!(config.docs.chunk_tokens, 1024);
        assert_eq!(config.docs.chunk_overlap, 128);
        assert_eq!(config.docs.retrieval_top_k, 10);
    }

    #[test]
    fn test_observability_config_deserialization() {
        let toml_str = r#"
            [observability]
            prometheus_url = "http://localhost:9090"
            error_rate_threshold = 0.1
            log_lookback_secs = 7200
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.observability.prometheus_url, "http://localhost:9090");
        assert!((config.observability.error_rate_threshold - 0.1).abs() < f32::EPSILON);
        assert_eq!(config.observability.log_lookback_secs, 7200);
        assert!(config.observability.service_map.is_empty());
        assert!(config.observability.log_paths.is_empty());
    }

    #[test]
    fn test_contracts_config_deserialization() {
        let toml_str = r#"
            [contracts]
            spec_paths = ["openapi.yaml", "proto/"]
            match_threshold = 0.7
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.contracts.spec_paths, vec!["openapi.yaml", "proto/"]);
        assert!((config.contracts.match_threshold - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_coverage_config_deserialization() {
        let toml_str = r#"
            [coverage]
            enabled = true

            [coverage.traces]
            enabled = true
            config_patterns = ["**/otel*.yaml"]
            env_var_patterns = ["OTEL_*"]

            [coverage.sdk]
            enabled = true
            patterns = ["stripe", "auth0"]
            risk_weight_new = 10
            risk_weight_modified = 5
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.coverage.enabled);
        assert!(config.coverage.traces.enabled);
        assert_eq!(
            config.coverage.traces.config_patterns,
            vec!["**/otel*.yaml"]
        );
        assert_eq!(config.coverage.traces.env_var_patterns, vec!["OTEL_*"]);
        assert!(config.coverage.sdk.enabled);
        assert_eq!(config.coverage.sdk.patterns, vec!["stripe", "auth0"]);
        assert_eq!(config.coverage.sdk.risk_weight_new, 10);
        assert_eq!(config.coverage.sdk.risk_weight_modified, 5);
        // Other sections should have defaults
        assert!(!config.coverage.services.enabled);
        assert_eq!(config.coverage.adr_staleness.threshold_days, 365);
    }

    #[test]
    fn test_resolve_local_model_config_env_override() {
        let env_values: std::collections::HashMap<&str, &str> = vec![
            ("CHANGEGUARD_EMBEDDING_MODEL", "test-model-env"),
            ("CHANGEGUARD_EMBEDDING_DIMENSIONS", "384"),
        ]
        .into_iter()
        .collect();

        let env_reader = |name: &str| env_values.get(name).map(|v| v.to_string());
        let dotenv_reader = |_: &str| None::<String>;

        let raw = LocalModelConfig::default();
        let resolved = resolve_local_model_config_with(&raw, &env_reader, &dotenv_reader);

        assert_eq!(resolved.embedding_model, "test-model-env");
        assert_eq!(resolved.dimensions, 384);
        assert_eq!(resolved.base_url, "");
    }

    #[test]
    fn test_resolve_local_model_config_toml_takes_priority() {
        let env_values: std::collections::HashMap<&str, &str> = vec![
            ("CHANGEGUARD_EMBEDDING_MODEL", "env-model"),
            ("CHANGEGUARD_LOCAL_MODEL_URL", "http://env:1234"),
        ]
        .into_iter()
        .collect();

        let env_reader = |name: &str| env_values.get(name).map(|v| v.to_string());
        let dotenv_reader = |_: &str| None::<String>;

        let raw = LocalModelConfig {
            base_url: "http://config:9999".to_string(),
            embedding_model: "config-model".to_string(),
            generation_model: "".to_string(),
            rerank_model: "".to_string(),
            dimensions: 0,
            context_window: 38000,
            timeout_secs: 60,
            prefer_local: false,
        };
        let resolved = resolve_local_model_config_with(&raw, &env_reader, &dotenv_reader);

        assert_eq!(resolved.base_url, "http://config:9999");
        assert_eq!(resolved.embedding_model, "config-model");
    }

    #[test]
    fn test_resolve_local_model_config_generation_model_env() {
        let env_values: std::collections::HashMap<&str, &str> = vec![
            ("CHANGEGUARD_GENERATION_MODEL", "qwen3-9b"),
            ("CHANGEGUARD_RERANK_MODEL", "bge-reranker"),
        ]
        .into_iter()
        .collect();

        let env_reader = |name: &str| env_values.get(name).map(|v| v.to_string());
        let dotenv_reader = |_: &str| None::<String>;

        let raw = LocalModelConfig::default();
        let resolved = resolve_local_model_config_with(&raw, &env_reader, &dotenv_reader);

        assert_eq!(resolved.generation_model, "qwen3-9b");
        assert_eq!(resolved.rerank_model, "bge-reranker");
    }

    #[test]
    fn test_resolve_local_model_config_dimensions_zero_unchanged() {
        let env_values: std::collections::HashMap<&str, &str> =
            vec![("CHANGEGUARD_EMBEDDING_DIMENSIONS", "0")]
                .into_iter()
                .collect();

        let env_reader = |name: &str| env_values.get(name).map(|v| v.to_string());
        let dotenv_reader = |_: &str| None::<String>;

        let raw = LocalModelConfig::default();
        let resolved = resolve_local_model_config_with(&raw, &env_reader, &dotenv_reader);

        assert_eq!(resolved.dimensions, 0);
    }

    #[test]
    fn test_resolve_local_model_config_dotenv_override() {
        let env_reader = |_: &str| None::<String>;
        let dotenv_values: std::collections::HashMap<&str, &str> = vec![
            ("CHANGEGUARD_EMBEDDING_MODEL", "dotenv-model"),
            ("CHANGEGUARD_LOCAL_MODEL_URL", "http://dotenv:5678"),
        ]
        .into_iter()
        .collect();
        let dotenv_reader = |name: &str| dotenv_values.get(name).map(|v| v.to_string());

        let raw = LocalModelConfig::default();
        let resolved = resolve_local_model_config_with(&raw, &env_reader, &dotenv_reader);

        assert_eq!(resolved.embedding_model, "dotenv-model");
        assert_eq!(resolved.base_url, "http://dotenv:5678");
    }
}
