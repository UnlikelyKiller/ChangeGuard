use miette::Result;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use crate::gemini::prompt::{build_system_prompt, build_user_prompt};
use crate::gemini::run_query;
use std::env;

pub fn execute_ask(query: String) -> Result<()> {
    let current_dir = env::current_dir().map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;
    
    let latest_packet = storage.get_latest_packet()?.ok_or_else(|| {
        miette::miette!("No impact report found. Run 'changeguard impact' first.")
    })?;

    let system_prompt = build_system_prompt();
    let user_prompt = build_user_prompt(&latest_packet, &query);

    run_query(&system_prompt, &user_prompt)?;

    Ok(())
}
