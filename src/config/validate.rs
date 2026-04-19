use miette::Result;
use crate::config::model::Config;

/// Validates the configuration.
pub fn validate_config(_config: &Config) -> Result<()> {
    // Currently, there are no complex validation rules for config.
    // In the future, we might validate API keys, model names, etc.
    Ok(())
}
