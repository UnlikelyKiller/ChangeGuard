use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::packet::{ChangedFile, FileAnalysisStatus, ImpactPacket};
use crate::output::diagnostics::{success_marker, warning_marker};
use crate::output::human::print_impact_summary;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use crate::util::clock::SystemClock;
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;
use owo_colors::OwoColorize;
use std::env;
use std::path::Path;

use crate::index::analysis::{AnalysisOutcome, analyze_file as analyze_changed_file};

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
    let mut packet = map_snapshot_to_packet(snapshot, &current_dir)?;

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

    // NEW ORCHESTRATED FLOW
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

pub(crate) fn refresh_federated_dependencies(
    current_dir: &Path,
    packet: &ImpactPacket,
    storage: &crate::state::storage::StorageManager,
) -> Result<()> {
    let utf8_current_dir = camino::Utf8PathBuf::from_path_buf(current_dir.to_path_buf())
        .map_err(|_| miette::miette!("Invalid UTF-8 path in current directory"))?;
    let scanner = crate::federated::scanner::FederatedScanner::new(utf8_current_dir);
    let (siblings, warnings) = scanner.scan_siblings()?;

    for warning in warnings {
        tracing::warn!("Federated discovery warning: {warning}");
    }

    let timestamp = chrono::Utc::now().to_rfc3339();
    for (path, schema) in siblings {
        crate::federated::storage::update_federated_link(
            storage.get_connection(),
            &schema.repo_name,
            path.as_str(),
            &timestamp,
        )?;
        crate::federated::storage::clear_federated_dependencies(
            storage.get_connection(),
            &schema.repo_name,
        )?;
        for (local_symbol, sibling_symbol) in
            scanner.discover_dependencies(packet, &schema.repo_name, &schema)?
        {
            crate::federated::storage::save_federated_dependencies(
                storage.get_connection(),
                &schema.repo_name,
                &local_symbol,
                &sibling_symbol,
            )?;
        }
    }

    Ok(())
}

fn map_snapshot_to_packet(snapshot: RepoSnapshot, base_dir: &Path) -> Result<ImpactPacket> {
    let mut packet = ImpactPacket {
        head_hash: snapshot.head_hash,
        branch_name: snapshot.branch_name,
        ..ImpactPacket::with_clock(&SystemClock)
    };

    let pb = ProgressBar::new(snapshot.changes.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );
    pb.set_message("Extracting symbols...");

    packet.changes = snapshot
        .changes
        .into_iter()
        .map(|c| {
            pb.set_message(format!("Extracting symbols from {}", c.path.display()));
            let (status, old_path) = match c.change_type {
                ChangeType::Added => ("Added".to_string(), None),
                ChangeType::Modified => ("Modified".to_string(), None),
                ChangeType::Deleted => ("Deleted".to_string(), None),
                ChangeType::Renamed { ref old_path } => {
                    ("Renamed".to_string(), Some(old_path.clone()))
                }
            };

            let outcome = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified) {
                analyze_changed_file(&c.path, base_dir)
            } else {
                AnalysisOutcome {
                    symbols: None,
                    imports: None,
                    runtime_usage: None,
                    analysis_status: FileAnalysisStatus::default(),
                    analysis_warnings: Vec::new(),
                }
            };

            pb.inc(1);
            ChangedFile {
                path: c.path,
                status,
                old_path,
                is_staged: c.is_staged,
                symbols: outcome.symbols,
                imports: outcome.imports,
                runtime_usage: outcome.runtime_usage,
                analysis_status: outcome.analysis_status,
                analysis_warnings: outcome.analysis_warnings,
                api_routes: Vec::new(),
                data_models: Vec::new(),
                ci_gates: Vec::new(),
            }
        })
        .collect();

    pb.finish_with_message("Symbol extraction complete.");
    Ok(packet)
}

// analyze_changed_file moved to crate::index::analysis::analyze_file
