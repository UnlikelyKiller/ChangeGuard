use crate::config::load::load_config;
use crate::gemini::modes::{GeminiMode, build_system_prompt, build_user_prompt};
use crate::gemini::run_query;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

pub fn execute_ask(query: Option<String>, mut mode: GeminiMode, narrative: bool) -> Result<()> {
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

    // Token budgeting (4 chars ~ 1 token).
    // Track 34 requirement: hard limit of 409,600 characters (~102,400 tokens).
    let char_limit = 409_600;

    let truncated = latest_packet.truncate_for_context(char_limit);

    let system_prompt = build_system_prompt(mode);

    let mut user_prompt = if mode == GeminiMode::Narrative {
        crate::gemini::narrative::NarrativeEngine::generate_risk_prompt(&latest_packet)
    } else {
        let effective_query = query.unwrap_or_default();
        build_user_prompt(mode, &latest_packet, &effective_query, diff.as_deref())
    };

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

    if let Err(e) = run_query(&system_prompt, &sanitize_result.sanitized, timeout_secs) {
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
