use std::fs;
use miette::Result;
use crate::policy::rules::Rules;
use crate::policy::error::PolicyError;
use crate::state::layout::Layout;

/// Loads the rules from the workspace root.
/// If the rules file does not exist, it returns the default rules.
pub fn load_rules(layout: &Layout) -> Result<Rules> {
    let path = layout.rules_file();
    
    if !path.exists() {
        return Ok(Rules::default());
    }

    let content = fs::read_to_string(&path).map_err(|e| PolicyError::ReadFailed {
        path: path.to_string(),
        source: e,
    })?;

    let rules: Rules = toml::from_str(&content).map_err(|e| PolicyError::ParseFailed {
        source: e,
    })?;

    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use camino::Utf8Path;
    use crate::policy::mode::Mode;

    #[test]
    fn test_load_default_rules_if_missing() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        
        let rules = load_rules(&layout).unwrap();
        assert_eq!(rules.global.mode, Mode::Analyze);
    }

    #[test]
    fn test_load_custom_rules() {
        let tmp = tempdir().unwrap();
        let root = Utf8Path::from_path(tmp.path()).unwrap();
        let layout = Layout::new(root);
        layout.ensure_state_dir().unwrap();

        let rules_path = layout.rules_file();
        fs::write(rules_path, "[global]\nmode = \"enforce\"").unwrap();

        let rules = load_rules(&layout).unwrap();
        assert_eq!(rules.global.mode, Mode::Enforce);
    }
}
