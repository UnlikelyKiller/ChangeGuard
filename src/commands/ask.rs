use crate::config::load::load_config;
use crate::gemini::prompt::{build_system_prompt, build_user_prompt};
use crate::gemini::run_query;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::Result;
use std::env;

pub fn execute_ask(query: String) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let latest_packet = storage.get_latest_packet()?.ok_or_else(|| {
        miette::miette!("No impact report found. Run 'changeguard impact' first.")
    })?;

    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(&latest_packet, &query);

    // Sanitize prompts to remove secrets before sending to Gemini
    let full_prompt = format!("{}\n\n{}", system_prompt, user_prompt);
    let sanitize_result = crate::gemini::sanitize::sanitize_for_gemini(&full_prompt);
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
    let config = load_config(&layout).unwrap_or_default();
    let timeout_secs = config.gemini.timeout_secs;

    run_query(
        "You are ChangeGuard, an expert software engineering assistant.",
        &sanitize_result.sanitized,
        timeout_secs,
    )?;

    Ok(())
}
