use crate::config::error::ConfigError;
use crate::config::model::{Config, GeminiConfig};
use miette::Result;

/// Validates the configuration, returning an error for invalid values.
pub fn validate_config(config: &Config) -> Result<()> {
    if config.watch.debounce_ms == 0 {
        return Err(ConfigError::ValidationFailed {
            reason: "watch.debounce_ms must be > 0".to_string(),
        }
        .into());
    }

    if let Some(0) = config.gemini.timeout_secs {
        return Err(ConfigError::ValidationFailed {
            reason: "gemini.timeout_secs must be > 0".to_string(),
        }
        .into());
    }

    validate_optional_model(&config.gemini, "model")?;
    validate_optional_model(&config.gemini, "fast_model")?;
    validate_optional_model(&config.gemini, "deep_model")?;

    for pattern in &config.watch.ignore_patterns {
        if globset::Glob::new(pattern).is_err() {
            return Err(ConfigError::ValidationFailed {
                reason: format!("watch.ignore_patterns contains invalid glob: '{}'", pattern),
            }
            .into());
        }
    }

    // Validate temporal config
    if config.temporal.min_shared_commits == 0 {
        return Err(ConfigError::ValidationFailed {
            reason: "temporal.min_shared_commits must be > 0".to_string(),
        }
        .into());
    }
    if config.temporal.min_revisions == 0 {
        return Err(ConfigError::ValidationFailed {
            reason: "temporal.min_revisions must be > 0".to_string(),
        }
        .into());
    }

    // Validate verify steps
    for (i, step) in config.verify.steps.iter().enumerate() {
        if step.command.trim().is_empty() {
            return Err(ConfigError::ValidationFailed {
                reason: format!("verify.steps[{}] has empty command", i),
            }
            .into());
        }
        if step.timeout_secs == Some(0) {
            return Err(ConfigError::ValidationFailed {
                reason: format!("verify.steps[{}] timeout_secs must be > 0", i),
            }
            .into());
        }
    }
    if config.verify.default_timeout_secs == 0 && !config.verify.steps.is_empty() {
        return Err(ConfigError::ValidationFailed {
            reason: "verify.default_timeout_secs must be > 0 when steps are defined".to_string(),
        }
        .into());
    }

    if config.verify.semantic_weight < 0.0 || config.verify.semantic_weight > 1.0 {
        return Err(ConfigError::ValidationFailed {
            reason: format!(
                "verify.semantic_weight must be in [0.0, 1.0], got {}",
                config.verify.semantic_weight
            ),
        }
        .into());
    }

    Ok(())
}

fn validate_optional_model(config: &GeminiConfig, field: &str) -> Result<()> {
    let value = match field {
        "model" => &config.model,
        "fast_model" => &config.fast_model,
        "deep_model" => &config.deep_model,
        _ => return Ok(()),
    };

    if let Some(model) = value
        && model.trim().is_empty()
    {
        return Err(ConfigError::ValidationFailed {
            reason: format!("gemini.{field} must be non-empty if present"),
        }
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::*;

    #[test]
    fn test_valid_default_config() {
        let config = Config::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_zero_debounce_ms() {
        let config = Config {
            watch: WatchConfig {
                debounce_ms: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("debounce_ms"));
    }

    #[test]
    fn test_zero_timeout_secs() {
        let config = Config {
            gemini: GeminiConfig {
                timeout_secs: Some(0),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("timeout_secs"));
    }

    #[test]
    fn test_empty_model() {
        let config = Config {
            gemini: GeminiConfig {
                model: Some("   ".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("model"));
    }

    #[test]
    fn test_valid_model() {
        let config = Config {
            gemini: GeminiConfig {
                model: Some("gemini-pro".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_empty_routed_model() {
        let config = Config {
            gemini: GeminiConfig {
                fast_model: Some("   ".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("fast_model"));
    }

    #[test]
    fn test_invalid_glob_pattern() {
        let config = Config {
            watch: WatchConfig {
                ignore_patterns: vec!["valid/**".to_string(), "[invalid".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("invalid glob"));
    }

    #[test]
    fn test_none_timeout_is_ok() {
        let config = Config {
            gemini: GeminiConfig {
                timeout_secs: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_none_model_is_ok() {
        let config = Config {
            gemini: GeminiConfig {
                model: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_verify_empty_steps_ok() {
        let config = Config {
            verify: VerifyConfig {
                steps: vec![],
                default_timeout_secs: 300,
                semantic_weight: 0.3,
            },
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_verify_empty_command_fails() {
        let config = Config {
            verify: VerifyConfig {
                steps: vec![VerifyStep {
                    description: "Missing command".to_string(),
                    command: "   ".to_string(),
                    timeout_secs: Some(60),
                }],
                default_timeout_secs: 300,
                semantic_weight: 0.3,
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("verify.steps[0]"));
        assert!(msg.contains("empty command"));
    }

    #[test]
    fn test_verify_zero_timeout_step_fails() {
        let config = Config {
            verify: VerifyConfig {
                steps: vec![VerifyStep {
                    description: "Bad timeout".to_string(),
                    command: "cargo test".to_string(),
                    timeout_secs: Some(0),
                }],
                default_timeout_secs: 300,
                semantic_weight: 0.3,
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("verify.steps[0]"));
        assert!(msg.contains("timeout_secs must be > 0"));
    }

    #[test]
    fn test_verify_zero_default_timeout_with_steps_fails() {
        let config = Config {
            verify: VerifyConfig {
                steps: vec![VerifyStep {
                    description: "Run tests".to_string(),
                    command: "cargo test".to_string(),
                    timeout_secs: Some(60),
                }],
                default_timeout_secs: 0,
                semantic_weight: 0.3,
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("default_timeout_secs"));
    }

    #[test]
    fn test_temporal_zero_min_shared_commits_fails() {
        let config = Config {
            temporal: TemporalConfig {
                min_shared_commits: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("min_shared_commits"));
    }

    #[test]
    fn test_temporal_zero_min_revisions_fails() {
        let config = Config {
            temporal: TemporalConfig {
                min_revisions: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("min_revisions"));
    }

    #[test]
    fn test_semantic_weight_out_of_range_rejected() {
        let config = Config {
            verify: VerifyConfig {
                semantic_weight: 1.5,
                ..Default::default()
            },
            ..Default::default()
        };
        let err = validate_config(&config).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(msg.contains("semantic_weight"));
    }

    #[test]
    fn test_semantic_weight_in_range_accepted() {
        let config = Config {
            verify: VerifyConfig {
                semantic_weight: 0.5,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
    }
}
