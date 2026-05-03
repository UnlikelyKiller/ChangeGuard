use crate::commands::ask::{self, Backend};
use crate::config::load;
use crate::config::model::Config;
use crate::policy::load as policy_load;
use crate::state::layout::Layout;
use miette::Result;

pub fn execute_config_verify() -> Result<()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {e}"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let mut success = true;

    println!("Verifying ChangeGuard configuration...");

    // Verify config.toml
    let config = match load::load_config(&layout) {
        Ok(cfg) => {
            println!("  ✅ config.toml is valid");
            Some(cfg)
        }
        Err(e) => {
            println!("  ❌ config.toml is invalid:\n    {e}");
            success = false;
            None
        }
    };

    // Verify rules.toml
    match policy_load::load_rules(&layout) {
        Ok(_) => {
            println!("  ✅ rules.toml is valid");
        }
        Err(e) => {
            println!("  ❌ rules.toml is invalid:\n    {e}");
            success = false;
        }
    }

    // Report ask backend
    if let Some(ref cfg) = config {
        println!("  {}", format_backend_line(cfg));
    }

    if success {
        println!("\nAll configurations are valid.");
        Ok(())
    } else {
        Err(miette::miette!("Configuration verification failed."))
    }
}

pub(crate) fn format_backend_line(config: &Config) -> String {
    format_backend_line_with(config, &|name| std::env::var(name).ok(), &|name| {
        crate::config::model::read_env_key(name)
    })
}

pub(crate) fn format_backend_line_with(
    config: &Config,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> String {
    let resolved = ask::resolve_backend_with(config, None, env_reader, dotenv_reader);
    match resolved {
        Backend::Gemini => {
            let api_status = if has_gemini_api_key_with(config, env_reader, dotenv_reader) {
                "API key present"
            } else {
                "API key missing"
            };
            if config.local_model.prefer_local {
                format!("Ask backend:   Gemini ({api_status}; prefer_local=true)")
            } else {
                format!("Ask backend:   Gemini ({api_status})")
            }
        }
        Backend::Local => {
            let base_url = if config.local_model.base_url.is_empty() {
                "(not configured)"
            } else {
                config.local_model.base_url.as_str()
            };
            let prefer = if config.local_model.prefer_local {
                ", prefer_local=true"
            } else {
                ""
            };
            format!("Ask backend:   Local ({base_url}{prefer})")
        }
    }
}

fn has_gemini_api_key_with(
    config: &Config,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> bool {
    if config
        .gemini
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty())
    {
        return true;
    }
    if let Some(key) = env_reader("GEMINI_API_KEY")
        && !key.trim().is_empty()
    {
        return true;
    }
    dotenv_reader("GEMINI_API_KEY").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::Config;

    fn empty_env(_name: &str) -> Option<String> {
        None
    }

    #[test]
    fn verify_reports_gemini_backend_with_api_key_in_config() {
        let mut config = Config::default();
        config.gemini.api_key = Some("test-key".to_string());
        let line = format_backend_line_with(&config, &empty_env, &empty_env);
        assert!(line.contains("Gemini"));
        assert!(line.contains("API key present"));
    }

    #[test]
    fn verify_reports_gemini_backend_with_api_key_missing() {
        let config = Config::default();
        let line = format_backend_line_with(&config, &empty_env, &empty_env);
        assert!(line.contains("Gemini"));
        assert!(line.contains("API key missing"));
    }

    #[test]
    fn verify_reports_local_backend_when_configured() {
        let mut config = Config::default();
        config.local_model.base_url = "http://localhost:11434".to_string();
        // No Gemini API key, so auto-select Local
        let line = format_backend_line_with(&config, &empty_env, &empty_env);
        assert!(line.contains("Local"));
        assert!(line.contains("http://localhost:11434"));
    }

    #[test]
    fn verify_reports_local_backend_with_prefer_local() {
        let mut config = Config::default();
        config.local_model.base_url = "http://localhost:8080".to_string();
        config.local_model.prefer_local = true;
        let line = format_backend_line_with(&config, &empty_env, &empty_env);
        assert!(line.contains("Local"));
        assert!(line.contains("http://localhost:8080"));
        assert!(line.contains("prefer_local=true"));
    }

    #[test]
    fn verify_reports_gemini_with_prefer_local() {
        let mut config = Config::default();
        config.gemini.api_key = Some("key".to_string());
        config.local_model.prefer_local = true;
        let line = format_backend_line_with(&config, &empty_env, &empty_env);
        assert!(line.contains("Gemini"));
        assert!(line.contains("prefer_local=true"));
    }
}
