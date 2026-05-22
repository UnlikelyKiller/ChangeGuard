use crate::commands::helpers::{get_layout, load_ledger_config};
use crate::impact::analysis::dead_code::ConfidenceScorer;
use crate::index::warn_if_stale;
use crate::output::diagnostics::success_marker;
use miette::Result;

pub fn execute_dead_code(threshold: f64, limit: usize, auto_index: bool) -> Result<()> {
    let layout = get_layout()?;
    let mut config = load_ledger_config(&layout);

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
    let repo_path = layout.root.as_std_path();

    let scorer = ConfidenceScorer::new(cozo, &storage, &config.dead_code, repo_path);
    let mut findings = scorer.scan_repo(limit)?;

    findings.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    findings.truncate(limit);

    crate::output::human::print_dead_code_summary(&findings, threshold);

    println!(
        "\n{} Scanned repository for dead code (threshold: {:.0}%, limit: {})",
        success_marker(),
        threshold * 100.0,
        limit
    );

    Ok(())
}
