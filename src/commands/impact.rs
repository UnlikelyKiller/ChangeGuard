use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::packet::{ChangedFile, ImpactPacket};
use crate::index::languages::parse_symbols;
use crate::index::references::extract_import_export;
use crate::index::runtime_usage::extract_runtime_usage;
use crate::output::diagnostics::{success_marker, warning_marker};
use crate::output::human::print_impact_summary;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use crate::util::clock::SystemClock;
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;
use owo_colors::OwoColorize;
use std::env;
use std::fs;
use std::path::Path;

pub fn execute_impact() -> Result<()> {
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

    // Load rules and perform risk analysis
    match crate::policy::load::load_rules(&layout) {
        Ok(rules) => {
            if let Err(e) = crate::impact::analysis::analyze_risk(&mut packet, &rules) {
                tracing::warn!("Risk analysis failed: {e}");
                println!(
                    "{} Risk analysis failed. Impact report written without risk scoring.",
                    warning_marker()
                );
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load rules: {e}");
            println!(
                "{} Could not load rules. Impact report written without risk scoring.",
                warning_marker()
            );
        }
    }

    packet.finalize();

    // Redact secrets before writing to disk
    let redactions = crate::impact::redact::redact_secrets(&mut packet);
    if !redactions.is_empty() {
        tracing::info!("Redacted {} secret(s) from impact packet", redactions.len());
    }

    write_impact_report(&layout, &packet)?;

    print_impact_summary(&packet);

    println!(
        "\n{} Wrote impact report to {}",
        success_marker(),
        ".changeguard/reports/latest-impact.json".cyan()
    );

    // Persist to SQLite
    let db_path = layout.state_subdir().join("ledger.db");
    match crate::state::storage::StorageManager::init(db_path.as_std_path()) {
        Ok(storage) => {
            if let Err(e) = storage.save_packet(&packet) {
                tracing::warn!("SQLite save failed: {e}");
                println!(
                    "{} Impact report saved to disk but SQLite ledger was not updated. The 'ask' command may not find this report.",
                    warning_marker()
                );
            }
        }
        Err(e) => {
            tracing::warn!("SQLite init failed: {e}");
            println!(
                "{} Could not initialize SQLite. Impact report saved to disk but not persisted to database.",
                warning_marker()
            );
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
            let status = match c.change_type {
                ChangeType::Added => "Added".to_string(),
                ChangeType::Modified => "Modified".to_string(),
                ChangeType::Deleted => "Deleted".to_string(),
                ChangeType::Renamed { .. } => "Renamed".to_string(),
            };

            let symbols = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified) {
                let full_path = base_dir.join(&c.path);
                if let Ok(content) = fs::read_to_string(&full_path) {
                    parse_symbols(&c.path, &content).ok().flatten()
                } else {
                    None
                }
            } else {
                None
            };
            let imports = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified) {
                let full_path = base_dir.join(&c.path);
                fs::read_to_string(&full_path)
                    .ok()
                    .and_then(|content| extract_import_export(&c.path, &content).ok().flatten())
            } else {
                None
            };
            let runtime_usage = if matches!(c.change_type, ChangeType::Added | ChangeType::Modified)
            {
                let full_path = base_dir.join(&c.path);
                fs::read_to_string(&full_path)
                    .ok()
                    .and_then(|content| extract_runtime_usage(&c.path, &content))
            } else {
                None
            };

            pb.inc(1);
            ChangedFile {
                path: c.path,
                status,
                is_staged: c.is_staged,
                symbols,
                imports,
                runtime_usage,
            }
        })
        .collect();

    pb.finish_with_message("Symbol extraction complete.");
    Ok(packet)
}
