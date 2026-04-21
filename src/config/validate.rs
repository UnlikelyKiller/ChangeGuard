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
}
