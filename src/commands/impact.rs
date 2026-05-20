use crate::config::load::load_config;
use crate::git::RepoSnapshot;
use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::output::diagnostics::{success_marker, warning_marker};
use crate::output::human::print_impact_summary;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use globset::{Glob, GlobSetBuilder};
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

pub fn execute_impact(
    all_parents: bool,
    summary: bool,
    _telemetry_coverage: bool,
    dead_code: bool,
) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    // Filter changes against config ignore_patterns
    let all_changes = get_repo_status(&repo)?;
    let changes = filter_changes(&layout, all_changes)?;

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };

    let mut packet = crate::impact::orchestrator::map_snapshot_to_packet(snapshot, &current_dir)?;

    // Load main config for temporal analysis
    let mut config = load_config(&layout).unwrap_or_else(|e| {
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
    if dead_code {
        config.dead_code.enabled = true;
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

/// Filter changes against config `watch.ignore_patterns` using glob matching.
fn filter_changes(
    layout: &Layout,
    changes: Vec<crate::git::FileChange>,
) -> Result<Vec<crate::git::FileChange>> {
    let config = match load_config(layout) {
        Ok(c) => c,
        Err(_) => return Ok(changes),
    };
    let mut builder = GlobSetBuilder::new();
    for pattern in &config.watch.ignore_patterns {
        builder.add(
            Glob::new(pattern)
                .map_err(|e| miette::miette!("Invalid glob pattern '{}': {}", pattern, e))?,
        );
    }
    let ignore_set = builder
        .build()
        .map_err(|e| miette::miette!("Failed to build glob set: {}", e))?;
    Ok(changes
        .into_iter()
        .filter(|change| {
            let path_str = change.path.to_string_lossy();
            !ignore_set.is_match(path_str.as_ref())
        })
        .collect())
}
