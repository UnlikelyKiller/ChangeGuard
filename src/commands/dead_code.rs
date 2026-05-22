use crate::config::load::load_config;
use crate::config::model::Config;
use crate::impact::analysis::dead_code::ConfidenceScorer;
use crate::index::warn_if_stale;
use crate::output::diagnostics::success_marker;
use crate::state::layout::Layout;
use miette::Result;
use std::env;

pub fn execute_dead_code(threshold: f64, limit: usize, auto_index: bool) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let mut config = load_config(&layout).unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {e}. Using defaults.");
        Config::default()
    });

    // --- Staleness check ---
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = crate::state::storage::StorageManager::init(db_path.as_std_path())?;
    let threshold_days = config.index.stale_threshold_days;

    let storage = if auto_index {
        crate::index::staleness::try_auto_index(storage, threshold_days)?
    } else {
        let _ = warn_if_stale(&storage, threshold_days);
        storage
    };

    // CLI overrides
    config.dead_code.enabled = true;
    config.dead_code.confidence_threshold = threshold;

    let cozo = storage.cozo.as_ref();
    let scorer = ConfidenceScorer::new(cozo, &storage, &config.dead_code, &current_dir);

    let findings = scorer.scan_repo(limit)?;

    crate::output::human::print_dead_code_summary(&findings, threshold);

    println!(
        "\n{} Scanned repository for dead code (threshold: {:.0}%, limit: {})",
        success_marker(),
        threshold * 100.0,
        limit
    );

    Ok(())
}
