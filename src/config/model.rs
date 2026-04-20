use serde::{Deserialize, Serialize};

pub type TomlError = toml::de::Error;

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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemporalConfig {
    #[serde(default = "default_max_commits")]
    pub max_commits: usize,
    #[serde(default = "default_max_files_per_commit")]
    pub max_files_per_commit: usize,
    #[serde(default = "default_coupling_threshold")]
    pub coupling_threshold: f32,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            max_commits: default_max_commits(),
            max_files_per_commit: default_max_files_per_commit(),
            coupling_threshold: default_coupling_threshold(),
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
    pub model: Option<String>,
    pub timeout_secs: Option<u64>,
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
}
