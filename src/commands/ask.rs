use crate::config::load::load_config;
use crate::config::model::Config;
use crate::config::model::{DEFAULT_GEMINI_DEEP_MODEL, DEFAULT_GEMINI_FAST_MODEL, GeminiConfig};
use crate::contracts::AffectedContract;
use crate::gemini::modes::{GeminiMode, build_system_prompt, build_user_prompt};
use crate::gemini::run_query;
use crate::impact::packet::{ImpactPacket, RelevantDecision, RiskLevel};
use crate::observability::signal::ObservabilitySignal;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Backend {
    Gemini,
    Local,
}

pub fn resolve_backend(config: &Config, explicit: Option<Backend>) -> Backend {
    resolve_backend_with(
        config,
        explicit,
        &|name| std::env::var(name).ok(),
        &|name| crate::config::model::read_env_key(name),
    )
}

pub(crate) fn resolve_backend_with(
    config: &Config,
    explicit: Option<Backend>,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> Backend {
    if let Some(backend) = explicit {
        return backend;
    }

    let local_configured = !config.local_model.base_url.is_empty();

    if config.local_model.prefer_local && local_configured {
        return Backend::Local;
    }

    if !has_gemini_api_key_with(&config.gemini, env_reader, dotenv_reader) && local_configured {
        return Backend::Local;
    }

    Backend::Gemini
}

fn has_gemini_api_key_with(
    config: &GeminiConfig,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> bool {
    if config
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

pub fn execute_ask(
    query: Option<String>,
    mut mode: GeminiMode,
    narrative: bool,
    backend: Option<Backend>,
) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let mut latest_packet = storage.get_latest_packet()?.ok_or_else(|| {
        miette::miette!("No impact report found. Run 'changeguard impact' first.")
    })?;

    if narrative {
        mode = GeminiMode::Narrative;
    }

    // For ReviewPatch, try to get the diff
    let diff = if mode == GeminiMode::ReviewPatch {
        get_working_tree_diff().or_else(|| {
            // Fall back to cached diff if no unstaged changes
            get_cached_diff()
        })
    } else {
        None
    };

    if mode == GeminiMode::ReviewPatch && diff.is_none() {
        println!(
            "{}",
            "Note: No diff available (working tree is clean). Falling back to general analysis."
                .yellow()
        );
    }

    // Read config for settings like timeout
    let config = load_config(&layout)?;
    let resolved = resolve_backend(&config, backend);

    let system_prompt = build_system_prompt(mode);

    let mut user_prompt = if mode == GeminiMode::Narrative {
        crate::gemini::narrative::NarrativeEngine::generate_risk_prompt(&latest_packet)
    } else {
        let effective_query = query.unwrap_or_default();
        build_user_prompt(mode, &latest_packet, &effective_query, diff.as_deref())
    };

    // Inject observability signals if available
    if !latest_packet.observability.is_empty() {
        let obs_block = format_observability_signals(&latest_packet.observability);
        user_prompt = format!("{}\n\n{}", obs_block, user_prompt);
    }

    // Inject affected contracts if available
    if !latest_packet.affected_contracts.is_empty() {
        let contracts_block = format_affected_contracts(&latest_packet.affected_contracts);
        user_prompt = format!("{}\n\n{}", contracts_block, user_prompt);
    }

    // Inject relevant doc decisions into the prompt if available (common to both backends)
    if !latest_packet.relevant_decisions.is_empty() {
        let decisions_block = format_relevant_decisions(&latest_packet.relevant_decisions);

        let combined = format!("{}\n\n{}", decisions_block, user_prompt);
        let combined_len = combined.len();
        let budget_chars = config.local_model.context_window * 4;

        if combined_len > budget_chars {
            // Trim decisions from the end until the combined text fits
            let mut decisions = latest_packet.relevant_decisions.clone();
            let budget_chars = budget_chars.saturating_sub(user_prompt.len() + 4); // +4 for "\n\n" separator
            while decisions.len() > 1 && format_relevant_decisions(&decisions).len() > budget_chars
            {
                decisions.pop();
            }
            tracing::warn!(
                "Doc context budget exceeded ({} chars > {} chars). Trimmed {} decisions.",
                combined_len,
                budget_chars,
                latest_packet.relevant_decisions.len() - decisions.len()
            );
            if !decisions.is_empty() {
                let trimmed_block = format_relevant_decisions(&decisions);
                user_prompt = format!("{}\n\n{}", trimmed_block, user_prompt);
            }
        } else {
            user_prompt = combined;
        }
    }

    match resolved {
        Backend::Local => {
            let max_tokens = config.local_model.context_window;
            let messages = crate::local_model::context::assemble_context(
                &system_prompt,
                &user_prompt,
                &[],
                max_tokens,
            );
            match crate::local_model::client::complete(
                &config.local_model,
                &messages,
                &crate::local_model::client::CompletionOptions::default(),
            ) {
                Ok(response) => {
                    println!("\n{}", "Local Model Response:".bold().green());
                    println!("{response}");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{}", e.red());
                    Err(miette::miette!("Local model failed: {e}"))
                }
            }
        }
        Backend::Gemini => {
            // Token budgeting: derive from config context_window.
            let char_limit = (config.gemini.context_window as f64 * 0.8 * 4.0) as usize;
            let truncated = latest_packet.truncate_for_context(char_limit);

            if truncated {
                user_prompt.push_str("\n\n[Packet truncated for Gemini submission]");
            }

            // The system prompt is static application text. Sanitize only user-supplied/context payload.
            let sanitize_result = crate::gemini::sanitize::sanitize_for_gemini(&user_prompt);
            if !sanitize_result.redactions.is_empty() {
                tracing::warn!(
                    "Sanitized {} secret(s) from prompt before sending to Gemini",
                    sanitize_result.redactions.len()
                );
            }
            if sanitize_result.truncated {
                tracing::warn!(
                    "Prompt truncated from {} bytes",
                    sanitize_result.original_bytes
                );
            }

            let timeout_secs = config.gemini.timeout_secs;
            let model = select_gemini_model(&config.gemini, mode, &latest_packet);

            if let Err(e) = run_query(
                &system_prompt,
                &sanitize_result.sanitized,
                timeout_secs,
                model,
                config.gemini.api_key.as_deref(),
            ) {
                let reports_dir = layout.reports_dir();
                std::fs::create_dir_all(&reports_dir).map_err(|write_err| {
                    miette::miette!(
                        "Gemini execution failed ({e}); additionally failed to create fallback report directory {}: {write_err}",
                        reports_dir
                    )
                })?;
                let fallback_path = reports_dir.join("fallback-impact.json");
                let json = serde_json::to_string_pretty(&latest_packet).map_err(|write_err| {
                    miette::miette!(
                        "Gemini execution failed ({e}); additionally failed to serialize fallback impact packet: {write_err}"
                    )
                })?;
                std::fs::write(&fallback_path, json).map_err(|write_err| {
                    miette::miette!(
                        "Gemini execution failed ({e}); additionally failed to write fallback impact packet to {}: {write_err}",
                        fallback_path
                    )
                })?;
                eprintln!(
                    "{}",
                    format!(
                        "Gemini execution failed. Fallback impact packet saved to {}",
                        fallback_path
                    )
                    .yellow()
                );
                return Err(e);
            }

            Ok(())
        }
    }
}

fn get_working_tree_diff() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "HEAD"])
        .output()
        .ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

fn get_cached_diff() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        Some(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        None
    }
}

pub(crate) fn select_gemini_model<'a>(
    config: &'a GeminiConfig,
    mode: GeminiMode,
    packet: &ImpactPacket,
) -> &'a str {
    if let Some(model) = non_empty(config.model.as_deref()) {
        return model;
    }

    if mode == GeminiMode::ReviewPatch || packet.risk_level == RiskLevel::High {
        return non_empty(config.deep_model.as_deref()).unwrap_or(DEFAULT_GEMINI_DEEP_MODEL);
    }

    non_empty(config.fast_model.as_deref()).unwrap_or(DEFAULT_GEMINI_FAST_MODEL)
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    let value = value?.trim();
    if value.is_empty() { None } else { Some(value) }
}

pub fn format_relevant_decisions(decisions: &[RelevantDecision]) -> String {
    if decisions.is_empty() {
        return String::new();
    }

    let mut out = String::from("## Relevant Architecture Documents\n\n");
    for decision in decisions {
        let heading = decision.heading.as_deref().unwrap_or("(untitled)");
        let file_path = decision.file_path.display();
        out.push_str(&format!("### {heading} ({file_path})\n"));
        out.push_str(&decision.excerpt);
        out.push_str("\n---\n");
    }
    out
}

pub fn format_observability_signals(signals: &[ObservabilitySignal]) -> String {
    if signals.is_empty() {
        return String::new();
    }

    let severity_label = |s: &ObservabilitySignal| match s.severity {
        crate::observability::signal::SignalSeverity::Critical => "CRITICAL",
        crate::observability::signal::SignalSeverity::Warning => "WARNING",
        crate::observability::signal::SignalSeverity::Normal => "NORMAL",
    };

    let mut out = String::from("## Live System Signals\n\n");
    for signal in signals {
        out.push_str(&format!(
            "{type}: {label} [{severity}] — {excerpt}\n",
            type = signal.signal_type,
            label = signal.signal_label,
            severity = severity_label(signal),
            excerpt = signal.excerpt.lines().next().unwrap_or(""),
        ));
    }
    out
}

pub fn format_affected_contracts(contracts: &[AffectedContract]) -> String {
    if contracts.is_empty() {
        return String::new();
    }

    let mut out = String::from("## Affected API Contracts\n\n");
    for contract in contracts {
        out.push_str(&format!(
            "{} {} ({}, similarity={:.2})\n",
            contract.method, contract.path, contract.spec_file, contract.similarity,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::Config;

    fn empty_env(_name: &str) -> Option<String> {
        None
    }

    fn api_key_env(_name: &str) -> Option<String> {
        Some("test-key-123".to_string())
    }

    fn test_config_local_configured() -> Config {
        let mut config = Config::default();
        config.local_model.base_url = "http://localhost:11434".to_string();
        config
    }

    fn test_config_prefer_local() -> Config {
        let mut config = Config::default();
        config.local_model.base_url = "http://localhost:11434".to_string();
        config.local_model.prefer_local = true;
        config
    }

    fn test_config_gemini_key_in_config() -> Config {
        let mut config = Config::default();
        config.gemini.api_key = Some("config-key".to_string());
        config
    }

    #[test]
    fn resolve_explicit_local() {
        let config = Config::default();
        assert_eq!(
            resolve_backend_with(&config, Some(Backend::Local), &empty_env, &empty_env),
            Backend::Local
        );
    }

    #[test]
    fn resolve_explicit_gemini() {
        let config = Config::default();
        assert_eq!(
            resolve_backend_with(&config, Some(Backend::Gemini), &empty_env, &empty_env),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_explicit_gemini_overrides_no_api_key() {
        let config = Config::default();
        assert_eq!(
            resolve_backend_with(&config, Some(Backend::Gemini), &empty_env, &empty_env),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_prefer_local_with_base_url() {
        let config = test_config_prefer_local();
        assert_eq!(
            resolve_backend_with(&config, None, &empty_env, &empty_env),
            Backend::Local
        );
    }

    #[test]
    fn resolve_prefer_local_without_base_url_falls_to_gemini() {
        let mut config = Config::default();
        config.local_model.prefer_local = true;
        // base_url is empty
        assert_eq!(
            resolve_backend_with(&config, None, &empty_env, &empty_env),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_no_api_key_local_configured() {
        let config = test_config_local_configured();
        assert_eq!(
            resolve_backend_with(&config, None, &empty_env, &empty_env),
            Backend::Local
        );
    }

    #[test]
    fn resolve_api_key_present_via_env() {
        let config = test_config_local_configured();
        assert_eq!(
            resolve_backend_with(&config, None, &api_key_env, &empty_env),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_api_key_present_via_config() {
        let config = test_config_gemini_key_in_config();
        assert_eq!(
            resolve_backend_with(&config, None, &empty_env, &empty_env),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_api_key_present_via_dotenv() {
        let config = test_config_local_configured();
        let dotenv_with_key = |_name: &str| Some("dotenv-key".to_string());
        assert_eq!(
            resolve_backend_with(&config, None, &empty_env, &dotenv_with_key),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_no_api_key_no_local_falls_to_gemini() {
        let config = Config::default();
        assert_eq!(
            resolve_backend_with(&config, None, &empty_env, &empty_env),
            Backend::Gemini
        );
    }

    #[test]
    fn resolve_explicit_local_overrides_api_key() {
        let config = test_config_gemini_key_in_config();
        assert_eq!(
            resolve_backend_with(&config, Some(Backend::Local), &api_key_env, &empty_env),
            Backend::Local
        );
    }

    #[test]
    fn flash_lite_is_default_for_routine_asks() {
        let packet = ImpactPacket {
            risk_level: RiskLevel::Medium,
            ..ImpactPacket::default()
        };

        assert_eq!(
            select_gemini_model(&GeminiConfig::default(), GeminiMode::Analyze, &packet),
            DEFAULT_GEMINI_FAST_MODEL
        );
    }

    #[test]
    fn pro_is_default_for_patch_review_and_high_risk() {
        let high_risk_packet = ImpactPacket {
            risk_level: RiskLevel::High,
            ..ImpactPacket::default()
        };
        let medium_risk_packet = ImpactPacket {
            risk_level: RiskLevel::Medium,
            ..ImpactPacket::default()
        };

        assert_eq!(
            select_gemini_model(
                &GeminiConfig::default(),
                GeminiMode::Analyze,
                &high_risk_packet
            ),
            DEFAULT_GEMINI_DEEP_MODEL
        );
        assert_eq!(
            select_gemini_model(
                &GeminiConfig::default(),
                GeminiMode::ReviewPatch,
                &medium_risk_packet,
            ),
            DEFAULT_GEMINI_DEEP_MODEL
        );
    }

    #[test]
    fn explicit_model_overrides_routing() {
        let config = GeminiConfig {
            model: Some("gemini-custom".to_string()),
            fast_model: Some("fast-custom".to_string()),
            deep_model: Some("deep-custom".to_string()),
            ..Default::default()
        };

        assert_eq!(
            select_gemini_model(&config, GeminiMode::ReviewPatch, &ImpactPacket::default()),
            "gemini-custom"
        );
    }

    #[test]
    fn format_relevant_decisions_empty_returns_empty() {
        let result = format_relevant_decisions(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn format_relevant_decisions_produces_markdown() {
        let decisions = vec![
            RelevantDecision {
                file_path: std::path::PathBuf::from("docs/guide.md"),
                heading: Some("Introduction".to_string()),
                excerpt: "This guide explains the architecture.".to_string(),
                similarity: 0.85,
                rerank_score: None,
                staleness_days: None,
            },
            RelevantDecision {
                file_path: std::path::PathBuf::from("docs/api.md"),
                heading: Some("API Reference".to_string()),
                excerpt: "Endpoints for the service.".to_string(),
                similarity: 0.6,
                rerank_score: Some(0.92),
                staleness_days: None,
            },
        ];

        let result = format_relevant_decisions(&decisions);

        assert!(result.contains("## Relevant Architecture Documents"));
        assert!(result.contains("### Introduction (docs/guide.md)"));
        assert!(result.contains("This guide explains the architecture."));
        assert!(result.contains("### API Reference (docs/api.md)"));
        assert!(result.contains("Endpoints for the service."));
        assert!(result.contains("---"));
    }

    #[test]
    fn format_relevant_decisions_untitled_heading() {
        let decisions = vec![RelevantDecision {
            file_path: std::path::PathBuf::from("docs/readme.md"),
            heading: None,
            excerpt: "Some content".to_string(),
            similarity: 0.5,
            rerank_score: None,
            staleness_days: None,
        }];

        let result = format_relevant_decisions(&decisions);
        assert!(result.contains("### (untitled) (docs/readme.md)"));
    }

    #[test]
    fn format_observability_signals_empty_returns_empty() {
        let result = format_observability_signals(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn format_affected_contracts_empty_returns_empty() {
        let result = format_affected_contracts(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn format_affected_contracts_produces_markdown() {
        let contracts = vec![
            AffectedContract {
                endpoint_id: "api/openapi.json::GET::/pets".to_string(),
                path: "/pets".to_string(),
                method: "GET".to_string(),
                summary: "List all pets".to_string(),
                similarity: 0.85,
                spec_file: "api/openapi.json".to_string(),
            },
            AffectedContract {
                endpoint_id: "api/openapi.json::DELETE::/users/{id}".to_string(),
                path: "/users/{id}".to_string(),
                method: "DELETE".to_string(),
                summary: "Delete a user".to_string(),
                similarity: 0.71,
                spec_file: "api/openapi.json".to_string(),
            },
        ];

        let result = format_affected_contracts(&contracts);

        assert!(result.contains("## Affected API Contracts"));
        assert!(result.contains("GET /pets"));
        assert!(result.contains("similarity=0.85"));
        assert!(result.contains("DELETE /users/{id}"));
        assert!(result.contains("similarity=0.71"));
    }

    #[test]
    fn format_observability_signals_produces_summary() {
        use crate::observability::signal::{ObservabilitySignal, SignalSeverity};

        let signals = vec![
            ObservabilitySignal::new(
                "error_rate",
                "GET /api",
                0.15,
                SignalSeverity::Critical,
                "Error rate 15% for GET /api",
                "prometheus",
            ),
            ObservabilitySignal::new(
                "log_anomaly",
                "ERROR: Connection refused",
                5.0,
                SignalSeverity::Warning,
                "Repeated connection errors",
                "log_file",
            ),
        ];

        let result = format_observability_signals(&signals);
        assert!(result.contains("## Live System Signals"));
        assert!(result.contains("error_rate"));
        assert!(result.contains("GET /api"));
        assert!(result.contains("CRITICAL"));
        assert!(result.contains("log_anomaly"));
        assert!(result.contains("WARNING"));
    }
}
