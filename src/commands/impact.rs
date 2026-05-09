use crate::git::RepoSnapshot;
use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::output::diagnostics::{success_marker, warning_marker};
use crate::output::human::print_impact_summary;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

pub fn execute_impact(all_parents: bool, summary: bool, _telemetry_coverage: bool) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let changes = get_repo_status(&repo)?;

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };

    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let mut packet = crate::impact::orchestrator::map_snapshot_to_packet(snapshot, &current_dir)?;

    // Load main config for temporal analysis
    let mut config = crate::config::load::load_config(&layout).unwrap_or_else(|e| {
        tracing::warn!("Failed to load config: {e}. Using defaults.");
        println!(
            "{} Could not load config. Using default temporal analysis settings.",
            warning_marker()
        );
        crate::config::model::Config::default()
    });

    // CLI override
    if all_parents {
        config.temporal.all_parents = true;
    }

    // Persist to SQLite and run Orchestrated Enrichment
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = crate::state::storage::StorageManager::init(db_path.as_std_path())?;

    let orchestrator = crate::impact::orchestrator::ImpactOrchestrator::with_builtins();
    orchestrator.run(&mut packet, &storage, &config, &current_dir)?;

    // Post-processing: Finalize and Redact
    packet.finalize();
    let redactions = crate::impact::redact::redact_secrets(&mut packet);
    if !redactions.is_empty() {
        tracing::info!("Redacted {} secret(s) from impact packet", redactions.len());
    }

    // Save to ledger
    if let Err(e) = storage.save_packet(&packet) {
        tracing::warn!("SQLite save failed: {e}");
    }

    // Write report
    write_impact_report(&layout, &packet)?;

    if summary {
        crate::output::human::print_impact_brief(&packet);
    } else {
        print_impact_summary(&packet, &config);
    }

    println!(
        "\n{} Wrote impact report to {}",
        success_marker(),
        ".changeguard/reports/latest-impact.json".cyan()
    );

    Ok(())
}
