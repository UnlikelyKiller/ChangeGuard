use crate::config::ConfigError;
use camino::Utf8PathBuf;
use std::fs;

pub const DEFAULT_CONFIG: &str = r#"[core]
strict = false
auto_fix = false

[watch]
debounce_ms = 1000
ignore_patterns = [
    "target", "target/**", ".git", ".git/**", "node_modules", "node_modules/**",
    ".claude", ".claude/**", ".codex", ".codex/**", ".opencode", ".opencode/**",
    ".agents", ".agents/**", ".changeguard", ".changeguard/**"
]

[temporal]
max_commits = 1000
max_files_per_commit = 50
coupling_threshold = 0.75
min_shared_commits = 3
min_revisions = 5
decay_half_life = 100

[hotspots]
max_commits = 500
limit = 10

# [verify]
# default_timeout_secs = 300
# Steps to run when `changeguard verify` is invoked without -c.
# Each step has a description, command, and optional timeout_secs (defaults to 300).
# [[verify.steps]]
# description = "Run project tests"
# command = "cargo test -j 1 -- --test-threads=1"
# timeout_secs = 300
# [[verify.steps]]
# description = "Check formatting"
# command = "cargo fmt --check"

[gemini]
# Prefer GEMINI_API_KEY in the environment or local .env.
# api_key = "..."
# Optional override for every ask mode:
# model = "gemini-3.1-pro-preview"
fast_model = "gemini-3.1-flash-lite-preview"
deep_model = "gemini-3.1-pro-preview"
timeout_secs = 120
context_window = 128000

[index]
stale_threshold_days = 3

[local_model]
# Use 127.0.0.1 — 'localhost' resolves to ::1 (IPv6) on Windows, which breaks IPv4-only servers
base_url = "http://127.0.0.1:8081"
"#;

pub const DEFAULT_CONFIG_TEMPLATE_ENV: &str = "CHANGEGUARD_DEFAULT_CONFIG";
pub const USER_DEFAULT_CONFIG_FILE: &str = "default-config.toml";

pub fn default_config_contents() -> Result<String, ConfigError> {
    if let Some(path) = default_config_template_path()
        && path.exists()
    {
        return fs::read_to_string(path.as_std_path()).map_err(|source| ConfigError::ReadFailed {
            path: path.to_string(),
            source,
        });
    }

    Ok(DEFAULT_CONFIG.to_string())
}

fn default_config_template_path() -> Option<Utf8PathBuf> {
    if let Some(path) = std::env::var_os(DEFAULT_CONFIG_TEMPLATE_ENV) {
        return Utf8PathBuf::from_path_buf(path.into()).ok();
    }

    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    Utf8PathBuf::from_path_buf(home.into())
        .ok()
        .map(|home| home.join(".changeguard").join(USER_DEFAULT_CONFIG_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_template_uses_127() {
        let config: crate::config::model::Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(config.local_model.base_url, "http://127.0.0.1:8081");
    }

    #[test]
    fn config_template_excludes_agent_dotfiles() {
        let config: crate::config::model::Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        let patterns = &config.watch.ignore_patterns;
        assert!(
            patterns.iter().any(|p| p == ".claude/**"),
            "missing .claude/**"
        );
        assert!(
            patterns.iter().any(|p| p == ".agents/**"),
            "missing .agents/**"
        );
        assert!(
            patterns.iter().any(|p| p == ".codex/**"),
            "missing .codex/**"
        );
        assert!(
            patterns.iter().any(|p| p == ".opencode/**"),
            "missing .opencode/**"
        );
        assert!(
            patterns.iter().any(|p| p == "target/**"),
            "missing target/**"
        );
    }
}
