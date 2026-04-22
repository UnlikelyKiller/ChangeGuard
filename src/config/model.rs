use serde::{Deserialize, Serialize};

pub type TomlError = toml::de::Error;
pub const DEFAULT_GEMINI_FAST_MODEL: &str = "gemini-3.1-flash-lite-preview";
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
}

fn default_verify_timeout() -> u64 {
    300
}

impl Default for VerifyConfig {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            default_timeout_secs: default_verify_timeout(),
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
    }

    #[test]
    fn test_verify_config_defaults() {
        let config = Config::default();
        assert!(config.verify.steps.is_empty());
        assert_eq!(config.verify.default_timeout_secs, 300);
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
}
