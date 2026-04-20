use crate::git::repo::open_repo;
use crate::impact::temporal::GixHistoryProvider;
use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use std::env;

pub fn execute_hotspots(
    limit: usize,
    commits: usize,
    json: bool,
    dir: Option<String>,
    lang: Option<String>,
    all_parents: bool,
) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let repo = open_repo(&current_dir)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    if !json {
        println!("Analyzing {} commits for temporal hotspots...", commits);
    }

    let history_provider = GixHistoryProvider::new(&repo);
    let hotspots = crate::impact::hotspots::calculate_hotspots(
        &storage,
        &history_provider,
        commits,
        limit,
        all_parents,
        dir.as_deref(),
        lang.as_deref(),
    )?;

    if json {
        println!("{}", serde_json::to_string_pretty(&hotspots).into_diagnostic()?);
    } else {
        crate::output::human::print_hotspots_table(&hotspots);
    }

    Ok(())
}
