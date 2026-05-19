use crate::config::model::{Config, GeminiConfig};
use crate::gemini::modes::GeminiMode;
use crate::gemini::wrapper::run_query;
use crate::impact::packet::ImpactPacket;
use crate::local_model::pruner;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::env;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Local,
    Gemini,
}

pub fn execute_ask(query: Option<String>, narrative: bool, backend: Option<Backend>) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let config = crate::config::load_config(&layout)?;

    let storage_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(storage_path.as_std_path())?;

    let mut latest_packet = storage.get_latest_packet()?.ok_or_else(|| {
        miette::miette!("No impact report found. Run 'changeguard impact' first.")
    })?;

    // 1. Integrate external AI-Brains context
    if let Some(ref q) = query
        && let Ok(bridge_records) = crate::bridge::client::query_unified(q)
    {
        for record in bridge_records {
            if let crate::bridge::model::BridgePayload::Insight {
                memory_id,
                relevance,
                content,
            } = record.payload
            {
                latest_packet
                    .ai_insights
                    .push(crate::impact::packet::AiInsight {
                        memory_id,
                        relevance,
                        content,
                    });
            }
        }
    }

    let mut mode = GeminiMode::Analyze;
    if narrative {
        mode = GeminiMode::Narrative;
    }

    // For ReviewPatch, try to get the diff
    let diff = if mode == GeminiMode::ReviewPatch {
        get_working_tree_diff()
    } else {
        None
    };

    if mode == GeminiMode::ReviewPatch && diff.is_none() {
        println!(
            "{}",
            "Note: No diff available (working tree is clean). Falling back to general analysis."
                .yellow()
        );
        mode = GeminiMode::Analyze;
    }

    let query_string = query.unwrap_or_else(|| {
        "Explain the overall system architecture and recent changes.".to_string()
    });

    // 2. Build Unified Context Summary using Pruner (Zero Duplication)
    let pruned = pruner::prune_impact_packet(&latest_packet, mode);
    let context_summary = pruner::format_pruned_packet(&pruned);

    let mut user_prompt = format!("{}\n\nQuestion: {}", context_summary, query_string);
    if let Some(d) = diff {
        user_prompt = format!("{}\n\nPatch Diff:\n```diff\n{}\n```", user_prompt, d);
    }

    let system_prompt = get_system_prompt(mode);
    let resolved_backend = resolve_backend(&config, backend);

    match resolved_backend {
        Backend::Local => {
            let max_tokens = config.local_model.context_window;

            // 3. Query relevant chunks from indexed docs
            let relevant_chunks = pruner::query_relevant_chunks(
                &query_string,
                &config.local_model,
                storage.get_connection(),
                config.local_model.chunk_top_k,
                config.local_model.chunk_min_similarity,
                config.local_model.chunk_dedup_threshold,
            )
            .unwrap_or_else(|e| {
                tracing::warn!("Chunk retrieval failed: {e}, proceeding without chunks");
                Vec::new()
            });

            // 4. Assemble context with budget enforcement
            let messages = crate::local_model::context::assemble_context(
                &system_prompt,
                &user_prompt,
                &relevant_chunks,
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
                    eprintln!("{}", e.to_string().red());
                    Err(miette::miette!("Local model failed: {e}"))
                }
            }
        }
        Backend::Gemini => {
            let budget_tokens = config.gemini.context_window;
            let char_limit = (budget_tokens as f64 * 0.8 * 4.0) as usize;

            // The system prompt is static application text. Sanitize context.
            let sanitize_result = crate::gemini::sanitize::sanitize_for_gemini(&user_prompt);
            let mut final_user_prompt = sanitize_result.sanitized;

            if final_user_prompt.len() > char_limit {
                tracing::warn!("Prompt exceeds Gemini budget, truncating...");
                final_user_prompt.truncate(char_limit);
                final_user_prompt.push_str("\n\n[Prompt truncated for Gemini budget]");
            }

            let timeout_secs = config.gemini.timeout_secs;
            let model = select_gemini_model(&config.gemini, mode, &latest_packet);

            run_query(
                &system_prompt,
                &final_user_prompt,
                timeout_secs,
                model,
                config.gemini.api_key.as_deref(),
            )?;

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

pub fn resolve_backend(config: &Config, explicit: Option<Backend>) -> Backend {
    resolve_backend_with(config, explicit, &|name| env::var(name).ok(), &|name| {
        crate::config::model::read_env_key(name)
    })
}

pub fn resolve_backend_with(
    config: &Config,
    explicit: Option<Backend>,
    env_reader: &dyn Fn(&str) -> Option<String>,
    dotenv_reader: &dyn Fn(&str) -> Option<String>,
) -> Backend {
    if let Some(b) = explicit {
        return b;
    }
    if config.local_model.prefer_local && !config.local_model.base_url.is_empty() {
        return Backend::Local;
    }

    let has_gemini_key = config.gemini.api_key.is_some()
        || env_reader("GEMINI_API_KEY").is_some()
        || dotenv_reader("GEMINI_API_KEY").is_some();

    if !has_gemini_key && !config.local_model.base_url.is_empty() {
        return Backend::Local;
    }
    Backend::Gemini
}

fn get_system_prompt(mode: GeminiMode) -> String {
    match mode {
        GeminiMode::ReviewPatch => "You are an expert code reviewer. Review the provided diff and impact summary. Identify risks, bugs, and architectural deviations.".to_string(),
        GeminiMode::Analyze => "You are a senior software architect. Analyze the provided impact summary and answer questions about the system design and changes.".to_string(),
        GeminiMode::Narrative => "You are a technical writer. Explain the changes and their impact in a clear, cohesive narrative suitable for a changelog or team update.".to_string(),
        GeminiMode::Suggest => "You are a lead engineer. Suggest improvements, refactors, or additional tests based on the provided impact summary.".to_string(),
    }
}

pub(crate) fn select_gemini_model<'a>(
    config: &'a GeminiConfig,
    mode: GeminiMode,
    _packet: &ImpactPacket,
) -> &'a str {
    if let Some(model) = config.model.as_deref()
        && !model.is_empty()
    {
        return model;
    }

    match mode {
        GeminiMode::ReviewPatch => "gemini-1.5-pro",
        GeminiMode::Analyze => "gemini-1.5-flash",
        GeminiMode::Narrative => "gemini-1.5-flash",
        GeminiMode::Suggest => "gemini-1.5-pro",
    }
}
