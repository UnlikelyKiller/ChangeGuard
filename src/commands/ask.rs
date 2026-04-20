use crate::config::load::load_config;
use crate::gemini::modes::{GeminiMode, build_system_prompt, build_user_prompt};
use crate::gemini::run_query;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

pub fn execute_ask(query: String, mode: GeminiMode) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let latest_packet = storage.get_latest_packet()?.ok_or_else(|| {
        miette::miette!("No impact report found. Run 'changeguard impact' first.")
    })?;

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

    let system_prompt = build_system_prompt(mode);
    
    // For Narrative mode, we might want to augment the query with our structured summary
    let effective_query = if mode == GeminiMode::Narrative && query.to_lowercase() == "summary" {
        crate::gemini::narrative::NarrativeEngine::generate_risk_prompt(&latest_packet)
    } else {
        query
    };

    let user_prompt = build_user_prompt(mode, &latest_packet, &effective_query, diff.as_deref());

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

    // Read timeout from config
    let config = load_config(&layout)?;
    let timeout_secs = config.gemini.timeout_secs;

    run_query(&system_prompt, &sanitize_result.sanitized, timeout_secs)?;

    Ok(())
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
