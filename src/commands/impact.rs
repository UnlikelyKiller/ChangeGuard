use crate::git::repo::{get_head_info, open_repo};
use crate::git::status::get_repo_status;
use crate::git::{ChangeType, RepoSnapshot};
use crate::impact::packet::{ChangedFile, ImpactPacket, RiskLevel};
use crate::index::languages::parse_symbols;
use crate::state::layout::Layout;
use crate::state::reports::write_impact_report;
use crate::ui::{print_header, success_marker, warning_marker};
use comfy_table::Table;
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

fn print_impact_summary(packet: &ImpactPacket) {
    print_header("ChangeGuard Impact Analysis");

    let risk_color = match packet.risk_level {
        RiskLevel::Low => "LOW".green().bold().to_string(),
        RiskLevel::Medium => "MEDIUM".yellow().bold().to_string(),
        RiskLevel::High => "HIGH".red().bold().to_string(),
    };

    println!("{:<15} {}", "Risk Level:".bold().cyan(), risk_color);

    if !packet.risk_reasons.is_empty() {
        println!("\n{}", "Risk Reasons:".bold());
        let mut table = Table::new();
        table.set_header(vec!["#", "Reason"]);
        for (i, reason) in packet.risk_reasons.iter().enumerate() {
            table.add_row(vec![(i + 1).to_string(), reason.to_string()]);
        }
        println!("{table}");
    }
}

fn map_snapshot_to_packet(snapshot: RepoSnapshot, base_dir: &Path) -> Result<ImpactPacket> {
    let mut packet = ImpactPacket::default();
    packet.head_hash = snapshot.head_hash;
    packet.branch_name = snapshot.branch_name;

    let pb = ProgressBar::new(snapshot.changes.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar()));
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

            pb.inc(1);
            ChangedFile {
                path: c.path,
                status,
                is_staged: c.is_staged,
                symbols,
            }
        })
        .collect();

    pb.finish_with_message("Symbol extraction complete.");
    Ok(packet)
}
