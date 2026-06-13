use crate::config::load::load_config;
use crate::git::RepoSnapshot;
use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::output::diagnostics::success_marker;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use miette::Result;
use owo_colors::OwoColorize;
use std::env;

pub fn execute_impact_silent() -> Result<crate::impact::packet::ImpactPacket> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    // Filter changes against config ignore_patterns
    let config = load_config(&layout).unwrap_or_else(|_| crate::config::model::Config::default());
    let all_changes = get_repo_status(&repo)?;
    let changes = crate::git::ignore::filter_ignored_changes(
        all_changes,
        &config.watch.ignore_patterns,
        true,
    )?;

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };

    let mut packet = crate::impact::orchestrator::map_snapshot_to_packet(snapshot, &current_dir)?;

    // Load main config for temporal analysis
    let config = load_config(&layout).unwrap_or_default();

    // Persist to SQLite and run Orchestrated Enrichment
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = crate::state::storage::StorageManager::init(db_path.as_std_path())?;

    let orchestrator = crate::impact::orchestrator::ImpactOrchestrator::with_builtins();
    orchestrator.run(&mut packet, &storage, &config, &current_dir)?;

    // Post-processing: Finalize and Redact
    packet.finalize();
    crate::impact::redact::redact_secrets(&mut packet);

    // Save to ledger
    if let Err(e) = storage.save_packet(&packet) {
        tracing::warn!("SQLite save failed: {e}");
    }

    // Write report
    write_impact_report(&layout, &packet)?;

    storage.shutdown()?;

    Ok(packet)
}

pub fn execute_impact(
    all_parents: bool,
    summary: bool,
    _telemetry_coverage: bool,
    dead_code: bool,
    json: bool,
    out: Option<std::path::PathBuf>,
) -> Result<()> {
    let current_dir = env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;

    let repo = open_repo(&current_dir)?;
    let (head_hash, branch_name) = get_head_info(&repo)?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    // Filter changes against config ignore_patterns
    let mut config =
        load_config(&layout).unwrap_or_else(|_| crate::config::model::Config::default());
    let all_changes = get_repo_status(&repo)?;
    let changes = crate::git::ignore::filter_ignored_changes(
        all_changes,
        &config.watch.ignore_patterns,
        true,
    )?;

    let is_clean = changes.is_empty();

    let snapshot = RepoSnapshot {
        head_hash,
        branch_name,
        is_clean,
        changes,
    };

    let mut packet = crate::impact::orchestrator::map_snapshot_to_packet(snapshot, &current_dir)?;

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

    storage.shutdown()?;

    // Handle --json and --out: serialize to stdout or file
    if json || out.is_some() {
        let json_output = serde_json::to_string_pretty(&packet)
            .map_err(|e| miette::miette!("Failed to serialize impact report: {}", e))?;

        if let Some(path) = out {
            std::fs::write(&path, &json_output).map_err(|e| {
                miette::miette!(
                    "Failed to write impact report to '{}': {}",
                    path.display(),
                    e
                )
            })?;
            eprintln!(
                "Wrote impact report to {}",
                path.display().to_string().cyan()
            );
        } else {
            println!("{}", json_output);
        }
        return Ok(());
    }

    if packet.tree_clean && packet.changes.is_empty() {
        println!(
            "\n{} Working tree is clean — no staged or modified files detected.",
            success_marker()
        );
        println!("  Run 'git add <files>' before scanning for impact.");
        println!(
            "  {} Run {} to check for pending transactions.",
            "HINT:".yellow().bold(),
            "`changeguard ledger status`".cyan()
        );
        return Ok(());
    }

    if summary {
        crate::output::human::print_impact_brief(&packet);
    } else {
        crate::output::human::print_impact_summary(&packet);
    }

    println!(
        "\n{} Wrote impact report to {}",
        success_marker(),
        ".changeguard/reports/latest-impact.json".cyan()
    );

    Ok(())
}
