use serde::{Deserialize, Serialize};

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
    /// Prefer `cargo nextest run` over `cargo test` when nextest is installed.
    /// None means true (auto-detect). Set to false to always use cargo test.
    #[serde(default)]
    pub prefer_nextest: Option<bool>,
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
            prefer_nextest: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_config_defaults() {
        let config = VerifyConfig::default();
        assert!(config.steps.is_empty());
        assert_eq!(config.default_timeout_secs, 300);
        assert!((config.semantic_weight - 0.3).abs() < f64::EPSILON);
    }
}
