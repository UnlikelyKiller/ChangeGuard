use std::fs;
use miette::Result;
use crate::config::model::Config;
use crate::config::error::ConfigError;
use crate::state::layout::Layout;

/// Loads the configuration from the workspace root.
/// If the configuration file does not exist, it returns the default configuration.
pub fn load_config(layout: &Layout) -> Result<Config> {
    let path = layout.config_file();
    
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| ConfigError::ReadFailed {
        path: path.to_string(),
        source: e,
    })?;

    let config: Config = toml::from_str(&content).map_err(|e| ConfigError::ParseFailed {
        source: e,
    })?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use camino::Utf8Path;

    #[test]
    fn test_load_default_config_if_missing() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        
        let config = load_config(&layout).unwrap();
        assert!(!config.core.strict);
    }

    #[test]
    fn test_load_custom_config() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        layout.ensure_state_dir().unwrap();

        let config_path = layout.config_file();
        fs::write(config_path, "[core]\nstrict = true").unwrap();

        let config = load_config(&layout).unwrap();
        assert!(config.core.strict);
    }
}
